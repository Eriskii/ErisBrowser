//! Selected draft2 cuts; each terminal refusal starts from a fresh Runtime.
use super::*;
use std::rc::Rc;

fn budget(runtime: &mut Runtime, work: usize, heap: usize) {
    runtime.steps = work;
    runtime.allocated = MAX_HEAP - heap;
}
fn failure<T>(result: Result<T>) -> ScriptError {
    match result {
        Ok(_) => panic!("expected abrupt completion"),
        Err(error) => error,
    }
}
fn resource<T>(result: Result<T>) {
    assert!(failure(result).is_resource_limit());
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
fn quiet(runtime: &Runtime) {
    assert!(runtime.frames.is_empty());
    assert!(runtime.console.is_empty());
    assert!(runtime.microtasks.queue.is_empty());
    assert!(runtime.promises.jobs.records.is_empty());
    assert_eq!(
        (runtime.calls, runtime.eval_depth, runtime.stack_units),
        (0, 0, 0)
    );
}
fn warm<T>(arena: &mut Arena<T>) {
    arena
        .records
        .try_reserve_exact(8 - arena.records.len())
        .unwrap();
    assert!(arena.records.capacity() >= 8);
    arena.admitted_capacity = 8;
}
fn exact_capacity<T>(values: &mut Vec<T>) {
    *values = std::mem::take(values).into_boxed_slice().into_vec();
    assert_eq!(values.len(), values.capacity());
}
fn growth<T>(len: usize) -> (usize, usize) {
    (
        8 + 2 * len * size_of::<T>().div_ceil(size_of::<Value>()),
        32 + (2 * len).max(8) * size_of::<T>(),
    )
}
fn leaf() -> usize {
    16 * (size_of::<PropertyKey>() + size_of::<Property>()) + 32 * size_of::<usize>() + 64
}
fn bag_base() -> usize {
    72 + size_of::<Option<AbortSlot>>() + size_of::<Option<f64>>()
}
fn bag() -> usize {
    bag_base() + 2 * size_of::<PropertyKey>()
}
fn metadata_heap() -> usize {
    832 + bag() + leaf() + size_of::<Native>()
}
fn snapshot_heap(kind: ReactionKind) -> usize {
    size_of::<ScriptObject>()
        + bag()
        + leaf()
        + match kind {
            ReactionKind::Fulfill => 828,
            ReactionKind::Reject => 832,
        }
}
fn completion_work(kind: ReactionKind) -> usize {
    match kind {
        ReactionKind::Fulfill => 163,
        ReactionKind::Reject => 164,
    }
}
fn raw_group(runtime: &mut Runtime, n: usize, remaining: usize, done: bool) -> AllCaptureId {
    let id = AllCaptureId(runtime.promises.all_captures.records.len());
    runtime
        .promises
        .all_captures
        .records
        .push(AllCapture::Group {
            resolve: handler(),
            values: Arena {
                records: vec![Value::Undefined; n],
                admitted_capacity: n,
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
        resolve,
        values,
        remaining: actual,
        input_done,
    } = &runtime.promises.all_captures.records[id.0]
    else {
        panic!("Group")
    };
    assert_eq!(
        (*actual, *input_done, values.admitted_capacity),
        (remaining, done, admitted)
    );
    assert_eq!(values.records, expected);
    assert!(matches!(resolve, Value::Native(_)));
}
fn called(runtime: &Runtime, id: AllCaptureId, group: AllCaptureId, index: usize, expected: bool) {
    let AllCapture::Element {
        group: actual_group,
        index: actual_index,
        called,
    } = &runtime.promises.all_captures.records[id.0]
    else {
        panic!("Element")
    };
    assert_eq!(
        (actual_group.0, *actual_index, *called),
        (group.0, index, expected)
    );
}
fn snapshot(runtime: &Runtime, value: &Value, kind: ReactionKind, payload: &Value) -> usize {
    let Value::Object(id) = value else {
        panic!("snapshot object")
    };
    let object = &runtime.objects[*id];
    let (status, key) = match kind {
        ReactionKind::Fulfill => ("fulfilled", "value"),
        ReactionKind::Reject => ("rejected", "reason"),
    };
    assert_eq!(
        object.prototype,
        Some(Value::Object(runtime.prototypes["Object"]))
    );
    assert_eq!(
        object.order,
        [PropertyKey::from("status"), PropertyKey::from(key)]
    );
    assert_eq!(object.values.len(), 2);
    for (key, expected) in [
        ("status", Value::String(status.into())),
        (key, payload.clone()),
    ] {
        let p = &object.values[&PropertyKey::from(key)];
        assert!(p.enumerable && p.configurable);
        let PropertyValue::Data { value, writable } = &p.value else {
            panic!("own data")
        };
        assert!(*writable);
        assert_eq!(value, &expected);
    }
    *id
}
fn metadata(runtime: &Runtime, value: &Value) -> usize {
    let Value::Native(native) = value else {
        panic!("callback")
    };
    let id = native.properties.unwrap().get();
    assert_eq!(native.name, "Promise.callback");
    assert_eq!(native.receiver, Value::Object(id));
    let object = &runtime.objects[id];
    assert_eq!(
        object.prototype,
        Some(Value::Function(runtime.function_prototype))
    );
    assert_eq!(
        object.order,
        [PropertyKey::from("length"), PropertyKey::from("name")]
    );
    assert_eq!(object.values.len(), 2);
    for (key, expected) in [
        ("length", Value::Number(1.0)),
        ("name", Value::String("".into())),
    ] {
        let p = &object.values[&PropertyKey::from(key)];
        assert!(!p.enumerable && p.configurable);
        let PropertyValue::Data { value, writable } = &p.value else {
            panic!("data")
        };
        assert!(!writable);
        assert_eq!(value, &expected);
    }
    id
}
fn pair_fixture(cold: bool) -> (Runtime, AllCaptureId) {
    let mut runtime = Runtime::try_new().unwrap();
    let id = raw_group(&mut runtime, 1, 1, false);
    runtime.objects.try_reserve_exact(2).unwrap();
    assert!(runtime.objects.capacity() >= runtime.objects.len() + 2);
    if cold {
        exact_capacity(&mut runtime.promises.all_captures.records);
    } else {
        warm(&mut runtime.promises.all_captures);
        warm(&mut runtime.promises.callbacks);
    }
    (runtime, id)
}
fn completion_fixture() -> (Runtime, AllCaptureId, AllCaptureId, Value) {
    let mut runtime = Runtime::try_new().unwrap();
    let group = raw_group(&mut runtime, 1, 2, false);
    let element = raw_element(&mut runtime, group, 0, false);
    runtime.objects.try_reserve_exact(1).unwrap();
    assert!(runtime.objects.capacity() > runtime.objects.len());
    assert_eq!(runtime.prototypes.len(), 25);
    (runtime, group, element, handler())
}

#[test]
fn all_settled_body16_and_public8_refuse_before_capability_or_author_work() {
    for public in [false, true] {
        let cuts: &[usize] = if public {
            &[7, 8, 23, 24, 25, 26, 27]
        } else {
            &[15, 16, 17, 18, 19]
        };
        for &work in cuts {
            let mut runtime = Runtime::try_new().unwrap();
            let mut document = Document::parse("");
            let retained = handler();
            let input = retained.clone();
            let objects = runtime.objects.len();
            budget(&mut runtime, work, 0);
            let result = if public {
                runtime.promise_native_operation("Promise.allSettled", input, &[], &mut document)
            } else {
                runtime.promise_all_settled(input, &[], &mut document)
            };
            let error = failure(result);
            let ordinary = work == if public { 27 } else { 19 };
            if ordinary {
                assert!(matches!(error.kind, ErrorKind::Runtime("TypeError")));
            } else {
                assert!(error.is_resource_limit());
            }
            if public {
                assert_eq!(runtime.microtasks_suspended(), !ordinary);
            }
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            assert_eq!(runtime.objects.len(), objects);
            assert_eq!(refs(&retained), 1);
            assert!(runtime.promises.all_captures.records.is_empty());
            assert!(runtime.promises.callbacks.records.is_empty());
            assert!(runtime.promises.captures.records.is_empty());
            assert!(runtime.promises.promises.records.is_empty());
            quiet(&runtime);
        }
    }
}

#[test]
fn all_settled_pair252_publishes_one_element_and_two_distinct_metadata_bags() {
    for cold in [false, true] {
        let (extra_work, extra_heap) = if cold {
            let (w, h) = growth::<AllCapture>(1);
            let (cw, ch) = growth::<CallbackRecord>(0);
            (w + cw, h + ch)
        } else {
            (0, 0)
        };
        let total = 252 + extra_work;
        let bytes = 2 * metadata_heap() + 2 * size_of::<ScriptObject>() + extra_heap;
        for (work, heap, success) in [
            (total, bytes, true),
            (total - 1, bytes, false),
            (total, bytes - 1, false),
        ] {
            let (mut runtime, id) = pair_fixture(cold);
            let objects = runtime.objects.len();
            let registry = runtime.native_properties.clone();
            let before = stored(&runtime);
            budget(&mut runtime, work, heap);
            let result = runtime.new_all_settled_elements(id, 0);
            assert_eq!(result.is_ok(), success);
            assert_eq!(runtime.objects.len(), objects + 2);
            assert_eq!(runtime.native_properties, registry);
            group(
                &runtime,
                id,
                &[Value::Undefined],
                if success { 2 } else { 1 },
                false,
                1,
            );
            assert_eq!(runtime.steps, 0);
            if success {
                let (fulfill, reject) = result.unwrap();
                assert_eq!(
                    (metadata(&runtime, &fulfill), metadata(&runtime, &reject)),
                    (objects, objects + 1)
                );
                assert_ne!(fulfill, reject);
                called(&runtime, AllCaptureId(1), id, 0, false);
                assert_eq!(runtime.promises.all_captures.records.len(), 2);
                assert_eq!(runtime.promises.callbacks.records.len(), 2);
                let records = &runtime.promises.callbacks.records;
                assert_eq!(
                    (records[0].object_id, records[1].object_id),
                    (objects, objects + 1)
                );
                assert!(matches!(
                    records[0].kind,
                    CallbackKind::AllSettledFulfill(AllCaptureId(1))
                ));
                assert!(matches!(
                    records[1].kind,
                    CallbackKind::AllSettledReject(AllCaptureId(1))
                ));
                assert_eq!(
                    (
                        runtime.promises.all_captures.admitted_capacity,
                        runtime.promises.callbacks.admitted_capacity
                    ),
                    (8, 8)
                );
                assert_eq!(runtime.allocated, MAX_HEAP);
            } else {
                resource(result);
                assert_eq!(stored(&runtime), before);
                assert_eq!(runtime.promises.all_captures.records.len(), 1);
                assert!(runtime.promises.callbacks.records.is_empty());
                assert_eq!(
                    (
                        runtime.promises.all_captures.admitted_capacity,
                        runtime.promises.callbacks.admitted_capacity
                    ),
                    if cold { (1, 0) } else { (8, 8) }
                );
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
            quiet(&runtime);
        }
    }
}

#[test]
fn all_settled_pair50_checks_live_group_before_any_metadata() {
    for work in [49, 50] {
        let (mut runtime, id) = pair_fixture(false);
        let objects = runtime.objects.len();
        budget(&mut runtime, work, MAX_HEAP);
        resource(runtime.new_all_settled_elements(id, 0));
        assert_eq!(
            (runtime.steps, runtime.allocated),
            (
                0,
                if work == 50 {
                    2 * size_of::<ScriptObject>()
                } else {
                    0
                }
            )
        );
        assert_eq!(runtime.objects.len(), objects);
        assert_eq!(runtime.promises.all_captures.records.len(), 1);
        assert!(runtime.promises.callbacks.records.is_empty());
        group(&runtime, id, &[Value::Undefined], 1, false, 1);
    }
    for invalid_case in 0..7 {
        let (mut runtime, id) = pair_fixture(false);
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
            2..=6 => {
                let AllCapture::Group {
                    input_done,
                    remaining,
                    values,
                    ..
                } = &mut runtime.promises.all_captures.records[0]
                else {
                    unreachable!()
                };
                match invalid_case {
                    2 => *input_done = true,
                    3 => index = 1,
                    4 => *remaining = 0,
                    5 => *remaining = usize::MAX,
                    6 => values.admitted_capacity = 0,
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
        let objects = runtime.objects.len();
        let before = stored(&runtime);
        budget(&mut runtime, 50, MAX_HEAP);
        resource(runtime.new_all_settled_elements(requested, index));
        assert_eq!(stored(&runtime), before);
        assert_eq!((runtime.steps, runtime.allocated), (0, 0));
        assert_eq!(runtime.objects.len(), objects);
        assert_eq!(runtime.promises.all_captures.records.len(), 1);
        assert!(runtime.promises.callbacks.records.is_empty());
        assert_eq!(
            (
                runtime.promises.all_captures.admitted_capacity,
                runtime.promises.callbacks.admitted_capacity
            ),
            (8, 8)
        );
    }
}

#[test]
fn all_settled_pair_reserves_all_capture_callbacks_objects_before_publication() {
    use all_allocation_probe::{ReserveSite, fail_once};
    for (position, site) in [
        ReserveSite::AllCapture,
        ReserveSite::Callbacks,
        ReserveSite::Objects,
    ]
    .into_iter()
    .enumerate()
    {
        let (mut runtime, id) = pair_fixture(true);
        exact_capacity(&mut runtime.objects);
        let objects = runtime.objects.len();
        let (aw, ah) = growth::<AllCapture>(1);
        let (cw, ch) = growth::<CallbackRecord>(0);
        let (ow, oh) = growth::<ScriptObject>(objects);
        let bytes = ah + ch + oh + 2 * size_of::<ScriptObject>();
        let before = stored(&runtime);
        budget(&mut runtime, 50 + aw + cw + ow, bytes);
        let guard = fail_once(site);
        resource(runtime.new_all_settled_elements(id, 0));
        assert_eq!(stored(&runtime), before);
        let observed = guard.observation();
        for i in 0..7 {
            assert_eq!(observed.attempts[i], usize::from(i <= position));
            assert_eq!(observed.completed[i], usize::from(i < position));
        }
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        assert_eq!(runtime.objects.len(), objects);
        assert_eq!(runtime.objects.capacity(), objects);
        assert_eq!(runtime.promises.all_captures.records.len(), 1);
        assert!(runtime.promises.callbacks.records.is_empty());
        assert_eq!(
            (
                runtime.promises.all_captures.admitted_capacity,
                runtime.promises.callbacks.admitted_capacity
            ),
            (1, 0)
        );
        assert_eq!(
            runtime.promises.all_captures.records.capacity() >= 8,
            position > 0
        );
        assert_eq!(
            runtime.promises.callbacks.records.capacity() >= 8,
            position > 1
        );
        group(&runtime, id, &[Value::Undefined], 1, false, 1);
        assert!(runtime.microtasks_suspended());
        quiet(&runtime);
    }
}

#[test]
fn all_settled_snapshot17_saved_object_lookup22_and_existing98_or99_are_separate() {
    for kind in [ReactionKind::Fulfill, ReactionKind::Reject] {
        let total = completion_work(kind) - 26;
        for work in [16, 17, 38, 39, total - 1, total] {
            let (mut runtime, _, _, payload) = completion_fixture();
            let objects = runtime.objects.len();
            budget(&mut runtime, work, MAX_HEAP);
            let result = runtime.new_all_settled_snapshot(kind, &payload);
            if work == total {
                assert_eq!(
                    snapshot(&runtime, &result.unwrap(), kind, &payload),
                    objects
                );
                assert_eq!(refs(&payload), 2);
                assert_eq!(runtime.allocated, snapshot_heap(kind));
            } else {
                resource(result);
                assert_eq!(refs(&payload), 1);
                if work <= 39 {
                    assert_eq!(runtime.objects.len(), objects);
                }
                assert!(runtime.microtasks_suspended());
            }
            assert_eq!(runtime.steps, 0);
            if work < 39 {
                assert_eq!(runtime.allocated, 0);
            }
            if work == 39 {
                assert_eq!(runtime.allocated, size_of::<ScriptObject>());
            }
            quiet(&runtime);
        }
    }
    for missing in [false, true] {
        let (mut runtime, _, _, payload) = completion_fixture();
        if missing {
            assert!(runtime.prototypes.remove("Object").is_some());
        } else {
            *runtime.prototypes.get_mut("Object").unwrap() = usize::MAX;
        }
        assert_eq!(runtime.prototypes.len(), if missing { 24 } else { 25 });
        let objects = runtime.objects.len();
        budget(&mut runtime, 39, MAX_HEAP);
        resource(runtime.new_all_settled_snapshot(ReactionKind::Fulfill, &payload));
        assert_eq!((runtime.steps, runtime.allocated), (0, 0));
        assert_eq!(runtime.objects.len(), objects);
        assert_eq!(refs(&payload), 1);
    }
}

#[test]
fn all_settled_completion_staged_work_consumes_gate_before_snapshot_but_not_group() {
    let slot = size_of::<ScriptObject>();
    let bag_base = bag_base();
    let bag = bag();
    let bpp = leaf();
    // Literal prefixes from the selected ledger, before the implementation.
    for (kind, rows) in [
        (
            ReactionKind::Fulfill,
            &[
                (11, false, 0, 0, 0),
                (12, false, 0, 0, 0),
                (25, false, 0, 0, 0),
                (26, true, 0, 0, 0),
                (42, true, 0, 0, 0),
                (43, true, 0, 0, 0),
                (64, true, 0, 0, 0),
                (65, true, 0, 0, slot),
                (73, true, 0, 0, slot + bag_base),
                (74, true, 1, 0, slot + bag),
                (93, true, 1, 0, slot + bag),
                (94, true, 1, 0, slot + bag + 88),
                (119, true, 1, 0, slot + bag + 88),
                (120, true, 1, 0, slot + bag + 188),
                (137, true, 1, 0, slot + bag + 188),
                (138, true, 1, 0, slot + bag + 272),
                (146, true, 1, 0, slot + bag + 272),
                (147, true, 1, 1, slot + bag + 552 + bpp),
                (162, true, 1, 1, slot + bag + 552 + bpp),
                (163, true, 1, 2, snapshot_heap(ReactionKind::Fulfill)),
            ][..],
        ),
        (
            ReactionKind::Reject,
            &[
                (11, false, 0, 0, 0),
                (12, false, 0, 0, 0),
                (25, false, 0, 0, 0),
                (26, true, 0, 0, 0),
                (42, true, 0, 0, 0),
                (43, true, 0, 0, 0),
                (64, true, 0, 0, 0),
                (65, true, 0, 0, slot),
                (73, true, 0, 0, slot + bag_base),
                (74, true, 1, 0, slot + bag),
                (93, true, 1, 0, slot + bag),
                (94, true, 1, 0, slot + bag + 88),
                (117, true, 1, 0, slot + bag + 88),
                (118, true, 1, 0, slot + bag + 184),
                (137, true, 1, 0, slot + bag + 184),
                (138, true, 1, 0, slot + bag + 272),
                (146, true, 1, 0, slot + bag + 272),
                (147, true, 1, 1, slot + bag + 552 + bpp),
                (163, true, 1, 1, slot + bag + 552 + bpp),
                (164, true, 1, 2, snapshot_heap(ReactionKind::Reject)),
            ][..],
        ),
    ] {
        for &(work, expected_called, object_count, properties, allocated) in rows {
            let (mut runtime, id, element, payload) = completion_fixture();
            let objects = runtime.objects.len();
            let success = work == completion_work(kind);
            budget(&mut runtime, work, MAX_HEAP);
            let result = runtime.complete_all_settled_element(element, kind, &payload);
            assert_eq!(runtime.objects.len(), objects + object_count);
            assert_eq!((runtime.steps, runtime.allocated), (0, allocated));
            called(&runtime, element, id, 0, expected_called);
            if success {
                assert!(result.unwrap().is_none());
                group(&runtime, id, &[Value::Object(objects)], 1, false, 1);
                snapshot(&runtime, &Value::Object(objects), kind, &payload);
                assert_eq!(refs(&payload), 2);
                assert!(!runtime.microtasks_suspended());
            } else {
                resource(result);
                group(&runtime, id, &[Value::Undefined], 2, false, 1);
                assert_eq!(refs(&payload), 1);
                assert!(runtime.microtasks_suspended());
            }
            if object_count != 0 {
                assert_eq!(runtime.objects[objects].values.len(), properties);
                assert_eq!(runtime.objects[objects].order.len(), properties);
                if properties == 1 {
                    assert_eq!(
                        runtime.objects[objects].order,
                        [PropertyKey::from("status")]
                    );
                }
            }
            quiet(&runtime);
        }
    }
}

#[test]
fn all_settled_completion_heap_short_stages_keep_only_unexposed_partial_bags() {
    let slot = size_of::<ScriptObject>();
    for kind in [ReactionKind::Fulfill, ReactionKind::Reject] {
        let reject = matches!(kind, ReactionKind::Reject);
        let stages = [
            (slot, 65, 0, 0),
            (slot + bag_base(), 73, 0, 0),
            (slot + bag(), 74, 0, 0),
            (slot + bag() + 88, 94, 1, 0),
            (
                slot + bag() + if reject { 184 } else { 188 },
                if reject { 118 } else { 120 },
                1,
                0,
            ),
            (slot + bag() + 272, 138, 1, 0),
            (slot + bag() + 552 + leaf(), 147, 1, 0),
            (snapshot_heap(kind), completion_work(kind), 1, 1),
        ];
        for (bytes, spent, object_count, properties) in stages {
            let (mut runtime, id, element, payload) = completion_fixture();
            let objects = runtime.objects.len();
            budget(&mut runtime, completion_work(kind), bytes - 1);
            resource(runtime.complete_all_settled_element(element, kind, &payload));
            assert_eq!(
                (runtime.steps, runtime.allocated),
                (completion_work(kind) - spent, MAX_HEAP + 1)
            );
            assert_eq!(runtime.objects.len(), objects + object_count);
            called(&runtime, element, id, 0, true);
            group(&runtime, id, &[Value::Undefined], 2, false, 1);
            assert_eq!(refs(&payload), 1);
            if object_count != 0 {
                assert_eq!(runtime.objects[objects].values.len(), properties);
            }
            assert!(runtime.microtasks_suspended());
            quiet(&runtime);
        }
    }
}

#[test]
fn all_settled_snapshot_object_growth_and_fail_once_are_reached_only_after_gate() {
    use all_allocation_probe::{ReserveSite, fail_once};
    for kind in [ReactionKind::Fulfill, ReactionKind::Reject] {
        for variant in 0..4 {
            let (mut runtime, id, element, payload) = completion_fixture();
            exact_capacity(&mut runtime.objects);
            let objects = runtime.objects.len();
            let (work, heap) = growth::<ScriptObject>(objects);
            let total = completion_work(kind) + work;
            let bytes = snapshot_heap(kind) + heap;
            budget(
                &mut runtime,
                if variant == 1 { total - 1 } else { total },
                if variant == 2 { bytes - 1 } else { bytes },
            );
            let guard = (variant == 3).then(|| fail_once(ReserveSite::Objects));
            let result = runtime.complete_all_settled_element(element, kind, &payload);
            called(&runtime, element, id, 0, true);
            if variant == 0 {
                assert!(result.unwrap().is_none());
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
                group(&runtime, id, &[Value::Object(objects)], 1, false, 1);
                snapshot(&runtime, &Value::Object(objects), kind, &payload);
            } else {
                resource(result);
                group(&runtime, id, &[Value::Undefined], 2, false, 1);
                assert_eq!(refs(&payload), 1);
                if let Some(guard) = guard {
                    let observed = guard.observation();
                    assert_eq!(observed.attempts, [0, 0, 1, 0, 0, 0, 0]);
                    assert_eq!(observed.completed, [0; 7]);
                    assert_eq!(runtime.objects.len(), objects);
                    assert_eq!(runtime.objects.capacity(), objects);
                    assert_eq!(runtime.steps, completion_work(kind) - 65);
                    assert_eq!(
                        runtime.allocated,
                        MAX_HEAP - bytes + heap + size_of::<ScriptObject>()
                    );
                } else {
                    assert_eq!(runtime.steps, 0);
                    assert_eq!(runtime.objects.len(), objects + 1);
                    assert_eq!(
                        runtime.allocated,
                        if variant == 1 {
                            MAX_HEAP
                                - if matches!(kind, ReactionKind::Fulfill) {
                                    276
                                } else {
                                    280
                                }
                        } else {
                            MAX_HEAP + 1
                        }
                    );
                }
                assert!(runtime.microtasks_suspended());
            }
            quiet(&runtime);
        }
        let (mut runtime, id, element, payload) = completion_fixture();
        let objects = runtime.objects.len();
        budget(&mut runtime, completion_work(kind), snapshot_heap(kind));
        let guard = fail_once(ReserveSite::Objects);
        assert!(
            runtime
                .complete_all_settled_element(element, kind, &payload)
                .unwrap()
                .is_none()
        );
        assert_eq!(guard.observation().attempts, [0; 7]);
        assert_eq!(guard.observation().completed, [0; 7]);
        group(&runtime, id, &[Value::Object(objects)], 1, false, 1);
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    }
}

#[derive(Debug, PartialEq)]
enum Stored {
    Group(Value, Vec<Value>, usize, usize, bool),
    Element(usize, usize, bool),
}
fn stored(runtime: &Runtime) -> Vec<Stored> {
    runtime
        .promises
        .all_captures
        .records
        .iter()
        .map(|record| match record {
            AllCapture::Group {
                resolve,
                values,
                remaining,
                input_done,
            } => Stored::Group(
                resolve.clone(),
                values.records.clone(),
                values.admitted_capacity,
                *remaining,
                *input_done,
            ),
            AllCapture::Element {
                group,
                index,
                called,
            } => Stored::Element(group.0, *index, *called),
        })
        .collect()
}

#[test]
fn all_settled_invalid_completion_stays_unconsumed_and_duplicate_skips_group() {
    for invalid_case in 0..8 {
        let (mut runtime, id, element, payload) = completion_fixture();
        let mut requested = element;
        match invalid_case {
            0 => requested = AllCaptureId(usize::MAX),
            1 => requested = id,
            2 => {
                let AllCapture::Element { group, .. } =
                    &mut runtime.promises.all_captures.records[element.0]
                else {
                    unreachable!()
                };
                *group = AllCaptureId(usize::MAX);
            }
            3 => {
                runtime.promises.all_captures.records[id.0] = AllCapture::Element {
                    group: id,
                    index: 0,
                    called: false,
                }
            }
            4 => {
                let AllCapture::Element { index, .. } =
                    &mut runtime.promises.all_captures.records[element.0]
                else {
                    unreachable!()
                };
                *index = 1;
            }
            5..=7 => {
                let AllCapture::Group {
                    values, remaining, ..
                } = &mut runtime.promises.all_captures.records[id.0]
                else {
                    unreachable!()
                };
                match invalid_case {
                    5 => values.admitted_capacity = 0,
                    6 => *remaining = 0,
                    7 => *remaining = 1,
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
        let before = stored(&runtime);
        let objects = runtime.objects.len();
        budget(
            &mut runtime,
            if invalid_case < 2 { 12 } else { 26 },
            MAX_HEAP,
        );
        resource(runtime.complete_all_settled_element(requested, ReactionKind::Fulfill, &payload));
        assert_eq!(stored(&runtime), before);
        assert_eq!((runtime.steps, runtime.allocated), (0, 0));
        assert_eq!(runtime.objects.len(), objects);
        assert_eq!(refs(&payload), 1);
        assert!(runtime.microtasks_suspended());
    }
    for kind in [ReactionKind::Fulfill, ReactionKind::Reject] {
        for work in [11, 12] {
            let mut runtime = Runtime::try_new().unwrap();
            let group = AllCaptureId(usize::MAX);
            let element = raw_element(&mut runtime, group, usize::MAX, true);
            let payload = handler();
            let before = stored(&runtime);
            let objects = runtime.objects.len();
            budget(&mut runtime, work, 0);
            let result = runtime.complete_all_settled_element(element, kind, &payload);
            if work == 12 {
                assert!(result.unwrap().is_none());
            } else {
                resource(result);
            }
            assert_eq!(stored(&runtime), before);
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            assert_eq!(runtime.objects.len(), objects);
            assert_eq!(refs(&payload), 1);
            assert_eq!(runtime.microtasks_suspended(), work == 11);
        }
    }
}

#[test]
fn all_settled_dispatch12_both_directions_share_one_called_gate() {
    for rejected_first in [false, true] {
        let kind = if rejected_first {
            ReactionKind::Reject
        } else {
            ReactionKind::Fulfill
        };
        let total = 12 + completion_work(kind);
        for work in [total - 1, total] {
            let (mut runtime, id) = pair_fixture(false);
            let pair = runtime.new_all_settled_elements(id, 0).unwrap();
            // Pair metadata consumed its two reserved slots; this cut measures a warm snapshot.
            runtime.objects.try_reserve_exact(1).unwrap();
            assert!(runtime.objects.capacity() > runtime.objects.len());
            let first = if rejected_first { &pair.1 } else { &pair.0 };
            let Value::Native(native) = first else {
                panic!("callback")
            };
            let payload = handler();
            let objects = runtime.objects.len();
            budget(&mut runtime, work, snapshot_heap(kind));
            let result = runtime.promise_callback(
                native,
                std::slice::from_ref(&payload),
                &mut Document::parse(""),
            );
            called(&runtime, AllCaptureId(1), id, 0, true);
            if work == total {
                assert_eq!(result.unwrap(), Value::Undefined);
                group(&runtime, id, &[Value::Object(objects)], 1, false, 1);
                snapshot(&runtime, &Value::Object(objects), kind, &payload);
                assert_eq!(refs(&payload), 2);
            } else {
                resource(result);
                group(&runtime, id, &[Value::Undefined], 2, false, 1);
                assert_eq!(refs(&payload), 1);
            }
            assert_eq!(runtime.steps, 0);
            quiet(&runtime);
        }
        for work in [23, 24] {
            let (mut runtime, id) = pair_fixture(false);
            let pair = runtime.new_all_settled_elements(id, 0).unwrap();
            let payload = handler();
            assert!(
                runtime
                    .complete_all_settled_element(AllCaptureId(1), kind, &payload)
                    .unwrap()
                    .is_none()
            );
            let second = if rejected_first { &pair.0 } else { &pair.1 };
            let Value::Native(native) = second else {
                panic!("callback")
            };
            let before = stored(&runtime);
            let objects = runtime.objects.len();
            budget(&mut runtime, work, 0);
            let result = runtime.promise_callback(native, &[], &mut Document::parse(""));
            if work == 24 {
                assert_eq!(result.unwrap(), Value::Undefined);
            } else {
                resource(result);
            }
            assert_eq!(stored(&runtime), before);
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            assert_eq!(runtime.objects.len(), objects);
            assert_eq!(refs(&payload), 2);
            called(&runtime, AllCaptureId(1), id, 0, true);
            assert_eq!(runtime.microtasks_suspended(), work == 23);
            quiet(&runtime);
        }
    }
}

#[test]
fn all_settled_ready_moves_entire_arena_and_opposite_duplicates_ignore_it() {
    let mut runtime = Runtime::try_new().unwrap();
    let id = raw_group(&mut runtime, 2, 2, true);
    let first = raw_element(&mut runtime, id, 0, false);
    let second = raw_element(&mut runtime, id, 1, false);
    let (pointer, resolve) = match &runtime.promises.all_captures.records[id.0] {
        AllCapture::Group {
            values, resolve, ..
        } => (values.records.as_ptr(), resolve.clone()),
        _ => unreachable!(),
    };
    runtime.objects.try_reserve_exact(2).unwrap();
    let objects = runtime.objects.len();
    budget(&mut runtime, 164, snapshot_heap(ReactionKind::Reject));
    assert!(
        runtime
            .complete_all_settled_element(second, ReactionKind::Reject, &Value::Number(7.0))
            .unwrap()
            .is_none()
    );
    group(
        &runtime,
        id,
        &[Value::Undefined, Value::Object(objects)],
        1,
        true,
        2,
    );
    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    // The preceding operation succeeded. A fresh isolated budget measures the second success.
    budget(&mut runtime, 163, snapshot_heap(ReactionKind::Fulfill));
    let ready = runtime
        .complete_all_settled_element(first, ReactionKind::Fulfill, &Value::Number(9.0))
        .unwrap()
        .unwrap();
    assert_eq!(ready.values.as_ptr(), pointer);
    assert_eq!(ready.resolve, resolve);
    assert_eq!(
        ready.values,
        [Value::Object(objects + 1), Value::Object(objects)]
    );
    snapshot(
        &runtime,
        &ready.values[0],
        ReactionKind::Fulfill,
        &Value::Number(9.0),
    );
    snapshot(
        &runtime,
        &ready.values[1],
        ReactionKind::Reject,
        &Value::Number(7.0),
    );
    let AllCapture::Group {
        resolve,
        values,
        remaining,
        input_done,
    } = &runtime.promises.all_captures.records[id.0]
    else {
        unreachable!()
    };
    assert_eq!(*resolve, Value::Undefined);
    assert_eq!(
        (
            *remaining,
            *input_done,
            values.admitted_capacity,
            values.records.len(),
            values.records.capacity()
        ),
        (0, true, 0, 0, 0)
    );
    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    for (element, kind) in [
        (first, ReactionKind::Reject),
        (second, ReactionKind::Fulfill),
    ] {
        budget(&mut runtime, 12, 0);
        assert!(
            runtime
                .complete_all_settled_element(element, kind, &Value::Undefined)
                .unwrap()
                .is_none()
        );
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    }
    assert_eq!(runtime.objects.len(), objects + 2);
    quiet(&runtime);
}

#[test]
fn all_settled_extension_preserves_old_all_first28_duplicate12_without_snapshot() {
    for work in [27, 28] {
        let (mut runtime, id, element, payload) = completion_fixture();
        let objects = runtime.objects.len();
        budget(&mut runtime, work, 0);
        let result = runtime.complete_all_element(element, &payload);
        if work == 28 {
            assert!(result.unwrap().is_none());
            group(&runtime, id, std::slice::from_ref(&payload), 1, false, 1);
            assert_eq!(refs(&payload), 2);
        } else {
            resource(result);
            group(&runtime, id, &[Value::Undefined], 2, false, 1);
            assert_eq!(refs(&payload), 1);
        }
        called(&runtime, element, id, 0, work == 28);
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        assert_eq!(runtime.objects.len(), objects);
    }
    for work in [11, 12] {
        let mut runtime = Runtime::try_new().unwrap();
        let id = raw_element(&mut runtime, AllCaptureId(usize::MAX), usize::MAX, true);
        let objects = runtime.objects.len();
        budget(&mut runtime, work, 0);
        let result = runtime.complete_all_element(id, &Value::Undefined);
        if work == 12 {
            assert!(result.unwrap().is_none());
        } else {
            resource(result);
        }
        called(&runtime, id, AllCaptureId(usize::MAX), usize::MAX, true);
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        assert_eq!(runtime.objects.len(), objects);
        quiet(&runtime);
    }
}
