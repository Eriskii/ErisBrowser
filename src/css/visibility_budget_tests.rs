//! Source-derived resource boundaries, separate from visibility's semantic oracle.
use super::*;
use crate::selectors::Error;

#[test]
fn visibility_budget_exact_cuts_and_shared_allowance() {
    // One prepaid grammar scan (8n+1), then the existing tokenizer (2n+1).
    // Literal totals intentionally do not call production to discover a budget.
    for (text, fee) in [
        ("hidden", 62),
        ("visible", 72),
        (r"\68 idden", 92),
        (r"h\000069 dden", 132),
    ] {
        let mut exact = fee;
        assert_eq!(valid_visibility(text, &mut exact), Ok(true), "{text}");
        assert_eq!(exact, 0);
        let mut short = fee - 1;
        assert_eq!(valid_visibility(text, &mut short), Err(Error::Limit));
        assert_eq!(short, 0);
    }
    let mut shared = 133;
    assert_eq!(valid_visibility("visible", &mut shared), Ok(true));
    assert_eq!(shared, 61);
    assert_eq!(valid_visibility("hidden", &mut shared), Err(Error::Limit));
    assert_eq!(shared, 0);
    assert_eq!(valid_visibility("hidden", &mut shared), Err(Error::Limit));
    assert_eq!(shared, 0);
}

#[test]
fn visibility_budget_maximum_escaped_value_and_preallocation_refusal() {
    let at_cap = format!(r"\68 idden/*{}*/", "x".repeat(4083));
    assert_eq!(at_cap.len(), 4096);
    let mut exact = 40962;
    assert_eq!(valid_visibility(&at_cap, &mut exact), Ok(true));
    assert_eq!(exact, 0);
    let mut short = 40961;
    assert_eq!(valid_visibility(&at_cap, &mut short), Err(Error::Limit));
    assert_eq!(short, 0);
    // The raw byte bound includes comments and whitespace, not decoded length.
    let over_cap = format!("{at_cap} ");
    for initial in [0, 1, 100_000] {
        let mut work = initial;
        assert_eq!(valid_visibility(&over_cap, &mut work), Err(Error::Limit));
        assert_eq!(work, initial);
        assert!(!supports_declaration_with_budget(
            "visibility",
            &over_cap,
            &mut work
        ));
        assert_eq!(work, initial);
    }
}

#[test]
fn visibility_budget_escape_abuse_is_bounded_without_token_splicing() {
    for text in [
        "hidden\0",
        "hidden\\",
        "\\\nhidden",
        r"\0 hidden",
        r"\110000hidden",
        "hi/**/dden",
        "hidden/**/visible",
        "hidden()",
    ] {
        let mut work = 100_000;
        assert_eq!(valid_visibility(text, &mut work), Ok(false), "{text:?}");
        assert!(work < 100_000);
    }
    let dense = r"\000061 ".repeat(512);
    assert_eq!(dense.len(), 4096);
    let mut exact = 40962;
    assert_eq!(valid_visibility(&dense, &mut exact), Ok(false));
    assert_eq!(exact, 0);
    let mut none = 0;
    assert_eq!(valid_visibility(&dense, &mut none), Err(Error::Limit));
    assert_eq!(none, 0);
}

#[test]
fn visibility_budget_supports_uses_the_callers_remaining_work() {
    // Existing two-argument envelope, property/value token scans, token walk,
    // then the selected grammar. These one-token literals cost 20n+107.
    for (text, fee) in [("hidden", 227), ("visible", 247), (r"\68 idden", 287)] {
        let mut exact = fee;
        assert!(supports_declaration_with_budget(
            "visibility",
            text,
            &mut exact
        ));
        assert_eq!(exact, 0);
        let mut short = fee - 1;
        assert!(!supports_declaration_with_budget(
            "visibility",
            text,
            &mut short
        ));
        assert_eq!(short, 0);
    }
    let mut shared = 473;
    assert!(supports_declaration_with_budget(
        "visibility",
        "visible",
        &mut shared
    ));
    assert_eq!(shared, 226);
    assert!(!supports_declaration_with_budget(
        "visibility",
        "hidden",
        &mut shared
    ));
    assert_eq!(shared, 0);
    // Neither a decoded value nor a negated condition may turn exhaustion true.
    for query in [r"(vi\73 ibility:\68 idden)", r"not (visibility:\68 idden)"] {
        let mut zero = 0;
        assert!(!supports_matches_with_budget(query, false, &mut zero));
        assert_eq!(zero, 0);
    }
}

#[test]
fn visibility_budget_cssom_refusal_preserves_the_old_declaration() {
    use crate::cssom::{Error as InlineError, InlineStyle};
    let mut setup = 100_000;
    let original =
        InlineStyle::parse("visibility:visible!important;color:red", &mut setup).unwrap();
    for initial in [0, 19] {
        let mut style = original.clone();
        let mut work = initial;
        // The existing setProperty entry fee is 10 + 9 + 1 = 20.
        assert_eq!(
            style.set_property("visibility", r"\68 idden", "", &mut work),
            Err(InlineError::Limit)
        );
        assert_eq!(work, 0);
        assert_eq!(style, original);
    }
    let oversized = format!(r"\68 idden/*{}*/ ", "x".repeat(4083));
    assert_eq!(oversized.len(), 4097);
    let mut style = original.clone();
    let mut work = 5000;
    assert_eq!(
        style.set_property("visibility", &oversized, "", &mut work),
        Err(InlineError::Limit)
    );
    assert_eq!(work, 892); // entry 10 + 4097 + 1, then the value cap refuses.
    assert_eq!(style, original);
}
