//! Literal mixed CSS-wide getter expectations, frozen before production edits.
use super::*;

const MARGINS: [&str; 4] = ["margin-top", "margin-right", "margin-bottom", "margin-left"];
const KEYWORDS: [&str; 5] = ["initial", "inherit", "unset", "revert", "revert-layer"];
fn work() -> usize {
    20_000_000
}
fn parse(source: &str) -> InlineStyle {
    InlineStyle::parse(source, &mut work()).unwrap()
}
fn get(style: &InlineStyle, name: &str) -> String {
    style.get_property_value(name, &mut work()).unwrap()
}
fn text(style: &InlineStyle) -> String {
    style.serialize(&mut work()).unwrap()
}
fn set(style: &mut InlineStyle, name: &str, value: &str, priority: &str) {
    assert!(
        style
            .set_property(name, value, priority, &mut work())
            .unwrap()
    );
}

#[test]
fn every_css_wide_keyword_refuses_each_mixed_margin_position() {
    for keyword in KEYWORDS {
        for target in MARGINS {
            let mut style = parse("margin:1px");
            set(&mut style, target, keyword, "");
            let before = style.clone();
            let serialized = text(&style);
            assert_eq!(get(&style, "margin"), "", "{target}: {keyword}");
            assert_eq!(get(&style, "MARGIN"), "");
            assert_eq!(get(&style, target), keyword);
            assert_eq!(style.len(), 4);
            for (index, name) in MARGINS.into_iter().enumerate() {
                assert_eq!(style.item(index), name);
                assert_eq!(
                    get(&style, name),
                    if name == target { keyword } else { "1px" }
                );
            }
            assert_eq!(style, before);
            assert_eq!(text(&style), serialized);
            assert_eq!(parse(&serialized), style);
        }
        let uniform = parse(&format!("margin:{keyword}"));
        assert_eq!(get(&uniform, "margin"), keyword);
    }
    let different =
        parse("margin-top:initial;margin-right:inherit;margin-bottom:initial;margin-left:inherit");
    assert_eq!(get(&different, "margin"), "");
    assert_eq!(get(&different, "margin-top"), "initial");
    assert_eq!(get(&different, "margin-right"), "inherit");
}

#[test]
fn all_existing_joining_routes_reject_a_mixed_wide_component() {
    // Every existing shorthand_value joining arm and each property routed there.
    // Logical aliases below retain this engine's existing physical target order.
    for (source, shorthand, target, ordinary) in [
        ("margin:1px", "margin", "margin-left", "1px"),
        ("padding:1px", "padding", "padding-left", "1px"),
        ("inset:1px", "inset", "left", "1px"),
        (
            "border-width:1px",
            "border-width",
            "border-left-width",
            "1px",
        ),
        (
            "border-color:red",
            "border-color",
            "border-left-color",
            "red",
        ),
        (
            "border-style:solid",
            "border-style",
            "border-left-style",
            "solid",
        ),
        ("row-gap:1px;column-gap:2px", "gap", "column-gap", "1px 2px"),
        (
            "row-gap:1px;column-gap:2px",
            "grid-gap",
            "column-gap",
            "1px 2px",
        ),
        (
            "margin-left:1px;margin-right:2px",
            "margin-inline",
            "margin-right",
            "1px 2px",
        ),
        (
            "margin-top:1px;margin-bottom:2px",
            "margin-block",
            "margin-bottom",
            "1px 2px",
        ),
        (
            "padding-left:1px;padding-right:2px",
            "padding-inline",
            "padding-right",
            "1px 2px",
        ),
        (
            "padding-top:1px;padding-bottom:2px",
            "padding-block",
            "padding-bottom",
            "1px 2px",
        ),
        (
            "align-items:center;justify-items:start",
            "place-items",
            "justify-items",
            "center start",
        ),
        (
            "align-self:center;justify-self:start",
            "place-self",
            "justify-self",
            "center start",
        ),
        (
            "align-content:center;justify-content:start",
            "place-content",
            "justify-content",
            "center start",
        ),
        (
            "border-top:1px solid red",
            "border-top",
            "border-top-color",
            "1px solid red",
        ),
        (
            "border-right:1px solid red",
            "border-right",
            "border-right-color",
            "1px solid red",
        ),
        (
            "border-bottom:1px solid red",
            "border-bottom",
            "border-bottom-color",
            "1px solid red",
        ),
        (
            "border-left:1px solid red",
            "border-left",
            "border-left-color",
            "1px solid red",
        ),
        (
            "flex-grow:1;flex-shrink:1;flex-basis:auto",
            "flex",
            "flex-basis",
            "1 1 auto",
        ),
        (
            "flex-direction:row;flex-wrap:nowrap",
            "flex-flow",
            "flex-wrap",
            "row nowrap",
        ),
        (
            "list-style-type:circle;list-style-position:inside",
            "list-style",
            "list-style-position",
            "circle inside",
        ),
        (
            "grid-column-start:1;grid-column-end:3",
            "grid-column",
            "grid-column-end",
            "1 / 3",
        ),
        (
            "grid-row-start:1;grid-row-end:3",
            "grid-row",
            "grid-row-end",
            "1 / 3",
        ),
        (
            "grid-row-start:1;grid-column-start:2;grid-row-end:3;grid-column-end:4",
            "grid-area",
            "grid-column-end",
            "1 / 2 / 3 / 4",
        ),
        ("background:red", "background", "background-clip", "red"),
    ] {
        let mut style = parse(source);
        assert_eq!(get(&style, shorthand), ordinary, "{shorthand}");
        set(&mut style, target, "initial", "");
        let before = style.clone();
        assert_eq!(get(&style, shorthand), "", "{shorthand}");
        assert_eq!(get(&style, target), "initial");
        assert_eq!(style, before);
    }
    // Every side still has the same triple, so the old border equality guard
    // alone would accept the invalid string "initial solid red".
    let mut border = parse("border:1px solid red");
    assert_eq!(get(&border, "border"), "1px solid red");
    set(&mut border, "border-width", "initial", "");
    assert_eq!(get(&border, "border"), "");
    assert_eq!(get(&border, "border-width"), "initial");
    for shorthand in ["background", "border", "font", "all"] {
        let uniform = parse(&format!("{shorthand}:revert-layer"));
        assert_eq!(get(&uniform, shorthand), "revert-layer");
    }
}

#[test]
fn wide_keyword_case_escapes_and_whole_token_boundaries_are_preserved() {
    for (authored, normalized) in [
        ("INITIAL", "initial"),
        (r"\69 nitial", "initial"),
        (r"i\6e herit", "inherit"),
        (r"u\6e set", "unset"),
        (r"\72 evert", "revert"),
        (r"revert\2d layer", "revert-layer"),
    ] {
        let mut style = parse("margin:1px");
        set(&mut style, "margin-right", authored, "");
        assert_eq!(get(&style, "margin-right"), normalized);
        assert_eq!(get(&style, "margin"), "");
    }
    let uniform = parse(
        r"margin-top:INITIAL;margin-right:\69 nitial;margin-bottom:initial;margin-left:Initial",
    );
    assert_eq!(get(&uniform, "margin"), "initial");
    let comment =
        parse("margin-top:1px/* initial */;margin-right:1px;margin-bottom:1px;margin-left:1px");
    assert_eq!(get(&comment, "margin"), "1px");
    let url = parse(r#"background:url("initial.png")"#);
    assert_eq!(get(&url, "background"), r#"url("initial.png")"#);
}

#[test]
fn importance_pending_and_direct_var_checks_keep_their_existing_order() {
    let mixed_priority =
        parse("margin-top:initial!important;margin-right:1px;margin-bottom:1px;margin-left:1px");
    assert_eq!(get(&mixed_priority, "margin"), "");
    assert_eq!(
        mixed_priority
            .get_property_priority("margin", &mut work())
            .unwrap(),
        ""
    );
    let all_important = parse(
        "margin-top:initial!important;margin-right:1px!important;margin-bottom:1px!important;margin-left:1px!important",
    );
    assert_eq!(get(&all_important, "margin"), "");
    assert_eq!(
        all_important
            .get_property_priority("margin", &mut work())
            .unwrap(),
        "important"
    );
    let original = parse("margin:var(--m, initial)!important");
    assert_eq!(get(&original, "margin"), "var(--m, initial)");
    assert_eq!(get(&original, "margin-left"), "");
    assert_eq!(text(&original), "margin: var(--m, initial) !important;");
    let partial = parse("margin:var(--m)!important;margin-left:initial!important");
    assert_eq!(get(&partial, "margin"), "");
    assert_eq!(get(&partial, "margin-left"), "initial");
    let wrong_origin = parse("border:var(--b)");
    assert_eq!(get(&wrong_origin, "border-width"), "");
    assert_eq!(get(&wrong_origin, "border"), "var(--b)");
    let direct =
        parse("margin-top:var(--m, initial);margin-right:1px;margin-bottom:1px;margin-left:1px");
    assert_eq!(get(&direct, "margin"), "");
    assert_eq!(get(&direct, "margin-top"), "var(--m, initial)");
}

#[test]
fn removing_mixed_wide_shorthand_returns_empty_and_preserves_other_records() {
    let mut style =
        parse("color:red;margin-top:initial;margin-right:1px;margin-bottom:1px;margin-left:1px");
    let before = style.clone();
    assert_eq!(style.remove_property("margin", &mut 0), Err(Error::Limit));
    assert_eq!(style, before);
    assert_eq!(style.remove_property("MARGIN", &mut work()).unwrap(), "");
    assert_eq!(style.len(), 1);
    assert_eq!(style.item(0), "color");
    assert_eq!(get(&style, "color"), "red");
    assert_eq!(text(&style), "color: red;");
    let mut direct = parse("margin-left:initial");
    assert_eq!(
        direct.remove_property("margin-left", &mut work()).unwrap(),
        "initial"
    );
    assert_eq!(direct.len(), 0);
    let mut uniform = parse("margin:inherit");
    assert_eq!(
        uniform.remove_property("margin", &mut work()).unwrap(),
        "inherit"
    );
    assert_eq!(uniform.len(), 0);
}
