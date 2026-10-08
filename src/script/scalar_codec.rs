//! Shared scalar encodings for non-shared binary-data storage.
//! Runtime callers retain responsibility for conversion and byte-access charges.
use super::to_i32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Codec {
    Int8,
    Uint8,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Float16,
    Float32,
    Float64,
}

impl Codec {
    #[cfg(test)]
    pub(super) const NAMES: [&'static str; 9] = [
        "Int8", "Uint8", "Int16", "Uint16", "Int32", "Uint32", "Float16", "Float32", "Float64",
    ];

    pub(super) fn named(name: &str) -> Option<Self> {
        Some(match name {
            "Int8" => Self::Int8,
            "Uint8" => Self::Uint8,
            "Int16" => Self::Int16,
            "Uint16" => Self::Uint16,
            "Int32" => Self::Int32,
            "Uint32" => Self::Uint32,
            "Float16" => Self::Float16,
            "Float32" => Self::Float32,
            "Float64" => Self::Float64,
            _ => return None,
        })
    }

    pub(super) fn width(self) -> usize {
        match self {
            Self::Int8 | Self::Uint8 => 1,
            Self::Int16 | Self::Uint16 | Self::Float16 => 2,
            Self::Int32 | Self::Uint32 | Self::Float32 => 4,
            Self::Float64 => 8,
        }
    }

    pub(super) fn work(self) -> usize {
        // Fixed scalar field/rounding operations plus at most eight scratch
        // byte writes and reversals. No backing-sized traversal or allocation.
        48 + 2 * self.width()
    }

    pub(super) fn encode(self, value: f64, little: bool) -> [u8; 8] {
        let word = match self {
            Self::Int8 | Self::Uint8 => u64::from(to_i32(value) as u8),
            Self::Int16 | Self::Uint16 => u64::from(to_i32(value) as u16),
            Self::Int32 | Self::Uint32 => u64::from(to_i32(value) as u32),
            Self::Float16 => u64::from(encode_f16(value)),
            Self::Float32 => u64::from(if value.is_nan() {
                0x7fc0_0000
            } else {
                (value as f32).to_bits()
            }),
            Self::Float64 => {
                if value.is_nan() {
                    0x7ff8_0000_0000_0000
                } else {
                    value.to_bits()
                }
            }
        };
        let width = self.width();
        let mut bytes = [0; 8];
        bytes[..width].copy_from_slice(&word.to_be_bytes()[8 - width..]);
        if little {
            bytes[..width].reverse();
        }
        bytes
    }

    pub(super) fn decode(self, mut bytes: [u8; 8], little: bool) -> f64 {
        let width = self.width();
        if little {
            bytes[..width].reverse();
        }
        let mut word = 0u64;
        for byte in &bytes[..width] {
            word = (word << 8) | u64::from(*byte);
        }
        let value = match self {
            Self::Int8 => (word as i8) as f64,
            Self::Uint8 => word as f64,
            Self::Int16 => (word as i16) as f64,
            Self::Uint16 => word as f64,
            Self::Int32 => (word as i32) as f64,
            Self::Uint32 => word as f64,
            Self::Float16 => decode_f16(word as u16),
            Self::Float32 => f32::from_bits(word as u32) as f64,
            Self::Float64 => f64::from_bits(word),
        };
        if value.is_nan() {
            f64::from_bits(0x7ff8_0000_0000_0000)
        } else {
            value
        }
    }
}

fn rounded_significand(significand: u64, shift: u32) -> u64 {
    debug_assert!((42..=53).contains(&shift));
    let retained = significand >> shift;
    let remainder = significand & ((1u64 << shift) - 1);
    let midpoint = 1u64 << (shift - 1);
    retained + u64::from(remainder > midpoint || (remainder == midpoint && retained & 1 != 0))
}

pub(super) fn encode_f16(value: f64) -> u16 {
    let bits = value.to_bits();
    let sign = ((bits >> 48) & 0x8000) as u16;
    let exponent = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & 0x000f_ffff_ffff_ffff;
    if exponent == 0x7ff {
        return if fraction == 0 { sign | 0x7c00 } else { 0x7e00 };
    }
    if exponent == 0 {
        return sign;
    }
    let mut exponent = exponent - 1023;
    if exponent > 15 {
        return sign | 0x7c00;
    }
    if exponent < -25 {
        return sign;
    }
    let significand = (1u64 << 52) | fraction;
    if exponent < -14 {
        // Subnormal units are 2^-24. Carry to 1024 is the smallest normal.
        return sign | rounded_significand(significand, (28 - exponent) as u32) as u16;
    }
    let mut retained = rounded_significand(significand, 42);
    if retained == 2048 {
        retained = 1024;
        exponent += 1;
    }
    if exponent > 15 {
        return sign | 0x7c00;
    }
    sign | (((exponent + 15) as u16) << 10) | (retained - 1024) as u16
}

pub(super) fn decode_f16(bits: u16) -> f64 {
    let sign = u64::from(bits & 0x8000) << 48;
    let exponent = (bits >> 10) & 31;
    let fraction = u64::from(bits & 1023);
    let result = if exponent == 31 {
        if fraction == 0 {
            sign | 0x7ff0_0000_0000_0000
        } else {
            0x7ff8_0000_0000_0000
        }
    } else if exponent != 0 {
        sign | (u64::from(exponent + 1008) << 52) | (fraction << 42)
    } else if fraction == 0 {
        sign
    } else {
        let highest = 63 - fraction.leading_zeros();
        sign | (u64::from(highest + 999) << 52) | ((fraction - (1u64 << highest)) << (52 - highest))
    };
    f64::from_bits(result)
}
