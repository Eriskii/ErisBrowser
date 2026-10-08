use super::*;
use crate::script::{MAX_HEAP, MAX_STEPS};

// Exact UTF-16/classification/length-four columns copied from the frozen 67
// rows. Numeric bits are an additive independent rational nearest-even
// derivation, not output produced by the candidate parser or formatter.
const LITERALS: &[(&str, &[u16], Option<u64>, bool)] = &[
    ("numeric_0", &[48], Some(0x0000000000000000), true),
    ("numeric_1", &[49], Some(0x3ff0000000000000), true),
    ("numeric_2", &[51], Some(0x4008000000000000), true),
    ("numeric_3", &[52], Some(0x4010000000000000), false),
    ("numeric_4", &[45, 48], Some(0x8000000000000000), false),
    ("numeric_5", &[45, 49], Some(0xbff0000000000000), false),
    ("numeric_6", &[48, 46, 53], Some(0x3fe0000000000000), false),
    (
        "numeric_7",
        &[49, 46, 50, 53],
        Some(0x3ff4000000000000),
        false,
    ),
    ("numeric_8", &[78, 97, 78], Some(0x7ff8000000000000), false),
    (
        "numeric_9",
        &[73, 110, 102, 105, 110, 105, 116, 121],
        Some(0x7ff0000000000000),
        false,
    ),
    (
        "numeric_10",
        &[45, 73, 110, 102, 105, 110, 105, 116, 121],
        Some(0xfff0000000000000),
        false,
    ),
    (
        "numeric_11",
        &[52, 50, 57, 52, 57, 54, 55, 50, 57, 53],
        Some(0x41efffffffe00000),
        false,
    ),
    (
        "numeric_12",
        &[52, 50, 57, 52, 57, 54, 55, 50, 57, 54],
        Some(0x41f0000000000000),
        false,
    ),
    (
        "numeric_13",
        &[
            57, 48, 48, 55, 49, 57, 57, 50, 53, 52, 55, 52, 48, 57, 57, 49,
        ],
        Some(0x433fffffffffffff),
        false,
    ),
    (
        "numeric_14",
        &[
            57, 48, 48, 55, 49, 57, 57, 50, 53, 52, 55, 52, 48, 57, 57, 50,
        ],
        Some(0x4340000000000000),
        false,
    ),
    (
        "numeric_15",
        &[
            57, 48, 48, 55, 49, 57, 57, 50, 53, 52, 55, 52, 48, 57, 57, 52,
        ],
        Some(0x4340000000000001),
        false,
    ),
    (
        "numeric_16",
        &[
            49, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 49, 48, 48,
        ],
        Some(0x43abc16d674ec801),
        false,
    ),
    (
        "numeric_17",
        &[
            49, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 46, 50,
        ],
        Some(0x430c6bf526340002),
        false,
    ),
    (
        "numeric_18",
        &[
            45, 49, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 46, 50,
        ],
        Some(0xc30c6bf526340002),
        false,
    ),
    (
        "numeric_19",
        &[49, 101, 45, 55],
        Some(0x3e7ad7f29abcaf48),
        false,
    ),
    (
        "numeric_20",
        &[48, 46, 48, 48, 48, 48, 48, 49],
        Some(0x3eb0c6f7a0b5ed8d),
        false,
    ),
    (
        "numeric_21",
        &[
            45, 48, 46, 48, 48, 48, 48, 48, 49, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 50,
        ],
        Some(0xbeb0c6f7a0b5ed8e),
        false,
    ),
    (
        "numeric_22",
        &[49, 101, 43, 50, 49],
        Some(0x444b1ae4d6e2ef50),
        false,
    ),
    (
        "numeric_23",
        &[53, 101, 45, 51, 50, 52],
        Some(0x0000000000000001),
        false,
    ),
    (
        "numeric_24",
        &[
            49, 46, 55, 57, 55, 54, 57, 51, 49, 51, 52, 56, 54, 50, 51, 49, 53, 55, 101, 43, 51,
            48, 56,
        ],
        Some(0x7fefffffffffffff),
        false,
    ),
    ("ordinary_25", &[], None, false),
    ("ordinary_26", &[43, 48], None, false),
    ("ordinary_27", &[48, 48], None, false),
    ("ordinary_28", &[48, 49], None, false),
    ("ordinary_29", &[45, 48, 48], None, false),
    ("ordinary_30", &[43, 49], None, false),
    ("ordinary_31", &[49, 46, 48], None, false),
    ("ordinary_32", &[49, 46], None, false),
    ("ordinary_33", &[49, 101, 48], None, false),
    ("ordinary_34", &[49, 69, 43, 50, 49], None, false),
    ("ordinary_35", &[49, 101, 50, 49], None, false),
    ("ordinary_36", &[49, 101, 43, 48, 50, 49], None, false),
    ("ordinary_37", &[49, 101, 45, 54], None, false),
    (
        "ordinary_38",
        &[48, 46, 48, 48, 48, 48, 48, 48, 49],
        None,
        false,
    ),
    (
        "ordinary_39",
        &[
            57, 48, 48, 55, 49, 57, 57, 50, 53, 52, 55, 52, 48, 57, 57, 51,
        ],
        None,
        false,
    ),
    (
        "ordinary_40",
        &[
            49, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 49, 50, 56,
        ],
        None,
        false,
    ),
    (
        "ordinary_41",
        &[
            49, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 46, 51,
        ],
        None,
        false,
    ),
    (
        "ordinary_42",
        &[
            45, 49, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 46, 51,
        ],
        None,
        false,
    ),
    (
        "ordinary_43",
        &[43, 73, 110, 102, 105, 110, 105, 116, 121],
        None,
        false,
    ),
    ("ordinary_44", &[110, 97, 110], None, false),
    ("ordinary_45", &[43, 78, 97, 78], None, false),
    ("ordinary_46", &[45, 78, 97, 78], None, false),
    ("ordinary_47", &[32, 49], None, false),
    ("ordinary_48", &[49, 32], None, false),
    ("ordinary_49", &[9, 49], None, false),
    ("ordinary_50", &[160, 49], None, false),
    ("ordinary_51", &[48, 120, 49], None, false),
    ("ordinary_52", &[48, 88, 49], None, false),
    ("ordinary_53", &[48, 98, 49], None, false),
    ("ordinary_54", &[48, 111, 49], None, false),
    ("ordinary_55", &[46, 53], None, false),
    ("ordinary_56", &[49, 95, 48], None, false),
    ("ordinary_57", &[49, 101, 51, 48, 57], None, false),
    ("ordinary_58", &[49, 101, 45, 52, 48, 48], None, false),
    ("ordinary_59", &[116, 114, 117, 101], None, false),
    (
        "ordinary_60",
        &[117, 110, 100, 101, 102, 105, 110, 101, 100],
        None,
        false,
    ),
    (
        "ordinary_61",
        &[
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48,
        ],
        None,
        false,
    ),
    (
        "ordinary_62",
        &[
            45, 48, 46, 48, 48, 48, 48, 48, 49, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 50, 48,
        ],
        None,
        false,
    ),
    ("ordinary_63", &[55296], None, false),
    ("ordinary_64", &[49, 57343], None, false),
    ("ordinary_65", &[65297], None, false),
    ("ordinary_66", &[49, 0], None, false),
];

fn fresh() -> Runtime {
    Runtime::try_new().unwrap()
}

fn counts(runtime: &Runtime) -> (usize, usize, usize, usize) {
    (
        runtime.objects.len(),
        runtime.functions.len(),
        runtime.arrays.len(),
        runtime.environments.len(),
    )
}

fn clean(runtime: &Runtime) {
    assert!(runtime.frames.is_empty());
    assert!(runtime.active_array_joins.is_empty());
    assert_eq!(
        (
            runtime.calls,
            runtime.eval_depth,
            runtime.stack_units,
            runtime.json_depth
        ),
        (0, 0, 0, 0)
    );
}

fn expect(actual: Index, bits: Option<u64>) {
    match (actual, bits) {
        (Index::Ordinary, None) => {}
        (Index::Numeric(number), Some(bits)) => {
            if bits == 0x7ff8_0000_0000_0000 {
                assert!(number.is_nan());
            } else {
                assert_eq!(number.to_bits(), bits);
            }
        }
        (actual, expected) => panic!("classification differs: {actual:?}, {expected:?}"),
    }
}

#[test]
fn frozen_utf16_classifications_and_exact_numeric_values() {
    assert_eq!(LITERALS.len(), 67);
    let mut runtime = fresh();
    let original = counts(&runtime);
    for &(name, units, bits, valid_for_four) in LITERALS {
        let key = JsString::from(units);
        runtime.steps = MAX_STEPS;
        let actual = runtime.typed_array_index(&key).unwrap();
        expect(actual, bits);
        let valid = match actual {
            Index::Ordinary => false,
            Index::Numeric(number) => {
                number.is_finite()
                    && !number.is_sign_negative()
                    && number.fract() == 0.0
                    && number < 4.0
            }
        };
        assert_eq!(valid, valid_for_four, "{name}");
        assert_eq!(key.units(), units, "{name}");
        assert_eq!(counts(&runtime), original);
        clean(&runtime);
    }
}

#[test]
fn allocation_free_paths_have_literal_exact_work_boundaries() {
    for (text, work, bits) in [
        ("", 8, None),
        ("0", 44, Some(0)),
        ("7", 44, Some(0x401c_0000_0000_0000)),
        ("12", 70, Some(0x4028_0000_0000_0000)),
        ("-1", 60, Some(0xbff0_0000_0000_0000)),
        ("-0", 34, Some(0x8000_0000_0000_0000)),
        ("NaN", 44, Some(0x7ff8_0000_0000_0000)),
        ("Infinity", 94, Some(0x7ff0_0000_0000_0000)),
        ("-Infinity", 104, Some(0xfff0_0000_0000_0000)),
        ("4294967295", 224, Some(0x41ef_ffff_ffe0_0000)),
        ("9007199254740991", 344, Some(0x433f_ffff_ffff_ffff)),
    ] {
        let key = JsString::from(text);
        for allowance in [work, work - 1] {
            let mut runtime = fresh();
            let original = counts(&runtime);
            runtime.steps = allowance;
            runtime.allocated = MAX_HEAP;
            let actual = runtime.typed_array_index(&key);
            if allowance == work {
                expect(actual.unwrap(), bits);
            } else {
                assert!(actual.unwrap_err().is_resource_limit(), "{text}");
            }
            assert_eq!(runtime.steps, 0, "{text}");
            assert_eq!(runtime.allocated, MAX_HEAP, "{text}");
            assert_eq!(counts(&runtime), original);
            clean(&runtime);
        }
    }
}

#[test]
fn slow_parse_format_and_correction_have_literal_work_and_heap_endpoints() {
    for (text, work, heap, bits) in [
        ("1.25", 301, 1064, Some(0x3ff4_0000_0000_0000)),
        ("1.0", 266, 1062, None),
        ("9007199254740992", 758, 1088, Some(0x4340_0000_0000_0000)),
        ("1000000000000000.2", 982, 1124, Some(0x430c_6bf5_2634_0002)),
        ("1000000000000000.3", 980, 1124, None),
        (
            "-1000000000000000.2",
            1003,
            1126,
            Some(0xc30c_6bf5_2634_0002),
        ),
        ("-1000000000000000.3", 1001, 1126, None),
    ] {
        let key = JsString::from(text);
        for (work_limit, heap_limit, succeeds) in [
            (work, heap, true),
            (work - 1, heap, false),
            (work, heap - 1, false),
        ] {
            let mut runtime = fresh();
            let original = counts(&runtime);
            runtime.steps = work_limit;
            runtime.allocated = MAX_HEAP - heap_limit;
            let result = runtime.typed_array_index(&key);
            if succeeds {
                expect(result.unwrap(), bits);
                assert_eq!(runtime.steps, 0, "{text}");
                assert_eq!(runtime.allocated, MAX_HEAP, "{text}");
            } else {
                assert!(result.unwrap_err().is_resource_limit(), "{text}");
            }
            assert_eq!(counts(&runtime), original);
            assert_eq!(key, JsString::from(text));
            clean(&runtime);
        }
    }
}

#[test]
fn long_and_nonscalar_names_stop_without_heap_or_lossy_projection() {
    for (units, work) in [
        (vec![u16::from(b'0'); 26], 8),
        (vec![0xd800; 256 * 1024], 8),
        (vec![0xd800], 12),
        (vec![u16::from(b'1'), 0xdfff], 16),
        (vec![0xff11], 12),
    ] {
        let key = JsString::from(units.as_slice());
        for allowance in [work, work - 1] {
            let mut runtime = fresh();
            runtime.steps = allowance;
            runtime.allocated = MAX_HEAP;
            let result = runtime.typed_array_index(&key);
            if allowance == work {
                expect(result.unwrap(), None);
            } else {
                assert!(result.unwrap_err().is_resource_limit());
            }
            assert_eq!(runtime.steps, 0);
            assert_eq!(runtime.allocated, MAX_HEAP);
            assert_eq!(key.units(), units);
            clean(&runtime);
        }
    }
}

#[test]
fn parse_base_format_and_correction_storage_are_separately_admitted() {
    let key = JsString::from("1000000000000000.2");
    // Three allocation stages: primitive parse 68, inherited formatter 1024,
    // actual even-tie correction 32. Existing charge() records a refused
    // admission before returning Resource, but publishes no runtime object.
    for (available, work, charged, succeeds) in [
        (0, 566, 68, false),
        (67, 566, 68, false),
        (68, 694, 1092, false),
        (1091, 694, 1092, false),
        (1092, 868, 1124, false),
        (1123, 868, 1124, false),
        (1124, 982, 1124, true),
    ] {
        let mut runtime = fresh();
        let original = counts(&runtime);
        runtime.steps = MAX_STEPS;
        runtime.allocated = MAX_HEAP - available;
        let result = runtime.typed_array_index(&key);
        if succeeds {
            expect(result.unwrap(), Some(0x430c_6bf5_2634_0002));
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        assert_eq!(MAX_STEPS - runtime.steps, work);
        assert_eq!(runtime.allocated, MAX_HEAP - available + charged);
        assert_eq!(counts(&runtime), original);
        assert_eq!(key, JsString::from("1000000000000000.2"));
        clean(&runtime);
    }
}

#[test]
fn ordinary_metadata_and_alphabet_mismatches_need_no_heap() {
    // Literal tariffs include the reached special/integer prefix and stop at
    // the first disallowed alphabet unit. These are not runtime-derived fees.
    for (text, work) in [
        ("length", 74),
        ("buffer", 74),
        ("constructor", 94),
        ("x", 54),
        ("byteLength", 90),
        ("byteOffset", 90),
        ("values", 74),
        ("entries", 88),
        ("keys", 66),
        ("eX", 74),
        ("1eX", 104),
    ] {
        let key = JsString::from(text);
        for allowance in [work, work - 1] {
            let mut runtime = fresh();
            let original = counts(&runtime);
            runtime.steps = allowance;
            runtime.allocated = MAX_HEAP;
            let result = runtime.typed_array_index(&key);
            if allowance == work {
                expect(result.unwrap(), None);
            } else {
                assert!(result.unwrap_err().is_resource_limit(), "{text}");
            }
            assert_eq!(runtime.steps, 0, "{text}");
            assert_eq!(runtime.allocated, MAX_HEAP, "{text}");
            assert_eq!(counts(&runtime), original);
            assert_eq!(key, JsString::from(text));
            clean(&runtime);
        }
    }
}
