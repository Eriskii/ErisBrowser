use super::*;

const METHODS: [&str; 4] = [
    "Object.seal",
    "Object.freeze",
    "Object.isSealed",
    "Object.isFrozen",
];

fn operation(name: &str) -> IntegrityOperation {
    IntegrityOperation::from_name(name).unwrap()
}

fn prepared() -> (Runtime, Document) {
    (
        Runtime::new(),
        Document::parse("<p id='kept'>unchanged</p>"),
    )
}

fn arenas(runtime: &Runtime) -> (usize, usize, usize, usize, usize, usize) {
    (
        runtime.objects.len(),
        runtime.arrays.len(),
        runtime.functions.len(),
        runtime.environments.len(),
        runtime.native_properties.len(),
        runtime.host_symbol_objects.len(),
    )
}

fn clean(runtime: &Runtime) {
    assert_eq!(runtime.calls, 0);
    assert_eq!(runtime.stack_units, 0);
    assert_eq!(runtime.eval_depth, 0);
    assert_eq!(runtime.json_depth, 0);
    assert!(runtime.frames.is_empty());
    assert!(runtime.active_array_joins.is_empty());
}

fn data(runtime: &Runtime, target: &Value, key: &PropertyKey) -> (Value, bool, bool, bool) {
    let property = runtime.own_property_key(target, key).unwrap();
    let PropertyValue::Data { value, writable } = property.value else {
        panic!("expected data property: {key:?}");
    };
    (value, writable, property.enumerable, property.configurable)
}

fn mapped(name: &str) -> (Runtime, Document, Value, usize, usize) {
    let (mut runtime, mut doc) = prepared();
    let target = runtime
        .execute(
            &format!("function retain({name}){{{name}=17;return arguments;}}retain(1)"),
            &mut doc,
        )
        .unwrap();
    let object = runtime.property_object(&target).unwrap();
    let env = runtime.objects[object].parameter_map[&JsString::from("0")].0;
    assert_eq!(
        runtime.environments[env].bindings[name].value,
        Value::Number(17.0)
    );
    (runtime, doc, target, object, env)
}

#[test]
fn frozen_integrity_cases_preserve_all_228_original_modes() {
    let mut modes = 0;
    for fixture in include_str!("../../../tests/conformance/object-integrity.js")
        .split("// CASE: ")
        .skip(1)
    {
        let (name, source) = fixture.split_once('\n').unwrap();
        for strict in [false, true] {
            let (mut runtime, mut doc) = prepared();
            for helper in [
                include_str!("../../../tests/upstream/test262/harness/sta.js"),
                include_str!("../../../tests/upstream/test262/harness/assert.js"),
                include_str!("../../../tests/upstream/test262/harness/propertyHelper.js"),
            ] {
                runtime.execute(helper, &mut doc).unwrap();
            }
            let result = if strict {
                runtime.execute_strict(source, &mut doc)
            } else {
                runtime.execute(source, &mut doc)
            };
            assert!(result.is_ok(), "{name}, strict={strict}: {result:?}");
            clean(&runtime);
            modes += 1;
        }
    }
    assert_eq!(modes, 228);
}

#[test]
fn integrity_primitive_paths_need_one_tick_and_no_heap_or_boxing() {
    let values = [
        Value::Undefined,
        Value::Null,
        Value::Bool(false),
        Value::Bool(true),
        Value::Number(-0.0),
        Value::Number(f64::NAN),
        Value::Number(f64::INFINITY),
        Value::Symbol(Symbol::unique(Some("identity".into()))),
        Value::String(JsString::from("")),
        Value::String(JsString::from(vec![0xd800; 32_768])),
    ];
    for name in METHODS {
        for value in &values {
            let (mut runtime, mut doc) = prepared();
            let before = arenas(&runtime);
            runtime.allocated = MAX_HEAP;
            runtime.steps = 1;
            let op = operation(name);
            let result = runtime
                .object_integrity(value.clone(), op, &mut doc)
                .unwrap();
            let expected = if op.query {
                Value::Bool(true)
            } else {
                value.clone()
            };
            assert!(json_same_value(&result, &expected), "{name}: {value:?}");
            assert_eq!(runtime.steps, 0);
            assert_eq!(runtime.allocated, MAX_HEAP);
            assert_eq!(arenas(&runtime), before);
            assert!(
                runtime
                    .object_integrity(value.clone(), op, &mut doc)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(arenas(&runtime), before);
            clean(&runtime);
        }
    }
}

#[test]
fn integrity_extensible_queries_skip_keys_virtual_strings_and_accessors() {
    for name in ["Object.isSealed", "Object.isFrozen"] {
        for shape in 0..3 {
            let (mut runtime, mut doc) = prepared();
            let target =
                match shape {
                    0 => runtime.object_ordered([]).unwrap(),
                    1 => runtime
                        .object_ordered((0..256).map(|index| {
                            (format!("key{index}").into(), Value::Number(index as f64))
                        }))
                        .unwrap(),
                    _ => runtime
                        .coerce_object(Value::String(JsString::from(vec![0xd800; 4_096])))
                        .unwrap(),
                };
            let object = runtime.property_object(&target).unwrap();
            // Invocation would fail; a predicate on an extensible target must
            // not read the property or reach its getter at all.
            runtime.objects[object].insert_property(
                "poison".into(),
                Property {
                    value: PropertyValue::Accessor {
                        get: Runtime::native("unreachable getter", Value::Undefined),
                        set: Value::Undefined,
                    },
                    enumerable: true,
                    configurable: true,
                },
            );
            let before = arenas(&runtime);
            let keys = runtime.objects[object].order.clone();
            runtime.allocated = MAX_HEAP;
            runtime.steps = 1;
            assert_eq!(
                runtime
                    .object_integrity(target, operation(name), &mut doc)
                    .unwrap(),
                Value::Bool(false)
            );
            assert_eq!(runtime.steps, 0);
            assert_eq!(runtime.allocated, MAX_HEAP);
            assert_eq!(arenas(&runtime), before);
            assert_eq!(runtime.objects[object].order, keys);
            assert!(!runtime.objects[object].non_extensible);
            clean(&runtime);
        }
    }
}

#[test]
fn integrity_native_dispatch_ignores_extra_contents_and_shares_entry_work() {
    for name in METHODS {
        let (mut runtime, mut doc) = prepared();
        let poison = runtime
            .execute(
                "({get valueOf(){throw 19;},get toString(){throw 23;}})",
                &mut doc,
            )
            .unwrap();
        let large = runtime.array(vec![poison.clone(); 2_048]).unwrap();
        let args = vec![
            Value::Number(-0.0),
            Value::String(JsString::from(vec![0xdfff; 32_768])),
            large,
            poison.clone(),
        ];
        let omitted = Vec::new();
        let native = Native {
            properties: None,
            name: name.into(),
            receiver: poison,
        };
        let before = arenas(&runtime);
        runtime.allocated = MAX_HEAP;
        runtime.steps = 2;
        let result = runtime.native_call(&native, args, &mut doc).unwrap();
        let query = operation(name).query;
        assert!(json_same_value(
            &result,
            &if query {
                Value::Bool(true)
            } else {
                Value::Number(-0.0)
            }
        ));
        assert_eq!(
            runtime.native_call(&native, omitted, &mut doc).unwrap(),
            if query {
                Value::Bool(true)
            } else {
                Value::Undefined
            }
        );
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, MAX_HEAP);
        assert_eq!(arenas(&runtime), before);
        assert!(
            runtime
                .native_call(&native, Vec::new(), &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        clean(&runtime);
    }
}

#[test]
fn integrity_snapshot_refusal_preserves_prevent_extensions_but_no_descriptors() {
    for name in ["Object.seal", "Object.freeze"] {
        for argument in [false, true] {
            for failure in ["entry", "work", "heap"] {
                let (mut runtime, mut doc, target, object, _) = if argument {
                    mapped("parameter")
                } else {
                    let (mut runtime, doc) = prepared();
                    let target = runtime
                        .object_ordered([
                            ("0".into(), Value::Number(17.0)),
                            ("other".into(), Value::Number(2.0)),
                        ])
                        .unwrap();
                    let object = runtime.property_object(&target).unwrap();
                    (runtime, doc, target, object, 0)
                };
                let before = data(&runtime, &target, &"0".into());
                let map = runtime.objects[object].parameter_map.clone();
                let keys = runtime.objects[object].order.clone();
                let shape = arenas(&runtime);
                runtime.steps = match failure {
                    "entry" => 0,
                    "work" => 1,
                    _ => MAX_STEPS,
                };
                if failure == "heap" {
                    runtime.allocated = MAX_HEAP;
                }
                let allocated = runtime.allocated;
                assert!(
                    runtime
                        .object_integrity(target.clone(), operation(name), &mut doc)
                        .unwrap_err()
                        .is_resource_limit()
                );
                assert_eq!(runtime.objects[object].non_extensible, failure != "entry");
                assert_eq!(data(&runtime, &target, &"0".into()), before);
                assert_eq!(runtime.objects[object].parameter_map, map);
                assert_eq!(runtime.objects[object].order, keys);
                assert_eq!(arenas(&runtime), shape);
                if failure != "heap" {
                    assert_eq!(runtime.allocated, allocated);
                }
                clean(&runtime);
            }
        }
    }
}

fn pair_array() -> (Runtime, Document, Value, usize, usize) {
    let (mut runtime, doc) = prepared();
    let target = runtime
        .array(vec![Value::Number(10.0), Value::Number(20.0)])
        .unwrap();
    let Value::Array(array) = target else {
        unreachable!()
    };
    let object = runtime.array_properties[array];
    (runtime, doc, target, object, array)
}

#[test]
fn integrity_array_partial_failure_keeps_exact_first_descriptor_prefix() {
    for frozen in [false, true] {
        let (mut measure, mut doc, target, object, _) = pair_array();
        let old_steps = measure.steps;
        let old_heap = measure.allocated;
        measure.tick().unwrap();
        measure.objects[object].non_extensible = true;
        let keys = measure.integrity_keys(&target, object).unwrap();
        assert_eq!(keys[0].3, PropertyKey::from("0"));
        assert_eq!(
            measure
                .integrity_flags(&target, object, &keys[0].3)
                .unwrap(),
            Some((true, Some(true)))
        );
        measure
            .integrity_define(&target, object, &keys[0].3, frozen, &mut doc)
            .unwrap();
        let prefix_work = old_steps - measure.steps;
        let prefix_heap = measure.allocated - old_heap;
        // Measure the next real write's allocation, not a copied estimate of
        // descriptor or Vec layout. The next flag read is allocation-free.
        let before_next = measure.allocated;
        measure
            .integrity_flags(&target, object, &keys[1].3)
            .unwrap();
        measure
            .integrity_define(&target, object, &keys[1].3, frozen, &mut doc)
            .unwrap();
        let next_heap = measure.allocated - before_next;
        assert!(next_heap > 0);

        for heap_failure in [false, true] {
            let (mut runtime, mut doc, target, object, array) = pair_array();
            if heap_failure {
                runtime.allocated = MAX_HEAP - prefix_heap - next_heap + 1;
            } else {
                runtime.steps = prefix_work;
            }
            let error = runtime
                .object_integrity(
                    target.clone(),
                    IntegrityOperation {
                        frozen,
                        query: false,
                    },
                    &mut doc,
                )
                .unwrap_err();
            assert!(error.is_resource_limit());
            assert!(runtime.objects[object].non_extensible);
            assert_eq!(
                data(&runtime, &target, &"0".into()),
                (Value::Number(10.0), !frozen, true, false)
            );
            assert_eq!(
                data(&runtime, &target, &"1".into()),
                (Value::Number(20.0), true, true, true)
            );
            assert!(
                runtime.objects[object]
                    .values
                    .contains_key(&PropertyKey::from("0"))
            );
            assert!(
                !runtime.objects[object]
                    .values
                    .contains_key(&PropertyKey::from("1"))
            );
            assert_eq!(
                runtime.arrays[array],
                vec![Value::Number(10.0), Value::Number(20.0)]
            );
            assert!(runtime.array_holes[array].is_empty());
            assert_eq!(runtime.array_lengths[array].value, 2);
            assert!(runtime.array_lengths[array].writable);
            if heap_failure {
                assert_eq!(runtime.allocated, MAX_HEAP + 1);
            }
            clean(&runtime);
        }
    }
}

#[test]
fn integrity_snapshot_preserves_key_order_and_uniqueness_across_storage_kinds() {
    let (mut runtime, mut doc) = prepared();
    let target = runtime
        .execute(
            "var first=Symbol('first'),second=Symbol('second'),a=[10,20,30];delete a[1];\
         Object.defineProperty(a,'1',{get:function(){throw 'getter ran';},configurable:true});\
         Object.defineProperty(a,'0',{writable:false});a[4294967294]=40;\
         a['4294967295']='nonindex';a['01']='noncanonical';a.x=1;a.y=2;delete a.x;a.x=3;\
         a[first]=1;a[second]=2;delete a[first];a[first]=3;a",
            &mut doc,
        )
        .unwrap();
    let Value::Array(array) = target else {
        unreachable!()
    };
    let object = runtime.array_properties[array];
    assert!(runtime.array_holes[array].contains(&1));
    assert!(
        runtime.objects[object]
            .values
            .contains_key(&PropertyKey::from("1"))
    );
    let Value::Symbol(first) = runtime.lookup(1, "first").unwrap().1 else {
        unreachable!()
    };
    let Value::Symbol(second) = runtime.lookup(1, "second").unwrap().1 else {
        unreachable!()
    };
    let mut expected: Vec<PropertyKey> = [
        "0",
        "1",
        "2",
        "4294967294",
        "length",
        "4294967295",
        "01",
        "y",
        "x",
    ]
    .into_iter()
    .map(PropertyKey::from)
    .collect();
    expected.extend([PropertyKey::Symbol(second), PropertyKey::Symbol(first)]);
    let actual: Vec<_> = runtime
        .integrity_keys(&target, object)
        .unwrap()
        .into_iter()
        .map(|entry| entry.3)
        .collect();
    assert_eq!(actual, expected);
    // These expectations are spelled independently; no own_keys helper is an oracle.
    runtime
        .object_integrity(target.clone(), operation("Object.freeze"), &mut doc)
        .unwrap();
    assert_eq!(
        runtime
            .integrity_keys(&target, object)
            .unwrap()
            .into_iter()
            .map(|entry| entry.3)
            .collect::<Vec<_>>(),
        expected
    );

    let target = runtime
        .coerce_object(Value::String(JsString::from(vec![0x41, 0xd800, 0xdfff])))
        .unwrap();
    let object = runtime.property_object(&target).unwrap();
    for key in ["1", "length"] {
        assert!(
            runtime
                .define_property_key(
                    &target,
                    &key.into(),
                    PropertyDescriptor::default(),
                    &mut doc
                )
                .unwrap()
        );
    }
    assert!(
        runtime.objects[object]
            .values
            .contains_key(&PropertyKey::from("1"))
    );
    assert!(
        runtime.objects[object]
            .values
            .contains_key(&PropertyKey::from("length"))
    );
    assert!(
        runtime
            .define_property_key(
                &target,
                &"tail".into(),
                PropertyDescriptor::data_property(Value::Number(4.0), true, true, true),
                &mut doc
            )
            .unwrap()
    );
    let expected: Vec<PropertyKey> = ["0", "1", "2", "length", "tail"]
        .into_iter()
        .map(PropertyKey::from)
        .collect();
    assert_eq!(
        runtime
            .integrity_keys(&target, object)
            .unwrap()
            .into_iter()
            .map(|entry| entry.3)
            .collect::<Vec<_>>(),
        expected
    );
    runtime
        .object_integrity(target.clone(), operation("Object.freeze"), &mut doc)
        .unwrap();
    assert_eq!(
        runtime
            .integrity_keys(&target, object)
            .unwrap()
            .into_iter()
            .map(|entry| entry.3)
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        data(&runtime, &target, &"1".into()),
        (
            Value::String(JsString::from(vec![0xd800])),
            false,
            true,
            false
        )
    );
    clean(&runtime);
}

#[test]
fn integrity_boxed_string_flags_do_not_materialize_virtual_values() {
    for units in [vec![0x41, 0xd800, 0xdfff], vec![0xd800; 256]] {
        let (mut runtime, mut doc) = prepared();
        let text = JsString::from(units.clone());
        let target = runtime.coerce_object(Value::String(text.clone())).unwrap();
        let object = runtime.property_object(&target).unwrap();
        let shape = arenas(&runtime);
        let keys = [PropertyKey::from("0"), PropertyKey::from("length")];
        runtime.allocated = MAX_HEAP;
        for key in keys {
            assert_eq!(
                runtime.integrity_flags(&target, object, &key).unwrap(),
                Some((false, Some(false)))
            );
        }
        assert_eq!(runtime.allocated, MAX_HEAP);
        assert_eq!(arenas(&runtime), shape);
        assert!(runtime.objects[object].values.is_empty());
        assert!(runtime.objects[object].order.is_empty());
        assert_eq!(runtime.objects[object].boxed, Some(Value::String(text)));

        for name in METHODS {
            let (mut runtime, mut doc2) = prepared();
            let target = runtime
                .coerce_object(Value::String(JsString::from(units.clone())))
                .unwrap();
            let object = runtime.property_object(&target).unwrap();
            runtime.objects[object].non_extensible = true;
            let before = runtime.allocated;
            let keys = runtime.integrity_keys(&target, object).unwrap();
            let snapshot_heap = runtime.allocated - before;
            assert_eq!(keys.len(), units.len() + 1);
            // All remaining flag-only visits must succeed with no heap left.
            runtime.allocated = MAX_HEAP;
            for (_, _, _, key) in &keys {
                assert_eq!(
                    runtime.integrity_flags(&target, object, key).unwrap(),
                    Some((false, Some(false)))
                );
            }
            let (mut actual, _) = prepared();
            let target = actual
                .coerce_object(Value::String(JsString::from(units.clone())))
                .unwrap();
            let object = actual.property_object(&target).unwrap();
            actual.objects[object].non_extensible = true;
            actual.allocated = MAX_HEAP - snapshot_heap;
            let result = actual
                .object_integrity(target.clone(), operation(name), &mut doc2)
                .unwrap();
            assert_eq!(
                result,
                if operation(name).query {
                    Value::Bool(true)
                } else {
                    target
                }
            );
            assert_eq!(actual.allocated, MAX_HEAP);
            assert!(actual.objects[object].values.is_empty());
            assert!(actual.objects[object].order.is_empty());
            assert!(
                actual
                    .object_integrity(Value::Object(object), operation(name), &mut doc)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert!(actual.objects[object].values.is_empty());
            clean(&actual);
        }
    }
}

fn sparse(length: u32, high: bool) -> (Runtime, Document, Value, usize, usize) {
    let (mut runtime, mut doc) = prepared();
    let target = runtime.array(Vec::new()).unwrap();
    let Value::Array(array) = target else {
        unreachable!()
    };
    let symbol = Symbol::unique(Some("tail".into()));
    let mut keys = vec![
        PropertyKey::from("2"),
        PropertyKey::from("label"),
        PropertyKey::Symbol(symbol),
    ];
    if high {
        keys.push("4294967294".into());
        keys.push("01".into());
    }
    for key in keys {
        assert!(
            runtime
                .define_property_key(
                    &target,
                    &key,
                    PropertyDescriptor::data_property(Value::Number(1.0), true, true, true),
                    &mut doc
                )
                .unwrap()
        );
    }
    assert!(
        runtime
            .define_property_key(
                &target,
                &"length".into(),
                PropertyDescriptor {
                    value: Some(Value::Number(length as f64)),
                    ..PropertyDescriptor::default()
                },
                &mut doc
            )
            .unwrap()
    );
    let object = runtime.array_properties[array];
    assert!(runtime.arrays[array].is_empty());
    (runtime, doc, target, object, array)
}

#[test]
fn integrity_sparse_arrays_charge_stored_shape_not_logical_u32_length() {
    for name in METHODS {
        let mut costs = Vec::new();
        for length in [16, 65_536, u32::MAX] {
            let (mut runtime, mut doc, target, object, array) = sparse(length, false);
            if operation(name).query {
                runtime
                    .object_integrity(target.clone(), operation("Object.freeze"), &mut doc)
                    .unwrap();
            }
            let order = runtime.objects[object].order.clone();
            let before = runtime.allocated;
            runtime.steps = 4_096;
            let result = runtime
                .object_integrity(target.clone(), operation(name), &mut doc)
                .unwrap();
            assert_eq!(
                result,
                if operation(name).query {
                    Value::Bool(true)
                } else {
                    target.clone()
                }
            );
            costs.push((4_096 - runtime.steps, runtime.allocated - before));
            assert_eq!(runtime.objects[object].order, order);
            assert!(runtime.arrays[array].is_empty());
            assert!(runtime.array_holes[array].is_empty());
            assert_eq!(runtime.array_lengths[array].value, length);
            assert_eq!(runtime.array_lengths[array].writable, name == "Object.seal");
            assert!(
                runtime
                    .own_property(&target, &JsString::from("1"))
                    .is_none()
            );
            clean(&runtime);
        }
        assert!(
            costs.iter().all(|cost| *cost == costs[0]),
            "{name}: {costs:?}"
        );
    }
    let (mut runtime, mut doc, target, object, array) = sparse(u32::MAX, true);
    runtime.steps = 4_096;
    runtime
        .object_integrity(target.clone(), operation("Object.freeze"), &mut doc)
        .unwrap();
    assert_eq!(runtime.array_lengths[array].value, u32::MAX);
    assert_eq!(runtime.objects[object].values.len(), 5);
    assert!(runtime.arrays[array].is_empty());
    assert_eq!(
        data(&runtime, &target, &"4294967294".into()),
        (Value::Number(1.0), false, true, false)
    );
}

#[test]
fn integrity_mapped_arguments_budget_names_and_preserve_or_detach_aliases() {
    for frozen in [false, true] {
        let mut costs = Vec::new();
        for length in [1, 257, 4_097] {
            let name = "a".repeat(length);
            let (mut runtime, mut doc, target, object, env) = mapped(&name);
            let key = JsString::from("0");
            let stored_name = runtime.objects[object].parameter_map[&key].1.as_ptr();
            let before = runtime.allocated;
            runtime.steps = MAX_STEPS;
            runtime
                .object_integrity(
                    target.clone(),
                    IntegrityOperation {
                        frozen,
                        query: false,
                    },
                    &mut doc,
                )
                .unwrap();
            costs.push((MAX_STEPS - runtime.steps, runtime.allocated - before));
            assert_eq!(
                data(&runtime, &target, &key.clone().into()),
                (Value::Number(17.0), !frozen, true, false)
            );
            assert_eq!(
                runtime.objects[object].parameter_map.contains_key(&key),
                !frozen
            );
            if !frozen {
                assert_eq!(
                    runtime.objects[object].parameter_map[&key].1.as_ptr(),
                    stored_name
                );
            }
            runtime.environments[env]
                .bindings
                .get_mut(&name)
                .unwrap()
                .value = Value::Number(23.0);
            assert_eq!(
                data(&runtime, &target, &key.into()).0,
                Value::Number(if frozen { 17.0 } else { 23.0 })
            );
            clean(&runtime);
        }
        assert!(
            costs.windows(2).all(|pair| pair[0].0 < pair[1].0),
            "{costs:?}"
        );
        // The implementation borrows parameter names; it need not allocate a
        // name-sized temporary merely to account for name comparison work.
        assert!(costs.iter().all(|cost| cost.1 == costs[0].1), "{costs:?}");
    }
}

#[test]
fn integrity_mapped_name_work_failure_preserves_the_unvisited_alias() {
    for name in ["a".to_string(), "a".repeat(4_097)] {
        let (mut measure, mut doc, target, object, _) = mapped(&name);
        measure.steps = MAX_STEPS;
        measure.tick().unwrap();
        measure.objects[object].non_extensible = true;
        let keys = measure.integrity_keys(&target, object).unwrap();
        assert_eq!(keys[0].3, PropertyKey::from("0"));
        measure
            .integrity_flags(&target, object, &keys[0].3)
            .unwrap();
        let prefix = MAX_STEPS - measure.steps;
        let before_define = measure.steps;
        measure
            .integrity_define(&target, object, &keys[0].3, true, &mut doc)
            .unwrap();
        let define_work = before_define - measure.steps;
        assert!(define_work > 1);

        let (mut runtime, mut doc, target, object, env) = mapped(&name);
        let index = JsString::from("0");
        let key: PropertyKey = index.clone().into();
        let map = runtime.objects[object].parameter_map.clone();
        let stored = runtime.objects[object].values[&key].clone();
        runtime.steps = prefix;
        assert!(
            runtime
                .object_integrity(target.clone(), operation("Object.freeze"), &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert!(runtime.objects[object].non_extensible);
        assert_eq!(runtime.objects[object].parameter_map, map);
        let current = &runtime.objects[object].values[&key];
        assert_eq!(current.configurable, stored.configurable);
        assert!(matches!(
            &current.value,
            PropertyValue::Data {
                value: Value::Number(1.0),
                writable: true
            }
        ));
        runtime.environments[env]
            .bindings
            .get_mut(&name)
            .unwrap()
            .value = Value::Number(29.0);
        assert_eq!(data(&runtime, &target, &key).0, Value::Number(29.0));
        clean(&runtime);
    }
}

#[test]
fn integrity_partial_argument_freeze_detaches_only_the_completed_mapping() {
    fn arguments() -> (Runtime, Document, Value, usize) {
        let (mut runtime, mut doc) = prepared();
        let target = runtime
            .execute(
                "function retain(a,b){a=17;b=23;return arguments;}retain(1,2)",
                &mut doc,
            )
            .unwrap();
        let object = runtime.property_object(&target).unwrap();
        (runtime, doc, target, object)
    }
    let (mut measure, mut doc, target, object) = arguments();
    let before = measure.steps;
    measure.tick().unwrap();
    measure.objects[object].non_extensible = true;
    let keys = measure.integrity_keys(&target, object).unwrap();
    measure
        .integrity_flags(&target, object, &keys[0].3)
        .unwrap();
    measure
        .integrity_define(&target, object, &keys[0].3, true, &mut doc)
        .unwrap();
    let prefix = before - measure.steps;

    let (mut runtime, mut doc, target, object) = arguments();
    let first = JsString::from("0");
    let second = JsString::from("1");
    let (env, name) = runtime.objects[object].parameter_map[&second].clone();
    runtime.steps = prefix;
    assert!(
        runtime
            .object_integrity(target.clone(), operation("Object.freeze"), &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert!(runtime.objects[object].non_extensible);
    assert!(!runtime.objects[object].parameter_map.contains_key(&first));
    assert!(runtime.objects[object].parameter_map.contains_key(&second));
    assert_eq!(
        data(&runtime, &target, &first.into()),
        (Value::Number(17.0), false, true, false)
    );
    assert_eq!(
        data(&runtime, &target, &second.clone().into()),
        (Value::Number(23.0), true, true, true)
    );
    runtime.environments[env]
        .bindings
        .get_mut(&name)
        .unwrap()
        .value = Value::Number(31.0);
    assert_eq!(
        data(&runtime, &target, &second.into()).0,
        Value::Number(31.0)
    );
    assert_eq!(
        data(&runtime, &target, &"length".into()),
        (Value::Number(2.0), true, false, true)
    );
    clean(&runtime);
}

#[test]
fn integrity_hosts_are_rejected_before_touching_existing_symbol_sidecars() {
    for name in METHODS {
        let (mut runtime, mut doc) = prepared();
        let node = doc.query_selector("#kept").unwrap();
        let targets = [
            Value::Window,
            Value::Document,
            Value::Node(node),
            Value::Style(node),
            Value::ClassList(node),
            Value::Console,
        ];
        for target in targets {
            let key = PropertyKey::Symbol(Symbol::unique(Some("sidecar".into())));
            assert!(
                runtime
                    .define_property_key(
                        &target,
                        &key,
                        PropertyDescriptor::data_property(Value::Number(7.0), true, true, true),
                        &mut doc
                    )
                    .unwrap()
            );
            let object = runtime.symbol_property_object(&target).unwrap();
            let shape = arenas(&runtime);
            let global = runtime.lookup(0, "globalThis").unwrap().1;
            let before_html = doc.outer_html(node);
            let before = runtime.allocated;
            runtime.allocated = MAX_HEAP;
            runtime.steps = 1;
            assert!(
                runtime
                    .object_integrity(target.clone(), operation(name), &mut doc)
                    .unwrap_err()
                    .is_unsupported()
            );
            assert_eq!(runtime.steps, 0);
            assert_eq!(runtime.allocated, MAX_HEAP);
            assert_eq!(arenas(&runtime), shape);
            assert!(!runtime.objects[object].non_extensible);
            assert_eq!(
                data(&runtime, &target, &key),
                (Value::Number(7.0), true, true, true)
            );
            assert_eq!(runtime.lookup(0, "globalThis").unwrap().1, global);
            assert_eq!(doc.outer_html(node), before_html);
            // The next independent target's setup is outside this measured call.
            runtime.allocated = before;
            runtime.steps = MAX_STEPS;
            clean(&runtime);
        }
    }
}

#[test]
fn integrity_own_properties_do_not_walk_host_or_cyclic_prototypes() {
    for cyclic in [false, true] {
        let (mut runtime, mut doc) = prepared();
        let target = runtime
            .object_ordered([("own".into(), Value::Number(1.0))])
            .unwrap();
        let object = runtime.property_object(&target).unwrap();
        runtime.objects[object].prototype = Some(if cyclic {
            target.clone()
        } else {
            Value::Window
        });
        runtime.steps = 512;
        runtime
            .object_integrity(target.clone(), operation("Object.freeze"), &mut doc)
            .unwrap();
        assert_eq!(
            runtime
                .object_integrity(target.clone(), operation("Object.isFrozen"), &mut doc)
                .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            data(&runtime, &target, &"own".into()),
            (Value::Number(1.0), false, true, false)
        );
        clean(&runtime);
    }
}

#[test]
fn integrity_repeated_calls_share_terminal_limits_and_unwind_machine_state() {
    for body in [
        "var target={};for(var i=0;i<32;i++)target['k'+i]=i;Object.seal(target);if(!Object.isSealed(target))throw 'guard';while(true){Object.seal(target);}",
        "var target=new String('abcdefghijklmnop');Object.preventExtensions(target);if(!Object.isFrozen(target))throw 'guard';while(true){Object.isFrozen(target);}",
    ] {
        let (mut runtime, mut doc) = prepared();
        let source = format!(
            "var prior=0,caught=false,finalized=false;try{{prior=7;{body}}}catch(e){{caught=true;}}finally{{finalized=true;}}"
        );
        assert!(
            runtime
                .execute(&source, &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.lookup(1, "prior").unwrap().1, Value::Number(7.0));
        assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Bool(false));
        assert_eq!(
            runtime.lookup(1, "finalized").unwrap().1,
            Value::Bool(false)
        );
        let target = runtime.lookup(1, "target").unwrap().1;
        assert!(runtime.objects[runtime.property_object(&target).unwrap()].non_extensible);
        clean(&runtime);
    }
}
