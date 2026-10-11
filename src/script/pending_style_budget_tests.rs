//! Independent Script/DOM wiring witnesses. Frozen before wiring implementation.
use super::*;

fn document() -> Document {
    Document::parse("<body><div id=target></div></body>")
}

fn partial(doc: &mut Document) -> NodeId {
    assert_eq!(
        Runtime::new().execute(
            "var s=document.getElementById('target').style; s.cssText='--m:1px 2px 3px 4px;margin:var(--m)!important'; s.marginLeft='9px'; s.marginLeft==='9px';",
            doc,
        ).unwrap(),
        Value::Bool(true),
    );
    doc.query_selector("#target").unwrap()
}

#[test]
fn typed_read_and_edit_refusal_never_parse_lossy_attribute() {
    let mut doc = document();
    let id = partial(&mut doc);
    let attribute = doc.attr(id, "style").unwrap().to_owned();
    let mut runtime = Runtime::new();
    runtime.steps = 9;
    assert!(
        runtime
            .style_read(id, 0, &doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.steps, 0);
    runtime.steps = 9;
    assert!(
        runtime
            .style_parse(id, 0, &doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.steps, 0);
    // One-entry typed lookup costs 10, borrows, and requires no scratch heap.
    runtime.steps = 10;
    runtime.allocated = MAX_HEAP;
    assert_eq!(runtime.style_read(id, 0, &doc).unwrap().len(), 5);
    assert_eq!(runtime.steps, 0);
    assert_eq!(runtime.allocated, MAX_HEAP);
    runtime.steps = MAX_STEPS;
    assert!(
        runtime
            .style_parse(id, 0, &doc)
            .unwrap_err()
            .is_resource_limit()
    );
    // Allocating borrowed getters still reserve the selected logical-size
    // operation scratch; metadata-only length remains an allocation-free read.
    runtime.steps = MAX_STEPS;
    runtime.allocated = MAX_HEAP - 4096;
    assert!(
        runtime
            .style_get(id, "cssText", &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    runtime.steps = MAX_STEPS;
    runtime.allocated = MAX_HEAP;
    assert_eq!(
        runtime.style_get(id, "length", &mut doc).unwrap(),
        Value::Number(5.0)
    );
    assert_eq!(runtime.allocated, MAX_HEAP);
    assert_eq!(doc.attr(id, "style"), Some(attribute.as_str()));
    assert_eq!(Runtime::new().execute(
        "document.getElementById('target').style.getPropertyPriority('margin-top')==='important';",
        &mut doc,
    ).unwrap(), Value::Bool(true));
}

#[test]
fn refused_prepared_commit_keeps_attribute_and_typed_state_together() {
    let mut doc = document();
    let id = partial(&mut doc);
    let attribute = doc.attr(id, "style").unwrap().to_owned();
    let mut runtime = Runtime::new();
    let mut staged = runtime.style_parse(id, 16, &doc).unwrap();
    assert!(
        staged
            .set_property("margin-left", "11px", "", &mut runtime.steps)
            .unwrap()
    );
    runtime.allocated = MAX_HEAP;
    assert!(
        runtime
            .style_commit(id, staged, &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(doc.attr(id, "style"), Some(attribute.as_str()));
    assert_eq!(
        Runtime::new()
            .execute(
                r#"
        var s=document.getElementById('target').style;
        s.length===5 && s.marginLeft==='9px' && s.marginTop==='' &&
        s.getPropertyPriority('margin-top')==='important';
    "#,
                &mut doc
            )
            .unwrap(),
        Value::Bool(true)
    );
}
