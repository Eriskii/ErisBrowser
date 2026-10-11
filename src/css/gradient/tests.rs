use super::*;
use crate::graphics::Color;

fn gradient(value: &str) -> ParsedGradient {
    match parse_image(value, [20.0, 16.0, 800.0, 600.0], &mut 1_000_000)
        .unwrap()
        .unwrap()
    {
        ParsedImage::Gradient(value) => value,
        ParsedImage::None | ParsedImage::Url(_) => panic!("expected gradient"),
    }
}

#[test]
fn directions_interpolation_and_single_color_have_literal_values() {
    for (value, expected) in [
        ("red", GradientDirection::Angle(180.0)),
        ("to right, red", GradientDirection::Angle(90.0)),
        ("in srgb to left, red", GradientDirection::Angle(270.0)),
        ("-450deg in srgb, red", GradientDirection::Angle(270.0)),
        ("0.25turn, red", GradientDirection::Angle(90.0)),
        ("100grad, red", GradientDirection::Angle(90.0)),
        ("0rad, red", GradientDirection::Angle(0.0)),
        ("-0e99, red", GradientDirection::Angle(0.0)),
        (
            "to top right in srgb, red",
            GradientDirection::Corner {
                right: true,
                bottom: false,
            },
        ),
        (
            "in srgb to left bottom, red",
            GradientDirection::Corner {
                right: false,
                bottom: true,
            },
        ),
    ] {
        let parsed = gradient(&format!("linear-gradient({value})"));
        assert_eq!(parsed.direction, expected, "{value}");
        assert_eq!(
            parsed.stops(),
            &[GradientStop {
                color: GradientColor::Color(Color::rgb(255, 0, 0)),
                position: None
            }]
        );
    }
}

#[test]
fn stops_retain_f64_coefficients_double_positions_and_current_color() {
    let parsed = gradient(
        "linear-gradient(currentColor -50% calc(2em + 10%), rgba(0, 255, 0, 0.5) 150%, blue calc(1000000in - 1px))",
    );
    assert_eq!(parsed.stops().len(), 4);
    assert_eq!(
        parsed.stops()[0],
        GradientStop {
            color: GradientColor::CurrentColor,
            position: Some(GradientPosition {
                px: 0.0,
                percent: -50.0
            })
        }
    );
    assert_eq!(
        parsed.stops()[1],
        GradientStop {
            color: GradientColor::CurrentColor,
            position: Some(GradientPosition {
                px: 40.0,
                percent: 10.0
            })
        }
    );
    assert_eq!(
        parsed.stops()[2].position,
        Some(GradientPosition {
            px: 0.0,
            percent: 150.0
        })
    );
    assert_eq!(
        parsed.stops()[2].color,
        GradientColor::Color(Color::rgba(0, 255, 0, 128))
    );
    assert_eq!(
        parsed.stops()[3].position,
        Some(GradientPosition {
            px: 95_999_999.0,
            percent: 0.0
        })
    );
}

#[test]
fn malformed_and_out_of_scope_values_are_rejected_wholly() {
    for value in [
        "linear-gradient()",
        "linear-gradient(red,)",
        "linear-gradient(,red)",
        "linear-gradient(45,red,blue)",
        "linear-gradient(1e999deg,red)",
        "linear-gradient(1e-999,red)",
        "linear-gradient(to top bottom,red)",
        "linear-gradient(to center,red)",
        "linear-gradient(to right left,red)",
        "linear-gradient(in oklab,red,blue)",
        "linear-gradient(in srgb in srgb,red)",
        "linear-gradient(90deg to right,red)",
        "linear-gradient(red,50%,blue)",
        "linear-gradient(color(srgb 1 0 0),blue)",
        "linear-gradient(rgb(1,2,3/0.5),blue)",
        "linear-gradient(red 1,blue)",
        "linear-gradient(red auto,blue)",
        "linear-gradient(red 1fr,blue)",
        "linear-gradient(red 0 10% 20%,blue)",
        "linear-gradient(red calc(10%+ 1px),blue)",
        "linear-gradient(red calc(10%/**/+/**/1px),blue)",
        "linear-gradient(red),,linear-gradient(blue)",
        "repeating-linear-gradient(red,blue)",
        "url(x),,url(y)",
        "none red",
        "linear-gradient(red) junk",
    ] {
        assert!(
            parse_image(value, [16.0; 4], &mut 1_000_000)
                .unwrap()
                .is_none(),
            "{value}"
        );
        assert!(
            !super::super::supports_declaration("background-image", value),
            "{value}"
        );
    }
}

#[test]
fn bounded_authored_stops_expand_without_exceeding_the_fixed_scratch() {
    let at_limit = format!("linear-gradient({})", ["red 0% 100%"; 16].join(","));
    assert_eq!(gradient(&at_limit).stops().len(), 32);
    let too_many = format!("linear-gradient({})", ["red"; 17].join(","));
    assert!(
        parse_image(&too_many, [16.0; 4], &mut 1_000_000)
            .unwrap()
            .is_none()
    );
    let mut remaining = 1_000_000;
    assert!(
        parse_image(&at_limit, [16.0; 4], &mut remaining)
            .unwrap()
            .is_some()
    );
    let used = 1_000_000 - remaining;
    let mut exact = used;
    assert!(
        parse_image(&at_limit, [16.0; 4], &mut exact)
            .unwrap()
            .is_some()
    );
    assert_eq!(exact, 0);
    let mut short = used - 1;
    assert!(matches!(
        parse_image(&at_limit, [16.0; 4], &mut short),
        Err(Error::Limit)
    ));
    assert_eq!(short, 0);
}

#[test]
fn cascade_resets_both_background_longhands_and_never_revives_old_winners() {
    let doc = super::super::Document::parse(
        "<style>div{background:linear-gradient(red,blue) lime}#reset{background:orange}#bad{background:linear-gradient(red,) orange}#var{--bad:linear-gradient(red,) orange;background:var(--bad)}#image{background:linear-gradient(blue)}#none{background:none}</style><div id=reset></div><div id=bad></div><div id=var></div><div id=image></div><div id=none></div>",
    );
    let styles = super::super::compute_styles(&doc, &doc.stylesheets(), 800.0, 600.0);
    let at = |id| &styles[doc.query_selector(id).unwrap()];
    assert!(at("#reset").backgrounds.image_slots()[0].is_none());
    assert_eq!(at("#reset").background_color, Color::rgb(255, 165, 0));
    assert!(at("#bad").backgrounds.image_slots()[0].is_some());
    assert_eq!(at("#bad").background_color, Color::rgb(0, 255, 0));
    for id in ["#var", "#none"] {
        assert!(at(id).backgrounds.image_slots()[0].is_none());
        assert_eq!(at(id).background_color, Color::TRANSPARENT);
    }
    assert!(at("#image").backgrounds.image_slots()[0].is_some());
    assert_eq!(at("#image").background_color, Color::TRANSPARENT);
}

#[test]
fn inherited_images_share_storage_without_resolving_current_color_early() {
    let doc = super::super::Document::parse(
        "<style>main{color:red;background-image:linear-gradient(currentColor)}#child{color:lime;background-image:inherit}#same{background-image:linear-gradient(currentColor)}</style><main id=parent><div id=child></div><div id=plain></div></main><div id=same></div>",
    );
    let styles = super::super::compute_styles(&doc, &doc.stylesheets(), 800.0, 600.0);
    let at = |id| &styles[doc.query_selector(id).unwrap()];
    let parent = gradient_handle(at("#parent"));
    assert!(Arc::ptr_eq(parent, gradient_handle(at("#child"))));
    assert!(Arc::ptr_eq(parent, gradient_handle(at("#same"))));
    assert_eq!(at("#child").color, Color::rgb(0, 255, 0));
    assert_eq!(parent.stops[0].color, GradientColor::CurrentColor);
    assert!(at("#plain").backgrounds.image_slots()[0].is_none());
}

#[test]
fn pool_admission_precedes_retained_storage_and_duplicate_allocation() {
    let parsed = gradient("linear-gradient(red,blue)");
    let mut probe = GradientPool::default();
    let mut work = 1_000_000;
    let first = probe.intern(&parsed, &mut work).unwrap().unwrap();
    let cost = 1_000_000 - work;
    let bytes = probe.bytes;
    assert!(
        bytes >= std::mem::size_of::<LinearGradient>() + 2 * std::mem::size_of::<GradientStop>()
    );
    let mut short = GradientPool::default();
    assert!(matches!(
        short.intern(&parsed, &mut (cost - 1)),
        Err(Error::Limit)
    ));
    assert_eq!(short.bytes, 0);
    assert!(short.values.is_empty());
    let mut exact = GradientPool::default();
    let mut exact_work = cost;
    assert!(exact.intern(&parsed, &mut exact_work).unwrap().is_some());
    assert_eq!(exact_work, 0);
    assert_eq!(exact.bytes, bytes);
    let mut full = GradientPool {
        bytes: MAX_RETAINED_GRADIENT_BYTES - bytes + 1,
        ..GradientPool::default()
    };
    assert!(full.intern(&parsed, &mut 1_000_000).unwrap().is_none());
    assert!(full.values.is_empty());
    let shared = probe.intern(&parsed, &mut 1_000_000).unwrap().unwrap();
    assert!(Arc::ptr_eq(&first, &shared));
    assert_eq!(probe.bytes, bytes);
    eprintln!(
        "gradient={} stop={} style={}",
        std::mem::size_of::<LinearGradient>(),
        std::mem::size_of::<GradientStop>(),
        std::mem::size_of::<super::super::ComputedStyle>()
    );
}

#[test]
fn comments_preserve_token_boundaries_across_supports_and_declarations() {
    for value in [
        "linear-gradient(to/**/right/**/in/**/srgb,red/**/0%,blue/**/100%)",
        "linear-gradient(rgb(255/**/ 0 0)0%,blue)",
    ] {
        assert!(
            super::super::supports_declaration("background-image", value),
            "{value}"
        );
        assert!(
            super::super::supports_matches(&format!("(background-image:{value})")),
            "{value}"
        );
        let doc = super::super::Document::parse(&format!(
            "<div id=x style='background-image:{value}'></div>"
        ));
        let styles = super::super::compute_styles(&doc, &[], 800.0, 600.0);
        assert!(
            styles[doc.query_selector("#x").unwrap()]
                .backgrounds
                .image_slots()[0]
                .is_some()
        );
    }
    assert!(!super::super::supports_declaration(
        "background-image",
        r"linear-gr\61 dient(red,blue)"
    ));
}

fn gradient_handle(style: &super::super::ComputedStyle) -> &Arc<LinearGradient> {
    match style.backgrounds.image_slots()[0].as_ref().unwrap() {
        super::super::BackgroundImage::Gradient(value) => value,
        super::super::BackgroundImage::Url(_) => panic!("expected gradient"),
    }
}
