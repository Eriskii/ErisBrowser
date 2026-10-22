//! Independent all-reset controls frozen before the merge optimization.
use super::*;
const WORK: usize = 2_000_000;
fn value(style: &InlineStyle, name: &str) -> String {
    style.get_property_value(name, &mut WORK.clone()).unwrap()
}
#[test]
fn pointer_events_all_initial_fits_a_bounded_parse_and_preserves_the_full_roster() {
    let mut work = 80_000;
    let style = InlineStyle::parse("all:initial", &mut work).unwrap();
    assert_eq!(style.len(), 88);
    assert_eq!(style.item(0), "display");
    assert_eq!(style.item(1), "visibility");
    assert_eq!(style.item(2), "pointer-events");
    assert_eq!(style.item(87), "vertical-align");
    for index in 0..88 {
        assert_eq!(value(&style, style.item(index)), "initial");
        assert_eq!(
            style
                .get_property_priority(style.item(index), &mut WORK.clone())
                .unwrap(),
            ""
        );
    }
    let text = style.serialize(&mut WORK.clone()).unwrap();
    let roundtrip = InlineStyle::parse(&text, &mut WORK.clone()).unwrap();
    assert_eq!(roundtrip.len(), 88);
    assert_eq!(roundtrip.serialize(&mut WORK.clone()).unwrap(), text);
}
#[test]
fn pointer_events_all_merge_preserves_earlier_importance_and_later_source_order() {
    let style = InlineStyle::parse(
        "color:red!important;all:initial;pointer-events:none",
        &mut WORK.clone(),
    )
    .unwrap();
    assert_eq!(style.len(), 88);
    assert_eq!(style.item(0), "color");
    assert_eq!(style.item(87), "pointer-events");
    assert_eq!(value(&style, "color"), "red");
    assert_eq!(
        style
            .get_property_priority("color", &mut WORK.clone())
            .unwrap(),
        "important"
    );
    assert_eq!(value(&style, "pointer-events"), "none");
    assert_eq!(value(&style, "display"), "initial");
    let style = InlineStyle::parse(
        "all:initial;all:inherit!important;pointer-events:none",
        &mut WORK.clone(),
    )
    .unwrap();
    assert_eq!(style.len(), 88);
    assert_eq!(value(&style, "pointer-events"), "inherit");
    assert_eq!(
        style
            .get_property_priority("pointer-events", &mut WORK.clone())
            .unwrap(),
        "important"
    );
}
#[test]
fn pointer_events_all_pending_keeps_shared_origin_and_partial_override_semantics() {
    let style = InlineStyle::parse("all:var(--reset)", &mut 80_000).unwrap();
    assert_eq!(style.len(), 88);
    assert_eq!(value(&style, "all"), "var(--reset)");
    assert_eq!(value(&style, "pointer-events"), "");
    assert_eq!(
        style.serialize(&mut WORK.clone()).unwrap(),
        "all: var(--reset);"
    );
    let style =
        InlineStyle::parse("all:var(--reset);pointer-events:none", &mut WORK.clone()).unwrap();
    assert_eq!(style.len(), 88);
    assert_eq!(style.item(87), "pointer-events");
    assert_eq!(value(&style, "all"), "");
    assert_eq!(value(&style, "pointer-events"), "none");
    assert_eq!(value(&style, "display"), "");
}
