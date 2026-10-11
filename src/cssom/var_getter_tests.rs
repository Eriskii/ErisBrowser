//! Independent proposed semantic cases. Source-only: not installed or executed.
use super::*;
const WORK: usize = 20_000_000;
fn fresh_work() -> usize {
    WORK
}
fn parse(source: &str) -> InlineStyle {
    InlineStyle::parse(source, &mut fresh_work()).unwrap()
}
fn get(style: &InlineStyle, property: &str) -> String {
    style
        .get_property_value(property, &mut fresh_work())
        .unwrap()
}
fn text(style: &InlineStyle) -> String {
    style.serialize(&mut fresh_work()).unwrap()
}
fn set(style: &mut InlineStyle, name: &str, value: &str, priority: &str) {
    assert!(
        style
            .set_property(name, value, priority, &mut fresh_work())
            .unwrap()
    );
}
const MARGIN: [&str; 4] = ["margin-top", "margin-right", "margin-bottom", "margin-left"];

#[test]
fn explicit_var_longhands_cannot_be_synthesized_as_a_shorthand() {
    for priority in ["", "important"] {
        let mut style = InlineStyle::default();
        for name in MARGIN {
            set(&mut style, name, "var(--x)", priority);
        }
        let before = style.clone();
        assert_eq!(style.len(), 4);
        assert_eq!(get(&style, "margin"), "");
        assert_eq!(get(&style, "MARGIN"), "");
        assert_eq!(
            style
                .get_property_priority("margin", &mut fresh_work())
                .unwrap(),
            priority
        );
        for (index, name) in MARGIN.into_iter().enumerate() {
            assert_eq!(style.item(index), name);
            assert_eq!(get(&style, name), "var(--x)");
        }
        assert_eq!(style, before);
        let reparsed = parse(&text(&style));
        assert_eq!(reparsed, style);
        assert_eq!(get(&reparsed, "margin"), "");
    }
    let style = parse(
        "margin-top:var(--t);margin-right:var(--r);margin-bottom:var(--b);margin-left:var(--l)",
    );
    assert_eq!(get(&style, "margin"), "");
}

#[test]
fn one_nested_or_escaped_variable_blocks_only_shorthand_synthesis() {
    for value in [
        "var(--x)",
        "VAR(--x)",
        r"v\61 r(--x)",
        "calc(1px + var(--x))",
        "var(--x, var(--fallback, 2px))",
    ] {
        let mut style = parse("margin:1px");
        set(&mut style, "margin-left", value, "");
        assert_eq!(get(&style, "margin-left"), value);
        assert_eq!(get(&style, "margin"), "");
        assert_eq!(get(&style, "margin-top"), "1px");
    }
    // Even a known local custom value does not make a specified-value query computed.
    let style = parse(
        "--x:2px;margin-top:var(--x);margin-right:var(--x);margin-bottom:var(--x);margin-left:var(--x)",
    );
    assert_eq!(get(&style, "margin"), "");
    assert_eq!(get(&style, "margin-top"), "var(--x)");
}

#[test]
fn complete_original_pending_source_still_reconstructs() {
    let mut style = parse("margin:var(--x)!important");
    assert_eq!(get(&style, "margin"), "var(--x)");
    assert_eq!(get(&style, "margin-left"), "");
    assert_eq!(text(&style), "margin: var(--x) !important;");
    // Equal raw text in an explicitly replaced longhand is not original pending metadata.
    set(&mut style, "margin-left", "var(--x)", "important");
    assert_eq!(get(&style, "margin"), "");
    assert_eq!(get(&style, "margin-left"), "var(--x)");
    let border = parse("border:var(--b)");
    assert_eq!(get(&border, "border"), "var(--b)");
    assert_eq!(get(&border, "border-width"), "");
    let background = parse("background:var(--bg)");
    assert_eq!(get(&background, "background"), "var(--bg)");
}

#[test]
fn direct_vars_refuse_each_existing_synthesis_route() {
    for (source, shorthand, longhand, raw) in [
        (
            "row-gap:var(--g);column-gap:var(--g)",
            "gap",
            "row-gap",
            "var(--g)",
        ),
        (
            "border-left-width:var(--w);border-left-style:solid;border-left-color:red",
            "border-left",
            "border-left-width",
            "var(--w)",
        ),
        (
            "flex-grow:1;flex-shrink:1;flex-basis:var(--b)",
            "flex",
            "flex-basis",
            "var(--b)",
        ),
        (
            "grid-row-start:var(--r);grid-row-end:auto",
            "grid-row",
            "grid-row-start",
            "var(--r)",
        ),
        (
            "background:transparent;background-image:var(--image)",
            "background",
            "background-image",
            "var(--image)",
        ),
        (
            "background:transparent;background-image:linear-gradient(var(--c),blue)",
            "background",
            "background-image",
            "linear-gradient(var(--c),blue)",
        ),
    ] {
        let style = parse(source);
        assert_eq!(get(&style, shorthand), "", "{source}");
        assert_eq!(get(&style, longhand), raw, "{source}");
    }
}

#[test]
fn token_aware_guard_ignores_comments_strings_and_url_payloads() {
    let ordinary =
        parse("margin-top:1px/* var(--x) */;margin-right:1px;margin-bottom:1px;margin-left:1px");
    assert_eq!(get(&ordinary, "margin"), "1px");
    let url = parse(r#"background:transparent;background-image:url("var(--x).png")"#);
    assert_eq!(get(&url, "background-image"), r#"url("var(--x).png")"#);
    assert_eq!(get(&url, "background"), r#"url("var(--x).png")"#);
    assert_eq!(get(&parse("margin:initial"), "margin"), "initial");
    assert_eq!(
        get(&parse("margin:1px 2px 3px 4px"), "margin"),
        "1px 2px 3px 4px"
    );
}

#[test]
fn remove_shorthand_returns_empty_and_still_removes_direct_var_records() {
    let mut style = parse(
        "color:red;margin-top:var(--x);margin-right:var(--x);margin-bottom:var(--x);margin-left:var(--x)",
    );
    let before = style.clone();
    assert_eq!(style.remove_property("margin", &mut 0), Err(Error::Limit));
    assert_eq!(style, before);
    assert_eq!(
        style.remove_property("margin", &mut fresh_work()).unwrap(),
        ""
    );
    assert_eq!(style.len(), 1);
    assert_eq!(style.item(0), "color");
    assert_eq!(get(&style, "color"), "red");
    let mut direct = parse("margin-left:var(--x)");
    assert_eq!(
        direct
            .remove_property("margin-left", &mut fresh_work())
            .unwrap(),
        "var(--x)"
    );
    assert_eq!(direct.len(), 0);
}
