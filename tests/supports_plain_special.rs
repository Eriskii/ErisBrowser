//! Selected existing value validators reached through decoded condition property names.
//! These controls preserve raw token boundaries and literal two-argument name refusal.
use eris::{css, page::Page, script::Value};
use url::Url;

fn runtime(id: &str, source: &str, expected: bool) {
    // Every row and mode gets a fresh Page and the ordinary public execution allowance.
    for strict in [false, true] {
        let mut page = Page::from_html(
            Url::parse("https://supports-plain-special.invalid/").unwrap(),
            "<div id=target></div>",
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

fn conditions(rows: &[(&str, &str, &str, bool)]) {
    for &(id, condition, source, expected) in rows {
        assert_eq!(
            css::supports_matches(condition),
            expected,
            "id={id}, condition={condition:?}"
        );
        runtime(id, source, expected);
    }
}

#[test]
fn supports_plain_special_background_image() {
    conditions(&[
        (
            r###"background-image-none"###,
            r###"(background\2d image:none)"###,
            r###"CSS.supports("(background\\2d image:none)");"###,
            true,
        ),
        (
            r###"background-image-quoted-url"###,
            r###"(background\2d image:url("asset.png"))"###,
            r###"CSS.supports("(background\\2d image:url(\"asset.png\"))");"###,
            true,
        ),
        (
            r###"background-image-quoted-comment-priority"###,
            r###"(background\2d image:url("a/**/b.png") !important)"###,
            r###"CSS.supports("(background\\2d image:url(\"a/**/b.png\") !important)");"###,
            true,
        ),
        (
            r###"background-image-split-url-function"###,
            r###"(background\2d image:url/**/("asset.png"))"###,
            r###"CSS.supports("(background\\2d image:url/**/(\"asset.png\"))");"###,
            false,
        ),
        (
            r###"background-image-gradient"###,
            r###"(background\2d image:linear-gradient(red,blue))"###,
            r###"CSS.supports("(background\\2d image:linear-gradient(red,blue))");"###,
            true,
        ),
        (
            r###"background-image-bad-gradient"###,
            r###"(background\2d image:linear-gradient(red,,blue))"###,
            r###"CSS.supports("(background\\2d image:linear-gradient(red,,blue))");"###,
            false,
        ),
        (
            r###"background-image-bad-priority"###,
            r###"(background\2d image:none !urgent)"###,
            r###"CSS.supports("(background\\2d image:none !urgent)");"###,
            false,
        ),
        (
            r###"background-image-extra-declaration"###,
            r###"(background\2d image:none; color:red)"###,
            r###"CSS.supports("(background\\2d image:none; color:red)");"###,
            false,
        ),
    ]);
}

#[test]
fn supports_plain_special_border_spacing() {
    conditions(&[
        (
            r###"border-spacing-pair"###,
            r###"(border\2d spacing:2px 3px)"###,
            r###"CSS.supports("(border\\2d spacing:2px 3px)");"###,
            true,
        ),
        (
            r###"border-spacing-separated-tokens"###,
            r###"(border\2d spacing:2px/**/3px)"###,
            r###"CSS.supports("(border\\2d spacing:2px/**/3px)");"###,
            true,
        ),
        (
            r###"border-spacing-calc-priority"###,
            r###"(border\2d spacing:calc(1px + 2px) 4px !important)"###,
            r###"CSS.supports("(border\\2d spacing:calc(1px + 2px) 4px !important)");"###,
            true,
        ),
        (
            r###"border-spacing-split-dimension"###,
            r###"(border\2d spacing:2/**/px)"###,
            r###"CSS.supports("(border\\2d spacing:2/**/px)");"###,
            false,
        ),
        (
            r###"border-spacing-comment-cannot-supply-math-space"###,
            r###"(border\2d spacing:calc(1px/**/+/**/2px))"###,
            r###"CSS.supports("(border\\2d spacing:calc(1px/**/+/**/2px))");"###,
            false,
        ),
        (
            r###"border-spacing-extra-component"###,
            r###"(border\2d spacing:2px 3px 4px)"###,
            r###"CSS.supports("(border\\2d spacing:2px 3px 4px)");"###,
            false,
        ),
        (
            r###"border-spacing-bad-priority"###,
            r###"(border\2d spacing:2px !urgent)"###,
            r###"CSS.supports("(border\\2d spacing:2px !urgent)");"###,
            false,
        ),
        (
            r###"border-spacing-existing-value-escape"###,
            r###"(border\2d spacing:2p\78  3PX)"###,
            r###"CSS.supports("(border\\2d spacing:2p\\78  3PX)");"###,
            true,
        ),
    ]);
}

#[test]
fn supports_plain_special_object_position() {
    conditions(&[
        (
            r###"object-position-keywords"###,
            r###"(object\2d position:left top)"###,
            r###"CSS.supports("(object\\2d position:left top)");"###,
            true,
        ),
        (
            r###"object-position-edges-priority"###,
            r###"(object\2d position:right 5px bottom 2px !important)"###,
            r###"CSS.supports("(object\\2d position:right 5px bottom 2px !important)");"###,
            true,
        ),
        (
            r###"object-position-calculation"###,
            r###"(object\2d position:calc(1px + 2px) 20%)"###,
            r###"CSS.supports("(object\\2d position:calc(1px + 2px) 20%)");"###,
            true,
        ),
        (
            r###"object-position-split-keyword"###,
            r###"(object\2d position:le/**/ft top)"###,
            r###"CSS.supports("(object\\2d position:le/**/ft top)");"###,
            false,
        ),
        (
            r###"object-position-comment-cannot-supply-math-space"###,
            r###"(object\2d position:calc(1px/**/+/**/2px) 20%)"###,
            r###"CSS.supports("(object\\2d position:calc(1px/**/+/**/2px) 20%)");"###,
            false,
        ),
        (
            r###"object-position-wrong-axes"###,
            r###"(object\2d position:left right)"###,
            r###"CSS.supports("(object\\2d position:left right)");"###,
            false,
        ),
        (
            r###"object-position-bad-priority"###,
            r###"(object\2d position:left top !urgent)"###,
            r###"CSS.supports("(object\\2d position:left top !urgent)");"###,
            false,
        ),
        (
            r###"object-position-old-value-escape-refusal"###,
            r###"(object\2d position:l\65 ft top)"###,
            r###"CSS.supports("(object\\2d position:l\\65 ft top)");"###,
            false,
        ),
    ]);
}

#[test]
fn supports_plain_special_literal_property_names() {
    for (id, property, value, source) in [
        (
            r###"background-image-literal-two-argument-name"###,
            r###"background\2d image"###,
            r###"none"###,
            r###"CSS.supports("background\\2d image", "none");"###,
        ),
        (
            r###"border-spacing-literal-two-argument-name"###,
            r###"border\2d spacing"###,
            r###"2px 3px"###,
            r###"CSS.supports("border\\2d spacing", "2px 3px");"###,
        ),
        (
            r###"object-position-literal-two-argument-name"###,
            r###"object\2d position"###,
            r###"left top"###,
            r###"CSS.supports("object\\2d position", "left top");"###,
        ),
    ] {
        assert!(
            !css::supports_declaration(property, value),
            "id={id}, property={property:?}, value={value:?}"
        );
        runtime(id, source, false);
    }
}
