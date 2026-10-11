use super::*;

const WORK: usize = 4_000_000;
fn parse(value: &str) -> InlineStyle {
    InlineStyle::parse(value, &mut WORK.clone()).unwrap()
}
fn get(style: &InlineStyle, name: &str) -> String {
    style.get_property_value(name, &mut WORK.clone()).unwrap()
}
fn text(style: &InlineStyle) -> String {
    style.serialize(&mut WORK.clone()).unwrap()
}
fn set(style: &mut InlineStyle, property: &str, value: &str) -> bool {
    style
        .set_property(property, value, "", &mut WORK.clone())
        .unwrap()
}

#[test]
fn five_background_longhands_roundtrip_and_remove_together() {
    let mut style = parse("background:linear-gradient(red,blue) center / 4px 2px no-repeat lime");
    for (name, expected) in [
        ("background-color", "lime"),
        ("background-image", "linear-gradient(red,blue)"),
        ("background-size", "4px 2px"),
        ("background-position", "center"),
        ("background-repeat", "no-repeat"),
    ] {
        assert_eq!(get(&style, name), expected);
    }
    assert_eq!(
        get(&style, "background"),
        "linear-gradient(red,blue) center / 4px 2px no-repeat lime"
    );
    assert_eq!(parse(&text(&style)), style);
    assert!(set(&mut style, "background", "blue"));
    assert_eq!(get(&style, "background"), "blue");
    assert_eq!(get(&style, "background-size"), "auto auto");
    assert_eq!(get(&style, "background-position"), "0% 0%");
    assert_eq!(get(&style, "background-repeat"), "repeat");
    assert_eq!(
        style
            .remove_property("background", &mut WORK.clone())
            .unwrap(),
        "blue"
    );
    assert_eq!(style, InlineStyle::default());
}

#[test]
fn new_geometry_setters_reject_invalid_values_without_erasing_all_five() {
    let original =
        parse("background:linear-gradient(red,blue) center / 4px 2px no-repeat lime;color:blue");
    for (property, value) in [
        ("background-size", "-1px"),
        ("background-size", "-1e-999px"),
        ("background-size", "calc(1px/**/+/**/2px)"),
        ("background-position", "top 20%"),
        ("background-position", "center 2px bottom"),
        ("background-repeat", "space"),
        ("background-repeat", "repeat-x repeat"),
        ("background", "red center / -1px"),
        ("background", "red / 4px"),
        ("background", "linear-gradient("),
    ] {
        let mut current = original.clone();
        let mut work = WORK;
        assert_eq!(
            current.set_property(property, value, "important", &mut work),
            Ok(false),
            "{property}:{value}"
        );
        assert_eq!(current, original);
        assert!(work > 0);
    }
    let mut current = original;
    assert!(set(&mut current, "background-position", "right 2px bottom"));
    assert_eq!(get(&current, "background-position"), "right 2px bottom");
    assert!(set(&mut current, "background-size", "calc(50% - 20px) 2px"));
    assert!(set(&mut current, "background-repeat", "REPEAT-Y"));
    assert_eq!(get(&current, "background-repeat"), "repeat-y");
    assert_eq!(parse(&text(&current)), current);
}

#[test]
fn background_shorthand_priority_and_mixed_wide_cover_five_fields() {
    for keyword in ["inherit", "initial", "unset", "revert", "revert-layer"] {
        for property in [
            "background-color",
            "background-image",
            "background-size",
            "background-position",
            "background-repeat",
        ] {
            let mut current =
                parse("background:linear-gradient(red) center / 4px 2px no-repeat lime");
            assert!(set(&mut current, property, keyword));
            assert_eq!(get(&current, "background"), "");
            assert!(set(&mut current, "background", keyword));
            assert_eq!(get(&current, "background"), keyword);
        }
    }
    let mut current =
        parse("background:linear-gradient(red) center / 4px 2px no-repeat lime!important");
    for property in [
        "background-color",
        "background-image",
        "background-size",
        "background-position",
        "background-repeat",
    ] {
        assert_eq!(
            current
                .get_property_priority(property, &mut WORK.clone())
                .unwrap(),
            "important"
        );
    }
    assert!(set(&mut current, "background-size", "2px"));
    assert_eq!(get(&current, "background"), "");
}

#[test]
fn pending_geometry_shorthand_serializes_and_resolves_all_five() {
    let current = parse(
        "background:linear-gradient(red) center / 4px 2px no-repeat lime;--bad:left / -1px;background:var(--bad)",
    );
    assert_eq!(get(&current, "background"), "var(--bad)");
    let doc = crate::dom::Document::parse(&format!("<div id=x style='{}'></div>", text(&current)));
    let styles = css::compute_styles(&doc, &[], 800.0, 600.0);
    let computed = &styles[doc.query_selector("#x").unwrap()];
    assert!(computed.background_image.is_none());
    assert_eq!(
        computed.background_color,
        crate::graphics::Color::TRANSPARENT
    );
    assert_eq!(computed.background_size, css::BackgroundSize::default());
    assert_eq!(
        computed.background_position,
        css::BackgroundPosition::default()
    );
    assert_eq!(computed.background_repeat, [true, true]);
}

#[test]
fn geometry_cssom_exact_and_short_budget_is_transactional() {
    let original = parse("background:lime;color:blue");
    for (property, value) in [
        (
            "background",
            "linear-gradient(currentColor,blue) right calc(25% + 2px) bottom / calc(50% - 2px) 2px no-repeat lime",
        ),
        ("background-size", "calc(50% - 2px) 2px"),
        ("background-position", "right calc(25% + 2px) bottom"),
        ("background-repeat", "no-repeat repeat"),
    ] {
        let mut measured = original.clone();
        let mut work = WORK;
        assert!(
            measured
                .set_property(property, value, "important", &mut work)
                .unwrap()
        );
        let cost = WORK - work;
        let mut exact = original.clone();
        let mut exact_work = cost;
        assert!(
            exact
                .set_property(property, value, "important", &mut exact_work)
                .unwrap()
        );
        assert_eq!(exact_work, 0);
        assert_eq!(exact, measured);
        for cut in [0, 1, cost - 1] {
            let mut short = original.clone();
            assert_eq!(
                short.set_property(property, value, "important", &mut cut.clone()),
                Err(Error::Limit)
            );
            assert_eq!(short, original);
        }
    }
}

#[test]
fn background_getter_requires_geometry_declarations_in_the_same_block() {
    // CSSOM getPropertyValue returns empty for a missing longhand, even when
    // its computed initial value would be enough to paint this background.
    let mut partial = parse("background-color:yellow;background-image:linear-gradient(red,blue)");
    assert_eq!(get(&partial, "background"), "");
    for (name, value) in [
        ("background-size", "auto auto"),
        ("background-position", "0% 0%"),
    ] {
        assert!(set(&mut partial, name, value));
        assert_eq!(get(&partial, "background"), "");
    }
    assert!(set(&mut partial, "background-repeat", "repeat"));
    assert_eq!(get(&partial, "background"), "");
    assert!(set(&mut partial, "background-origin", "padding-box"));
    assert_eq!(get(&partial, "background"), "");
    assert!(set(&mut partial, "background-clip", "border-box"));
    assert_eq!(
        get(&partial, "background"),
        "linear-gradient(red,blue) yellow"
    );
    for name in [
        "background-size",
        "background-position",
        "background-repeat",
    ] {
        let mut missing = partial.clone();
        assert!(
            !missing
                .remove_property(name, &mut WORK.clone())
                .unwrap()
                .is_empty()
        );
        assert_eq!(get(&missing, "background"), "", "missing {name}");
        assert_eq!(get(&missing, "background-color"), "yellow");
        assert_eq!(
            get(&missing, "background-image"),
            "linear-gradient(red,blue)"
        );
    }

    // A shorthand initialization explicitly supplies all seven declarations.
    // Later valid color/image setters and rejected setters retain that basis.
    let mut initialized = InlineStyle::default();
    assert!(set(&mut initialized, "background", "none"));
    assert!(set(&mut initialized, "background-color", "yellow"));
    let image = "linear-gradient(to right,currentColor 50%,transparent 50%)";
    assert!(set(&mut initialized, "background-image", image));
    assert_eq!(
        get(&initialized, "background"),
        "linear-gradient(to right,currentcolor 50%,transparent 50%) yellow"
    );
    let saved = initialized.clone();
    assert!(!set(
        &mut initialized,
        "background-image",
        "linear-gradient(red,)"
    ));
    assert!(!set(
        &mut initialized,
        "background",
        "linear-gradient(red,) orange"
    ));
    assert_eq!(initialized, saved);
    assert_eq!(get(&initialized, "background-size"), "auto auto");
    assert_eq!(get(&initialized, "background-position"), "0% 0%");
    assert_eq!(get(&initialized, "background-repeat"), "repeat");
}

#[test]
fn validated_background_longhands_keep_normalization_priority_and_pending_values() {
    for (property, authored, expected) in [
        (
            "background-image",
            "linear-gradient(to right,CURRENTColor 50%,transparent 50%)",
            "linear-gradient(to right,currentcolor 50%,transparent 50%)",
        ),
        ("background-size", "AUTO 2px", "auto 2px"),
        (
            "background-position",
            "RIGHT calc(25% + 2px) BOTTOM",
            "right calc(25% + 2px) bottom",
        ),
        ("background-repeat", "REPEAT-Y", "repeat-y"),
    ] {
        assert_eq!(
            records(property, authored, true, &mut WORK.clone()).unwrap(),
            Some(vec![Record {
                name: property.into(),
                value: expected.into(),
                important: true,
                pending: None,
            }])
        );
        for wide in ["inherit", "initial", "unset", "revert", "revert-layer"] {
            assert_eq!(
                records(property, wide, false, &mut WORK.clone()).unwrap(),
                Some(vec![Record {
                    name: property.into(),
                    value: wide.into(),
                    important: false,
                    pending: None,
                }])
            );
        }
        // Longhand variables retain authored custom-property case and are
        // accepted before ordinary validation, exactly as other pending values.
        assert_eq!(
            records(property, "var(--Saved)", true, &mut WORK.clone()).unwrap(),
            Some(vec![Record {
                name: property.into(),
                value: "var(--Saved)".into(),
                important: true,
                pending: None,
            }])
        );
    }
    let original =
        parse("background:linear-gradient(red) center / 4px 2px no-repeat lime;color:blue");
    for (property, invalid) in [
        ("background-image", "linear-gradient(red,)"),
        ("background-image", "linear-gradient(red]"),
        ("background-size", "-1e-999px"),
        ("background-size", "calc(1px/**/+/**/2px)"),
        ("background-position", "top 20%"),
        ("background-repeat", "repeat-x repeat"),
        ("background-repeat", "repeat\\"),
    ] {
        let mut current = original.clone();
        assert_eq!(
            current.set_property(property, invalid, "important", &mut WORK.clone()),
            Ok(false)
        );
        assert_eq!(current, original);
    }
}

#[test]
fn validated_background_longhands_pay_one_parser_pass_before_record_publication() {
    // Literal source ledger for records(), including tokenization, variable
    // syntax, normalization allowance, first validation and storage prepay.
    // none: 60 + (49 + 49 + 270) + 69 = 497
    // size0: 24 + (13 + 13 + 19) + 66 = 135
    // pos0:  24 + (13 + 13 + 35) + 66 = 151
    // repeat:84 + (73 + 73 + 51) + 71 = 352
    for (property, value, cost) in [
        ("background-image", "none", 497),
        ("background-size", "0", 135),
        ("background-position", "0", 151),
        ("background-repeat", "repeat", 352),
    ] {
        let expected = Some(vec![Record {
            name: property.into(),
            value: value.into(),
            important: true,
            pending: None,
        }]);
        let mut exact = cost;
        assert_eq!(records(property, value, true, &mut exact), Ok(expected));
        assert_eq!(exact, 0);
        for cut in [0, 1, cost - 1] {
            let mut work = cut;
            assert_eq!(records(property, value, true, &mut work), Err(Error::Limit));
            assert_eq!(work, 0);
        }
    }
}
