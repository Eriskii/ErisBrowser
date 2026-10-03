use super::*;

const CASES: &str = include_str!("../../../tests/fixtures/node-predicates.js");

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
fn independent(name: &str) {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = format!("{CASES}\nnodePredicateCases.{name}();");
        let result = if strict {
            runtime.execute_strict(&source, &mut doc)
        } else {
            runtime.execute(&source, &mut doc)
        };
        assert_eq!(
            result.unwrap(),
            Value::Bool(true),
            "{name}, strict={strict}"
        );
        clean(&runtime);
    }
}
macro_rules! cases {($($name:ident),* $(,)?)=>{$(#[test]fn $name(){independent(stringify!($name));})*};}
cases!(
    metadata,
    direct_children_include_empty_text_and_comments,
    contains_inclusive_direction_and_disjoint_trees,
    same_node_uses_identity_not_data_or_position,
    required_arguments_differ_from_nullable_values,
    interface_arguments_do_not_coerce,
    receiver_brand_cannot_be_forged,
    ignored_extra_arguments_are_evaluated_but_not_converted,
    argument_expressions_finish_before_tree_observation,
    template_content_is_a_separate_tree,
    document_and_public_parent_aliases,
    saved_calls_survive_shadow_replacement_and_deletion,
    native_relations_ignore_authored_getters,
    authentic_alternate_prototype_still_has_node_brand,
    represented_leaf_kinds_and_detached_payloads,
);

#[test]
fn prior_predicate_inventory_after_removing_later_configurable_operations() {
    // Restoring creation order is impossible; each realm is disposable. The
    // independent equality fixture checks the current pristine inventory.
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = format!(
            r#"{CASES}
        (function(){{
          const names=['getRootNode','isEqualNode'],saved=[];
          for(const name of names){{
            const d=Object.getOwnPropertyDescriptor(Node.prototype,name);
            if(!d||!d.configurable||typeof d.value!=='function')throw new Error('later descriptor');
            saved.push(d);
          }}
          try{{
            for(const name of names)if(!delete Node.prototype[name])throw new Error('later delete');
            if(nodePredicateCases.represented_complete_key_order()!==true)throw new Error('prior predicate inventory');
          }}finally{{
            for(let i=0;i<names.length;i++)Object.defineProperty(Node.prototype,names[i],saved[i]);
          }}
          for(let i=0;i<names.length;i++){{
            const d=Object.getOwnPropertyDescriptor(Node.prototype,names[i]),old=saved[i];
            if(d.value!==old.value||d.writable!==old.writable||d.enumerable!==old.enumerable||
               d.configurable!==old.configurable)throw new Error('later restoration');
          }}
          return true;
        }})()"#
        );
        let result = if strict {
            runtime.execute_strict(&source, &mut doc)
        } else {
            runtime.execute(&source, &mut doc)
        };
        assert_eq!(result.unwrap(), Value::Bool(true), "strict={strict}");
        clean(&runtime);
    }
}

#[test]
fn predicates_bootstrap_reports_actual_admission() {
    let runtime = Runtime::uninitialized().finish_bootstrap().unwrap();
    println!(
        "NODE_PREDICATES_BOOTSTRAP steps={} allocated={} objects={} capacity={} native={} prototypes={}",
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

fn branch() -> (Runtime, Document, NodeId, NodeId, NodeId) {
    let (runtime, mut doc) = fresh();
    let parent = doc.create_document_fragment();
    let element = doc.create_element("div");
    let leaf = doc.create_text_node("kept");
    doc.append_child(parent, element);
    doc.append_child(element, leaf);
    (runtime, doc, parent, element, leaf)
}
fn state(doc: &Document) -> (String, usize, Vec<usize>) {
    (
        format!("{doc:?}"),
        doc.nodes.capacity(),
        doc.nodes.iter().map(|n| n.children.capacity()).collect(),
    )
}
fn body(
    runtime: &mut Runtime,
    doc: &Document,
    method: usize,
    parent: NodeId,
    leaf: NodeId,
) -> Result<Value> {
    match method {
        0 => runtime.node_has_child_nodes(Value::Node(parent), doc),
        1 => runtime.node_is_same_node(Value::Node(leaf), &[Value::Node(leaf)], doc),
        2 => runtime.node_contains(Value::Node(parent), &[Value::Node(leaf)], doc),
        3 => runtime.node_contains(Value::Node(leaf), &[Value::Node(parent)], doc),
        4 => runtime.node_contains(Value::Node(leaf), &[Value::Null], doc),
        5 => runtime.node_is_same_node(Value::Node(leaf), &[Value::Undefined], doc),
        _ => unreachable!(),
    }
}

#[test]
fn measured_body_cutpoints_use_no_owned_heap_and_never_change_document() {
    for method in 0..6 {
        let (mut measured, doc, parent, _, leaf) = branch();
        let before = (measured.steps, measured.allocated);
        let expected = body(&mut measured, &doc, method, parent, leaf).unwrap();
        let work = before.0 - measured.steps;
        assert!(work > 0);
        assert_eq!(measured.allocated, before.1);
        for short in [false, true] {
            let (mut runtime, doc, parent, _, leaf) = branch();
            let original = state(&doc);
            let objects = runtime.objects.len();
            runtime.steps = work - usize::from(short);
            runtime.allocated = MAX_HEAP;
            let result = body(&mut runtime, &doc, method, parent, leaf);
            if short {
                assert!(result.unwrap_err().is_resource_limit());
            } else {
                assert_eq!(result.unwrap(), expected);
            }
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            assert_eq!(state(&doc), original);
            assert_eq!(runtime.objects.len(), objects);
            clean(&runtime);
        }
    }
}

#[test]
fn every_ancestor_work_cut_is_read_only_including_scan_admission() {
    let (mut measured, doc, parent, middle, leaf) = branch();
    let before = measured.steps;
    assert_eq!(
        measured
            .node_contains(Value::Node(parent), &[Value::Node(leaf)], &doc)
            .unwrap(),
        Value::Bool(true)
    );
    let cost = before - measured.steps;
    let original = state(&doc);
    let (mut runtime, _) = fresh();
    for available in 0..=cost {
        runtime.steps = available;
        runtime.allocated = MAX_HEAP;
        let result = runtime.node_contains(Value::Node(parent), &[Value::Node(leaf)], &doc);
        if available == cost {
            assert_eq!(result.unwrap(), Value::Bool(true));
        } else {
            assert!(result.unwrap_err().is_resource_limit(), "cut={available}");
        }
        assert_eq!(state(&doc), original);
        assert_eq!(doc.nodes[middle].children, [leaf]);
        assert_eq!(runtime.allocated, MAX_HEAP);
        clean(&runtime);
    }
}

#[test]
fn receiver_brand_precedes_arity_and_explicit_nullish_never_coerces() {
    let (mut runtime, doc, parent, _, leaf) = branch();
    for receiver in [
        Value::Node(usize::MAX),
        Value::Node(doc.nodes.len()),
        Value::Null,
        Value::Object(0),
    ] {
        runtime.steps = 16;
        let error = runtime.node_contains(receiver, &[], &doc).unwrap_err();
        assert_eq!(error.name(), "TypeError");
        assert_eq!(runtime.steps, 0);
    }
    runtime.steps = 19;
    assert!(
        runtime
            .node_contains(Value::Node(parent), &[], &doc)
            .unwrap_err()
            .is_resource_limit()
    );
    runtime.steps = 20;
    assert_eq!(
        runtime
            .node_contains(Value::Node(parent), &[], &doc)
            .unwrap_err()
            .name(),
        "TypeError"
    );
    for arg in [Value::Null, Value::Undefined] {
        runtime.steps = MAX_STEPS;
        assert_eq!(
            runtime
                .node_contains(Value::Node(parent), std::slice::from_ref(&arg), &doc)
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            runtime
                .node_is_same_node(Value::Node(leaf), &[arg], &doc)
                .unwrap(),
            Value::Bool(false)
        );
    }
    for bad in [
        Value::Node(usize::MAX),
        Value::Node(doc.nodes.len()),
        Value::Object(0),
        Value::Number(1.0),
    ] {
        runtime.steps = MAX_STEPS;
        assert_eq!(
            runtime
                .node_contains(Value::Node(parent), std::slice::from_ref(&bad), &doc)
                .unwrap_err()
                .name(),
            "TypeError"
        );
        assert_eq!(
            runtime
                .node_is_same_node(Value::Node(leaf), &[bad], &doc)
                .unwrap_err()
                .name(),
            "TypeError"
        );
    }
    clean(&runtime);
}

#[test]
fn document_aliases_and_all_represented_leaf_kinds_use_internal_identity() {
    let (mut runtime, mut doc) = fresh();
    let text = doc.create_text_node("raw document child");
    doc.append_child(doc.root, text);
    let comment = doc.create_comment("kept");
    let pi = doc.create_processing_instruction("kept", "data");
    let doctype = doc
        .nodes
        .iter()
        .position(|n| matches!(n.kind, NodeKind::Doctype(_)))
        .unwrap();
    let original = state(&doc);
    for receiver in [Value::Document, Value::Node(doc.root)] {
        for other in [Value::Document, Value::Node(doc.root)] {
            assert_eq!(
                runtime
                    .node_is_same_node(receiver.clone(), std::slice::from_ref(&other), &doc)
                    .unwrap(),
                Value::Bool(true)
            );
            assert_eq!(
                runtime
                    .node_contains(receiver.clone(), &[other], &doc)
                    .unwrap(),
                Value::Bool(true)
            );
        }
        assert_eq!(
            runtime
                .node_has_child_nodes(receiver.clone(), &doc)
                .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            runtime
                .node_contains(receiver, &[Value::Node(text)], &doc)
                .unwrap(),
            Value::Bool(true)
        );
    }
    for id in [text, comment, pi, doctype] {
        assert_eq!(
            runtime.node_has_child_nodes(Value::Node(id), &doc).unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            runtime
                .node_is_same_node(Value::Node(id), &[Value::Node(id)], &doc)
                .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            runtime
                .node_contains(Value::Node(id), &[Value::Node(id)], &doc)
                .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            runtime
                .node_contains(Value::Node(id), &[Value::Document], &doc)
                .unwrap(),
            Value::Bool(false)
        );
    }
    assert_eq!(state(&doc), original);
    clean(&runtime);
}

#[test]
fn depth_limit_accepts_256_edges_and_rejects_an_additional_host_edge() {
    let (mut runtime, mut doc) = fresh();
    let ancestor = doc.create_document_fragment();
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
    let mut cursor = leaf;
    let mut depth = 0;
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
    runtime.allocated = MAX_HEAP;
    let before = runtime.steps;
    assert_eq!(
        runtime
            .node_contains(Value::Node(ancestor), &[Value::Node(leaf)], &doc)
            .unwrap(),
        Value::Bool(true)
    );
    let work = before - runtime.steps;
    assert!(work < 10000);
    assert_eq!(runtime.allocated, MAX_HEAP);
    let extra = doc.create_text_node("too deep");
    let parent = doc.nodes[leaf].parent.unwrap();
    let element = doc.create_element("span");
    doc.nodes[parent].children = vec![element];
    doc.nodes[element].parent = Some(parent);
    doc.nodes[element].children.push(extra);
    doc.nodes[extra].parent = Some(element);
    doc.nodes[leaf].parent = None;
    runtime.steps = MAX_STEPS;
    assert_eq!(
        runtime
            .node_contains(Value::Node(ancestor), &[Value::Node(extra)], &doc)
            .unwrap_err()
            .name(),
        "TypeError"
    );
    clean(&runtime);
}

#[test]
fn malformed_local_parent_membership_and_leaf_shapes_refuse_without_mutation() {
    for case in 0..8 {
        let (mut runtime, mut doc, parent, middle, leaf) = branch();
        match case {
            0 => doc.nodes[leaf].parent = Some(usize::MAX),
            1 => doc.nodes[middle].children.clear(),
            2 => doc.nodes[middle].children.push(leaf),
            3 => doc.nodes[leaf].parent = Some(leaf),
            4 => doc.nodes[middle].kind = NodeKind::Comment("wrong parent".into()),
            5 => doc.nodes[parent].parent = Some(middle),
            6 => doc.nodes[doc.root].parent = Some(middle),
            7 => doc.nodes[leaf].children.push(middle),
            _ => unreachable!(),
        }
        let target = if case == 6 { doc.root } else { leaf };
        let original = state(&doc);
        assert_eq!(
            runtime
                .node_contains(Value::Node(parent), &[Value::Node(target)], &doc)
                .unwrap_err()
                .name(),
            "TypeError",
            "case={case}"
        );
        assert_eq!(state(&doc), original);
        clean(&runtime);
    }
    let (mut runtime, mut doc, parent, _, leaf) = branch();
    doc.nodes[leaf].children.push(parent);
    assert_eq!(
        runtime
            .node_has_child_nodes(Value::Node(leaf), &doc)
            .unwrap_err()
            .name(),
        "TypeError"
    );
}

#[test]
fn reciprocal_cycles_are_bounded_but_self_and_null_do_not_audit_unreached_links() {
    let (mut runtime, mut doc) = fresh();
    let a = doc.create_element("div");
    let b = doc.create_element("span");
    let outside = doc.create_text_node("outside");
    doc.nodes[a].parent = Some(b);
    doc.nodes[b].children.push(a);
    doc.nodes[b].parent = Some(a);
    doc.nodes[a].children.push(b);
    let original = state(&doc);
    assert_eq!(
        runtime
            .node_contains(Value::Node(outside), &[Value::Node(a)], &doc)
            .unwrap_err()
            .name(),
        "TypeError"
    );
    assert!(runtime.steps > MAX_STEPS / 2);
    assert_eq!(state(&doc), original);
    assert_eq!(
        runtime
            .node_contains(Value::Node(a), &[Value::Node(a)], &doc)
            .unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        runtime
            .node_contains(Value::Node(a), &[Value::Null], &doc)
            .unwrap(),
        Value::Bool(false)
    );
    assert_eq!(
        runtime
            .node_is_same_node(Value::Node(a), &[Value::Node(a)], &doc)
            .unwrap(),
        Value::Bool(true)
    );
    clean(&runtime);
}

#[test]
fn unrelated_arena_and_payloads_do_not_change_small_predicate_costs() {
    let mut costs = Vec::new();
    for extra in [false, true] {
        let (mut runtime, mut doc, parent, _, leaf) = branch();
        if extra {
            for _ in 0..1000 {
                doc.create_comment("");
            }
            doc.create_comment(&"x".repeat(MAX_STRING + 1));
        }
        runtime.steps = MAX_STEPS;
        runtime.allocated = MAX_HEAP;
        let before = runtime.steps;
        assert_eq!(
            runtime
                .node_has_child_nodes(Value::Node(parent), &doc)
                .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            runtime
                .node_is_same_node(Value::Node(leaf), &[Value::Node(leaf)], &doc)
                .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            runtime
                .node_contains(Value::Node(parent), &[Value::Node(leaf)], &doc)
                .unwrap(),
            Value::Bool(true)
        );
        costs.push((before - runtime.steps, runtime.allocated));
    }
    assert_eq!(costs[0], costs[1]);
}

#[test]
fn wide_membership_scan_is_paid_and_terminal_without_catch_or_finally() {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let parent = doc.create_document_fragment();
        let leaf = doc.create_text_node("retained");
        doc.append_child(parent, leaf);
        for _ in 0..26000 {
            let child = doc.create_comment("");
            doc.nodes[parent].children.push(child);
            doc.nodes[child].parent = Some(parent);
        }
        runtime
            .execute(
                "var target,child,effect=0,caught=false,finalized=false",
                &mut doc,
            )
            .unwrap();
        runtime.environments[0]
            .bindings
            .get_mut("target")
            .unwrap()
            .value = Value::Node(parent);
        runtime.environments[0]
            .bindings
            .get_mut("child")
            .unwrap()
            .value = Value::Node(leaf);
        let original = state(&doc);
        let source =
            "try{target.contains(child,effect=1);}catch(e){caught=true;}finally{finalized=true;}";
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
        assert_eq!(
            runtime.environments[0].bindings["caught"].value,
            Value::Bool(false)
        );
        assert_eq!(
            runtime.environments[0].bindings["finalized"].value,
            Value::Bool(false)
        );
        assert_eq!(state(&doc), original);
        clean(&runtime);
    }
}

#[test]
fn saved_invocation_measured_work_heap_cuts_preserve_cleanup_and_metadata() {
    fn ready(name: &str) -> (Runtime, Document, Value, Value, Vec<Value>) {
        let (mut runtime, mut doc, parent, _, leaf) = branch();
        let method = runtime
            .execute(&format!("Node.prototype.{name}"), &mut doc)
            .unwrap();
        let args = if name == "hasChildNodes" {
            vec![]
        } else {
            vec![Value::Node(leaf)]
        };
        (runtime, doc, method, Value::Node(parent), args)
    }
    for name in ["hasChildNodes", "isSameNode", "contains"] {
        let (mut measured, mut doc, method, receiver, args) = ready(name);
        let before = (measured.steps, measured.allocated);
        let expected = measured.call(method, args, receiver, &mut doc).unwrap();
        let (work, heap) = (before.0 - measured.steps, measured.allocated - before.1);
        assert!(heap > 0);
        for (steps, bytes, success) in [
            (work, heap, true),
            (work - 1, heap, false),
            (work, heap - 1, false),
            (8, heap, false),
        ] {
            let (mut runtime, mut doc, method, receiver, args) = ready(name);
            let original = state(&doc);
            let bag = runtime.property_object(&method).unwrap();
            let metadata = format!("{:?}", runtime.objects[bag].values);
            runtime.steps = steps;
            runtime.allocated = MAX_HEAP - bytes;
            let result = runtime.call(method, args, receiver, &mut doc);
            if success {
                assert_eq!(result.unwrap(), expected);
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            } else {
                assert!(result.unwrap_err().is_resource_limit());
            }
            assert_eq!(state(&doc), original);
            assert_eq!(format!("{:?}", runtime.objects[bag].values), metadata);
            clean(&runtime);
        }
    }
}

#[test]
fn two_live_realms_keep_distinct_predicate_metadata_bags() {
    let (mut a, mut da) = fresh();
    let (mut b, mut db) = fresh();
    for name in ["hasChildNodes", "isSameNode", "contains"] {
        a.execute(
            &format!("Object.defineProperty(Node.prototype.{name},'name',{{value:'A'}})"),
            &mut da,
        )
        .unwrap();
        assert_eq!(
            b.execute(&format!("Node.prototype.{name}.name"), &mut db)
                .unwrap(),
            Value::String(name.into())
        );
        b.execute(
            &format!("Object.defineProperty(Node.prototype.{name},'name',{{value:'B'}})"),
            &mut db,
        )
        .unwrap();
        assert_eq!(
            a.execute(&format!("Node.prototype.{name}.name"), &mut da)
                .unwrap(),
            Value::String("A".into())
        );
        assert_eq!(
            b.execute(&format!("Node.prototype.{name}.name"), &mut db)
                .unwrap(),
            Value::String("B".into())
        );
    }
    clean(&a);
    clean(&b);
}
