use super::*;
use crate::css::{self, BackgroundImage, ComputedStyle, SourceBase, StyleSource};
use crate::dom::Document;
use crate::graphics::Color;
use std::sync::Arc;

fn style(value: &str) -> ComputedStyle {
    let doc = Document::parse(&format!("<div id=x style='{value}'></div>"));
    css::compute_styles(&doc, &[], 80.0, 60.0)[doc.query_selector("#x").unwrap()].clone()
}
fn supports(name: &str, value: &str, expected: bool) {
    assert_eq!(
        css::supports_declaration(name, value),
        expected,
        "{name}: {value}"
    );
    assert_eq!(
        css::supports_matches(&format!("({name}:{value})")),
        expected
    );
    assert_eq!(css::supports_property(name, value), expected);
}

#[test]
fn independent_lengths_preserve_none_slots_and_cycle_only_at_use() {
    let s = style(
        "background-image:none,linear-gradient(red),none;background-size:4px 2px,cover;background-position:2px 0px;background-repeat:repeat-x,no-repeat,repeat-y,repeat;background-origin:content-box,padding-box;background-clip:padding-box,content-box",
    );
    let b = &s.backgrounds;
    assert_eq!(
        [
            b.image_count(),
            b.sizes().len(),
            b.positions().len(),
            b.repeats().len(),
            b.origins().len(),
            b.clips().len()
        ],
        [3, 2, 1, 4, 2, 2]
    );
    assert!(b.image_slots()[0].is_none() && b.image_slots()[2].is_none());
    assert!(matches!(
        &b.image_slots()[1],
        Some(BackgroundImage::Gradient(_))
    ));
    for (i, (repeat, origin, clip)) in [
        (
            [true, false],
            BackgroundBox::Content,
            BackgroundBox::Padding,
        ),
        (
            [false, false],
            BackgroundBox::Padding,
            BackgroundBox::Content,
        ),
        (
            [false, true],
            BackgroundBox::Content,
            BackgroundBox::Padding,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let layer = b.layer(i).unwrap();
        assert_eq!(
            (layer.index, layer.repeat, layer.origin, layer.clip),
            (i, repeat, origin, clip)
        );
        assert_eq!(layer.position.x.offset, Length::Px(2.0));
    }
    assert_eq!(b.layer(2).unwrap().size, b.sizes()[0]);
    assert!(b.layer(3).is_none());
    assert_eq!(b.bottom_clip(), BackgroundBox::Padding);
}

#[test]
fn equal_repeated_values_keep_authored_cardinality_and_single_none_identity() {
    let one = style("background-image:none");
    let two = style(
        "background-image:none,none;background-repeat:repeat,repeat,repeat;background-clip:content-box",
    );
    assert!(one.backgrounds.is_single_none());
    assert!(!two.backgrounds.is_single_none());
    assert_eq!(two.backgrounds.image_count(), 2);
    assert_eq!(two.backgrounds.repeats().len(), 3);
    assert_eq!(two.backgrounds.bottom_clip(), BackgroundBox::Content);
}

#[test]
fn shorthand_expands_seven_targets_with_per_layer_defaults_and_final_color() {
    let declarations = css::parse_declarations(
        "background:none content-box,linear-gradient(red,blue) right 2px bottom / 4px 2px no-repeat padding-box border-box lime!important",
    );
    assert_eq!(declarations.len(), 7);
    for (d, (name, value)) in declarations.iter().zip([
        ("background-color", "lime"),
        ("background-image", "none, linear-gradient(red,blue)"),
        ("background-size", "auto auto, 4px 2px"),
        ("background-position", "0% 0%, right 2px bottom"),
        ("background-repeat", "repeat, no-repeat"),
        ("background-origin", "content-box, padding-box"),
        ("background-clip", "content-box, border-box"),
    ]) {
        assert_eq!((&*d.name, &*d.value), (name, value));
        assert!(d.important);
    }
    let s = style("background:none content-box,linear-gradient(red,blue) lime");
    assert_eq!(s.background_color, Color::rgb(0, 255, 0));
    assert_eq!(
        s.backgrounds.origins(),
        [BackgroundBox::Content, BackgroundBox::Padding]
    );
    assert_eq!(
        s.backgrounds.clips(),
        [BackgroundBox::Content, BackgroundBox::Border]
    );
}

#[test]
fn exactly_eight_layers_are_admitted_ninth_and_empty_items_reject_atomically() {
    let eight = ["none"; 8].join(",");
    let nine = ["none"; 9].join(",");
    for name in ["background", "background-image"] {
        supports(name, &eight, true);
        supports(name, &nine, false);
        for bad in [
            "none,",
            ",none",
            "none,,none",
            "none, initial",
            "initial,none",
        ] {
            supports(name, bad, false);
        }
        let s = style(&format!("{name}:linear-gradient(red);{name}:{nine}"));
        assert_eq!(s.backgrounds.image_count(), 1);
        assert!(matches!(
            &s.backgrounds.image_slots()[0],
            Some(BackgroundImage::Gradient(_))
        ));
    }
    assert_eq!(
        style(&format!("background:{eight}"))
            .backgrounds
            .image_count(),
        8
    );
}

#[test]
fn color_is_legal_only_in_the_final_layer_and_no_new_grammar_is_enabled() {
    for value in [
        "red,none",
        "transparent,none",
        "none red,none",
        "none,none,",
        "none fixed,none",
        "none,radial-gradient(red,blue)",
        "none / 2px,none",
        "none,none text",
    ] {
        supports("background", value, false);
    }
    for value in [
        "none,red",
        "none,none blue",
        "linear-gradient(red,blue),none lime",
    ] {
        supports("background", value, true);
    }
    assert_eq!(
        style("background:linear-gradient(red),none blue;background:red,none").background_color,
        Color::rgb(0, 0, 255)
    );
}

#[test]
fn commas_inside_quoted_unquoted_urls_and_gradient_functions_do_not_split_layers() {
    let doc = {
        let mut d = Document::parse("<div id=x></div>");
        d.initialize_url(::url::Url::parse("https://d.test/base/").unwrap());
        d
    };
    let sources = [StyleSource::new(
        r#"#x{background-image:url(A,B/**/C.PNG),url("D,E.PNG"),linear-gradient(rgb(1,2,3),blue)}"#,
    )];
    let styles = css::compute_styles_from_sources(&doc, &sources, 80.0, 60.0);
    let b = &styles[doc.query_selector("#x").unwrap()].backgrounds;
    assert_eq!(b.image_count(), 3);
    for (i, expected) in [
        (0, "https://d.test/base/A,B/**/C.PNG"),
        (1, "https://d.test/base/D,E.PNG"),
    ] {
        let Some(BackgroundImage::Url(value)) = &b.image_slots()[i] else {
            panic!("URL")
        };
        let css::url::ResolvedUrl::Absolute(value) = value.as_ref() else {
            panic!("absolute")
        };
        assert_eq!(value.as_str(), expected);
    }
    assert!(matches!(
        &b.image_slots()[2],
        Some(BackgroundImage::Gradient(_))
    ));
}

#[test]
fn malformed_nested_syntax_and_oversized_raw_values_remain_bounded() {
    for value in [
        "none,url(",
        "none,linear-gradient(red,blue]",
        "none,[none)",
        "none,url(A B)",
        "none,calc(1px)",
        "none,none\\\n",
    ] {
        assert_eq!(
            valid("background-image", value, &mut 1_000_000),
            Ok(false),
            "{value:?}"
        );
    }
    let raw = " ".repeat(4097);
    assert_eq!(
        valid("background-image", &raw, &mut 1_000_000),
        Err(Error::Limit)
    );
    assert_eq!(
        valid("background-image", "none,none", &mut 0),
        Err(Error::Limit)
    );
}

#[test]
fn dependent_lists_accept_box_escapes_and_reject_mixed_wide_keywords() {
    for (name, value) in [
        ("background-size", "cover,2px auto"),
        ("background-position", "right 2px bottom,25% 75%"),
        ("background-repeat", "repeat-x,repeat-y"),
        ("background-origin", r"\63 ontent-box,padd\69 ng-box"),
        ("background-clip", "border-box,content-box"),
    ] {
        supports(name, value, true);
        supports(name, &format!("{value},initial"), false);
    }
    let s = style(r"background-origin:\63 ontent-box,padd\69 ng-box");
    assert_eq!(
        s.backgrounds.origins(),
        [BackgroundBox::Content, BackgroundBox::Padding]
    );
    assert!(!css::supports_property(
        "object-position",
        "right 2px bottom"
    ));
}

#[test]
fn inheritance_all_reset_var_expansion_and_invalid_computed_lists_are_atomic() {
    let doc = Document::parse(
        "<div id=p style='--layers:none,linear-gradient(red);background:var(--layers);background-repeat:no-repeat,repeat-x'><div id=i style='background:inherit'></div><div id=plain></div><div id=reset style='background:inherit;all:initial'></div><div id=bad style='--bad:none,garbage;background-image:var(--bad)'></div></div>",
    );
    let s = css::compute_styles(&doc, &[], 80.0, 60.0);
    let at = |id| &s[doc.query_selector(id).unwrap()];
    assert_eq!(at("#p").backgrounds.image_count(), 2);
    assert!(Arc::ptr_eq(&at("#p").backgrounds, &at("#i").backgrounds));
    for id in ["#plain", "#reset", "#bad"] {
        assert!(at(id).backgrounds.is_single_none());
    }
    assert_eq!(
        at("#i").backgrounds.repeats(),
        [[false, false], [true, false]]
    );
}

#[test]
fn all_url_atoms_use_the_winning_consuming_base_after_revert_and_var() {
    let mut doc =
        Document::parse("<div id=p><div id=i></div><div id=v></div></div><div id=r></div>");
    doc.initialize_url(::url::Url::parse("https://doc.test/").unwrap());
    let sheet = |text, base| {
        StyleSource::with_base(
            text,
            SourceBase::Stylesheet(Arc::new(::url::Url::parse(base).unwrap())),
        )
    };
    let sheets = [
        sheet(
            "@layer low,high;@layer low{#r{background-image:url(A),none,url(B)}}#p{--images:url(A),none,url(B);background-image:var(--images)}",
            "https://a.test/css/main.css",
        ),
        sheet(
            "@layer high{#r{background-image:url(X),url(Y);background-image:revert-layer}}#i{background-image:inherit}#v{background-image:var(--images)}",
            "https://b.test/sheet.css",
        ),
    ];
    let s = css::compute_styles_from_sources(&doc, &sheets, 80.0, 60.0);
    for (id, base) in [
        ("#p", "https://a.test/css/"),
        ("#i", "https://a.test/css/"),
        ("#r", "https://a.test/css/"),
        ("#v", "https://b.test/"),
    ] {
        let b = &s[doc.query_selector(id).unwrap()].backgrounds;
        assert_eq!(b.image_count(), 3);
        assert!(b.image_slots()[1].is_none());
        for (i, end) in [(0, "A"), (2, "B")] {
            let Some(BackgroundImage::Url(u)) = &b.image_slots()[i] else {
                panic!("URL")
            };
            let css::url::ResolvedUrl::Absolute(u) = u.as_ref() else {
                panic!("absolute")
            };
            assert_eq!(u.as_str(), format!("{base}{end}"));
        }
    }
}
