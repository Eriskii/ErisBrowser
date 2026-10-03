use super::*;

const CASES: &str = include_str!("../../../tests/fixtures/document-title.js");
fn fresh() -> (Runtime, Document) {
    (
        Runtime::try_new().unwrap(),
        Document::parse("<!doctype html><html><head></head><body><p>kept</p></body></html>"),
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
        let source = format!("{CASES}\ndocumentTitleCases.{name}();");
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
    direct_text_ascii_whitespace,
    exact_units_and_non_ascii_spaces,
    setter_exact_fresh_text_and_retention,
    nullish_symbol_and_abrupt_conversion,
    callback_selects_current_title,
    connected_order_and_template_trees,
    missing_title_empty_creation_and_missing_head,
    html_namespace_non_html_root,
    no_root_and_foreign_root_convert_before_ignore,
    svg_root_first_direct_svg_title,
    svg_missing_title_creates_first_child,
    ordinary_descriptors_shadows_deletion_and_brands,
);

#[test]
fn title_bootstrap_admission() {
    let runtime = Runtime::uninitialized().finish_bootstrap().unwrap();
    println!(
        "TITLE_BOOTSTRAP steps={} allocated={} objects={} capacity={} native={} prototypes={}",
        runtime.steps,
        runtime.allocated,
        runtime.objects.len(),
        runtime.objects.capacity(),
        runtime.native_properties.len(),
        runtime.prototypes.len()
    );
    assert_eq!(runtime.objects.len(), dom_prototypes::BOOTSTRAP_OBJECTS);
    assert!(runtime.steps > 0);
    assert!(runtime.allocated < MAX_HEAP);
    clean(&runtime);
}

#[test]
fn title_function_bags_are_isolated_and_attribute_precedes_operations() {
    let (mut runtime, mut doc) = fresh();
    assert_eq!(runtime.execute(r#"(function(){
        const names=Object.getOwnPropertyNames(Document.prototype);
        if(names[0]!=='title'||names[names.length-1]!=='constructor')throw new Error('attribute order');
        const d=Object.getOwnPropertyDescriptor(Document.prototype,'title');
        if(d.get===d.set||Object.getPrototypeOf(d.get)!==Function.prototype||Object.getPrototypeOf(d.set)!==Function.prototype)throw new Error('identity');
        for(const f of [d.get,d.set]){
            const n=Object.getOwnPropertyDescriptor(f,'name'),l=Object.getOwnPropertyDescriptor(f,'length');
            if(n.writable||n.enumerable||!n.configurable||l.writable||l.enumerable||!l.configurable||Object.getOwnPropertyDescriptor(f,'prototype')!==undefined)throw new Error('metadata');
            let caught=false;try{Reflect.construct(f,[]);}catch(e){caught=e instanceof TypeError;}
            if(!caught)throw new Error('constructibility');
        }
        Object.defineProperty(d.get,'name',{value:'changed'});
        if(d.set.name!=='set title')throw new Error('shared bag');
        return true;
    })()"#, &mut doc).unwrap(),Value::Bool(true));
    let (mut other, mut other_doc) = fresh();
    assert_eq!(
        other
            .execute(
                "Object.getOwnPropertyDescriptor(Document.prototype,'title').get.name",
                &mut other_doc
            )
            .unwrap(),
        Value::String("get title".into())
    );
    clean(&runtime);
    clean(&other);
}

fn existing(full: bool) -> (Runtime, Document, NodeId) {
    let (runtime, mut doc) = fresh();
    let title = doc.create_element("title");
    let head = doc.query_selector("head").unwrap();
    let text = doc.create_text_node(" old ");
    doc.append_child(title, text);
    doc.append_child(head, title);
    if full {
        doc.nodes.shrink_to_fit();
    } else {
        doc.nodes.try_reserve_exact(1).unwrap();
    }
    (runtime, doc, title)
}
fn fill_nodes(doc: &mut Document, count: usize) {
    doc.nodes.resize_with(count, || crate::dom::Node {
        parent: None,
        children: Vec::new(),
        kind: NodeKind::Comment(String::new().into()),
    });
}

#[test]
fn genuine_document_alias_and_invalid_receivers_check_brand_before_conversion() {
    let (mut runtime, mut doc, title) = existing(false);
    let value = runtime
        .execute(
            "var titleConversions=0;({toString(){titleConversions++;return 'changed';}})",
            &mut doc,
        )
        .unwrap();
    for receiver in [
        Value::Node(title),
        Value::Node(doc.nodes.len()),
        Value::Node(usize::MAX),
        Value::Object(0),
        Value::Null,
    ] {
        for method in ["get", "set"] {
            assert_eq!(
                runtime
                    .document_title_native(
                        method,
                        receiver.clone(),
                        std::slice::from_ref(&value),
                        &mut doc
                    )
                    .unwrap_err()
                    .name(),
                "TypeError"
            );
        }
    }
    assert_eq!(
        runtime.environments[0].bindings["titleConversions"].value,
        Value::Number(0.0)
    );
    let root = Value::Node(doc.root);
    runtime
        .document_title_native("set", root.clone(), std::slice::from_ref(&value), &mut doc)
        .unwrap();
    assert_eq!(
        runtime
            .document_title_native("get", root, &[], &mut doc)
            .unwrap(),
        Value::String("changed".into())
    );
    assert_eq!(
        runtime.environments[0].bindings["titleConversions"].value,
        Value::Number(1.0)
    );
    clean(&runtime);
}

#[test]
fn ignored_no_root_branch_needs_no_payload_or_node_allocation() {
    let (mut runtime, mut doc) = fresh();
    let root = document_element(&doc).unwrap();
    doc.remove_child(doc.root, root);
    fill_nodes(&mut doc, MAX_NODES);
    let input = Value::String(vec![0xd800; 40_000].into());
    let original = format!("{doc:?}");
    runtime.steps = 1000;
    runtime.allocated = MAX_HEAP;
    assert_eq!(
        runtime
            .document_title_native("set", Value::Document, &[input], &mut doc)
            .unwrap(),
        Value::Undefined
    );
    assert_eq!(runtime.allocated, MAX_HEAP);
    assert_eq!(format!("{doc:?}"), original);
    assert!(runtime.steps > 0);
    clean(&runtime);
}

#[test]
fn existing_title_setter_measured_exact_and_one_short_has_no_partial_replacement() {
    let input = Value::String(vec![32, 0xd800, 9, 65, 32].into());
    for full in [false, true] {
        let (mut runtime, mut doc, _) = existing(full);
        let before = (runtime.steps, runtime.allocated);
        runtime
            .document_title_native(
                "set",
                Value::Document,
                std::slice::from_ref(&input),
                &mut doc,
            )
            .unwrap();
        let work = before.0 - runtime.steps;
        let bytes = runtime.allocated - before.1;
        let expected = format!("{doc:?}");
        for (steps, heap, success) in [
            (work, bytes, true),
            (work - 1, bytes, false),
            (work, bytes - 1, false),
        ] {
            let (mut runtime, mut doc, _) = existing(full);
            let original = format!("{doc:?}");
            let capacity = doc.nodes.capacity();
            runtime.steps = steps;
            runtime.allocated = MAX_HEAP - heap;
            let result = runtime.document_title_native(
                "set",
                Value::Document,
                std::slice::from_ref(&input),
                &mut doc,
            );
            if success {
                assert_eq!(result.unwrap(), Value::Undefined);
                assert_eq!(format!("{doc:?}"), expected);
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert_eq!(format!("{doc:?}"), original);
                assert_eq!(doc.nodes.capacity(), capacity);
                if steps == work - 1 {
                    assert_eq!(runtime.steps, 0);
                } else {
                    assert!(runtime.allocated > MAX_HEAP);
                }
            }
            clean(&runtime);
        }
    }
}

#[test]
fn normalized_getter_measured_exact_and_one_short_includes_final_rc_copy() {
    fn setup() -> (Runtime, Document) {
        let (runtime, mut doc, title) = existing(false);
        doc.clear_children(title);
        for units in [vec![9, 0xd800], vec![0xdc00, 32, 32, 65, 9]] {
            let node = doc
                .create_text_node_owned(crate::dom::DomString::from_units_owned(units).unwrap())
                .unwrap();
            doc.append_child(title, node);
            let comment = doc.create_comment("ignored");
            doc.append_child(title, comment);
        }
        (runtime, doc)
    }
    let (mut runtime, mut doc) = setup();
    let before = (runtime.steps, runtime.allocated);
    let expected = runtime
        .document_title_native("get", Value::Document, &[], &mut doc)
        .unwrap();
    assert_eq!(expected, Value::String(vec![0xd800, 0xdc00, 32, 65].into()));
    let work = before.0 - runtime.steps;
    let heap = runtime.allocated - before.1;
    for (steps, bytes, success) in [
        (work, heap, true),
        (work - 1, heap, false),
        (work, heap - 1, false),
    ] {
        let (mut runtime, mut doc) = setup();
        let original = format!("{doc:?}");
        let capacity = doc.nodes.capacity();
        runtime.steps = steps;
        runtime.allocated = MAX_HEAP - bytes;
        let result = runtime.document_title_native("get", Value::Document, &[], &mut doc);
        if success {
            assert_eq!(result.unwrap(), expected);
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            if steps == work - 1 {
                assert_eq!(runtime.steps, 0);
            } else {
                assert!(runtime.allocated > MAX_HEAP);
            }
        }
        assert_eq!(format!("{doc:?}"), original);
        assert_eq!(doc.nodes.capacity(), capacity);
        clean(&runtime);
    }
}

#[test]
fn missing_title_consumes_last_slot_before_nonempty_content_is_refused() {
    for empty in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let head = doc.query_selector("head").unwrap();
        fill_nodes(&mut doc, MAX_NODES - 1);
        doc.nodes.try_reserve_exact(1).unwrap();
        let count = doc.nodes.len();
        let bytes = doc.retained_bytes();
        let input = Value::String(if empty {
            JsString::default()
        } else {
            vec![0xd800].into()
        });
        let result = runtime.document_title_native("set", Value::Document, &[input], &mut doc);
        if empty {
            assert_eq!(result.unwrap(), Value::Undefined);
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        assert_eq!(doc.nodes.len(), count + 1);
        assert_eq!(doc.retained_bytes(), bytes + 5);
        assert_eq!(doc.nodes[head].children, [count]);
        assert_eq!(doc.nodes[count].parent, Some(head));
        assert_eq!(doc.namespace(count), Some(Namespace::Html));
        assert_eq!(doc.tag(count), Some("title"));
        assert!(doc.nodes[count].children.is_empty());
        clean(&runtime);
    }
}

#[test]
fn new_title_work_refusals_retain_only_the_reached_creation_and_insertion_prefix() {
    let input = Value::String(vec![65, 0xd800].into());
    let (mut runtime, mut doc) = fresh();
    let before = runtime.steps;
    runtime
        .document_title_native(
            "set",
            Value::Document,
            std::slice::from_ref(&input),
            &mut doc,
        )
        .unwrap();
    let work = before - runtime.steps;
    let mut phases = [false; 3];
    for remaining in 0..work {
        let (mut runtime, mut doc) = fresh();
        let head = doc.query_selector("head").unwrap();
        let count = doc.nodes.len();
        let bytes = doc.retained_bytes();
        let original = format!("{doc:?}");
        runtime.steps = remaining;
        assert!(
            runtime
                .document_title_native(
                    "set",
                    Value::Document,
                    std::slice::from_ref(&input),
                    &mut doc
                )
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.steps, 0);
        match doc.nodes.len() - count {
            0 => {
                phases[0] = true;
                assert_eq!(format!("{doc:?}"), original);
            }
            1 => {
                assert_eq!(doc.tag(count), Some("title"));
                assert_eq!(doc.retained_bytes(), bytes + 5);
                assert!(doc.nodes[count].children.is_empty());
                if doc.nodes[count].parent.is_none() {
                    phases[1] = true;
                    assert!(doc.nodes[head].children.is_empty());
                } else {
                    phases[2] = true;
                    assert_eq!(doc.nodes[count].parent, Some(head));
                    assert_eq!(doc.nodes[head].children, [count]);
                }
            }
            other => panic!("unexpected outer publication prefix {other}"),
        }
        clean(&runtime);
    }
    assert_eq!(phases, [true, true, true]);
}

#[test]
fn title_replacement_checks_callback_current_node_and_byte_capacity() {
    for bytes in [false, true] {
        let (mut runtime, mut doc, title) = existing(false);
        let value=runtime.execute("var titleEffect=0;({toString(){titleEffect++;document.createTextNode('\\ud800');return '\\udc00';}})",&mut doc).unwrap();
        if bytes {
            while doc.retained_bytes() < crate::dom::MAX_DOM_BYTES - 2 {
                let remaining = crate::dom::MAX_DOM_BYTES - 2 - doc.retained_bytes();
                assert_ne!(
                    doc.create_text_node(&"x".repeat(remaining.min(8 * 1024 * 1024))),
                    doc.root
                );
            }
        } else {
            fill_nodes(&mut doc, MAX_NODES - 1);
        }
        doc.nodes.try_reserve_exact(1).unwrap();
        let count = doc.nodes.len();
        let children = doc.nodes[title].children.clone();
        assert!(
            runtime
                .document_title_native(
                    "set",
                    Value::Document,
                    std::slice::from_ref(&value),
                    &mut doc
                )
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(
            runtime.environments[0].bindings["titleEffect"].value,
            Value::Number(1.0)
        );
        assert_eq!(doc.nodes.len(), count + 1);
        assert_eq!(doc.nodes[title].children, children);
        assert!(doc.nodes[count].parent.is_none());
        let NodeKind::Text(data) = &doc.nodes[count].kind else {
            panic!()
        };
        assert_eq!(data.units().collect::<Vec<_>>(), [0xd800]);
        if bytes {
            assert_eq!(doc.retained_bytes(), crate::dom::MAX_DOM_BYTES);
        }
        clean(&runtime);
    }
}

#[test]
fn terminal_title_conversion_retains_author_effects_without_outer_creation_or_catch() {
    let (mut runtime, mut doc) = fresh();
    let error=runtime.execute("var titleTrace=0;try{document.title={toString(){titleTrace=1;document.body.appendChild(new Text('prefix'));while(true){};}};}catch(e){titleTrace=2;}finally{titleTrace=3;}",&mut doc).unwrap_err();
    assert!(error.is_resource_limit());
    assert_eq!(
        runtime.environments[0].bindings["titleTrace"].value,
        Value::Number(1.0)
    );
    assert!(doc.query_selector("title").is_none());
    assert_eq!(
        doc.text_content(doc.query_selector("body").unwrap()),
        "keptprefix"
    );
    clean(&runtime);
}
