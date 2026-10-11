//! No new tariff: the complete existing 8V+1 debit precedes classification.
use super::*;
fn parse(source: &str) -> InlineStyle {
    let mut work = 20_000_000;
    InlineStyle::parse(source, &mut work).unwrap()
}

#[test]
fn mixed_and_uniform_wide_getters_keep_literal_exact_and_one_short_cuts() {
    // N50, p6, K4: Q=7+153+228=388. The entire8V+1 is paid
    // even when the first/last selected component makes synthesis impossible.
    for (source, expected, fee) in [
        (
            "margin-top:initial;margin-right:1px;margin-bottom:1px;margin-left:1px",
            "",
            549,
        ),
        (
            "margin-top:1px;margin-right:1px;margin-bottom:1px;margin-left:initial",
            "",
            549,
        ),
        (
            "margin-top:inherit;margin-right:1px;margin-bottom:1px;margin-left:1px",
            "",
            549,
        ),
        (
            "margin-top:unset;margin-right:1px;margin-bottom:1px;margin-left:1px",
            "",
            533,
        ),
        (
            "margin-top:revert;margin-right:1px;margin-bottom:1px;margin-left:1px",
            "",
            541,
        ),
        (
            "margin-top:revert-layer;margin-right:1px;margin-bottom:1px;margin-left:1px",
            "",
            589,
        ),
        (
            "margin-top:initial;margin-right:inherit;margin-bottom:initial;margin-left:inherit",
            "",
            645,
        ),
        ("margin:initial", "initial", 645),
    ] {
        let style = parse(source);
        let before = style.clone();
        let mut exact = fee;
        assert_eq!(
            style.get_property_value("margin", &mut exact),
            Ok(expected.into()),
            "{source}"
        );
        assert_eq!(exact, 0);
        let mut short = fee - 1;
        assert_eq!(
            style.get_property_value("margin", &mut short),
            Err(Error::Limit),
            "{source}"
        );
        assert_eq!(short, 0);
        assert_eq!(style, before);
    }
}

#[test]
fn earlier_priority_pending_and_ad_var_tariffs_remain_exact() {
    let priority =
        parse("margin-top:initial!important;margin-right:1px;margin-bottom:1px;margin-left:1px");
    let before = priority.clone();
    let mut exact = 388; // Q only; importance mismatch precedes ordinary8V+1.
    assert_eq!(
        priority.get_property_value("margin", &mut exact),
        Ok(String::new())
    );
    assert_eq!(exact, 0);
    assert_eq!(
        priority.get_property_value("margin", &mut 387),
        Err(Error::Limit)
    );
    assert_eq!(priority, before);
    for (source, expected, fee) in [
        ("margin:1px", "1px", 517),
        (
            "margin-top:var(--x);margin-right:var(--x);margin-bottom:var(--x);margin-left:var(--x)",
            "",
            677,
        ),
        ("margin:var(--m)!important", "var(--m)", 536),
    ] {
        let style = parse(source);
        let before = style.clone();
        let mut exact = fee;
        assert_eq!(
            style.get_property_value("margin", &mut exact),
            Ok(expected.into())
        );
        assert_eq!(exact, 0);
        let mut short = fee - 1;
        assert_eq!(
            style.get_property_value("margin", &mut short),
            Err(Error::Limit)
        );
        assert_eq!(short, 0);
        assert_eq!(style, before);
    }
}

#[test]
fn mixed_wide_removal_exact2770_and_short2769_are_transactional() {
    let original = parse("margin-top:initial;margin-right:1px;margin-bottom:1px;margin-left:1px");
    assert_eq!(original.logical_bytes(), 78);
    assert_eq!(original.release_work(), 83);
    assert_eq!(original.clone_work(), 1218);
    // Getter549 + secondQ388 + clone1218 + mutation457 +
    // empty staged summary1 + equality(1+2*(78+0))157 =2770.
    let mut short_style = original.clone();
    let mut short = 2769;
    assert_eq!(
        short_style.remove_property("margin", &mut short),
        Err(Error::Limit)
    );
    assert_eq!(short, 0);
    assert_eq!(short_style, original);
    let mut exact_style = original.clone();
    let mut exact = 2770;
    assert_eq!(
        exact_style.remove_property("margin", &mut exact),
        Ok(String::new())
    );
    assert_eq!(exact, 0);
    assert_eq!(exact_style, InlineStyle::default());
    assert_eq!(exact_style.len(), 0);
}
