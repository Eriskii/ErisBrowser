use super::*;
use crate::selectors::Error;

#[test]
fn validator_preserves_payload_case_comments_punctuation_and_escapes() {
    for (source, expected) in [
        ("url(A/**/B.PNG?Q=Case#X)", "A/**/B.PNG?Q=Case#X"),
        ("/**/URL(\"A;!{B}/**/C\")/*tail*/", "A;!{B}/**/C"),
        ("url(a;{b}!c)", "a;{b}!c"),
        (r"u\72l(A\)B)", "A)B"),
        (r"\75rl(A\ B)", "A B"),
        ("url()", ""),
        ("url('#Icon')", "#Icon"),
        ("url(\"é雪\")", "é雪"),
        ("url(\0)", "\u{fffd}"),
        ("url(\"A\\\r\nB\")", "AB"),
    ] {
        assert_eq!(valid(source, &mut 100_000), Ok(true), "{source:?}");
        let mut budget = UrlBudget {
            work: 100_000,
            bytes: 100_000,
        };
        assert_eq!(
            decode(source, &mut budget).unwrap().unwrap().as_str(),
            expected
        );
    }
}

#[test]
fn validator_rejects_incomplete_tokens_modifiers_layers_and_tail_injection() {
    for source in [
        "",
        "A.png",
        "url (A)",
        "u/**/rl(A)",
        "url(A",
        "url(\"A\"",
        "url(A\\)",
        "url(A B)",
        "url(A(B)",
        "url(A\u{1}B)",
        "url(A\\\nB)",
        "url(/*x*/\"A\")",
        "url(A)/*",
        "url(A)]",
        "url(A);color:red",
        "url(A)!important",
        "url(A),url(B)",
        "url(A) url(B)",
        "url(\"A\" type(image/png))",
    ] {
        assert_eq!(valid(source, &mut 100_000), Ok(false), "{source:?}");
    }
}

#[test]
fn validator_exact_literal_work_and_all_short_cuts() {
    for (source, bytes, total, accepted) in [
        ("url(A)", 6, 93, true),
        ("url()", 5, 83, true),
        ("URL(\"A\")", 8, 113, true),
        ("url(A/**/B)", 11, 143, true),
        ("url(A", 5, 83, false),
        ("", 0, 33, false),
    ] {
        assert_eq!(source.len(), bytes);
        let mut work = total;
        assert_eq!(valid(source, &mut work), Ok(accepted));
        assert_eq!(work, 0);
        for short in 0..total {
            let mut work = short;
            assert_eq!(
                valid(source, &mut work),
                Err(Error::Limit),
                "{source:?}/{short}"
            );
            assert_eq!(work, 0);
        }
    }
}

#[test]
fn validator_uses_cumulative_work_without_storage_or_resolution_authority() {
    let mut work = 186;
    assert_eq!(valid("url(A)", &mut work), Ok(true));
    assert_eq!(work, 93);
    assert_eq!(valid("url(B)", &mut work), Ok(true));
    assert_eq!(work, 0);
    assert_eq!(valid("url(C)", &mut work), Err(Error::Limit));
    for value in [
        "url(file:///Private)",
        "url('http://[invalid')",
        "url(custom:A)",
    ] {
        assert_eq!(valid(value, &mut 100_000), Ok(true));
    }
}

#[test]
fn validator_maximum_raw_boundary_and_nul_replacement_scan() {
    for body in ["A".repeat(4091), "\0".repeat(4091)] {
        let value = format!("url({body})");
        assert_eq!(value.len(), 4096);
        let mut work = 40993;
        assert_eq!(valid(&value, &mut work), Ok(true));
        assert_eq!(work, 0);
        assert_eq!(valid(&value, &mut 40992), Err(Error::Limit));
        let mut untouched = 123;
        assert_eq!(valid(&(value + " "), &mut untouched), Err(Error::Limit));
        assert_eq!(untouched, 123);
    }
}

#[test]
fn decoder_keeps_original_work_storage_and_late_invalid_admission() {
    let storage = 18 + std::mem::size_of::<String>();
    let mut exact = UrlBudget {
        work: 93,
        bytes: storage,
    };
    assert_eq!(decode("url(A)", &mut exact).unwrap().unwrap().as_str(), "A");
    assert_eq!(exact, UrlBudget { work: 0, bytes: 0 });
    let mut short = UrlBudget {
        work: 93,
        bytes: storage - 1,
    };
    assert_eq!(decode("url(A)", &mut short), Err(UrlError::Limit));
    assert_eq!(short, UrlBudget { work: 0, bytes: 0 });
    let mut early = UrlBudget {
        work: 83,
        bytes: 100,
    };
    assert_eq!(decode("bogus", &mut early), Ok(None));
    assert_eq!(
        early,
        UrlBudget {
            work: 0,
            bytes: 100
        }
    );
    let mut late = UrlBudget {
        work: 83,
        bytes: 15 + std::mem::size_of::<String>(),
    };
    assert_eq!(decode("url(A", &mut late), Ok(None));
    assert_eq!(late, UrlBudget { work: 0, bytes: 0 });
}
