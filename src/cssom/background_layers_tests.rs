use super::*;
const WORK: usize = 4_000_000;
fn parse(s: &str) -> InlineStyle {
    InlineStyle::parse(s, &mut WORK.clone()).unwrap()
}
fn get(s: &InlineStyle, n: &str) -> String {
    s.get_property_value(n, &mut WORK.clone()).unwrap()
}
fn set(s: &mut InlineStyle, n: &str, v: &str, p: &str) -> bool {
    s.set_property(n, v, p, &mut WORK.clone()).unwrap()
}
fn text(s: &InlineStyle) -> String {
    s.serialize(&mut WORK.clone()).unwrap()
}

#[test]
fn layered_shorthand_preserves_order_seven_records_and_final_color() {
    let s = parse("background:none content-box,linear-gradient(red,blue) lime!important");
    assert_eq!(s.len(), 7);
    assert_eq!(
        get(&s, "background-image"),
        "none, linear-gradient(red,blue)"
    );
    assert_eq!(get(&s, "background-origin"), "content-box, padding-box");
    assert_eq!(
        get(&s, "background"),
        "none content-box, linear-gradient(red,blue) lime"
    );
    assert_eq!(parse(&text(&s)), s);
    assert_eq!(
        parse(&format!("background:{}!important", get(&s, "background"))),
        s
    );
}
#[test]
fn unequal_noninitial_lists_have_empty_shorthand_without_losing_longhands() {
    for (name, value) in [
        ("background-repeat", "no-repeat"),
        ("background-repeat", "no-repeat,no-repeat,no-repeat"),
        ("background-origin", "content-box"),
        ("background-position", "2px 0px"),
        ("background-size", "cover"),
    ] {
        let mut s = parse("background:none,none");
        assert!(set(&mut s, name, value, ""));
        assert_eq!(get(&s, "background"), "", "{name}:{value}");
        assert_eq!(parse(&text(&s)), s);
    }
}
#[test]
fn singleton_initial_dependents_may_be_omitted_but_repeated_initials_cannot() {
    for (name, value) in [
        ("background-repeat", "repeat"),
        ("background-origin", "padding-box"),
        ("background-clip", "border-box"),
        ("background-size", "auto auto"),
        ("background-position", "0% 0%"),
    ] {
        let mut s = parse("background:none,none");
        assert!(set(&mut s, name, value, ""));
        assert_eq!(get(&s, "background"), "none, none");
        assert_eq!(get(&s, name), value);
        assert_eq!(parse(&text(&s)), s);
    }
    let mut s = parse("background:none,none");
    set(&mut s, "background-repeat", "repeat,repeat,repeat", "");
    assert_eq!(get(&s, "background"), "");
}
#[test]
fn coordinating_image_length_is_never_expanded_to_fit_dependent_lists() {
    let mut s = parse("background:none");
    set(&mut s, "background-position", "2px 0px,4px 0px", "");
    assert_eq!(get(&s, "background"), "");
    assert_eq!(get(&s, "background-image"), "none");
    let mut s = parse("background:none,none");
    set(&mut s, "background-image", "none,none,none", "");
    assert_eq!(get(&s, "background"), "");
}
#[test]
fn invalid_setters_and_budget_refusal_preserve_the_complete_declaration() {
    let mut s = parse("background:linear-gradient(red),none blue!important");
    let before = s.clone();
    for value in [
        "red,none",
        "none,,none",
        "none,url(",
        "none,none initial",
        "none,none,none,none,none,none,none,none,none",
    ] {
        assert!(!set(&mut s, "background", value, "important"));
        assert_eq!(s, before);
    }
    assert_eq!(
        s.set_property("background", "none,none", "", &mut 0),
        Err(Error::Limit)
    );
    assert_eq!(s, before);
}
#[test]
fn priority_wide_pending_variables_and_removal_remain_seven_target_atomic() {
    let mut s = parse("background:none,none!important");
    set(&mut s, "background-repeat", "repeat", "");
    assert_eq!(get(&s, "background"), "");
    for value in [
        "inherit",
        "initial",
        "unset",
        "revert",
        "revert-layer",
        "var(--layers)",
    ] {
        let mut s = parse(&format!("background:{value}!important"));
        assert_eq!(get(&s, "background"), value);
        assert_eq!(
            s.remove_property("background", &mut WORK.clone()).unwrap(),
            value
        );
        assert_eq!(s, InlineStyle::default());
    }
}
#[test]
fn url_payloads_and_escaped_visual_boxes_survive_list_normalization() {
    let s = parse(
        r#"background:url("Dir/A,B/**/C.PNG?Q=X Y") \63 ontent-box,url(D,E.PNG) padding-box lime"#,
    );
    assert_eq!(
        get(&s, "background-image"),
        r#"url("Dir/A,B/**/C.PNG?Q=X Y"), url(D,E.PNG)"#
    );
    assert_eq!(get(&s, "background-origin"), "content-box, padding-box");
    assert_eq!(parse(&text(&s)), s);
}
#[test]
fn exactly_eight_layers_fit_the_existing_value_and_declaration_limits() {
    let eight = ["none"; 8].join(",");
    let mut s = InlineStyle::default();
    assert!(set(&mut s, "background", &eight, ""));
    assert_eq!(s.len(), 7);
    assert_eq!(get(&s, "background"), ["none"; 8].join(", "));
    assert!(!set(&mut s, "background", &["none"; 9].join(","), ""));
    assert_eq!(get(&s, "background"), ["none"; 8].join(", "));
}
