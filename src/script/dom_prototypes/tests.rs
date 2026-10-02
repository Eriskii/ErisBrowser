use super::*;

#[test]
fn dom_prototypes_bootstrap_admission() {
    let mut runtime = Runtime::uninitialized();
    let result = runtime
        .reserve_bootstrap_objects()
        .and_then(|()| machine::initialize(&mut runtime))
        .and_then(|()| runtime.initialize_intrinsics());
    println!(
        "DOM_PROTOTYPE_BOOTSTRAP remaining={} allocated={} objects={} capacity={} native={} prototypes={} result={result:?}",
        runtime.steps,
        runtime.allocated,
        runtime.objects.len(),
        runtime.objects.capacity(),
        runtime.native_properties.len(),
        runtime.prototypes.len()
    );
    result.unwrap();
    assert_eq!(runtime.objects.len(), BOOTSTRAP_OBJECTS);
}

// The unchanged aggregate in tests/fixtures/dom-prototypes.js is retained as
// an external ordinary-success case. It currently reaches the work limit;
// independently bounded cases below do not reclassify that observation.

fn constructor_witness_clean(runtime: &Runtime) {
    assert_eq!(runtime.calls, 0);
    assert_eq!(runtime.stack_units, 0);
    assert_eq!(runtime.eval_depth, 0);
    assert!(runtime.frames.is_empty());
}

fn constructor_witness_target(
    runtime: &mut Runtime,
    doc: &mut Document,
    prototype: Value,
) -> Value {
    let target = runtime
        .execute("(function ConstructorWitness() {}).bind(null)", doc)
        .unwrap();
    assert!(runtime.is_constructor(target.clone()).unwrap());
    assert!(
        runtime
            .define_own(
                &target,
                &"prototype".into(),
                PropertyDescriptor::data_property(prototype, true, false, true),
            )
            .unwrap()
    );
    target
}

fn constructor_witness_id(value: Value) -> NodeId {
    let Value::Node(id) = value else {
        panic!("constructor did not return a genuine node: {value:?}");
    };
    id
}

fn constructor_witness_kind(doc: &Document, id: NodeId, name: &str, text: &str) {
    assert_eq!(doc.nodes[id].parent, None);
    assert!(doc.nodes[id].children.is_empty());
    match (&doc.nodes[id].kind, name) {
        (NodeKind::DocumentFragment { host: None }, "DocumentFragment") => {}
        (NodeKind::Text(value), "Text") | (NodeKind::Comment(value), "Comment") => {
            assert_eq!(value, text);
        }
        other => panic!("unexpected node kind: {other:?}"),
    }
}

#[test]
fn dom_constructor_default_and_primitive_fallback_keep_overrides_sparse() {
    for name in ["DocumentFragment", "Text", "Comment"] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut doc = Document::parse("<p>kept</p>");
        let default = Value::Object(runtime.dom_proto_id(name).unwrap());
        let constructor = runtime.environments[0].bindings[name].value.clone();
        let fallback = constructor_witness_target(&mut runtime, &mut doc, Value::Null);
        for target in [constructor, fallback] {
            let count = doc.nodes.len();
            let value = runtime
                .dom_interface_construct(&format!("{PREFIX}{name}"), &[], target, &mut doc)
                .unwrap();
            let id = constructor_witness_id(value.clone());
            assert_eq!(id, count);
            assert_eq!(doc.nodes.len(), count + 1);
            constructor_witness_kind(&doc, id, name, "");
            assert_eq!(
                runtime.prototype_of_in(&value, &doc).unwrap(),
                Some(default.clone())
            );
            assert!(runtime.dom_prototypes.overrides.is_empty());
            constructor_witness_clean(&runtime);
        }
    }
}

#[test]
fn dom_constructor_explicit_prototype_preserves_distinct_native_node_identity() {
    for name in ["DocumentFragment", "Text", "Comment"] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut doc = Document::parse("<p>kept</p>");
        let prototype = runtime.object_ordered([]).unwrap();
        let target = constructor_witness_target(&mut runtime, &mut doc, prototype.clone());
        let first = doc.nodes.len();
        for expected in [first, first + 1] {
            let value = runtime
                .dom_interface_construct(
                    &format!("{PREFIX}{name}"),
                    &[Value::String("A𝄞".into())],
                    target.clone(),
                    &mut doc,
                )
                .unwrap();
            let id = constructor_witness_id(value.clone());
            assert_eq!(id, expected);
            constructor_witness_kind(&doc, id, name, "A𝄞");
            assert_eq!(runtime.dom_prototypes.overrides.get(&id), Some(&prototype));
            assert_eq!(
                runtime.prototype_of_in(&value, &doc).unwrap(),
                Some(prototype.clone())
            );
        }
        assert_eq!(doc.nodes.len(), first + 2);
        assert_eq!(runtime.dom_prototypes.overrides.len(), 2);
        constructor_witness_clean(&runtime);
    }
}

fn constructor_witness_cutpoint_setup(grow: bool) -> (Runtime, Document, Value, Value) {
    let mut runtime = Runtime::try_new().unwrap();
    let mut doc = Document::parse("<p>kept</p>");
    let prototype = runtime.object_ordered([]).unwrap();
    let target = constructor_witness_target(&mut runtime, &mut doc, prototype.clone());
    // Test-owned capacity setup, outside the measured constructor span.
    doc.nodes = doc.nodes.into_boxed_slice().into_vec();
    assert_eq!(doc.nodes.len(), doc.nodes.capacity());
    if !grow {
        doc.nodes.try_reserve_exact(1).unwrap();
        assert!(doc.nodes.capacity() > doc.nodes.len());
    }
    runtime.steps = MAX_STEPS;
    (runtime, doc, target, prototype)
}

#[test]
fn dom_constructor_measured_exact_and_one_short_admission_has_no_partial_node() {
    for grow in [false, true] {
        let arguments = [Value::String("A𝄞".into())];
        let (mut measure, mut doc, target, _) = constructor_witness_cutpoint_setup(grow);
        let before_heap = measure.allocated;
        measure
            .dom_interface_construct("DOM.Interface.Text", &arguments, target, &mut doc)
            .unwrap();
        let work = MAX_STEPS - measure.steps;
        let heap = measure.allocated - before_heap;
        assert!(work > 0 && heap > 0);
        // Measure real debits; do not duplicate the production fee formula.
        for (steps, available, succeeds) in [
            (work, heap, true),
            (work - 1, heap, false),
            (work, heap - 1, false),
        ] {
            let (mut runtime, mut doc, target, prototype) =
                constructor_witness_cutpoint_setup(grow);
            let before = format!("{doc:?}");
            let count = doc.nodes.len();
            let capacity = doc.nodes.capacity();
            runtime.steps = steps;
            runtime.allocated = MAX_HEAP - available;
            let result =
                runtime.dom_interface_construct("DOM.Interface.Text", &arguments, target, &mut doc);
            if succeeds {
                assert_eq!(constructor_witness_id(result.unwrap()), count);
                assert_eq!(doc.nodes.len(), count + 1);
                constructor_witness_kind(&doc, count, "Text", "A𝄞");
                assert_eq!(
                    runtime.dom_prototypes.overrides.get(&count),
                    Some(&prototype)
                );
                assert_eq!(runtime.dom_prototypes.overrides.len(), 1);
                assert_eq!(runtime.steps, 0);
                assert_eq!(runtime.allocated, MAX_HEAP);
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert_eq!(format!("{doc:?}"), before);
                assert_eq!(doc.nodes.capacity(), capacity);
                assert!(runtime.dom_prototypes.overrides.is_empty());
            }
            constructor_witness_clean(&runtime);
        }
    }
}

#[test]
fn dom_constructor_accepts_surrogate_pairs_and_explicitly_refuses_lone_units() {
    for name in ["Text", "Comment"] {
        for units in [vec![0xd800], vec![0xdc00], vec![0x61, 0xd800]] {
            let mut runtime = Runtime::try_new().unwrap();
            let mut doc = Document::parse("<p>kept</p>");
            let target = runtime.environments[0].bindings[name].value.clone();
            let qualified = format!("{PREFIX}{name}");
            let valid = runtime
                .dom_interface_construct(
                    &qualified,
                    &[Value::String(vec![0xd834, 0xdd1e].into())],
                    target.clone(),
                    &mut doc,
                )
                .unwrap();
            constructor_witness_kind(&doc, constructor_witness_id(valid), name, "𝄞");
            let before = format!("{doc:?}");
            let error = runtime
                .dom_interface_construct(
                    &qualified,
                    &[Value::String(units.into())],
                    target,
                    &mut doc,
                )
                .unwrap_err();
            assert!(error.is_unsupported());
            assert_eq!(format!("{doc:?}"), before);
            assert!(runtime.dom_prototypes.overrides.is_empty());
            constructor_witness_clean(&runtime);
        }
    }
}

#[test]
fn dom_prototype_invalid_node_cannot_be_rescued_by_cached_override() {
    let mut runtime = Runtime::try_new().unwrap();
    let doc = Document::parse("<p>kept</p>");
    let prototype = Value::Object(runtime.dom_proto_id("Text").unwrap());
    for id in [doc.nodes.len(), usize::MAX] {
        for cached in [false, true] {
            if cached {
                // A deliberately stale private entry; JavaScript cannot forge NodeIds.
                runtime
                    .dom_prototypes
                    .overrides
                    .insert(id, prototype.clone());
            }
            let error = runtime.prototype_of_in(&Value::Node(id), &doc).unwrap_err();
            assert_eq!(error.name(), "TypeError");
            assert_eq!(runtime.dom_prototypes.overrides.contains_key(&id), cached);
        }
    }
    constructor_witness_clean(&runtime);
}

#[test]
fn dom_prototypes_independent_interface_mapping_in_both_modes() {
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut doc = Document::parse("<html><head></head><body></body></html>");
        let source = include_str!("../../../tests/fixtures/dom-interface-mapping.js");
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        assert_eq!(result.unwrap(), Value::Bool(true));
        constructor_witness_clean(&runtime);
    }
}

#[test]
fn dom_constructor_rechecks_node_and_text_admission_after_prototype_getter() {
    for exhaust_text in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut doc = Document::parse("");
        runtime
            .execute(
                r#"
            var witnessCalls = 0, witnessNode;
            var witnessTarget = (function () {}).bind(null);
            Object.defineProperty(witnessTarget, 'prototype', {
                get: function () {
                    witnessCalls++;
                    witnessNode = new Text('x');
                    return Object.create(Text.prototype);
                }
            });
        "#,
                &mut doc,
            )
            .unwrap();
        let target = runtime.environments[0].bindings["witnessTarget"]
            .value
            .clone();
        // Populate existing host state outside the measured author operation.
        if exhaust_text {
            let block = "z".repeat(8 * 1024 * 1024);
            while doc.retained_bytes() < crate::dom::MAX_DOM_BYTES - 1 {
                let bytes = (crate::dom::MAX_DOM_BYTES - 1 - doc.retained_bytes()).min(block.len());
                doc.create_text_node(&block[..bytes]);
            }
        } else {
            while doc.nodes.len() < crate::dom::MAX_NODES - 1 {
                doc.create_comment("");
            }
        }
        let count = doc.nodes.len();
        let retained = doc.retained_bytes();
        let error = runtime
            .dom_interface_construct(
                "DOM.Interface.Text",
                &[Value::String("outer".into())],
                target,
                &mut doc,
            )
            .unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(doc.nodes.len(), count + 1);
        assert_eq!(doc.retained_bytes(), retained + 1);
        assert_eq!(
            runtime.environments[0].bindings["witnessCalls"].value,
            Value::Number(1.0)
        );
        assert_eq!(
            runtime.environments[0].bindings["witnessNode"].value,
            Value::Node(count)
        );
        constructor_witness_kind(&doc, count, "Text", "x");
        assert!(runtime.dom_prototypes.overrides.is_empty());
        constructor_witness_clean(&runtime);
    }
}

#[test]
fn dom_prototype_metadata_order_and_complete_scoped_unscopables() {
    let mut runtime = Runtime::try_new().unwrap();
    let mut doc = Document::parse("");
    assert_eq!(runtime.execute(r#"
        (function () {
            var nodeKeys = Object.getOwnPropertyNames(Node.prototype);
            if (nodeKeys.indexOf('ELEMENT_NODE') >= nodeKeys.indexOf('constructor') ||
                nodeKeys.indexOf('DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC') >=
                  nodeKeys.indexOf('constructor')) throw new Error('constant creation order');
            var parent = 'append,prepend,replaceChildren';
            var child = 'after,before,remove,replaceWith';
            var expected = [parent, parent, 'after,append,before,prepend,remove,replaceChildren,replaceWith', child, child];
            var prototypes = [Document.prototype, DocumentFragment.prototype, Element.prototype,
                              CharacterData.prototype, DocumentType.prototype];
            for (var i = 0; i < prototypes.length; i++) {
                var bag = prototypes[i][Symbol.unscopables];
                if (Object.getPrototypeOf(bag) !== null || Object.keys(bag).sort().join(',') !== expected[i])
                    throw new Error('scoped unscopables inventory');
                if (bag.moveBefore !== undefined) throw new Error('moveBefore is not unscopable');
            }
            return true;
        })()
    "#, &mut doc).unwrap(), Value::Bool(true));
    constructor_witness_clean(&runtime);
}

#[test]
fn dom_interface_direct_metadata_keeps_identity_and_ordinary_mutation() {
    let mut runtime = Runtime::try_new().unwrap();
    let mut doc = Document::parse("");
    let text = runtime.environments[0].bindings["Text"].value.clone();
    let Value::Native(native) = &text else {
        panic!()
    };
    let properties = native.properties.unwrap().get();
    assert_eq!(runtime.property_object(&text), Some(properties));
    assert!(!runtime.native_properties.contains_key(&native.name));
    // The auxiliary bag handle is outside the existing callable identity key.
    let mut without_cache = (**native).clone();
    without_cache.properties = None;
    assert_eq!(**native, without_cache);
    assert!(
        runtime
            .object_is_values(&text, &Value::Native(Rc::new(without_cache)))
            .unwrap()
    );
    assert_eq!(runtime.execute(r#"
        (function () {
            var Original = Text, marker = {}, calls = 0;
            Object.defineProperty(Original, 'extra', {get: function () {
                if (this !== Original) throw new Error('native accessor receiver');
                calls++; return marker;
            }, configurable: true});
            if (Original.extra !== marker || calls !== 1) throw new Error('native accessor');
            if (!delete Original.extra || Object.getOwnPropertyDescriptor(Original, 'extra') !== undefined)
                throw new Error('ordinary delete');
            Text = 0;
            try {
                var node = new Original('kept');
                if (Object.getPrototypeOf(node) !== Original.prototype || node.constructor !== Original)
                    throw new Error('intrinsic prototype depends on global shadow');
            } finally { Text = Original; }
            return true;
        })()
    "#, &mut doc).unwrap(), Value::Bool(true));
    constructor_witness_clean(&runtime);
}

#[test]
fn dom_interface_cached_lookup_cost_is_independent_of_legacy_registry_size() {
    let mut runtime = Runtime::try_new().unwrap();
    let doc = Document::parse("");
    let cached = runtime.environments[0].bindings["Text"].value.clone();
    let legacy = runtime.environments[0].bindings["Number"].value.clone();
    let key = JsString::from("length");
    let mut costs = Vec::new();
    for grow in [false, true] {
        if grow {
            for i in 0..1024 {
                runtime
                    .native_properties
                    .insert(format!("private-cost-{i}"), 0);
            }
        }
        let mut row = Vec::new();
        for receiver in [&cached, &legacy] {
            runtime.steps = MAX_STEPS;
            assert!(
                runtime
                    .reduce_property_in(receiver, &key, &doc)
                    .unwrap()
                    .is_some()
            );
            row.push(MAX_STEPS - runtime.steps);
        }
        costs.push(row);
    }
    assert_eq!(costs[0][0], costs[1][0]);
    assert!(costs[1][1] > costs[0][1]);
    constructor_witness_clean(&runtime);
}

#[test]
fn dom_native_metadata_slot_allocation_is_admitted_before_factory_return() {
    let mut measure = Runtime::uninitialized();
    let before = measure.allocated;
    let value = measure.alloc_native("private", Value::Undefined).unwrap();
    let bytes = measure.allocated - before;
    assert!(bytes > 0);
    let Value::Native(native) = value else {
        panic!()
    };
    assert!(native.properties.is_none());
    for available in [bytes - 1, bytes] {
        let mut runtime = Runtime::uninitialized();
        runtime.allocated = MAX_HEAP - available;
        let result = runtime.alloc_native("private", Value::Undefined);
        assert_eq!(result.is_ok(), available == bytes);
        if let Err(error) = result {
            assert!(error.is_resource_limit());
        }
        assert!(runtime.objects.is_empty());
        assert!(runtime.native_properties.is_empty());
        assert_eq!(runtime.steps, MAX_STEPS);
    }
}

#[test]
fn dom_metadata_sorted_merge_rejects_duplicates_and_preserves_values() {
    for count in [0, 1, 11, 12, 143, 144] {
        let old: BTreeMap<_, _> = (0..count)
            .map(|i| (format!("key-{:04}", 2 * i), i))
            .collect();
        let new: Vec<_> = (0..count)
            .map(|i| (format!("key-{:04}", 2 * i + 1), 1000 + i))
            .collect();
        validate_staged(&new).unwrap();
        let output = merged_map(old, new, Vec::with_capacity(2 * count)).unwrap();
        assert_eq!(output.len(), 2 * count);
        for i in 0..count {
            assert_eq!(output[&format!("key-{:04}", 2 * i)], i);
            assert_eq!(output[&format!("key-{:04}", 2 * i + 1)], 1000 + i);
        }
    }
    assert!(
        validate_staged(&[("b", 1), ("a", 2)])
            .unwrap_err()
            .is_resource_limit()
    );
    assert!(
        validate_staged(&[("a", 1), ("a", 2)])
            .unwrap_err()
            .is_resource_limit()
    );
    let error = merged_map(
        BTreeMap::from([("a", 1)]),
        vec![("a", 2)],
        Vec::with_capacity(2),
    )
    .unwrap_err();
    assert!(error.is_resource_limit());
}

#[test]
fn dom_constructor_conversion_precedes_prototype_get_and_plain_call_refuses() {
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut doc = Document::parse("");
        let source = r#"
            (function () {
                var trace = '', prototype = Object.create(Text.prototype);
                var target = (function () {}).bind(null);
                Object.defineProperty(target, 'prototype', {get: function () {
                    trace += 'P'; return prototype;
                }});
                var argument = {toString: function () { trace += 'C'; return 'kept'; }};
                var text = Reflect.construct(Text, [argument], target);
                if (trace !== 'CP' || Object.getPrototypeOf(text) !== prototype || text.textContent !== 'kept')
                    throw new Error('constructor operation order');
                trace = '';
                var error;
                try { Text(argument); } catch (caught) { error = caught; }
                if (!(error instanceof TypeError) || trace !== '') throw new Error('plain call converted');
                var illegal = [Node, Element, CharacterData];
                for (var i = 0; i < illegal.length; i++) {
                    error = undefined;
                    try { new illegal[i](argument); } catch (caught) { error = caught; }
                    if (!(error instanceof TypeError) || trace !== '') throw new Error('illegal constructor converted');
                }
                return true;
            })()
        "#;
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        assert_eq!(result.unwrap(), Value::Bool(true));
        constructor_witness_clean(&runtime);
    }
}

#[test]
fn dom_unimplemented_real_constructors_remain_explicit_and_do_not_publish() {
    let name = "Document";
    let mut runtime = Runtime::try_new().unwrap();
    let mut doc = Document::parse("<p>kept</p>");
    let target = runtime.environments[0].bindings[name].value.clone();
    let before = format!("{doc:?}");
    let error = runtime
        .dom_interface_construct(&format!("{PREFIX}{name}"), &[], target, &mut doc)
        .unwrap_err();
    assert!(error.is_unsupported());
    assert_eq!(format!("{doc:?}"), before);
    assert!(runtime.dom_prototypes.overrides.is_empty());
    constructor_witness_clean(&runtime);
}

fn run_independent_category(name: &str, source: &str) {
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut doc = Document::parse("<!doctype html><html><head></head><body></body></html>");
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        println!(
            "DOM_CATEGORY name={name} strict={strict} remaining={} result={result:?}",
            runtime.steps
        );
        assert_eq!(result.unwrap(), Value::Bool(true));
        constructor_witness_clean(&runtime);
    }
}

#[test]
fn dom_prototypes_category_core_interface_descriptors() {
    run_independent_category(
        "core-interface-descriptors",
        include_str!("../../../tests/fixtures/dom-prototypes-core-interface-descriptors.js"),
    );
}

#[test]
fn dom_prototypes_category_core_node_constants() {
    run_independent_category(
        "core-node-constants",
        include_str!("../../../tests/fixtures/dom-prototypes-core-node-constants.js"),
    );
}

#[test]
fn dom_prototypes_category_parent_method_identities() {
    run_independent_category(
        "parent-method-identities",
        include_str!("../../../tests/fixtures/dom-prototypes-parent-method-identities.js"),
    );
}

#[test]
fn dom_prototypes_category_document_parent_methods() {
    run_independent_category(
        "document-parent-methods",
        include_str!("../../../tests/fixtures/dom-prototypes-document-parent-methods.js"),
    );
}

#[test]
fn dom_prototypes_category_element_parent_methods() {
    run_independent_category(
        "element-parent-methods",
        include_str!("../../../tests/fixtures/dom-prototypes-element-parent-methods.js"),
    );
}

#[test]
fn dom_prototypes_category_fragment_parent_methods() {
    run_independent_category(
        "fragment-parent-methods",
        include_str!("../../../tests/fixtures/dom-prototypes-fragment-parent-methods.js"),
    );
}
