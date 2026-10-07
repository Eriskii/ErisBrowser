//! Integer-only proposal implementation. All arithmetic is u32 or a pair of
//! u32 words. Unit opacity is a no-layer scope, never a call to channel48.

pub const ZERO: u32 = 0;
pub const TINY: u32 = 1;
pub const GENERAL: u32 = 2;
pub const UNIT: u32 = 3;
pub const INVALID: u32 = 4;
pub const TINY_BITS: u32 = 0x3300_0000;
pub const ONE_BITS: u32 = 0x3f80_0000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wide {
    pub hi: u32,
    pub lo: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rounded {
    pub significand: u32,
    pub shift: u32,
}

pub fn classify(raw: u32) -> u32 {
    if raw & 0x7fff_ffff == 0 {
        return ZERO;
    }
    if raw > ONE_BITS {
        return INVALID;
    }
    if raw == ONE_BITS {
        return UNIT;
    }
    if raw <= TINY_BITS { TINY } else { GENERAL }
}

// Add modulo 2^64. Valid channel operands have a proved non-overflowing sum.
pub fn add(a: Wide, b: Wide) -> Wide {
    let lo = a.lo.wrapping_add(b.lo);
    Wide {
        hi: a.hi.wrapping_add(b.hi).wrapping_add(u32::from(lo < a.lo)),
        lo,
    }
}

pub fn left(x: Wide, shift: u32) -> Wide {
    if shift == 0 {
        return x;
    }
    if shift < 32 {
        return Wide {
            hi: (x.hi << shift) | (x.lo >> (32 - shift)),
            lo: x.lo << shift,
        };
    }
    if shift == 32 {
        return Wide { hi: x.lo, lo: 0 };
    }
    if shift < 64 {
        return Wide {
            hi: x.lo << (shift - 32),
            lo: 0,
        };
    }
    Wide { hi: 0, lo: 0 }
}

pub fn right(x: Wide, shift: u32) -> Wide {
    if shift == 0 {
        return x;
    }
    if shift < 32 {
        return Wide {
            hi: x.hi >> shift,
            lo: (x.lo >> shift) | (x.hi << (32 - shift)),
        };
    }
    if shift == 32 {
        return Wide { hi: 0, lo: x.hi };
    }
    if shift < 64 {
        return Wide {
            hi: 0,
            lo: x.hi >> (shift - 32),
        };
    }
    Wide { hi: 0, lo: 0 }
}

pub fn bit_length(x: Wide) -> u32 {
    if x.hi != 0 {
        64 - x.hi.leading_zeros()
    } else {
        32 - x.lo.leading_zeros()
    }
}

fn bit(x: Wide, position: u32) -> bool {
    if position < 32 {
        ((x.lo >> position) & 1) != 0
    } else {
        ((x.hi >> (position - 32)) & 1) != 0
    }
}

fn lower_nonzero(x: Wide, count: u32) -> bool {
    if count == 0 {
        return false;
    }
    if count < 32 {
        return x.lo & ((1 << count) - 1) != 0;
    }
    if count == 32 {
        return x.lo != 0;
    }
    if count < 64 {
        return x.lo != 0 || x.hi & ((1 << (count - 32)) - 1) != 0;
    }
    x.lo != 0 || x.hi != 0
}

// This representation also preserves the possible 2^64 rounded result of an
// arbitrary u64 input. That result is outside the channel domain and is never
// restored to a pair by channel48. The verifier checks the carry separately.
pub fn round24_parts(x: Wide) -> Rounded {
    let bits = bit_length(x);
    if bits <= 24 {
        return Rounded {
            significand: x.lo,
            shift: 0,
        };
    }
    let shift = bits - 24; // 1..=40
    let q = right(x, shift).lo;
    let guard = bit(x, shift - 1);
    let sticky = lower_nonzero(x, shift - 1);
    Rounded {
        significand: q + u32::from(guard && (sticky || q & 1 != 0)),
        shift,
    }
}

// Precondition: the rounded result fits 64 bits. Proved for every channel use.
pub fn round24(x: Wide) -> Wide {
    let r = round24_parts(x);
    left(
        Wide {
            hi: 0,
            lo: r.significand,
        },
        r.shift,
    )
}

pub fn round_even(x: u32, shift: u32) -> u32 {
    if shift == 0 {
        return x;
    }
    if shift > 32 {
        return 0;
    }
    if shift == 32 {
        return u32::from(x > 0x8000_0000);
    }
    let q = x >> shift;
    let r = x & ((1 << shift) - 1);
    let half = 1 << (shift - 1);
    q + u32::from(r > half || (r == half && q & 1 != 0))
}

fn nearest_odd(x: u32, divisor: u32) -> u32 {
    x / divisor + u32::from(x % divisor > divisor / 2)
}

// a <= 65535, b <= 2^24 (including the identity inverse).
pub fn product16(a: u32, b: u32) -> Wide {
    let low = a * (b & 65535);
    let upper = a * (b >> 16);
    let lo = low.wrapping_add(upper << 16);
    Wide {
        hi: (upper >> 16) + u32::from(lo < low),
        lo,
    }
}

// Both operands have at most 24 bits. The exact product has at most 48 bits.
pub fn product24(a: u32, b: u32) -> Wide {
    add(product16(a & 65535, b), left(product16(a >> 16, b), 16))
}

pub fn inverse(alpha: u32, raw: u32) -> u32 {
    // Caller accepts only ZERO/TINY/GENERAL; UNIT preserves no-layer semantics.
    if alpha == 0 || classify(raw) <= TINY {
        return 1 << 24;
    }
    let m = (raw & 0x7f_ffff) | (1 << 23);
    let r = 150 - ((raw >> 23) & 255); // 24..48
    if alpha == 65535 {
        return (1 << 24) - round_even(m, r - 24);
    }
    let h = 31 - alpha.leading_zeros();
    let x = alpha << (23 - h);
    let q = x + nearest_odd(x, 65535);
    let product = round24_parts(product24(q, m));
    let denominator = 39 + r - h - product.shift;
    (1 << 24) - round_even(product.significand, denominator - 24)
}

// GENERAL raw bits only. Inputs are premultiplied u16 channels. The exact
// source and rounded destination sum fits64 before and after rounding.
pub fn channel48(source: u32, destination: u32, inverse: u32, raw: u32) -> Wide {
    let m = (raw & 0x7f_ffff) | (1 << 23);
    let r = 150 - ((raw >> 23) & 255);
    let source_pair = left(round24(product16(source, m)), 48 - r);
    let destination_pair = left(round24(product16(destination, inverse)), 24);
    round24(add(source_pair, destination_pair))
}

fn significand(x: Wide, bits: u32) -> u32 {
    if bits <= 24 {
        x.lo << (24 - bits)
    } else {
        right(x, bits - 24).lo
    }
}

// x is zero or an exactly represented binary32 on the given fixed scale.
pub fn fixed_bits(x: Wide, scale: u32) -> u32 {
    let bits = bit_length(x);
    if bits == 0 {
        return 0;
    }
    ((bits + 126 - scale) << 23) | (significand(x, bits) & 0x7f_ffff)
}

pub fn parent48(x: Wide) -> u32 {
    (x.hi >> 16) + u32::from((x.hi & 65535) >= 32768)
}

pub fn root_bits48(x: Wide) -> u32 {
    let bits = bit_length(x);
    if bits == 0 {
        return 0;
    }
    let m = significand(x, bits);
    let (scaled, shift) = if m >= 8_421_376 { (m, 8) } else { (2 * m, 9) };
    let rounded = scaled - nearest_odd(scaled, 257);
    ((bits + 78 - shift) << 23) | (rounded & 0x7f_ffff)
}

pub fn root48(x: Wide) -> u32 {
    let bits = root_bits48(x);
    if bits == 0 {
        return 0;
    }
    let shift = 150 - ((bits >> 23) & 255);
    if shift > 24 {
        return 0;
    }
    let m = (bits & 0x7f_ffff) | (1 << 23);
    (m + (1 << (shift - 1))) >> shift
}
