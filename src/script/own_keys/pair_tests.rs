//! Independent Single/Pair/Tree tests, authored before reading the candidate.
use super::*;

fn fresh() -> (Runtime, Document) {
    let mut runtime = Runtime::try_new().unwrap();
    runtime.steps = MAX_STEPS;
    (runtime, Document::parse("<p>kept</p>"))
}
fn object(runtime: &mut Runtime, names: &[JsString]) -> Value {
    runtime
        .object_ordered(names.iter().cloned().map(|key| (key, Value::Bool(true))))
        .unwrap()
}
fn reset(runtime: &mut Runtime) {
    runtime.steps = MAX_STEPS;
    runtime.allocated = 0;
}
fn used(runtime: &Runtime) -> (usize, usize) {
    (MAX_STEPS - runtime.steps, runtime.allocated)
}
fn retained<'a>(visited: &'a VisitedNames, key: &JsString) -> &'a JsString {
    match &visited.buckets[&key.len()] {
        NameBucket::Single(first) => {
            assert_eq!(first, key);
            first
        }
        NameBucket::Pair(pair) => pair.iter().find(|old| *old == key).unwrap(),
        NameBucket::Tree(tree) => tree.get_key_value(key).unwrap().0,
    }
}
fn no_reader(_: &mut Runtime, _: &Value, _: &JsString) -> Result<Option<Property>> {
    panic!("visited duplicate must not read a descriptor")
}
fn refused_reader(runtime: &mut Runtime, _: &Value, _: &JsString) -> Result<Option<Property>> {
    runtime.work(5)?;
    Err(ScriptError::resource("independent descriptor refusal"))
}
fn clean(runtime: &Runtime) {
    assert!(runtime.frames.is_empty());
    assert!(runtime.active_array_joins.is_empty());
    assert_eq!(
        (
            runtime.calls,
            runtime.eval_depth,
            runtime.stack_units,
            runtime.json_depth
        ),
        (0, 0, 0, 0)
    );
}
fn seed(runtime: &mut Runtime, target: &Value, keys: &[JsString]) -> VisitedNames {
    let mut visited = VisitedNames::default();
    for key in keys {
        runtime
            .for_in_visit(&mut visited, target, key)
            .unwrap()
            .unwrap();
    }
    visited
}
fn authored(source: &str) {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        assert_eq!(result.unwrap(), Value::Bool(true), "strict={strict}");
        clean(&runtime);
    }
}

#[test]
fn pair_first_empty_and_arbitrary_utf16_keep_original_payload() {
    for units in [
        vec![],
        vec![0],
        vec![0xd800],
        vec![0xdc00, 0xd800],
        vec![0xd800; 512],
    ] {
        let (mut runtime, _) = fresh();
        let key = JsString::from(units);
        let equal = JsString::from(key.units().to_vec());
        if !key.is_empty() {
            assert_ne!(key.units().as_ptr(), equal.units().as_ptr());
        }
        let target = object(&mut runtime, std::slice::from_ref(&key));
        let mut visited = VisitedNames::default();
        reset(&mut runtime);
        assert!(
            runtime
                .for_in_visit(&mut visited, &target, &key)
                .unwrap()
                .is_some()
        );
        assert_eq!(used(&runtime), (31, VISITED_BUCKET_BYTES));
        assert!(matches!(visited.buckets[&key.len()], NameBucket::Single(_)));
        let before = visited.clone();
        runtime.steps = key.len() + 7;
        runtime.allocated = MAX_HEAP;
        assert!(
            runtime
                .for_in_visit_using(&mut visited, &target, &equal, no_reader)
                .unwrap()
                .is_none()
        );
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, MAX_HEAP);
        assert_eq!(visited, before);
        assert_eq!(
            retained(&visited, &key).units().as_ptr(),
            key.units().as_ptr()
        );
        clean(&runtime);
    }
}

#[test]
fn pair_utf16_order_identity_and_third_name_promotion() {
    let rows = [
        vec![vec![0xd800], vec![0], vec![0xdc00], vec![0xfffd]],
        vec![vec![0xfffd], vec![0xdc00], vec![0], vec![0xd800]],
        vec![vec![0xd800, 0xdc00], vec![0xdc00, 0xd800], vec![0, 0]],
        vec![
            vec![0xd800; 512],
            [vec![0xd800; 511], vec![0xdc00]].concat(),
            [vec![0xd800; 511], vec![0xfffd]].concat(),
        ],
    ];
    for row in rows {
        let keys: Vec<JsString> = row.into_iter().map(JsString::from).collect();
        let (mut runtime, _) = fresh();
        let target = object(&mut runtime, &keys);
        let mut visited = VisitedNames::default();
        for (i, key) in keys.iter().enumerate() {
            reset(&mut runtime);
            assert!(
                runtime
                    .for_in_visit(&mut visited, &target, key)
                    .unwrap()
                    .is_some()
            );
            match i {
                0 => assert!(matches!(visited.buckets[&key.len()], NameBucket::Single(_))),
                1 => {
                    assert_eq!(used(&runtime), (15 + key.len(), 0));
                    let NameBucket::Pair(pair) = &visited.buckets[&key.len()] else {
                        panic!("second name must be inline")
                    };
                    assert_eq!(pair, &[keys[0].clone(), keys[1].clone()]);
                }
                2 => {
                    assert_eq!(used(&runtime), (96 + 5 * key.len(), VISITED_NAME_BYTES));
                    assert!(matches!(visited.buckets[&key.len()], NameBucket::Tree(_)));
                }
                _ => assert!(matches!(visited.buckets[&key.len()], NameBucket::Tree(_))),
            }
            for old in &keys[..=i] {
                assert_eq!(
                    retained(&visited, old).units().as_ptr(),
                    old.units().as_ptr()
                );
            }
        }
        clean(&runtime);
    }
}

#[test]
fn pair_duplicates_compare_only_reached_positions_without_reader_or_heap() {
    for length in [1, 2, 512] {
        let (mut runtime, _) = fresh();
        let keys = [
            JsString::from(vec![0xd800; length]),
            JsString::from(vec![0xdc00; length]),
        ];
        let target = object(&mut runtime, &keys);
        let before = seed(&mut runtime, &target, &keys);
        assert!(matches!(before.buckets[&length], NameBucket::Pair(_)));
        for (position, key) in keys.iter().enumerate() {
            let equal = JsString::from(key.units().to_vec());
            assert_ne!(equal.units().as_ptr(), key.units().as_ptr());
            let work = 6 + (position + 1) * (length + 1);
            for available in [work - 1, work] {
                let mut visited = before.clone();
                runtime.steps = available;
                runtime.allocated = MAX_HEAP;
                let result = runtime.for_in_visit_using(&mut visited, &target, &equal, no_reader);
                if available == work {
                    assert!(result.unwrap().is_none());
                } else {
                    assert!(result.unwrap_err().is_resource_limit());
                }
                assert_eq!(runtime.steps, 0);
                assert_eq!(runtime.allocated, MAX_HEAP);
                assert_eq!(visited, before);
                assert_eq!(
                    retained(&visited, key).units().as_ptr(),
                    key.units().as_ptr()
                );
            }
        }
        clean(&runtime);
    }
}

#[test]
fn pair_missing_and_refused_descriptors_never_publish_or_promote() {
    let (mut runtime, _) = fresh();
    let keys: Vec<JsString> = ["aa", "bb", "cc"].into_iter().map(JsString::from).collect();
    let target = object(&mut runtime, &keys);
    let missing = object(&mut runtime, &[]);
    for count in [0, 1, 2] {
        let before = seed(&mut runtime, &target, &keys[..count]);
        let key = &keys[count];
        let prefix = [2, 9, 12][count];
        let mut visited = before.clone();
        runtime.steps = prefix;
        runtime.allocated = MAX_HEAP;
        assert!(
            runtime
                .for_in_visit(&mut visited, &missing, key)
                .unwrap()
                .is_none()
        );
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, MAX_HEAP);
        assert_eq!(visited, before);
        runtime.steps = prefix + 5;
        assert!(
            runtime
                .for_in_visit_using(&mut visited, &target, key, refused_reader)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, MAX_HEAP);
        assert_eq!(visited, before);
        for old in &keys[..count] {
            assert_eq!(
                retained(&visited, old).units().as_ptr(),
                old.units().as_ptr()
            );
        }
        // A missing different length must not insert an empty outer bucket.
        runtime.steps = if count == 0 { 2 } else { 4 };
        assert!(
            runtime
                .for_in_visit(&mut visited, &missing, &"long".into())
                .unwrap()
                .is_none()
        );
        assert_eq!(runtime.steps, 0);
        assert_eq!(visited, before);
        reset(&mut runtime);
    }
    clean(&runtime);
}

#[test]
fn pair_second_name_all_work_cuts_need_no_heap_and_keep_single_on_failure() {
    for order in [["aa", "bb"], ["bb", "aa"]] {
        let (mut runtime, _) = fresh();
        let keys: Vec<JsString> = order.into_iter().map(JsString::from).collect();
        let target = object(&mut runtime, &keys);
        let before = seed(&mut runtime, &target, &keys[..1]);
        let id = runtime.property_object(&target).unwrap();
        let stored_order = runtime.objects[id].order.clone();
        let objects = runtime.objects.len();
        // 1 tick +3 outer +2 variant +3 equality +8 inline publication.
        for available in 0..=17 {
            let mut visited = before.clone();
            runtime.steps = available;
            runtime.allocated = MAX_HEAP;
            let result = runtime.for_in_visit(&mut visited, &target, &keys[1]);
            if available == 17 {
                assert!(result.unwrap().is_some());
                assert!(matches!(visited.buckets[&2], NameBucket::Pair(_)));
                assert_eq!(
                    retained(&visited, &keys[1]).units().as_ptr(),
                    keys[1].units().as_ptr()
                );
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert_eq!(visited, before);
            }
            assert_eq!(runtime.steps, 0);
            assert_eq!(runtime.allocated, MAX_HEAP);
            assert_eq!(
                retained(&visited, &keys[0]).units().as_ptr(),
                keys[0].units().as_ptr()
            );
            assert_eq!(runtime.objects.len(), objects);
            assert_eq!(runtime.objects[id].order, stored_order);
            clean(&runtime);
        }
    }
}

#[test]
fn pair_third_name_all_work_and_heap_cuts_preserve_both_original_payloads() {
    for order in [["aa", "bb", "cc"], ["cc", "aa", "bb"], ["cc", "bb", "aa"]] {
        let (mut runtime, _) = fresh();
        let keys: Vec<JsString> = order.into_iter().map(JsString::from).collect();
        let target = object(&mut runtime, &keys);
        let before = seed(&mut runtime, &target, &keys[..2]);
        // 1 tick +3 outer +2 variant +3+3 equality +94 full promotion.
        for available in 0..=106 {
            let mut visited = before.clone();
            runtime.steps = available;
            runtime.allocated = MAX_HEAP - VISITED_NAME_BYTES;
            let result = runtime.for_in_visit(&mut visited, &target, &keys[2]);
            if available == 106 {
                assert!(result.unwrap().is_some());
                assert!(matches!(visited.buckets[&2], NameBucket::Tree(_)));
                assert_eq!(runtime.allocated, MAX_HEAP);
                assert_eq!(
                    retained(&visited, &keys[2]).units().as_ptr(),
                    keys[2].units().as_ptr()
                );
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert_eq!(visited, before);
                assert_eq!(runtime.allocated, MAX_HEAP - VISITED_NAME_BYTES);
            }
            assert_eq!(runtime.steps, 0);
            for key in &keys[..2] {
                assert_eq!(
                    retained(&visited, key).units().as_ptr(),
                    key.units().as_ptr()
                );
            }
            clean(&runtime);
        }
        let mut visited = before.clone();
        runtime.steps = 106;
        runtime.allocated = MAX_HEAP - VISITED_NAME_BYTES + 1;
        assert!(
            runtime
                .for_in_visit(&mut visited, &target, &keys[2])
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, MAX_HEAP + 1);
        assert_eq!(visited, before);
        for key in &keys[..2] {
            assert_eq!(
                retained(&visited, key).units().as_ptr(),
                key.units().as_ptr()
            );
        }
        clean(&runtime);
    }
}

#[test]
fn pair_hidden_names_shadow_but_missing_names_remain_live_on_later_objects() {
    authored(
        r#"(function(){
      var tail=Object.create(null);tail.aa=1;tail.bb=2;tail.cc=3;tail.dd=4;
      var source=Object.create(tail),reads=0;source.start=1;
      Object.defineProperty(source,'aa',{get:function(){reads++;throw 'getter';},enumerable:false});
      Object.defineProperty(source,'bb',{value:2,enumerable:false});source.cc=9;
      var seen='';for(var key in source){seen+=key+'|';if(key==='start')delete source.cc;}
      return seen==='start|cc|dd|'&&reads===0;
    })()"#,
    );
    authored(
        r#"(function(){
      var tail=Object.create(null);tail.aa=1;tail.bb=2;tail.cc=3;
      var source=Object.create(tail);
      Object.defineProperty(source,'aa',{value:4,enumerable:false,configurable:true});
      Object.defineProperty(source,'bb',{value:5,enumerable:false,configurable:true});source.zz=6;
      var seen='';for(var key in source){seen+=key+'|';if(key==='zz'){
        delete source.bb;Object.defineProperty(source,'aa',{enumerable:true});
      }}
      return seen==='zz|cc|'&&source.aa===4&&!Object.hasOwn(source,'bb');
    })()"#,
    );
}

#[test]
fn pair_snapshot_keeps_new_prototype_and_typed_index_descriptors_fresh() {
    authored(
        r#"(function(){
      var old=Object.create(null);old.old=1;
      var source=Object.create(old);source.aa=1;source.bb=2;
      var next=Object.create(null);next.bb=9;next.cc=3;
      var seen='';for(var key in source){seen+=key+'|';if(key==='aa')Object.setPrototypeOf(source,next);}
      return seen==='aa|bb|cc|';
    })()"#,
    );
    authored(
        r#"(function(){
      var buffer=new ArrayBuffer(2,{maxByteLength:4}),view=new Uint8Array(buffer);view[0]=7;view[1]=9;
      var tail=Object.create(null);tail[1]=11;tail.x=1;tail.y=2;Object.setPrototypeOf(view,tail);
      var seen='';for(var key in view){seen+=key+'|';if(key==='0')buffer.resize(1);}
      return seen==='0|1|x|y|'&&view[0]===7&&buffer.byteLength===1;
    })()"#,
    );
}

#[test]
fn pair_general_tree_growth_keeps_original_search_and_admission_schedule() {
    // Literal bounds for length4 and one occupied outer bucket, including the
    // unchanged variant2, search and 24-per-level move envelopes.
    for (count, work) in [
        (3, 72),
        (10, 107),
        (11, 137),
        (12, 142),
        (70, 192),
        (71, 272),
        (72, 272),
    ] {
        let (mut runtime, _) = fresh();
        let keys: Vec<JsString> = (0..=count).map(|i| format!("p{i:03}").into()).collect();
        let target = object(&mut runtime, &keys);
        let before = seed(&mut runtime, &target, &keys[..count]);
        assert!(matches!(before.buckets[&4], NameBucket::Tree(_)));
        for available in [work - 1, work] {
            let mut visited = before.clone();
            runtime.steps = available;
            runtime.allocated = MAX_HEAP - VISITED_NAME_BYTES;
            let result = runtime.for_in_visit(&mut visited, &target, &keys[count]);
            if available == work {
                assert!(result.unwrap().is_some());
                assert_eq!(runtime.allocated, MAX_HEAP);
                assert_eq!(
                    retained(&visited, &keys[count]).units().as_ptr(),
                    keys[count].units().as_ptr()
                );
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert_eq!(visited, before);
                assert_eq!(runtime.allocated, MAX_HEAP - VISITED_NAME_BYTES);
            }
            assert_eq!(runtime.steps, 0);
            for old in &keys[..count] {
                assert_eq!(
                    retained(&visited, old).units().as_ptr(),
                    old.units().as_ptr()
                );
            }
            clean(&runtime);
        }
        let mut visited = before.clone();
        runtime.steps = work;
        runtime.allocated = MAX_HEAP - VISITED_NAME_BYTES + 1;
        assert!(
            runtime
                .for_in_visit(&mut visited, &target, &keys[count])
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(visited, before);
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, MAX_HEAP + 1);
    }
}

#[test]
fn pair_outer_buckets_and_cumulative_heap_remain_independent() {
    let (mut runtime, _) = fresh();
    let keys: Vec<JsString> = ["aa", "bb", "cc", "long"]
        .into_iter()
        .map(JsString::from)
        .collect();
    let target = object(&mut runtime, &keys);
    let mut visited = VisitedNames::default();
    reset(&mut runtime);
    runtime
        .for_in_visit(&mut visited, &target, &keys[0])
        .unwrap()
        .unwrap();
    assert_eq!(used(&runtime), (31, VISITED_BUCKET_BYTES));
    runtime.steps = 17;
    runtime.allocated = MAX_HEAP;
    runtime
        .for_in_visit(&mut visited, &target, &keys[1])
        .unwrap()
        .unwrap();
    assert_eq!(runtime.steps, 0);
    assert_eq!(runtime.allocated, MAX_HEAP);
    let before = visited.clone();
    runtime.steps = 106;
    assert!(
        runtime
            .for_in_visit(&mut visited, &target, &keys[2])
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(visited, before);
    runtime.steps = 57;
    runtime.allocated = MAX_HEAP - VISITED_BUCKET_BYTES;
    runtime
        .for_in_visit(&mut visited, &target, &keys[3])
        .unwrap()
        .unwrap();
    assert_eq!(runtime.steps, 0);
    assert_eq!(runtime.allocated, MAX_HEAP);
    assert!(matches!(visited.buckets[&2], NameBucket::Pair(_)));
    assert!(matches!(visited.buckets[&4], NameBucket::Single(_)));
    assert_eq!(visited.buckets.len(), 2);
    clean(&runtime);
}
