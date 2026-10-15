//! Source-derived selected Any cuts; fixture work precedes measured resets.
use super::*;
use std::{mem::size_of, rc::Rc};

fn refusal<T>(result: Result<T>) -> ScriptError {
    match result {
        Err(error) => error,
        Ok(_) => panic!("expected refusal"),
    }
}
fn reset(runtime: &mut Runtime, work: usize, heap: usize) {
    runtime.steps = work;
    runtime.allocated = MAX_HEAP - heap;
}
fn node_bytes() -> usize {
    16 * (size_of::<PropertyKey>() + size_of::<Property>()) + 32 * size_of::<usize>() + 64
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
    assert_eq!(
        (runtime.calls, runtime.eval_depth, runtime.stack_units),
        (0, 0, 0)
    );
}
fn owner(runtime: &Runtime) -> usize {
    let Value::Native(native) = &runtime.promises.intrinsics.as_ref().unwrap().constructor else {
        panic!("saved constructor")
    };
    native.properties.unwrap().get()
}
fn exact<T>(values: &mut Vec<T>) {
    *values = std::mem::take(values).into_boxed_slice().into_vec();
    assert_eq!(values.len(), values.capacity());
}
fn before_any() -> Runtime {
    let mut runtime = Runtime::uninitialized();
    runtime.steps = 100_797;
    assert!(
        runtime
            .initialize_bootstrap()
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.steps, 0);
    assert_eq!(runtime.objects.len(), 754);
    assert!(runtime.objects.capacity() >= 755);
    assert_eq!(runtime.prototypes["AggregateError"], 752);
    assert_eq!(runtime.native_properties["AggregateError"], 753);
    assert_eq!(
        (
            runtime.native_properties.len(),
            runtime.prototypes.len(),
            runtime.environments[0].bindings.len()
        ),
        (300, 26, 213)
    );
    let bag = &runtime.objects[owner(&runtime)];
    assert_eq!((bag.values.len(), bag.order.len()), (9, 9));
    assert!(!bag.values.contains_key(&PropertyKey::from("any")));
    runtime
}
fn installer_fixture(grow: bool) -> Runtime {
    let mut runtime = before_any();
    let id = owner(&runtime);
    if grow {
        exact(&mut runtime.objects[id].order);
    } else {
        runtime.objects[id].order.try_reserve_exact(1).unwrap();
    }
    assert_eq!(
        runtime.objects[id].order.len() == runtime.objects[id].order.capacity(),
        grow
    );
    runtime
}
fn installer_heap(grow: bool) -> usize {
    size_of::<Option<AbortSlot>>()
        + size_of::<Option<f64>>()
        + (if grow { 12 } else { 2 }) * size_of::<PropertyKey>()
        + node_bytes()
        + size_of::<Native>()
        + 1179
}
fn installed(runtime: &Runtime, success: bool) {
    assert_eq!(runtime.objects.len(), 755);
    assert_eq!(runtime.prototypes["AggregateError"], 752);
    assert_eq!(runtime.native_properties["AggregateError"], 753);
    let bag = &runtime.objects[owner(runtime)];
    assert_eq!(
        (bag.values.len(), bag.order.len()),
        if success { (10, 10) } else { (9, 9) }
    );
    assert_eq!(bag.values.contains_key(&PropertyKey::from("any")), success);
    if success {
        let PropertyValue::Data {
            value: Value::Native(native),
            writable,
        } = &bag.values[&PropertyKey::from("any")].value
        else {
            panic!("method property")
        };
        assert!(*writable);
        assert_eq!(native.name, "Promise.any");
        assert_eq!(native.properties.unwrap().get(), 754);
        let property = &bag.values[&PropertyKey::from("any")];
        assert!(!property.enumerable && property.configurable);
    }
    quiet(runtime);
}

#[test]
fn any_late_installer171_or189_work_cuts_preserve_all_prior_ids() {
    for grow in [false, true] {
        let total = if grow { 189 } else { 171 };
        for success in [false, true] {
            let mut runtime = installer_fixture(grow);
            let capacity = runtime.objects.capacity();
            reset(
                &mut runtime,
                total - usize::from(!success),
                installer_heap(grow),
            );
            let result = runtime.install_promise_any_intrinsic();
            if success {
                result.unwrap();
            } else {
                assert!(refusal(result).is_resource_limit());
            }
            assert_eq!(runtime.steps, 0);
            assert_eq!(runtime.objects.capacity(), capacity);
            assert_eq!(
                runtime.allocated,
                if success {
                    MAX_HEAP
                } else {
                    MAX_HEAP
                        - 268
                        - if grow {
                            10 * size_of::<PropertyKey>()
                        } else {
                            0
                        }
                }
            );
            installed(&runtime, success);
        }
    }
}

#[test]
fn any_late_installer_typed_heap_exact_and_one_short_precede_owner_publication() {
    for grow in [false, true] {
        for success in [false, true] {
            let mut runtime = installer_fixture(grow);
            reset(
                &mut runtime,
                if grow { 189 } else { 171 },
                installer_heap(grow) - usize::from(!success),
            );
            let result = runtime.install_promise_any_intrinsic();
            if success {
                result.unwrap();
            } else {
                assert!(refusal(result).is_resource_limit());
            }
            assert_eq!(
                (runtime.steps, runtime.allocated),
                (0, if success { MAX_HEAP } else { MAX_HEAP + 1 })
            );
            installed(&runtime, success);
        }
    }
}

#[test]
fn any_late_installer12_admission_and_shape_guards_precede_text_or_method_allocation() {
    for bad in 0..4 {
        for work in [11, 12] {
            let mut runtime = before_any();
            let id = owner(&runtime);
            match bad {
                0 => (),
                1 => runtime.promises.intrinsics = None,
                2 => runtime.promises.intrinsics.as_mut().unwrap().constructor = Value::Undefined,
                3 => {
                    assert!(
                        runtime.objects[id]
                            .values
                            .remove(&PropertyKey::from("length"))
                            .is_some()
                    );
                }
                _ => unreachable!(),
            }
            reset(&mut runtime, work, 0);
            assert!(refusal(runtime.install_promise_any_intrinsic()).is_resource_limit());
            assert_eq!(
                (runtime.steps, runtime.allocated, runtime.objects.len()),
                (0, MAX_HEAP, 754)
            );
            assert!(
                !runtime.objects[id]
                    .values
                    .contains_key(&PropertyKey::from("any"))
            );
            quiet(&runtime);
        }
    }
    let mut runtime = installer_fixture(false);
    runtime.steps = 171;
    runtime.install_promise_any_intrinsic().unwrap();
    let allocated = runtime.allocated;
    runtime.steps = 12;
    assert!(refusal(runtime.install_promise_any_intrinsic()).is_resource_limit());
    assert_eq!((runtime.steps, runtime.allocated), (0, allocated));
    installed(&runtime, true);
}

#[test]
fn any_raw_startup_observes_physical_order_branch_and_keeps_public_reset() {
    let mut runtime = before_any();
    let id = owner(&runtime);
    let before_order = (
        runtime.objects[id].order.len(),
        runtime.objects[id].order.capacity(),
    );
    let before_objects = (runtime.objects.len(), runtime.objects.capacity());
    let before_heap = runtime.allocated;
    let grow = before_order.0 == before_order.1;
    let work = if grow { 189 } else { 171 };
    runtime.steps = work;
    runtime.install_promise_any_intrinsic().unwrap();
    assert_eq!(runtime.steps, 0);
    assert_eq!(runtime.allocated - before_heap, installer_heap(grow));
    let after_order = (
        runtime.objects[id].order.len(),
        runtime.objects[id].order.capacity(),
    );
    let raw = Runtime::uninitialized().finish_bootstrap().unwrap();
    let raw_id = owner(&raw);
    // The retained capacity observation selects the formula; no allocator ABI is guessed.
    assert_eq!(raw.steps, BOOTSTRAP_STEPS - 100_797 - work);
    assert_eq!(raw.allocated, runtime.allocated);
    assert_eq!((raw.objects.len(), raw.objects.capacity()), (755, 755));
    assert_eq!(
        (
            raw.native_properties.len(),
            raw.prototypes.len(),
            raw.environments[0].bindings.len()
        ),
        (300, 26, 213)
    );
    eprintln!(
        "any startup: before_order={before_order:?} after_order={after_order:?} before_objects={before_objects:?} raw_objects={:?} raw_order={:?} work={} remaining={} allocated={} maps={:?}",
        (raw.objects.len(), raw.objects.capacity()),
        (
            raw.objects[raw_id].order.len(),
            raw.objects[raw_id].order.capacity()
        ),
        BOOTSTRAP_STEPS - raw.steps,
        raw.steps,
        raw.allocated,
        (
            raw.native_properties.len(),
            raw.prototypes.len(),
            raw.environments[0].bindings.len()
        )
    );
    let public = Runtime::try_new().unwrap();
    assert_eq!(public.steps, MAX_STEPS);
    assert_eq!(public.allocated, raw.allocated);
    quiet(&public);
}

fn warm_destinations(runtime: &mut Runtime) {
    runtime.objects.try_reserve_exact(2).unwrap();
    assert!(runtime.objects.capacity() >= runtime.objects.len() + 2);
    runtime.arrays.try_reserve_exact(1).unwrap();
    runtime.array_properties.try_reserve_exact(1).unwrap();
    runtime.array_holes.try_reserve_exact(1).unwrap();
    runtime.array_lengths.try_reserve_exact(1).unwrap();
    assert!(runtime.arrays.capacity() > runtime.arrays.len());
    assert!(runtime.array_properties.capacity() > runtime.array_properties.len());
    assert!(runtime.array_holes.capacity() > runtime.array_holes.len());
    assert!(runtime.array_lengths.capacity() > runtime.array_lengths.len());
}
fn ready_fixture(n: usize) -> (Runtime, Vec<Value>, Value) {
    let mut runtime = Runtime::try_new().unwrap();
    let marker = Runtime::native("Object.is", Value::Undefined);
    // Real paid Group/placeholder/first-completion ownership precedes the reset.
    let group = runtime
        .new_all_group(Runtime::native("Object.is", Value::Undefined))
        .unwrap();
    for _ in 0..n {
        let index = runtime.append_all_placeholder(group).unwrap();
        runtime.new_all_resolve_element(group, index).unwrap();
        let element = AllCaptureId(runtime.promises.all_captures.records.len() - 1);
        assert!(
            runtime
                .complete_all_element(element, &marker)
                .unwrap()
                .is_none()
        );
    }
    let ready = runtime.finish_all_input(group).unwrap().unwrap();
    assert_eq!(refs(&marker), n + 1);
    warm_destinations(&mut runtime);
    (runtime, ready.values, marker)
}
fn array_plan_heap(n: usize) -> usize {
    32 + n * size_of::<Value>()
        + size_of::<ScriptObject>()
        + size_of::<Vec<Value>>()
        + size_of::<usize>()
        + size_of::<BTreeSet<usize>>()
        + size_of::<array_properties::ArrayLength>()
}
fn base_bag() -> usize {
    72 + size_of::<Option<AbortSlot>>() + size_of::<Option<f64>>()
}
fn intrinsic_heap(n: usize) -> usize {
    2 * size_of::<ScriptObject>()
        + 2 * size_of::<Option<AbortSlot>>()
        + 2 * size_of::<Option<f64>>()
        + 3 * size_of::<PropertyKey>()
        + node_bytes()
        + n * size_of::<Value>()
        + size_of::<Vec<Value>>()
        + size_of::<usize>()
        + size_of::<BTreeSet<usize>>()
        + size_of::<array_properties::ArrayLength>()
        + 544
}
fn intrinsic_result(
    runtime: &Runtime,
    result: Value,
    objects: usize,
    arrays: usize,
    marker: &Value,
    n: usize,
) {
    assert_eq!(result, Value::Object(objects));
    let object = &runtime.objects[objects];
    assert_eq!(object.intrinsic_error, Some("AggregateError"));
    assert_eq!(
        object.prototype,
        Some(Value::Object(runtime.prototypes["AggregateError"]))
    );
    assert_eq!(object.order, [PropertyKey::from("errors")]);
    let property = &object.values[&PropertyKey::from("errors")];
    assert!(!property.enumerable && property.configurable);
    let PropertyValue::Data { value, writable } = &property.value else {
        panic!("errors data")
    };
    assert!(*writable);
    assert_eq!(value, &Value::Array(arrays));
    assert_eq!(runtime.arrays[arrays].len(), n);
    for value in &runtime.arrays[arrays] {
        assert_eq!(value, marker);
    }
    assert!(runtime.array_holes[arrays].is_empty());
    assert_eq!(refs(marker), n + 1);
}

#[test]
fn any_intrinsic134_work_prefixes_preserve_paid_list_ownership_and_partial_publication() {
    for n in [0, 2] {
        for work in [
            7, 8, 51, 52, 59, 60, 67, 68, 69, 91, 92, 99, 100, 101, 104, 105, 124, 125, 133, 134,
        ] {
            let (mut runtime, values, marker) = ready_fixture(n);
            let objects = runtime.objects.len();
            let arrays = runtime.arrays.len();
            reset(&mut runtime, work, intrinsic_heap(n));
            let result = runtime.intrinsic_aggregate_error_from_list(values);
            assert_eq!(runtime.steps, 0);
            let added_objects = usize::from(work >= 69) + usize::from(work >= 101);
            assert_eq!(
                (runtime.objects.len(), runtime.arrays.len()),
                (objects + added_objects, arrays + usize::from(work >= 101))
            );
            let spent = usize::from(work >= 60) * size_of::<ScriptObject>()
                + usize::from(work >= 68) * base_bag()
                + usize::from(work >= 69) * 3 * size_of::<PropertyKey>()
                + usize::from(work >= 92) * array_plan_heap(n)
                + usize::from(work >= 100) * base_bag()
                + usize::from(work >= 125) * 88
                + usize::from(work >= 134) * (280 + node_bytes());
            assert_eq!(runtime.allocated, MAX_HEAP - intrinsic_heap(n) + spent);
            if work == 134 {
                intrinsic_result(&runtime, result.unwrap(), objects, arrays, &marker, n);
            } else {
                assert!(refusal(result).is_resource_limit());
                assert!(runtime.microtasks_suspended());
                assert_eq!(refs(&marker), if work >= 101 { n + 1 } else { 1 });
            }
            quiet(&runtime);
        }
    }
}

#[test]
fn any_intrinsic_typed_heap_exact_and_one_short_keep_dense_errors_identity() {
    for n in [0, 2] {
        for success in [false, true] {
            let (mut runtime, values, marker) = ready_fixture(n);
            let objects = runtime.objects.len();
            let arrays = runtime.arrays.len();
            reset(&mut runtime, 134, intrinsic_heap(n) - usize::from(!success));
            let result = runtime.intrinsic_aggregate_error_from_list(values);
            assert_eq!(
                (runtime.steps, runtime.allocated),
                (0, if success { MAX_HEAP } else { MAX_HEAP + 1 })
            );
            assert_eq!(
                (runtime.objects.len(), runtime.arrays.len()),
                (objects + 2, arrays + 1)
            );
            if success {
                intrinsic_result(&runtime, result.unwrap(), objects, arrays, &marker, n);
            } else {
                assert!(refusal(result).is_resource_limit());
                assert!(runtime.objects[objects].values.is_empty());
                assert!(runtime.microtasks_suspended());
                assert_eq!(refs(&marker), n + 1);
            }
            quiet(&runtime);
        }
    }
}

#[test]
fn any_intrinsic_array_reserve_faults_leave_only_the_branded_object() {
    use super::all_allocation_probe::{ReserveSite, fail_once};
    for (index, site) in [
        (3, ReserveSite::ArrayValues),
        (4, ReserveSite::ArrayProperties),
        (5, ReserveSite::ArrayHoles),
        (6, ReserveSite::ArrayLengths),
    ] {
        let (mut runtime, values, marker) = ready_fixture(2);
        match site {
            ReserveSite::ArrayValues => exact(&mut runtime.arrays),
            ReserveSite::ArrayProperties => exact(&mut runtime.array_properties),
            ReserveSite::ArrayHoles => exact(&mut runtime.array_holes),
            ReserveSite::ArrayLengths => exact(&mut runtime.array_lengths),
            _ => unreachable!(),
        }
        let objects = runtime.objects.len();
        let arrays = runtime.arrays.len();
        reset(&mut runtime, MAX_STEPS, MAX_HEAP);
        let guard = fail_once(site);
        assert!(refusal(runtime.intrinsic_aggregate_error_from_list(values)).is_resource_limit());
        let mut reached = [0; 7];
        reached[index] = 1;
        assert_eq!(guard.observation().attempts, reached);
        assert_eq!(guard.observation().completed, [0; 7]);
        assert_eq!(
            (
                runtime.objects.len(),
                runtime.arrays.len(),
                runtime.array_properties.len(),
                runtime.array_holes.len(),
                runtime.array_lengths.len()
            ),
            (objects + 1, arrays, arrays, arrays, arrays)
        );
        assert_eq!(
            runtime.objects[objects].intrinsic_error,
            Some("AggregateError")
        );
        assert!(runtime.objects[objects].values.is_empty());
        assert_eq!(refs(&marker), 1);
        assert!(runtime.microtasks_suspended());
        quiet(&runtime);
    }
}

#[test]
fn promise_any_body17_and_constructor20_stop_before_private_publication() {
    for work in [16, 17, 18, 19, 20] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut doc = Document::parse("");
        let retained = Runtime::native("Object.is", Value::Undefined);
        let input = retained.clone();
        let objects = runtime.objects.len();
        reset(&mut runtime, work, 0);
        let failure = refusal(runtime.promise_any(input, &[], &mut doc));
        if work == 20 {
            assert!(matches!(failure.kind, ErrorKind::Runtime("TypeError")));
        } else {
            assert!(failure.is_resource_limit());
        }
        assert_eq!(
            (runtime.steps, runtime.allocated, runtime.objects.len()),
            (0, MAX_HEAP, objects)
        );
        assert_eq!(refs(&retained), 1);
        assert!(runtime.promises.captures.records.is_empty());
        assert!(runtime.promises.callbacks.records.is_empty());
        assert!(runtime.promises.all_captures.records.is_empty());
        quiet(&runtime);
    }
}

#[test]
fn promise_any_public_wrapper8_observes_entry_refusal_with_old_terminal_policy() {
    for work in [7, 8, 24, 25, 26, 27, 28] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut doc = Document::parse("");
        let objects = runtime.objects.len();
        reset(&mut runtime, work, 0);
        let failure = refusal(runtime.promise_native_operation(
            "Promise.any",
            Value::Bool(false),
            &[],
            &mut doc,
        ));
        if work == 28 {
            assert!(matches!(failure.kind, ErrorKind::Runtime("TypeError")));
            assert!(!runtime.microtasks_suspended());
        } else {
            assert!(failure.is_resource_limit());
            assert!(runtime.microtasks_suspended());
        }
        assert_eq!(
            (runtime.steps, runtime.allocated, runtime.objects.len()),
            (0, MAX_HEAP, objects)
        );
        assert!(runtime.promises.captures.records.is_empty());
        assert!(runtime.promises.callbacks.records.is_empty());
        assert!(runtime.promises.all_captures.records.is_empty());
        quiet(&runtime);
    }
}

fn warm<T>(arena: &mut Arena<T>) {
    arena
        .records
        .try_reserve_exact(8 - arena.records.len())
        .unwrap();
    arena.admitted_capacity = 8;
}
fn callback_fixture(cold: bool) -> (Runtime, AllCaptureId) {
    let mut runtime = Runtime::try_new().unwrap();
    let group = runtime
        .new_all_group(Runtime::native("Object.is", Value::Undefined))
        .unwrap();
    assert_eq!(runtime.append_all_placeholder(group).unwrap(), 0);
    runtime.objects.try_reserve_exact(1).unwrap();
    assert!(runtime.objects.capacity() > runtime.objects.len());
    if cold {
        exact(&mut runtime.promises.all_captures.records);
        runtime.promises.all_captures.admitted_capacity = 1;
        assert_eq!(runtime.promises.callbacks.admitted_capacity, 0);
    } else {
        warm(&mut runtime.promises.all_captures);
        warm(&mut runtime.promises.callbacks);
    }
    (runtime, group)
}
fn callback_heap() -> usize {
    904 + size_of::<Option<AbortSlot>>()
        + size_of::<Option<f64>>()
        + 2 * size_of::<PropertyKey>()
        + node_bytes()
        + size_of::<Native>()
        + size_of::<ScriptObject>()
}
fn group_state(
    runtime: &Runtime,
    group: AllCaptureId,
    remaining: usize,
    value: Value,
    called: Option<bool>,
) {
    let AllCapture::Group {
        values,
        remaining: actual,
        input_done,
        ..
    } = &runtime.promises.all_captures.records[group.0]
    else {
        panic!("Group")
    };
    assert_eq!((*actual, *input_done), (remaining, false));
    assert_eq!(values.records, [value]);
    if let Some(expected) = called {
        let AllCapture::Element { called, .. } = &runtime.promises.all_captures.records[1] else {
            panic!("Element")
        };
        assert_eq!(*called, expected);
    }
}

#[test]
fn any_reject_factory_reuses136_and_typed_growth_before_count_publication() {
    for cold in [false, true] {
        let growth = if cold {
            16 + 2 * size_of::<AllCapture>().div_ceil(size_of::<Value>())
        } else {
            0
        };
        let work = 136 + growth;
        let heap = callback_heap()
            + if cold {
                64 + 8 * (size_of::<AllCapture>() + size_of::<CallbackRecord>())
            } else {
                0
            };
        for (work_cut, heap_cut, success) in [
            (work, heap, true),
            (work - 1, heap, false),
            (work, heap - 1, false),
        ] {
            let (mut runtime, group) = callback_fixture(cold);
            let objects = runtime.objects.len();
            reset(&mut runtime, work_cut, heap_cut);
            let result = runtime.new_any_reject_element(group, 0);
            assert_eq!(runtime.objects.len(), objects + 1);
            assert_eq!(runtime.steps, 0);
            if success {
                let Value::Native(callback) = result.unwrap() else {
                    panic!("callback")
                };
                assert_eq!(callback.properties.unwrap().get(), objects);
                assert_eq!(runtime.promises.callbacks.records.len(), 1);
                assert!(matches!(
                    runtime.promises.callbacks.records[0].kind,
                    CallbackKind::AnyReject(AllCaptureId(1))
                ));
                group_state(&runtime, group, 2, Value::Undefined, Some(false));
                assert_eq!(runtime.allocated, MAX_HEAP);
            } else {
                assert!(refusal(result).is_resource_limit());
                assert!(runtime.promises.callbacks.records.is_empty());
                assert_eq!(runtime.promises.all_captures.records.len(), 1);
                group_state(&runtime, group, 1, Value::Undefined, None);
                assert_eq!(
                    (
                        runtime.promises.all_captures.admitted_capacity,
                        runtime.promises.callbacks.admitted_capacity
                    ),
                    if cold { (1, 0) } else { (8, 8) }
                );
                assert_eq!(
                    runtime.allocated,
                    if work_cut < work {
                        MAX_HEAP - size_of::<Native>() - 48
                    } else {
                        MAX_HEAP + 1
                    }
                );
            }
            quiet(&runtime);
        }
    }
}

#[test]
fn any_dispatch38_or22_preserves_one_gate_and_duplicates_after_storage_transfer() {
    for phase in 0..3 {
        let total = if phase == 0 { 38 } else { 22 };
        for work in [total - 1, total] {
            let (mut runtime, group) = callback_fixture(false);
            let Value::Native(callback) = runtime.new_any_reject_element(group, 0).unwrap() else {
                panic!("callback")
            };
            if phase != 0 {
                assert!(
                    runtime
                        .complete_all_element(AllCaptureId(1), &Value::Number(3.0))
                        .unwrap()
                        .is_none()
                );
            }
            if phase == 2 {
                let ready = runtime.finish_all_input(group).unwrap().unwrap();
                assert_eq!(ready.values, [Value::Number(3.0)]);
            }
            let arrays = runtime.arrays.len();
            let mut doc = Document::parse("");
            reset(&mut runtime, work, 0);
            let result = runtime.promise_callback(&callback, &[Value::Number(17.0)], &mut doc);
            if work == total {
                assert_eq!(result.unwrap(), Value::Undefined);
            } else {
                assert!(refusal(result).is_resource_limit());
            }
            assert_eq!(
                (runtime.steps, runtime.allocated, runtime.arrays.len()),
                (0, MAX_HEAP, arrays)
            );
            if phase == 2 {
                let AllCapture::Group {
                    values,
                    remaining,
                    input_done,
                    ..
                } = &runtime.promises.all_captures.records[group.0]
                else {
                    panic!("Group")
                };
                assert_eq!(
                    (
                        *remaining,
                        *input_done,
                        values.records.len(),
                        values.admitted_capacity
                    ),
                    (0, true, 0, 0)
                );
            } else {
                let changed = phase == 0 && work == total;
                group_state(
                    &runtime,
                    group,
                    if phase != 0 || changed { 1 } else { 2 },
                    if phase != 0 {
                        Value::Number(3.0)
                    } else if changed {
                        Value::Number(17.0)
                    } else {
                        Value::Undefined
                    },
                    Some(phase != 0 || changed),
                );
            }
            quiet(&runtime);
        }
    }
}
