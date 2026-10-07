//! Portable integer opacity arithmetic versus independent staged-f32 Canvas
//! arithmetic. These six groups initialize no renderer, device or GPU.
mod integer;
use integer::Wide;

const BROAD_SAMPLES: u32 = 262_144;
const SEED: u32 = 0x91e1_0da5;

#[derive(Default, Debug, PartialEq, Eq)]
struct Counts {
    pair_checks: u64,
    round24_checks: u64,
    inverse_checks: u64,
    channel_checks: u64,
    root_checks: u64,
    root_boundary_checks: u64,
    broad_samples: u64,
    literal_checks: u64,
}

// Explicit function boundaries preserve the source's separate float operators.
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

fn oracle_inverse(alpha: u32, k: u32) -> f32 {
    subtract(
        1.0,
        multiply(divide(alpha as f32, 65_535.0), divide(k as f32, 256.0)),
    )
}

fn oracle_channel(source: u32, alpha: u32, destination: u32, k: u32) -> f32 {
    plus(
        multiply(source as f32, divide(k as f32, 256.0)),
        multiply(destination as f32, oracle_inverse(alpha, k)),
    )
}

// Wide conversions are verifier-only; the portable algorithm never uses u64.
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
    let actual_bits = integer::root_bits(x);
    let actual = integer::root(x);
    if actual_bits != expected.to_bits() || actual != expected.round() as u32 {
        return Err(format!(
            "root {label}: fixed={x:?} channel_bits={:#010x} expected_bits={:#010x} actual_bits={actual_bits:#010x} expected_integer={} actual_integer={actual}",
            expected_channel.to_bits(),
            expected.to_bits(),
            expected.round() as u32
        ));
    }
    c.root_checks += 1;
    Ok(actual)
}

fn check_channel(
    source: u32,
    alpha: u32,
    destination: u32,
    k: u32,
    root: bool,
    label: &str,
    c: &mut Counts,
) -> Result<u32, String> {
    let inverse = integer::inverse(alpha, k);
    let x = integer::channel(source, destination, inverse, k);
    let expected = oracle_channel(source, alpha, destination, k);
    let bits = integer::fixed_bits(x);
    let parent = integer::parent(x);
    if bits != expected.to_bits() || parent != expected.round() as u32 || parent > 65535 {
        return Err(format!(
            "channel {label}: S={source} A={alpha} D={destination} k={k} J={inverse} fixed={x:?} expected_bits={:#010x} actual_bits={bits:#010x} expected_integer={} actual_integer={parent}",
            expected.to_bits(),
            expected.round() as u32
        ));
    }
    c.channel_checks += 1;
    if root {
        check_root(x, expected, label, c)
    } else {
        Ok(parent)
    }
}

fn primitive_boundaries(c: &mut Counts) -> Result<(), String> {
    for a in [0u32, 1, 2, 255, 256, 257, 32767, 32768, 65534, 65535] {
        for b in [
            0u32, 1, 2, 65535, 65536, 65537, 8_388_607, 8_388_608, 8_388_609, 16_777_214,
            16_777_215, 16_777_216,
        ] {
            let actual = value(integer::product(a, b));
            let expected = u64::from(a) * u64::from(b);
            if actual != expected {
                return Err(format!(
                    "pair product: a={a} b={b} expected={expected} actual={actual}"
                ));
            }
            c.pair_checks += 1;
        }
    }
    let additions = [
        0u64,
        1,
        (1 << 23) - 1,
        1 << 23,
        (1 << 24) - 1,
        (1 << 32) - 1,
        1 << 32,
        (1 << 40) - 1,
    ];
    for a in additions {
        for b in additions {
            let actual = value(integer::add(wide(a), wide(b)));
            if actual != a + b {
                return Err(format!(
                    "pair addition: a={a} b={b} expected={} actual={actual}",
                    a + b
                ));
            }
            c.pair_checks += 1;
        }
    }
    for shift in 1..=17 {
        for q in [(1u64 << 23), (1 << 23) + 1, (1 << 24) - 2, (1 << 24) - 1] {
            let half = 1u64 << (shift - 1);
            for r in [0, half - 1, half, half + 1, (1 << shift) - 1] {
                let n = (q << shift) + r;
                let actual = value(integer::round24(wide(n)));
                let expected = (n as f32) as u64;
                if actual != expected {
                    return Err(format!(
                        "R24: n={n} shift={shift} q={q} remainder={r} expected={expected} actual={actual}"
                    ));
                }
                c.round24_checks += 1;
            }
        }
    }
    for n in [0u64, 1, 2, (1 << 23) - 1, 1 << 23, (1 << 24) - 1, 1 << 24] {
        let actual = value(integer::round24(wide(n)));
        if actual != (n as f32) as u64 {
            return Err(format!("small R24 n={n} actual={actual}"));
        }
        c.round24_checks += 1;
    }
    Ok(())
}

fn exhaustive_inverse(c: &mut Counts) -> Result<(), String> {
    for alpha in 1..=65534 {
        for k in 1..=255 {
            let j = integer::inverse(alpha, k);
            let bits = integer::fixed_bits(Wide { hi: 0, lo: j });
            let expected = oracle_inverse(alpha, k).to_bits();
            if bits != expected {
                return Err(format!(
                    "inverse: A={alpha} k={k} J={j} expected_bits={expected:#010x} actual_bits={bits:#010x}"
                ));
            }
            c.inverse_checks += 1;
        }
    }
    // Endpoint identities are additional to all 16,711,170 interior pairs.
    for alpha in [0, 65535] {
        for k in 1..=255 {
            let j = integer::inverse(alpha, k);
            let actual = integer::fixed_bits(Wide { hi: 0, lo: j });
            let expected = oracle_inverse(alpha, k).to_bits();
            if actual != expected {
                return Err(format!(
                    "inverse endpoint: A={alpha} k={k} expected={expected:#010x} actual={actual:#010x}"
                ));
            }
            c.inverse_checks += 1;
        }
    }
    Ok(())
}

fn channel_boundaries(c: &mut Counts) -> Result<(), String> {
    let alphas = [
        0u32, 1, 2, 127, 128, 255, 256, 257, 258, 32767, 32768, 32895, 32896, 32897, 43689, 43690,
        43691, 65533, 65534, 65535,
    ];
    for alpha in alphas {
        for k in [1, 2, 63, 64, 65, 127, 128, 129, 191, 192, 193, 254, 255] {
            for source in [0, 1.min(alpha), alpha / 2, alpha.saturating_sub(1), alpha] {
                for destination in [0, 1, 127, 128, 255, 256, 257, 32767, 32768, 65534, 65535] {
                    check_channel(source, alpha, destination, k, false, "boundary-parent", c)?;
                }
                for destination8 in [0, 1, 33, 34, 44, 127, 128, 254, 255] {
                    check_channel(
                        source,
                        alpha,
                        destination8 * 257,
                        k,
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
    // Both neighbors of each root half-integer after scaling by 257. These
    // target division bits as well as the final integer, not scene admission.
    for n in 0..255 {
        let center = multiply(plus(n as f32, 0.5), 257.0).to_bits();
        for bits in [center - 2, center - 1, center, center + 1, center + 2] {
            let input = f32::from_bits(bits);
            let fixed = (f64::from(input) * 16_777_216.0) as u64;
            check_root(wide(fixed), input, "root-half-neighbor", c)?;
            c.root_boundary_checks += 1;
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
        for shift in 0..=16 {
            let fixed = m << shift;
            let input = (fixed as f64 / 16_777_216.0) as f32;
            check_root(wide(fixed), input, "root-normalization", c)?;
            c.root_boundary_checks += 1;
        }
    }
    for fixed in [
        0u64,
        1 << 16,
        (1 << 16) + 1,
        1 << 17,
        (1 << 23) - 1,
        1 << 23,
    ] {
        let input = (fixed as f64 / 16_777_216.0) as f32;
        check_root(wide(fixed), input, "root-zero-shift", c)?;
        c.root_boundary_checks += 1;
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
        let k = next(&mut state) % 255 + 1;
        let channel = check_channel(source, alpha, destination, k, false, "broad-parent", c)?;
        let out_alpha = check_channel(alpha, alpha, destination_alpha, k, false, "broad-alpha", c)?;
        if channel > out_alpha {
            return Err(format!(
                "premultiplication: sample={index} S={source} A={alpha} D={destination} DA={destination_alpha} k={k} channel={channel} alpha={out_alpha}"
            ));
        }
        let destination8 = next(&mut state) & 255;
        check_channel(source, alpha, destination8 * 257, k, true, "broad-root", c)?;
        c.broad_samples += 1;
    }
    Ok(())
}

fn literals(c: &mut Counts) -> Result<(), String> {
    let first = check_channel(0, 55 * 257, 34 * 257, 192, true, "literal-direct", c)?;
    if first != 28 {
        return Err(format!(
            "literal-direct expected=28 actual={first}; ideal-rational would be29"
        ));
    }
    c.literal_checks += 1;
    let inner = check_channel(0, 38 * 257, 151 * 257, 128, false, "literal-inner", c)?;
    let alpha = check_channel(
        38 * 257,
        38 * 257,
        65535,
        128,
        false,
        "literal-inner-alpha",
        c,
    )?;
    if (inner, alpha) != (35916, 65535) {
        return Err(format!(
            "literal-inner expected=(35916,65535) actual=({inner},{alpha})"
        ));
    }
    let second = check_channel(inner, alpha, 44 * 257, 127, true, "literal-nested", c)?;
    if second != 92 {
        return Err(format!(
            "literal-nested expected=92 actual={second}; ideal-rational intermediate would yield91"
        ));
    }
    c.literal_checks += 1;
    let empty_parent_alpha = check_channel(
        128 * 257,
        128 * 257,
        0,
        128,
        false,
        "literal-empty-parent",
        c,
    )?;
    let third = check_channel(
        0,
        empty_parent_alpha,
        65535,
        128,
        true,
        "literal-transparent-parent",
        c,
    )?;
    if (empty_parent_alpha, third) != (16448, 223) {
        return Err(format!(
            "literal-transparent-parent expected=(16448,223) actual=({empty_parent_alpha},{third})"
        ));
    }
    c.literal_checks += 1;
    Ok(())
}

#[test]
fn integer_pair_and_r24_boundaries() {
    let mut counts = Counts::default();
    primitive_boundaries(&mut counts).expect("integer pair and R24 boundary correspondence");
    assert_eq!(
        counts,
        Counts {
            pair_checks: 184,
            round24_checks: 347,
            ..Counts::default()
        }
    );
}

#[test]
fn literal_canvas_rounding_counterexamples() {
    let mut counts = Counts::default();
    literals(&mut counts).expect("literal staged-f32 rounding witnesses");
    assert_eq!(
        counts,
        Counts {
            channel_checks: 6,
            root_checks: 3,
            literal_checks: 3,
            ..Counts::default()
        }
    );
}

#[test]
fn all_alpha_opacity_inverse_pairs_match_staged_f32() {
    let mut counts = Counts::default();
    exhaustive_inverse(&mut counts).expect("complete alpha/opacity inverse correspondence");
    assert_eq!(
        counts,
        Counts {
            inverse_checks: 16_711_680,
            ..Counts::default()
        }
    );
}

#[test]
fn targeted_channel_boundaries_match_staged_f32() {
    let mut counts = Counts::default();
    channel_boundaries(&mut counts).expect("targeted premultiplied channel correspondence");
    assert_eq!(
        counts,
        Counts {
            channel_checks: 26_000,
            root_checks: 11_700,
            ..Counts::default()
        }
    );
}

#[test]
fn root_division_boundary_bits_match_staged_f32() {
    let mut counts = Counts::default();
    root_boundaries(&mut counts).expect("root division boundary bit correspondence");
    assert_eq!(
        counts,
        Counts {
            root_checks: 1_400,
            root_boundary_checks: 1_400,
            ..Counts::default()
        }
    );
}

#[test]
fn deterministic_premultiplied_samples_match_staged_f32() {
    let mut counts = Counts::default();
    broad(&mut counts).expect("deterministic premultiplied sample correspondence");
    assert_eq!(
        counts,
        Counts {
            channel_checks: 786_432,
            root_checks: 262_144,
            broad_samples: 262_144,
            ..Counts::default()
        }
    );
}
