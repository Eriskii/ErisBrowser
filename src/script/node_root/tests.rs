use super::*;

const CASES: &str = include_str!("../../../tests/fixtures/node-root.js");

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
        assert_eq!(
            evaluate(
                &mut runtime,
                &mut doc,
                &format!("{CASES}\nnodeRootCases.{name}();"),
                strict
            ),
            Value::Bool(true),
            "{name}, strict={strict}"
        );
    }
}
macro_rules! cases {($($name:ident),* $(,)?)=>{$(#[test]fn $name(){independent(stringify!($name));})*};}
cases!(
    metadata_and_inherited_identity,
    connected_detached_and_document_aliases,
    leaf_roots_preserve_exact_data_and_identity,
    template_content_never_crosses_host,
    default_options_do_not_read_object_prototype,
    inherited_non_enumerable_member_get_once,
    object_options_and_boolean_conversion_do_not_coerce,
    non_null_primitive_options_throw,
    receiver_brand_precedes_dictionary_get,
    callback_moves_receiver_and_ancestor_before_root_read,
    getter_abrupt_identity_retains_completed_move,
    argument_expressions_and_ignored_extra_conversion,
    saved_method_shadow_replacement_and_deletion,
    internal_tree_ignores_public_properties_and_nested_call,
    authentic_alternate_prototype_keeps_node_brand,
);

#[test]
fn prior_root_inventory_after_removing_configurable_equality() {
    // The old fixture remains byte-exact. Descriptor restoration changes key
    // order, so this wrapper always uses a disposable realm.
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = format!(
            r#"{CASES}
        (function(){{
          const saved=Object.getOwnPropertyDescriptor(Node.prototype,'isEqualNode');
          if(!saved||!saved.configurable||typeof saved.value!=='function')throw new Error('equality descriptor');
          const connection=Object.getOwnPropertyDescriptor(Node.prototype,'isConnected');
          if(!connection||typeof connection.get!=='function'||connection.set!==undefined||
             !connection.enumerable||!connection.configurable)throw new Error('connection descriptor');
          const position=Object.getOwnPropertyDescriptor(Node.prototype,'compareDocumentPosition');
          if(!position||!position.configurable||typeof position.value!=='function')throw new Error('position descriptor');
          const clone=Object.getOwnPropertyDescriptor(Node.prototype,'cloneNode');
          if(!clone||typeof clone.value!=='function'||!clone.writable||!clone.enumerable||!clone.configurable)throw new Error('clone descriptor');
          try{{
            if(!delete Node.prototype.cloneNode)throw new Error('clone delete');
            if(!delete Node.prototype.compareDocumentPosition)throw new Error('position delete');
            if(!delete Node.prototype.isConnected)throw new Error('connection delete');
            if(!delete Node.prototype.isEqualNode)throw new Error('equality delete');
            if(nodeRootCases.represented_complete_key_order()!==true)throw new Error('prior root inventory');
          }}finally{{
            Object.defineProperty(Node.prototype,'cloneNode',clone);
            Object.defineProperty(Node.prototype,'compareDocumentPosition',position);
            Object.defineProperty(Node.prototype,'isConnected',connection);
            Object.defineProperty(Node.prototype,'isEqualNode',saved);
          }}
          const d=Object.getOwnPropertyDescriptor(Node.prototype,'isEqualNode');
          if(d.value!==saved.value||d.writable!==saved.writable||d.enumerable!==saved.enumerable||
             d.configurable!==saved.configurable)throw new Error('equality restoration');
          const restoredConnection=Object.getOwnPropertyDescriptor(Node.prototype,'isConnected');
          if(restoredConnection.get!==connection.get||restoredConnection.set!==undefined||
             restoredConnection.enumerable!==connection.enumerable||restoredConnection.configurable!==connection.configurable)
            throw new Error('connection restoration');
          const restoredPosition=Object.getOwnPropertyDescriptor(Node.prototype,'compareDocumentPosition');
          if(restoredPosition.value!==position.value||restoredPosition.writable!==position.writable||
             restoredPosition.enumerable!==position.enumerable||restoredPosition.configurable!==position.configurable)
            throw new Error('position restoration');
          const restoredClone=Object.getOwnPropertyDescriptor(Node.prototype,'cloneNode');
          if(restoredClone.value!==clone.value||restoredClone.writable!==clone.writable||restoredClone.enumerable!==clone.enumerable||restoredClone.configurable!==clone.configurable)throw new Error('clone restoration');
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
fn root_bootstrap_reports_actual_admission() {
    let runtime = Runtime::uninitialized().finish_bootstrap().unwrap();
    println!(
        "NODE_ROOT_BOOTSTRAP steps={} allocated={} objects={} capacity={} native={} prototypes={}",
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
    let middle = doc.create_element("div");
    let leaf = doc.create_text_node("kept");
    doc.append_child(parent, middle);
    doc.append_child(middle, leaf);
    (runtime, doc, parent, middle, leaf)
}
fn state(doc: &Document) -> (String, usize, Vec<usize>) {
    (
        format!("{doc:?}"),
        doc.nodes.capacity(),
        doc.nodes
            .iter()
            .map(|node| node.children.capacity())
            .collect(),
    )
}

#[test]
fn all_default_body_work_cuts_are_read_only_and_need_no_owned_heap() {
    let (mut runtime, mut doc, parent, _, leaf) = branch();
    let before = runtime.steps;
    assert_eq!(
        runtime
            .node_get_root_node(Value::Node(leaf), &[], &mut doc)
            .unwrap(),
        Value::Node(parent)
    );
    let work = before - runtime.steps;
    let original = state(&doc);
    for available in 0..=work {
        runtime.steps = available;
        runtime.allocated = MAX_HEAP;
        let result = runtime.node_get_root_node(Value::Node(leaf), &[], &mut doc);
        if available == work {
            assert_eq!(result.unwrap(), Value::Node(parent));
        } else {
            assert!(result.unwrap_err().is_resource_limit(), "cut={available}");
        }
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        assert_eq!(state(&doc), original);
        clean(&runtime);
    }
    for args in [vec![], vec![Value::Undefined], vec![Value::Null]] {
        runtime.steps = work;
        assert_eq!(
            runtime
                .node_get_root_node(Value::Node(leaf), &args, &mut doc)
                .unwrap(),
            Value::Node(parent)
        );
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    }
}

#[test]
fn dictionary_key_and_lookup_have_measured_work_and_heap_admission() {
    fn ready() -> (Runtime, Document, NodeId, NodeId, Value) {
        let (mut runtime, mut doc, parent, _, leaf) = branch();
        let options = runtime.execute("({composed:true})", &mut doc).unwrap();
        (runtime, doc, parent, leaf, options)
    }
    let (mut measured, mut doc, parent, leaf, options) = ready();
    let before = (measured.steps, measured.allocated);
    assert_eq!(
        measured
            .node_get_root_node(Value::Node(leaf), &[options], &mut doc)
            .unwrap(),
        Value::Node(parent)
    );
    let (work, heap) = (before.0 - measured.steps, measured.allocated - before.1);
    assert!(heap > 0);
    for (steps, bytes, success) in [
        (work, heap, true),
        (work - 1, heap, false),
        (work, heap - 1, false),
    ] {
        let (mut runtime, mut doc, parent, leaf, options) = ready();
        let original = state(&doc);
        runtime.steps = steps;
        runtime.allocated = MAX_HEAP - bytes;
        let result = runtime.node_get_root_node(Value::Node(leaf), &[options], &mut doc);
        if success {
            assert_eq!(result.unwrap(), Value::Node(parent));
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        assert_eq!(state(&doc), original);
        clean(&runtime);
    }
}

#[test]
fn brand_then_dictionary_abrupt_precede_local_tree_checks() {
    let (mut runtime, mut doc, _, _, leaf) = branch();
    let options = runtime
        .execute(
            "var hits=0,sentinel={};({get composed(){hits++;throw sentinel;}})",
            &mut doc,
        )
        .unwrap();
    let sentinel = runtime.environments[0].bindings["sentinel"].value.clone();
    for receiver in [
        Value::Null,
        Value::Number(1.0),
        Value::Node(usize::MAX),
        Value::Node(doc.nodes.len()),
    ] {
        runtime.steps = 16;
        let error = runtime
            .node_get_root_node(receiver, std::slice::from_ref(&options), &mut doc)
            .unwrap_err();
        assert_eq!(error.name(), "TypeError");
        assert_eq!(runtime.steps, 0);
        assert_eq!(
            runtime.environments[0].bindings["hits"].value,
            Value::Number(0.0)
        );
    }
    doc.nodes[leaf].parent = Some(usize::MAX);
    let original = state(&doc);
    runtime.steps = MAX_STEPS;
    let error = runtime
        .node_get_root_node(Value::Node(leaf), &[options], &mut doc)
        .unwrap_err();
    assert_eq!(error.kind, ErrorKind::Thrown(sentinel));
    assert_eq!(
        runtime.environments[0].bindings["hits"].value,
        Value::Number(1.0)
    );
    for options in [
        Value::Bool(false),
        Value::Number(0.0),
        Value::String("".into()),
    ] {
        runtime.steps = 24;
        assert_eq!(
            runtime
                .node_get_root_node(Value::Node(leaf), &[options], &mut doc)
                .unwrap_err()
                .name(),
            "TypeError"
        );
        assert_eq!(runtime.steps, 0);
    }
    assert_eq!(state(&doc), original);
    clean(&runtime);
}

#[test]
fn all_represented_object_dictionary_kinds_use_original_receiver_once() {
    // Each expression creates a real value through an existing public path.
    // The inherited accessor must also be reached for legacy host variants.
    for expression in [
        "({})",
        "[]",
        "(function(){})",
        "new Text('options')",
        "document",
        "window",
        "console",
        "Math",
        "JSON",
        "document.body.style",
        "document.body.classList",
        "Node.prototype.getRootNode",
        "Object(1)",
        "Object('x')",
        "Object(true)",
        "Object(Symbol('s'))",
    ] {
        for strict in [false, true] {
            let (mut runtime, mut doc) = fresh();
            let source = format!(
                r#"(function(){{
                const root=Node.prototype.getRootNode,target=new Text('kept'),options={expression};
                let hits=0;
                Object.defineProperty(Object.prototype,'composed',{{configurable:true,get:function(){{
                    if(this!==options)throw new Error('original receiver');hits++;return true;
                }}}});
                try{{if(root.call(target,options)!==target||hits!==1)throw new Error('one inherited Get');}}
                finally{{delete Object.prototype.composed;}}
                return true;
            }})()"#
            );
            assert_eq!(
                evaluate(&mut runtime, &mut doc, &source, strict),
                Value::Bool(true),
                "{expression}, strict={strict}"
            );
        }
    }
}

#[test]
fn event_composed_uses_its_ordinary_native_accessor() {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = r#"(function(){
            const target=new Text('target'),event=new Event('sample',{composed:true});
            const old=Object.getOwnPropertyDescriptor(Event.prototype,'composed');
            if(!old||typeof old.get!=='function'||old.get.call(event)!==true)throw new Error('Event prerequisite');
            let hits=0;
            Object.defineProperty(Event.prototype,'composed',{configurable:true,get:function(){
                if(this!==event)throw new Error('Event receiver');hits++;return old.get.call(this);
            }});
            try{if(target.getRootNode(event)!==target||hits!==1)throw new Error('Event Get');}
            finally{Object.defineProperty(Event.prototype,'composed',old);}
            return true;
        })()"#;
        assert_eq!(
            evaluate(&mut runtime, &mut doc, source, strict),
            Value::Bool(true)
        );
    }
}

#[test]
fn document_alias_and_detached_leaf_kinds_return_canonical_identity() {
    let (mut runtime, mut doc) = fresh();
    let text = doc.create_text_node("raw Document text");
    doc.append_child(doc.root, text);
    let comment = doc.create_comment("kept");
    let pi = doc.create_processing_instruction("kept", "data");
    let doctype = doc
        .nodes
        .iter()
        .position(|n| matches!(n.kind, NodeKind::Doctype(_)))
        .unwrap();
    let fragment = doc.create_document_fragment();
    let original = state(&doc);
    for receiver in [
        Value::Document,
        Value::Node(doc.root),
        Value::Node(text),
        Value::Node(doctype),
    ] {
        assert_eq!(
            runtime.node_get_root_node(receiver, &[], &mut doc).unwrap(),
            Value::Document
        );
    }
    for id in [comment, pi, fragment] {
        assert_eq!(
            runtime
                .node_get_root_node(Value::Node(id), &[], &mut doc)
                .unwrap(),
            Value::Node(id)
        );
    }
    assert_eq!(state(&doc), original);
    clean(&runtime);
}

#[test]
fn accepted_snapshot_depth_endpoint_and_overdeep_host_edge_are_distinct() {
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
            .node_get_root_node(Value::Node(leaf), &[], &mut doc)
            .unwrap(),
        Value::Node(ancestor)
    );
    assert!(before - runtime.steps < 10000);
    assert_eq!(runtime.allocated, MAX_HEAP);
    let extra = doc.create_text_node("too deep");
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
            .node_get_root_node(Value::Node(extra), &[], &mut doc)
            .unwrap_err()
            .name(),
        "TypeError"
    );
    assert_eq!(state(&doc), original);
    clean(&runtime);
}

#[test]
fn malformed_reached_links_and_reciprocal_cycles_refuse_without_mutation() {
    for case in 0..10 {
        let (mut runtime, mut doc, parent, middle, leaf) = branch();
        match case {
            0 => doc.nodes[leaf].parent = Some(usize::MAX),
            1 => doc.nodes[middle].children.clear(),
            2 => doc.nodes[middle].children.push(leaf),
            3 => doc.nodes[leaf].parent = Some(leaf),
            4 => doc.nodes[middle].kind = NodeKind::Comment("wrong".into()),
            5 => doc.nodes[parent].parent = Some(middle),
            6 => doc.nodes[doc.root].parent = Some(middle),
            7 => doc.nodes[leaf].children.push(middle),
            8 => doc.nodes[doc.root].kind = NodeKind::Comment("false root".into()),
            9 => {
                doc.nodes[parent].kind = doc.nodes[middle].kind.clone();
                doc.nodes[parent].parent = Some(middle);
                doc.nodes[middle].children.push(parent);
            }
            _ => unreachable!(),
        }
        let target = if case == 6 || case == 8 {
            doc.root
        } else {
            leaf
        };
        let original = state(&doc);
        assert_eq!(
            runtime
                .node_get_root_node(Value::Node(target), &[], &mut doc)
                .unwrap_err()
                .name(),
            "TypeError",
            "case={case}"
        );
        assert_eq!(state(&doc), original);
        assert!(runtime.steps > MAX_STEPS / 2);
        clean(&runtime);
    }
}

#[test]
fn unrelated_arena_and_large_payload_do_not_change_root_body_cost() {
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
                .node_get_root_node(Value::Node(leaf), &[], &mut doc)
                .unwrap(),
            Value::Node(parent)
        );
        costs.push((before - runtime.steps, runtime.allocated));
    }
    assert_eq!(costs[0], costs[1]);
}

#[test]
fn dictionary_callback_and_wide_scan_share_terminal_budget() {
    for strict in [false, true] {
        let (mut runtime, mut doc, parent, _, leaf) = branch();
        for _ in 0..26000 {
            let child = doc.create_comment("");
            doc.nodes[parent].children.push(child);
            doc.nodes[child].parent = Some(parent);
        }
        runtime.execute("var target,effect=0,hits=0,caught=false,finalized=false;var options={get composed(){hits++;return true;}}", &mut doc).unwrap();
        runtime.environments[0]
            .bindings
            .get_mut("target")
            .unwrap()
            .value = Value::Node(leaf);
        let original = state(&doc);
        let source = "try{target.getRootNode(options,effect=1);}catch(e){caught=true;}finally{finalized=true;}";
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        assert!(result.unwrap_err().is_resource_limit());
        for name in ["effect", "hits"] {
            assert_eq!(
                runtime.environments[0].bindings[name].value,
                Value::Number(1.0)
            );
        }
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
fn getter_effects_survive_final_work_refusal_without_budget_reset() {
    fn ready() -> (Runtime, Document, NodeId, Value) {
        let (mut runtime, mut doc, _, _, leaf) = branch();
        let options = runtime
            .execute(
                "var hits=0;({get composed(){hits++;return false;}})",
                &mut doc,
            )
            .unwrap();
        (runtime, doc, leaf, options)
    }
    let (mut measured, mut doc, leaf, options) = ready();
    let before = (measured.steps, measured.allocated);
    measured
        .node_get_root_node(Value::Node(leaf), &[options], &mut doc)
        .unwrap();
    let (work, heap) = (before.0 - measured.steps, measured.allocated - before.1);
    let (mut runtime, mut doc, leaf, options) = ready();
    let original = state(&doc);
    runtime.steps = work - 1;
    runtime.allocated = MAX_HEAP - heap;
    assert!(
        runtime
            .node_get_root_node(Value::Node(leaf), &[options], &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    assert_eq!(
        runtime.environments[0].bindings["hits"].value,
        Value::Number(1.0)
    );
    assert_eq!(state(&doc), original);
    clean(&runtime);
}

#[test]
fn saved_native_invocation_cuts_preserve_cleanup_and_metadata() {
    fn ready(dictionary: bool) -> (Runtime, Document, NodeId, Value, Vec<Value>) {
        let (mut runtime, mut doc, _, _, leaf) = branch();
        let method = runtime
            .execute("Node.prototype.getRootNode", &mut doc)
            .unwrap();
        let args = if dictionary {
            vec![runtime.execute("({composed:true})", &mut doc).unwrap()]
        } else {
            vec![]
        };
        (runtime, doc, leaf, method, args)
    }
    for dictionary in [false, true] {
        let (mut measured, mut doc, leaf, method, args) = ready(dictionary);
        let before = (measured.steps, measured.allocated);
        let expected = measured
            .call(method, args, Value::Node(leaf), &mut doc)
            .unwrap();
        let (work, heap) = (before.0 - measured.steps, measured.allocated - before.1);
        assert!(heap > 0);
        for (steps, bytes, success) in [
            (work, heap, true),
            (work - 1, heap, false),
            (work, heap - 1, false),
            (8, heap, false),
        ] {
            let (mut runtime, mut doc, leaf, method, args) = ready(dictionary);
            let original = state(&doc);
            let bag = runtime.property_object(&method).unwrap();
            let metadata = format!("{:?}", runtime.objects[bag].values);
            runtime.steps = steps;
            runtime.allocated = MAX_HEAP - bytes;
            let result = runtime.call(method, args, Value::Node(leaf), &mut doc);
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
fn two_live_realms_retain_separate_root_metadata_bags() {
    let (mut a, mut da) = fresh();
    let (mut b, mut db) = fresh();
    a.execute(
        "Object.defineProperty(Node.prototype.getRootNode,'name',{value:'A'})",
        &mut da,
    )
    .unwrap();
    assert_eq!(
        b.execute("Node.prototype.getRootNode.name", &mut db)
            .unwrap(),
        Value::String("getRootNode".into())
    );
    b.execute(
        "Object.defineProperty(Node.prototype.getRootNode,'name',{value:'B'})",
        &mut db,
    )
    .unwrap();
    assert_eq!(
        a.execute("Node.prototype.getRootNode.name", &mut da)
            .unwrap(),
        Value::String("A".into())
    );
    assert_eq!(
        b.execute("Node.prototype.getRootNode.name", &mut db)
            .unwrap(),
        Value::String("B".into())
    );
    clean(&a);
    clean(&b);
}
