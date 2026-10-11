use super::*;

const WORK: usize = 4_000_000;
const NAMES: [&str; 7] = [
    "background-color",
    "background-image",
    "background-size",
    "background-position",
    "background-repeat",
    "background-origin",
    "background-clip",
];
fn parse(source: &str) -> InlineStyle {
    InlineStyle::parse(source, &mut WORK.clone()).unwrap()
}
fn get(style: &InlineStyle, property: &str) -> String {
    style
        .get_property_value(property, &mut WORK.clone())
        .unwrap()
}
fn text(style: &InlineStyle) -> String {
    style.serialize(&mut WORK.clone()).unwrap()
}
fn set(style: &mut InlineStyle, property: &str, value: &str, priority: &str) -> bool {
    style
        .set_property(property, value, priority, &mut WORK.clone())
        .unwrap()
}

#[test]
fn seven_records_reset_and_remove_atomically_with_stable_order() {
    let mut style = parse(
        "background:url(A.PNG) right 2px bottom 3px / 4px 5px no-repeat repeat content-box padding-box red",
    );
    assert_eq!(style.len(), 7);
    for (index, (name, value)) in NAMES
        .into_iter()
        .zip([
            "red",
            "url(A.PNG)",
            "4px 5px",
            "right 2px bottom 3px",
            "no-repeat repeat",
            "content-box",
            "padding-box",
        ])
        .enumerate()
    {
        assert_eq!(style.item(index), name);
        assert_eq!(get(&style, name), value);
    }
    assert_eq!(
        get(&style, "background"),
        "url(A.PNG) right 2px bottom 3px / 4px 5px no-repeat repeat red content-box padding-box"
    );
    assert_eq!(parse(&text(&style)), style);
    assert!(set(&mut style, "background", "blue", "important"));
    assert_eq!(get(&style, "background"), "blue");
    for (name, expected) in NAMES.into_iter().zip([
        "blue",
        "none",
        "auto auto",
        "0% 0%",
        "repeat",
        "padding-box",
        "border-box",
    ]) {
        assert_eq!(get(&style, name), expected);
        assert_eq!(
            style
                .get_property_priority(name, &mut WORK.clone())
                .unwrap(),
            "important"
        );
    }
    assert_eq!(
        style
            .remove_property("background", &mut WORK.clone())
            .unwrap(),
        "blue"
    );
    assert_eq!(style, InlineStyle::default());
}

#[test]
fn box_suffix_roundtrips_all_nine_pairs_and_preserves_default_layer_strings() {
    for origin in ["border-box", "padding-box", "content-box"] {
        for clip in ["border-box", "padding-box", "content-box"] {
            let source = format!("background:red {origin} {clip}");
            let style = parse(&source);
            let expected = if origin == "padding-box" && clip == "border-box" {
                "red".to_owned()
            } else if origin == clip {
                format!("red {origin}")
            } else {
                format!("red {origin} {clip}")
            };
            assert_eq!(get(&style, "background"), expected);
            assert_eq!(parse(&format!("background:{expected}")), style);
        }
    }
    for (source, expected) in [
        ("red", "red"),
        ("linear-gradient(red,blue)", "linear-gradient(red,blue)"),
        ("url(A.PNG) red", "url(A.PNG) red"),
        (
            "linear-gradient(red) center / 4px 2px no-repeat lime",
            "linear-gradient(red) center / 4px 2px no-repeat lime",
        ),
    ] {
        assert_eq!(
            get(&parse(&format!("background:{source}")), "background"),
            expected
        );
    }
}

#[test]
fn escaped_keywords_normalize_without_rewriting_url_payloads() {
    for (authored, expected) in [
        (r"\70 adding-box", "padding-box"),
        (r"\63 ontent-box", "content-box"),
        (r"b\6f rder-box", "border-box"),
    ] {
        for property in ["background-origin", "background-clip"] {
            let mut style = InlineStyle::default();
            assert!(set(&mut style, property, authored, ""));
            assert_eq!(get(&style, property), expected);
            assert!(css::supports_property(property, authored));
            assert_eq!(parse(&text(&style)), style);
        }
    }
    let mut style = parse(
        r#"background:url("Dir/content-box/**/Logo.PNG?Q=A B") \63 ontent-box/**/red/**/padding-box"#,
    );
    assert_eq!(
        get(&style, "background-image"),
        r#"url("Dir/content-box/**/Logo.PNG?Q=A B")"#
    );
    assert_eq!(get(&style, "background-origin"), "content-box");
    assert_eq!(get(&style, "background-clip"), "padding-box");
    assert_eq!(parse(&text(&style)), style);
    let saved = style.clone();
    assert!(!set(&mut style, "background-origin", "padd/**/ing-box", ""));
    assert!(!set(&mut style, r"background-\6f rigin", "border-box", ""));
    assert_eq!(style, saved);
}

#[test]
fn invalid_box_values_preserve_all_records_text_and_priority() {
    let original = parse(
        "background:url(A.PNG) center / 4px no-repeat content-box padding-box red!important;color:blue",
    );
    for (property, invalid) in [
        ("background-origin", ""),
        ("background-origin", "text"),
        ("background-clip", "border-area"),
        ("background-origin", "content-box padding-box"),
        ("background-clip", "content-box,, padding-box"),
        ("background-origin", "calc(1px)"),
        ("background-clip", "content-box junk"),
        ("background", "content-box padding-box border-box"),
        ("background", "left / content-box"),
        ("background", "content-box / 2px"),
        ("background", "inherit content-box"),
        ("background", "red content-box; color:lime"),
        ("background", "url(A) content-box,, blue"),
        ("background-origin", "border-box\\"),
        ("background-clip", "content-box\0"),
    ] {
        // An empty CSSOM setter intentionally removes its property; test that
        // separate API behavior below rather than classifying it as syntax.
        if invalid.is_empty() {
            continue;
        }
        let mut style = original.clone();
        assert!(
            !set(&mut style, property, invalid, "important"),
            "{property}: {invalid}"
        );
        assert_eq!(style, original);
        assert_eq!(text(&style), text(&original));
        assert_eq!(
            style
                .get_property_priority("background", &mut WORK.clone())
                .unwrap(),
            "important"
        );
    }
    let mut removed = original.clone();
    assert!(set(&mut removed, "background-clip", "", ""));
    assert_eq!(get(&removed, "background"), "");
    assert_eq!(get(&removed, "background-origin"), "content-box");
}

#[test]
fn every_missing_priority_or_mixed_wide_component_blocks_shorthand_serialization() {
    let original = parse("background:red content-box padding-box!important");
    for property in NAMES {
        let mut missing = original.clone();
        assert!(
            !missing
                .remove_property(property, &mut WORK.clone())
                .unwrap()
                .is_empty()
        );
        assert_eq!(get(&missing, "background"), "");
        let mut priority = original.clone();
        let value = get(&priority, property);
        assert!(set(&mut priority, property, &value, ""));
        assert_eq!(get(&priority, "background"), "");
        for wide in ["inherit", "initial", "unset", "revert", "revert-layer"] {
            let mut mixed = original.clone();
            assert!(set(&mut mixed, property, wide, "important"));
            assert_eq!(get(&mixed, "background"), "");
            assert!(set(&mut mixed, "background", wide, "important"));
            assert_eq!(get(&mixed, "background"), wide);
            assert_eq!(mixed.len(), 7);
            for name in NAMES {
                assert_eq!(get(&mixed, name), wide);
            }
        }
    }
}

#[test]
fn pending_seven_targets_keep_identity_priority_and_atomic_removal() {
    let mut pending = parse("background:var(--Saved)");
    assert_eq!(pending.len(), 1);
    assert_eq!(get(&pending, "background"), "var(--Saved)");
    assert_eq!(
        pending.records[0].pending.as_ref().unwrap(),
        &NAMES.map(str::to_owned)
    );
    assert!(set(&mut pending, "background-origin", "content-box", ""));
    assert_eq!(get(&pending, "background"), "");
    assert_eq!(get(&pending, "background-origin"), "content-box");
    assert_eq!(
        pending
            .remove_property("background", &mut WORK.clone())
            .unwrap(),
        ""
    );
    assert_eq!(pending, InlineStyle::default());
    let mut important = parse("background:var(--Saved)!important");
    let saved = important.clone();
    assert!(!set(&mut important, "background-clip", "content-box", ""));
    assert_eq!(important, saved);
    assert!(set(
        &mut important,
        "background-clip",
        "content-box",
        "important"
    ));
    assert_eq!(get(&important, "background"), "");
    assert_eq!(parse(&text(&important)), important);
}

#[test]
fn cssom_wide_and_var_records_reach_computed_box_values() {
    for (value, expected) in [
        (
            "inherit",
            (css::BackgroundBox::Content, css::BackgroundBox::Padding),
        ),
        (
            "initial",
            (css::BackgroundBox::Padding, css::BackgroundBox::Border),
        ),
        (
            "unset",
            (css::BackgroundBox::Padding, css::BackgroundBox::Border),
        ),
        (
            "var(--good)",
            (css::BackgroundBox::Content, css::BackgroundBox::Content),
        ),
        (
            "var(--bad)",
            (css::BackgroundBox::Padding, css::BackgroundBox::Border),
        ),
    ] {
        let mut style = parse(
            "--good:red content-box;--bad:red content-box padding-box border-box;background:blue border-box content-box",
        );
        assert!(set(&mut style, "background", value, ""));
        let doc = crate::dom::Document::parse(&format!(
            "<div style='background-origin:content-box;background-clip:padding-box'><i id=x style='{}'></i></div>",
            text(&style)
        ));
        let styles = css::compute_styles(&doc, &[], 100.0, 100.0);
        let child = &styles[doc.query_selector("#x").unwrap()];
        assert_eq!(
            (child.backgrounds.origins()[0], child.backgrounds.clips()[0]),
            expected
        );
    }
    let all = parse("background:red content-box;all:initial");
    assert_eq!(get(&all, "background-origin"), "initial");
    assert_eq!(get(&all, "background-clip"), "initial");
}

#[test]
fn new_box_records_keep_literal_one_pass_admission_and_old_work_limits() {
    // Single Ident length n: records scans12n+12, two validators24n+2,
    // shared box parser5n+9, normalized storage n+65 =>42n+88.
    for property in ["background-origin", "background-clip"] {
        for (value, cost) in [
            ("border-box", 508),
            ("padding-box", 550),
            ("content-box", 550),
        ] {
            let mut work = cost;
            assert_eq!(
                records(property, value, true, &mut work),
                Ok(Some(vec![Record {
                    name: property.into(),
                    value: value.into(),
                    important: true,
                    pending: None,
                }]))
            );
            assert_eq!(work, 0);
            for mut cut in [0, 1, cost - 1] {
                assert_eq!(records(property, value, true, &mut cut), Err(Error::Limit));
                assert_eq!(cut, 0);
            }
        }
    }
}

#[test]
fn literal_setter_budget_refusal_publishes_no_partial_record() {
    // Empty block: API bytes + records fee + two property query scans +1 staging.
    for (property, value, cost) in [
        ("background-origin", "border-box", 582),
        ("background-origin", "padding-box", 625),
        ("background-clip", "border-box", 576),
        ("background-clip", "content-box", 619),
    ] {
        let mut exact = InlineStyle::default();
        let mut work = cost;
        assert_eq!(
            exact.set_property(property, value, "important", &mut work),
            Ok(true)
        );
        assert_eq!(work, 0);
        assert_eq!(get(&exact, property), value);
        for mut cut in [0, 1, cost - 1] {
            let mut refused = InlineStyle::default();
            assert_eq!(
                refused.set_property(property, value, "important", &mut cut),
                Err(Error::Limit)
            );
            assert_eq!(refused, InlineStyle::default());
        }
    }
}

#[test]
fn seven_record_expansion_respects_existing256_record_limit_transactionally() {
    for custom_count in [249, 250] {
        let mut style = InlineStyle {
            records: (0..custom_count)
                .map(|index| Record {
                    name: format!("--saved{index}"),
                    value: "0".into(),
                    important: false,
                    pending: None,
                })
                .collect(),
        };
        let before = style.clone();
        let result = style.set_property(
            "background",
            "red content-box padding-box",
            "",
            &mut WORK.clone(),
        );
        if custom_count == 249 {
            assert_eq!(result, Ok(true));
            assert_eq!(style.len(), 256);
            assert_eq!(get(&style, "background-origin"), "content-box");
            assert_eq!(get(&style, "background-clip"), "padding-box");
            assert_eq!(&style.records[..249], &before.records);
        } else {
            assert_eq!(result, Err(Error::Limit));
            assert_eq!(style, before);
        }
    }
}
