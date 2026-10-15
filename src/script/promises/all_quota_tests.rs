//! Independent private cuts frozen before Promise.all production.
use super::*;
use std::rc::Rc;

fn reset(runtime: &mut Runtime, work: usize, heap: usize) {
    runtime.steps = work;
    runtime.allocated = MAX_HEAP - heap;
}
fn handler() -> Value {
    Runtime::native("Object.is", Value::Undefined)
}
fn refs(value: &Value) -> usize {
    let Value::Native(native) = value else {
        panic!("Native fixture")
    };
    Rc::strong_count(native)
}
fn same(actual: &Value, expected: &Value) {
    match (actual, expected) {
        (Value::Number(a), Value::Number(b)) => assert_eq!(a.to_bits(), b.to_bits()),
        _ => assert_eq!(actual, expected),
    }
}
fn quiet(runtime: &Runtime) {
    assert!(runtime.microtasks.queue.is_empty());
    assert!(runtime.promises.jobs.records.is_empty());
    assert!(runtime.console.is_empty());
    assert!(runtime.frames.is_empty());
}
fn resource<T>(result: Result<T>) {
    match result {
        Err(error) => assert!(error.is_resource_limit(), "{error:?}"),
        Ok(_) => panic!("expected resource refusal"),
    }
}
fn warm<T>(arena: &mut Arena<T>, admitted: usize) {
    arena
        .records
        .try_reserve_exact(admitted - arena.records.len())
        .unwrap();
    arena.admitted_capacity = admitted;
}
fn raw_group(
    runtime: &mut Runtime,
    values: Vec<Value>,
    remaining: usize,
    done: bool,
) -> AllCaptureId {
    let id = AllCaptureId(runtime.promises.all_captures.records.len());
    let admitted = values.len();
    runtime
        .promises
        .all_captures
        .records
        .push(AllCapture::Group {
            resolve: handler(),
            values: Arena {
                records: values,
                admitted_capacity: admitted,
            },
            remaining,
            input_done: done,
        });
    runtime.promises.all_captures.admitted_capacity = runtime.promises.all_captures.records.len();
    id
}
fn raw_element(
    runtime: &mut Runtime,
    group: AllCaptureId,
    index: usize,
    called: bool,
) -> AllCaptureId {
    let id = AllCaptureId(runtime.promises.all_captures.records.len());
    runtime
        .promises
        .all_captures
        .records
        .push(AllCapture::Element {
            group,
            index,
            called,
        });
    runtime.promises.all_captures.admitted_capacity = runtime.promises.all_captures.records.len();
    id
}
fn group(
    runtime: &Runtime,
    id: AllCaptureId,
    expected: &[Value],
    remaining: usize,
    done: bool,
    admitted: usize,
) {
    let AllCapture::Group {
        values,
        remaining: actual,
        input_done,
        ..
    } = &runtime.promises.all_captures.records[id.0]
    else {
        panic!("Group")
    };
    assert_eq!(
        (*actual, *input_done, values.admitted_capacity),
        (remaining, done, admitted)
    );
    assert_eq!(values.records.len(), expected.len());
    for (a, b) in values.records.iter().zip(expected) {
        same(a, b);
    }
}
fn element(
    runtime: &Runtime,
    id: AllCaptureId,
    expected_group: AllCaptureId,
    expected_index: usize,
    expected_called: bool,
) {
    let AllCapture::Element {
        group,
        index,
        called,
    } = &runtime.promises.all_captures.records[id.0]
    else {
        panic!("Element")
    };
    assert_eq!(
        (group.0, *index, *called),
        (expected_group.0, expected_index, expected_called)
    );
}
fn metadata_bytes() -> usize {
    let leaf =
        16 * (size_of::<PropertyKey>() + size_of::<Property>()) + 32 * size_of::<usize>() + 64;
    904 + size_of::<Option<AbortSlot>>()
        + size_of::<Option<f64>>()
        + 2 * size_of::<PropertyKey>()
        + leaf
        + size_of::<Native>()
}
fn metadata(runtime: &Runtime, value: &Value) -> usize {
    let Value::Native(native) = value else {
        panic!("Native callback")
    };
    let id = native.properties.unwrap().get();
    assert_eq!(native.name, "Promise.callback");
    assert_eq!(native.receiver, Value::Object(id));
    let bag = &runtime.objects[id];
    assert_eq!(
        bag.prototype,
        Some(Value::Function(runtime.function_prototype))
    );
    assert_eq!(
        bag.order,
        [PropertyKey::from("length"), PropertyKey::from("name")]
    );
    assert_eq!(bag.values.len(), 2);
    for (key, expected) in [
        ("length", Value::Number(1.0)),
        ("name", Value::String("".into())),
    ] {
        let property = &bag.values[&PropertyKey::from(key)];
        assert!(!property.enumerable && property.configurable);
        let PropertyValue::Data { value, writable } = &property.value else {
            panic!("data")
        };
        assert!(!writable);
        assert_eq!(value, &expected);
    }
    id
}
fn factory_fixture(cold: bool) -> (Runtime, AllCaptureId) {
    let mut runtime = Runtime::try_new().unwrap();
    let group = raw_group(&mut runtime, vec![Value::Undefined], 1, false);
    runtime.objects.try_reserve_exact(2).unwrap();
    if !cold {
        warm(&mut runtime.promises.all_captures, 8);
        warm(&mut runtime.promises.callbacks, 8);
    }
    (runtime, group)
}
fn exact_capacity<T>(records: &mut Vec<T>) {
    *records = std::mem::take(records).into_boxed_slice().into_vec();
    assert_eq!(records.len(), records.capacity());
}
fn full_growth<T>(len: usize) -> (usize, usize) {
    // Independent full-vector case of the held plan; no production planner call.
    let target = (2 * len).max(8);
    let movement = size_of::<T>().div_ceil(size_of::<Value>());
    (8 + 2 * len * movement, 32 + target * size_of::<T>())
}
fn row_bytes() -> usize {
    size_of::<Vec<Value>>()
        + size_of::<usize>()
        + size_of::<BTreeSet<usize>>()
        + size_of::<array_properties::ArrayLength>()
}
fn array_heap(n: usize) -> usize {
    32 + n * size_of::<Value>()
        + size_of::<ScriptObject>()
        + row_bytes()
        + 72
        + size_of::<Option<AbortSlot>>()
        + size_of::<Option<f64>>()
}
fn array_fixture() -> Runtime {
    let mut runtime = Runtime::try_new().unwrap();
    runtime.objects.try_reserve_exact(2).unwrap();
    runtime.arrays.try_reserve_exact(2).unwrap();
    runtime.array_properties.try_reserve_exact(2).unwrap();
    runtime.array_holes.try_reserve_exact(2).unwrap();
    runtime.array_lengths.try_reserve_exact(2).unwrap();
    runtime
}
fn array_lengths(runtime: &Runtime) -> [usize; 5] {
    [
        runtime.objects.len(),
        runtime.arrays.len(),
        runtime.array_properties.len(),
        runtime.array_holes.len(),
        runtime.array_lengths.len(),
    ]
}

#[test]
fn all_group18_or26_admits_owned_resolve_only_after_work_heap_and_reserve() {
    for cold in [false, true] {
        let total = if cold { 26 } else { 18 };
        let bytes = if cold {
            32 + 8 * size_of::<AllCapture>()
        } else {
            0
        };
        let mut cuts = vec![(total, bytes, true), (total - 1, bytes, false)];
        if cold {
            cuts.push((total, bytes - 1, false));
        }
        for (work, heap, success) in cuts {
            let mut runtime = Runtime::try_new().unwrap();
            if !cold {
                warm(&mut runtime.promises.all_captures, 8);
            }
            let resolve = handler();
            let old_objects = runtime.objects.len();
            reset(&mut runtime, work, heap);
            let result = runtime.new_all_group(resolve.clone());
            assert_eq!(result.is_ok(), success);
            assert_eq!(runtime.objects.len(), old_objects);
            quiet(&runtime);
            if success {
                let id = result.unwrap();
                assert_eq!(id.0, 0);
                group(&runtime, id, &[], 1, false, 0);
                let AllCapture::Group {
                    resolve: retained, ..
                } = &runtime.promises.all_captures.records[0]
                else {
                    panic!("Group")
                };
                assert_eq!(retained, &resolve);
                assert_eq!(refs(&resolve), 2);
                assert_eq!(runtime.promises.all_captures.admitted_capacity, 8);
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
                assert!(!runtime.microtasks_suspended());
            } else {
                resource(result);
                assert_eq!(refs(&resolve), 1);
                assert!(runtime.promises.all_captures.records.is_empty());
                assert_eq!(
                    runtime.promises.all_captures.admitted_capacity,
                    if cold { 0 } else { 8 }
                );
                assert_eq!(
                    runtime.allocated,
                    if work < total {
                        MAX_HEAP - heap
                    } else {
                        MAX_HEAP + 1
                    }
                );
                assert!(runtime.microtasks_suspended());
            }
        }
    }
}

#[test]
fn all_group_growth_uses_typed_movement_even_with_physical_spare() {
    let total = 18 + 8 + 16 * size_of::<AllCapture>().div_ceil(size_of::<Value>());
    let bytes = 32 + 16 * size_of::<AllCapture>();
    for (work, heap, success) in [
        (total, bytes, true),
        (total - 1, bytes, false),
        (total, bytes - 1, false),
    ] {
        let mut runtime = Runtime::try_new().unwrap();
        for _ in 0..8 {
            raw_group(&mut runtime, Vec::new(), 1, false);
        }
        runtime
            .promises
            .all_captures
            .records
            .try_reserve_exact(32)
            .unwrap();
        assert!(runtime.promises.all_captures.records.capacity() >= 40);
        reset(&mut runtime, work, heap);
        let result = runtime.new_all_group(Value::Null);
        assert_eq!(result.is_ok(), success);
        for i in 0..8 {
            group(&runtime, AllCaptureId(i), &[], 1, false, 0);
        }
        assert_eq!(
            runtime.promises.all_captures.records.len(),
            if success { 9 } else { 8 }
        );
        assert_eq!(
            runtime.promises.all_captures.admitted_capacity,
            if success { 16 } else { 8 }
        );
        if success {
            assert_eq!(
                (result.unwrap().0, runtime.steps, runtime.allocated),
                (8, 0, MAX_HEAP)
            );
        } else {
            resource(result);
            assert!(runtime.microtasks_suspended());
        }
    }
    assert!(growth_plan::<AllCapture>(9, 8, 1).is_err());
    assert!(growth_plan::<AllCapture>(usize::MAX, usize::MAX, 1).is_err());
    assert!(growth_plan::<AllCapture>(0, 0, MAX_HEAP / size_of::<AllCapture>() + 1).is_err());
}

#[test]
fn all_placeholder12_20_or36_preserves_initialized_values_and_logical_admission() {
    for (len, admitted, total, target) in [(0, 8, 12, 8), (0, 0, 20, 8), (8, 8, 36, 16)] {
        let bytes = if admitted == target {
            0
        } else {
            32 + target * size_of::<Value>()
        };
        let mut cuts = vec![(total, bytes, true), (total - 1, bytes, false)];
        if bytes != 0 {
            cuts.push((total, bytes - 1, false));
        }
        for (work, heap, success) in cuts {
            let mut runtime = Runtime::try_new().unwrap();
            let old = vec![Value::Number(7.0); len];
            let id = raw_group(&mut runtime, old.clone(), 1, false);
            let AllCapture::Group { values, .. } = &mut runtime.promises.all_captures.records[id.0]
            else {
                panic!("Group")
            };
            values.records.try_reserve_exact(32).unwrap();
            values.admitted_capacity = admitted;
            reset(&mut runtime, work, heap);
            let result = runtime.append_all_placeholder(id);
            assert_eq!(result.is_ok(), success);
            let mut expected = old;
            if success {
                expected.push(Value::Undefined);
            }
            group(
                &runtime,
                id,
                &expected,
                1,
                false,
                if success { target } else { admitted },
            );
            if success {
                assert_eq!(
                    (result.unwrap(), runtime.steps, runtime.allocated),
                    (len, 0, MAX_HEAP)
                );
            } else {
                resource(result);
                assert!(runtime.microtasks_suspended());
            }
            quiet(&runtime);
        }
    }
}

#[test]
fn all_element136_or158_keeps_late_metadata_refusal_unpublished() {
    for cold in [false, true] {
        let grow = if cold {
            16 + 2 * size_of::<AllCapture>().div_ceil(size_of::<Value>())
        } else {
            0
        };
        let total = 136 + grow;
        let bytes = metadata_bytes()
            + size_of::<ScriptObject>()
            + if cold {
                64 + 8 * (size_of::<AllCapture>() + size_of::<CallbackRecord>())
            } else {
                0
            };
        for (work, heap, success) in [
            (total, bytes, true),
            (total - 1, bytes, false),
            (total, bytes - 1, false),
        ] {
            let (mut runtime, id) = factory_fixture(cold);
            let old_objects = runtime.objects.len();
            let old_registry = runtime.native_properties.clone();
            reset(&mut runtime, work, heap);
            let result = runtime.new_all_resolve_element(id, 0);
            assert_eq!(result.is_ok(), success);
            assert_eq!(runtime.objects.len(), old_objects + 1);
            assert_eq!(runtime.native_properties, old_registry);
            group(
                &runtime,
                id,
                &[Value::Undefined],
                if success { 2 } else { 1 },
                false,
                1,
            );
            quiet(&runtime);
            if success {
                let callback = result.unwrap();
                assert_eq!(metadata(&runtime, &callback), old_objects);
                element(&runtime, AllCaptureId(1), id, 0, false);
                assert_eq!(runtime.promises.callbacks.records.len(), 1);
                assert_eq!(runtime.promises.callbacks.records[0].object_id, old_objects);
                assert!(matches!(
                    runtime.promises.callbacks.records[0].kind,
                    CallbackKind::AllResolve(AllCaptureId(1))
                ));
                assert_eq!(
                    (
                        runtime.promises.all_captures.admitted_capacity,
                        runtime.promises.callbacks.admitted_capacity
                    ),
                    (8, 8)
                );
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            } else {
                resource(result);
                assert_eq!(runtime.promises.all_captures.records.len(), 1);
                assert!(runtime.promises.callbacks.records.is_empty());
                assert_eq!(
                    (
                        runtime.promises.all_captures.admitted_capacity,
                        runtime.promises.callbacks.admitted_capacity
                    ),
                    if cold { (1, 0) } else { (8, 8) }
                );
                assert_eq!(runtime.steps, 0);
                assert_eq!(
                    runtime.allocated,
                    if work < total {
                        MAX_HEAP - size_of::<Native>() - 48
                    } else {
                        MAX_HEAP + 1
                    }
                );
                assert!(runtime.microtasks_suspended());
            }
        }
    }
}

#[test]
fn all_element_prefix35_precedes_metadata_and_validates_live_group() {
    for work in [34, 35] {
        let (mut runtime, id) = factory_fixture(false);
        let objects = runtime.objects.len();
        reset(&mut runtime, work, MAX_HEAP);
        resource(runtime.new_all_resolve_element(id, 0));
        assert_eq!(runtime.steps, 0);
        assert_eq!(
            runtime.allocated,
            if work == 35 {
                size_of::<ScriptObject>()
            } else {
                0
            }
        );
        assert_eq!(runtime.objects.len(), objects);
        assert_eq!(runtime.promises.all_captures.records.len(), 1);
        assert!(runtime.promises.callbacks.records.is_empty());
        group(&runtime, id, &[Value::Undefined], 1, false, 1);
    }
    for invalid_case in 0..6 {
        let (mut runtime, id) = factory_fixture(false);
        let mut requested = id;
        let mut index = 0;
        match invalid_case {
            0 => requested = AllCaptureId(usize::MAX),
            1 => {
                runtime.promises.all_captures.records[0] = AllCapture::Element {
                    group: id,
                    index: 0,
                    called: false,
                }
            }
            2 => {
                if let AllCapture::Group { input_done, .. } =
                    &mut runtime.promises.all_captures.records[0]
                {
                    *input_done = true;
                }
            }
            3 => index = 1,
            4 => {
                if let AllCapture::Group { remaining, .. } =
                    &mut runtime.promises.all_captures.records[0]
                {
                    *remaining = usize::MAX;
                }
            }
            5 => {
                if let AllCapture::Group { remaining, .. } =
                    &mut runtime.promises.all_captures.records[0]
                {
                    *remaining = 0;
                }
            }
            _ => unreachable!(),
        }
        let old_objects = runtime.objects.len();
        reset(&mut runtime, 10_000, MAX_HEAP);
        resource(runtime.new_all_resolve_element(requested, index));
        assert_eq!(runtime.objects.len(), old_objects);
        assert_eq!(runtime.allocated, 0);
        assert_eq!(runtime.promises.all_captures.records.len(), 1);
        assert!(runtime.promises.callbacks.records.is_empty());
        assert_eq!(
            (
                runtime.promises.all_captures.admitted_capacity,
                runtime.promises.callbacks.admitted_capacity
            ),
            (8, 8)
        );
        assert!(runtime.microtasks_suspended());
    }
}

#[test]
fn all_factory_late_reserve_failures_preserve_group_count_and_all_logical_rows() {
    use all_allocation_probe::{ReserveSite, fail_once};
    for (failure_index, site) in [
        ReserveSite::AllCapture,
        ReserveSite::Callbacks,
        ReserveSite::Objects,
    ]
    .into_iter()
    .enumerate()
    {
        let (mut runtime, id) = factory_fixture(true);
        for _ in 1..8 {
            raw_element(&mut runtime, id, 0, true);
        }
        exact_capacity(&mut runtime.promises.all_captures.records);
        exact_capacity(&mut runtime.objects);
        let old_objects = runtime.objects.len();
        let AllCapture::Group {
            resolve: Value::Native(resolve),
            ..
        } = &runtime.promises.all_captures.records[0]
        else {
            panic!("retained resolve")
        };
        let resolve_identity = resolve.clone();
        let (object_work, object_bytes) = full_growth::<ScriptObject>(old_objects);
        let record_work = 8 + 16 * size_of::<AllCapture>().div_ceil(size_of::<Value>());
        let record_bytes = 32 + 16 * size_of::<AllCapture>();
        let callback_bytes = 32 + 8 * size_of::<CallbackRecord>();
        let prefix = 35 + record_work + 8 + object_work;
        let planned_heap = record_bytes + callback_bytes + object_bytes + size_of::<ScriptObject>();
        reset(&mut runtime, prefix + 101, MAX_HEAP);
        let guard = fail_once(site);
        resource(runtime.new_all_resolve_element(id, 0));
        let observation = guard.observation();
        let mut attempts = [0; 7];
        let mut completed = [0; 7];
        for slot in attempts.iter_mut().take(failure_index + 1) {
            *slot = 1;
        }
        for slot in completed.iter_mut().take(failure_index) {
            *slot = 1;
        }
        assert_eq!(observation.attempts, attempts);
        assert_eq!(observation.completed, completed);
        assert_eq!((runtime.steps, runtime.allocated), (101, planned_heap));
        assert_eq!(runtime.objects.len(), old_objects);
        assert_eq!(runtime.promises.all_captures.records.len(), 8);
        assert_eq!(runtime.promises.all_captures.admitted_capacity, 8);
        assert!(runtime.promises.callbacks.records.is_empty());
        assert_eq!(runtime.promises.callbacks.admitted_capacity, 0);
        group(&runtime, id, &[Value::Undefined], 1, false, 1);
        let AllCapture::Group {
            resolve: Value::Native(resolve),
            ..
        } = &runtime.promises.all_captures.records[0]
        else {
            panic!("retained resolve")
        };
        assert!(Rc::ptr_eq(resolve, &resolve_identity));
        assert_eq!(Rc::strong_count(resolve), 2);
        for i in 1..8 {
            element(&runtime, AllCaptureId(i), id, 0, true);
        }
        assert_eq!(
            runtime.promises.all_captures.records.capacity() >= 16,
            failure_index > 0
        );
        assert_eq!(
            runtime.promises.callbacks.records.capacity() >= 8,
            failure_index > 1
        );
        assert_eq!(runtime.objects.capacity(), old_objects);
        assert!(runtime.microtasks_suspended());
        quiet(&runtime);
    }
}

#[test]
fn all_reserve_probe_is_fail_once_nested_and_unwind_scoped() {
    use all_allocation_probe::{ReserveSite, before, completed, fail_once};
    let outer = fail_once(ReserveSite::Objects);
    before(ReserveSite::AllCapture).unwrap();
    completed(ReserveSite::AllCapture);
    let saved = outer.observation();
    let caught = std::panic::catch_unwind(|| {
        let inner = fail_once(ReserveSite::Callbacks);
        resource(before(ReserveSite::Callbacks));
        before(ReserveSite::Callbacks).unwrap();
        completed(ReserveSite::Callbacks);
        assert_eq!(inner.observation().attempts, [0, 2, 0, 0, 0, 0, 0]);
        assert_eq!(inner.observation().completed, [0, 1, 0, 0, 0, 0, 0]);
        panic!("intentional Promise.all probe unwind");
    });
    assert!(caught.is_err());
    assert_eq!(outer.observation(), saved);
    resource(before(ReserveSite::Objects));
    before(ReserveSite::Objects).unwrap();
    completed(ReserveSite::Objects);
    assert_eq!(outer.observation().attempts, [1, 0, 2, 0, 0, 0, 0]);
    assert_eq!(outer.observation().completed, [1, 0, 1, 0, 0, 0, 0]);
    drop(outer);
    // Every destination is warm: no actual reserve, no instrumentation event,
    // and the armed failure remains available rather than being consumed.
    let (mut runtime, id) = factory_fixture(false);
    reset(
        &mut runtime,
        136,
        metadata_bytes() + size_of::<ScriptObject>(),
    );
    let warm_guard = fail_once(ReserveSite::AllCapture);
    runtime.new_all_resolve_element(id, 0).unwrap();
    assert_eq!(warm_guard.observation().attempts, [0; 7]);
    assert_eq!(warm_guard.observation().completed, [0; 7]);
    resource(before(ReserveSite::AllCapture));
}

#[test]
fn all_element12_or28_cuts_preserve_once_gate_before_mutation() {
    for work in [27, 28] {
        let mut runtime = Runtime::try_new().unwrap();
        let id = raw_group(&mut runtime, vec![Value::Undefined], 2, false);
        let callback = raw_element(&mut runtime, id, 0, false);
        let value = handler();
        reset(&mut runtime, work, 0);
        let result = runtime.complete_all_element(callback, &value);
        if work == 28 {
            assert!(result.unwrap().is_none());
            group(&runtime, id, std::slice::from_ref(&value), 1, false, 1);
            element(&runtime, callback, id, 0, true);
            assert_eq!(refs(&value), 2);
            assert!(!runtime.microtasks_suspended());
        } else {
            resource(result);
            group(&runtime, id, &[Value::Undefined], 2, false, 1);
            element(&runtime, callback, id, 0, false);
            assert_eq!(refs(&value), 1);
            assert!(runtime.microtasks_suspended());
        }
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    }
    for work in [11, 12] {
        let mut runtime = Runtime::try_new().unwrap();
        let callback = raw_element(&mut runtime, AllCaptureId(usize::MAX), usize::MAX, true);
        reset(&mut runtime, work, 0);
        let result = runtime.complete_all_element(callback, &Value::Number(9.0));
        if work == 12 {
            assert!(result.unwrap().is_none());
        } else {
            resource(result);
        }
        // A duplicate must not access the intentionally invalid/moved group.
        element(
            &runtime,
            callback,
            AllCaptureId(usize::MAX),
            usize::MAX,
            true,
        );
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    }
}

#[test]
fn all_direct_callback38_or22_includes_existing_single_record_lookup10() {
    for duplicate in [false, true] {
        let total = if duplicate { 22 } else { 38 };
        for work in [total - 1, total] {
            let (mut runtime, id) = factory_fixture(false);
            let Value::Native(callback) = runtime.new_all_resolve_element(id, 0).unwrap() else {
                panic!("Native callback")
            };
            if duplicate
                && let AllCapture::Element { called, .. } =
                    &mut runtime.promises.all_captures.records[1]
            {
                *called = true;
            }
            let mut document = Document::parse("");
            reset(&mut runtime, work, 0);
            let result = runtime.promise_callback(&callback, &[Value::Number(17.0)], &mut document);
            if work == total {
                assert_eq!(result.unwrap(), Value::Undefined);
            } else {
                resource(result);
            }
            let changed = !duplicate && work == total;
            group(
                &runtime,
                id,
                &[if changed {
                    Value::Number(17.0)
                } else {
                    Value::Undefined
                }],
                if changed { 1 } else { 2 },
                false,
                1,
            );
            element(&runtime, AllCaptureId(1), id, 0, duplicate || changed);
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            quiet(&runtime);
        }
    }
}

#[test]
fn all_finish16_consumes_sentinel_once_and_moves_empty_or_completed_values() {
    for (values, remaining) in [
        (vec![], 1),
        (vec![Value::Number(9.0)], 1),
        (vec![Value::Undefined], 2),
    ] {
        for work in [15, 16] {
            let mut runtime = Runtime::try_new().unwrap();
            let id = raw_group(&mut runtime, values.clone(), remaining, false);
            reset(&mut runtime, work, 0);
            let result = runtime.finish_all_input(id);
            if work == 15 {
                resource(result);
                group(&runtime, id, &values, remaining, false, values.len());
            } else if remaining == 1 {
                let ready = result.unwrap().expect("ready");
                assert_eq!(ready.values, values);
                assert!(matches!(ready.resolve, Value::Native(_)));
                group(&runtime, id, &[], 0, true, 0);
            } else {
                assert!(result.unwrap().is_none());
                group(&runtime, id, &values, 1, true, values.len());
            }
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        }
    }
}

#[test]
fn all_ready_transfer_preserves_order_bits_and_vector_without_reopening_group() {
    let mut runtime = Runtime::try_new().unwrap();
    let mut values = vec![Value::Undefined, Value::Undefined];
    values.try_reserve_exact(6).unwrap();
    let pointer = values.as_ptr();
    let id = raw_group(&mut runtime, values, 3, false);
    if let AllCapture::Group { values, .. } = &mut runtime.promises.all_captures.records[id.0] {
        values.admitted_capacity = 8;
    }
    let first = raw_element(&mut runtime, id, 0, false);
    let second = raw_element(&mut runtime, id, 1, false);
    let nan = Value::Number(f64::from_bits(0x7ff8_0000_0000_005a));
    let minus_zero = Value::Number(-0.0);
    reset(&mut runtime, 28, 0);
    assert!(
        runtime
            .complete_all_element(second, &minus_zero)
            .unwrap()
            .is_none()
    );
    group(
        &runtime,
        id,
        &[Value::Undefined, minus_zero.clone()],
        2,
        false,
        8,
    );
    reset(&mut runtime, 16, 0);
    assert!(runtime.finish_all_input(id).unwrap().is_none());
    group(
        &runtime,
        id,
        &[Value::Undefined, minus_zero.clone()],
        1,
        true,
        8,
    );
    reset(&mut runtime, 28, 0);
    let ready = runtime
        .complete_all_element(first, &nan)
        .unwrap()
        .expect("ready");
    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    assert_eq!(ready.values.as_ptr(), pointer);
    same(&ready.values[0], &nan);
    same(&ready.values[1], &minus_zero);
    group(&runtime, id, &[], 0, true, 0);
    let AllCapture::Group {
        values, resolve, ..
    } = &runtime.promises.all_captures.records[id.0]
    else {
        panic!("Group")
    };
    assert_eq!(values.records.capacity(), 0);
    assert_eq!(resolve, &Value::Undefined);
    for callback in [first, second] {
        reset(&mut runtime, 12, 0);
        assert!(
            runtime
                .complete_all_element(callback, &Value::Number(99.0))
                .unwrap()
                .is_none()
        );
    }
    for operation in 0..3 {
        reset(&mut runtime, 10_000, MAX_HEAP);
        match operation {
            0 => resource(runtime.append_all_placeholder(id)),
            1 => resource(runtime.new_all_resolve_element(id, 0)),
            2 => resource(runtime.finish_all_input(id)),
            _ => unreachable!(),
        }
        group(&runtime, id, &[], 0, true, 0);
    }
    same(&ready.values[0], &nan);
    same(&ready.values[1], &minus_zero);
}

#[test]
fn all_array32_owns_dense_data_and_cached_prototype_at_exact_heap_cuts() {
    for n in [0, 2] {
        let bytes = array_heap(n);
        for (work, heap, success) in [
            (32, bytes, true),
            (31, bytes, false),
            (32, bytes - 1, false),
        ] {
            let mut runtime = array_fixture();
            let old = array_lengths(&runtime);
            let intrinsic = runtime.array_prototype.unwrap();
            let poison = Runtime::native("Promise.all.forbidden", Value::Undefined);
            runtime.environments[0]
                .bindings
                .get_mut("Array")
                .unwrap()
                .value = poison.clone();
            let prototype_bag = runtime.array_properties[intrinsic];
            runtime.objects[prototype_bag].insert_property(
                PropertyKey::from("0"),
                Property {
                    value: PropertyValue::Accessor {
                        get: poison.clone(),
                        set: poison,
                    },
                    enumerable: true,
                    configurable: true,
                },
            );
            let value = handler();
            let values = if n == 0 {
                vec![]
            } else {
                vec![value.clone(), Value::Number(-0.0)]
            };
            let pointer = values.as_ptr();
            reset(&mut runtime, work, heap);
            let result = runtime.new_all_values_array(values);
            if success {
                let Value::Array(id) = result.unwrap() else {
                    panic!("Array")
                };
                assert_eq!(id, old[1]);
                assert_eq!(array_lengths(&runtime), old.map(|length| length + 1));
                assert_eq!(runtime.arrays[id].as_ptr(), pointer);
                assert_eq!(runtime.arrays[id].len(), n);
                if n != 0 {
                    assert_eq!(runtime.arrays[id][0], value);
                    same(&runtime.arrays[id][1], &Value::Number(-0.0));
                }
                assert!(runtime.array_holes[id].is_empty());
                assert_eq!(runtime.array_lengths[id].value, n as u32);
                assert!(runtime.array_lengths[id].writable);
                assert_eq!(runtime.array_properties[id], old[0]);
                let bag = &runtime.objects[old[0]];
                assert_eq!(bag.prototype, Some(Value::Array(intrinsic)));
                assert!(bag.values.is_empty() && bag.order.is_empty());
                assert_eq!(refs(&value), if n == 0 { 1 } else { 2 });
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            } else {
                resource(result);
                assert_eq!(array_lengths(&runtime), old);
                assert_eq!(refs(&value), 1);
                assert!(runtime.microtasks_suspended());
                assert_eq!(
                    runtime.allocated,
                    if work == 31 { MAX_HEAP } else { MAX_HEAP + 1 }
                );
            }
            quiet(&runtime);
        }
    }
}

#[test]
fn all_array_five_reserve_failures_leave_parallel_rows_unpublished() {
    use all_allocation_probe::{ReserveSite, fail_once};
    let sites = [
        ReserveSite::Objects,
        ReserveSite::ArrayValues,
        ReserveSite::ArrayProperties,
        ReserveSite::ArrayHoles,
        ReserveSite::ArrayLengths,
    ];
    for (stage, site) in sites.into_iter().enumerate() {
        let mut runtime = Runtime::try_new().unwrap();
        exact_capacity(&mut runtime.objects);
        exact_capacity(&mut runtime.arrays);
        exact_capacity(&mut runtime.array_properties);
        exact_capacity(&mut runtime.array_holes);
        exact_capacity(&mut runtime.array_lengths);
        let old = array_lengths(&runtime);
        assert!(old[1..].iter().all(|len| *len == old[1]));
        let plans = [
            full_growth::<ScriptObject>(old[0]),
            full_growth::<Vec<Value>>(old[1]),
            full_growth::<usize>(old[2]),
            full_growth::<BTreeSet<usize>>(old[3]),
            full_growth::<array_properties::ArrayLength>(old[4]),
        ];
        let prefix = 23 + plans.iter().map(|x| x.0).sum::<usize>();
        let heap_prefix = array_heap(1)
            - (72 + size_of::<Option<AbortSlot>>() + size_of::<Option<f64>>())
            + plans.iter().map(|x| x.1).sum::<usize>();
        let value = handler();
        reset(&mut runtime, prefix + 9, MAX_HEAP);
        let guard = fail_once(site);
        resource(runtime.new_all_values_array(vec![value.clone()]));
        let observation = guard.observation();
        let mut attempts = [0; 7];
        let mut completed = [0; 7];
        for slot in attempts.iter_mut().skip(2).take(stage + 1) {
            *slot = 1;
        }
        for slot in completed.iter_mut().skip(2).take(stage) {
            *slot = 1;
        }
        assert_eq!(observation.attempts, attempts);
        assert_eq!(observation.completed, completed);
        assert_eq!((runtime.steps, runtime.allocated), (9, heap_prefix));
        assert_eq!(array_lengths(&runtime), old);
        let capacities = [
            runtime.objects.capacity(),
            runtime.arrays.capacity(),
            runtime.array_properties.capacity(),
            runtime.array_holes.capacity(),
            runtime.array_lengths.capacity(),
        ];
        for (i, (capacity, length)) in capacities.into_iter().zip(old).enumerate() {
            assert_eq!(capacity > length, i < stage);
        }
        assert_eq!(refs(&value), 1);
        assert!(runtime.microtasks_suspended());
        quiet(&runtime);
    }
}

#[test]
fn all_array_rejects_desynchronized_rows_or_invalid_intrinsic_before_publication() {
    for invalid_case in 0..3 {
        let mut runtime = array_fixture();
        match invalid_case {
            0 => {
                runtime.array_holes.pop();
            }
            1 => runtime.array_prototype = None,
            2 => runtime.array_prototype = Some(usize::MAX),
            _ => unreachable!(),
        }
        let old = array_lengths(&runtime);
        reset(&mut runtime, 10_000, MAX_HEAP);
        resource(runtime.new_all_values_array(Vec::new()));
        assert_eq!(array_lengths(&runtime), old);
        assert_eq!(runtime.allocated, 0);
        assert!(runtime.microtasks_suspended());
    }
}

#[test]
fn all_consumed_ready_gate_stays_consumed_after_array_failure_and_suspension() {
    let mut runtime = array_fixture();
    let id = raw_group(&mut runtime, vec![Value::Undefined], 1, true);
    let callback = raw_element(&mut runtime, id, 0, false);
    reset(&mut runtime, 28, 0);
    let ready = runtime
        .complete_all_element(callback, &Value::Number(41.0))
        .unwrap()
        .expect("ready");
    group(&runtime, id, &[], 0, true, 0);
    element(&runtime, callback, id, 0, true);
    let old = array_lengths(&runtime);
    reset(&mut runtime, 32, 0);
    resource(runtime.new_all_values_array(ready.values));
    assert!(runtime.microtasks_suspended());
    assert_eq!(array_lengths(&runtime), old);
    group(&runtime, id, &[], 0, true, 0);
    element(&runtime, callback, id, 0, true);
    reset(&mut runtime, 12, MAX_HEAP);
    assert!(
        runtime
            .complete_all_element(callback, &Value::Number(99.0))
            .unwrap()
            .is_none()
    );
    assert!(runtime.microtasks_suspended());
    reset(&mut runtime, 10_000, MAX_HEAP);
    resource(runtime.queue_microtask(&Value::Undefined, &[handler()]));
    quiet(&runtime);
}

#[test]
fn all_private_readers_reject_wrong_record_kinds_and_uncommittable_counts() {
    for operation in 0..3 {
        for wrong_variant in [false, true] {
            let mut runtime = Runtime::try_new().unwrap();
            let id = raw_group(&mut runtime, vec![Value::Undefined], 1, false);
            let other = raw_element(&mut runtime, id, 0, false);
            let requested = if wrong_variant {
                if operation == 2 { id } else { other }
            } else {
                AllCaptureId(usize::MAX)
            };
            reset(&mut runtime, 10_000, MAX_HEAP);
            match operation {
                0 => resource(runtime.append_all_placeholder(requested)),
                1 => resource(runtime.finish_all_input(requested)),
                2 => resource(runtime.complete_all_element(requested, &Value::Null)),
                _ => unreachable!(),
            }
            group(&runtime, id, &[Value::Undefined], 1, false, 1);
            element(&runtime, other, id, 0, false);
            assert_eq!(runtime.allocated, 0);
            assert!(runtime.microtasks_suspended());
        }
    }
    // A first callback cannot publish zero remaining while input is still open.
    // This invalid private fixture ensures the proof precedes the once-bit write.
    let mut runtime = Runtime::try_new().unwrap();
    let id = raw_group(&mut runtime, vec![Value::Undefined], 1, false);
    let callback = raw_element(&mut runtime, id, 0, false);
    reset(&mut runtime, 28, 0);
    resource(runtime.complete_all_element(callback, &Value::Null));
    group(&runtime, id, &[Value::Undefined], 1, false, 1);
    element(&runtime, callback, id, 0, false);
}
