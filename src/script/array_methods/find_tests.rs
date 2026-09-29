use super::*;

const FIND_METHODS: [&str; 4] = ["find", "findIndex", "findLast", "findLastIndex"];

fn reverse(method: &str) -> bool {
    matches!(method, "findLast" | "findLastIndex")
}

fn found(method: &str, index: u64, value: Value) -> Value {
    if matches!(method, "findIndex" | "findLastIndex") {
        Value::Number(index as f64)
    } else {
        value
    }
}

fn missing(method: &str) -> Value {
    if matches!(method, "findIndex" | "findLastIndex") {
        Value::Number(-1.0)
    } else {
        Value::Undefined
    }
}

fn prepared() -> (Runtime, Document) {
    (
        Runtime::new(),
        Document::parse("<p id='kept'>unchanged</p>"),
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

fn setup(source: &str) -> (Runtime, Document, Value, Value) {
    let (mut runtime, mut doc) = prepared();
    let target = runtime.execute(source, &mut doc).unwrap();
    let callback = runtime.lookup(1, "predicate").unwrap().1;
    (runtime, doc, target, callback)
}

// Measure the real length/Get prefix on an equivalent fresh runtime. In
// particular this deliberately contains no HasProperty step: holes are read.
fn first_get_prefix(runtime: &mut Runtime, target: &Value, doc: &mut Document) -> Value {
    runtime.tick().unwrap();
    runtime.coerce_object(target.clone()).unwrap();
    runtime.charge(64).unwrap();
    let length = runtime
        .reduce_get(target, &JsString::from("length"), doc)
        .unwrap();
    runtime.number_value(length, doc).unwrap();
    runtime.tick().unwrap();
    let key = runtime.reduce_index_key(0).unwrap();
    runtime.reduce_get(target, &key, doc).unwrap()
}

#[test]
fn frozen_find_sources_preserve_all_504_original_modes() {
    let mut modes = 0;
    for fixture in include_str!("../../../tests/conformance/array-find.js")
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
    assert_eq!(modes, 504);
}

#[test]
fn find_callback_preflight_keeps_getter_effects_and_charges_hole_callbacks() {
    for source in [
        "var reads=0,calls=0,target={length:1};Object.defineProperty(target,'0',{get:function(){reads++;target.touched=7;return 4;}});function predicate(){calls++;return true;}target",
        "var reads=0,calls=0,target=Object.create(null);target.length=1;function predicate(){calls++;return true;}target",
        "var reads=0,calls=0,target={length:1};Object.defineProperty(target,'0',{get:undefined});function predicate(){calls++;return true;}target",
    ] {
        let (mut measure, mut doc, target, _) = setup(source);
        let before = measure.allocated;
        first_get_prefix(&mut measure, &target, &mut doc);
        let prefix = measure.allocated - before;
        let vector = 32 + 3 * std::mem::size_of::<Value>();
        let getter_ran = measure.lookup(1, "reads").unwrap().1;
        for method in FIND_METHODS {
            let (mut runtime, mut doc, target, callback) = setup(source);
            let args = vec![callback];
            runtime.allocated = MAX_HEAP - prefix - vector + 1;
            let error = runtime
                .array_method(target.clone(), method, args, &mut doc)
                .unwrap_err();
            assert!(error.is_resource_limit(), "{method}: {error:?}");
            assert_eq!(runtime.allocated, MAX_HEAP + 1);
            assert_eq!(runtime.lookup(1, "reads").unwrap().1, getter_ran);
            assert_eq!(runtime.lookup(1, "calls").unwrap().1, Value::Number(0.0));
            if getter_ran == Value::Number(1.0) {
                let object = runtime.property_object(&target).unwrap();
                assert_eq!(
                    runtime.objects[object].get("touched"),
                    Some(&Value::Number(7.0))
                );
            }
            clean(&runtime);
        }
    }
}

#[test]
fn find_holes_and_getterless_accessors_are_visited_in_both_directions() {
    for method in FIND_METHODS {
        let (mut runtime, mut doc, target, callback) = setup(
            "var calls=0,log='',target=Object.create(null);target.length=3;Object.defineProperty(target,'1',{get:undefined});target[2]=undefined;function predicate(value,index,object){if(value!==undefined||object!==target||arguments.length!==3)throw 'bad visit';calls++;log+=index;return false;}target",
        );
        assert_eq!(
            runtime
                .array_method(target, method, vec![callback], &mut doc)
                .unwrap(),
            missing(method)
        );
        assert_eq!(runtime.lookup(1, "calls").unwrap().1, Value::Number(3.0));
        assert_eq!(
            runtime.lookup(1, "log").unwrap().1,
            Value::String(if reverse(method) { "210" } else { "012" }.into())
        );
        clean(&runtime);
    }
}

#[test]
fn find_first_match_allocates_from_shape_not_logical_length() {
    for method in FIND_METHODS {
        for hole in [false, true] {
            let mut costs = Vec::new();
            for length in [
                1.0,
                65_537.0,
                4_294_967_295.0,
                4_294_967_296.0,
                9_007_199_254_740_991.0,
                f64::INFINITY,
                1e100,
            ] {
                let (mut runtime, mut doc) = prepared();
                let callback = runtime.execute("var calls=0,lastIndex=-1;function predicate(v,k){calls++;lastIndex=k;return true;}predicate", &mut doc).unwrap();
                let effective = length.min(9_007_199_254_740_991.0) as u64;
                let index = if reverse(method) { effective - 1 } else { 0 };
                let target = runtime
                    .object_ordered([("length".into(), Value::Number(length))])
                    .unwrap();
                let object = runtime.property_object(&target).unwrap();
                runtime.objects[object].prototype = None;
                if !hole {
                    assert!(
                        runtime
                            .define_property_key(
                                &target,
                                &index.to_string().into(),
                                PropertyDescriptor::data_property(
                                    Value::Number(7.0),
                                    true,
                                    true,
                                    true
                                ),
                                &mut doc
                            )
                            .unwrap()
                    );
                }
                let arrays = runtime.arrays.len();
                let before = runtime.allocated;
                runtime.steps = 4_096;
                let result = runtime
                    .array_method(target, method, vec![callback], &mut doc)
                    .unwrap();
                assert_eq!(
                    result,
                    found(
                        method,
                        index,
                        if hole {
                            Value::Undefined
                        } else {
                            Value::Number(7.0)
                        }
                    )
                );
                assert_eq!(runtime.lookup(1, "calls").unwrap().1, Value::Number(1.0));
                assert_eq!(
                    runtime.lookup(1, "lastIndex").unwrap().1,
                    Value::Number(index as f64)
                );
                assert_eq!(runtime.arrays.len(), arrays);
                assert_eq!(
                    runtime.objects[object].values.len(),
                    if hole { 1 } else { 2 }
                );
                costs.push(runtime.allocated - before);
                clean(&runtime);
            }
            assert!(
                costs.iter().all(|cost| *cost == costs[0]),
                "{method}, hole={hole}: {costs:?}"
            );
        }
        let (mut runtime, mut doc) = prepared();
        let callback = runtime
            .execute("function predicate(){return true;}predicate", &mut doc)
            .unwrap();
        let target = runtime.array(Vec::new()).unwrap();
        let Value::Array(array) = target else {
            unreachable!()
        };
        runtime.array_lengths[array].value = u32::MAX;
        runtime.steps = 4_096;
        assert_eq!(
            runtime
                .array_method(target, method, vec![callback], &mut doc)
                .unwrap(),
            found(
                method,
                if reverse(method) {
                    u64::from(u32::MAX) - 1
                } else {
                    0
                },
                Value::Undefined
            )
        );
        assert!(runtime.arrays[array].is_empty());
        assert!(runtime.array_holes[array].is_empty());
        assert_eq!(runtime.array_lengths[array].value, u32::MAX);
        clean(&runtime);
    }
}

#[test]
fn find_index_key_checks_work_and_heap_before_decimal_storage() {
    for (index, expected) in [
        (0, "0"),
        (9, "9"),
        (10, "10"),
        (99, "99"),
        (100, "100"),
        (4_294_967_294, "4294967294"),
        (999_999_999_999_999, "999999999999999"),
        (1_000_000_000_000_000, "1000000000000000"),
        (9_007_199_254_740_990, "9007199254740990"),
        (9_007_199_254_740_991, "9007199254740991"),
    ] {
        let (mut runtime, _) = prepared();
        let before = runtime.allocated;
        let work = expected.len() + 1;
        runtime.steps = work - 1;
        assert!(
            runtime
                .reduce_index_key(index)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.allocated, before);
        runtime.steps = work;
        runtime.allocated = MAX_HEAP - 64;
        assert_eq!(
            runtime.reduce_index_key(index).unwrap(),
            JsString::from(expected)
        );
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, MAX_HEAP);
        runtime.steps = work;
        runtime.allocated = MAX_HEAP - 63;
        assert!(
            runtime
                .reduce_index_key(index)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.allocated, MAX_HEAP + 1);
    }
    for index in [9_007_199_254_740_992, u64::MAX] {
        let (mut runtime, _) = prepared();
        let before = (runtime.steps, runtime.allocated);
        assert!(
            runtime
                .reduce_index_key(index)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!((runtime.steps, runtime.allocated), before);
    }
}

fn mapped_argument(name: &str) -> (Runtime, Document, Value, Value, usize, usize) {
    let (mut runtime, mut doc, target, callback) = setup(&format!(
        "var calls=0;function retain({name}){{{name}=17;return arguments;}}function predicate(){{calls++;return true;}}retain(1)"
    ));
    let object = runtime.property_object(&target).unwrap();
    let env = runtime.objects[object].parameter_map[&JsString::from("0")].0;
    // A one-parameter sloppy arguments object has a stale stored descriptor
    // and a live environment value. No synthetic/unreachable mapping is used.
    assert_eq!(
        runtime.environments[env].bindings[name].value,
        Value::Number(17.0)
    );
    assert_eq!(
        runtime
            .reduce_get(&target, &JsString::from("0"), &mut doc)
            .unwrap(),
        Value::Number(17.0)
    );
    (runtime, doc, target, callback, object, env)
}

#[test]
fn find_mapped_binding_names_charge_before_live_reads_and_preserve_aliases() {
    let mut costs = Vec::new();
    for length in [1, 257, 4_097] {
        let name = "a".repeat(length);
        let (mut measure, mut doc, target, _, object, _) = mapped_argument(&name);
        let key = JsString::from("0");
        let name_pointer = measure.objects[object].parameter_map[&key].1.as_ptr();
        measure.steps = MAX_STEPS;
        let before = measure.allocated;
        assert_eq!(
            measure.reduce_get(&target, &key, &mut doc).unwrap(),
            Value::Number(17.0)
        );
        let work = MAX_STEPS - measure.steps;
        costs.push((work, measure.allocated - before));
        assert_eq!(
            measure.objects[object].parameter_map[&key].1.as_ptr(),
            name_pointer
        );

        let (mut limited, mut doc, target, _, object, env) = mapped_argument(&name);
        let before = limited.allocated;
        limited.steps = work - 1;
        assert!(
            limited
                .reduce_get(&target, &key, &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(limited.allocated, before);
        assert!(limited.objects[object].parameter_map.contains_key(&key));
        assert_eq!(
            limited.environments[env].bindings[&name].value,
            Value::Number(17.0)
        );
        assert_eq!(limited.lookup(1, "calls").unwrap().1, Value::Number(0.0));
        clean(&limited);

        for method in FIND_METHODS {
            let (mut runtime, mut doc, target, callback, object, env) = mapped_argument(&name);
            runtime.steps = MAX_STEPS;
            assert_eq!(
                runtime
                    .array_method(target.clone(), method, vec![callback], &mut doc)
                    .unwrap(),
                found(method, 0, Value::Number(17.0))
            );
            assert_eq!(runtime.lookup(1, "calls").unwrap().1, Value::Number(1.0));
            assert!(runtime.objects[object].parameter_map.contains_key(&key));
            runtime.environments[env]
                .bindings
                .get_mut(&name)
                .unwrap()
                .value = Value::Number(23.0);
            assert!(matches!(
                runtime.own_property(&target, &key).unwrap().value,
                PropertyValue::Data {
                    value: Value::Number(23.0),
                    writable: true
                }
            ));
            clean(&runtime);
        }
    }
    assert!(
        costs.windows(2).all(|pair| pair[0].0 < pair[1].0),
        "{costs:?}"
    );
    assert!(costs.iter().all(|cost| cost.1 == 128), "{costs:?}");
}

#[test]
fn find_property_work_tracks_value_mapping_and_hole_tree_sizes() {
    for present in [false, true] {
        let mut costs = Vec::new();
        for size in [1, 32, 512] {
            let (mut runtime, mut doc) = prepared();
            let target = runtime
                .object_ordered((0..size).map(|index| {
                    (
                        if present && index == 0 {
                            "0".into()
                        } else {
                            format!("stored{index}").into()
                        },
                        Value::Number(7.0),
                    )
                }))
                .unwrap();
            let object = runtime.property_object(&target).unwrap();
            runtime.objects[object].prototype = None;
            let key = JsString::from("0");
            runtime.steps = MAX_STEPS;
            let before = runtime.allocated;
            assert_eq!(
                runtime.reduce_get(&target, &key, &mut doc).unwrap(),
                if present {
                    Value::Number(7.0)
                } else {
                    Value::Undefined
                }
            );
            let work = MAX_STEPS - runtime.steps;
            assert_eq!(runtime.allocated - before, 128);
            costs.push(work);
            // Lookup preflight is independent of the descriptor value. Replace
            // it with a real author getter without changing the table size.
            let getter = runtime
                .execute(
                    "var reads=0;function read(){reads++;return 7;}read",
                    &mut doc,
                )
                .unwrap();
            let changed_key = if present {
                key.clone()
            } else {
                JsString::from("stored0")
            };
            runtime.objects[object]
                .values
                .get_mut(&changed_key.into())
                .unwrap()
                .value = PropertyValue::Accessor {
                get: getter,
                set: Value::Undefined,
            };
            let before = runtime.allocated;
            runtime.steps = work - 1;
            assert!(
                runtime
                    .reduce_get(&target, &key, &mut doc)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(runtime.allocated, before);
            assert_eq!(runtime.lookup(1, "reads").unwrap().1, Value::Number(0.0));
        }
        assert!(costs.windows(2).all(|pair| pair[0] < pair[1]), "{costs:?}");
    }

    let mut hole_costs = Vec::new();
    for holes in [1, 32, 512] {
        let (mut runtime, mut doc) = prepared();
        let target = runtime.array(vec![Value::Number(7.0); 1_024]).unwrap();
        for index in 1..=holes {
            assert!(
                runtime
                    .delete_property(target.clone(), &JsString::from(index.to_string()))
                    .unwrap()
            );
        }
        runtime.steps = MAX_STEPS;
        let before = runtime.allocated;
        assert_eq!(
            runtime
                .reduce_get(&target, &JsString::from("0"), &mut doc)
                .unwrap(),
            Value::Number(7.0)
        );
        assert_eq!(runtime.allocated - before, 128);
        hole_costs.push(MAX_STEPS - runtime.steps);
    }
    assert!(
        hole_costs.windows(2).all(|pair| pair[0] < pair[1]),
        "{hole_costs:?}"
    );

    let mut map_costs = Vec::new();
    let names = (0..32)
        .map(|index| format!("p{index}"))
        .collect::<Vec<_>>()
        .join(",");
    let arguments = (0..32).map(|_| "1").collect::<Vec<_>>().join(",");
    for detach in [true, false] {
        let (mut runtime, mut doc) = prepared();
        let target = runtime
            .execute(
                &format!("function retain({names}){{return arguments;}}retain({arguments})"),
                &mut doc,
            )
            .unwrap();
        let object = runtime.property_object(&target).unwrap();
        if detach {
            for index in 1..32 {
                assert!(
                    runtime
                        .define_property_key(
                            &target,
                            &index.to_string().into(),
                            PropertyDescriptor {
                                writable: Some(false),
                                ..PropertyDescriptor::default()
                            },
                            &mut doc
                        )
                        .unwrap()
                );
            }
        }
        assert_eq!(
            runtime.objects[object].parameter_map.len(),
            if detach { 1 } else { 32 }
        );
        runtime.steps = MAX_STEPS;
        assert_eq!(
            runtime
                .reduce_get(&target, &JsString::from("0"), &mut doc)
                .unwrap(),
            Value::Number(1.0)
        );
        map_costs.push(MAX_STEPS - runtime.steps);
    }
    assert!(map_costs[0] < map_costs[1], "{map_costs:?}");
}

#[test]
fn find_host_and_cyclic_prototypes_are_reached_only_when_needed() {
    for method in FIND_METHODS {
        for host_kind in 0..4 {
            let (mut runtime, mut doc) = prepared();
            let node = doc.query_selector("#kept").unwrap();
            let host = match host_kind {
                0 => Value::Window,
                1 => Value::Document,
                2 => Value::Node(node),
                _ => Value::Style(node),
            };
            let sidecar = runtime
                .ensure_symbol_property_object(&host)
                .unwrap()
                .unwrap();
            let callback = runtime.lookup(1, "Boolean").unwrap().1;
            let before = runtime.allocated;
            assert!(
                runtime
                    .array_method(host, method, vec![callback], &mut doc)
                    .unwrap_err()
                    .is_unsupported()
            );
            assert_eq!(runtime.allocated, before);
            assert!(!runtime.objects[sidecar].non_extensible);
            clean(&runtime);
        }
        for decisive in [true, false] {
            let (mut runtime, mut doc, target, callback) = setup(&format!(
                "var calls=0,target=Object.create(window);target.length=2;target[{}]=7;function predicate(){{calls++;return {decisive};}}target",
                if reverse(method) { 1 } else { 0 }
            ));
            let result = runtime.array_method(target, method, vec![callback], &mut doc);
            if decisive {
                assert_eq!(
                    result.unwrap(),
                    found(
                        method,
                        if reverse(method) { 1 } else { 0 },
                        Value::Number(7.0)
                    )
                );
            } else {
                assert!(result.unwrap_err().is_unsupported());
            }
            assert_eq!(runtime.lookup(1, "calls").unwrap().1, Value::Number(1.0));
            clean(&runtime);
        }
        let (mut runtime, mut doc, target, callback) =
            setup("var calls=0,target={length:1};function predicate(){calls++;return true;}target");
        let next = runtime.object_ordered([]).unwrap();
        let first_id = runtime.property_object(&target).unwrap();
        let next_id = runtime.property_object(&next).unwrap();
        runtime.objects[first_id].prototype = Some(next);
        runtime.objects[next_id].prototype = Some(target.clone());
        runtime.steps = MAX_STEPS;
        assert!(
            runtime
                .array_method(target, method, vec![callback], &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.lookup(1, "calls").unwrap().1, Value::Number(0.0));
        clean(&runtime);
    }
}

#[test]
fn find_inherited_getter_uses_original_receiver_at_the_depth_boundary() {
    for depth in [MAX_DEPTH, MAX_DEPTH + 1] {
        for method in FIND_METHODS {
            let source = format!(
                "var reads=0,calls=0,target={{length:1}},cursor=target;for(var i=1;i<{depth};i++){{var next={{}};Object.setPrototypeOf(cursor,next);cursor=next;}}Object.defineProperty(cursor,'0',{{get:function(){{if(this!==target)throw 'wrong receiver';reads++;return 9;}}}});function predicate(){{calls++;return true;}}target"
            );
            let (mut runtime, mut doc, target, callback) = setup(&source);
            runtime.steps = MAX_STEPS;
            let result = runtime.array_method(target, method, vec![callback], &mut doc);
            if depth == MAX_DEPTH {
                assert_eq!(result.unwrap(), found(method, 0, Value::Number(9.0)));
                assert_eq!(runtime.lookup(1, "reads").unwrap().1, Value::Number(1.0));
                assert_eq!(runtime.lookup(1, "calls").unwrap().1, Value::Number(1.0));
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert_eq!(runtime.lookup(1, "reads").unwrap().1, Value::Number(0.0));
                assert_eq!(runtime.lookup(1, "calls").unwrap().1, Value::Number(0.0));
            }
            clean(&runtime);
        }
    }
}

#[test]
fn find_boxed_string_unit_allocation_follows_edge_preflight() {
    for index in 0..3 {
        for shortage in [false, true] {
            let (mut runtime, mut doc) = prepared();
            let units = [0x41, 0xd800, 0xdfff];
            let text = JsString::from(units.to_vec());
            let target = runtime.coerce_object(Value::String(text.clone())).unwrap();
            let object = runtime.property_object(&target).unwrap();
            let key = JsString::from(index.to_string());
            runtime.allocated = MAX_HEAP - 128 + usize::from(shortage);
            let result = runtime.reduce_get(&target, &key, &mut doc);
            if shortage {
                assert!(result.unwrap_err().is_resource_limit());
            } else {
                assert_eq!(
                    result.unwrap(),
                    Value::String(JsString::from(vec![units[index]]))
                );
            }
            assert_eq!(runtime.allocated, MAX_HEAP + usize::from(shortage));
            assert_eq!(runtime.objects[object].boxed, Some(Value::String(text)));
            assert!(runtime.objects[object].values.is_empty());
            assert!(runtime.objects[object].order.is_empty());
            clean(&runtime);
        }
    }
}

#[test]
fn find_saved_values_keep_identity_without_result_coercion_or_deep_copy() {
    for method in FIND_METHODS {
        let (mut runtime, mut doc, target, callback) = setup(
            "var reads=0,target={length:1},truth={get valueOf(){reads++;throw 'valueOf';},get toString(){reads++;throw 'toString';},get then(){reads++;throw 'then';}};Object.defineProperty(truth,Symbol.toPrimitive,{get:function(){reads++;throw 'primitive';}});function predicate(v,k,o){o[k]='replacement';arguments[0]='changed';return truth;}target",
        );
        let token = JsString::from(vec![0xd800; 8_192]);
        let pointer = token.units().as_ptr();
        assert!(
            runtime
                .define_property_key(
                    &target,
                    &"0".into(),
                    PropertyDescriptor::data_property(
                        Value::String(token.clone()),
                        true,
                        true,
                        true
                    ),
                    &mut doc
                )
                .unwrap()
        );
        let result = runtime
            .array_method(target.clone(), method, vec![callback], &mut doc)
            .unwrap();
        assert_eq!(result, found(method, 0, Value::String(token)));
        if let Value::String(text) = result {
            assert_eq!(text.units().as_ptr(), pointer);
        }
        assert_eq!(runtime.lookup(1, "reads").unwrap().1, Value::Number(0.0));
        assert!(
            matches!(runtime.own_property(&target, &JsString::from("0")).unwrap().value, PropertyValue::Data { value: Value::String(text), .. } if text == JsString::from("replacement"))
        );
        clean(&runtime);
    }
}

#[test]
fn find_repeated_direct_calls_share_cumulative_heap_and_work() {
    for method in FIND_METHODS {
        for native in [false, true] {
            let source = "var calls=0,target={length:1,0:7};function predicate(){calls++;return true;}target";
            let (mut measure, mut doc, target, callback) = setup(source);
            let callback = if native {
                measure.lookup(1, "Boolean").unwrap().1
            } else {
                callback
            };
            let before = measure.allocated;
            assert_eq!(
                measure
                    .array_method(target, method, vec![callback], &mut doc)
                    .unwrap(),
                found(method, 0, Value::Number(7.0))
            );
            let one_call = measure.allocated - before;
            assert!(one_call > 0);

            let (mut runtime, mut doc, target, callback) = setup(source);
            let callback = if native {
                runtime.lookup(1, "Boolean").unwrap().1
            } else {
                callback
            };
            runtime.steps = MAX_STEPS;
            runtime.allocated = MAX_HEAP - 3 * one_call;
            let mut completed = 0;
            let mut previous = (runtime.steps, runtime.allocated);
            let mut stopped = false;
            for _ in 0..8 {
                match runtime.array_method(target.clone(), method, vec![callback.clone()], &mut doc)
                {
                    Ok(value) => {
                        assert_eq!(value, found(method, 0, Value::Number(7.0)));
                        completed += 1;
                    }
                    Err(error) => {
                        assert!(error.is_resource_limit());
                        stopped = true;
                    }
                }
                assert!(runtime.steps < previous.0);
                assert!(runtime.allocated > previous.1);
                previous = (runtime.steps, runtime.allocated);
                clean(&runtime);
                if stopped {
                    break;
                }
            }
            assert!(stopped);
            assert_eq!(completed, 3);
            if !native {
                assert_eq!(runtime.lookup(1, "calls").unwrap().1, Value::Number(3.0));
            }
        }
    }
}

#[test]
fn find_recursive_callbacks_and_hole_scans_have_uncatchable_shared_limits() {
    for method in FIND_METHODS {
        let guard = format!(
            "if([7].{method}(function(){{return true;}})!=={})throw 'missing method';",
            if method.contains("Index") { 0 } else { 7 }
        );
        for body in [
            format!(
                "var o={{get length(){{return Array.prototype.{method}.call(o,function(){{}});}}}};Array.prototype.{method}.call(o,function(){{}});"
            ),
            format!(
                "var o={{length:{{valueOf:function(){{return Array.prototype.{method}.call(o,function(){{}});}}}}}};Array.prototype.{method}.call(o,function(){{}});"
            ),
            format!(
                "var o={{length:1,get 0(){{return Array.prototype.{method}.call(o,function(){{}});}}}};Array.prototype.{method}.call(o,function(){{}});"
            ),
            format!(
                "function predicate(){{return [1].{method}(predicate);}}[1].{method}(predicate);"
            ),
            format!("[1].{method}(function(){{while(true){{}}}});"),
            format!(
                "Array.prototype.{method}.call({{length:Infinity}},function(){{visits++;return false;}});"
            ),
        ] {
            let (mut runtime, mut doc) = prepared();
            let source = format!(
                "{guard}var prior=0,caught=false,finalized=false,visits=0;try{{prior=7;{body}}}catch(e){{caught=true;}}finally{{finalized=true;}}"
            );
            assert!(
                runtime
                    .execute(&source, &mut doc)
                    .unwrap_err()
                    .is_resource_limit(),
                "{method}: {body}"
            );
            assert_eq!(runtime.lookup(1, "prior").unwrap().1, Value::Number(7.0));
            assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Bool(false));
            assert_eq!(
                runtime.lookup(1, "finalized").unwrap().1,
                Value::Bool(false)
            );
            if body.contains("visits++") {
                assert!(runtime.lookup(1, "visits").unwrap().1.as_number().unwrap() > 0.0);
            }
            clean(&runtime);
        }
        let (mut runtime, mut doc) = prepared();
        let result = runtime.execute(&format!("{guard}var reason={{}},caught;try{{Array.prototype.{method}.call({{length:Infinity}},function(){{throw reason;}});}}catch(e){{caught=e;}}caught===reason"), &mut doc).unwrap();
        assert_eq!(result, Value::Bool(true));
        clean(&runtime);
    }
}
