//! Literal resource cuts frozen separately from the pointer-events producer.
use super::*;
use crate::selectors::Error;

#[test]
fn pointer_events_budget_exact_cuts_and_shared_allowance() {
    // The contract fixes 8n+1 grammar admission followed by 2n+1 tokenization.
    for (text, fee) in [
        ("auto", 42),
        ("none", 42),
        (r"\6e one", 72),
        (r"n\6f ne", 72),
    ] {
        let mut exact = fee;
        assert_eq!(valid_pointer_events(text, &mut exact), Ok(true), "{text}");
        assert_eq!(exact, 0);
        let mut short = fee - 1;
        assert_eq!(valid_pointer_events(text, &mut short), Err(Error::Limit));
        assert_eq!(short, 0);
    }
    let mut shared = 83;
    assert_eq!(valid_pointer_events("auto", &mut shared), Ok(true));
    assert_eq!(shared, 41);
    assert_eq!(valid_pointer_events("none", &mut shared), Err(Error::Limit));
    assert_eq!(shared, 0);
    assert_eq!(valid_pointer_events("none", &mut shared), Err(Error::Limit));
    assert_eq!(shared, 0);
}

#[test]
fn pointer_events_budget_maximum_value_and_preallocation_refusal() {
    let at_cap = format!(r"\6e one/*{}*/", "x".repeat(4085));
    assert_eq!(at_cap.len(), 4096);
    let mut exact = 40962;
    assert_eq!(valid_pointer_events(&at_cap, &mut exact), Ok(true));
    assert_eq!(exact, 0);
    let mut short = 40961;
    assert_eq!(valid_pointer_events(&at_cap, &mut short), Err(Error::Limit));
    assert_eq!(short, 0);
    let over_cap = format!("{at_cap} ");
    assert_eq!(over_cap.len(), 4097);
    for initial in [0, 1, 100_000] {
        let mut work = initial;
        assert_eq!(
            valid_pointer_events(&over_cap, &mut work),
            Err(Error::Limit)
        );
        assert_eq!(work, initial);
    }
}

#[test]
fn pointer_events_budget_hostile_values_and_dense_escapes() {
    let hostile = [
        "none\0",
        "none\\",
        "\\\nnone",
        r"\0 none",
        r"\110000none",
        "no/**/ne",
        "none/**/auto",
        "none()",
    ];
    // This is an actual NUL in the runtime string, not a literal backslash-u.
    assert_eq!(hostile[0].as_bytes(), &[110, 111, 110, 101, 0]);
    for text in hostile {
        let mut work = 100_000;
        assert_eq!(valid_pointer_events(text, &mut work), Ok(false), "{text:?}");
        assert!(work < 100_000);
    }
    let dense = r"\000061 ".repeat(512);
    assert_eq!(dense.len(), 4096);
    let mut exact = 40962;
    assert_eq!(valid_pointer_events(&dense, &mut exact), Ok(false));
    assert_eq!(exact, 0);
    let mut zero = 0;
    assert_eq!(valid_pointer_events(&dense, &mut zero), Err(Error::Limit));
    assert_eq!(zero, 0);
}

#[test]
fn pointer_events_budget_supports_preserves_shared_work() {
    // These fixed one-token cuts include the existing supports envelope.
    for (text, fee) in [("auto", 227), ("none", 227), (r"\6e one", 287)] {
        let mut exact = fee;
        assert!(supports_declaration_with_budget(
            "pointer-events",
            text,
            &mut exact
        ));
        assert_eq!(exact, 0);
        let mut short = fee - 1;
        assert!(!supports_declaration_with_budget(
            "pointer-events",
            text,
            &mut short
        ));
        assert_eq!(short, 0);
    }
    let mut shared = 453;
    assert!(supports_declaration_with_budget(
        "pointer-events",
        "auto",
        &mut shared
    ));
    assert_eq!(shared, 226);
    assert!(!supports_declaration_with_budget(
        "pointer-events",
        "none",
        &mut shared
    ));
    assert_eq!(shared, 0);
    for query in [
        r"(pointer-events:\6e one)",
        r"(pointer\2d events:none)",
        "not (pointer-events:none)",
    ] {
        let mut zero = 0;
        assert!(!supports_matches_with_budget(query, false, &mut zero));
        assert_eq!(zero, 0);
    }
}

#[test]
fn pointer_events_budget_cssom_refusal_preserves_declarations() {
    use crate::cssom::{Error as InlineError, InlineStyle};
    let mut setup = 100_000;
    let original =
        InlineStyle::parse("pointer-events:auto!important;color:red", &mut setup).unwrap();
    for initial in [0, 21] {
        let mut style = original.clone();
        let mut work = initial;
        // Entry admission is 14 property bytes + 7 value bytes + 1 = 22.
        assert_eq!(
            style.set_property("pointer-events", r"\6e one", "", &mut work),
            Err(InlineError::Limit)
        );
        assert_eq!(work, 0);
        assert_eq!(style, original);
    }
    let oversized = format!(r"\6e one/*{}*/ ", "x".repeat(4085));
    assert_eq!(oversized.len(), 4097);
    let mut style = original.clone();
    let mut work = 5000;
    assert_eq!(
        style.set_property("pointer-events", &oversized, "", &mut work),
        Err(InlineError::Limit)
    );
    assert_eq!(work, 888); // Entry 14 + 4097 + 1 = 4112, then raw value refusal.
    assert_eq!(style, original);
}
