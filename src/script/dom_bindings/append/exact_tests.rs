use super::*;
use crate::dom::Node;

const CASES: &str = include_str!("../../../../tests/fixtures/append-domstrings.js");
fn fresh() -> (Runtime, Document) {
    (
        Runtime::new(),
        Document::parse("<!doctype html><p>kept</p>"),
    )
}
fn clean(runtime: &Runtime) {
    assert!(runtime.frames.is_empty());
    assert_eq!(
        (runtime.calls, runtime.eval_depth, runtime.stack_units),
        (0, 0, 0)
    );
}
fn independent(name: &str) {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = format!("{CASES}\nappendExactCases.{name}();");
        let result = if strict {
            runtime.execute_strict(&source, &mut doc)
        } else {
            runtime.execute(&source, &mut doc)
        };
        assert_eq!(result.unwrap(), Value::Bool(true), "{name} strict={strict}");
        clean(&runtime);
    }
}
macro_rules! cases {($($name:ident),* $(,)?)=>{$(#[test]fn $name(){independent(stringify!($name));})*};}
cases!(
    separate_text_arguments_keep_units_and_identity,
    empty_and_nullish_are_actual_string_arguments,
    conversions_complete_before_moves_and_nodes_are_not_coerced,
    abrupt_later_conversion_keeps_author_prefix_without_moves,
    final_document_failure_retains_exact_assembled_fragment,
    cycle_failure_keeps_full_exact_argument_order,
    defining_interface_brand_precedes_string_conversion,
);

#[test]
fn materialization_exact_and_one_short_precede_current_text_publication() {
    fn setup(full: bool) -> (Runtime, Document) {
        let (runtime, mut doc) = fresh();
        if full {
            doc.nodes.shrink_to_fit();
        } else {
            doc.nodes.try_reserve_exact(1).unwrap();
        }
        (runtime, doc)
    }
    for text in [
        JsString::default(),
        JsString::from("A\u{1d11e}"),
        JsString::from(vec![65, 0xd800, 0xdc00, 0xdfff]),
    ] {
        for full in [false, true] {
            let (mut runtime, mut doc) = setup(full);
            let before = (runtime.steps, runtime.allocated);
            let id = runtime.dom_append_text(&text, &mut doc).unwrap();
            let work = before.0 - runtime.steps;
            let heap = runtime.allocated - before.1;
            let expected = format!("{doc:?}");
            assert!(doc.nodes[id].parent.is_none());
            let NodeKind::Text(data) = &doc.nodes[id].kind else {
                panic!()
            };
            assert_eq!(data.units().collect::<Vec<_>>(), text.units());
            for (steps, bytes, success) in [
                (work, heap, true),
                (work - 1, heap, false),
                (work, heap - 1, false),
            ] {
                let (mut runtime, mut doc) = setup(full);
                let original = format!("{doc:?}");
                let capacity = doc.nodes.capacity();
                let count = doc.nodes.len();
                runtime.steps = steps;
                runtime.allocated = MAX_HEAP - bytes;
                let result = runtime.dom_append_text(&text, &mut doc);
                if success {
                    assert_eq!(result.unwrap(), count);
                    assert_eq!(format!("{doc:?}"), expected);
                    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
                } else {
                    assert!(result.unwrap_err().is_resource_limit());
                    assert_eq!(format!("{doc:?}"), original);
                    assert_eq!(doc.nodes.capacity(), capacity);
                }
                clean(&runtime);
            }
        }
    }
}

fn move_setup() -> (Runtime, Document, NodeId, NodeId, NodeId) {
    let (runtime, mut doc) = fresh();
    let source = doc.create_element("aside");
    let target = doc.create_element("div");
    let child = doc.create_element("i");
    doc.append_child(source, child);
    (runtime, doc, source, target, child)
}
fn fill_nodes(doc: &mut Document, count: usize) {
    doc.nodes.resize_with(count, || Node {
        parent: None,
        children: Vec::new(),
        kind: NodeKind::Comment(String::new().into()),
    });
}
fn check_unmoved(doc: &Document, source: NodeId, target: NodeId, child: NodeId) {
    assert_eq!(doc.nodes[source].children, [child]);
    assert_eq!(doc.nodes[child].parent, Some(source));
    assert!(doc.nodes[target].children.is_empty());
}
fn exact_detached_text(doc: &Document, id: NodeId, units: &[u16]) {
    assert_eq!(doc.nodes[id].parent, None);
    let NodeKind::Text(data) = &doc.nodes[id].kind else {
        panic!()
    };
    assert_eq!(data.units().collect::<Vec<_>>(), units);
}

#[test]
fn second_text_node_cap_refusal_keeps_exact_first_text_and_unmoved_arguments() {
    let (mut runtime, mut doc, source, target, child) = move_setup();
    fill_nodes(&mut doc, MAX_NODES - 1);
    doc.nodes.try_reserve_exact(1).unwrap();
    let first = doc.nodes.len();
    let error = runtime
        .dom_append(
            target,
            &[
                Value::Node(child),
                Value::String(vec![0xd800].into()),
                Value::String(vec![0xdc00].into()),
            ],
            &mut doc,
        )
        .unwrap_err();
    assert!(error.is_resource_limit());
    assert_eq!(doc.nodes.len(), first + 1);
    exact_detached_text(&doc, first, &[0xd800]);
    check_unmoved(&doc, source, target, child);
    clean(&runtime);
}

#[test]
fn second_text_byte_refusal_keeps_exact_first_text_and_unmoved_arguments() {
    let (mut runtime, mut doc, source, target, child) = move_setup();
    while doc.retained_bytes() < crate::dom::MAX_DOM_BYTES - 2 {
        let remaining = crate::dom::MAX_DOM_BYTES - 2 - doc.retained_bytes();
        let id = doc.create_text_node(&"x".repeat(remaining.min(8 * 1024 * 1024)));
        assert_ne!(id, doc.root);
    }
    doc.nodes.try_reserve_exact(2).unwrap();
    let first = doc.nodes.len();
    let error = runtime
        .dom_append(
            target,
            &[
                Value::Node(child),
                Value::String(vec![0xd800].into()),
                Value::String("x".into()),
            ],
            &mut doc,
        )
        .unwrap_err();
    assert!(error.is_resource_limit());
    assert_eq!(doc.nodes.len(), first + 1);
    assert_eq!(doc.retained_bytes(), crate::dom::MAX_DOM_BYTES);
    exact_detached_text(&doc, first, &[0xd800]);
    check_unmoved(&doc, source, target, child);
    clean(&runtime);
}

#[test]
fn fragment_node_cap_refusal_occurs_after_all_exact_texts_materialize() {
    let (mut runtime, mut doc, source, target, child) = move_setup();
    fill_nodes(&mut doc, MAX_NODES - 2);
    doc.nodes.try_reserve_exact(2).unwrap();
    let first = doc.nodes.len();
    let error = runtime
        .dom_append(
            target,
            &[
                Value::Node(child),
                Value::String(vec![0xd800].into()),
                Value::String(vec![0xdc00].into()),
            ],
            &mut doc,
        )
        .unwrap_err();
    assert!(error.is_resource_limit());
    assert_eq!(doc.nodes.len(), first + 2);
    exact_detached_text(&doc, first, &[0xd800]);
    exact_detached_text(&doc, first + 1, &[0xdc00]);
    check_unmoved(&doc, source, target, child);
    clean(&runtime);
}

#[test]
fn actual_large_arena_growth_refuses_before_text_or_fragment_publication() {
    for fragment in [false, true] {
        let (mut runtime, mut doc, source, target, child) = move_setup();
        fill_nodes(&mut doc, MAX_NODES - 1);
        doc.nodes.shrink_to_fit();
        let count = doc.nodes.len();
        let capacity = doc.nodes.capacity();
        let bytes = doc.retained_bytes();
        assert_eq!(count, capacity);
        let result = if fragment {
            runtime.dom_append(target, &[Value::Node(child), Value::Node(child)], &mut doc)
        } else {
            runtime
                .dom_append_text(&JsString::from(vec![0xd800]), &mut doc)
                .map(|_| ())
        };
        assert!(result.unwrap_err().is_resource_limit());
        assert_eq!(runtime.steps, 0);
        // The work refusal precedes the unaffordable buffer charge.
        assert!(runtime.allocated < MAX_HEAP);
        assert_eq!(
            (doc.nodes.len(), doc.nodes.capacity(), doc.retained_bytes()),
            (count, capacity, bytes)
        );
        check_unmoved(&doc, source, target, child);
        clean(&runtime);
    }
}

#[test]
fn later_conversion_runs_before_large_exact_payload_materialization_refuses() {
    let (mut runtime, mut doc, source, target, child) = move_setup();
    let callback=runtime.execute("var exactAppendPrefix=0;({toString(){exactAppendPrefix++;document.createTextNode('callback');return 'last';}})",&mut doc).unwrap();
    let first = doc.nodes.len();
    let input = Value::String(vec![0xd800; 40_000].into());
    let error = runtime
        .dom_append(target, &[Value::Node(child), input, callback], &mut doc)
        .unwrap_err();
    assert!(error.is_resource_limit());
    assert_eq!(
        runtime.environments[0].bindings["exactAppendPrefix"].value,
        Value::Number(1.0)
    );
    assert_eq!(doc.nodes.len(), first + 1);
    exact_detached_text(&doc, first, &[99, 97, 108, 108, 98, 97, 99, 107]);
    check_unmoved(&doc, source, target, child);
    clean(&runtime);
}

#[test]
fn callback_consumption_of_last_slot_precedes_fresh_outer_admission() {
    let (mut runtime, mut doc, source, target, child) = move_setup();
    runtime.execute("var exactAppendTrace='';var first={toString(){exactAppendTrace+='A';document.createTextNode('\\ud800');return '\\udc00';}},last={toString(){exactAppendTrace+='B';return 'last';}};",&mut doc).unwrap();
    fill_nodes(&mut doc, MAX_NODES - 1);
    doc.nodes.try_reserve_exact(1).unwrap();
    let count = doc.nodes.len();
    let first = runtime.environments[0].bindings["first"].value.clone();
    let last = runtime.environments[0].bindings["last"].value.clone();
    let error = runtime
        .dom_append(target, &[Value::Node(child), first, last], &mut doc)
        .unwrap_err();
    assert!(error.is_resource_limit());
    assert_eq!(
        runtime.environments[0].bindings["exactAppendTrace"].value,
        Value::String("AB".into())
    );
    assert_eq!(doc.nodes.len(), count + 1);
    exact_detached_text(&doc, count, &[0xd800]);
    check_unmoved(&doc, source, target, child);
    clean(&runtime);
}

#[test]
fn append_canonicalizes_each_argument_without_merging_its_text_nodes() {
    let (mut runtime, mut doc) = fresh();
    let target = doc.create_element("div");
    runtime
        .dom_append(
            target,
            &[
                Value::String(vec![0xd800, 0xdc00].into()),
                Value::String(vec![0xd800].into()),
                Value::String(vec![0xdc00].into()),
                Value::String(JsString::default()),
            ],
            &mut doc,
        )
        .unwrap();
    let children = &doc.nodes[target].children;
    assert_eq!(children.len(), 4);
    let data: Vec<_> = children
        .iter()
        .map(|&id| {
            assert_eq!(doc.nodes[id].parent, Some(target));
            let NodeKind::Text(data) = &doc.nodes[id].kind else {
                panic!()
            };
            data
        })
        .collect();
    assert_eq!(data[0].scalar(), Some("\u{10000}"));
    assert_eq!(data[1].raw_units(), Some([0xd800].as_slice()));
    assert_eq!(data[2].raw_units(), Some([0xdc00].as_slice()));
    assert_eq!(data[3].scalar(), Some(""));
    assert_eq!(doc.text_content(target), "\u{10000}\u{10000}");
    clean(&runtime);
}

#[test]
fn callback_consumption_of_last_bytes_precedes_fresh_outer_admission() {
    let (mut runtime, mut doc, source, target, child) = move_setup();
    runtime.execute("var exactAppendTrace='';var first={toString(){exactAppendTrace+='A';document.createTextNode('\\ud800');return '\\udc00';}},last={toString(){exactAppendTrace+='B';return 'last';}};", &mut doc).unwrap();
    while doc.retained_bytes() < crate::dom::MAX_DOM_BYTES - 2 {
        let remaining = crate::dom::MAX_DOM_BYTES - 2 - doc.retained_bytes();
        let id = doc.create_text_node(&"x".repeat(remaining.min(8 * 1024 * 1024)));
        assert_ne!(id, doc.root);
    }
    doc.nodes.try_reserve_exact(1).unwrap();
    let count = doc.nodes.len();
    let first = runtime.environments[0].bindings["first"].value.clone();
    let last = runtime.environments[0].bindings["last"].value.clone();
    let error = runtime
        .dom_append(target, &[Value::Node(child), first, last], &mut doc)
        .unwrap_err();
    assert!(error.is_resource_limit());
    assert_eq!(
        runtime.environments[0].bindings["exactAppendTrace"].value,
        Value::String("AB".into())
    );
    assert_eq!(
        (doc.nodes.len(), doc.retained_bytes()),
        (count + 1, crate::dom::MAX_DOM_BYTES)
    );
    exact_detached_text(&doc, count, &[0xd800]);
    check_unmoved(&doc, source, target, child);
    clean(&runtime);
}
