use super::*;

const CASES: &str = include_str!("../../../tests/fixtures/node-connected.js");

fn fresh() -> (Runtime, Document) {
    (
        Runtime::try_new().unwrap(),
        Document::parse("<!doctype html><html><head></head><body></body></html>"),
    )
}
fn clean(runtime: &Runtime) {
    assert!(runtime.frames.is_empty());
    assert_eq!(
        (runtime.calls, runtime.eval_depth, runtime.stack_units),
        (0, 0, 0)
    );
}
fn state(doc: &Document) -> (String, usize, Vec<usize>) {
    (
        format!("{doc:?}"),
        doc.nodes.capacity(),
        doc.nodes.iter().map(|n| n.children.capacity()).collect(),
    )
}
fn evaluate(runtime: &mut Runtime, doc: &mut Document, source: &str, strict: bool) -> Value {
    let value = if strict {
        runtime.execute_strict(source, doc)
    } else {
        runtime.execute(source, doc)
    }
    .unwrap();
    clean(runtime);
    value
}
fn independent(name: &str) {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        // The frozen fixture requires the literal language-mode boolean; it
        // must not infer the current mode from observed assignment behavior.
        let source = format!("{CASES}\nnodeConnectedCases.{name}({strict});");
        assert_eq!(
            evaluate(&mut runtime, &mut doc, &source, strict),
            Value::Bool(true),
            "{name}, strict={strict}"
        );
    }
}
macro_rules! cases {($($name:ident),* $(,)?)=>{$(#[test]fn $name(){independent(stringify!($name));})*};}
cases!(
    metadata_and_cached_getter,
    authentic_receiver_brand_without_properties,
    document_alias_and_connected_character_kinds,
    detached_leaves_branches_and_fragment_splicing,
    current_connection_after_repeated_subtree_moves,
    template_content_is_separate_from_ordinary_children,
    namespace_visibility_and_empty_data_do_not_decide,
    receiver_and_extra_argument_effects_precede_read,
    throwing_argument_preserves_completed_prefix,
    ignored_extra_values_and_internal_field_shadows,
    readonly_assignment_follows_current_language_mode,
    strict_readonly_assignment_keeps_rhs_effects,
    own_data_and_accessor_shadows_delete_normally,
    prototype_replacement_deletion_and_saved_getter,
    authentic_alternate_prototype_retains_connection_brand,
);

#[test]
fn prior_connection_inventory_after_removing_later_configurable_operation() {
    // Descriptor restoration changes creation order; each realm is disposable.
    // Keep the original fixture exact and let the new fixture check pristine order.
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = format!(
            r#"{CASES}
        (function(){{
          const saved=Object.getOwnPropertyDescriptor(Node.prototype,'compareDocumentPosition');
          if(!saved||typeof saved.value!=='function'||!saved.writable||!saved.enumerable||!saved.configurable)
            throw new Error('position descriptor');
          try{{
            if(!delete Node.prototype.compareDocumentPosition)throw new Error('position delete');
            if(nodeConnectedCases.represented_complete_key_order({strict})!==true)throw new Error('prior connection inventory');
          }}finally{{Object.defineProperty(Node.prototype,'compareDocumentPosition',saved);}}
          const d=Object.getOwnPropertyDescriptor(Node.prototype,'compareDocumentPosition');
          if(d.value!==saved.value||d.writable!==saved.writable||d.enumerable!==saved.enumerable||
             d.configurable!==saved.configurable)throw new Error('position restoration');
          return true;
        }})()"#
        );
        assert_eq!(
            evaluate(&mut runtime, &mut doc, &source, strict),
            Value::Bool(true)
        );
    }
}

#[test]
fn connected_bootstrap_reports_actual_admission() {
    let runtime = Runtime::uninitialized().finish_bootstrap().unwrap();
    println!(
        "NODE_CONNECTED_BOOTSTRAP steps={} allocated={} objects={} capacity={} native={} prototypes={}",
        runtime.steps,
        runtime.allocated,
        runtime.objects.len(),
        runtime.objects.capacity(),
        runtime.native_properties.len(),
        runtime.prototypes.len()
    );
    assert_eq!(runtime.objects.len(), dom_prototypes::BOOTSTRAP_OBJECTS);
    assert!(runtime.steps > 0 && runtime.allocated < MAX_HEAP);
    clean(&runtime);
}

fn branch(connected: bool) -> (Runtime, Document, NodeId, NodeId, NodeId) {
    let (runtime, mut doc) = fresh();
    let parent = doc.create_element("div");
    let middle = doc.create_element("span");
    let leaf = doc.create_text_node("kept");
    doc.append_child(parent, middle);
    doc.append_child(middle, leaf);
    if connected {
        doc.append_child(doc.root, parent);
    }
    (runtime, doc, parent, middle, leaf)
}

#[test]
fn authentic_brand_debit_precedes_tree_work_without_body_heap() {
    let (mut runtime, doc) = fresh();
    let original = state(&doc);
    for receiver in [
        Value::Undefined,
        Value::Null,
        Value::Bool(false),
        Value::Number(0.0),
        Value::String("node".into()),
        Value::Node(usize::MAX),
        Value::Node(doc.nodes.len()),
    ] {
        for available in [15, 16] {
            runtime.steps = available;
            runtime.allocated = MAX_HEAP;
            let error = runtime
                .node_is_connected(receiver.clone(), &doc)
                .unwrap_err();
            if available == 15 {
                assert!(error.is_resource_limit());
            } else {
                assert_eq!(error.name(), "TypeError");
            }
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            assert_eq!(state(&doc), original);
            clean(&runtime);
        }
    }
}

#[test]
fn all_small_body_work_cuts_leave_links_capacities_and_heap_unchanged() {
    for connected in [false, true] {
        let (mut runtime, doc, _, _, leaf) = branch(connected);
        let original = state(&doc);
        runtime.steps = MAX_STEPS;
        runtime.allocated = MAX_HEAP;
        assert_eq!(
            runtime.node_is_connected(Value::Node(leaf), &doc).unwrap(),
            Value::Bool(connected)
        );
        let work = MAX_STEPS - runtime.steps;
        for available in 0..=work {
            runtime.steps = available;
            let result = runtime.node_is_connected(Value::Node(leaf), &doc);
            if available == work {
                assert_eq!(result.unwrap(), Value::Bool(connected));
            } else {
                assert!(result.unwrap_err().is_resource_limit(), "cut={available}");
            }
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            assert_eq!(state(&doc), original);
            clean(&runtime);
        }
    }
}

#[test]
fn shared_walk_matches_default_root_and_final_boolean_has_its_own_debit() {
    for connected in [false, true] {
        let (mut runtime, mut doc, parent, _, leaf) = branch(connected);
        for id in [doc.root, parent, leaf] {
            runtime.allocated = MAX_HEAP;
            runtime.steps = MAX_STEPS;
            let root = runtime.node_ordinary_root_from_id(id, &doc).unwrap();
            let walk = MAX_STEPS - runtime.steps;
            runtime.steps = MAX_STEPS;
            assert_eq!(
                runtime
                    .node_get_root_node(Value::Node(id), &[], &mut doc)
                    .unwrap(),
                root
            );
            assert_eq!(MAX_STEPS - runtime.steps, walk + 26);
            runtime.steps = MAX_STEPS;
            let expected = Value::Bool(matches!(root, Value::Document));
            assert_eq!(
                runtime.node_is_connected(Value::Node(id), &doc).unwrap(),
                expected
            );
            assert_eq!(MAX_STEPS - runtime.steps, walk + 18);
            for left in [0, 1] {
                runtime.steps = walk + 16 + left;
                assert!(
                    runtime
                        .node_is_connected(Value::Node(id), &doc)
                        .unwrap_err()
                        .is_resource_limit()
                );
                assert_eq!(runtime.steps, 0);
            }
            assert_eq!(runtime.allocated, MAX_HEAP);
        }
        clean(&runtime);
    }
}

#[test]
fn document_alias_empty_document_and_host_only_leaf_kinds() {
    let (mut runtime, mut doc) = fresh();
    let text = doc.create_text_node("raw Document text");
    let comment = doc.create_comment("comment");
    let pi = doc.create_processing_instruction("kept", "data");
    let doctype = doc
        .nodes
        .iter()
        .position(|n| matches!(n.kind, NodeKind::Doctype(_)))
        .unwrap();
    let fragment = doc.create_document_fragment();
    for id in [text, comment, pi] {
        doc.append_child(doc.root, id);
    }
    let original = state(&doc);
    runtime.allocated = MAX_HEAP;
    for receiver in [
        Value::Document,
        Value::Node(doc.root),
        Value::Node(text),
        Value::Node(comment),
        Value::Node(pi),
        Value::Node(doctype),
    ] {
        assert_eq!(
            runtime.node_is_connected(receiver, &doc).unwrap(),
            Value::Bool(true)
        );
    }
    assert_eq!(
        runtime
            .node_is_connected(Value::Node(fragment), &doc)
            .unwrap(),
        Value::Bool(false)
    );
    assert_eq!(state(&doc), original);
    for id in doc.nodes[doc.root].children.clone() {
        doc.nodes[id].parent = None;
    }
    doc.nodes[doc.root].children.clear();
    for receiver in [Value::Document, Value::Node(doc.root)] {
        runtime.steps = 34;
        assert_eq!(
            runtime.node_is_connected(receiver, &doc).unwrap(),
            Value::Bool(true)
        );
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    }
    clean(&runtime);
}

#[test]
fn malformed_reached_links_preserve_shared_root_errors_and_work() {
    for case in 0..11 {
        let (mut runtime, mut doc, parent, middle, leaf) = branch(false);
        match case {
            0 => doc.nodes[leaf].parent = Some(usize::MAX),
            1 => doc.nodes[middle].children.clear(),
            2 => doc.nodes[middle].children.push(leaf),
            3 => doc.nodes[leaf].parent = Some(leaf),
            4 => doc.nodes[middle].kind = NodeKind::Comment("wrong".into()),
            5 => doc.nodes[parent].kind = NodeKind::DocumentFragment { host: None },
            6 => doc.nodes[doc.root].parent = Some(middle),
            7 => doc.nodes[leaf].children.push(middle),
            8 => {
                doc.nodes[doc.root].children.clear();
                doc.nodes[doc.root].kind = NodeKind::Comment("false root".into());
            }
            9 => {
                doc.nodes[parent].parent = Some(middle);
                doc.nodes[middle].children.push(parent);
            }
            10 => doc.nodes[middle].kind = NodeKind::Document,
            _ => unreachable!(),
        }
        if case == 5 {
            doc.nodes[parent].parent = Some(middle);
        }
        let target = if case == 6 || case == 8 {
            doc.root
        } else {
            leaf
        };
        let original = state(&doc);
        runtime.allocated = MAX_HEAP;
        runtime.steps = MAX_STEPS;
        let error = runtime
            .node_ordinary_root_from_id(target, &doc)
            .unwrap_err();
        assert_eq!(error.name(), "TypeError", "case={case}");
        let walk = MAX_STEPS - runtime.steps;
        runtime.steps = MAX_STEPS;
        assert_eq!(
            runtime
                .node_is_connected(Value::Node(target), &doc)
                .unwrap_err()
                .name(),
            error.name()
        );
        assert_eq!(MAX_STEPS - runtime.steps, walk + 16);
        runtime.steps = MAX_STEPS;
        assert_eq!(
            runtime
                .node_get_root_node(Value::Node(target), &[], &mut doc)
                .unwrap_err()
                .name(),
            error.name()
        );
        assert_eq!(MAX_STEPS - runtime.steps, walk + 26);
        assert_eq!(runtime.allocated, MAX_HEAP);
        assert_eq!(state(&doc), original);
        clean(&runtime);
    }
}

#[test]
fn accepted_256_edge_snapshots_fit_and_the_257th_host_edge_refuses() {
    for connected in [false, true] {
        let (mut runtime, mut doc) = fresh();
        // Isolate a unary root so the literal fee includes one child per edge.
        for id in doc.nodes[doc.root].children.clone() {
            doc.nodes[id].parent = None;
        }
        doc.nodes[doc.root].children.clear();
        let ancestor = if connected {
            doc.root
        } else {
            doc.create_document_fragment()
        };
        let mut parent = ancestor;
        for _ in 1..crate::dom::MAX_DEPTH {
            let child = doc.create_element("div");
            doc.nodes[parent].children.push(child);
            doc.nodes[child].parent = Some(parent);
            parent = child;
        }
        let leaf = doc.create_text_node("edge");
        doc.nodes[parent].children.push(leaf);
        doc.nodes[leaf].parent = Some(parent);
        let (mut cursor, mut depth) = (leaf, 0);
        while let Some(next) = doc.nodes[cursor].parent {
            depth += 1;
            cursor = next;
        }
        assert_eq!((depth, cursor), (crate::dom::MAX_DEPTH, ancestor));
        let mut doc = Document::from_snapshot(
            doc.nodes,
            doc.root,
            false,
            crate::dom::DocumentMode::NoQuirks,
        )
        .unwrap();
        let original = state(&doc);
        runtime.allocated = MAX_HEAP;
        runtime.steps = MAX_STEPS;
        assert_eq!(
            runtime.node_is_connected(Value::Node(leaf), &doc).unwrap(),
            Value::Bool(connected)
        );
        let work = MAX_STEPS - runtime.steps;
        assert_eq!(work, 7458);
        for available in [work - 1, work] {
            runtime.steps = available;
            let result = runtime.node_is_connected(Value::Node(leaf), &doc);
            if available == work {
                assert_eq!(result.unwrap(), Value::Bool(connected));
            } else {
                assert!(result.unwrap_err().is_resource_limit());
            }
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            assert_eq!(state(&doc), original);
        }
        let extra = doc.create_text_node("overdeep");
        let element = doc.create_element("span");
        doc.nodes[parent].children = vec![element];
        doc.nodes[element].parent = Some(parent);
        doc.nodes[element].children.push(extra);
        doc.nodes[extra].parent = Some(element);
        doc.nodes[leaf].parent = None;
        let original = state(&doc);
        runtime.steps = MAX_STEPS;
        assert_eq!(
            runtime
                .node_is_connected(Value::Node(extra), &doc)
                .unwrap_err()
                .name(),
            "TypeError"
        );
        assert_eq!(state(&doc), original);
        assert_eq!(runtime.allocated, MAX_HEAP);
        clean(&runtime);
    }
}

#[test]
fn unrelated_arena_and_large_payload_do_not_change_body_cost() {
    let mut costs = Vec::new();
    for extra in [false, true] {
        let (mut runtime, mut doc, _, _, leaf) = branch(false);
        if extra {
            for _ in 0..1000 {
                doc.create_comment("");
            }
            doc.create_comment(&"x".repeat(MAX_STRING + 1));
        }
        runtime.steps = MAX_STEPS;
        runtime.allocated = MAX_HEAP;
        assert_eq!(
            runtime.node_is_connected(Value::Node(leaf), &doc).unwrap(),
            Value::Bool(false)
        );
        costs.push((MAX_STEPS - runtime.steps, runtime.allocated));
    }
    assert_eq!(costs[0], costs[1]);
}

#[test]
fn new_dispatch_is_paid_and_existing_accessor_routes_keep_their_fees() {
    let (mut runtime, mut doc) = fresh();
    let text = doc.create_text_node("");
    for method in [
        "getNodeValue",
        "setNodeValue",
        "getTextContent",
        "setTextContent",
    ] {
        // Invalid receivers make the reached old route end before payload or
        // setter conversion, isolating its unchanged discriminator and match.
        runtime.steps = MAX_STEPS;
        assert_eq!(
            runtime
                .node_data_native(method, Value::Null, &[], &mut doc)
                .unwrap_err()
                .name(),
            "TypeError"
        );
        assert_eq!(MAX_STEPS - runtime.steps, 9);
    }
    runtime.steps = MAX_STEPS;
    runtime.allocated = 0;
    runtime
        .node_data_call_preflight("DOM.Node.getIsConnected")
        .unwrap();
    assert_eq!((MAX_STEPS - runtime.steps, runtime.allocated), (27, 55));
    runtime.steps = MAX_STEPS;
    runtime.allocated = MAX_HEAP;
    assert_eq!(
        runtime
            .node_data_native("getIsConnected", Value::Node(text), &[], &mut doc)
            .unwrap(),
        Value::Bool(false)
    );
    assert_eq!(
        (MAX_STEPS - runtime.steps, runtime.allocated),
        (24 + 34, MAX_HEAP)
    );
    clean(&runtime);
}

#[test]
fn saved_native_invocation_boundaries_preserve_cleanup_and_metadata() {
    fn ready() -> (Runtime, Document, NodeId, Value) {
        let (mut runtime, mut doc, _, _, leaf) = branch(false);
        let getter = runtime
            .execute(
                "Object.getOwnPropertyDescriptor(Node.prototype,'isConnected').get",
                &mut doc,
            )
            .unwrap();
        (runtime, doc, leaf, getter)
    }
    let (mut measured, mut doc, leaf, getter) = ready();
    let before = (measured.steps, measured.allocated);
    assert_eq!(
        measured
            .call(getter, vec![], Value::Node(leaf), &mut doc)
            .unwrap(),
        Value::Bool(false)
    );
    let (work, heap) = (before.0 - measured.steps, measured.allocated - before.1);
    assert!(heap >= 55);
    for (steps, bytes, success) in [
        (work, heap, true),
        (work - 1, heap, false),
        (work, heap - 1, false),
        (8, heap, false),
    ] {
        let (mut runtime, mut doc, leaf, getter) = ready();
        let original = state(&doc);
        let bag = runtime.property_object(&getter).unwrap();
        let metadata = format!("{:?}", runtime.objects[bag].values);
        let objects = (runtime.objects.len(), runtime.objects.capacity());
        runtime.steps = steps;
        runtime.allocated = MAX_HEAP - bytes;
        let result = runtime.call(getter, vec![], Value::Node(leaf), &mut doc);
        if success {
            assert_eq!(result.unwrap(), Value::Bool(false));
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        assert_eq!(state(&doc), original);
        assert_eq!(format!("{:?}", runtime.objects[bag].values), metadata);
        assert_eq!((runtime.objects.len(), runtime.objects.capacity()), objects);
        clean(&runtime);
    }
}

#[test]
fn wide_scan_is_terminal_and_preserves_completed_argument_effects() {
    for strict in [false, true] {
        let (mut runtime, mut doc, parent, _, leaf) = branch(false);
        for _ in 0..26000 {
            let child = doc.create_comment("");
            doc.nodes[parent].children.push(child);
            doc.nodes[child].parent = Some(parent);
        }
        runtime.execute("var target,effect=0,caught=false,finalized=false;var getter=Object.getOwnPropertyDescriptor(Node.prototype,'isConnected').get;", &mut doc).unwrap();
        runtime.environments[0]
            .bindings
            .get_mut("target")
            .unwrap()
            .value = Value::Node(leaf);
        let original = state(&doc);
        let source =
            "try{getter.call(target,effect=1);}catch(e){caught=true;}finally{finalized=true;}";
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        assert!(result.unwrap_err().is_resource_limit());
        assert_eq!(
            runtime.environments[0].bindings["effect"].value,
            Value::Number(1.0)
        );
        for name in ["caught", "finalized"] {
            assert_eq!(
                runtime.environments[0].bindings[name].value,
                Value::Bool(false)
            );
        }
        assert_eq!(state(&doc), original);
        clean(&runtime);
    }
}

#[test]
fn two_live_realms_retain_separate_getter_metadata_bags() {
    let (mut a, mut da) = fresh();
    let (mut b, mut db) = fresh();
    let getter = "Object.getOwnPropertyDescriptor(Node.prototype,'isConnected').get";
    a.execute(
        &format!("Object.defineProperty({getter},'name',{{value:'A'}})"),
        &mut da,
    )
    .unwrap();
    assert_eq!(
        b.execute(&format!("{getter}.name"), &mut db).unwrap(),
        Value::String("get isConnected".into())
    );
    b.execute(
        &format!("Object.defineProperty({getter},'name',{{value:'B'}})"),
        &mut db,
    )
    .unwrap();
    assert_eq!(
        a.execute(&format!("{getter}.name"), &mut da).unwrap(),
        Value::String("A".into())
    );
    assert_eq!(
        b.execute(&format!("{getter}.name"), &mut db).unwrap(),
        Value::String("B".into())
    );
    clean(&a);
    clean(&b);
}
