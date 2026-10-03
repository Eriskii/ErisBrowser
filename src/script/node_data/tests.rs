use super::*;

const ORIGINAL: &str = include_str!("../../../tests/fixtures/node-data.js");
const EXACT: &str = include_str!("../../../tests/fixtures/node-data-exact.js");

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
fn independent(source: &str, collection: &str, name: &str) {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = format!("{source}\n{collection}.{name}();");
        let result = if strict {
            runtime.execute_strict(&source, &mut doc)
        } else {
            runtime.execute(&source, &mut doc)
        };
        assert_eq!(result.unwrap(), Value::Bool(true), "{name} strict={strict}");
        clean(&runtime);
    }
}
macro_rules! cases {
    ($source:ident, $collection:literal, $($name:ident),* $(,)?) => {$ (
        #[test] fn $name() { independent($source, $collection, stringify!($name)); }
    )*};
}
cases!(
    ORIGINAL,
    "nodeDataCases",
    descriptors_and_represented_getters,
    nullable_setter_defaults,
    ignored_kinds_still_convert,
    authentic_brand_before_conversion,
    character_data_uses_current_state,
    container_replacement_uses_current_children,
    ordinary_shadow_and_prototype_deletion,
    template_content_is_separate,
);
cases!(
    EXACT,
    "exactNodeDataCases",
    character_data_exact_identity,
    cross_node_pair_and_text_only_aggregation,
    equal_container_write_still_creates_new_text,
    fragment_exact_replacement_and_nullable_conversion,
    ignored_kind_conversion_and_extra_arguments,
    exact_callback_children_and_abrupt_prefix,
    saved_intrinsics_ignore_own_shadows,
    base_selection_follows_replaced_children,
    details_groups_separate_after_detachment,
    template_exact_direct_children_are_separate,
);

#[test]
fn node_data_bootstrap_admission() {
    let runtime = Runtime::uninitialized().finish_bootstrap().unwrap();
    println!(
        "NODE_DATA_BOOTSTRAP steps={} allocated={} objects={} capacity={} native={} prototypes={}",
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
fn attributes_precede_operations_and_constants_and_have_separate_nonconstructable_function_bags() {
    let (mut runtime, mut doc) = fresh();
    let source = r#"(function(){
        const keys=Object.getOwnPropertyNames(Node.prototype);
        if(keys[0]!=='nodeValue'||keys[1]!=='textContent'||keys[2]!=='hasChildNodes'||
           keys[3]!=='normalize'||keys[4]!=='isSameNode'||keys[5]!=='contains'||
           keys[6]!=='ELEMENT_NODE'||
           keys[keys.length-1]!=='constructor')throw new Error('IDL order');
        const a=Object.getOwnPropertyDescriptor(Node.prototype,'nodeValue');
        const b=Object.getOwnPropertyDescriptor(Node.prototype,'textContent');
        const functions=[a.get,a.set,b.get,b.set];
        for(let i=0;i<functions.length;i++){
            if(Object.getPrototypeOf(functions[i])!==Function.prototype)throw new Error('Function prototype');
            let caught=false;try{Reflect.construct(functions[i],[]);}catch(e){caught=e instanceof TypeError;}
            if(!caught)throw new Error('accessor constructibility');
            for(let j=0;j<i;j++)if(functions[i]===functions[j])throw new Error('identity');
        }
        Object.defineProperty(a.get,'name',{value:'changed'});
        if(a.set.name!=='set nodeValue'||b.get.name!=='get textContent'||b.set.name!=='set textContent')throw new Error('bags');
        return true;
    })()"#;
    assert_eq!(
        runtime.execute(source, &mut doc).unwrap(),
        Value::Bool(true)
    );
    let (mut other, mut other_doc) = fresh();
    assert_eq!(
        other
            .execute(
                "Object.getOwnPropertyDescriptor(Node.prototype,'nodeValue').get.name",
                &mut other_doc
            )
            .unwrap(),
        Value::String("get nodeValue".into())
    );
    clean(&runtime);
    clean(&other);
}

#[test]
fn doctype_is_authentic_and_invalid_ids_fail_before_conversion() {
    let (mut runtime, mut doc) = fresh();
    let doctype = doc
        .nodes
        .iter()
        .position(|node| matches!(node.kind, NodeKind::Doctype(_)))
        .unwrap();
    let conversion = runtime
        .execute(
            "var nodeConverted=0;({toString(){nodeConverted++;return '\\ud800';}})",
            &mut doc,
        )
        .unwrap();
    for method in ["getNodeValue", "getTextContent"] {
        assert_eq!(
            runtime
                .node_data_native(method, Value::Node(doctype), &[], &mut doc)
                .unwrap(),
            Value::Null
        );
    }
    let before = format!("{doc:?}");
    for method in ["setNodeValue", "setTextContent"] {
        assert_eq!(
            runtime
                .node_data_native(
                    method,
                    Value::Node(doctype),
                    std::slice::from_ref(&conversion),
                    &mut doc
                )
                .unwrap(),
            Value::Undefined
        );
    }
    assert_eq!(
        runtime.environments[0].bindings["nodeConverted"].value,
        Value::Number(2.0)
    );
    for receiver in [
        Value::Node(doc.nodes.len()),
        Value::Node(usize::MAX),
        Value::Object(0),
        Value::Null,
    ] {
        for method in [
            "getNodeValue",
            "getTextContent",
            "setNodeValue",
            "setTextContent",
        ] {
            assert_eq!(
                runtime
                    .node_data_native(
                        method,
                        receiver.clone(),
                        std::slice::from_ref(&conversion),
                        &mut doc
                    )
                    .unwrap_err()
                    .name(),
                "TypeError"
            );
        }
    }
    assert_eq!(
        runtime.environments[0].bindings["nodeConverted"].value,
        Value::Number(2.0)
    );
    assert_eq!(format!("{doc:?}"), before);
    clean(&runtime);
}

fn fill_nodes(doc: &mut Document, count: usize) {
    while doc.nodes.len() < count {
        doc.nodes.push(crate::dom::Node {
            parent: None,
            children: Vec::new(),
            kind: NodeKind::Comment(String::new().into()),
        });
    }
}

#[test]
fn ignored_setters_need_no_dom_capacity_or_payload_storage() {
    let (mut runtime, mut doc) = fresh();
    let element = doc.query_selector("p").unwrap();
    let fragment = doc.create_document_fragment();
    fill_nodes(&mut doc, MAX_NODES);
    runtime.allocated = MAX_HEAP;
    let units = Value::String(vec![0xd800, 0xdc00, 0xdfff].into());
    let before = format!("{doc:?}");
    for (method, node) in [
        ("setNodeValue", Value::Node(element)),
        ("setNodeValue", Value::Node(fragment)),
        ("setNodeValue", Value::Document),
        ("setTextContent", Value::Document),
    ] {
        runtime
            .node_data_native(method, node, std::slice::from_ref(&units), &mut doc)
            .unwrap();
        assert_eq!(runtime.allocated, MAX_HEAP);
    }
    assert_eq!(format!("{doc:?}"), before);
    clean(&runtime);
}

fn setup(kind: &str, full: bool) -> (Runtime, Document, Value) {
    let (runtime, mut doc) = fresh();
    let id = match kind {
        "Text" => doc.create_text_node("old"),
        "Comment" => doc.create_comment("old"),
        "PI" => doc.create_processing_instruction("probe", "old"),
        "Element" => doc.query_selector("p").unwrap(),
        "Fragment" => {
            let id = doc.create_document_fragment();
            let child = doc.create_text_node("old");
            doc.append_child(id, child);
            id
        }
        _ => unreachable!(),
    };
    if full {
        doc.nodes.shrink_to_fit();
    } else {
        doc.nodes.try_reserve_exact(1).unwrap();
    }
    (runtime, doc, Value::Node(id))
}

#[test]
fn all_setter_kinds_measured_exact_and_one_short_preserve_unpublished_state() {
    let input = Value::String(vec![65, 0xd800, 66, 0xdc00].into());
    for kind in ["Text", "Comment", "PI", "Element", "Fragment"] {
        for full in [false, true] {
            for method in ["setNodeValue", "setTextContent"] {
                if method == "setNodeValue" && matches!(kind, "Element" | "Fragment") {
                    continue;
                }
                let (mut runtime, mut doc, node) = setup(kind, full);
                let before = (runtime.steps, runtime.allocated);
                runtime
                    .node_data_native(method, node, std::slice::from_ref(&input), &mut doc)
                    .unwrap();
                let work = before.0 - runtime.steps;
                let heap = runtime.allocated - before.1;
                let expected = format!("{doc:?}");
                for (steps, bytes, success) in [
                    (work, heap, true),
                    (work - 1, heap, false),
                    (work, heap - 1, false),
                ] {
                    let (mut runtime, mut doc, node) = setup(kind, full);
                    let original = format!("{doc:?}");
                    let capacity = doc.nodes.capacity();
                    runtime.steps = steps;
                    runtime.allocated = MAX_HEAP - bytes;
                    let result = runtime.node_data_native(
                        method,
                        node,
                        std::slice::from_ref(&input),
                        &mut doc,
                    );
                    if success {
                        assert_eq!(
                            result.unwrap(),
                            Value::Undefined,
                            "{kind} {method} full={full}"
                        );
                        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
                        assert_eq!(format!("{doc:?}"), expected);
                    } else {
                        assert!(
                            result.unwrap_err().is_resource_limit(),
                            "{kind} {method} full={full}"
                        );
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
    }
}

#[test]
fn exact_getter_refusal_keeps_document_and_has_no_partial_string() {
    for kind in ["Text", "Comment", "PI", "Element", "Fragment"] {
        let (mut runtime, mut doc, node) = setup(kind, false);
        let before = (runtime.steps, runtime.allocated);
        let expected = runtime
            .node_data_native("getTextContent", node, &[], &mut doc)
            .unwrap();
        let work = before.0 - runtime.steps;
        let heap = runtime.allocated - before.1;
        for (steps, bytes, success) in [
            (work, heap, true),
            (work - 1, heap, false),
            (work, heap - 1, false),
        ] {
            let (mut runtime, mut doc, node) = setup(kind, false);
            let original = format!("{doc:?}");
            runtime.steps = steps;
            runtime.allocated = MAX_HEAP - bytes;
            let result = runtime.node_data_native("getTextContent", node, &[], &mut doc);
            if success {
                assert_eq!(result.unwrap(), expected);
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            } else {
                assert!(result.unwrap_err().is_resource_limit());
            }
            assert_eq!(format!("{doc:?}"), original);
            clean(&runtime);
        }
    }
    let (mut runtime, mut doc) = fresh();
    let id = doc.create_text_node(&"x".repeat(MAX_STRING + 1));
    let before = doc.nodes.len();
    assert!(
        runtime
            .node_data_native("getTextContent", Value::Node(id), &[], &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(doc.nodes.len(), before);
    clean(&runtime);
}

#[test]
fn empty_container_replacement_does_not_require_a_new_node_slot() {
    let (mut runtime, mut doc) = fresh();
    let id = doc.query_selector("p").unwrap();
    let child = doc.nodes[id].children[0];
    fill_nodes(&mut doc, MAX_NODES);
    let count = doc.nodes.len();
    let bytes = doc.retained_bytes();
    runtime
        .node_data_native("setTextContent", Value::Node(id), &[], &mut doc)
        .unwrap();
    assert!(doc.nodes[id].children.is_empty());
    assert_eq!(doc.nodes[child].parent, None);
    assert_eq!((doc.nodes.len(), doc.retained_bytes()), (count, bytes));
    clean(&runtime);
}

#[test]
fn container_admission_uses_callback_current_node_capacity() {
    let (mut runtime, mut doc) = fresh();
    runtime.execute("var box=document.querySelector('p'),effect=0;var value={toString(){effect++;document.createTextNode('callback');return 'outer';}};",&mut doc).unwrap();
    fill_nodes(&mut doc, MAX_NODES - 1);
    doc.nodes.try_reserve_exact(1).unwrap();
    let node = runtime.environments[0].bindings["box"].value.clone();
    let value = runtime.environments[0].bindings["value"].value.clone();
    let Value::Node(id) = node else { panic!() };
    let children = doc.nodes[id].children.clone();
    assert!(
        runtime
            .node_data_native("setTextContent", Value::Node(id), &[value], &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(
        runtime.environments[0].bindings["effect"].value,
        Value::Number(1.0)
    );
    assert_eq!(doc.nodes.len(), MAX_NODES);
    assert_eq!(doc.nodes[id].children, children);
    assert!(matches!(&doc.nodes.last().unwrap().kind,NodeKind::Text(data) if data=="callback"));
    clean(&runtime);
}

#[test]
fn accessor_callback_terminal_failure_retains_prefix_and_cleans_execution() {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = "var box=document.querySelector('p'),caught=false,finalized=false,prefix=false;try{box.textContent={toString(){prefix=true;box.appendChild(new Text('callback'));while(true){}}};}catch(e){caught=true;}finally{finalized=true;}";
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        assert!(result.unwrap_err().is_resource_limit());
        assert_eq!(
            runtime.environments[0].bindings["prefix"].value,
            Value::Bool(true)
        );
        assert_eq!(
            runtime.environments[0].bindings["caught"].value,
            Value::Bool(false)
        );
        assert_eq!(
            runtime.environments[0].bindings["finalized"].value,
            Value::Bool(false)
        );
        let Value::Node(id) = runtime.environments[0].bindings["box"].value else {
            panic!()
        };
        assert_eq!(doc.text_content(id), "keptcallback");
        clean(&runtime);
    }
}

#[test]
fn native_getter_call_failure_unwinds_temporary_invocation() {
    fn ready() -> (Runtime, Document, Value, Value) {
        let (mut runtime, mut doc) = fresh();
        let get = runtime
            .execute(
                "Object.getOwnPropertyDescriptor(Node.prototype,'textContent').get",
                &mut doc,
            )
            .unwrap();
        let node = Value::Node(doc.create_text_node("exact"));
        (runtime, doc, get, node)
    }
    let (mut runtime, mut doc, get, node) = ready();
    let before = (runtime.steps, runtime.allocated);
    let expected = runtime.call(get, Vec::new(), node, &mut doc).unwrap();
    let work = before.0 - runtime.steps;
    let heap = runtime.allocated - before.1;
    for (steps, bytes, success) in [
        (work, heap, true),
        (work - 1, heap, false),
        (work, heap - 1, false),
        (8, heap, false),
    ] {
        let (mut runtime, mut doc, get, node) = ready();
        runtime.steps = steps;
        runtime.allocated = MAX_HEAP - bytes;
        let result = runtime.call(get, Vec::new(), node, &mut doc);
        if success {
            assert_eq!(result.unwrap(), expected);
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        clean(&runtime);
    }
}

#[test]
fn container_admission_rechecks_bytes_without_credit_for_detached_children() {
    let (mut runtime, mut doc) = fresh();
    runtime.execute("var box=document.querySelector('p'),effect=0;var value={toString(){effect++;box.appendChild(document.createTextNode('!'));return 'outside';}};", &mut doc).unwrap();
    while doc.retained_bytes() < crate::dom::MAX_DOM_BYTES - 1 {
        let remaining = crate::dom::MAX_DOM_BYTES - 1 - doc.retained_bytes();
        let id = doc.create_text_node(&"x".repeat(remaining.min(8 * 1024 * 1024)));
        assert_ne!(id, doc.root);
    }
    doc.nodes.try_reserve_exact(1).unwrap();
    let Value::Node(id) = runtime.environments[0].bindings["box"].value else {
        panic!()
    };
    let old = doc.nodes[id].children[0];
    let count = doc.nodes.len();
    let input = runtime.environments[0].bindings["value"].value.clone();
    let error = runtime
        .node_data_native("setTextContent", Value::Node(id), &[input], &mut doc)
        .unwrap_err();
    assert!(error.is_resource_limit());
    assert_eq!(
        runtime.environments[0].bindings["effect"].value,
        Value::Number(1.0)
    );
    assert_eq!(
        (doc.nodes.len(), doc.retained_bytes()),
        (count + 1, crate::dom::MAX_DOM_BYTES)
    );
    assert_eq!(doc.nodes[id].children, vec![old, count]);
    assert_eq!(doc.nodes[old].parent, Some(id));
    assert_eq!(doc.nodes[count].parent, Some(id));
    assert_eq!(doc.text_content(id), "kept!");
    clean(&runtime);
}
