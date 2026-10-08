use super::*;
use crate::script::{MAX_HEAP, MAX_STEPS};

// CanonicalNumericIndexString accepts "-0" or a decimal Number::toString
// spelling. The latter begins with a digit, '-', 'N' (NaN) or 'I' (Infinity).
// https://tc39.es/ecma262/multipage/abstract-operations.html#sec-canonicalnumericindexstring
// https://tc39.es/ecma262/multipage/ecmascript-data-types-and-values.html#sec-numeric-types-number-tostring

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
        (Index::Numeric(number), Some(0x7ff8_0000_0000_0000)) => assert!(number.is_nan()),
        (Index::Numeric(number), Some(bits)) => assert_eq!(number.to_bits(), bits),
        pair => panic!("unexpected classification: {pair:?}"),
    }
}

#[test]
fn rejected_prefix_classes_ignore_remaining_utf16_without_storage() {
    let mut runtime = Runtime::try_new().unwrap();
    let original = counts(&runtime);
    // Neighbours of '-', digits, I and N; lower-case specials, signs, decimal
    // punctuation, whitespace/NUL, non-ASCII digits and isolated surrogates.
    for first in [
        0, 9, 32, 43, 44, 46, 47, 58, 72, 74, 77, 79, 101, 105, 110, 0xa0, 0x660, 0xff11, 0xd800,
        0xdfff,
    ] {
        for length in [1, 25] {
            let mut units = vec![0xdfff; length];
            units[0] = first;
            let key = JsString::from(units.as_slice());
            let pointer = key.units().as_ptr();
            runtime.steps = 16;
            runtime.allocated = MAX_HEAP;
            expect(runtime.typed_array_index(&key).unwrap(), None);
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            assert_eq!(key.units(), units);
            assert_eq!(key.units().as_ptr(), pointer);
            assert_eq!(counts(&runtime), original);
            clean(&runtime);
        }
    }
    // Every permitted prefix must reach the inherited tail, even where the
    // one-character input itself is ordinary. The prefix stage alone cannot
    // accept a number or decide that N, I or '-' is an ordinary key.
    for first in [45, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 73, 78] {
        let key = JsString::from(vec![first]);
        runtime.steps = 16;
        runtime.allocated = MAX_HEAP;
        assert!(
            runtime
                .typed_array_index(&key)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        assert_eq!(counts(&runtime), original);
        clean(&runtime);
    }
}

#[test]
fn permitted_prefixes_preserve_specials_fraction_exponents_and_extremes() {
    let mut runtime = Runtime::try_new().unwrap();
    for (text, bits) in [
        ("0", 0x0000_0000_0000_0000),
        ("-0", 0x8000_0000_0000_0000),
        ("NaN", 0x7ff8_0000_0000_0000),
        ("Infinity", 0x7ff0_0000_0000_0000),
        ("-Infinity", 0xfff0_0000_0000_0000),
        ("0.5", 0x3fe0_0000_0000_0000),
        ("1.25", 0x3ff4_0000_0000_0000),
        ("1e-7", 0x3e7a_d7f2_9abc_af48),
        ("1e+21", 0x444b_1ae4_d6e2_ef50),
        ("5e-324", 0x0000_0000_0000_0001),
        ("1.7976931348623157e+308", 0x7fef_ffff_ffff_ffff),
        ("9007199254740992", 0x4340_0000_0000_0000),
        ("-0.0000010000000000000002", 0xbeb0_c6f7_a0b5_ed8e),
    ] {
        let key = JsString::from(text);
        runtime.steps = MAX_STEPS;
        expect(runtime.typed_array_index(&key).unwrap(), Some(bits));
        assert_eq!(key, JsString::from(text));
        clean(&runtime);
    }
}

#[test]
fn allowed_prefix_is_not_numeric_acceptance_and_other_spellings_stay_ordinary() {
    let mut runtime = Runtime::try_new().unwrap();
    for text in [
        "+1",
        ".5",
        "e1",
        "Infinityx",
        "NaNx",
        "nan",
        "-NaN",
        "N",
        "I",
        "-",
        "01",
        "1e0",
        "1E+21",
        "0x1",
        "1e309",
        "1e-400",
    ] {
        runtime.steps = MAX_STEPS;
        expect(
            runtime.typed_array_index(&JsString::from(text)).unwrap(),
            None,
        );
        clean(&runtime);
    }
    for units in [
        &[49, 0xdfff][..],
        &[78, 0][..],
        &[45, 0xff11][..],
        &[73, 0xd800][..],
    ] {
        let key = JsString::from(units);
        runtime.steps = MAX_STEPS;
        expect(runtime.typed_array_index(&key).unwrap(), None);
        assert_eq!(key.units(), units);
        clean(&runtime);
    }
}

#[test]
fn original_length_guard_still_precedes_the_new_prefix_debit() {
    for units in [Vec::new(), vec![48; 26], vec![78; 26], vec![0xd800; 4096]] {
        let key = JsString::from(units.as_slice());
        for allowance in [0, 7, 8] {
            let mut runtime = Runtime::try_new().unwrap();
            runtime.steps = allowance;
            runtime.allocated = MAX_HEAP;
            let result = runtime.typed_array_index(&key);
            if allowance == 8 {
                expect(result.unwrap(), None);
            } else {
                assert!(result.unwrap_err().is_resource_limit());
            }
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            assert_eq!(key.units(), units);
            clean(&runtime);
        }
    }
}

#[test]
fn prefix_and_unchanged_tail_have_literal_exact_terminal_work_heap_cuts() {
    // Existing tail rows gain precisely8 work; rejected metadata names cost16.
    // 1.25 retains its independently frozen1064-byte parse/format allocation.
    for (text, work, heap, bits) in [
        ("foo", 16, 0, None),
        ("+1", 16, 0, None),
        ("0", 52, 0, Some(0)),
        ("-0", 42, 0, Some(0x8000_0000_0000_0000)),
        ("NaN", 52, 0, Some(0x7ff8_0000_0000_0000)),
        ("Infinity", 102, 0, Some(0x7ff0_0000_0000_0000)),
        ("-Infinity", 112, 0, Some(0xfff0_0000_0000_0000)),
        ("1.25", 309, 1064, Some(0x3ff4_0000_0000_0000)),
    ] {
        let key = JsString::from(text);
        for allowance in [0, 7, 8, 15, work - 1, work] {
            let mut runtime = Runtime::try_new().unwrap();
            let original = counts(&runtime);
            runtime.steps = allowance;
            runtime.allocated = MAX_HEAP - heap;
            let result = runtime.typed_array_index(&key);
            if allowance == work {
                expect(result.unwrap(), bits);
                assert_eq!(runtime.allocated, MAX_HEAP);
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                if allowance <= 16 {
                    assert_eq!(runtime.allocated, MAX_HEAP - heap);
                }
            }
            assert_eq!(runtime.steps, 0);
            assert_eq!(counts(&runtime), original);
            assert_eq!(key, JsString::from(text));
            clean(&runtime);
        }
        if heap != 0 {
            for room in [0, heap - 1] {
                let mut runtime = Runtime::try_new().unwrap();
                let original = counts(&runtime);
                runtime.steps = work;
                runtime.allocated = MAX_HEAP - room;
                assert!(
                    runtime
                        .typed_array_index(&key)
                        .unwrap_err()
                        .is_resource_limit()
                );
                assert_eq!(counts(&runtime), original);
                assert_eq!(key, JsString::from(text));
                clean(&runtime);
            }
        }
    }
}
