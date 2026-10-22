//! Independent literal resource cuts selected before producer inspection or execution.
use super::{supports_declaration_with_budget, supports_matches_with_budget};

#[test]
fn supports_plain_budget_exact_font_declarations() {
    for (property, value, fee) in [
        ("font-size", "12px", 455),
        ("font-size", "0", 377),
        ("font-size", "initial", 533),
    ] {
        let mut exact = fee;
        assert!(supports_declaration_with_budget(
            property, value, &mut exact
        ));
        assert_eq!(exact, 0);
        let mut short = fee - 1;
        assert!(!supports_declaration_with_budget(
            property, value, &mut short
        ));
        assert_eq!(short, 0);
    }
}

#[test]
fn supports_plain_budget_exact_condition_names() {
    for (query, implied, fee) in [
        ("(font-size:12px)", false, 538),
        ("font-size:12px", true, 509),
        (r"(w\69 dth:1px)", false, 286),
        (r"w\69 dth:1px", true, 257),
        (r"(font\2d size:12px)", false, 713),
    ] {
        let mut exact = fee;
        assert!(supports_matches_with_budget(query, implied, &mut exact));
        assert_eq!(exact, 0);
        let mut short = fee - 1;
        assert!(!supports_matches_with_budget(query, implied, &mut short));
        assert_eq!(short, 0);
    }
}

#[test]
fn supports_plain_budget_shared_work() {
    let mut work = 909;
    assert!(supports_declaration_with_budget(
        "font-size",
        "12px",
        &mut work
    ));
    assert_eq!(work, 454);
    assert!(!supports_declaration_with_budget(
        "font-size",
        "12px",
        &mut work
    ));
    assert_eq!(work, 0);
}

#[test]
fn supports_plain_budget_raw_value_boundary() {
    // This is a private CSS allowance, not a Runtime 100000-work success claim.
    let at_cap = "0".repeat(4096);
    assert_eq!(at_cap.len(), 4096);
    let mut exact = 106847;
    assert!(supports_declaration_with_budget(
        "font-size",
        &at_cap,
        &mut exact
    ));
    assert_eq!(exact, 0);
    let mut short = 106846;
    assert!(!supports_declaration_with_budget(
        "font-size",
        &at_cap,
        &mut short
    ));
    assert_eq!(short, 0);
    let over_cap = "0".repeat(4097);
    assert_eq!(over_cap.len(), 4097);
    for initial in [0, 1, 200000] {
        let mut work = initial;
        assert!(!supports_declaration_with_budget(
            "font-size",
            &over_cap,
            &mut work
        ));
        assert_eq!(work, initial);
    }
}

#[test]
fn supports_plain_budget_grammar_vs_exhaustion() {
    // These are Eris source-admission boundaries, not universal precision claims.
    for (property, value, fee) in [
        ("font-size", "-1e-999px", 585),
        ("font-size", "1e-999", 507),
    ] {
        let mut work = fee;
        assert!(!supports_declaration_with_budget(
            property, value, &mut work
        ));
        assert_eq!(work, 0);
    }
    for (query, initial, expected) in [
        ("not (font-size:-1px)", 100000, true),
        ("not (font-size:-1px)", 538, false),
        ("(display:block) or (font-size:12px)", 538, false),
        (r"not (w\69 dth:-1px)", 100000, true),
        (r"not (w\69 dth:-1px)", 286, false),
    ] {
        let mut work = initial;
        assert_eq!(
            supports_matches_with_budget(query, false, &mut work),
            expected,
            "{query}"
        );
        if expected {
            assert!(work > 0);
        } else {
            assert_eq!(work, 0);
        }
    }
}
