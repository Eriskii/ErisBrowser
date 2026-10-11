//! Independent cascade witnesses; no candidate helper supplies expected values.
use super::*;
use crate::script::{Runtime, Value};

fn execute(doc: &mut Document, source: &str) {
    assert_eq!(
        Runtime::new().execute(source, doc).unwrap(),
        Value::Bool(true)
    );
}

#[test]
fn typed_inline_traversal_has_literal_shared_work_boundaries() {
    let mut doc = Document::parse("<div id=a></div>");
    let id = doc.query_selector("#a").unwrap();
    execute(
        &mut doc,
        "var s=document.getElementById('a').style;s.setProperty('margin','var(--m)','important');s.marginLeft='9px';true;",
    );
    // One typed entry lookup 10; four-record visit 4; name/value bytes in order
    // 18,20,21,14. Prefix totals: 14,32,52,73,87. Pending source Arc is borrowed.
    for (initial, expected_count) in [(9, 0), (13, 0), (31, 0), (32, 1), (86, 3), (87, 4)] {
        let mut cascade = CascadedProperties::default();
        let mut work = initial;
        insert_inline_declarations(&doc, id, &mut cascade, 0, &mut work);
        assert_eq!(cascade.count, expected_count, "initial={initial}");
        assert_eq!(work, 0, "initial={initial}");
        assert_eq!(
            cascade.properties.contains_key("margin-left"),
            initial == 87
        );
        if expected_count > 0 {
            let candidate = cascade.properties["margin-top"].values().next().unwrap();
            assert_eq!(candidate.value, "var(--m)");
            assert_eq!(candidate.pending_shorthand.as_deref(), Some("margin"));
            assert!(candidate.priority.0);
            assert!(candidate.priority.1);
        }
    }
    // An empty typed-map lookup is still prepaid, and cannot silently reset the
    // budget or fall back to parsing a style attribute when it is refused.
    let ordinary = Document::parse("<div id=a style='margin-left:9px'></div>");
    let mut work = 7;
    let mut cascade = CascadedProperties::default();
    insert_inline_declarations(
        &ordinary,
        ordinary.query_selector("#a").unwrap(),
        &mut cascade,
        0,
        &mut work,
    );
    assert_eq!((work, cascade.count), (0, 0));
}

#[test]
fn allocation_free_shorthand_count_matches_supported_literal_rosters() {
    for (name, count) in [
        ("all", 86),
        ("background", 7),
        ("border", 12),
        ("border-top", 3),
        ("margin", 4),
        ("border-width", 4),
        ("padding-inline", 2),
        ("gap", 2),
        ("grid-area", 4),
        ("font", 5),
        ("flex", 3),
        ("flex-flow", 2),
        ("place-items", 2),
        ("list-style", 2),
    ] {
        assert_eq!(shorthand_property_count(name), Some(count), "{name}");
        assert_eq!(shorthand_properties(name).unwrap().len(), count, "{name}");
    }
    assert_eq!(shorthand_property_count("color"), None);
    assert_eq!(shorthand_property_count("bogus"), None);
}
