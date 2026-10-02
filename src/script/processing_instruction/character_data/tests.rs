use super::*;
const CASES: &str = include_str!("../../../../tests/fixtures/character-data.js");
fn fresh() -> (Runtime, Document) {
    (Runtime::try_new().unwrap(), Document::parse("<p>kept</p>"))
}
fn clean(runtime: &Runtime) {
    assert!(runtime.frames.is_empty());
    assert_eq!(
        (runtime.calls, runtime.stack_units, runtime.eval_depth),
        (0, 0, 0)
    );
}
fn independent_case(name: &str) {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = format!("{CASES}\ncharacterDataCases.{name}();");
        let result = if strict {
            runtime.execute_strict(&source, &mut doc)
        } else {
            runtime.execute(&source, &mut doc)
        };
        assert_eq!(result.unwrap(), Value::Bool(true), "{name} strict={strict}");
        clean(&runtime);
    }
}
macro_rules! independent {($($name:ident),* $(,)?)=>{$(#[test]fn $name(){independent_case(stringify!($name));})*};}
independent!(
    metadata,
    scalar_family_and_tree,
    substring_half_pairs_and_bounds,
    unsigned_long_modulo_and_defaults,
    brands_and_arity_precede_conversion,
    replace_conversion_order_and_fresh_data,
    callback_growth_and_shrink_change_bounds,
    append_uses_length_after_data_conversion,
    substring_and_delete_read_after_both_numbers,
    abrupt_identity_precedes_range_check,
    domstring_null_and_extra_arguments,
    complete_splice_repairs_surrogate_boundaries,
    pi_mutation_does_not_repeat_creation_validation
);

#[test]
fn character_data_bootstrap_admission() {
    let runtime = Runtime::uninitialized().finish_bootstrap().unwrap();
    println!(
        "CHARACTER_DATA_BOOTSTRAP steps={} allocated={} objects={} capacity={} native={} prototypes={}",
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

#[test]
fn tiny_substring_of_large_dom_data_only_visits_selected_prefix() {
    let (mut runtime, mut doc) = fresh();
    let old = format!("\u{1d11e}{}", "x".repeat(MAX_STRING + 1));
    let node = Value::Node(doc.create_text_node(&old));
    let before = runtime.steps;
    for (offset, expected) in [(0, 0xd834), (1, 0xdd1e), (2, 0x78)] {
        assert_eq!(
            runtime
                .character_data_native(
                    "substringData",
                    node.clone(),
                    &[Value::Number(offset as f64), Value::Number(1.0)],
                    &mut doc
                )
                .unwrap(),
            Value::String(vec![expected].into())
        );
    }
    assert!(before - runtime.steps < 1000);
    assert!(runtime.steps > 0);
    assert_eq!(
        doc.text_content(match node {
            Value::Node(id) => id,
            _ => unreachable!(),
        }),
        old
    );
    clean(&runtime);
}

fn setup_node(kind: &str) -> (Runtime, Document, Value) {
    let (runtime, mut doc) = fresh();
    let id = match kind {
        "Text" => doc.create_text_node("A\u{1d11e}B"),
        "Comment" => doc.create_comment("A\u{1d11e}B"),
        "PI" => doc.create_processing_instruction("probe", "A\u{1d11e}B"),
        _ => unreachable!(),
    };
    (runtime, doc, Value::Node(id))
}

#[test]
fn all_methods_measured_exact_and_one_short_admission() {
    let methods = [
        (
            "substringData",
            vec![Value::Number(1.0), Value::Number(1.0)],
        ),
        ("appendData", vec![Value::String("!".into())]),
        (
            "insertData",
            vec![Value::Number(1.0), Value::String("x".into())],
        ),
        ("deleteData", vec![Value::Number(1.0), Value::Number(2.0)]),
        (
            "replaceData",
            vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::String("x".into()),
            ],
        ),
    ];
    for kind in ["Text", "Comment", "PI"] {
        for (method, args) in &methods {
            let (mut runtime, mut doc, node) = setup_node(kind);
            let before = (runtime.steps, runtime.allocated);
            let expected = runtime
                .character_data_native(method, node, args, &mut doc)
                .unwrap();
            let expected_document = format!("{doc:?}");
            let work = before.0 - runtime.steps;
            let heap = runtime.allocated - before.1;
            assert!(work > 0 && heap > 0);
            for (steps, bytes, success) in [
                (work, heap, true),
                (work - 1, heap, false),
                (work, heap - 1, false),
            ] {
                let (mut runtime, mut doc, node) = setup_node(kind);
                let before = format!("{doc:?}");
                runtime.steps = steps;
                runtime.allocated = MAX_HEAP - bytes;
                let result = runtime.character_data_native(method, node, args, &mut doc);
                if success {
                    assert_eq!(result.unwrap(), expected);
                    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
                    assert_eq!(format!("{doc:?}"), expected_document);
                } else {
                    assert!(result.unwrap_err().is_resource_limit());
                    assert_eq!(format!("{doc:?}"), before);
                }
                clean(&runtime);
            }
        }
    }
}

#[test]
fn unchanged_nonscalar_policy_sources_produce_exact_standard_units() {
    // Original source bytes and the earlier Unsupported assertions/results are
    // retained in preparation evidence. These are normative outcome checks.
    let policies: [(&str, &str, &[u16]); 4] = [
        (
            "delete-leaves-low-unit",
            include_str!(
                "../../../../tests/fixtures/character-data-policy-delete-leaves-low-unit.js"
            ),
            &[65, 56606, 66],
        ),
        (
            "insert-between-pair-leaves-unpaired-units",
            include_str!(
                "../../../../tests/fixtures/character-data-policy-insert-between-pair-leaves-unpaired-units.js"
            ),
            &[65, 55348, 120, 56606, 66],
        ),
        (
            "append-lone-high-unit",
            include_str!(
                "../../../../tests/fixtures/character-data-policy-append-lone-high-unit.js"
            ),
            &[111, 108, 100, 55296],
        ),
        (
            "replace-leaves-lone-high-unit",
            include_str!(
                "../../../../tests/fixtures/character-data-policy-replace-leaves-lone-high-unit.js"
            ),
            &[65, 55348, 66],
        ),
    ];
    for (id, source, expected) in policies {
        for strict in [false, true] {
            let (mut runtime, mut doc) = fresh();
            let result = if strict {
                runtime.execute_strict(source, &mut doc)
            } else {
                runtime.execute(source, &mut doc)
            };
            assert!(result.is_ok(), "{id} strict={strict}: {result:?}");
            let node = runtime.environments[0].bindings["n"].value.clone();
            assert_eq!(
                runtime
                    .pi_native("data", node.clone(), &[], &mut doc)
                    .unwrap(),
                Value::String(expected.to_vec().into())
            );
            let Value::Node(node) = node else { panic!() };
            let data = current_data(&doc, node).unwrap();
            assert_eq!(data.raw_units(), Some(expected));
            clean(&runtime);
        }
    }
}

#[test]
fn method_mutation_preserves_callback_prefix_on_terminal_refusal() {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = "var n=new Text('old'),seen='',caught=false,finalized=false;try{n.appendData({toString:function(){seen='prefix';n.data='inner';while(true){}}});}catch(e){caught=true;}finally{finalized=true;}";
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        assert!(result.unwrap_err().is_resource_limit());
        assert_eq!(
            runtime.environments[0].bindings["seen"].value,
            Value::String("prefix".into())
        );
        assert_eq!(
            runtime.environments[0].bindings["caught"].value,
            Value::Bool(false)
        );
        assert_eq!(
            runtime.environments[0].bindings["finalized"].value,
            Value::Bool(false)
        );
        let Value::Node(id) = runtime.environments[0].bindings["n"].value else {
            panic!()
        };
        assert!(matches!(&doc.nodes[id].kind,NodeKind::Text(text) if text=="inner"));
        clean(&runtime);
    }
}

#[test]
fn mutation_rechecks_current_retained_bytes_after_data_conversion() {
    let (mut runtime, mut doc) = fresh();
    runtime.execute("var n=new Text('old'),seen=0,value={toString:function(){seen++;document.createTextNode('x');return '!';}};",&mut doc).unwrap();
    while doc.retained_bytes() < crate::dom::MAX_DOM_BYTES - 1 {
        let length = (crate::dom::MAX_DOM_BYTES - 1 - doc.retained_bytes()).min(8 * 1024 * 1024);
        let id = doc.create_text_node(&"x".repeat(length));
        assert_ne!(id, doc.root);
    }
    let count = doc.nodes.len();
    let node = runtime.environments[0].bindings["n"].value.clone();
    let value = runtime.environments[0].bindings["value"].value.clone();
    let error = runtime
        .character_data_native("appendData", node.clone(), &[value], &mut doc)
        .unwrap_err();
    assert!(error.is_resource_limit());
    assert_eq!(doc.nodes.len(), count + 1);
    assert_eq!(doc.retained_bytes(), crate::dom::MAX_DOM_BYTES);
    assert_eq!(
        runtime.environments[0].bindings["seen"].value,
        Value::Number(1.0)
    );
    assert_eq!(
        runtime.pi_native("data", node, &[], &mut doc).unwrap(),
        Value::String("old".into())
    );
    clean(&runtime);
}

#[test]
fn prototype_method_delete_does_not_resurrect_and_saved_brand_survives() {
    let source = r#"(function(){var n=new Text('ab'),p=CharacterData.prototype;var names=['substringData','appendData','insertData','deleteData','replaceData'];for(var i=0;i<names.length;i++){var key=names[i],d=Object.getOwnPropertyDescriptor(p,key);try{p[key]=17;if(n[key]!==17)throw new Error('replace');delete p[key];if(n[key]!==undefined)throw new Error('resurrect');var caught=false;try{d.value.call({},0,0,'');}catch(e){caught=e instanceof TypeError;}if(!caught)throw new Error('brand');}finally{Object.defineProperty(p,key,d);}}return n.substringData(0,2)==='ab';})()"#;
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        assert_eq!(result.unwrap(), Value::Bool(true));
        clean(&runtime);
    }
}

#[test]
fn tiny_substring_of_large_exact_units_visits_only_the_selected_prefix() {
    let (mut runtime, mut doc) = fresh();
    let mut data = vec![0x78; MAX_STRING + 1];
    data[0] = 0xd800;
    data[1] = 0xdc00;
    data[2] = 0xd800;
    let id = doc
        .create_text_node_owned(crate::dom::DomString::from_units_owned(data).unwrap())
        .unwrap();
    let before = runtime.steps;
    for (offset, expected) in [(0, 0xd800), (1, 0xdc00), (2, 0xd800), (3, 0x78)] {
        assert_eq!(
            runtime
                .character_data_native(
                    "substringData",
                    Value::Node(id),
                    &[Value::Number(offset as f64), Value::Number(1.0)],
                    &mut doc
                )
                .unwrap(),
            Value::String(vec![expected].into())
        );
    }
    assert!(before - runtime.steps < 1000);
    assert!(current_data(&doc, id).unwrap().raw_units().is_some());
    clean(&runtime);
}

#[test]
fn exact_splice_measured_boundaries_preserve_outer_data_and_payload_ledger() {
    for (initial, method, args, expected, scalar) in [
        (
            vec![0xd800],
            "appendData",
            vec![Value::String(vec![0xdc00].into())],
            vec![0xd800, 0xdc00],
            true,
        ),
        (
            vec![0xd800, 0x41, 0xdc00],
            "replaceData",
            vec![
                Value::Number(1.0),
                Value::Number(1.0),
                Value::String("B".into()),
            ],
            vec![0xd800, 0x42, 0xdc00],
            false,
        ),
        (
            vec![0xd800, 0x78, 0xdc00],
            "deleteData",
            vec![Value::Number(1.0), Value::Number(1.0)],
            vec![0xd800, 0xdc00],
            true,
        ),
    ] {
        for kind in ["Text", "Comment", "PI"] {
            let setup = || {
                let (runtime, mut doc) = fresh();
                let data = crate::dom::DomString::from_units_owned(initial.clone()).unwrap();
                let id = match kind {
                    "Text" => doc.create_text_node_owned(data),
                    "Comment" => doc.create_comment_owned(data),
                    _ => doc.create_processing_instruction_owned("p".into(), data),
                }
                .unwrap();
                (runtime, doc, id)
            };
            let (mut witness, mut document, id) = setup();
            let before = (witness.steps, witness.allocated, document.retained_bytes());
            witness
                .character_data_native(method, Value::Node(id), &args, &mut document)
                .unwrap();
            let value = current_data(&document, id).unwrap();
            assert_eq!(value.units().collect::<Vec<_>>(), expected);
            assert_eq!(value.scalar().is_some(), scalar);
            let bytes = if scalar { 4 } else { 6 };
            assert_eq!(value.stored_bytes(), bytes);
            assert_eq!(
                document.retained_bytes(),
                before.2 - initial.len() * 2 + bytes
            );
            let (work, heap) = (before.0 - witness.steps, witness.allocated - before.1);
            for (steps, available, success) in [
                (work, heap, true),
                (work - 1, heap, false),
                (work, heap - 1, false),
            ] {
                let (mut runtime, mut doc, id) = setup();
                let saved = format!("{doc:?}");
                runtime.steps = steps;
                runtime.allocated = MAX_HEAP - available;
                let result =
                    runtime.character_data_native(method, Value::Node(id), &args, &mut doc);
                if success {
                    assert_eq!(result.unwrap(), Value::Undefined);
                    assert_eq!(
                        current_data(&doc, id).unwrap().units().collect::<Vec<_>>(),
                        expected
                    );
                } else {
                    assert!(result.unwrap_err().is_resource_limit());
                    assert_eq!(format!("{doc:?}"), saved);
                }
                clean(&runtime);
            }
        }
    }
}
