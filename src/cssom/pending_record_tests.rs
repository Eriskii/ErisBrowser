use super::*;

const WORK: usize = 20_000_000;
fn parse(source: &str) -> InlineStyle {
    InlineStyle::parse(source, &mut WORK.clone()).unwrap()
}
fn value(style: &InlineStyle, property: &str) -> String {
    style
        .get_property_value(property, &mut WORK.clone())
        .unwrap()
}
fn priority(style: &InlineStyle, property: &str) -> &'static str {
    style
        .get_property_priority(property, &mut WORK.clone())
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
fn names(style: &InlineStyle) -> Vec<&str> {
    (0..style.len()).map(|index| style.item(index)).collect()
}

#[test]
fn pending_record_lowering_changes_one_side_and_keeps_variable_source() {
    let mut style = parse("--m:1px 2px 3px 4px;margin:var(--m)!important");
    assert_eq!(
        names(&style),
        [
            "--m",
            "margin-top",
            "margin-right",
            "margin-bottom",
            "margin-left"
        ]
    );
    assert!(set(&mut style, "margin-left", "9px", ""));
    assert_eq!(value(&style, "margin-left"), "9px");
    assert_eq!(priority(&style, "margin-left"), "");
    for side in ["margin-top", "margin-right", "margin-bottom"] {
        assert_eq!(value(&style, side), "");
        assert_eq!(priority(&style, side), "important");
    }
    assert_eq!(value(&style, "margin"), "");
    assert_eq!(
        text(&style),
        "--m: 1px 2px 3px 4px; margin-top:  !important; margin-right:  !important; margin-bottom:  !important; margin-left: 9px;"
    );
    assert!(set(&mut style, "--m", "5px 6px 7px 8px", ""));
    assert_eq!(value(&style, "margin-left"), "9px");
    assert_eq!(priority(&style, "margin-top"), "important");
    assert_eq!(
        text(&style),
        "--m: 5px 6px 7px 8px; margin-top:  !important; margin-right:  !important; margin-bottom:  !important; margin-left: 9px;"
    );
    assert!(set(&mut style, "margin-left", "9px", "important"));
    assert!(set(&mut style, "margin-left", "9px", ""));
    assert_eq!(priority(&style, "margin-left"), "");
}

#[test]
fn pending_record_partial_removal_returns_empty_but_deletes_only_target() {
    let mut style = parse("--m:1px 2px 3px 4px;margin:var(--m)!important");
    assert_eq!(
        style
            .remove_property("margin-left", &mut WORK.clone())
            .unwrap(),
        ""
    );
    assert_eq!(
        names(&style),
        ["--m", "margin-top", "margin-right", "margin-bottom"]
    );
    assert_eq!(priority(&style, "margin-left"), "");
    assert_eq!(priority(&style, "margin-top"), "important");
    assert_eq!(
        text(&style),
        "--m: 1px 2px 3px 4px; margin-top:  !important; margin-right:  !important; margin-bottom:  !important;"
    );
    assert!(set(&mut style, "--m", "5px 6px 7px 8px", ""));
    assert_eq!(style.len(), 4);
    assert_eq!(value(&style, "margin-left"), "");
    assert!(set(&mut style, "margin-top", "10px", ""));
    assert_eq!(
        style
            .remove_property("margin-top", &mut WORK.clone())
            .unwrap(),
        "10px"
    );
    assert_eq!(names(&style), ["--m", "margin-right", "margin-bottom"]);
    assert_eq!(
        style.remove_property("margin", &mut WORK.clone()).unwrap(),
        ""
    );
    assert_eq!(names(&style), ["--m"]);
}

#[test]
fn pending_record_complete_getters_and_literal_subshorthand_stay_distinct() {
    let mut style = parse("border:var(--b)!important");
    assert_eq!(style.len(), 12);
    assert_eq!(value(&style, "border"), "var(--b)");
    assert_eq!(text(&style), "border: var(--b) !important;");
    for property in [
        "border-width",
        "border-style",
        "border-color",
        "border-left",
        "border-top-width",
    ] {
        assert_eq!(value(&style, property), "");
        assert_eq!(priority(&style, property), "important");
    }
    assert!(set(&mut style, "border-left", "3px solid blue", ""));
    assert_eq!(value(&style, "border-left"), "3px solid blue");
    assert_eq!(value(&style, "border"), "");
    assert_eq!(priority(&style, "border-left"), "");
    assert_eq!(priority(&style, "border-top-width"), "important");
    assert_eq!(
        text(&style),
        "border-top-width:  !important; border-top-style:  !important; border-top-color:  !important; border-right-width:  !important; border-right-style:  !important; border-right-color:  !important; border-bottom-width:  !important; border-bottom-style:  !important; border-bottom-color:  !important; border-left-width: 3px; border-left-style: solid; border-left-color: blue;"
    );
    assert!(set(&mut style, "border", "var(--b)", "important"));
    assert_eq!(value(&style, "border"), "var(--b)");
    assert_eq!(
        style.remove_property("border", &mut WORK.clone()).unwrap(),
        "var(--b)"
    );
    assert_eq!(style.len(), 0);
}

#[test]
fn pending_record_background_has_seven_slots_and_partial_text_is_lossy() {
    let mut style = parse("background:var(--Saved)!important");
    assert_eq!(
        names(&style),
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
    assert_eq!(value(&style, "background"), "var(--Saved)");
    assert!(set(&mut style, "background-clip", "content-box", ""));
    assert_eq!(priority(&style, "background-origin"), "important");
    assert_eq!(priority(&style, "background-clip"), "");
    assert_eq!(value(&style, "background-origin"), "");
    let serialized = text(&style);
    assert_eq!(
        serialized,
        "background-color:  !important; background-image:  !important; background-size:  !important; background-position:  !important; background-repeat:  !important; background-origin:  !important; background-clip: content-box;"
    );
    let reparsed = parse(&serialized);
    assert_eq!(names(&reparsed), ["background-clip"]);
    assert_eq!(style.len(), 7);
    assert_eq!(value(&reparsed, "background-clip"), "content-box");
    assert!(set(&mut style, "background", "var(--Saved)", "important"));
    assert_eq!(text(&style), "background: var(--Saved) !important;");
}

#[test]
fn pending_record_parse_importance_differs_from_setter_and_never_revives_old() {
    let mut style = parse("margin-left:7px; margin:var(--m)!important; margin-left:9px");
    assert_eq!(style.len(), 4);
    assert_eq!(value(&style, "margin"), "var(--m)");
    assert_eq!(value(&style, "margin-left"), "");
    assert_eq!(priority(&style, "margin-left"), "important");
    assert!(set(&mut style, "margin-left", "9px", ""));
    assert_eq!(value(&style, "margin-left"), "9px");
    assert_eq!(
        style
            .remove_property("margin-left", &mut WORK.clone())
            .unwrap(),
        "9px"
    );
    assert_eq!(value(&style, "margin-left"), "");
    assert_eq!(priority(&style, "margin-left"), "");
    assert_eq!(style.len(), 3);
    let mut direct = parse("margin:var(--m)!important");
    assert!(set(&mut direct, "margin-left", "var(--m)", "important"));
    assert_eq!(value(&direct, "margin-left"), "var(--m)");
    assert_eq!(value(&direct, "margin"), "");
}

#[test]
fn pending_record_setter_keeps_position_but_removed_target_reappends() {
    let mut style = parse("color:red;margin:var(--m)!important;width:1px");
    assert!(set(&mut style, "margin-left", "9px", ""));
    assert_eq!(
        names(&style),
        [
            "color",
            "margin-top",
            "margin-right",
            "margin-bottom",
            "margin-left",
            "width"
        ]
    );
    assert_eq!(style.item(6), "");
    assert_eq!(
        style
            .remove_property("margin-left", &mut WORK.clone())
            .unwrap(),
        "9px"
    );
    assert!(set(&mut style, "margin-left", "10px", ""));
    assert_eq!(
        names(&style),
        [
            "color",
            "margin-top",
            "margin-right",
            "margin-bottom",
            "width",
            "margin-left"
        ]
    );
    assert_eq!(
        text(&style),
        "color: red; margin-top:  !important; margin-right:  !important; margin-bottom:  !important; width: 1px; margin-left: 10px;"
    );
}

#[test]
fn pending_record_all_expansion_uses_same256_record_cap() {
    let style = parse("--a:initial;all:var(--a)");
    assert_eq!(style.len(), 89);
    assert_eq!(style.item(0), "--a");
    assert_eq!(style.item(1), "display");
    assert_eq!(style.item(22), "border-spacing");
    assert_eq!(style.item(88), "vertical-align");
    assert_eq!(style.item(89), "");
    assert_eq!(value(&style, "all"), "var(--a)");
    assert_eq!(text(&style), "--a: initial; all: var(--a);");
    for custom_count in [168, 169] {
        let source = (0..custom_count)
            .map(|i| format!("--c{i}:0;"))
            .collect::<String>();
        let mut style = parse(&source);
        let before = style.clone();
        let result = style.set_property("all", "var(--a)", "", &mut WORK.clone());
        if custom_count == 168 {
            assert_eq!(result, Ok(true));
            assert_eq!(style.len(), 256);
        } else {
            assert_eq!(result, Err(Error::Limit));
            assert_eq!(style, before);
        }
    }
}

#[test]
fn pending_record_logical_storage_exact64k_charges_each_shared_reference() {
    // 88 target names total 1073 bytes; 88*(728+4)+1073=65489.
    // --pad contributes 5+38+4=47, giving exactly 65536.
    let pending = format!("var(--x,{})", "0".repeat(719));
    assert_eq!(pending.len(), 728);
    let filler = "x".repeat(38);
    let source = format!("all:{pending};--pad:{filler}");
    let mut style = parse(&source);
    assert_eq!(style.len(), 89);
    assert_eq!(value(&style, "all"), pending);
    let before = style.clone();
    assert_eq!(
        style.set_property("--pad", &format!("{filler}x"), "", &mut WORK.clone()),
        Err(Error::Limit)
    );
    assert_eq!(style, before);
    assert_eq!(
        InlineStyle::parse(&format!("{source}x"), &mut WORK.clone()),
        Err(Error::Limit)
    );
}

#[test]
fn pending_record_invalid_and_unfunded_mutations_preserve_exact_block() {
    let original = parse("color:blue;margin:var(--m)!important");
    for (property, raw, priority) in [
        ("margin-left", "9px", "urgent"),
        ("margin-left", "nonsense", ""),
        ("background", "linear-gradient(", ""),
    ] {
        let mut style = original.clone();
        assert!(!set(&mut style, property, raw, priority));
        assert_eq!(style, original);
    }
    for work in [0, 1, 14] {
        let mut style = original.clone();
        assert_eq!(
            style.set_property("margin-left", "9px", "", &mut work.clone()),
            Err(Error::Limit)
        );
        assert_eq!(style, original);
    }
    let mut style = original.clone();
    assert_eq!(
        style.remove_property("margin-left", &mut 0),
        Err(Error::Limit)
    );
    assert_eq!(style, original);
}

#[test]
fn pending_record_serialization_exact_and_one_short_work_cuts_are_literal() {
    let mut style = parse("margin:var(--m)!important");
    let before = style.clone();
    let mut exact = 461;
    assert_eq!(
        style.serialize(&mut exact).unwrap(),
        "margin: var(--m) !important;"
    );
    assert_eq!(exact, 0);
    assert_eq!(style.serialize(&mut 460), Err(Error::Limit));
    assert_eq!(style, before);
    assert!(set(&mut style, "margin-left", "9px", ""));
    let before = style.clone();
    let mut exact = 413;
    assert_eq!(
        style.serialize(&mut exact).unwrap(),
        "margin-top:  !important; margin-right:  !important; margin-bottom:  !important; margin-left: 9px;"
    );
    assert_eq!(exact, 0);
    assert_eq!(style.serialize(&mut 412), Err(Error::Limit));
    assert_eq!(style, before);
}
