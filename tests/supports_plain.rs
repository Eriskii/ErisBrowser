//! Primitive font-size and escaped condition-name controls frozen before producer inspection.
//! Deferred font keywords, math, aliases and escaped numeric units are outside this slice.
use eris::{
    css::{self, Length},
    page::Page,
    script::Value,
};
use url::Url;

fn pairs(rows: &[(&str, &str, &str, bool)]) {
    for &(id, property, value, expected) in rows {
        assert_eq!(
            css::supports_declaration(property, value),
            expected,
            "id={id}, property={property:?}, value={value:?}"
        );
    }
}

fn conditions(rows: &[(&str, &str, bool)]) {
    for &(id, query, expected) in rows {
        assert_eq!(
            css::supports_matches(query),
            expected,
            "id={id}, query={query:?}"
        );
    }
}

fn runtime(id: &str, setup: &str, source: &str, expected: bool) {
    // A fresh Page for every source and mode avoids assumptions about combined query budgets.
    for strict in [false, true] {
        let mut page = Page::from_html(
            Url::parse("https://supports-plain.invalid/").unwrap(),
            setup,
            false,
        );
        let result = if strict {
            page.runtime.execute_strict(source, &mut page.document)
        } else {
            page.runtime.execute(source, &mut page.document)
        };
        assert_eq!(
            result.unwrap(),
            Value::Bool(expected),
            "id={id}, strict={strict}"
        );
        assert!(
            page.diagnostics.is_empty(),
            "id={id}: {:?}",
            page.diagnostics
        );
    }
}

#[test]
fn supports_plain_primitive_font_values() {
    pairs(&[
        (
            r###"font-numeric-1"###,
            r###"font-size"###,
            r###"12px"###,
            true,
        ),
        (
            r###"font-numeric-2"###,
            r###"font-size"###,
            r###"12PX"###,
            true,
        ),
        (
            r###"font-numeric-3"###,
            r###"font-size"###,
            r###"1.5em"###,
            true,
        ),
        (
            r###"font-numeric-4"###,
            r###"font-size"###,
            r###"125%"###,
            true,
        ),
        (
            r###"font-numeric-5"###,
            r###"font-size"###,
            r###"0"###,
            true,
        ),
        (
            r###"font-numeric-6"###,
            r###"font-size"###,
            r###"+0"###,
            true,
        ),
        (
            r###"font-numeric-7"###,
            r###"font-size"###,
            r###"-0"###,
            true,
        ),
        (
            r###"font-numeric-8"###,
            r###"font-size"###,
            r###"1e1px"###,
            true,
        ),
        (
            r###"font-numeric-9"###,
            r###"font-size"###,
            r###"0.5rem"###,
            true,
        ),
        (
            r###"font-numeric-10"###,
            r###"font-size"###,
            r###"2vw"###,
            true,
        ),
    ]);
}

#[test]
fn supports_plain_font_css_wide_keywords() {
    pairs(&[
        (
            r###"font-wide-1"###,
            r###"font-size"###,
            r###"initial"###,
            true,
        ),
        (
            r###"font-wide-2"###,
            r###"font-size"###,
            r###"inherit"###,
            true,
        ),
        (
            r###"font-wide-3"###,
            r###"font-size"###,
            r###"unset"###,
            true,
        ),
        (
            r###"font-wide-4"###,
            r###"font-size"###,
            r###"revert"###,
            true,
        ),
        (
            r###"font-wide-5"###,
            r###"font-size"###,
            r###"revert-layer"###,
            true,
        ),
    ]);
}

#[test]
fn supports_plain_invalid_primitive_values() {
    pairs(&[
        (
            r###"font-invalid-1"###,
            r###"font-size"###,
            r###"-1px"###,
            false,
        ),
        (
            r###"font-invalid-2"###,
            r###"font-size"###,
            r###"-1%"###,
            false,
        ),
        (
            r###"font-invalid-3"###,
            r###"font-size"###,
            r###"12"###,
            false,
        ),
        (
            r###"font-invalid-4"###,
            r###"font-size"###,
            r###"12 px"###,
            false,
        ),
        (
            r###"font-invalid-5"###,
            r###"font-size"###,
            r###"12px 13px"###,
            false,
        ),
        (
            r###"font-invalid-6"###,
            r###"font-size"###,
            r###"12px;"###,
            false,
        ),
        (
            r###"font-invalid-7"###,
            r###"font-size"###,
            r###"12px !important"###,
            false,
        ),
        (
            r###"font-invalid-8"###,
            r###"font-size"###,
            r###""12px""###,
            false,
        ),
        (
            r###"font-invalid-9"###,
            r###"font-size"###,
            r###"NaNpx"###,
            false,
        ),
        (
            r###"font-invalid-10"###,
            r###"font-size"###,
            r###"1/**/px"###,
            false,
        ),
        (
            r###"font-invalid-11"###,
            r###"font-size"###,
            r###"auto"###,
            false,
        ),
        (
            r###"font-invalid-12"###,
            r###"font-size"###,
            r###"none"###,
            false,
        ),
    ]);
}

#[test]
fn supports_plain_literal_property_names() {
    pairs(&[
        (
            r###"property-case"###,
            r###"FONT-SIZE"###,
            r###"12px"###,
            true,
        ),
        (
            r###"literal-name-no-trim"###,
            r###" font-size"###,
            r###"12px"###,
            false,
        ),
        (
            r###"literal-name-no-decode"###,
            r###"font\2d size"###,
            r###"12px"###,
            false,
        ),
        (
            r###"literal-width-name-no-decode"###,
            r###"w\69 dth"###,
            r###"12px"###,
            false,
        ),
    ]);
}

#[test]
fn supports_plain_font_conditions() {
    conditions(&[
        (r###"font-condition"###, r###"(font-size:12px)"###, true),
        (
            r###"font-condition-priority"###,
            r###"(font-size:12px !important)"###,
            true,
        ),
        (
            r###"font-negated-invalid"###,
            r###"not (font-size:-1px)"###,
            true,
        ),
        (
            r###"font-negated-valid"###,
            r###"not (font-size:12px)"###,
            false,
        ),
    ]);
}

#[test]
fn supports_plain_escaped_condition_names_and_boolean_structure() {
    conditions(&[
        (r###"width-escaped"###, r###"(w\69 dth:12px)"###, true),
        (r###"width-escaped-upper"###, r###"(\57 IDTH:12px)"###, true),
        (r###"font-escaped"###, r###"(font\2d size:12px)"###, true),
        (
            r###"display-escaped-existing"###,
            r###"(d\69 splay:flex)"###,
            true,
        ),
        (
            r###"escaped-priority"###,
            r###"(w\69 dth:12px !important)"###,
            true,
        ),
        (
            r###"escaped-name-comment"###,
            r###"(w\69 dth/**/:12px)"###,
            true,
        ),
        (
            r###"escaped-invalid-value"###,
            r###"(w\69 dth:-1px)"###,
            false,
        ),
        (r###"escaped-unknown"###, r###"(un\6b nown:12px)"###, false),
        (
            r###"escaped-property-token-split"###,
            r###"(w/**/idth:12px)"###,
            false,
        ),
        (
            r###"escaped-name-junk"###,
            r###"(w\69 dth junk:12px)"###,
            false,
        ),
        (
            r###"mixed-boolean"###,
            r###"(font-size:12px) and (w\69 dth:12px)"###,
            true,
        ),
        (
            r###"mixed-negated-invalid"###,
            r###"(font-size:12px) and (not (w\69 dth:-1px))"###,
            true,
        ),
    ]);
}

#[test]
fn supports_plain_implied_forms_require_runtime_parentheses_rule() {
    conditions(&[
        (r###"implied-font"###, r###"font-size:12px"###, false),
        (
            r###"implied-escaped-width"###,
            r###"w\69 dth:12px"###,
            false,
        ),
    ]);
}

#[test]
fn supports_plain_numeric_sign_and_zero() {
    pairs(&[
        (r###"numeric-1"###, r###"font-size"###, r###"0"###, true),
        (r###"numeric-2"###, r###"font-size"###, r###"+0"###, true),
        (r###"numeric-3"###, r###"font-size"###, r###"-0"###, true),
        (
            r###"numeric-4"###,
            r###"font-size"###,
            r###"-0.000e+9"###,
            true,
        ),
        (
            r###"numeric-5"###,
            r###"font-size"###,
            r###"0e-1000"###,
            true,
        ),
        (
            r###"numeric-6"###,
            r###"font-size"###,
            r###"-1e-1000px"###,
            false,
        ),
        (
            r###"numeric-7"###,
            r###"font-size"###,
            r###"-1e-1000%"###,
            false,
        ),
        (
            r###"numeric-8"###,
            r###"font-size"###,
            r###"1e-1000"###,
            false,
        ),
        (
            r###"numeric-9"###,
            r###"font-size"###,
            r###"-1e-1000"###,
            false,
        ),
        (
            r###"numeric-10"###,
            r###"font-size"###,
            r###"1e-1000px"###,
            true,
        ),
        (
            r###"numeric-11"###,
            r###"font-size"###,
            r###"1e-1000%"###,
            true,
        ),
        (
            r###"numeric-12"###,
            r###"font-size"###,
            r###"-0e+9px"###,
            true,
        ),
        (
            r###"numeric-13"###,
            r###"font-size"###,
            r###"+0.0e-1000%"###,
            true,
        ),
    ]);
    conditions(&[
        (r###"numeric-1"###, r###"(font-size:0)"###, true),
        (r###"numeric-2"###, r###"(font-size:+0)"###, true),
        (r###"numeric-3"###, r###"(font-size:-0)"###, true),
        (r###"numeric-4"###, r###"(font-size:-0.000e+9)"###, true),
        (r###"numeric-5"###, r###"(font-size:0e-1000)"###, true),
        (r###"numeric-6"###, r###"(font-size:-1e-1000px)"###, false),
        (r###"numeric-7"###, r###"(font-size:-1e-1000%)"###, false),
        (r###"numeric-8"###, r###"(font-size:1e-1000)"###, false),
        (r###"numeric-9"###, r###"(font-size:-1e-1000)"###, false),
        (r###"numeric-10"###, r###"(font-size:1e-1000px)"###, true),
        (r###"numeric-11"###, r###"(font-size:1e-1000%)"###, true),
        (r###"numeric-12"###, r###"(font-size:-0e+9px)"###, true),
        (r###"numeric-13"###, r###"(font-size:+0.0e-1000%)"###, true),
    ]);
}

#[test]
fn supports_plain_runtime_selected_rows() {
    for (id, source, expected) in [
        (
            r###"font-numeric-1"###,
            r###"CSS.supports("font-size", "12px");
"###,
            true,
        ),
        (
            r###"font-numeric-2"###,
            r###"CSS.supports("font-size", "12PX");
"###,
            true,
        ),
        (
            r###"font-numeric-3"###,
            r###"CSS.supports("font-size", "1.5em");
"###,
            true,
        ),
        (
            r###"font-numeric-4"###,
            r###"CSS.supports("font-size", "125%");
"###,
            true,
        ),
        (
            r###"font-numeric-5"###,
            r###"CSS.supports("font-size", "0");
"###,
            true,
        ),
        (
            r###"font-numeric-6"###,
            r###"CSS.supports("font-size", "+0");
"###,
            true,
        ),
        (
            r###"font-numeric-7"###,
            r###"CSS.supports("font-size", "-0");
"###,
            true,
        ),
        (
            r###"font-numeric-8"###,
            r###"CSS.supports("font-size", "1e1px");
"###,
            true,
        ),
        (
            r###"font-numeric-9"###,
            r###"CSS.supports("font-size", "0.5rem");
"###,
            true,
        ),
        (
            r###"font-numeric-10"###,
            r###"CSS.supports("font-size", "2vw");
"###,
            true,
        ),
        (
            r###"font-wide-1"###,
            r###"CSS.supports("font-size", "initial");
"###,
            true,
        ),
        (
            r###"font-wide-2"###,
            r###"CSS.supports("font-size", "inherit");
"###,
            true,
        ),
        (
            r###"font-wide-3"###,
            r###"CSS.supports("font-size", "unset");
"###,
            true,
        ),
        (
            r###"font-wide-4"###,
            r###"CSS.supports("font-size", "revert");
"###,
            true,
        ),
        (
            r###"font-wide-5"###,
            r###"CSS.supports("font-size", "revert-layer");
"###,
            true,
        ),
        (
            r###"font-invalid-1"###,
            r###"CSS.supports("font-size", "-1px");
"###,
            false,
        ),
        (
            r###"font-invalid-2"###,
            r###"CSS.supports("font-size", "-1%");
"###,
            false,
        ),
        (
            r###"font-invalid-3"###,
            r###"CSS.supports("font-size", "12");
"###,
            false,
        ),
        (
            r###"font-invalid-4"###,
            r###"CSS.supports("font-size", "12 px");
"###,
            false,
        ),
        (
            r###"font-invalid-5"###,
            r###"CSS.supports("font-size", "12px 13px");
"###,
            false,
        ),
        (
            r###"font-invalid-6"###,
            r###"CSS.supports("font-size", "12px;");
"###,
            false,
        ),
        (
            r###"font-invalid-7"###,
            r###"CSS.supports("font-size", "12px !important");
"###,
            false,
        ),
        (
            r###"font-invalid-8"###,
            r###"CSS.supports("font-size", "\"12px\"");
"###,
            false,
        ),
        (
            r###"font-invalid-9"###,
            r###"CSS.supports("font-size", "NaNpx");
"###,
            false,
        ),
        (
            r###"font-invalid-10"###,
            r###"CSS.supports("font-size", "1/**/px");
"###,
            false,
        ),
        (
            r###"font-invalid-11"###,
            r###"CSS.supports("font-size", "auto");
"###,
            false,
        ),
        (
            r###"font-invalid-12"###,
            r###"CSS.supports("font-size", "none");
"###,
            false,
        ),
        (
            r###"property-case"###,
            r###"CSS.supports("FONT-SIZE", "12px");
"###,
            true,
        ),
        (
            r###"literal-name-no-trim"###,
            r###"CSS.supports(" font-size", "12px");
"###,
            false,
        ),
        (
            r###"literal-name-no-decode"###,
            r###"CSS.supports("font\\2d size", "12px");
"###,
            false,
        ),
        (
            r###"literal-width-name-no-decode"###,
            r###"CSS.supports("w\\69 dth", "12px");
"###,
            false,
        ),
        (
            r###"font-condition"###,
            r###"CSS.supports("(font-size:12px)");
"###,
            true,
        ),
        (
            r###"font-condition-priority"###,
            r###"CSS.supports("(font-size:12px !important)");
"###,
            true,
        ),
        (
            r###"font-negated-invalid"###,
            r###"CSS.supports("not (font-size:-1px)");
"###,
            true,
        ),
        (
            r###"font-negated-valid"###,
            r###"CSS.supports("not (font-size:12px)");
"###,
            false,
        ),
        (
            r###"width-escaped"###,
            r###"CSS.supports("(w\\69 dth:12px)");
"###,
            true,
        ),
        (
            r###"width-escaped-upper"###,
            r###"CSS.supports("(\\57 IDTH:12px)");
"###,
            true,
        ),
        (
            r###"font-escaped"###,
            r###"CSS.supports("(font\\2d size:12px)");
"###,
            true,
        ),
        (
            r###"display-escaped-existing"###,
            r###"CSS.supports("(d\\69 splay:flex)");
"###,
            true,
        ),
        (
            r###"escaped-priority"###,
            r###"CSS.supports("(w\\69 dth:12px !important)");
"###,
            true,
        ),
        (
            r###"escaped-name-comment"###,
            r###"CSS.supports("(w\\69 dth/**/:12px)");
"###,
            true,
        ),
        (
            r###"escaped-invalid-value"###,
            r###"CSS.supports("(w\\69 dth:-1px)");
"###,
            false,
        ),
        (
            r###"escaped-unknown"###,
            r###"CSS.supports("(un\\6b nown:12px)");
"###,
            false,
        ),
        (
            r###"escaped-property-token-split"###,
            r###"CSS.supports("(w/**/idth:12px)");
"###,
            false,
        ),
        (
            r###"escaped-name-junk"###,
            r###"CSS.supports("(w\\69 dth junk:12px)");
"###,
            false,
        ),
        (
            r###"mixed-boolean"###,
            r###"CSS.supports("(font-size:12px) and (w\\69 dth:12px)");
"###,
            true,
        ),
        (
            r###"mixed-negated-invalid"###,
            r###"CSS.supports("(font-size:12px) and (not (w\\69 dth:-1px))");
"###,
            true,
        ),
        (
            r###"implied-font"###,
            r###"CSS.supports("font-size:12px");
"###,
            true,
        ),
        (
            r###"implied-escaped-width"###,
            r###"CSS.supports("w\\69 dth:12px");
"###,
            true,
        ),
    ] {
        runtime(id, "<div id=target></div>", source, expected);
    }
}

#[test]
fn supports_plain_runtime_numeric_both_forms() {
    for (id, source, expected) in [
        (
            r###"numeric-1-pair"###,
            r###"CSS.supports("font-size", "0");
"###,
            true,
        ),
        (
            r###"numeric-1-condition"###,
            r###"CSS.supports("(font-size:0)");
"###,
            true,
        ),
        (
            r###"numeric-2-pair"###,
            r###"CSS.supports("font-size", "+0");
"###,
            true,
        ),
        (
            r###"numeric-2-condition"###,
            r###"CSS.supports("(font-size:+0)");
"###,
            true,
        ),
        (
            r###"numeric-3-pair"###,
            r###"CSS.supports("font-size", "-0");
"###,
            true,
        ),
        (
            r###"numeric-3-condition"###,
            r###"CSS.supports("(font-size:-0)");
"###,
            true,
        ),
        (
            r###"numeric-4-pair"###,
            r###"CSS.supports("font-size", "-0.000e+9");
"###,
            true,
        ),
        (
            r###"numeric-4-condition"###,
            r###"CSS.supports("(font-size:-0.000e+9)");
"###,
            true,
        ),
        (
            r###"numeric-5-pair"###,
            r###"CSS.supports("font-size", "0e-1000");
"###,
            true,
        ),
        (
            r###"numeric-5-condition"###,
            r###"CSS.supports("(font-size:0e-1000)");
"###,
            true,
        ),
        (
            r###"numeric-6-pair"###,
            r###"CSS.supports("font-size", "-1e-1000px");
"###,
            false,
        ),
        (
            r###"numeric-6-condition"###,
            r###"CSS.supports("(font-size:-1e-1000px)");
"###,
            false,
        ),
        (
            r###"numeric-7-pair"###,
            r###"CSS.supports("font-size", "-1e-1000%");
"###,
            false,
        ),
        (
            r###"numeric-7-condition"###,
            r###"CSS.supports("(font-size:-1e-1000%)");
"###,
            false,
        ),
        (
            r###"numeric-8-pair"###,
            r###"CSS.supports("font-size", "1e-1000");
"###,
            false,
        ),
        (
            r###"numeric-8-condition"###,
            r###"CSS.supports("(font-size:1e-1000)");
"###,
            false,
        ),
        (
            r###"numeric-9-pair"###,
            r###"CSS.supports("font-size", "-1e-1000");
"###,
            false,
        ),
        (
            r###"numeric-9-condition"###,
            r###"CSS.supports("(font-size:-1e-1000)");
"###,
            false,
        ),
        (
            r###"numeric-10-pair"###,
            r###"CSS.supports("font-size", "1e-1000px");
"###,
            true,
        ),
        (
            r###"numeric-10-condition"###,
            r###"CSS.supports("(font-size:1e-1000px)");
"###,
            true,
        ),
        (
            r###"numeric-11-pair"###,
            r###"CSS.supports("font-size", "1e-1000%");
"###,
            true,
        ),
        (
            r###"numeric-11-condition"###,
            r###"CSS.supports("(font-size:1e-1000%)");
"###,
            true,
        ),
        (
            r###"numeric-12-pair"###,
            r###"CSS.supports("font-size", "-0e+9px");
"###,
            true,
        ),
        (
            r###"numeric-12-condition"###,
            r###"CSS.supports("(font-size:-0e+9px)");
"###,
            true,
        ),
        (
            r###"numeric-13-pair"###,
            r###"CSS.supports("font-size", "+0.0e-1000%");
"###,
            true,
        ),
        (
            r###"numeric-13-condition"###,
            r###"CSS.supports("(font-size:+0.0e-1000%)");
"###,
            true,
        ),
    ] {
        runtime(id, "<div id=target></div>", source, expected);
    }
}

#[test]
fn supports_plain_stylesheet_font_fixture() {
    let page = Page::from_html(
        Url::parse("https://supports-plain.invalid/").unwrap(),
        r###"<!doctype html><style>#parent{font-size:20px}#target{font-size:12px}@supports (font-size:12px){#target{width:7px}}</style><div id=parent><div id=target></div></div>"###,
        false,
    );
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    let target = page.document.query_selector("#target").unwrap();
    let styles =
        css::compute_styles_from_sources(&page.document, &page.stylesheets(), 800.0, 600.0);
    assert_eq!(styles[target].font_size, 12.0);
    assert_eq!(styles[target].width, Length::Px(7.0));
}

#[test]
fn supports_plain_stylesheet_percentage_fixture() {
    let page = Page::from_html(
        Url::parse("https://supports-plain.invalid/").unwrap(),
        r###"<!doctype html><style>#parent{font-size:20px}#target{font-size:125%}@supports (font-size:125%){#target{width:9px}}</style><div id=parent><div id=target></div></div>"###,
        false,
    );
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    let target = page.document.query_selector("#target").unwrap();
    let styles =
        css::compute_styles_from_sources(&page.document, &page.stylesheets(), 800.0, 600.0);
    assert_eq!(styles[target].font_size, 25.0);
    assert_eq!(styles[target].width, Length::Px(9.0));
}

#[test]
fn supports_plain_stylesheet_escaped_name_fixture() {
    let page = Page::from_html(
        Url::parse("https://supports-plain.invalid/").unwrap(),
        r###"<!doctype html><style>#target{w\69 dth:12px}@supports (w\69 dth:12px){#target{height:7px}}</style><div id=target></div>"###,
        false,
    );
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    let target = page.document.query_selector("#target").unwrap();
    let styles =
        css::compute_styles_from_sources(&page.document, &page.stylesheets(), 800.0, 600.0);
    assert_eq!(styles[target].width, Length::Px(12.0));
    assert_eq!(styles[target].height, Length::Px(7.0));
}

#[test]
fn supports_plain_cssom_query_parity_fixture() {
    runtime(
        r###"CSSOM-query-parity"###,
        r###"<div id=target></div>"###,
        r###"var target=document.querySelector("#target"); target.style.setProperty("font-size","12px"); target.style.getPropertyValue("font-size") === "12px" && CSS.supports("font-size","12px");"###,
        true,
    );
}
