//! Portable integer arithmetic only: u32 scalars and two-word unsigned values.
//! Inputs are premultiplied u16 channels and k in 1..=255. No float or u64.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wide {
    pub hi: u32,
    pub lo: u32,
}

pub fn add(a: Wide, b: Wide) -> Wide {
    let lo = a.lo.wrapping_add(b.lo);
    Wide {
        hi: a.hi + b.hi + u32::from(lo < a.lo),
        lo,
    }
}

/// a <= 65535; b <= 2^24 (the upper endpoint is the identity inverse).
pub fn product(a: u32, b: u32) -> Wide {
    let low = a * (b & 65535);
    let upper = a * (b >> 16);
    let lo = low.wrapping_add(upper << 16);
    Wide {
        hi: (upper >> 16) + u32::from(lo < low),
        lo,
    }
}

pub fn bit_length(x: Wide) -> u32 {
    if x.hi != 0 {
        64 - x.hi.leading_zeros()
    } else {
        32 - x.lo.leading_zeros()
    }
}

/// Exact nearest-integer, ties-even; shift is 0..=23.
pub fn round_even(x: u32, shift: u32) -> u32 {
    if shift == 0 {
        return x;
    }
    let q = x >> shift;
    let r = x & ((1 << shift) - 1);
    let half = 1 << (shift - 1);
    q + u32::from(r > half || (r == half && q & 1 != 0))
}

fn round_odd_divisor(x: u32, divisor: u32) -> u32 {
    x / divisor + u32::from(x % divisor > divisor / 2)
}

/// Keep the scale, rounding an integer with <=41 input bits to 24 significant
/// bits. A carry into a new top bit is retained in the returned pair.
pub fn round24(x: Wide) -> Wide {
    let bits = bit_length(x);
    if bits <= 24 {
        return x;
    }
    let shift = bits - 24; // 1..=17 under the input bound.
    let q = (x.hi << (32 - shift)) | (x.lo >> shift);
    let r = x.lo & ((1 << shift) - 1);
    let half = 1 << (shift - 1);
    let rounded = q + u32::from(r > half || (r == half && q & 1 != 0));
    Wide {
        hi: rounded >> (32 - shift),
        lo: rounded << shift,
    }
}

/// Returns exactly RN32(1 - RN32(RN32(A/65535) * k/256)) * 2^24.
pub fn inverse(alpha: u32, k: u32) -> u32 {
    if alpha == 0 {
        return 1 << 24;
    }
    if alpha == 65535 {
        return (256 - k) << 16;
    }
    let h = 31 - alpha.leading_zeros();
    let x = alpha << (23 - h);
    let m = x + round_odd_divisor(x, 65535);
    let t = m * k;
    let bits = 32 - t.leading_zeros();
    let shift = bits.saturating_sub(24);
    let u = round_even(t, shift);
    let r = 47 - h - shift;
    (1 << 24) - round_even(u, r - 24)
}

/// Returns the exact binary32 channel C as the integer C * 2^24.
pub fn channel(source: u32, destination: u32, inverse: u32, k: u32) -> Wide {
    let weighted = source * k;
    let source_fixed = Wide {
        hi: weighted >> 16,
        lo: weighted << 16,
    };
    let destination_fixed = round24(product(destination, inverse));
    round24(add(source_fixed, destination_fixed))
}

fn significand(x: Wide, bits: u32) -> u32 {
    if bits <= 24 {
        return x.lo << (24 - bits);
    }
    let shift = bits - 24; // 1..=17; low discarded bits are zero after round24.
    (x.hi << (32 - shift)) | (x.lo >> shift)
}

/// Bit pattern of the normal binary32 value x / 2^24; x is zero or an R24
/// result in the channel range. This is also useful for inverse-stage checks.
pub fn fixed_bits(x: Wide) -> u32 {
    let bits = bit_length(x);
    if bits == 0 {
        return 0;
    }
    let m = significand(x, bits);
    ((bits + 102) << 23) | (m & 0x7f_ffff)
}

pub fn parent(x: Wide) -> u32 {
    let biased = add(x, Wide { hi: 0, lo: 1 << 23 });
    (biased.hi << 8) | (biased.lo >> 24)
}

/// Bit pattern of RN32((x/2^24)/257), including the separate division rounding.
pub fn root_bits(x: Wide) -> u32 {
    let bits = bit_length(x);
    if bits == 0 {
        return 0;
    }
    let m = significand(x, bits);
    let (scaled, shift) = if m >= 8_421_376 { (m, 8) } else { (2 * m, 9) };
    let rounded = scaled - round_odd_divisor(scaled, 257);
    ((bits + 102 - shift) << 23) | (rounded & 0x7f_ffff)
}

pub fn root(x: Wide) -> u32 {
    let bits = root_bits(x);
    if bits == 0 {
        return 0;
    }
    let exponent = (bits >> 23) & 255;
    let shift = 150 - exponent;
    if shift > 24 {
        return 0;
    }
    let m = (bits & 0x7f_ffff) | (1 << 23);
    (m + (1 << (shift - 1))) >> shift
}
