use super::*;
use crate::css::{self, BackgroundImage, ComputedStyle, SourceBase, StyleSource};
use crate::dom::Document;
use crate::graphics::Color;
use std::sync::Arc;

fn style(declarations: &str) -> ComputedStyle {
    let doc = Document::parse(&format!("<div id=x style='{declarations}'></div>"));
    css::compute_styles(&doc, &[], 80.0, 60.0)[doc.query_selector("#x").unwrap()].clone()
}
fn boxes(s: &ComputedStyle) -> (BackgroundBox, BackgroundBox) {
    (s.background_origin, s.background_clip)
}
fn supported(property: &str, value: &str) {
    assert!(
        css::supports_declaration(property, value),
        "{property}:{value}"
    );
    assert!(
        css::supports_matches(&format!("({property}:{value})")),
        "{property}:{value}"
    );
}

#[test]
fn defaults_noninheritance_and_explicit_copy_are_distinct() {
    use BackgroundBox::*;
    assert_eq!(boxes(&ComputedStyle::default()), (Padding, Border));
    let doc = Document::parse(
        "<div id=p style='background-origin:content-box;background-clip:padding-box'><div id=plain></div><div id=copy style='background-origin:inherit;background-clip:inherit'></div></div>",
    );
    let styles = css::compute_styles(&doc, &[], 80.0, 60.0);
    for (id, expected) in [
        ("#p", (Content, Padding)),
        ("#plain", (Padding, Border)),
        ("#copy", (Content, Padding)),
    ] {
        assert_eq!(boxes(&styles[doc.query_selector(id).unwrap()]), expected);
    }
}

#[test]
fn longhands_accept_complete_decoded_identifiers() {
    use BackgroundBox::*;
    for (value, expected) in [
        ("border-box", Border),
        ("padding-box", Padding),
        ("content-box", Content),
        (" BORDER-BOX ", Border),
        ("/**/padding-box/**/", Padding),
        (r"\70 adding-box", Padding),
        (r"\63 ontent-box", Content),
        (r"padd\69 ng-box", Padding),
    ] {
        assert_eq!(
            parse_box(value, &mut 100_000),
            Ok(Some(expected)),
            "{value}"
        );
        for name in ["background-origin", "background-clip"] {
            supported(name, value);
            let s = style(&format!("{name}:{value}"));
            assert_eq!(
                if name.ends_with("origin") {
                    s.background_origin
                } else {
                    s.background_clip
                },
                expected
            );
        }
    }
}

#[test]
fn shorthand_defaults_one_two_and_interleaved_boxes_are_literal() {
    use BackgroundBox::*;
    for (value, expected) in [
        ("red", (Padding, Border)),
        ("red content-box", (Content, Content)),
        ("padding-box", (Padding, Padding)),
        ("border-box url(\"A.png\")", (Border, Border)),
        ("none content-box", (Content, Content)),
        ("padding-box content-box", (Padding, Content)),
        ("content-box red padding-box", (Content, Padding)),
        ("border-box no-repeat border-box", (Border, Border)),
        ("content-box/**/red/**/padding-box", (Content, Padding)),
        (r"\63 ontent-box red padd\69 ng-box", (Content, Padding)),
    ] {
        supported("background", value);
        assert_eq!(
            boxes(&style(&format!("background:{value}"))),
            expected,
            "{value}"
        );
    }
    assert_eq!(
        parts("content-box red padding-box", &mut 100_000),
        Ok(Some([
            "red",
            "none",
            "auto auto",
            "0% 0%",
            "repeat",
            "content-box",
            "padding-box"
        ]))
    );
}

#[test]
fn maximum_thirteen_components_preserve_all_seven_slices() {
    let value = "linear-gradient(red,blue) right 2px bottom 3px / 4px 5px no-repeat repeat content-box padding-box red";
    assert_eq!(
        parts(value, &mut 1_000_000),
        Ok(Some([
            "red",
            "linear-gradient(red,blue)",
            "4px 5px",
            "right 2px bottom 3px",
            "no-repeat repeat",
            "content-box",
            "padding-box"
        ]))
    );
    supported("background", value);
    let s = style(&format!("background:{value}"));
    assert_eq!(boxes(&s), (BackgroundBox::Content, BackgroundBox::Padding));
    assert_eq!(s.background_color, Color::rgb(255, 0, 0));
    assert!(matches!(
        s.background_image,
        Some(BackgroundImage::Gradient(_))
    ));
    assert_eq!(
        s.background_size,
        BackgroundSize::Explicit {
            width: Length::Px(4.0),
            height: Length::Px(5.0)
        }
    );
    assert_eq!(s.background_repeat, [false, true]);
    assert_eq!(
        s.background_position,
        BackgroundPosition {
            x: PositionComponent {
                from_end: true,
                offset: Length::Px(2.0)
            },
            y: PositionComponent {
                from_end: true,
                offset: Length::Px(3.0)
            },
        }
    );
}

#[test]
fn invalid_longhands_preserve_earlier_winners_and_reject_syntax_normally() {
    for value in [
        "",
        "text",
        "border-area",
        "margin-box",
        "content-box padding-box",
        "content-box, padding-box",
        "calc(1px)",
        "padd/**/ing-box",
        "padding-box!",
        "url(",
        "[content-box)",
        "padding-box\0",
        "padding-box\\\n",
    ] {
        for name in ["background-origin", "background-clip"] {
            assert!(!css::supports_declaration(name, value), "{name}:{value:?}");
            assert!(
                !css::supports_matches(&format!("({name}:{value})")),
                "{name}:{value:?}"
            );
            assert_eq!(parse_box(value, &mut 100_000), Ok(None), "{value:?}");
        }
        let s = style(&format!(
            "background-origin:content-box;background-clip:padding-box;background-origin:{value};background-clip:{value}"
        ));
        assert_eq!(
            boxes(&s),
            (BackgroundBox::Content, BackgroundBox::Padding),
            "{value:?}"
        );
    }
}

#[test]
fn invalid_shorthands_never_publish_partial_box_or_color_state() {
    for value in [
        "content-box padding-box border-box",
        "text",
        "border-area",
        "margin-box",
        "content-box, padding-box",
        "content-box junk",
        "left / content-box",
        "content-box / 2px",
        "inherit content-box",
        "content-box initial",
        "content-box unset",
        "revert content-box",
        "content-box revert-layer",
        "content-box linear-gradient(",
    ] {
        assert_eq!(parts(value, &mut 1_000_000), Ok(None), "{value}");
        assert!(!css::supports_declaration("background", value), "{value}");
        assert!(
            !css::supports_matches(&format!("(background:{value})")),
            "{value}"
        );
        let s = style(&format!(
            "background:lime content-box padding-box;background:{value}"
        ));
        assert_eq!(
            boxes(&s),
            (BackgroundBox::Content, BackgroundBox::Padding),
            "{value}"
        );
        assert_eq!(s.background_color, Color::rgb(0, 255, 0), "{value}");
    }
}

#[test]
fn box_escape_exception_does_not_broaden_other_supports_values_or_names() {
    for (name, value) in [
        (r"background-\63 lip", "content-box"),
        ("color", r"r\65 d"),
        ("background", r"r\65 d content-box"),
        ("background-repeat", r"r\65 peat"),
        ("background-origin", r"\0 content-box"),
    ] {
        assert!(!css::supports_declaration(name, value), "{name}:{value}");
        assert!(
            !css::supports_matches(&format!("({name}:{value})")),
            "{name}:{value}"
        );
    }
    assert!(!css::supports_declaration(
        "object-position",
        "right 2px bottom"
    ));
    supported("background-position", "right 2px bottom");
}

#[test]
fn wide_and_all_reset_both_boxes_without_inheriting_them_by_default() {
    use BackgroundBox::*;
    for property in ["background", "all"] {
        for wide in ["initial", "unset", "revert", "revert-layer"] {
            let s = style(&format!(
                "background:lime content-box padding-box;{property}:{wide}"
            ));
            assert_eq!(boxes(&s), (Padding, Border), "{property}:{wide}");
        }
    }
    let doc = Document::parse(
        "<div style='background:lime content-box padding-box'><div id=a style='background:inherit'></div><div id=b style='all:inherit'></div></div>",
    );
    let styles = css::compute_styles(&doc, &[], 80.0, 60.0);
    for id in ["#a", "#b"] {
        assert_eq!(
            boxes(&styles[doc.query_selector(id).unwrap()]),
            (Content, Padding)
        );
    }
}

#[test]
fn pending_invalid_values_default_instead_of_resurrecting_old_boxes() {
    use BackgroundBox::*;
    assert_eq!(
        boxes(&style(
            "--bad:text;background-origin:border-box;background-origin:var(--bad);background-clip:content-box;background-clip:var(--missing)"
        )),
        (Padding, Border)
    );
    let s = style(
        "background:lime content-box;--bad:url(a.png) content-box padding-box border-box;background:var(--bad)",
    );
    assert_eq!(boxes(&s), (Padding, Border));
    assert_eq!(s.background_color, Color::TRANSPARENT);
    assert!(s.background_image.is_none());
    assert_eq!(s.background_size, BackgroundSize::default());
    assert_eq!(s.background_position, BackgroundPosition::default());
    assert_eq!(s.background_repeat, [true, true]);
}

#[test]
fn pending_shorthand_emits_seven_targets_and_valid_substitution() {
    let mut declarations = Vec::new();
    css::expand_declaration_with_budget(
        "background",
        "var(--bg)",
        true,
        &mut declarations,
        &mut 100_000,
    )
    .unwrap();
    assert_eq!(
        declarations
            .iter()
            .map(|d| d.name.as_str())
            .collect::<Vec<_>>(),
        [
            "background-color",
            "background-image",
            "background-size",
            "background-position",
            "background-repeat",
            "background-origin",
            "background-clip"
        ]
    );
    assert!(declarations.iter().all(|d| d.important
        && d.pending_shorthand.as_deref() == Some("background")
        && d.value == "var(--bg)"));
    let s = style(
        "--bg:linear-gradient(red,blue) right / 4px no-repeat content-box padding-box;background:var(--bg)",
    );
    assert_eq!(boxes(&s), (BackgroundBox::Content, BackgroundBox::Padding));
    assert!(matches!(
        s.background_image,
        Some(BackgroundImage::Gradient(_))
    ));
    assert_eq!(s.background_repeat, [false, false]);
}

#[test]
fn layer_rollback_importance_and_inline_rollback_keep_independent_winners() {
    use BackgroundBox::*;
    let doc = Document::parse(
        "<style>@layer old,new;@layer old{#normal,#inline{background:content-box padding-box}#important{background:content-box padding-box!important;background-origin:revert-layer!important}}@layer new{#normal{background-origin:border-box;background-origin:revert-layer;background-clip:content-box}#important{background:border-box content-box!important}}</style><div id=normal></div><div id=important></div><div id=inline style='background:revert-layer'></div>",
    );
    let styles = css::compute_styles(&doc, &doc.stylesheets(), 80.0, 60.0);
    for (id, expected) in [
        ("#normal", (Content, Content)),
        ("#important", (Padding, Padding)),
        ("#inline", (Content, Padding)),
    ] {
        assert_eq!(
            boxes(&styles[doc.query_selector(id).unwrap()]),
            expected,
            "{id}"
        );
    }
}

#[test]
fn url_payload_consuming_base_and_inherited_identity_survive_box_selection() {
    use BackgroundBox::*;
    let doc = Document::parse(
        "<div id=p><div id=c><div id=g style='background-clip:padding-box'></div></div></div><div id=same></div>",
    );
    let sources = [
        StyleSource::with_base(
            "#p,#same{--bg:url(\"Dir/content-box/**/Logo.png?Q=A B\") content-box padding-box}",
            SourceBase::Stylesheet(Arc::new(
                ::url::Url::parse("https://wrong.test/vars/").unwrap(),
            )),
        ),
        StyleSource::with_base(
            "#p,#same{background:var(--bg)}#c{background:inherit}#same{background-origin:border-box;background-clip:border-box}",
            SourceBase::Stylesheet(Arc::new(
                ::url::Url::parse("https://right.test/css/main.css").unwrap(),
            )),
        ),
    ];
    let styles = css::compute_styles_from_sources(&doc, &sources, 80.0, 60.0);
    let p = &styles[doc.query_selector("#p").unwrap()];
    let c = &styles[doc.query_selector("#c").unwrap()];
    let g = &styles[doc.query_selector("#g").unwrap()];
    let same = &styles[doc.query_selector("#same").unwrap()];
    assert_eq!(boxes(p), (Content, Padding));
    assert_eq!(boxes(c), (Content, Padding));
    assert_eq!(boxes(g), (Padding, Padding));
    assert!(g.background_image.is_none());
    let Some(BackgroundImage::Url(a)) = &p.background_image else {
        panic!("parent URL")
    };
    let Some(BackgroundImage::Url(b)) = &c.background_image else {
        panic!("inherited URL")
    };
    let Some(BackgroundImage::Url(d)) = &same.background_image else {
        panic!("same URL")
    };
    assert!(Arc::ptr_eq(a, b));
    assert!(Arc::ptr_eq(a, d));
    let css::url::ResolvedUrl::Absolute(url) = a.as_ref() else {
        panic!("absolute URL")
    };
    assert_eq!(
        url.as_str(),
        "https://right.test/css/Dir/content-box/**/Logo.png?Q=A%20B"
    );
}

#[test]
fn direct_parser_literal_work_cuts_and_raw_bounds_are_fixed() {
    for (value, expected, fee) in [
        ("border-box", BackgroundBox::Border, 59),
        ("padding-box", BackgroundBox::Padding, 64),
        ("content-box", BackgroundBox::Content, 64),
    ] {
        let mut work = fee;
        assert_eq!(parse_box(value, &mut work), Ok(Some(expected)));
        assert_eq!(work, 0);
        let mut short = fee - 1;
        assert_eq!(parse_box(value, &mut short), Err(Error::Limit));
        assert_eq!(short, 0);
    }
    assert_eq!(
        parse_box(&" ".repeat(4097), &mut 100_000),
        Err(Error::Limit)
    );
    assert_eq!(std::mem::size_of::<BackgroundBox>(), 1);
}

#[test]
fn seven_record_expansion_prepays_literal_storage_without_partial_publication() {
    // content-box: span416 + tokenizer23 + classification41 =480;
    // seven borrowed values total57 bytes + seven record reservations448 =505.
    let mut exact = 985;
    let mut declarations = Vec::new();
    css::expand_declaration_with_budget(
        "background",
        "content-box",
        false,
        &mut declarations,
        &mut exact,
    )
    .unwrap();
    assert_eq!(exact, 0);
    assert_eq!(declarations.len(), 7);
    assert_eq!(
        declarations
            .iter()
            .map(|d| d.value.as_str())
            .collect::<Vec<_>>(),
        [
            "transparent",
            "none",
            "auto auto",
            "0% 0%",
            "repeat",
            "content-box",
            "content-box"
        ]
    );
    let mut short = 984;
    let mut refused = Vec::new();
    assert_eq!(
        css::expand_declaration_with_budget(
            "background",
            "content-box",
            false,
            &mut refused,
            &mut short
        ),
        Err(Error::Limit)
    );
    assert!(refused.is_empty());
    assert_eq!(short, 0);
}

#[test]
fn a_single_calculation_is_a_valid_background_position_not_a_visual_box() {
    // Background position accepts one length; its omitted second axis centers.
    // The same function is still invalid for either visual-box longhand.
    supported("background", "calc(1px)");
    assert_eq!(
        parts("calc(1px)", &mut 100_000),
        Ok(Some([
            "transparent",
            "none",
            "auto auto",
            "calc(1px)",
            "repeat",
            "padding-box",
            "border-box",
        ]))
    );
    let s = style("background:lime content-box;padding:2px;background:calc(1px)");
    assert_eq!(boxes(&s), (BackgroundBox::Padding, BackgroundBox::Border));
    assert_eq!(s.background_color, Color::TRANSPARENT);
    assert!(s.background_image.is_none());
    assert_eq!(s.background_size, BackgroundSize::default());
    assert_eq!(s.background_repeat, [true, true]);
    assert_eq!(
        s.background_position,
        BackgroundPosition {
            x: PositionComponent {
                from_end: false,
                offset: Length::Calc {
                    px: 1.0,
                    percent: 0.0,
                    has_percent: false
                }
            },
            y: PositionComponent {
                from_end: false,
                offset: Length::Percent(50.0)
            },
        }
    );
    for property in ["background-origin", "background-clip"] {
        assert!(!css::supports_declaration(property, "calc(1px)"));
    }
}
