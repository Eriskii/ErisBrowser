//! Literal admission cuts for two inline visited names and third-name promotion.
use super::*;

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

fn keys(length: usize) -> [JsString; 4] {
    [0xd800, 0xd801, 0xdc00, 0xffff].map(|last| {
        let mut units = vec![0xd800; length];
        units[length - 1] = last;
        JsString::from(units)
    })
}

fn seed(keys: &[JsString], count: usize) -> VisitedNames {
    let bucket = match count {
        1 => NameBucket::Single(keys[0].clone()),
        2 => NameBucket::Pair([keys[0].clone(), keys[1].clone()]),
        3 => NameBucket::Tree(keys[..3].iter().cloned().map(|key| (key, ())).collect()),
        _ => panic!("literal seed"),
    };
    VisitedNames {
        buckets: [(keys[0].len(), bucket)].into_iter().collect(),
    }
}

fn target(runtime: &mut Runtime, keys: &[JsString]) -> Value {
    runtime
        .object_ordered(keys.iter().cloned().map(|key| (key, Value::Bool(true))))
        .unwrap()
}

fn forbidden(_: &mut Runtime, _: &Value, _: &JsString) -> Result<Option<Property>> {
    panic!("duplicate read a descriptor")
}

#[test]
fn pair_literal_every_work_cut_preserves_source_set_and_handle_identity() {
    // Values are independent of production helpers: second15+L, third96+5L,
    // and unchanged three-key tree insertion60+3L. Reader has no work debit.
    for (length, second, third, fourth) in [(1, 16, 101, 63), (2, 17, 106, 66), (33, 48, 261, 159)]
    {
        for reverse in [false, true] {
            let mut names = keys(length);
            if reverse {
                names.reverse();
            }
            let mut runtime = Runtime::try_new().unwrap();
            let object = target(&mut runtime, &names);
            let objects = runtime.objects.len();
            for (count, work, heap) in [(1, second, 0), (2, third, 576), (3, fourth, 576)] {
                let original = seed(&names, count);
                for allowance in 0..=work {
                    let mut visited = original.clone();
                    runtime.steps = allowance;
                    runtime.allocated = MAX_HEAP - heap;
                    let result = runtime.for_in_visit(&mut visited, &object, &names[count]);
                    if allowance == work {
                        assert!(result.unwrap().is_some());
                        assert_eq!(runtime.steps, 0);
                        assert_eq!(runtime.allocated, MAX_HEAP);
                        assert_eq!(
                            visited.buckets[&length]
                                .get_key_value(&names[count])
                                .unwrap()
                                .0
                                .units()
                                .as_ptr(),
                            names[count].units().as_ptr()
                        );
                    } else {
                        assert!(result.unwrap_err().is_resource_limit());
                        assert_eq!(visited, original);
                        assert_eq!(runtime.allocated, MAX_HEAP - heap);
                    }
                    for key in &names[..count] {
                        assert_eq!(
                            visited.buckets[&length]
                                .get_key_value(key)
                                .unwrap()
                                .0
                                .units()
                                .as_ptr(),
                            key.units().as_ptr()
                        );
                    }
                    assert_eq!(runtime.objects.len(), objects);
                    clean(&runtime);
                }
            }
        }
    }
}

#[test]
fn pair_duplicate_cuts_stop_before_reader_and_never_require_heap() {
    let names = keys(2);
    let mut runtime = Runtime::try_new().unwrap();
    let original = seed(&names, 2);
    for (index, work) in [(0, 9), (1, 12)] {
        // A distinct Rc with equal code units must preserve the first identity.
        let key = JsString::from(names[index].units().to_vec());
        for allowance in 0..=work {
            let mut visited = original.clone();
            runtime.steps = allowance;
            runtime.allocated = MAX_HEAP;
            let result =
                runtime.for_in_visit_using(&mut visited, &Value::Undefined, &key, forbidden);
            if allowance == work {
                assert!(result.unwrap().is_none());
            } else {
                assert!(result.unwrap_err().is_resource_limit());
            }
            assert_eq!(runtime.steps, 0);
            assert_eq!(runtime.allocated, MAX_HEAP);
            assert_eq!(visited, original);
            assert_eq!(
                visited.buckets[&2]
                    .get_key_value(&key)
                    .unwrap()
                    .0
                    .units()
                    .as_ptr(),
                names[index].units().as_ptr()
            );
            clean(&runtime);
        }
    }
}

#[test]
fn pair_storage_is_charged_at_outer_admission_and_third_name_only() {
    assert_eq!(std::mem::size_of::<JsString>(), 16);
    assert_eq!(std::mem::size_of::<NameBucket>(), 40);
    assert_eq!(std::mem::size_of::<VisitedNames>(), 24);
    assert_eq!(VISITED_BUCKET_BYTES, 1088);
    assert_eq!(VISITED_NAME_BYTES, 576);
    let names = keys(2);
    let mut runtime = Runtime::try_new().unwrap();
    let object = target(&mut runtime, &names);
    for room in [0, 1087, 1088] {
        let mut visited = VisitedNames::default();
        runtime.steps = 31;
        runtime.allocated = MAX_HEAP - room;
        let result = runtime.for_in_visit(&mut visited, &object, &names[0]);
        if room == 1088 {
            assert!(result.unwrap().is_some());
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            assert!(visited.buckets.is_empty());
        }
        assert_eq!(runtime.allocated, MAX_HEAP - room + 1088);
    }
    let mut visited = seed(&names, 1);
    runtime.steps = 17;
    runtime.allocated = MAX_HEAP;
    runtime
        .for_in_visit(&mut visited, &object, &names[1])
        .unwrap()
        .unwrap();
    assert_eq!(runtime.steps, 0);
    assert_eq!(runtime.allocated, MAX_HEAP);
    assert!(matches!(visited.buckets[&2], NameBucket::Pair(_)));
    let original = visited;
    for room in [0, 575, 576] {
        let mut visited = original.clone();
        runtime.steps = 106;
        runtime.allocated = MAX_HEAP - room;
        let result = runtime.for_in_visit(&mut visited, &object, &names[2]);
        if room == 576 {
            assert!(result.unwrap().is_some());
            assert!(matches!(visited.buckets[&2], NameBucket::Tree(_)));
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            assert_eq!(visited, original);
        }
        assert_eq!(runtime.allocated, MAX_HEAP - room + 576);
        clean(&runtime);
    }
    println!("PAIR_STORAGE bucket=40 outer=1088 inner=576 visited=24");
}
