use super::*;
use crate::css::{
    ComputedStyle, ObjectPosition, compute_styles, supports_declaration, supports_matches,
};
use crate::dom::Document;
use crate::graphics::Color;
use std::sync::Arc;

const UNITS: [f32; 4] = [20.0, 16.0, 800.0, 600.0];
fn start(offset: Length) -> PositionComponent {
    PositionComponent {
        from_end: false,
        offset,
    }
}
fn end(offset: Length) -> PositionComponent {
    PositionComponent {
        from_end: true,
        offset,
    }
}
fn style(declarations: &str) -> ComputedStyle {
    let doc = Document::parse(&format!("<div id=x style='{declarations}'></div>"));
    compute_styles(&doc, &[], 800.0, 600.0)[doc.query_selector("#x").unwrap()].clone()
}
fn geometry(style: &ComputedStyle) -> (BackgroundSize, BackgroundPosition, [bool; 2]) {
    (
        style.background_size,
        style.background_position,
        style.background_repeat,
    )
}

#[test]
fn positions_have_independent_one_two_three_four_value_literals() {
    use Length::{Percent as P, Px};
    for (value, x, y) in [
        ("left", start(P(0.0)), start(P(50.0))),
        ("top", start(P(50.0)), start(P(0.0))),
        ("center", start(P(50.0)), start(P(50.0))),
        ("25%", start(P(25.0)), start(P(50.0))),
        ("top left", start(P(0.0)), start(P(0.0))),
        ("center left", start(P(0.0)), start(P(50.0))),
        ("right 2px", start(P(100.0)), start(Px(2.0))),
        ("20% bottom", start(P(20.0)), start(P(100.0))),
        ("right 2px bottom", end(Px(2.0)), end(Px(0.0))),
        ("bottom right 2px", end(Px(2.0)), end(Px(0.0))),
        ("left top 3px", start(Px(0.0)), start(Px(3.0))),
        ("right 2px center", end(Px(2.0)), start(P(50.0))),
        ("center bottom -3px", start(P(50.0)), end(Px(-3.0))),
        ("bottom 1px right 2px", end(Px(2.0)), end(Px(1.0))),
        ("left -5% top 120%", start(P(-5.0)), start(P(120.0))),
    ] {
        assert_eq!(
            parse_position(value, UNITS, &mut 100_000),
            Ok(Some(BackgroundPosition { x, y })),
            "{value}"
        );
        assert!(
            supports_declaration("background-position", value),
            "{value}"
        );
        assert!(
            supports_matches(&format!("(background-position:{value})")),
            "{value}"
        );
        assert_eq!(
            style(&format!("background-position:{value}")).background_position,
            BackgroundPosition { x, y }
        );
    }
    for value in ["right 2px bottom", "center bottom -3px", "left top 3px"] {
        assert!(!supports_declaration("object-position", value));
        assert_eq!(
            style(&format!("object-position:{value}")).object_position,
            ObjectPosition::default()
        );
    }
}

#[test]
fn sizes_preserve_auto_axes_and_calculated_range_semantics() {
    use Length::{Auto, Calc, Percent, Px};
    for (value, expected) in [
        ("auto", BackgroundSize::default()),
        (
            "4px",
            BackgroundSize::Explicit {
                width: Px(4.0),
                height: Auto,
            },
        ),
        (
            "auto 2px",
            BackgroundSize::Explicit {
                width: Auto,
                height: Px(2.0),
            },
        ),
        (
            "25% 75%",
            BackgroundSize::Explicit {
                width: Percent(25.0),
                height: Percent(75.0),
            },
        ),
        ("contain", BackgroundSize::Contain),
        ("COVER", BackgroundSize::Cover),
        (
            "0 -0px",
            BackgroundSize::Explicit {
                width: Px(0.0),
                height: Px(0.0),
            },
        ),
        (
            "calc(1px - 2px)",
            BackgroundSize::Explicit {
                width: Calc {
                    px: 0.0,
                    percent: 0.0,
                    has_percent: false,
                },
                height: Auto,
            },
        ),
        (
            "calc(50% - 20px)",
            BackgroundSize::Explicit {
                width: Calc {
                    px: -20.0,
                    percent: 50.0,
                    has_percent: true,
                },
                height: Auto,
            },
        ),
        (
            "calc(0% - 1px)",
            BackgroundSize::Explicit {
                width: Calc {
                    px: -1.0,
                    percent: 0.0,
                    has_percent: true,
                },
                height: Auto,
            },
        ),
    ] {
        assert_eq!(
            parse_size(value, UNITS, &mut 100_000),
            Ok(Some(expected)),
            "{value}"
        );
        assert!(supports_declaration("background-size", value));
        assert_eq!(
            style(&format!("background-size:{value}")).background_size,
            expected
        );
    }
    let computed = style(
        "font-size:20px;background-size:calc(2em + 10%) 1rem;background-position:right calc(2em + 10%) bottom -5px",
    );
    assert_eq!(
        computed.background_size,
        BackgroundSize::Explicit {
            width: Calc {
                px: 40.0,
                percent: 10.0,
                has_percent: true
            },
            height: Px(16.0)
        }
    );
    assert_eq!(
        computed.background_position,
        BackgroundPosition {
            x: end(Calc {
                px: 40.0,
                percent: 10.0,
                has_percent: true
            }),
            y: end(Px(-5.0))
        }
    );
}

#[test]
fn repeat_axes_and_aliases_are_exact() {
    for (value, expected) in [
        ("repeat", [true, true]),
        ("no-repeat", [false, false]),
        ("repeat-x", [true, false]),
        ("repeat-y", [false, true]),
        ("repeat repeat", [true, true]),
        ("repeat no-repeat", [true, false]),
        ("no-repeat repeat", [false, true]),
        ("no-repeat no-repeat", [false, false]),
    ] {
        assert_eq!(parse_repeat(value, &mut 100_000), Ok(Some(expected)));
        assert!(supports_declaration("background-repeat", value));
        assert_eq!(
            style(&format!("background-repeat:{value}")).background_repeat,
            expected
        );
    }
}

#[test]
fn invalid_geometry_is_rejected_across_shared_entrypoints() {
    for (property, values) in [
        (
            "background-position",
            &[
                "",
                "auto",
                "1",
                "1e-999",
                "top 20%",
                "20% left",
                "left right",
                "top bottom",
                "left 2px 3px",
                "center 2px bottom",
                "left center 3px",
                "left 2px top 3px junk",
                "left,top",
                "calc(1px/**/+/**/2px)",
            ][..],
        ),
        (
            "background-size",
            &[
                "",
                "-1px",
                "-1%",
                "-1e-999px",
                "-1e-999%",
                "1",
                "1e-999",
                "1fr",
                "cover auto",
                "auto contain",
                "auto auto auto",
                "1px,2px",
                "calc(1px+ 2px)",
                "calc(1px * 2)",
                "calc(infinity * 1px)",
            ][..],
        ),
        (
            "background-repeat",
            &[
                "space",
                "round",
                "repeat-x repeat",
                "repeat repeat-y",
                "repeat repeat repeat",
                "repeat,no-repeat",
            ][..],
        ),
    ] {
        for value in values {
            assert_eq!(
                valid(property, value, &mut 100_000),
                Ok(false),
                "{property}:{value}"
            );
            assert!(!supports_declaration(property, value), "{property}:{value}");
            assert!(
                !supports_matches(&format!("({property}:{value})")),
                "{property}:{value}"
            );
            let before = style(
                "background-size:4px 2px;background-position:25% 75%;background-repeat:no-repeat",
            );
            let after = style(&format!(
                "background-size:4px 2px;background-position:25% 75%;background-repeat:no-repeat;{property}:{value}"
            ));
            assert_eq!(geometry(&after), geometry(&before), "{property}:{value}");
        }
    }
}

#[test]
fn shorthand_acceptance_and_five_literal_slices_are_atomic() {
    let value = "repeat-x right 2px bottom / 25% auto linear-gradient(red,blue) lime";
    assert_eq!(
        parts(value, &mut 1_000_000),
        Ok(Some([
            "lime",
            "linear-gradient(red,blue)",
            "25% auto",
            "right 2px bottom",
            "repeat-x"
        ]))
    );
    assert_eq!(
        parts("red", &mut 100_000),
        Ok(Some(["red", "none", "auto auto", "0% 0%", "repeat"]))
    );
    for value in [
        "red",
        "none",
        "linear-gradient(red,blue) lime",
        "linear-gradient(red,blue) center / 4px 2px no-repeat lime",
        value,
        "linear-gradient(red)center/4px no-repeat",
        "none 0 / cover repeat-y",
        "top right no-repeat",
        "no-repeat red",
        "linear-gradient(red) repeat",
    ] {
        assert!(parts(value, &mut 1_000_000).unwrap().is_some(), "{value}");
        assert!(supports_declaration("background", value), "{value}");
    }
    for value in [
        "red blue",
        "none none",
        "center /",
        "/ 4px",
        "left / 4px / 2px",
        "left repeat top",
        "repeat-x repeat",
        "left top bottom",
        "red center / -1px",
        "red padding-box",
        "red fixed",
        "red,blue",
        "linear-gradient(red) space",
        "initial red",
        "inherit no-repeat",
        "linear-gradient(",
    ] {
        assert_eq!(parts(value, &mut 1_000_000), Ok(None), "{value}");
        assert!(!supports_declaration("background", value), "{value}");
    }
}

#[test]
fn shorthand_resets_all_geometry_and_invalid_variables_never_resurrect() {
    let prefix = "background:linear-gradient(red,blue) center / 4px 2px no-repeat lime;";
    let initial = (
        BackgroundSize::default(),
        BackgroundPosition::default(),
        [true, true],
    );
    let before = style(prefix);
    for reset in [
        "orange",
        "linear-gradient(blue)",
        "none",
        "initial",
        "unset",
    ] {
        let after = style(&format!("{prefix}background:{reset}"));
        assert_eq!(geometry(&after), initial, "{reset}");
    }
    let invalid = style(&format!("{prefix}background:red center / -1px"));
    assert_eq!(geometry(&invalid), geometry(&before));
    assert_eq!(invalid.background_color, Color::rgb(0, 255, 0));
    let late = style(&format!(
        "{prefix}--bad:red center / -1px;background:var(--bad)"
    ));
    assert_eq!(geometry(&late), initial);
    assert_eq!(late.background_color, Color::TRANSPARENT);
    assert!(late.background_image.is_none());
    for (property, invalid) in [
        ("background-size", "-1px"),
        ("background-position", "top 20%"),
        ("background-repeat", "space"),
    ] {
        let late = style(&format!("{prefix}--bad:{invalid};{property}:var(--bad)"));
        match property {
            "background-size" => assert_eq!(late.background_size, initial.0),
            "background-position" => assert_eq!(late.background_position, initial.1),
            _ => assert_eq!(late.background_repeat, initial.2),
        }
    }
}

#[test]
fn inheritance_rollback_and_pool_sharing_keep_geometry_independent() {
    let doc = Document::parse(
        "<style>@layer a,b;@layer a{#rollback{background:linear-gradient(red) center / 4px 2px no-repeat lime}}@layer b{#rollback{background:orange;background:revert-layer}}#parent{background:linear-gradient(red) center / 4px 2px no-repeat lime}#inherit{background:inherit}#same{background:linear-gradient(red) right / cover repeat-y}</style><div id=parent><div id=inherit></div><div id=plain></div></div><div id=same></div><div id=rollback></div>",
    );
    let computed = compute_styles(&doc, &doc.stylesheets(), 800.0, 600.0);
    let at = |id| &computed[doc.query_selector(id).unwrap()];
    assert_eq!(geometry(at("#inherit")), geometry(at("#parent")));
    assert_eq!(geometry(at("#plain")), geometry(&ComputedStyle::default()));
    assert_eq!(geometry(at("#rollback")), geometry(at("#parent")));
    let parent = at("#parent").background_image.as_ref().unwrap();
    assert!(Arc::ptr_eq(
        parent,
        at("#inherit").background_image.as_ref().unwrap()
    ));
    assert!(Arc::ptr_eq(
        parent,
        at("#same").background_image.as_ref().unwrap()
    ));
    assert_ne!(geometry(at("#same")), geometry(at("#parent")));
}

#[test]
fn comment_token_boundaries_keep_calc_whitespace_strict() {
    for (property, value) in [
        ("background-position", "right/**/2px/**/bottom"),
        ("background-size", "calc(50% - 2px)/**/auto"),
        ("background-repeat", "repeat/**/no-repeat"),
        (
            "background",
            "linear-gradient(rgb(255/**/ 0 0),blue)center/calc(50% - 2px) 2px no-repeat",
        ),
    ] {
        assert!(supports_declaration(property, value), "{property}:{value}");
        assert!(
            supports_matches(&format!("({property}:{value})")),
            "{property}:{value}"
        );
    }
    for property in ["background-size", "background-position"] {
        assert!(!supports_declaration(property, "calc(1px/**/+/**/2px)"));
        assert!(!supports_matches(&format!(
            "({property}:calc(1px/**/+/**/2px))"
        )));
    }
}

#[test]
fn parser_work_bounds_and_fixed_style_storage_are_explicit() {
    // Zero position: tokenizer3 + take2 + lexical6 + grammar24 =35.
    let zero = BackgroundPosition {
        x: start(Length::Px(0.0)),
        y: PositionComponent::default(),
    };
    assert_eq!(parse_position("0", UNITS, &mut 35), Ok(Some(zero)));
    assert_eq!(parse_position("0", UNITS, &mut 34), Err(Error::Limit));
    // Zero size: tokenizer3 + take2 + lexical6 + final grammar8 =19.
    assert_eq!(
        parse_size("0", UNITS, &mut 19),
        Ok(Some(BackgroundSize::Explicit {
            width: Length::Px(0.0),
            height: Length::Auto
        }))
    );
    assert_eq!(parse_size("0", UNITS, &mut 18), Err(Error::Limit));
    for (property, value) in [
        (
            "background",
            "linear-gradient(red,blue) right calc(25% + 2px) bottom / calc(50% - 1px) 2px no-repeat lime",
        ),
        ("background-size", "calc(50% - 1px) 2px"),
        ("background-position", "right calc(25% + 2px) bottom"),
        ("background-repeat", "repeat no-repeat"),
    ] {
        let mut work = 1_000_000;
        assert_eq!(valid(property, value, &mut work), Ok(true));
        let used = 1_000_000 - work;
        let mut exact = used;
        assert_eq!(valid(property, value, &mut exact), Ok(true));
        assert_eq!(exact, 0);
        assert_eq!(valid(property, value, &mut (used - 1)), Err(Error::Limit));
    }
    assert_eq!(
        parse_size(&"0 ".repeat(2049), UNITS, &mut 1_000_000),
        Err(Error::Limit)
    );
    let deep = format!("{}1px{}", "calc(".repeat(18), ")".repeat(18));
    assert_eq!(parse_size(&deep, UNITS, &mut 1_000_000), Err(Error::Limit));
    let terms = format!("calc({})", ["1px"; 257].join(" + "));
    assert_eq!(parse_size(&terms, UNITS, &mut 1_000_000), Err(Error::Limit));
    println!(
        "background geometry sizes: size={} position={} repeat={} computed_style={} maximum_nodes=100000",
        std::mem::size_of::<BackgroundSize>(),
        std::mem::size_of::<BackgroundPosition>(),
        std::mem::size_of::<[bool; 2]>(),
        std::mem::size_of::<ComputedStyle>()
    );
    assert!(std::mem::size_of::<BackgroundSize>() <= 64);
    assert!(std::mem::size_of::<BackgroundPosition>() <= 48);
}
