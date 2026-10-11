use super::*;

const WORK: usize = 4_000_000;
fn parse(source: &str) -> InlineStyle {
    InlineStyle::parse(source, &mut WORK.clone()).unwrap()
}
fn get(style: &InlineStyle, name: &str) -> String {
    style.get_property_value(name, &mut WORK.clone()).unwrap()
}
fn text(style: &InlineStyle) -> String {
    style.serialize(&mut WORK.clone()).unwrap()
}
fn set(style: &mut InlineStyle, name: &str, value: &str, priority: &str) -> bool {
    style
        .set_property(name, value, priority, &mut WORK.clone())
        .unwrap()
}

#[test]
fn border_spacing_cssom_literal_roundtrip_priority_and_removal() {
    let mut style = parse("border-spacing:2px 3px!important;color:blue;border-spacing:9px");
    assert_eq!(style.len(), 2);
    assert_eq!(style.item(0), "border-spacing");
    assert_eq!(style.item(1), "color");
    assert_eq!(get(&style, "border-spacing"), "2px 3px");
    assert_eq!(
        text(&style),
        "border-spacing: 2px 3px !important; color: blue;"
    );
    assert_eq!(parse(&text(&style)), style);
    // CSSOM setProperty step 9 replaces the value and unsets important when
    // priority is empty: https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-setproperty
    assert!(set(&mut style, "border-spacing", "0", ""));
    assert_eq!(get(&style, "border-spacing"), "0");
    assert_eq!(
        style.get_property_priority("border-spacing", &mut WORK.clone()),
        Ok("")
    );
    assert_eq!(get(&style, "color"), "blue");
    assert_eq!(text(&style), "border-spacing: 0; color: blue;");
    assert_eq!(parse(&text(&style)), style);
    assert!(set(&mut style, "border-spacing", "3px", "important"));
    assert_eq!(get(&style, "border-spacing"), "3px");
    assert_eq!(
        style.get_property_priority("border-spacing", &mut WORK.clone()),
        Ok("important")
    );
    assert_eq!(parse(&text(&style)), style);
    assert_eq!(
        style.remove_property("border-spacing", &mut WORK.clone()),
        Ok("3px".to_owned())
    );
    assert_eq!(text(&style), "color: blue;");
    assert_eq!(get(&style, "border-spacing"), "");
}

#[test]
fn border_spacing_cssom_invalid_and_budget_refusals_retain_both_axes() {
    let original = parse("border-spacing:2px 3px!important;color:blue");
    for value in [
        "calc(1px +)",
        "calc(1px",
        "calc(10% - 10% + 2px)",
        "0%",
        "2px -3px",
        "1",
        "1px 2px 3px",
        "2px,3px",
        "initial 2px",
    ] {
        let mut style = original.clone();
        assert_eq!(
            style.set_property("border-spacing", value, "important", &mut WORK.clone()),
            Ok(false),
            "{value}"
        );
        assert_eq!(style, original, "{value}");
        assert_eq!(text(&style), text(&original), "{value}");
    }
    let mut style = original.clone();
    assert_eq!(
        style.set_property("border-spacing", "4px 5px", "important", &mut 0),
        Err(Error::Limit)
    );
    assert_eq!(style, original);
    assert!(!set(&mut style, "border-spacing", "4px 5px", "urgent"));
    assert_eq!(style, original);
}

#[test]
fn border_spacing_cssom_wide_and_pending_values_reach_computed_pairs() {
    for (value, expected) in [
        ("inherit", [5.0, 9.0]),
        ("unset", [5.0, 9.0]),
        ("initial", [0.0, 0.0]),
        ("revert", [5.0, 9.0]),
        ("revert-layer", [5.0, 9.0]),
        ("var(--good)", [2.0, 3.0]),
        ("var(--bad)", [5.0, 9.0]),
        ("calc(5px - 2px) calc(1px - 3px)", [3.0, 0.0]),
    ] {
        let mut style = parse("--good:2px 3px;--bad:10%;border-spacing:7px 11px");
        assert!(set(&mut style, "border-spacing", value, ""));
        assert_eq!(get(&style, "border-spacing"), value);
        assert_eq!(parse(&text(&style)), style);
        let doc = crate::dom::Document::parse(&format!(
            "<div style='border-spacing:5px 9px'><i id=x style='{}'></i></div>",
            text(&style)
        ));
        let styles = css::compute_styles(&doc, &[], 320.0, 240.0);
        assert_eq!(
            styles[doc.query_selector("#x").unwrap()].border_spacing,
            expected,
            "{value}"
        );
    }
    let style = parse("border-spacing:7px 11px;all:initial");
    assert_eq!(get(&style, "border-spacing"), "initial");
    let doc =
        crate::dom::Document::parse(&format!("<table id=x style='{}'></table>", text(&style)));
    let styles = css::compute_styles(&doc, &[], 320.0, 240.0);
    assert_eq!(
        styles[doc.query_selector("#x").unwrap()].border_spacing,
        [0.0, 0.0]
    );
}

#[test]
fn border_spacing_cssom_border_shorthand_does_not_reset_spacing() {
    let mut style = parse("border-spacing:2px 3px!important;color:blue");
    for value in ["1px solid red", "initial", "none"] {
        assert!(set(&mut style, "border", value, "important"));
        assert_eq!(get(&style, "border-spacing"), "2px 3px");
        assert_eq!(
            style.get_property_priority("border-spacing", &mut WORK.clone()),
            Ok("important")
        );
        assert_eq!(parse(&text(&style)), style);
    }
    style.remove_property("border", &mut WORK.clone()).unwrap();
    assert_eq!(get(&style, "border-spacing"), "2px 3px");
    assert_eq!(get(&style, "color"), "blue");
    assert!(set(&mut style, "border-spacing", "", ""));
    assert_eq!(text(&style), "color: blue;");
}
