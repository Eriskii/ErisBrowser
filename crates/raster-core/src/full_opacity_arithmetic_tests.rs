//! Independent staged-f32 regression oracle retained from the standalone verifier.
//! Wider integers and floats occur only in this independent test driver.
mod integer;
use integer::Wide;

const SCALE: f64 = 281_474_976_710_656.0; // 2^48, exact
const BROAD_SAMPLES: u32 = 262_144;
const SEED: u32 = 0x91e1_0da5;
const OPACITIES: [u32; 30] = [
    0x0000_0000,
    0x0000_0001,
    0x007f_ffff,
    0x0080_0000,
    0x0080_0001,
    0x32ff_ffff,
    0x3300_0000,
    0x3300_0001,
    0x337f_ffff,
    0x3380_0000,
    0x3380_0001,
    0x3a83_126f,
    0x3b03_126f,
    0x3c23_d70a,
    0x3d4c_cccd,
    0x3dcc_cccc,
    0x3dcc_cccd,
    0x3dcc_ccce,
    0x3e4c_cccd,
    0x3e99_999a,
    0x3eaa_aaab,
    0x3eff_ffff,
    0x3f00_0000,
    0x3f00_0001,
    0x3f33_3332,
    0x3f33_3333,
    0x3f33_3334,
    0x3f66_6666,
    0x3f7f_fffe,
    0x3f7f_ffff,
];

#[derive(Default, Debug, PartialEq, Eq)]
struct Counts {
    classification: u64,
    products: u64,
    additions: u64,
    shifts: u64,
    round_even: u64,
    round24: u64,
    inverse: u64,
    channels: u64,
    general_inverse_bits: u64,
    general_channel_bits: u64,
    tiny_final_channels: u64,
    roots: u64,
    general_root_bits: u64,
    tiny_final_roots: u64,
    root_boundaries: u64,
    broad_samples: u64,
    literals: u64,
}

// Separate functions preserve Canvas's individual binary32 operations. There
// is no mul_add, integer-helper call, or exact-rational shortcut in this oracle.
#[inline(never)]
fn multiply(a: f32, b: f32) -> f32 {
    a * b
}
#[inline(never)]
fn divide(a: f32, b: f32) -> f32 {
    a / b
}
#[inline(never)]
fn subtract(a: f32, b: f32) -> f32 {
    a - b
}
#[inline(never)]
fn plus(a: f32, b: f32) -> f32 {
    a + b
}

fn oracle_inverse(alpha: u32, raw: u32) -> f32 {
    subtract(
        1.0,
        multiply(divide(alpha as f32, 65535.0), f32::from_bits(raw)),
    )
}

fn oracle_channel(source: u32, alpha: u32, destination: u32, raw: u32) -> f32 {
    plus(
        multiply(source as f32, f32::from_bits(raw)),
        multiply(destination as f32, oracle_inverse(alpha, raw)),
    )
}

fn wide(x: u64) -> Wide {
    Wide {
        hi: (x >> 32) as u32,
        lo: x as u32,
    }
}
fn value(x: Wide) -> u64 {
    (u64::from(x.hi) << 32) | u64::from(x.lo)
}

fn check_root(x: Wide, expected_channel: f32, label: &str, c: &mut Counts) -> Result<u32, String> {
    let expected = divide(expected_channel, 257.0);
    let bits = integer::root_bits48(x);
    let actual = integer::root48(x);
    if bits != expected.to_bits() || actual != expected.round() as u32 {
        return Err(format!(
            "root {label}: fixed={x:?} C_bits={:#010x} expected_bits={:#010x} actual_bits={bits:#010x} expected_integer={} actual_integer={actual}",
            expected_channel.to_bits(),
            expected.to_bits(),
            expected.round() as u32
        ));
    }
    c.roots += 1;
    c.general_root_bits += 1;
    Ok(actual)
}

fn check_channel(
    source: u32,
    alpha: u32,
    destination: u32,
    raw: u32,
    root: bool,
    label: &str,
    c: &mut Counts,
) -> Result<u32, String> {
    if source > alpha || alpha > 65535 || destination > 65535 || (root && destination % 257 != 0) {
        return Err(format!(
            "invalid verifier channel setup {label}: S={source} A={alpha} D={destination} root={root}"
        ));
    }
    let class = integer::classify(raw);
    let expected = oracle_channel(source, alpha, destination, raw);
    if class <= integer::TINY {
        // This is final-storage identity, not equality of intermediate C bits.
        if destination != expected.round() as u32 {
            return Err(format!(
                "tiny parent {label}: S={source} A={alpha} D={destination} raw={raw:#010x} C_bits={:#010x} expected={}",
                expected.to_bits(),
                expected.round() as u32
            ));
        }
        c.channels += 1;
        c.tiny_final_channels += 1;
        if root {
            let actual = destination / 257;
            let expected_root = divide(expected, 257.0);
            if actual != expected_root.round() as u32 {
                return Err(format!(
                    "tiny root {label}: S={source} A={alpha} D={destination} raw={raw:#010x} C_bits={:#010x} root_bits={:#010x} expected={} actual={actual}",
                    expected.to_bits(),
                    expected_root.to_bits(),
                    expected_root.round() as u32
                ));
            }
            c.roots += 1;
            c.tiny_final_roots += 1;
            return Ok(actual);
        }
        return Ok(destination);
    }
    if class != integer::GENERAL {
        return Err(format!("invalid pop class {class} raw={raw:#010x}"));
    }
    let j = integer::inverse(alpha, raw);
    let inverse_bits = integer::fixed_bits(Wide { hi: 0, lo: j }, 24);
    let expected_inverse = oracle_inverse(alpha, raw).to_bits();
    if inverse_bits != expected_inverse {
        return Err(format!(
            "channel inverse {label}: A={alpha} raw={raw:#010x} J={j} expected_bits={expected_inverse:#010x} actual_bits={inverse_bits:#010x}"
        ));
    }
    c.general_inverse_bits += 1;
    let x = integer::channel48(source, destination, j, raw);
    let bits = integer::fixed_bits(x, 48);
    let parent = integer::parent48(x);
    if bits != expected.to_bits() || parent != expected.round() as u32 || parent > 65535 {
        return Err(format!(
            "channel {label}: S={source} A={alpha} D={destination} raw={raw:#010x} J={j} fixed={x:?} expected_bits={:#010x} actual_bits={bits:#010x} expected_integer={} actual_integer={parent}",
            expected.to_bits(),
            expected.round() as u32
        ));
    }
    c.channels += 1;
    c.general_channel_bits += 1;
    if root {
        check_root(x, expected, label, c)
    } else {
        Ok(parent)
    }
}

fn primitive_boundaries(c: &mut Counts) -> Result<(), String> {
    for (raw, expected) in [
        (0x0000_0000, 0),
        (0x8000_0000, 0),
        (0x0000_0001, 1),
        (0x007f_ffff, 1),
        (0x0080_0000, 1),
        (0x32ff_ffff, 1),
        (0x3300_0000, 1),
        (0x3300_0001, 2),
        (0x3f7f_ffff, 2),
        (0x3f80_0000, 3),
        (0x3f80_0001, 4),
        (0x7f80_0000, 4),
        (0x7fc0_0000, 4),
        (0x7f80_0001, 4),
        (0xff80_0000, 4),
        (0xffc0_0000, 4),
        (0x8000_0001, 4),
        (0xbf00_0000, 4),
    ] {
        let actual = integer::classify(raw);
        if actual != expected {
            return Err(format!(
                "classify raw={raw:#010x} expected={expected} actual={actual}"
            ));
        }
        c.classification += 1;
    }
    for a in [0u32, 1, 2, 255, 256, 257, 32767, 32768, 65534, 65535] {
        for b in [
            0u32, 1, 2, 65535, 65536, 65537, 8_388_607, 8_388_608, 8_388_609, 16_777_214,
            16_777_215, 16_777_216,
        ] {
            let actual = value(integer::product16(a, b));
            let expected = u64::from(a) * u64::from(b);
            if actual != expected {
                return Err(format!(
                    "product16 a={a} b={b} expected={expected} actual={actual}"
                ));
            }
            c.products += 1;
        }
    }
    let factors = [0u32, 1, 255, 256, 65535, 65536, 8_388_608, 16_777_215];
    for a in factors {
        for b in factors {
            let actual = value(integer::product24(a, b));
            let expected = u64::from(a) * u64::from(b);
            if actual != expected {
                return Err(format!(
                    "product24 a={a} b={b} expected={expected} actual={actual}"
                ));
            }
            c.products += 1;
        }
    }
    let words = [
        0u64,
        1,
        (1 << 23) - 1,
        1 << 23,
        (1 << 24) - 1,
        (1 << 32) - 1,
        1 << 32,
        u64::MAX,
    ];
    for a in words {
        for b in words {
            let actual = value(integer::add(wide(a), wide(b)));
            let expected = (u128::from(a) + u128::from(b)) as u64;
            if actual != expected {
                return Err(format!(
                    "add a={a} b={b} expected={expected} actual={actual}"
                ));
            }
            c.additions += 1;
        }
        for shift in [
            0, 1, 15, 16, 17, 23, 24, 31, 32, 33, 39, 40, 47, 48, 63, 64, 65,
        ] {
            let l = value(integer::left(wide(a), shift));
            let r = value(integer::right(wide(a), shift));
            let expected_l = if shift < 64 { a << shift } else { 0 };
            let expected_r = if shift < 64 { a >> shift } else { 0 };
            if (l, r) != (expected_l, expected_r) {
                return Err(format!(
                    "shift a={a} shift={shift} expected=({expected_l},{expected_r}) actual=({l},{r})"
                ));
            }
            c.shifts += 2;
        }
    }
    for x in [
        0u32,
        1,
        (1 << 23) - 1,
        1 << 23,
        1 << 24,
        0x8000_0000,
        0x8000_0001,
        u32::MAX,
    ] {
        for shift in 0..=40 {
            let denominator = 1u64 << shift;
            let q = u64::from(x) / denominator;
            let rem = u64::from(x) % denominator;
            let expected =
                q + u64::from(2 * rem > denominator || (2 * rem == denominator && q % 2 != 0));
            let actual = integer::round_even(x, shift);
            if u64::from(actual) != expected {
                return Err(format!(
                    "round_even x={x} shift={shift} expected={expected} actual={actual}"
                ));
            }
            c.round_even += 1;
        }
    }
    for shift in 1..=40 {
        for q in [1u64 << 23, (1 << 23) + 1, (1 << 24) - 2, (1 << 24) - 1] {
            let half = 1u64 << (shift - 1);
            for rem in [0, half - 1, half, half + 1, (1 << shift) - 1] {
                let n = (q << shift) + rem;
                check_round24(n, c)?;
            }
        }
    }
    for n in [0u64, 1, 2, (1 << 23) - 1, 1 << 23, (1 << 24) - 1, 1 << 24] {
        check_round24(n, c)?;
    }
    Ok(())
}

fn check_round24(n: u64, c: &mut Counts) -> Result<(), String> {
    let parts = integer::round24_parts(wide(n));
    let actual = u128::from(parts.significand) << parts.shift;
    let expected = (n as f32) as u128;
    if actual != expected {
        return Err(format!(
            "R24 n={n} parts={parts:?} expected={expected} actual={actual}"
        ));
    }
    // An arbitrary all-high input can round to 2^64. Parts preserve that carry;
    // round24's pair reconstruction is only called inside its fit precondition.
    if expected <= u128::from(u64::MAX) && value(integer::round24(wide(n))) != expected as u64 {
        return Err(format!("R24 pair reconstruction n={n} expected={expected}"));
    }
    c.round24 += 1;
    Ok(())
}

fn exhaustive_selected(c: &mut Counts) -> Result<(), String> {
    for raw in OPACITIES {
        for alpha in 0..=65535 {
            let j = integer::inverse(alpha, raw);
            let bits = integer::fixed_bits(Wide { hi: 0, lo: j }, 24);
            let expected = oracle_inverse(alpha, raw).to_bits();
            if bits != expected {
                return Err(format!(
                    "inverse A={alpha} raw={raw:#010x} J={j} expected_bits={expected:#010x} actual_bits={bits:#010x}"
                ));
            }
            c.inverse += 1;
            check_channel(
                alpha / 2,
                alpha,
                (alpha * 4051 + 113) & 65535,
                raw,
                false,
                "exhaustive-parent",
                c,
            )?;
            check_channel(
                alpha,
                alpha,
                (alpha >> 8) * 257,
                raw,
                true,
                "exhaustive-root",
                c,
            )?;
        }
    }
    Ok(())
}

fn channel_boundaries(c: &mut Counts) -> Result<(), String> {
    for alpha in [
        0u32, 1, 2, 127, 128, 255, 256, 257, 258, 32767, 32768, 32895, 32896, 32897, 43689, 43690,
        43691, 65533, 65534, 65535,
    ] {
        for raw in OPACITIES {
            for source in [0, 1.min(alpha), alpha / 2, alpha.saturating_sub(1), alpha] {
                for destination in [0, 1, 127, 128, 255, 256, 257, 32767, 32768, 65534, 65535] {
                    check_channel(source, alpha, destination, raw, false, "boundary-parent", c)?;
                }
                for destination8 in [0, 1, 33, 34, 44, 127, 128, 254, 255] {
                    check_channel(
                        source,
                        alpha,
                        destination8 * 257,
                        raw,
                        true,
                        "boundary-root",
                        c,
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn root_boundaries(c: &mut Counts) -> Result<(), String> {
    // Five binary32 neighbors around all 255 RGB8 half-integer thresholds.
    for n in 0..255 {
        let center = multiply(plus(n as f32, 0.5), 257.0).to_bits();
        for bits in [center - 2, center - 1, center, center + 1, center + 2] {
            let input = f32::from_bits(bits);
            let fixed = (f64::from(input) * SCALE) as u64;
            check_root(wide(fixed), input, "root-half-neighbor", c)?;
            c.root_boundaries += 1;
        }
    }
    for m in [
        8_388_608u64,
        8_388_609,
        8_421_375,
        8_421_376,
        8_421_377,
        16_777_214,
        16_777_215,
    ] {
        for shift in 0..=40 {
            let fixed = m << shift;
            let input = (fixed as f64 / SCALE) as f32;
            check_root(wide(fixed), input, "root-normalization", c)?;
            c.root_boundaries += 1;
        }
    }
    for fixed in [0u64, 1 << 23, (1 << 23) + 1, 1 << 24, 1 << 47] {
        let input = (fixed as f64 / SCALE) as f32;
        check_root(wide(fixed), input, "root-zero-large-shift", c)?;
        c.root_boundaries += 1;
    }
    Ok(())
}

fn next(state: &mut u32) -> u32 {
    *state ^= *state << 13;
    *state ^= *state >> 17;
    *state ^= *state << 5;
    *state
}

fn broad(c: &mut Counts) -> Result<(), String> {
    let mut state = SEED;
    for index in 0..BROAD_SAMPLES {
        let alpha = next(&mut state) & 65535;
        let source = next(&mut state) % (alpha + 1);
        let destination_alpha = next(&mut state) & 65535;
        let destination = next(&mut state) % (destination_alpha + 1);
        // Exactly half the corpus covers each branch. The positive general
        // interval includes every valid exponent, not just the selected roster.
        let raw = if index % 2 == 0 {
            integer::TINY_BITS + 1 + next(&mut state) % (integer::ONE_BITS - integer::TINY_BITS - 1)
        } else {
            next(&mut state) % (integer::TINY_BITS + 1)
        };
        let channel = check_channel(source, alpha, destination, raw, false, "broad-parent", c)?;
        let out_alpha = check_channel(
            alpha,
            alpha,
            destination_alpha,
            raw,
            false,
            "broad-alpha",
            c,
        )?;
        if channel > out_alpha {
            return Err(format!(
                "premultiplication sample={index} S={source} A={alpha} D={destination} DA={destination_alpha} raw={raw:#010x} channel={channel} alpha={out_alpha}"
            ));
        }
        let destination8 = next(&mut state) & 255;
        check_channel(
            source,
            alpha,
            destination8 * 257,
            raw,
            true,
            "broad-root",
            c,
        )?;
        c.broad_samples += 1;
    }
    Ok(())
}

fn literals(c: &mut Counts) -> Result<(), String> {
    // Exact eight rows from the held prospective literal-candidates.json.
    for (name, raw, source8, background, expected) in [
        (
            "decimal-point-one-is-not-nearest-grid",
            0x3dcc_cccd,
            232u32,
            0x000000u32,
            0x171717u32,
        ),
        (
            "decimal-point-seven-needs-source-product-rounding",
            0x3f33_3333,
            5,
            0x000000,
            0x040404,
        ),
        ("smallest-subnormal", 0x0000_0001, 255, 0x334455, 0x334455),
        ("largest-subnormal", 0x007f_ffff, 255, 0x334455, 0x334455),
        ("smallest-normal", 0x0080_0000, 255, 0x334455, 0x334455),
        (
            "tiny-identity-threshold",
            0x3300_0000,
            255,
            0x334455,
            0x334455,
        ),
        (
            "next-above-identity-threshold",
            0x3300_0001,
            255,
            0x000000,
            0x000000,
        ),
        (
            "largest-value-below-unit-retains-layer-semantics",
            0x3f7f_ffff,
            5,
            0x000000,
            0x050505,
        ),
    ] {
        let mut actual = 0;
        for shift in [16, 8, 0] {
            let destination = ((background >> shift) & 255) * 257;
            actual |=
                check_channel(source8 * 257, 65535, destination, raw, true, name, c)? << shift;
        }
        if actual != expected {
            return Err(format!(
                "literal {name}: expected={expected:06x} actual={actual:06x}"
            ));
        }
        c.literals += 1;
    }
    Ok(())
}

#[test]
fn pair_shift_and_r24_boundaries() {
    let mut counts = Counts::default();
    primitive_boundaries(&mut counts)
        .unwrap_or_else(|error| panic!("{error}; completed={counts:?}"));
    assert_eq!(
        counts,
        Counts {
            classification: 18,
            products: 184,
            additions: 64,
            shifts: 272,
            round_even: 328,
            round24: 807,
            ..Counts::default()
        }
    );
    println!("pair_shift_and_r24_boundaries: {counts:?}");
}

#[test]
fn eight_frozen_literal_pixels() {
    let mut counts = Counts::default();
    literals(&mut counts).unwrap_or_else(|error| panic!("{error}; completed={counts:?}"));
    assert_eq!(
        counts,
        Counts {
            channels: 24,
            general_inverse_bits: 12,
            general_channel_bits: 12,
            tiny_final_channels: 12,
            roots: 24,
            general_root_bits: 12,
            tiny_final_roots: 12,
            literals: 8,
            ..Counts::default()
        }
    );
    println!("eight_frozen_literal_pixels: {counts:?}");
}

#[test]
fn all_alphas_at_selected_raw_opacity_patterns() {
    let mut counts = Counts::default();
    exhaustive_selected(&mut counts)
        .unwrap_or_else(|error| panic!("{error}; completed={counts:?}"));
    assert_eq!(
        counts,
        Counts {
            inverse: 1_966_080,
            channels: 3_932_160,
            general_inverse_bits: 3_014_656,
            general_channel_bits: 3_014_656,
            tiny_final_channels: 917_504,
            roots: 1_966_080,
            general_root_bits: 1_507_328,
            tiny_final_roots: 458_752,
            ..Counts::default()
        }
    );
    println!("all_alphas_at_selected_raw_opacity_patterns: {counts:?}");
}

#[test]
fn targeted_channel_and_tiny_final_boundaries() {
    let mut counts = Counts::default();
    channel_boundaries(&mut counts).unwrap_or_else(|error| panic!("{error}; completed={counts:?}"));
    assert_eq!(
        counts,
        Counts {
            channels: 60_000,
            general_inverse_bits: 46_000,
            general_channel_bits: 46_000,
            tiny_final_channels: 14_000,
            roots: 27_000,
            general_root_bits: 20_700,
            tiny_final_roots: 6_300,
            ..Counts::default()
        }
    );
    println!("targeted_channel_and_tiny_final_boundaries: {counts:?}");
}

#[test]
fn root_division_bits_at_half_and_normalization_boundaries() {
    let mut counts = Counts::default();
    root_boundaries(&mut counts).unwrap_or_else(|error| panic!("{error}; completed={counts:?}"));
    assert_eq!(
        counts,
        Counts {
            roots: 1_567,
            general_root_bits: 1_567,
            root_boundaries: 1_567,
            ..Counts::default()
        }
    );
    println!("root_division_bits_at_half_and_normalization_boundaries: {counts:?}");
}

#[test]
fn deterministic_balanced_premultiplied_samples() {
    let mut counts = Counts::default();
    broad(&mut counts).unwrap_or_else(|error| panic!("{error}; completed={counts:?}"));
    assert_eq!(
        counts,
        Counts {
            channels: 786_432,
            general_inverse_bits: 393_216,
            general_channel_bits: 393_216,
            tiny_final_channels: 393_216,
            roots: 262_144,
            general_root_bits: 131_072,
            tiny_final_roots: 131_072,
            broad_samples: 262_144,
            ..Counts::default()
        }
    );
    println!("deterministic_balanced_premultiplied_samples: {counts:?}");
}
