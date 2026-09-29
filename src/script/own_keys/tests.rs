use super::*;

fn fresh() -> (Runtime, Document) {
    (Runtime::new(), Document::parse("<p>kept</p>"))
}

fn clean(runtime: &Runtime) {
    assert_eq!(runtime.calls, 0);
    assert_eq!(runtime.stack_units, 0);
    assert_eq!(runtime.eval_depth, 0);
    assert_eq!(runtime.json_depth, 0);
    assert!(runtime.frames.is_empty());
    assert!(runtime.active_array_joins.is_empty());
}

fn ordinary(runtime: &mut Runtime, names: &[&str]) -> Value {
    runtime
        .object_ordered(names.iter().map(|name| ((*name).into(), Value::Bool(true))))
        .unwrap()
}

fn names(values: &[&str]) -> Vec<JsString> {
    values.iter().map(|value| (*value).into()).collect()
}

fn run(runtime: &mut Runtime, unit: &Rc<code::Unit>, doc: &mut Document) -> Result<Value> {
    match machine::evaluate_statements(runtime, unit, machine::ListOwner::Program, 1, doc)? {
        Flow::Normal(value) => Ok(value.unwrap_or(Value::Undefined)),
        _ => panic!("unexpected completion"),
    }
}

#[test]
fn own_keys_frozen_independent_fixture_preserves_all_86_modes() {
    let mut modes = 0;
    for fixture in include_str!("../../../tests/conformance/own-keys.js")
        .split("// CASE: ")
        .skip(1)
    {
        let (name, source) = fixture.split_once('\n').unwrap();
        for strict in [false, true] {
            let (mut runtime, mut doc) = fresh();
            for helper in [
                include_str!("../../../tests/upstream/test262/harness/sta.js"),
                include_str!("../../../tests/upstream/test262/harness/assert.js"),
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
    assert_eq!(modes, 86);
}

#[test]
fn own_keys_literal_numeric_creation_and_virtual_overlap_order() {
    let (mut runtime, mut doc) = fresh();
    let array = runtime
        .execute(
            r#"
        var a=[0,1,2];delete a[1];
        Object.defineProperty(a,'0',{get:function(){throw 'value';},configurable:true});
        Object.defineProperty(a,'1',{value:7,enumerable:false,configurable:true});
        a.z=1;a[4294967294]=8;a['4294967295']=9;a['01']=10;a.x=1;
        delete a.z;a.z=2;a[Symbol('s')]=3;a;
    "#,
            &mut doc,
        )
        .unwrap();
    assert_eq!(
        runtime.own_keys(&array).unwrap(),
        names(&[
            "0",
            "1",
            "2",
            "4294967294",
            "length",
            "4294967295",
            "01",
            "x",
            "z"
        ])
    );
    let string = runtime
        .execute(
            r#"
        var s=Object('\uD800x');
        Object.defineProperty(s,'0',{value:'\uD800'});
        Object.defineProperty(s,'length',{value:2});
        s.z=1;s[10]=10;s[2]=2;s['01']=3;s;
    "#,
            &mut doc,
        )
        .unwrap();
    assert_eq!(
        runtime.own_keys(&string).unwrap(),
        names(&["0", "1", "2", "10", "length", "z", "01"])
    );
    clean(&runtime);
}

#[test]
fn own_keys_long_names_clone_handles_without_text_dependent_cost() {
    let mut costs = Vec::new();
    for length in [3, 512, 4096] {
        let (mut runtime, _) = fresh();
        let key = JsString::from(vec![0xd800; length]);
        let suffix = JsString::from(vec![0xdc00; length]);
        let target = runtime
            .object_ordered([
                (key.clone(), Value::Bool(true)),
                (suffix.clone(), Value::Bool(false)),
            ])
            .unwrap();
        let before = (runtime.steps, runtime.allocated);
        let result = runtime.own_keys(&target).unwrap();
        costs.push((before.0 - runtime.steps, runtime.allocated - before.1));
        assert_eq!(result, vec![key.clone(), suffix]);
        assert_eq!(result[0].units().as_ptr(), key.units().as_ptr());
    }
    assert_eq!(costs[0], costs[1]);
    assert_eq!(costs[1], costs[2]);
}

#[test]
fn own_keys_snapshot_work_and_heap_boundaries_preserve_input() {
    let prepare = || {
        let (mut runtime, doc) = fresh();
        let target = ordinary(&mut runtime, &["tail", "8", "2", "01", "1"]);
        (runtime, doc, target)
    };
    let (mut measured, _, target) = prepare();
    let before = (measured.steps, measured.allocated);
    let expected = measured.own_keys(&target).unwrap();
    assert_eq!(expected, names(&["1", "2", "8", "tail", "01"]));
    let work = before.0 - measured.steps;
    let heap = measured.allocated - before.1;
    for available in [0, 1, work / 2, work - 1, work] {
        let (mut runtime, _, target) = prepare();
        let id = runtime.property_object(&target).unwrap();
        let order = runtime.objects[id].order.clone();
        let arenas = (runtime.objects.len(), runtime.arrays.len());
        runtime.steps = available;
        let result = runtime.own_keys(&target);
        if available == work {
            assert_eq!(result.unwrap(), expected);
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        assert_eq!(runtime.objects[id].order, order);
        assert_eq!(runtime.objects[id].values.len(), order.len());
        assert_eq!((runtime.objects.len(), runtime.arrays.len()), arenas);
        clean(&runtime);
    }
    for available in [heap - 1, heap] {
        let (mut runtime, _, target) = prepare();
        let arenas = (runtime.objects.len(), runtime.arrays.len());
        runtime.allocated = MAX_HEAP - available;
        let result = runtime.own_keys(&target);
        if available == heap {
            assert_eq!(result.unwrap(), expected);
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        assert_eq!((runtime.objects.len(), runtime.arrays.len()), arenas);
    }
}

#[test]
fn own_keys_integer_sort_skips_ordered_input_and_interrupts_safely() {
    let prepare = |reversed: bool| {
        let (mut runtime, _) = fresh();
        let indices: Vec<_> = if reversed {
            (0..32).rev().collect()
        } else {
            (0..32).collect()
        };
        let target = runtime
            .object_ordered(
                indices
                    .into_iter()
                    .map(|i| (i.to_string().into(), Value::Number(i as f64))),
            )
            .unwrap();
        (runtime, target)
    };
    let (mut runtime, target) = prepare(false);
    let before = runtime.steps;
    let expected = runtime.own_keys(&target).unwrap();
    let ordered_work = before - runtime.steps;
    let (mut runtime, target) = prepare(true);
    let before = runtime.steps;
    assert_eq!(runtime.own_keys(&target).unwrap(), expected);
    let unordered_work = before - runtime.steps;
    assert!(unordered_work > ordered_work);
    let (mut runtime, target) = prepare(true);
    let id = runtime.property_object(&target).unwrap();
    let order = runtime.objects[id].order.clone();
    runtime.steps = unordered_work - 1;
    assert!(runtime.own_keys(&target).unwrap_err().is_resource_limit());
    assert_eq!(runtime.objects[id].order, order);
    runtime.steps = unordered_work;
    assert_eq!(runtime.own_keys(&target).unwrap(), expected);
}

#[test]
fn own_keys_sparse_max_length_costs_only_retained_shape() {
    let mut costs = Vec::new();
    for length in [3, u32::MAX] {
        let (mut runtime, mut doc) = fresh();
        let target = runtime.array(Vec::new()).unwrap();
        runtime
            .define_property_key(
                &target,
                &"2".into(),
                PropertyDescriptor::data_property(Value::Number(8.0), true, true, true),
                &mut doc,
            )
            .unwrap();
        let Value::Array(id) = target else {
            panic!("array")
        };
        runtime.array_lengths[id].value = length;
        assert!(runtime.arrays[id].is_empty());
        let before = (runtime.steps, runtime.allocated);
        assert_eq!(runtime.own_keys(&target).unwrap(), names(&["2", "length"]));
        costs.push((before.0 - runtime.steps, runtime.allocated - before.1));
        assert_eq!(runtime.array_lengths[id].value, length);
    }
    assert_eq!(costs[0], costs[1]);
}

#[test]
fn own_keys_visited_search_accounts_for_text_and_tree_height() {
    let mut costs = Vec::new();
    for (count, width) in [(1, 16), (12, 16), (72, 16), (72, 512)] {
        let prefix = "p".repeat(width);
        let mut visited: BTreeMap<JsString, ()> = (0..count)
            .map(|i| (format!("{prefix}{i:04}").into(), ()))
            .collect();
        let key: JsString = format!("{prefix}zzzz").into();
        let (mut runtime, _) = fresh();
        let missing = ordinary(&mut runtime, &[]);
        let before = runtime.steps;
        assert!(
            runtime
                .for_in_visit(&mut visited, &missing, &key)
                .unwrap()
                .is_none()
        );
        let cost = before - runtime.steps;
        costs.push(cost);
        runtime.steps = cost - 1;
        assert!(
            runtime
                .for_in_visit(&mut visited, &missing, &key)
                .unwrap_err()
                .is_resource_limit()
        );
        runtime.steps = cost;
        let present = visited.first_key_value().unwrap().0.clone();
        assert!(
            runtime
                .for_in_visit(&mut visited, &missing, &present)
                .unwrap()
                .is_none()
        );
        assert_eq!(visited.len(), count);
    }
    assert!(costs.windows(2).all(|pair| pair[1] > pair[0]));
    let (mut runtime, _) = fresh();
    let missing = ordinary(&mut runtime, &[]);
    runtime.allocated = MAX_HEAP;
    runtime.steps = 1;
    assert!(
        runtime
            .for_in_visit(&mut BTreeMap::new(), &missing, &"long".repeat(4096).into())
            .unwrap()
            .is_none()
    );
    assert_eq!(runtime.allocated, MAX_HEAP);
    assert_eq!(runtime.steps, 0);
}

#[test]
fn own_keys_visited_insertion_is_prepaid_and_keeps_text_identity() {
    let key: JsString = "x".repeat(1024).into();
    let seed: BTreeMap<JsString, ()> = [("a".into(), ()), ("b".into(), ())].into();
    let prepare = || {
        let (mut runtime, doc) = fresh();
        let target = runtime
            .object_ordered([(key.clone(), Value::Bool(true))])
            .unwrap();
        (runtime, doc, target)
    };
    let (mut runtime, _, target) = prepare();
    let mut visited = seed.clone();
    let before = (runtime.steps, runtime.allocated);
    assert!(
        runtime
            .for_in_visit(&mut visited, &target, &key)
            .unwrap()
            .is_some()
    );
    let cost = (before.0 - runtime.steps, runtime.allocated - before.1);
    assert_eq!(
        visited.get_key_value(&key).unwrap().0.units().as_ptr(),
        key.units().as_ptr()
    );
    for heap_failure in [false, true] {
        let (mut runtime, _, target) = prepare();
        let mut visited = seed.clone();
        if heap_failure {
            runtime.allocated = MAX_HEAP - cost.1 + 1;
        } else {
            runtime.steps = cost.0 - 1;
        }
        assert!(
            runtime
                .for_in_visit(&mut visited, &target, &key)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(visited, seed);
    }
}

#[test]
fn own_keys_visited_occupied_and_missing_do_not_allocate_or_insert() {
    for (populated, occupied) in [(false, false), (true, false), (true, true)] {
        let (mut runtime, _) = fresh();
        let target = ordinary(&mut runtime, if occupied { &["key"] } else { &[] });
        let mut visited: BTreeMap<JsString, ()> = if populated {
            [(JsString::from(if occupied { "key" } else { "other" }), ())].into()
        } else {
            BTreeMap::new()
        };
        let original = visited.clone();
        runtime.allocated = MAX_HEAP;
        assert!(
            runtime
                .for_in_visit(&mut visited, &target, &"key".into())
                .unwrap()
                .is_none()
        );
        assert_eq!(runtime.allocated, MAX_HEAP);
        assert_eq!(visited, original);
        clean(&runtime);
    }
}

#[test]
fn own_keys_visited_vacant_hidden_descriptor_pays_before_cached_insertion() {
    let prepare = || {
        let (mut runtime, doc) = fresh();
        let target = ordinary(&mut runtime, &["hidden"]);
        let id = runtime.property_object(&target).unwrap();
        runtime.objects[id].attributes("hidden", true, false, true);
        let missing = ordinary(&mut runtime, &[]);
        let visited: BTreeMap<JsString, ()> = [(JsString::from("already"), ())].into();
        (runtime, doc, target, missing, visited)
    };
    let key = JsString::from("hidden");
    let (mut measure, _, _, missing, mut visited) = prepare();
    let before = measure.steps;
    assert!(
        measure
            .for_in_visit(&mut visited, &missing, &key)
            .unwrap()
            .is_none()
    );
    let search = before - measure.steps;
    for heap_failure in [false, true] {
        let (mut runtime, _, target, _, mut visited) = prepare();
        let original = visited.clone();
        if heap_failure {
            runtime.allocated = MAX_HEAP;
        } else {
            runtime.steps = search;
        }
        assert!(
            runtime
                .for_in_visit(&mut visited, &target, &key)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(visited, original);
        assert!(!runtime.own_property(&target, &key).unwrap().enumerable);
        clean(&runtime);
    }
    let (mut runtime, _, target, _, mut visited) = prepare();
    let property = runtime
        .for_in_visit(&mut visited, &target, &key)
        .unwrap()
        .unwrap();
    assert!(!property.enumerable);
    assert!(visited.contains_key(&key));
    runtime.allocated = MAX_HEAP;
    assert!(
        runtime
            .for_in_visit(&mut visited, &target, &key)
            .unwrap()
            .is_none()
    );
}

#[test]
fn own_keys_caller_buffers_reserve_before_push_and_check_overflow() {
    let (mut runtime, _) = fresh();
    runtime.allocated = MAX_HEAP;
    assert!(runtime.own_key_values(0).unwrap().is_empty());
    assert!(runtime.own_key_descriptors(0).unwrap().is_empty());
    for descriptors in [false, true] {
        let bytes = 7 * if descriptors {
            std::mem::size_of::<(PropertyKey, PropertyDescriptor)>()
        } else {
            std::mem::size_of::<Value>()
        };
        for allowance in [bytes - 1, bytes] {
            let (mut runtime, _) = fresh();
            runtime.allocated = MAX_HEAP - allowance;
            let result = if descriptors {
                runtime.own_key_descriptors(7).map(|v| v.capacity())
            } else {
                runtime.own_key_values(7).map(|v| v.capacity())
            };
            if allowance == bytes {
                assert!(result.unwrap() >= 7);
            } else {
                assert!(result.unwrap_err().is_resource_limit());
            }
        }
    }
    let (mut runtime, _) = fresh();
    let arenas = (runtime.objects.len(), runtime.arrays.len());
    assert!(
        runtime
            .own_key_values(usize::MAX)
            .unwrap_err()
            .is_resource_limit()
    );
    assert!(
        runtime
            .own_key_descriptors(usize::MAX)
            .err()
            .unwrap()
            .is_resource_limit()
    );
    assert_eq!((runtime.objects.len(), runtime.arrays.len()), arenas);
}

#[test]
fn own_keys_values_capacity_failure_precedes_first_getter() {
    let setup = |runtime: &mut Runtime, doc: &mut Document| {
        runtime
            .execute(
                "var reads=0;var source={get x(){reads++;return 7;}};source",
                doc,
            )
            .unwrap()
    };
    let (mut measured, mut doc) = fresh();
    let source = setup(&mut measured, &mut doc);
    let before = measured.allocated;
    measured.own_keys(&source).unwrap();
    let snapshot = measured.allocated - before;
    let (mut runtime, mut doc) = fresh();
    let source = setup(&mut runtime, &mut doc);
    let object = runtime.lookup(1, "Object").unwrap().1;
    let Value::Native(native) = runtime.get(object, "values", &mut doc).unwrap() else {
        panic!("native")
    };
    let arrays = runtime.arrays.len();
    runtime.allocated = MAX_HEAP - snapshot - std::mem::size_of::<Value>() + 1;
    assert!(
        runtime
            .native_call(&native, vec![source], &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.lookup(1, "reads").unwrap().1, Value::Number(0.0));
    assert_eq!(runtime.arrays.len(), arrays);
    clean(&runtime);
}

#[test]
fn own_keys_descriptor_capacity_failure_precedes_collection_and_writes() {
    let setup = |runtime: &mut Runtime, doc: &mut Document| {
        runtime
            .execute(
                "var reads=0;var source={get x(){reads++;return {value:7};}};source",
                doc,
            )
            .unwrap()
    };
    let (mut measured, mut doc) = fresh();
    let source = setup(&mut measured, &mut doc);
    let before = measured.allocated;
    measured.own_property_keys(&source).unwrap();
    let snapshot = measured.allocated - before;
    let (mut runtime, mut doc) = fresh();
    let source = setup(&mut runtime, &mut doc);
    let target = ordinary(&mut runtime, &[]);
    runtime.allocated =
        MAX_HEAP - snapshot - std::mem::size_of::<(PropertyKey, PropertyDescriptor)>() + 1;
    assert!(
        runtime
            .define_properties(target.clone(), source, &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.lookup(1, "reads").unwrap().1, Value::Number(0.0));
    assert!(
        runtime.objects[runtime.property_object(&target).unwrap()]
            .order
            .is_empty()
    );
    clean(&runtime);
}

#[test]
fn own_keys_shrink_snapshot_refusal_keeps_conversion_effects_without_deletion() {
    let prepare = || {
        let (mut runtime, mut doc) = fresh();
        runtime.execute("var calls=0;var a=[0,1,2,3];var size={valueOf:function(){calls++;a.note=7;return 1;}}", &mut doc).unwrap();
        let array = runtime.lookup(1, "a").unwrap().1;
        let value = runtime.lookup(1, "size").unwrap().1;
        (runtime, doc, array, value)
    };
    let (mut measured, mut doc, array, value) = prepare();
    let before = measured.allocated;
    measured.number_value(value.clone(), &mut doc).unwrap();
    measured.number_value(value, &mut doc).unwrap();
    measured.own_keys(&array).unwrap();
    let needed = measured.allocated - before;
    let (mut runtime, mut doc, array, value) = prepare();
    runtime.allocated = MAX_HEAP - needed + 1;
    assert!(
        runtime
            .define_property_key(
                &array,
                &"length".into(),
                PropertyDescriptor {
                    value: Some(value),
                    ..PropertyDescriptor::default()
                },
                &mut doc
            )
            .unwrap_err()
            .is_resource_limit()
    );
    let Value::Array(id) = array else {
        panic!("array")
    };
    assert_eq!(runtime.lookup(1, "calls").unwrap().1, Value::Number(2.0));
    assert_eq!(runtime.array_lengths[id].value, 4);
    assert_eq!(runtime.arrays[id].len(), 4);
    assert!(runtime.array_holes[id].is_empty());
    assert!(runtime.own_property(&array, &"note".into()).is_some());
    clean(&runtime);
}

#[test]
fn own_keys_for_in_keeps_future_prototype_work_lazy() {
    let (mut runtime, mut doc) = fresh();
    let parent = runtime
        .object_ordered(
            (0..512).map(|i| (format!("{}{i}", "p".repeat(256)).into(), Value::Bool(true))),
        )
        .unwrap();
    let target = ordinary(&mut runtime, &["first"]);
    let id = runtime.property_object(&target).unwrap();
    runtime.objects[id].prototype = Some(parent);
    runtime.define(1, "source", target, true).unwrap();
    assert_eq!(
        runtime
            .execute(
                "var seen='';for(var k in source){seen=k;break;}seen",
                &mut doc
            )
            .unwrap(),
        Value::String("first".into())
    );
    clean(&runtime);
}

#[test]
fn own_keys_for_in_resource_cuts_preserve_prior_bodies_and_cleanup() {
    let unit = parser::Parser::program(
        "try{for(box.key in source){seen++;}}catch(e){caught++;}finally{finalized++;}seen",
    )
    .unwrap();
    let prepare = || {
        let (mut runtime, mut doc) = fresh();
        runtime.execute("var seen=0,assigned=0,caught=0,finalized=0;var source={a:1,b:2,c:3};var box={set key(v){assigned++;}}", &mut doc).unwrap();
        runtime.steps = MAX_STEPS;
        (runtime, doc)
    };
    let (mut measured, mut doc) = prepare();
    assert_eq!(
        run(&mut measured, &unit, &mut doc).unwrap(),
        Value::Number(3.0)
    );
    let used = MAX_STEPS - measured.steps;
    let mut prior_effect_stop = false;
    for allowance in (0..used).step_by(7) {
        let (mut runtime, mut doc) = prepare();
        runtime.steps = allowance;
        assert!(
            run(&mut runtime, &unit, &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Number(0.0));
        let Value::Number(seen) = runtime.lookup(1, "seen").unwrap().1 else {
            panic!("seen")
        };
        if seen < 3.0 {
            assert_eq!(
                runtime.lookup(1, "finalized").unwrap().1,
                Value::Number(0.0)
            );
            if seen > 0.0 {
                prior_effect_stop = true;
            }
        }
        clean(&runtime);
    }
    assert!(prior_effect_stop);
}

#[test]
fn own_keys_for_in_host_and_valid_id_cycle_stop_at_reached_boundary() {
    for host in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let target = ordinary(&mut runtime, &["first"]);
        let id = runtime.property_object(&target).unwrap();
        runtime.objects[id].prototype = Some(if host {
            Value::Document
        } else {
            target.clone()
        });
        runtime.define(1, "source", target, true).unwrap();
        let error = runtime.execute("var seen=0,caught=0,finalized=0;try{for(var k in source){seen++;}}catch(e){caught++;}finally{finalized++;}", &mut doc).unwrap_err();
        if host {
            assert!(error.is_unsupported());
        } else {
            assert!(error.is_resource_limit());
        }
        assert_eq!(runtime.lookup(1, "seen").unwrap().1, Value::Number(1.0));
        assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Number(0.0));
        assert_eq!(
            runtime.lookup(1, "finalized").unwrap().1,
            Value::Number(0.0)
        );
        clean(&runtime);
    }
}

#[test]
fn own_keys_repeated_snapshots_and_visited_sets_share_cumulative_heap() {
    let (mut runtime, _) = fresh();
    let target = ordinary(&mut runtime, &["a", "b", "c"]);
    let before = runtime.allocated;
    runtime.own_keys(&target).unwrap();
    let heap = runtime.allocated - before;
    runtime.allocated = MAX_HEAP - 2 * heap;
    runtime.steps = MAX_STEPS;
    runtime.own_keys(&target).unwrap();
    runtime.own_keys(&target).unwrap();
    assert_eq!(runtime.allocated, MAX_HEAP);
    assert!(runtime.own_keys(&target).unwrap_err().is_resource_limit());
    assert!(runtime.steps < MAX_STEPS);
    let (mut runtime, _) = fresh();
    let target = ordinary(&mut runtime, &["a", "b", "c"]);
    let mut visited = BTreeMap::new();
    let before = runtime.allocated;
    runtime
        .for_in_visit(&mut visited, &target, &"a".into())
        .unwrap()
        .unwrap();
    let heap = runtime.allocated - before;
    runtime.allocated = MAX_HEAP - heap;
    runtime
        .for_in_visit(&mut visited, &target, &"b".into())
        .unwrap()
        .unwrap();
    assert!(
        runtime
            .for_in_visit(&mut visited, &target, &"c".into())
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(visited.into_keys().collect::<Vec<_>>(), names(&["a", "b"]));
    clean(&runtime);
}

#[test]
fn own_keys_number_constant_controls_keep_original_success_and_failure() {
    let positive = r#"assert.sameValue(typeof Number.EPSILON,'number');assert.sameValue(Number.EPSILON,2.220446049250313e-16);assert.sameValue(typeof Number.MAX_SAFE_INTEGER,'number');assert.sameValue(Number.MAX_SAFE_INTEGER,9007199254740991);assert.sameValue(typeof Number.MIN_SAFE_INTEGER,'number');assert.sameValue(Number.MIN_SAFE_INTEGER,-9007199254740991);assert.sameValue(typeof Number.MAX_VALUE,'number');assert.sameValue(Number.MAX_VALUE,1.7976931348623157e308);assert.sameValue(typeof Number.MIN_VALUE,'number');assert.sameValue(Number.MIN_VALUE,5e-324);assert.sameValue(typeof Number.NaN,'number');assert.sameValue(Number.NaN,NaN);assert.sameValue(typeof Number.NEGATIVE_INFINITY,'number');assert.sameValue(Number.NEGATIVE_INFINITY,-Infinity);assert.sameValue(typeof Number.POSITIVE_INFINITY,'number');assert.sameValue(Number.POSITIVE_INFINITY,Infinity);verifyProperty(Number,'EPSILON',{value:2.220446049250313e-16,writable:false,enumerable:false,configurable:false},{restore:true});verifyProperty(Number,'MAX_SAFE_INTEGER',{value:9007199254740991,writable:false,enumerable:false,configurable:false},{restore:true});verifyProperty(Number,'MIN_SAFE_INTEGER',{value:-9007199254740991,writable:false,enumerable:false,configurable:false},{restore:true});verifyProperty(Number,'MAX_VALUE',{value:1.7976931348623157e308,writable:false,enumerable:false,configurable:false},{restore:true});verifyProperty(Number,'MIN_VALUE',{value:5e-324,writable:false,enumerable:false,configurable:false},{restore:true});verifyProperty(Number,'NaN',{value:NaN,writable:false,enumerable:false,configurable:false},{restore:true});verifyProperty(Number,'NEGATIVE_INFINITY',{value:-Infinity,writable:false,enumerable:false,configurable:false},{restore:true});verifyProperty(Number,'POSITIVE_INFINITY',{value:Infinity,writable:false,enumerable:false,configurable:false},{restore:true});assert.sameValue(Number.MAX_SAFE_INTEGER,9007199254740991);"#;
    let negative = r#"assert.sameValue(typeof Number.EPSILON,'number');assert.sameValue(Number.EPSILON,2.220446049250313e-16);assert.sameValue(typeof Number.MAX_SAFE_INTEGER,'number');assert.sameValue(Number.MAX_SAFE_INTEGER,9007199254740991);assert.sameValue(typeof Number.MIN_SAFE_INTEGER,'number');assert.sameValue(Number.MIN_SAFE_INTEGER,-9007199254740991);assert.sameValue(typeof Number.MAX_VALUE,'number');assert.sameValue(Number.MAX_VALUE,1.7976931348623157e308);assert.sameValue(typeof Number.MIN_VALUE,'number');assert.sameValue(Number.MIN_VALUE,5e-324);assert.sameValue(typeof Number.NaN,'number');assert.sameValue(Number.NaN,NaN);assert.sameValue(typeof Number.NEGATIVE_INFINITY,'number');assert.sameValue(Number.NEGATIVE_INFINITY,-Infinity);assert.sameValue(typeof Number.POSITIVE_INFINITY,'number');assert.sameValue(Number.POSITIVE_INFINITY,Infinity);verifyProperty(Number,'EPSILON',{value:2.220446049250313e-16,writable:false,enumerable:false,configurable:false},{restore:true});verifyProperty(Number,'MAX_SAFE_INTEGER',{value:9007199254740991,writable:false,enumerable:false,configurable:false},{restore:true});verifyProperty(Number,'MIN_SAFE_INTEGER',{value:-9007199254740991,writable:false,enumerable:false,configurable:false},{restore:true});verifyProperty(Number,'MAX_VALUE',{value:1.7976931348623157e308,writable:false,enumerable:false,configurable:false},{restore:true});verifyProperty(Number,'MIN_VALUE',{value:5e-324,writable:false,enumerable:false,configurable:false},{restore:true});verifyProperty(Number,'NaN',{value:NaN,writable:false,enumerable:false,configurable:false},{restore:true});verifyProperty(Number,'NEGATIVE_INFINITY',{value:-Infinity,writable:false,enumerable:false,configurable:false},{restore:true});verifyProperty(Number,'POSITIVE_INFINITY',{value:Infinity,writable:false,enumerable:false,configurable:false},{restore:true});assert.sameValue(Number.MAX_SAFE_INTEGER,0);"#;
    let mut failures = Vec::new();
    for strict in [false, true] {
        for (source, positive) in [(positive, true), (negative, false)] {
            let (mut runtime, mut doc) = fresh();
            for helper in [
                include_str!("../../../tests/upstream/test262/harness/assert.js"),
                include_str!("../../../tests/upstream/test262/harness/sta.js"),
                include_str!("../../../tests/upstream/test262/harness/propertyHelper.js"),
            ] {
                runtime.execute(helper, &mut doc).unwrap();
            }
            let result = if strict {
                runtime.execute_strict(source, &mut doc)
            } else {
                runtime.execute(source, &mut doc)
            };
            eprintln!(
                "constant-control strict={strict} positive={positive} remaining={} result={result:?}",
                runtime.steps
            );
            let matched = if positive {
                result.is_ok()
            } else {
                result
                    .as_ref()
                    .is_err_and(|error| error.name() == "Test262Error")
            };
            if !matched {
                failures.push((strict, positive, result));
            }
            clean(&runtime);
        }
    }
    assert!(failures.is_empty(), "{failures:?}");
}
