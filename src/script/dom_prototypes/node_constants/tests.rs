use super::*;

const CASES: &str = include_str!("../../../../tests/fixtures/node-constants.js");

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
        let mut runtime = Runtime::try_new().unwrap();
        let mut doc = Document::parse("");
        let source = format!("{CASES}\nnodeConstantCases.{name}();");
        assert_eq!(
            evaluate(&mut runtime, &mut doc, &source, strict),
            Value::Bool(true),
            "{name}, strict={strict}"
        );
    }
}

macro_rules! cases {($($name:ident),* $(,)?) => {$(#[test] fn $name() {independent(stringify!($name));})*};}
cases!(
    constructor_constant_descriptors,
    prototype_constant_descriptors,
    constructor_complete_key_order,
    saved_nonconstant_metadata_and_accessors,
    separate_mutable_constructor_and_prototype_bags,
    realm_markers_are_initially_absent,
);

#[test]
fn prior_node_constant_inventory_after_removing_later_configurable_operations() {
    // This historical fixture describes the preceding represented inventory.
    // Deletion/restoration intentionally changes creation order, so each mode
    // uses a disposable realm; the new root fixture checks pristine order.
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut doc = Document::parse("");
        let source = format!(
            r#"{CASES}
          (function(){{
            var names=['getRootNode','hasChildNodes','normalize','isSameNode','contains'],saved=[];
            for(var i=0;i<names.length;i++){{
              var d=Object.getOwnPropertyDescriptor(Node.prototype,names[i]);
              if(!d||!d.configurable||typeof d.value!=='function')throw new Error('later operation descriptor');
              saved.push(d);
            }}
            try{{
              for(var i=0;i<names.length;i++)if(!delete Node.prototype[names[i]])throw new Error('later operation delete');
              if(nodeConstantCases.prototype_complete_key_order()!==true)throw new Error('prior inventory');
            }}finally{{
              for(var i=0;i<names.length;i++)Object.defineProperty(Node.prototype,names[i],saved[i]);
            }}
            for(var i=0;i<names.length;i++){{
              var restored=Object.getOwnPropertyDescriptor(Node.prototype,names[i]),d=saved[i];
              if(restored.value!==d.value||restored.writable!==d.writable||
                 restored.enumerable!==d.enumerable||restored.configurable!==d.configurable)
                throw new Error('restore descriptor');
            }}
            return true;
          }})()
        "#
        );
        assert_eq!(
            evaluate(&mut runtime, &mut doc, &source, strict),
            Value::Bool(true)
        );
    }
}

#[test]
fn static_permutation_has_the_literal_strict_key_order() {
    assert_eq!(
        SORTED.map(|index| NODE_CONSTANTS[index].0),
        [
            "ATTRIBUTE_NODE",
            "CDATA_SECTION_NODE",
            "COMMENT_NODE",
            "DOCUMENT_FRAGMENT_NODE",
            "DOCUMENT_NODE",
            "DOCUMENT_POSITION_CONTAINED_BY",
            "DOCUMENT_POSITION_CONTAINS",
            "DOCUMENT_POSITION_DISCONNECTED",
            "DOCUMENT_POSITION_FOLLOWING",
            "DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC",
            "DOCUMENT_POSITION_PRECEDING",
            "DOCUMENT_TYPE_NODE",
            "ELEMENT_NODE",
            "ENTITY_NODE",
            "ENTITY_REFERENCE_NODE",
            "NOTATION_NODE",
            "PROCESSING_INSTRUCTION_NODE",
            "TEXT_NODE",
        ]
    );
}

#[test]
fn two_live_realms_retain_the_independent_interleaved_markers() {
    for strict in [false, true] {
        let mut a = Runtime::try_new().unwrap();
        let mut b = Runtime::try_new().unwrap();
        let mut a_doc = Document::parse("");
        let mut b_doc = Document::parse("");
        evaluate(&mut a, &mut a_doc, CASES, strict);
        evaluate(&mut b, &mut b_doc, CASES, strict);
        for (first, source) in [
            (true, "nodeConstantRealmSteps.seed_a()"),
            (
                false,
                "nodeConstantCases.realm_markers_are_initially_absent()",
            ),
            (false, "nodeConstantRealmSteps.seed_b()"),
            (true, "nodeConstantRealmSteps.verify_a()"),
            (false, "nodeConstantRealmSteps.verify_b()"),
        ] {
            let (runtime, doc) = if first {
                (&mut a, &mut a_doc)
            } else {
                (&mut b, &mut b_doc)
            };
            assert_eq!(
                evaluate(runtime, doc, source, strict),
                Value::Bool(true),
                "{source}, strict={strict}"
            );
        }
    }
}

// Recover authentic preconstant bags from a test-owned complete realm. No
// production refund/reset is introduced; normalize only test order capacity.
fn setup() -> (Runtime, usize, usize, PropertyKey) {
    let mut runtime = Runtime::try_new().unwrap();
    let prototype = runtime.dom_proto_id("Node").unwrap();
    let constructor = runtime.environments[0].bindings["Node"].value.clone();
    let properties = runtime.property_object(&constructor).unwrap();
    let tag = runtime.well_known_key("toStringTag");
    for owner in [prototype, properties] {
        let bag = &mut runtime.objects[owner];
        for (name, _) in NODE_CONSTANTS {
            bag.values.remove(&PropertyKey::from(*name));
        }
        if owner == prototype {
            bag.values.remove(&PropertyKey::from("constructor"));
        }
        let values = &bag.values;
        bag.order.retain(|key| values.contains_key(key));
        bag.order.shrink_to_fit();
        let capacity = if owner == prototype { 27 } else { 21 };
        bag.order
            .try_reserve_exact(capacity - bag.order.len())
            .unwrap();
        let old = if owner == prototype { 8 } else { 3 };
        assert_eq!(bag.values.len(), old);
        assert_eq!(bag.order.len(), old);
    }
    (runtime, prototype, properties, tag)
}

fn other_fields(bag: &ScriptObject) -> String {
    let abort = bag.abort.map(|slot| match slot {
        AbortSlot::Controller(id) => (true, id),
        AbortSlot::Signal(id) => (false, id),
    });
    format!(
        "{:?}",
        (
            (
                &bag.prototype,
                &bag.boxed,
                bag.non_extensible,
                bag.intrinsic_error,
                &bag.parameter_map,
                bag.arguments
            ),
            (
                bag.regexp.as_ref().map(Rc::as_ptr),
                bag.date_value,
                bag.event,
                bag.event_target,
                abort,
                bag.namespace
            ),
        )
    )
}

#[derive(Debug, PartialEq)]
struct Snapshot {
    values: String,
    order: Vec<PropertyKey>,
    capacity: usize,
    other: String,
}
fn snapshot(runtime: &Runtime, owner: usize) -> Snapshot {
    let bag = &runtime.objects[owner];
    Snapshot {
        values: format!("{:?}", bag.values),
        order: bag.order.clone(),
        capacity: bag.order.capacity(),
        other: other_fields(bag),
    }
}

fn legacy_constants(runtime: &mut Runtime, prototype: usize, properties: usize) {
    // Counterfactual uses the unchanged general insertion helper, not a copy
    // of its fee formula. The pre-optimization production source is retained.
    for &(key, value) in NODE_CONSTANTS {
        let key: PropertyKey = runtime.dom_proto_text(key).unwrap().into();
        for owner in [prototype, properties] {
            runtime
                .dom_proto_property(
                    owner,
                    key.clone(),
                    Property::data(Value::Number(value as f64), false, true, false),
                )
                .unwrap();
        }
    }
}

#[test]
fn batch_matches_general_insertion_and_saves_measured_work_and_storage() {
    let (mut batch, prototype, properties, tag) = setup();
    let (before_work, before_heap) = (batch.steps, batch.allocated);
    batch
        .install_node_constants(prototype, properties, &tag)
        .unwrap();
    let cost = (before_work - batch.steps, batch.allocated - before_heap);
    let (mut legacy, old_prototype, old_properties, _) = setup();
    let (before_work, before_heap) = (legacy.steps, legacy.allocated);
    legacy_constants(&mut legacy, old_prototype, old_properties);
    let old_cost = (before_work - legacy.steps, legacy.allocated - before_heap);
    assert!(cost.0 < old_cost.0);
    assert!(cost.1 < old_cost.1);
    assert_eq!(
        snapshot(&batch, prototype),
        snapshot(&legacy, old_prototype)
    );
    assert_eq!(
        snapshot(&batch, properties),
        snapshot(&legacy, old_properties)
    );
    println!(
        "NODE_CONSTANT_BATCH work={} heap={} old_work={} old_heap={}",
        cost.0, cost.1, old_cost.0, old_cost.1
    );
    clean(&batch);
    clean(&legacy);
}

#[test]
fn measured_exact_and_one_short_admission_precedes_taking_either_map() {
    let (mut measured, prototype, properties, tag) = setup();
    let (steps, heap) = (measured.steps, measured.allocated);
    measured
        .install_node_constants(prototype, properties, &tag)
        .unwrap();
    let work = steps - measured.steps;
    let storage = measured.allocated - heap;
    for (work_short, heap_short) in [(false, false), (true, false), (false, true)] {
        let (mut runtime, prototype, properties, tag) = setup();
        let before = [
            snapshot(&runtime, prototype),
            snapshot(&runtime, properties),
        ];
        let objects = runtime.objects.len();
        runtime.steps = work - usize::from(work_short);
        runtime.allocated = MAX_HEAP - storage + usize::from(heap_short);
        let result = runtime.install_node_constants(prototype, properties, &tag);
        if work_short || heap_short {
            assert!(result.unwrap_err().is_resource_limit());
            assert_eq!(
                [
                    snapshot(&runtime, prototype),
                    snapshot(&runtime, properties)
                ],
                before
            );
            if work_short {
                assert_eq!(runtime.steps, 0);
            }
            if heap_short {
                assert!(runtime.allocated > MAX_HEAP);
            }
        } else {
            result.unwrap();
            assert_eq!(runtime.steps, 0);
            assert_eq!(runtime.allocated, MAX_HEAP);
            assert_eq!(runtime.objects[prototype].values.len(), 26);
            assert_eq!(runtime.objects[properties].values.len(), 21);
            assert_eq!(
                runtime.objects[prototype].order.capacity(),
                before[0].capacity
            );
            assert_eq!(
                runtime.objects[properties].order.capacity(),
                before[1].capacity
            );
        }
        assert_eq!(runtime.objects.len(), objects);
        clean(&runtime);
    }
}

#[test]
fn batch_keeps_saved_native_handles_and_all_other_object_fields() {
    let (mut runtime, prototype, properties, tag) = setup();
    for owner in [prototype, properties] {
        let bag = &mut runtime.objects[owner];
        bag.boxed = Some(Value::Number(owner as f64));
        bag.non_extensible = true;
        bag.intrinsic_error = Some("TypeError");
        bag.parameter_map
            .insert("retained".into(), (0, "binding".into()));
        bag.arguments = true;
        bag.date_value = Some(123.5);
        bag.event = Some(7);
        bag.event_target = true;
        bag.abort = Some(AbortSlot::Signal(8));
        bag.namespace = Some("retained");
    }
    let before = [
        snapshot(&runtime, prototype),
        snapshot(&runtime, properties),
    ];
    let mut handles = Vec::new();
    for name in ["nodeValue", "textContent"] {
        let property = &runtime.objects[prototype].values[&PropertyKey::from(name)];
        let PropertyValue::Accessor {
            get: Value::Native(get),
            set: Value::Native(set),
        } = &property.value
        else {
            panic!("native accessor missing")
        };
        handles.push((name, get.clone(), set.clone()));
    }
    let methods: Vec<_> = [
        "getRootNode",
        "hasChildNodes",
        "normalize",
        "isSameNode",
        "contains",
    ]
    .into_iter()
    .map(|name| {
        let PropertyValue::Data {
            value: Value::Native(method),
            ..
        } = &runtime.objects[prototype].values[&PropertyKey::from(name)].value
        else {
            panic!("Node method Native missing")
        };
        (name, method.clone())
    })
    .collect();
    runtime
        .install_node_constants(prototype, properties, &tag)
        .unwrap();
    for (name, method) in methods {
        let PropertyValue::Data {
            value: Value::Native(after),
            ..
        } = &runtime.objects[prototype].values[&PropertyKey::from(name)].value
        else {
            panic!("Node method Native missing after batch")
        };
        assert!(Rc::ptr_eq(&method, after), "{name}");
    }
    for (owner, old) in [prototype, properties].into_iter().zip(before) {
        assert_eq!(other_fields(&runtime.objects[owner]), old.other);
        assert_eq!(&runtime.objects[owner].order[..old.order.len()], &old.order);
        assert_eq!(runtime.objects[owner].order.capacity(), old.capacity);
    }
    for (name, old_get, old_set) in handles {
        let property = &runtime.objects[prototype].values[&PropertyKey::from(name)];
        let PropertyValue::Accessor {
            get: Value::Native(get),
            set: Value::Native(set),
        } = &property.value
        else {
            panic!("native accessor missing")
        };
        assert!(Rc::ptr_eq(get, &old_get));
        assert!(Rc::ptr_eq(set, &old_set));
    }
}

#[test]
fn invalid_owner_shapes_refuse_before_allocating_or_taking_maps() {
    for case in 0..20 {
        let (mut runtime, prototype, properties, tag) = setup();
        let (mut p, mut c, mut key) = (prototype, properties, tag);
        match case {
            0 => p = usize::MAX,
            1 => c = usize::MAX,
            2 => c = p,
            3 => std::mem::swap(&mut p, &mut c),
            4 | 8 => {
                let bag = &mut runtime.objects[prototype];
                let old = bag.values.remove(&PropertyKey::from("nodeValue")).unwrap();
                let name = if case == 4 {
                    "wrong".to_owned()
                } else {
                    "x".repeat(1024)
                };
                bag.values.insert(name.into(), old);
            }
            5 => runtime.objects[prototype].order.swap(0, 1),
            6 => {
                runtime.objects[properties]
                    .values
                    .remove(&PropertyKey::from("name"));
            }
            7 => runtime.objects[properties].order.shrink_to_fit(),
            9 => key = PropertyKey::from("toStringTag"),
            10 => {
                runtime.objects[prototype]
                    .values
                    .remove(&PropertyKey::from("normalize"));
            }
            11 => {
                runtime.objects[properties].values.insert(
                    "normalize".into(),
                    Property::data(Value::Undefined, true, true, true),
                );
                runtime.objects[properties].order.push("normalize".into());
            }
            12 => runtime.objects[prototype].order.swap(2, 3),
            13..=15 => {
                runtime.objects[prototype].values.remove(&PropertyKey::from(
                    ["hasChildNodes", "isSameNode", "contains"][case - 13],
                ));
            }
            16 => runtime.objects[prototype].order.shrink_to_fit(),
            17 => runtime.objects[prototype].order.swap(3, 4),
            18 => {
                let bag = &mut runtime.objects[properties];
                let old = bag.values.remove(&PropertyKey::from("name")).unwrap();
                bag.values.insert("isSameNode".into(), old);
            }
            19 => {
                runtime.objects[prototype]
                    .values
                    .remove(&PropertyKey::from("getRootNode"));
            }
            _ => unreachable!(),
        }
        let before = [
            snapshot(&runtime, prototype),
            snapshot(&runtime, properties),
        ];
        let heap = runtime.allocated;
        assert!(
            runtime
                .install_node_constants(p, c, &key)
                .unwrap_err()
                .is_resource_limit(),
            "case {case}"
        );
        assert_eq!(
            [
                snapshot(&runtime, prototype),
                snapshot(&runtime, properties)
            ],
            before,
            "case {case}"
        );
        assert_eq!(runtime.allocated, heap);
        clean(&runtime);
    }
}

#[test]
fn invalid_permutation_length_and_indices_refuse_before_consumption() {
    for short in [true, false] {
        let (mut runtime, prototype, properties, tag) = setup();
        let mut sorted = SORTED.to_vec();
        if short {
            sorted.pop();
        } else {
            sorted[4] = usize::MAX;
        }
        let before = [
            snapshot(&runtime, prototype),
            snapshot(&runtime, properties),
        ];
        let heap = runtime.allocated;
        assert!(
            runtime
                .node_constants_with_order(prototype, properties, &tag, &sorted)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(
            [
                snapshot(&runtime, prototype),
                snapshot(&runtime, properties)
            ],
            before
        );
        assert_eq!(runtime.allocated, heap);
    }
}

#[test]
fn duplicate_and_unsorted_staging_discard_the_consumed_initializer() {
    for duplicate in [false, true] {
        let (mut runtime, prototype, properties, tag) = setup();
        let mut sorted = SORTED;
        if duplicate {
            sorted[1] = sorted[0];
        } else {
            sorted.swap(0, 1);
        }
        let before = [
            snapshot(&runtime, prototype),
            snapshot(&runtime, properties),
        ];
        let (work, heap) = (runtime.steps, runtime.allocated);
        assert!(
            runtime
                .node_constants_with_order(prototype, properties, &tag, &sorted)
                .unwrap_err()
                .is_resource_limit()
        );
        assert!(runtime.objects[prototype].values.is_empty());
        assert!(runtime.objects[properties].values.is_empty());
        for (owner, old) in [prototype, properties].into_iter().zip(before) {
            assert_eq!(runtime.objects[owner].order, old.order);
            assert_eq!(runtime.objects[owner].order.capacity(), old.capacity);
            assert_eq!(other_fields(&runtime.objects[owner]), old.other);
        }
        assert!(runtime.steps < work);
        assert!(runtime.allocated > heap);
        clean(&runtime);
        // Do not execute author code or retry this deliberately invalid realm.
    }
}

#[test]
fn node_constant_batch_bootstrap_reports_actual_remaining_budget() {
    let runtime = Runtime::uninitialized().finish_bootstrap().unwrap();
    println!(
        "NODE_CONSTANT_BOOTSTRAP steps={} allocated={} objects={} capacity={} native={} prototypes={}",
        runtime.steps,
        runtime.allocated,
        runtime.objects.len(),
        runtime.objects.capacity(),
        runtime.native_properties.len(),
        runtime.prototypes.len()
    );
    assert_eq!(runtime.objects.len(), BOOTSTRAP_OBJECTS);
    clean(&runtime);
}
