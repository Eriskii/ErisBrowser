use super::*;

const CASES: &str = include_str!("../../../tests/fixtures/processing-instruction.js");
fn fresh() -> (Runtime, Document) {
    (
        Runtime::try_new().unwrap(),
        Document::parse(
            "<!doctype html><html><head></head><body><p id='kept'>kept</p></body></html>",
        ),
    )
}
fn clean(runtime: &Runtime) {
    assert!(runtime.frames.is_empty());
    assert_eq!(
        (runtime.calls, runtime.eval_depth, runtime.stack_units),
        (0, 0, 0)
    );
}
fn independent_case(name: &str) {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = format!("{CASES}\npiFollowupCases.{name}();");
        let result = if strict {
            runtime.execute_strict(&source, &mut doc)
        } else {
            runtime.execute(&source, &mut doc)
        };
        assert_eq!(result.unwrap(), Value::Bool(true), "{name} strict={strict}");
        clean(&runtime);
    }
}
macro_rules! independent {
    ($($name:ident),* $(,)?) => {$ (
        #[test] fn $name() { independent_case(stringify!($name)); }
    )*};
}
independent!(
    constructor_arity_defaults_and_call,
    factory_brand_arity_and_required_data,
    xml_name_valid_targets,
    xml_name_invalid_targets,
    xml_name_unicode_boundaries,
    data_terminator_and_literal_storage,
    genuine_node_identity_and_data,
    conversions_then_alternate_prototype,
    validation_follows_conversions_and_prototype,
    conversion_abrupt_identity,
    prototype_abrupt_precedes_validation,
    factory_conversion_order_and_abrupt_identity,
);

#[test]
fn pi_bootstrap_admission_is_bounded() {
    let runtime = Runtime::uninitialized().finish_bootstrap().unwrap();
    println!(
        "PI_BOOTSTRAP steps={} allocated={} objects={} capacity={} native={} prototypes={}",
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
}

fn custom_target(runtime: &mut Runtime, doc: &mut Document) -> Value {
    runtime.execute("var piCustom={};var piTarget=(function(){}).bind(null);Object.defineProperty(piTarget,'prototype',{value:piCustom});piTarget;", doc).unwrap()
}

#[test]
fn pi_constructor_measured_exact_and_one_short_never_publish_partial_node() {
    for full in [false, true] {
        fn setup(full: bool) -> (Runtime, Document, Value) {
            let (mut runtime, mut doc) = fresh();
            let target = custom_target(&mut runtime, &mut doc);
            if full {
                doc.nodes.shrink_to_fit();
                assert_eq!(doc.nodes.len(), doc.nodes.capacity());
            } else {
                doc.nodes.try_reserve_exact(1).unwrap();
            }
            (runtime, doc, target)
        }
        let args = [
            Value::String("probe".into()),
            Value::String("\u{1d11e} retained".into()),
        ];
        let (mut runtime, mut doc, target) = setup(full);
        let before = (runtime.steps, runtime.allocated);
        runtime.pi_construct(&args, target, &mut doc).unwrap();
        let work = before.0 - runtime.steps;
        let heap = runtime.allocated - before.1;
        for (steps, bytes, success) in [
            (work, heap, true),
            (work - 1, heap, false),
            (work, heap - 1, false),
        ] {
            let (mut runtime, mut doc, target) = setup(full);
            let before = format!("{doc:?}");
            let count = doc.nodes.len();
            runtime.steps = steps;
            runtime.allocated = MAX_HEAP - bytes;
            let result = runtime.pi_construct(&args, target, &mut doc);
            if success {
                let node = result.unwrap();
                assert_eq!(node, Value::Node(count));
                assert_eq!(runtime.steps, 0);
                assert_eq!(runtime.allocated, MAX_HEAP);
                runtime.steps = MAX_STEPS;
                let expected = runtime.environments[0].bindings["piCustom"].value.clone();
                assert_eq!(
                    runtime.prototype_of_in(&node, &doc).unwrap(),
                    Some(expected)
                );
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert_eq!(format!("{doc:?}"), before);
                // A later genuine default node with the same unused ID must
                // not inherit a prematurely published custom override.
                runtime.steps = MAX_STEPS;
                runtime.allocated = 0;
                let default = runtime.environments[0].bindings["ProcessingInstruction"]
                    .value
                    .clone();
                let node = runtime.pi_construct(&args, default, &mut doc).unwrap();
                let expected =
                    Value::Object(runtime.dom_proto_id("ProcessingInstruction").unwrap());
                assert_eq!(
                    runtime.prototype_of_in(&node, &doc).unwrap(),
                    Some(expected)
                );
            }
            clean(&runtime);
        }
    }
}

#[test]
fn character_data_setter_measured_refusal_preserves_exact_old_state() {
    for kind in ["Text", "Comment", "ProcessingInstruction"] {
        fn setup(kind: &str) -> (Runtime, Document, Value) {
            let (mut runtime, mut doc) = fresh();
            let source = if kind == "ProcessingInstruction" {
                "new ProcessingInstruction('probe','old')".into()
            } else {
                format!("new {kind}('old')")
            };
            let node = runtime.execute(&source, &mut doc).unwrap();
            (runtime, doc, node)
        }
        let value = Value::String("\u{1d11e}new?>".into());
        let (mut runtime, mut doc, node) = setup(kind);
        let before = (runtime.steps, runtime.allocated);
        runtime
            .pi_native("setData", node, std::slice::from_ref(&value), &mut doc)
            .unwrap();
        let work = before.0 - runtime.steps;
        let heap = runtime.allocated - before.1;
        for (steps, bytes, success) in [
            (work, heap, true),
            (work - 1, heap, false),
            (work, heap - 1, false),
        ] {
            let (mut runtime, mut doc, node) = setup(kind);
            let before = format!("{doc:?}");
            runtime.steps = steps;
            runtime.allocated = MAX_HEAP - bytes;
            let result = runtime.pi_native(
                "setData",
                node.clone(),
                std::slice::from_ref(&value),
                &mut doc,
            );
            if success {
                result.unwrap();
                runtime.steps = MAX_STEPS;
                runtime.allocated = 0;
                assert_eq!(
                    runtime
                        .pi_native("data", node.clone(), &[], &mut doc)
                        .unwrap(),
                    value
                );
                assert_eq!(
                    runtime.pi_native("length", node, &[], &mut doc).unwrap(),
                    Value::Number(7.0)
                );
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert_eq!(format!("{doc:?}"), before);
            }
            clean(&runtime);
        }
    }
}

#[test]
fn pi_newtarget_side_effect_consumes_last_node_before_fresh_admission() {
    let (mut runtime, mut doc) = fresh();
    runtime.execute("var piEffect=0;var piTarget=(function(){}).bind(null);Object.defineProperty(piTarget,'prototype',{get:function(){piEffect++;document.createTextNode('callback');return {};}});",&mut doc).unwrap();
    while doc.nodes.len() < MAX_NODES - 1 {
        doc.nodes.push(crate::dom::Node {
            parent: None,
            children: vec![],
            kind: NodeKind::Comment(String::new()),
        });
    }
    let target = runtime.environments[0].bindings["piTarget"].value.clone();
    let before = doc.nodes.len();
    let error = runtime
        .pi_construct(&[Value::String("probe".into())], target, &mut doc)
        .unwrap_err();
    assert!(error.is_resource_limit());
    assert_eq!(doc.nodes.len(), before + 1);
    assert!(matches!(&doc.nodes.last().unwrap().kind,NodeKind::Text(text) if text=="callback"));
    assert_eq!(
        runtime.environments[0].bindings["piEffect"].value,
        Value::Number(1.0)
    );
    clean(&runtime);
}

#[test]
fn pi_and_character_data_lone_surrogates_refuse_without_mutation() {
    let (mut runtime, mut doc) = fresh();
    let node = runtime
        .execute("new ProcessingInstruction('probe','old')", &mut doc)
        .unwrap();
    let text = Value::String(vec![0xd800].into());
    let before = format!("{doc:?}");
    let constructor = runtime.environments[0].bindings["ProcessingInstruction"]
        .value
        .clone();
    assert!(
        runtime
            .pi_construct(
                &[Value::String("probe".into()), text.clone()],
                constructor,
                &mut doc
            )
            .unwrap_err()
            .is_unsupported()
    );
    assert_eq!(format!("{doc:?}"), before);
    assert!(
        runtime
            .pi_native("setData", node.clone(), &[text], &mut doc)
            .unwrap_err()
            .is_unsupported()
    );
    assert_eq!(format!("{doc:?}"), before);
    for receiver in [
        Value::Object(0),
        Value::Document,
        Value::Node(doc.nodes.len()),
    ] {
        assert!(runtime.pi_native("data", receiver, &[], &mut doc).is_err());
    }
    clean(&runtime);
}

fn accessor_case(name: &str) {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = format!(
            "{}\npiAccessorCases.{name}();",
            include_str!("../../../tests/fixtures/processing-instruction-accessors.js")
        );
        let result = if strict {
            runtime.execute_strict(&source, &mut doc)
        } else {
            runtime.execute(&source, &mut doc)
        };
        assert_eq!(result.unwrap(), Value::Bool(true), "{name} strict={strict}");
        clean(&runtime);
    }
}
macro_rules! accessor {
    ($($name:ident),* $(,)?) => {$ (
        #[test] fn $name() { accessor_case(stringify!($name)); }
    )*};
}
// The original authentic_brands_before_conversion remains an external
// ordinary-success expectation unmet at host setPrototypeOf (attempt 1).
accessor!(
    descriptors_and_placement,
    scalar_data_and_setter_defaults,
    setter_callback_order_and_abrupt_identity,
    own_shadows_and_readonly_assignments,
    prototype_replacement_deletion_and_saved_getters,
    pi_setter_clone_and_html_serialization
);

#[test]
fn pi_saved_accessor_call_admission_restores_outer_stack() {
    fn setup() -> (Runtime, Document, Value, Value) {
        let (mut runtime, mut doc) = fresh();
        let getter = runtime
            .execute(
                "Object.getOwnPropertyDescriptor(CharacterData.prototype,'length').get",
                &mut doc,
            )
            .unwrap();
        let node = Value::Node(doc.create_text_node("\u{1d11e}"));
        runtime.frames.try_reserve_exact(32).unwrap();
        runtime.stack_units = 8;
        (runtime, doc, getter, node)
    }
    let (mut runtime, mut doc, getter, node) = setup();
    let before = (runtime.steps, runtime.allocated);
    assert_eq!(
        runtime.call(getter, vec![], node, &mut doc).unwrap(),
        Value::Number(2.0)
    );
    let work = before.0 - runtime.steps;
    let heap = runtime.allocated - before.1;
    assert!(heap > 0);
    for (steps, bytes, success) in [
        (work, heap, true),
        (work - 1, heap, false),
        (work, heap - 1, false),
    ] {
        let (mut runtime, mut doc, getter, node) = setup();
        let before = format!("{doc:?}");
        runtime.steps = steps;
        runtime.allocated = MAX_HEAP - bytes;
        let result = runtime.call(getter, vec![], node, &mut doc);
        if success {
            assert_eq!(result.unwrap(), Value::Number(2.0));
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        assert_eq!(format!("{doc:?}"), before);
        assert_eq!(runtime.stack_units, 8);
        runtime.stack_units = 0;
        clean(&runtime);
    }
}

#[test]
fn pi_and_data_recheck_retained_bytes_after_callback_effects() {
    for setter in [false, true] {
        let (mut runtime, mut doc) = fresh();
        runtime.execute("var piEffect=0;var piNode=new ProcessingInstruction('probe','');var piValue={toString:function(){piEffect++;document.createTextNode('x');return 'two';}};var piTarget=(function(){}).bind(null);Object.defineProperty(piTarget,'prototype',{get:function(){piEffect++;document.createTextNode('x');return {};}});",&mut doc).unwrap();
        // Authentic builder operations keep the real retained-byte ledger.
        // Fill all but one byte outside the measured author execution.
        while doc.retained_bytes() < crate::dom::MAX_DOM_BYTES - 1 {
            let length =
                (crate::dom::MAX_DOM_BYTES - 1 - doc.retained_bytes()).min(8 * 1024 * 1024);
            let data = "x".repeat(length);
            let id = doc.create_text_node(&data);
            assert_ne!(id, doc.root);
        }
        let count = doc.nodes.len();
        let node = runtime.environments[0].bindings["piNode"].value.clone();
        let error = if setter {
            let value = runtime.environments[0].bindings["piValue"].value.clone();
            runtime
                .pi_native("setData", node.clone(), &[value], &mut doc)
                .unwrap_err()
        } else {
            let target = runtime.environments[0].bindings["piTarget"].value.clone();
            runtime
                .pi_construct(&[Value::String("probe".into())], target, &mut doc)
                .unwrap_err()
        };
        assert!(error.is_resource_limit());
        assert_eq!(doc.retained_bytes(), crate::dom::MAX_DOM_BYTES);
        assert_eq!(doc.nodes.len(), count + 1);
        assert!(matches!(&doc.nodes.last().unwrap().kind,NodeKind::Text(text) if text=="x"));
        assert_eq!(
            runtime.environments[0].bindings["piEffect"].value,
            Value::Number(1.0)
        );
        assert_eq!(
            runtime.pi_native("data", node, &[], &mut doc).unwrap(),
            Value::String("".into())
        );
        clean(&runtime);
    }
}

#[test]
fn authentic_brands_with_constructor_prototype_override() {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = include_str!(
            "../../../tests/fixtures/processing-instruction-authentic-constructor-override.js"
        );
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        assert_eq!(result.unwrap(), Value::Bool(true));
        clean(&runtime);
    }
}
