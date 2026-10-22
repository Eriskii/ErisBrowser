//! Literal cuts frozen by the root contract before producer inspection or execution.
use super::{VariableSyntax, supports_declaration_with_budget, supports_matches_with_budget};
use crate::selectors::Error;

#[test]
fn supports_var_budget_exact_declarations_and_shared_allowance() {
    for (property, value, fee) in [
        ("width", "var(--x)", 952),
        ("width", r"v\61 r(--x)", 988),
        ("width", r#"var(--x,";")"#, 1012),
        ("pointer-events", "var(--x)", 2212),
    ] {
        let mut exact = fee;
        assert!(
            supports_declaration_with_budget(property, value, &mut exact),
            "{property}: {value}"
        );
        assert_eq!(exact, 0);
        let mut short = fee - 1;
        assert!(!supports_declaration_with_budget(
            property, value, &mut short
        ));
        assert_eq!(short, 0);
    }
    let mut shared = 1903;
    assert!(supports_declaration_with_budget(
        "width",
        "var(--x)",
        &mut shared
    ));
    assert_eq!(shared, 951);
    assert!(!supports_declaration_with_budget(
        "width",
        "var(--x)",
        &mut shared
    ));
    assert_eq!(shared, 0);
}

#[test]
fn supports_var_budget_raw_value_cap_and_exact_admission() {
    let at_cap = format!("var(--x)/*{}*/", "x".repeat(4084));
    assert_eq!(at_cap.len(), 4096);
    let mut exact = 50008;
    assert!(supports_declaration_with_budget(
        "width", &at_cap, &mut exact
    ));
    assert_eq!(exact, 0);
    let mut short = 50007;
    assert!(!supports_declaration_with_budget(
        "width", &at_cap, &mut short
    ));
    assert_eq!(short, 0);
    let over_cap = format!("{at_cap} ");
    assert_eq!(over_cap.len(), 4097);
    for initial in [0, 1, 100_000] {
        let mut work = initial;
        assert!(!supports_declaration_with_budget(
            "width", &over_cap, &mut work
        ));
        assert_eq!(work, initial);
    }
}

#[test]
fn supports_var_budget_exhaustion_cannot_become_negated_or_short_circuit_success() {
    let mut funded = 100_000;
    assert!(supports_matches_with_budget(
        "not (width:var(foo))",
        false,
        &mut funded
    ));
    let mut late_short = 952;
    assert!(!supports_matches_with_budget(
        "not (width:var(foo))",
        false,
        &mut late_short
    ));
    assert_eq!(late_short, 0);
    let mut or_short = 952;
    assert!(!supports_matches_with_budget(
        "(display:block) or (width:var(--x))",
        false,
        &mut or_short
    ));
    assert_eq!(or_short, 0);
}

#[test]
fn supports_var_budget_typed_parser_cuts_and_legacy_option_wrapper() {
    let mut exact = 30;
    assert!(VariableSyntax::parse_checked("var(--x)", &mut exact).is_ok());
    assert_eq!(exact, 0);
    let mut short = 29;
    assert!(matches!(
        VariableSyntax::parse_checked("var(--x)", &mut short),
        Err(Error::Limit)
    ));
    assert_eq!(short, 0);
    let mut invalid = 30;
    assert!(matches!(
        VariableSyntax::parse_checked("var(foo)", &mut invalid),
        Err(Error::Invalid)
    ));
    assert_eq!(invalid, 0);

    let mut old_exact = 30;
    assert!(VariableSyntax::parse("var(--x)", &mut old_exact).is_some());
    assert_eq!(old_exact, 0);
    let mut old_invalid = 30;
    assert!(VariableSyntax::parse("var(foo)", &mut old_invalid).is_none());
    assert_eq!(old_invalid, 0);
    let mut old_short = 29;
    assert!(VariableSyntax::parse("var(--x)", &mut old_short).is_none());
    assert_eq!(old_short, 0);
}

#[test]
fn supports_var_budget_name_grammar_and_decoder_limits_remain_distinct() {
    const LOCAL_NAME: &str = "var(--xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx)";
    const DECODER_CAP_NAME: &str = "var(--xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx)";
    assert_eq!(LOCAL_NAME.len(), 262);
    assert_eq!(DECODER_CAP_NAME.len(), 1030);
    assert_eq!(
        LOCAL_NAME
            .strip_prefix("var(")
            .unwrap()
            .strip_suffix(')')
            .unwrap()
            .len(),
        257
    );
    assert_eq!(
        DECODER_CAP_NAME
            .strip_prefix("var(")
            .unwrap()
            .strip_suffix(')')
            .unwrap()
            .len(),
        1025
    );

    let mut local = 100_000;
    assert!(matches!(
        VariableSyntax::parse_checked(LOCAL_NAME, &mut local),
        Err(Error::Invalid)
    ));
    let mut decoder = 100_000;
    assert!(matches!(
        VariableSyntax::parse_checked(DECODER_CAP_NAME, &mut decoder),
        Err(Error::Limit)
    ));
    let mut old_local = 100_000;
    assert!(VariableSyntax::parse(LOCAL_NAME, &mut old_local).is_none());
    let mut old_decoder = 100_000;
    assert!(VariableSyntax::parse(DECODER_CAP_NAME, &mut old_decoder).is_none());
}

#[test]
fn supports_var_budget_no_var_keyword_preserves_the_old_literal_cut() {
    let mut exact = 227;
    assert!(supports_declaration_with_budget(
        "pointer-events",
        "none",
        &mut exact
    ));
    assert_eq!(exact, 0);
    let mut short = 226;
    assert!(!supports_declaration_with_budget(
        "pointer-events",
        "none",
        &mut short
    ));
    assert_eq!(short, 0);
}
