use super::*;
use crate::dom::DomString;

const CASES: &str = include_str!("../../../tests/fixtures/node-clone.js");

fn fresh() -> (Runtime, Document) {
    (Runtime::try_new().unwrap(), Document::parse(""))
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
        doc.nodes
            .iter()
            .map(|node| node.children.capacity())
            .collect(),
    )
}
fn execute(runtime: &mut Runtime, doc: &mut Document, source: &str, strict: bool) -> Result<Value> {
    let result = if strict {
        runtime.execute_strict(source, doc)
    } else {
        runtime.execute(source, doc)
    };
    clean(runtime);
    result
}
fn independent(name: &str) {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = format!("{CASES}\nnodeCloneCases.{name}({strict});");
        assert_eq!(
            execute(&mut runtime, &mut doc, &source, strict).unwrap(),
            Value::Bool(true),
            "{name}, strict={strict}"
        );
    }
}
macro_rules! cases {($($name:ident),* $(,)?)=>{$(#[test]fn $name(){independent(stringify!($name));})*};}
cases!(
    metadata_and_cached_method,
    represented_complete_key_order,
    authentic_receiver_without_public_property_reads,
    optional_boolean_defaults_and_truthy_objects,
    argument_expressions_finish_before_copy_and_throw,
    exact_text_comment_data_and_independent_mutation,
    processing_instruction_mutated_data_is_not_revalidated,
    shallow_element_fields_without_children,
    deep_current_child_order_and_text_segmentation,
    foreign_namespaces_and_attribute_annotations,
    fragment_copy_detachment_and_later_splice,
    shallow_template_has_fresh_empty_content,
    deep_nested_template_copies_both_distinct_graphs,
    cloned_template_content_is_an_independent_fragment,
    detached_details_and_base_copies_preserve_source_state,
    clone_omits_own_state_listeners_and_prototype_override,
    saved_method_shadows_replacement_and_no_resurrection,
);

fn exact(units: &[u16]) -> DomString {
    DomString::from_units_owned(units.to_vec()).unwrap()
}
fn seed(branch: bool) -> (Document, NodeId) {
    let mut doc = Document::parse("");
    let text = doc.create_text_node_owned(exact(&[0xd800, 65])).unwrap();
    let source = if branch {
        let parent = doc.create_element("div");
        doc.set_attr(parent, "data-kept", "yes");
        doc.append_child(parent, text);
        assert_eq!(doc.nodes[text].parent, Some(parent));
        parent
    } else {
        text
    };
    for node in &mut doc.nodes {
        node.children.shrink_to_fit();
    }
    doc.nodes.shrink_to_fit();
    (doc, source)
}
fn copy(runtime: &mut Runtime, doc: &mut Document, source: NodeId) -> Result<NodeId> {
    match runtime.node_clone_node(Value::Node(source), &[Value::Bool(true)], doc)? {
        Value::Node(id) => Ok(id),
        _ => panic!("clone result is not a node"),
    }
}

#[test]
fn clone_bootstrap_reports_actual_admission_and_legacy_removal() {
    let mut runtime = Runtime::uninitialized().finish_bootstrap().unwrap();
    println!(
        "NODE_CLONE_BOOTSTRAP steps={} allocated={} objects={} capacity={} native={} prototypes={}",
        runtime.steps,
        runtime.allocated,
        runtime.objects.len(),
        runtime.objects.capacity(),
        runtime.native_properties.len(),
        runtime.prototypes.len()
    );
    println!(
        "NODE_CLONE_TYPES native={} legacy_metadata={} key={} entry={} block={} object={}",
        std::mem::size_of::<Native>(),
        NATIVE_METADATA_BYTES,
        std::mem::size_of::<PropertyKey>(),
        std::mem::size_of::<(PropertyKey, Property)>(),
        16 * (std::mem::size_of::<PropertyKey>() + std::mem::size_of::<Property>())
            + 32 * std::mem::size_of::<usize>()
            + 64,
        std::mem::size_of::<ScriptObject>()
    );
    assert_eq!(runtime.objects.len(), dom_prototypes::BOOTSTRAP_OBJECTS);
    assert!(!runtime.native_properties.contains_key("DOM.cloneNode"));
    assert!(!runtime.native_properties.contains_key("DOM.Node.cloneNode"));
    assert_eq!(runtime.native_properties.len(), 299);
    let prototype = runtime.dom_proto_id("Node").unwrap();
    let PropertyValue::Data {
        value: Value::Native(method),
        writable: true,
    } = &runtime.objects[prototype].values[&PropertyKey::from("cloneNode")].value
    else {
        panic!("ordinary clone method")
    };
    assert!(method.properties.is_some());
    assert_eq!(method.name, "DOM.Node.cloneNode");
    assert!(runtime.steps > 0 && runtime.allocated < MAX_HEAP);
    clean(&runtime);
}

#[test]
fn brand_and_optional_boolean_precede_document_refusal() {
    let (mut runtime, mut doc) = fresh();
    let root = doc.root;
    for receiver in [
        Value::Null,
        Value::Undefined,
        Value::Bool(true),
        Value::Node(usize::MAX),
    ] {
        let original = state(&doc);
        for steps in [15, 16] {
            runtime.steps = steps;
            runtime.allocated = MAX_HEAP;
            let error = runtime
                .node_clone_node(receiver.clone(), &[], &mut doc)
                .unwrap_err();
            if steps == 16 {
                assert_eq!(error.name(), "TypeError");
            } else {
                assert!(error.is_resource_limit());
            }
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            assert_eq!(state(&doc), original);
        }
    }
    for receiver in [Value::Document, Value::Node(root)] {
        for steps in [23, 24] {
            runtime.steps = steps;
            runtime.allocated = MAX_HEAP;
            let original = state(&doc);
            let error = runtime
                .node_clone_node(receiver.clone(), &[Value::Bool(true)], &mut doc)
                .unwrap_err();
            if steps == 24 {
                assert!(error.is_unsupported());
            } else {
                assert!(error.is_resource_limit());
            }
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            assert_eq!(state(&doc), original);
        }
    }
    for malformed in [false, true] {
        let (_, mut doc) = fresh();
        let id = if malformed {
            let id = doc.create_element("div");
            doc.nodes[id].kind = NodeKind::Document;
            id
        } else {
            doc.nodes[doc.root].parent = Some(doc.root);
            doc.root
        };
        runtime.steps = 24;
        runtime.allocated = MAX_HEAP;
        let original = state(&doc);
        assert_eq!(
            runtime
                .node_clone_node(Value::Node(id), &[], &mut doc)
                .unwrap_err()
                .name(),
            "TypeError"
        );
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        assert_eq!(state(&doc), original);
    }
    for (argument, deep) in [
        (None, false),
        (Some(Value::Undefined), false),
        (Some(Value::Null), false),
        (Some(Value::Bool(false)), false),
        (Some(Value::Number(0.0)), false),
        (Some(Value::Number(f64::NAN)), false),
        (Some(Value::String("".into())), false),
        (Some(Value::Bool(true)), true),
        (Some(Value::Number(1.0)), true),
        (Some(Value::String("false".into())), true),
    ] {
        let (mut doc, source) = seed(true);
        runtime.steps = MAX_STEPS;
        runtime.allocated = 1024;
        let args: Vec<_> = argument.into_iter().collect();
        let Value::Node(result) = runtime
            .node_clone_node(Value::Node(source), &args, &mut doc)
            .unwrap()
        else {
            panic!("clone result")
        };
        assert_eq!(doc.nodes[result].children.len(), usize::from(deep));
        assert_eq!(doc.nodes[result].parent, None);
    }
    clean(&runtime);
}

#[test]
fn exact_leaf_results_use_intrinsic_interfaces_and_no_source_metadata() {
    let (mut runtime, mut doc) = fresh();
    let text=runtime.execute("function Other(){}; Other.prototype={custom:true}; var original=Reflect.construct(Text,['x'],Other); original.own=9; original;",&mut doc).unwrap();
    let Value::Node(text_id) = text else {
        panic!("Text")
    };
    doc.replace_character_data(text_id, exact(&[0xd800, 0]))
        .unwrap();
    let comment = doc.create_comment_owned(exact(&[0xdc00, 13, 10])).unwrap();
    let pi = doc
        .create_processing_instruction_owned("probe".into(), exact(&[63, 62, 0xdfff]))
        .unwrap();
    for (source, interface, units) in [
        (text_id, "Text", &[0xd800, 0][..]),
        (comment, "Comment", &[0xdc00, 13, 10][..]),
        (pi, "ProcessingInstruction", &[63, 62, 0xdfff][..]),
    ] {
        let before = (runtime.objects.len(), runtime.native_properties.len());
        let fresh = doc.nodes.len();
        let result = copy(&mut runtime, &mut doc, source).unwrap();
        assert_eq!(result, fresh);
        assert_eq!(
            (runtime.objects.len(), runtime.native_properties.len()),
            before
        );
        assert_eq!(doc.nodes[result].parent, None);
        assert!(doc.nodes[result].children.is_empty());
        let data = match &doc.nodes[result].kind {
            NodeKind::Text(data) | NodeKind::Comment(data) => data,
            NodeKind::ProcessingInstruction { target, data } => {
                assert_eq!(target, "probe");
                data
            }
            _ => panic!("kind"),
        };
        assert_eq!(data.units().collect::<Vec<_>>(), units);
        assert_eq!(
            runtime.prototype_of_in(&Value::Node(result), &doc).unwrap(),
            Some(Value::Object(runtime.dom_proto_id(interface).unwrap()))
        );
        assert_eq!(
            runtime.get(Value::Node(result), "own", &mut doc).unwrap(),
            Value::Undefined
        );
        doc.replace_character_data(source, "changed".into())
            .unwrap();
        assert_eq!(runtime.dom_text_units(result, &doc).unwrap(), units);
    }
    clean(&runtime);
}

#[test]
fn absolute_budget_copyback_matches_checked_document_seam() {
    let (mut runtime, _) = fresh();
    for branch in [false, true] {
        for (steps, allocated) in [(MAX_STEPS, 777), (26, 777), (MAX_STEPS, MAX_HEAP)] {
            let (mut direct, source) = seed(branch);
            let (mut wrapped, other) = seed(branch);
            assert_eq!(source, other);
            let mut budget = DomMutationBudget {
                steps: steps - 26,
                allocated,
                heap_limit: MAX_HEAP,
            };
            let expected = direct
                .clone_node_checked(source, true, &mut budget)
                .map(Value::Node)
                .map_err(clone_error);
            runtime.steps = steps;
            runtime.allocated = allocated;
            let actual =
                runtime.node_clone_node(Value::Node(other), &[Value::Bool(true)], &mut wrapped);
            match (actual, expected) {
                (Ok(a), Ok(b)) => assert_eq!(a, b),
                (Err(a), Err(b)) => {
                    assert_eq!(a.name(), b.name());
                    assert_eq!(a.is_resource_limit(), b.is_resource_limit());
                }
                _ => panic!("wrapper/seam outcome"),
            }
            assert_eq!(
                (runtime.steps, runtime.allocated),
                (budget.steps, budget.allocated)
            );
            assert_eq!(state(&wrapped), state(&direct));
        }
    }
    clean(&runtime);
}

#[test]
fn small_native_body_work_cuts_are_atomic() {
    let (mut runtime, _) = fresh();
    let (mut measured, source) = seed(false);
    runtime.steps = MAX_STEPS;
    runtime.allocated = 1024;
    let expected_id = copy(&mut runtime, &mut measured, source).unwrap();
    let work = MAX_STEPS - runtime.steps;
    let heap = runtime.allocated - 1024;
    assert!(work > 26 && heap > 0);
    for steps in 0..=work {
        let (mut doc, source) = seed(false);
        let original = state(&doc);
        runtime.steps = steps;
        runtime.allocated = MAX_HEAP - heap;
        let result = copy(&mut runtime, &mut doc, source);
        if steps == work {
            assert_eq!(result.unwrap(), expected_id);
            assert_eq!(state(&doc), state(&measured));
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            assert_eq!(runtime.steps, 0);
            assert_eq!(state(&doc), original);
        }
        clean(&runtime);
    }
}

#[test]
fn native_body_heap_endpoints_precede_publication() {
    let (mut runtime, _) = fresh();
    for branch in [false, true] {
        let (mut measured, source) = seed(branch);
        runtime.steps = MAX_STEPS;
        runtime.allocated = 1024;
        let expected = copy(&mut runtime, &mut measured, source).unwrap();
        let (work, heap) = (MAX_STEPS - runtime.steps, runtime.allocated - 1024);
        for (steps, bytes, success) in [
            (work, heap, true),
            (work - 1, heap, false),
            (work, heap - 1, false),
            (work, 0, false),
        ] {
            let (mut doc, source) = seed(branch);
            let original = state(&doc);
            runtime.steps = steps;
            runtime.allocated = MAX_HEAP - bytes;
            let result = copy(&mut runtime, &mut doc, source);
            if success {
                assert_eq!(result.unwrap(), expected);
                assert_eq!(state(&doc), state(&measured));
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert_eq!(state(&doc), original);
            }
            clean(&runtime);
        }
    }
}

#[test]
fn selected_malformed_errors_map_without_partial_copy() {
    let (mut runtime, _) = fresh();
    for case in 0..4 {
        let (mut doc, source) = seed(true);
        match case {
            0 => doc.nodes[source].children.push(usize::MAX),
            1 => {
                let child = doc.create_comment("bad backlink");
                doc.nodes[source].children.push(child);
            }
            2 => {
                let child = doc.nodes[source].children[0];
                doc.nodes[source].children.push(child);
            }
            3 => {
                let child = doc.nodes[source].children[0];
                doc.nodes[child].children.push(source);
            }
            _ => unreachable!(),
        }
        let original = state(&doc);
        runtime.steps = MAX_STEPS;
        runtime.allocated = 1234;
        assert_eq!(
            copy(&mut runtime, &mut doc, source).unwrap_err().name(),
            "TypeError"
        );
        assert_eq!(state(&doc), original);
        assert!(runtime.steps < MAX_STEPS && runtime.allocated >= 1234);
        clean(&runtime);
    }
    for error in [DomDataError::InvalidNode, DomDataError::InvalidData] {
        assert_eq!(clone_error(error).name(), "TypeError");
    }
    for error in [DomDataError::LimitExceeded, DomDataError::AllocationFailed] {
        assert!(clone_error(error).is_resource_limit());
    }
}

#[test]
fn scoped_dispatch_preserves_existing_node_routes() {
    let (mut runtime, mut doc) = fresh();
    runtime.steps = MAX_STEPS;
    runtime.allocated = 0;
    runtime
        .node_data_call_preflight("DOM.Node.cloneNode")
        .unwrap();
    assert_eq!((MAX_STEPS - runtime.steps, runtime.allocated), (22, 50));
    runtime.steps = MAX_STEPS;
    assert_eq!(
        runtime
            .node_data_native("cloneNode", Value::Null, &[], &mut doc)
            .unwrap_err()
            .name(),
        "TypeError"
    );
    assert_eq!(MAX_STEPS - runtime.steps, 19 + 16);
    for (name, work) in [
        ("getRootNode", 12 + 16),
        ("isEqualNode", 23 + 16),
        ("hasChildNodes", 14 + 16),
        ("normalize", 10 + 16),
        ("isSameNode", 11 + 16),
        ("compareDocumentPosition", 24 + 16),
        ("contains", 9 + 16),
        ("getIsConnected", 24 + 16),
        ("getNodeValue", 9),
        ("setNodeValue", 9),
        ("getTextContent", 9),
        ("setTextContent", 9),
    ] {
        runtime.steps = MAX_STEPS;
        assert_eq!(
            runtime
                .node_data_native(name, Value::Null, &[], &mut doc)
                .unwrap_err()
                .name(),
            "TypeError"
        );
        assert_eq!(MAX_STEPS - runtime.steps, work, "{name}");
    }
    let source = doc.create_text_node("");
    runtime.steps = MAX_STEPS;
    assert_eq!(
        runtime
            .dom_native("cloneNode", Value::Node(source), &[], &mut doc)
            .unwrap_err()
            .name(),
        "TypeError"
    );
    clean(&runtime);
}

#[test]
fn saved_native_boundaries_preserve_metadata_and_cleanup() {
    fn ready() -> (Runtime, Document, NodeId, Value) {
        let (mut runtime, _) = fresh();
        let (mut doc, source) = seed(false);
        let method = runtime
            .execute("Node.prototype.cloneNode", &mut doc)
            .unwrap();
        (runtime, doc, source, method)
    }
    let (mut measured, mut doc, source, method) = ready();
    let initial = (measured.steps, measured.allocated);
    let expected = measured
        .call(method, vec![], Value::Node(source), &mut doc)
        .unwrap();
    let (work, heap) = (initial.0 - measured.steps, measured.allocated - initial.1);
    for (steps, bytes, success) in [
        (work, heap, true),
        (work - 1, heap, false),
        (work, heap - 1, false),
        (8, heap, false),
    ] {
        let (mut runtime, mut doc, source, method) = ready();
        let original = state(&doc);
        let bag = runtime.property_object(&method).unwrap();
        let metadata = format!("{:?}", runtime.objects[bag].values);
        let objects = (runtime.objects.len(), runtime.objects.capacity());
        let Value::Native(native) = &method else {
            panic!("Native")
        };
        let saved = native.clone();
        runtime.steps = steps;
        runtime.allocated = MAX_HEAP - bytes;
        let result = runtime.call(method.clone(), vec![], Value::Node(source), &mut doc);
        if success {
            assert_eq!(result.unwrap(), expected);
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            assert_eq!(state(&doc), original);
        }
        let Value::Native(native) = &method else {
            panic!("Native")
        };
        assert!(Rc::ptr_eq(native, &saved));
        assert_eq!(format!("{:?}", runtime.objects[bag].values), metadata);
        assert_eq!((runtime.objects.len(), runtime.objects.capacity()), objects);
        clean(&runtime);
    }
}

#[test]
fn terminal_shared_work_keeps_argument_prefix_and_distinguishes_later_vm_failure() {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = doc.create_text_node(&"x".repeat(3 * MAX_STEPS));
        runtime.execute("var target,effect=0,caught=false,finalized=false,completed=false;var clone=Node.prototype.cloneNode;",&mut doc).unwrap();
        runtime.environments[0]
            .bindings
            .get_mut("target")
            .unwrap()
            .value = Value::Node(source);
        let original = state(&doc);
        let error=execute(&mut runtime,&mut doc,"try{clone.call(target,true,effect=1);completed=true;}catch(e){caught=true;}finally{finalized=true;}",strict).unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(state(&doc), original);
        assert_eq!(
            runtime.environments[0].bindings["effect"].value,
            Value::Number(1.0)
        );
        for name in ["caught", "finalized", "completed"] {
            assert_eq!(
                runtime.environments[0].bindings[name].value,
                Value::Bool(false)
            );
        }
        // Native success is retained when a later authored loop exhausts. No
        // claim of expression/evaluation rollback follows from body atomicity.
        let (mut runtime, mut doc) = fresh();
        runtime
            .execute(
                "var source=new Text('kept'),copied,caught=false,finalized=false;",
                &mut doc,
            )
            .unwrap();
        let old_len = doc.nodes.len();
        let error=execute(&mut runtime,&mut doc,"try{copied=source.cloneNode();while(true){}}catch(e){caught=true;}finally{finalized=true;}",strict).unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(doc.nodes.len(), old_len + 1);
        assert_eq!(
            runtime.environments[0].bindings["copied"].value,
            Value::Node(old_len)
        );
        assert!(
            matches!(&doc.nodes[old_len].kind,NodeKind::Text(data) if data.scalar()==Some("kept"))
        );
        for name in ["caught", "finalized"] {
            assert_eq!(
                runtime.environments[0].bindings[name].value,
                Value::Bool(false)
            );
        }
    }
}

#[test]
fn extra_expression_exhaustion_prevents_native_entry() {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        runtime.execute("var source=new Text('kept'),effect=0,caught=false,finalized=false;function exhaust(){effect=1;while(true){}}",&mut doc).unwrap();
        let original = state(&doc);
        let result = execute(
            &mut runtime,
            &mut doc,
            "try{source.cloneNode(false,exhaust());}catch(e){caught=true;}finally{finalized=true;}",
            strict,
        );
        assert!(result.unwrap_err().is_resource_limit());
        assert_eq!(state(&doc), original);
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
    }
}

#[test]
fn two_live_realms_have_distinct_mutable_clone_metadata() {
    let (mut a, mut da) = fresh();
    let (mut b, mut db) = fresh();
    a.execute(
        "Object.defineProperty(Node.prototype.cloneNode,'name',{value:'A'})",
        &mut da,
    )
    .unwrap();
    assert_eq!(
        b.execute("Node.prototype.cloneNode.name", &mut db).unwrap(),
        Value::String("cloneNode".into())
    );
    b.execute(
        "Object.defineProperty(Node.prototype.cloneNode,'name',{value:'B'})",
        &mut db,
    )
    .unwrap();
    assert_eq!(
        a.execute("Node.prototype.cloneNode.name", &mut da).unwrap(),
        Value::String("A".into())
    );
    assert_eq!(
        b.execute("Node.prototype.cloneNode.name", &mut db).unwrap(),
        Value::String("B".into())
    );
    clean(&a);
    clean(&b);
}
