//! Independent selected Promise.try quotas, frozen before production inspection.
use super::*;
use std::{mem::size_of, num::NonZeroUsize, rc::Rc};

fn failure<T>(result: Result<T>) -> ScriptError {
    match result {
        Err(error) => error,
        Ok(_) => panic!("expected refusal"),
    }
}
fn reset(runtime: &mut Runtime, work: usize, heap: usize) {
    runtime.steps = work;
    runtime.allocated = MAX_HEAP - heap;
}
fn quiet(runtime: &Runtime) {
    assert!(runtime.frames.is_empty());
    assert!(runtime.console.is_empty());
    assert_eq!(
        (runtime.calls, runtime.eval_depth, runtime.stack_units),
        (0, 0, 0)
    );
}
fn refs(value: &Value) -> usize {
    let Value::Native(native) = value else {
        panic!("native marker")
    };
    Rc::strong_count(native)
}
fn same(left: &Value, right: &Value) {
    let (Value::Native(left), Value::Native(right)) = (left, right) else {
        panic!("native markers")
    };
    assert!(Rc::ptr_eq(left, right));
}
fn markers(n: usize) -> Vec<Value> {
    (0..n)
        .map(|_| Runtime::native("Object.is", Value::Undefined))
        .collect()
}
fn node_bytes() -> usize {
    16 * (size_of::<PropertyKey>() + size_of::<Property>()) + 32 * size_of::<usize>() + 64
}
fn bag_bytes() -> usize {
    72 + size_of::<Option<AbortSlot>>() + size_of::<Option<f64>>()
}
fn tail_heap(n: usize) -> usize {
    32 + n * size_of::<Value>()
}
fn installer_heap(grow: bool) -> usize {
    size_of::<Option<AbortSlot>>()
        + size_of::<Option<f64>>()
        + (if grow { 14 } else { 2 }) * size_of::<PropertyKey>()
        + 4 * node_bytes()
        + size_of::<Native>()
        + 1179
}
fn exact<T>(values: &mut Vec<T>) {
    *values = std::mem::take(values).into_boxed_slice().into_vec();
    assert_eq!(values.len(), values.capacity());
}
fn owner(runtime: &Runtime) -> usize {
    let Value::Native(native) = &runtime.promises.intrinsics.as_ref().unwrap().constructor else {
        panic!("saved constructor")
    };
    native.properties.unwrap().get()
}

#[test]
fn promise_try_tail_exact_work_heap_and_handle_identity() {
    for n in [0, 1, 3] {
        for mode in 0..3 {
            let mut runtime = Runtime::try_new().unwrap();
            let values = markers(n);
            let total = 8 + 3 * n;
            let heap = tail_heap(n);
            reset(
                &mut runtime,
                total - usize::from(mode == 0),
                heap - usize::from(mode == 1),
            );
            let result = runtime.promise_try_arguments(&values);
            assert_eq!(runtime.steps, 0);
            if mode == 2 {
                let copied = result.unwrap();
                assert_eq!(copied.len(), n);
                assert!(copied.capacity() >= n);
                for (original, copy) in values.iter().zip(&copied) {
                    same(original, copy);
                    assert_eq!(refs(original), 2);
                }
                drop(copied);
                assert_eq!(runtime.allocated, MAX_HEAP);
                assert!(!runtime.microtasks_suspended());
            } else {
                assert!(failure(result).is_resource_limit());
                assert_eq!(
                    runtime.allocated,
                    if mode == 0 {
                        MAX_HEAP - heap
                    } else {
                        MAX_HEAP + 1
                    }
                );
                assert!(runtime.microtasks_suspended());
            }
            assert!(values.iter().all(|value| refs(value) == 1));
            quiet(&runtime);
        }
    }
}

#[test]
fn promise_try_tail_limit_precedes_storage_and_maximum_is_inclusive() {
    let oversized = vec![Value::Undefined; 65_537];
    for work in [7, 8, 9] {
        let mut runtime = Runtime::try_new().unwrap();
        reset(&mut runtime, work, 0);
        assert!(failure(runtime.promise_try_arguments(&oversized)).is_resource_limit());
        assert_eq!(
            (runtime.steps, runtime.allocated),
            (work.saturating_sub(8), MAX_HEAP)
        );
        assert!(runtime.microtasks_suspended());
        quiet(&runtime);
    }
    let values = vec![Value::Undefined; 65_536];
    let mut runtime = Runtime::try_new().unwrap();
    // Direct inert helper allowance proves the internal ceiling only. It does
    // not assert this tail is reachable within the public100000 work allowance.
    reset(&mut runtime, 8 + 3 * values.len(), tail_heap(values.len()));
    let copied = runtime.promise_try_arguments(&values).unwrap();
    assert_eq!(copied, values);
    assert!(copied.capacity() >= copied.len());
    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    assert!(!runtime.microtasks_suspended());
    quiet(&runtime);
}

#[test]
fn promise_try_public8_body16_and_callback1_precede_tail() {
    for work in [7, 8, 23, 24] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut doc = Document::parse("");
        let objects = runtime.objects.len();
        reset(&mut runtime, work, 0);
        let error = failure(runtime.promise_native_operation(
            "Promise.try",
            Value::Bool(false),
            &[],
            &mut doc,
        ));
        if work == 24 {
            assert!(matches!(error.kind, ErrorKind::Runtime("TypeError")));
            assert!(!runtime.microtasks_suspended());
        } else {
            assert!(error.is_resource_limit());
            assert!(runtime.microtasks_suspended());
        }
        assert_eq!(
            (runtime.steps, runtime.allocated, runtime.objects.len()),
            (0, MAX_HEAP, objects)
        );
        assert!(runtime.promises.captures.records.is_empty());
        assert!(runtime.promises.callbacks.records.is_empty());
        quiet(&runtime);
    }
    for work in [24, 25, 32, 33] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut doc = Document::parse("");
        let receiver = Value::Object(runtime.prototypes["Object"]);
        let objects = runtime.objects.len();
        reset(&mut runtime, work, 32);
        assert!(
            failure(runtime.promise_native_operation("Promise.try", receiver, &[], &mut doc))
                .is_resource_limit()
        );
        assert_eq!(
            (runtime.steps, runtime.allocated),
            (0, MAX_HEAP - 32 + usize::from(work == 33) * 32)
        );
        assert_eq!(runtime.objects.len(), objects);
        assert!(runtime.promises.captures.records.is_empty());
        assert!(runtime.promises.callbacks.records.is_empty());
        assert!(runtime.microtasks_suspended());
        quiet(&runtime);
    }
}

#[test]
fn promise_try_call_first_tick_releases_copies_and_outer_observer_suspends() {
    for n in [0, 2] {
        let retained = markers(n + 1);
        let mut runtime = Runtime::try_new().unwrap();
        let mut doc = Document::parse("");
        runtime.work(n + 1).unwrap();
        let args = retained.clone();
        assert!(retained.iter().all(|value| refs(value) == 2));
        let objects = runtime.objects.len();
        let receiver = Value::Object(runtime.prototypes["Object"]);
        reset(&mut runtime, 33 + 3 * n, tail_heap(n));
        assert!(
            failure(runtime.promise_native_operation("Promise.try", receiver, &args, &mut doc))
                .is_resource_limit()
        );
        assert_eq!(
            (runtime.steps, runtime.allocated, runtime.objects.len()),
            (0, MAX_HEAP, objects)
        );
        assert!(retained.iter().all(|value| refs(value) == 2));
        assert!(runtime.promises.captures.records.is_empty());
        assert!(runtime.promises.callbacks.records.is_empty());
        assert!(runtime.microtasks_suspended());
        quiet(&runtime);
        // Old Call's first tick precedes its own machine observer. This paired
        // control makes the selected outer Promise observer obligation explicit.
        let mut control = Runtime::try_new().unwrap();
        control.work(9 + 3 * n).unwrap();
        let callback = retained[0].clone();
        let tail = retained[1..].to_vec();
        reset(&mut control, 0, 0);
        assert!(
            failure(control.call(callback, tail, Value::Undefined, &mut doc)).is_resource_limit()
        );
        assert_eq!((control.steps, control.allocated), (0, MAX_HEAP));
        assert!(!control.microtasks_suspended());
        assert!(retained.iter().all(|value| refs(value) == 2));
        quiet(&control);
    }
}

fn before_try() -> (Runtime, usize, usize) {
    let mut runtime = Runtime::uninitialized();
    runtime.steps = 100_797;
    assert!(failure(runtime.initialize_bootstrap()).is_resource_limit());
    assert_eq!((runtime.steps, runtime.objects.len()), (0, 754));
    assert!(runtime.objects.capacity() >= 757);
    assert_eq!(runtime.prototypes["AggregateError"], 752);
    assert_eq!(runtime.native_properties["AggregateError"], 753);
    let id = owner(&runtime);
    assert_eq!(
        (
            runtime.objects[id].values.len(),
            runtime.objects[id].order.len()
        ),
        (9, 9)
    );
    let any_work = if runtime.objects[id].order.len() == runtime.objects[id].order.capacity() {
        189
    } else {
        171
    };
    runtime.steps = any_work;
    runtime.install_promise_any_intrinsic().unwrap();
    assert_eq!((runtime.steps, runtime.objects.len()), (0, 755));
    let with_resolvers_work =
        if runtime.objects[id].order.len() == runtime.objects[id].order.capacity() {
            326
        } else {
            306
        };
    runtime.steps = with_resolvers_work;
    runtime.install_promise_with_resolvers_intrinsic().unwrap();
    assert_eq!((runtime.steps, runtime.objects.len()), (0, 756));
    assert_eq!(
        (
            runtime.objects[id].values.len(),
            runtime.objects[id].order.len()
        ),
        (11, 11)
    );
    assert!(
        !runtime.objects[id]
            .values
            .contains_key(&PropertyKey::from("try"))
    );
    (runtime, any_work, with_resolvers_work)
}
fn installer_fixture(grow: bool) -> Runtime {
    let (mut runtime, _, _) = before_try();
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
fn state(runtime: &Runtime, metadata: bool, fields: usize, published: bool) {
    assert_eq!(runtime.objects.len(), 756 + usize::from(metadata));
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
    let bag = &runtime.objects[owner(runtime)];
    assert_eq!(
        (bag.values.len(), bag.order.len()),
        if published { (12, 12) } else { (11, 11) }
    );
    for (name, id) in [("any", 754), ("withResolvers", 755)] {
        let PropertyValue::Data {
            value: Value::Native(native),
            ..
        } = &bag.values[&PropertyKey::from(name)].value
        else {
            panic!("old method")
        };
        assert_eq!(native.properties.unwrap().get(), id);
    }
    assert_eq!(
        bag.values.contains_key(&PropertyKey::from("try")),
        published
    );
    if metadata {
        let method = &runtime.objects[756];
        assert_eq!(
            method.prototype,
            Some(Value::Function(runtime.function_prototype))
        );
        assert_eq!(
            method.order,
            [PropertyKey::from("length"), PropertyKey::from("name")][..fields]
        );
        assert_eq!(method.values.len(), fields);
        for (index, (key, expected)) in [
            ("length", Value::Number(1.0)),
            ("name", Value::String("try".into())),
        ]
        .into_iter()
        .enumerate()
        {
            if index >= fields {
                break;
            }
            let property = &method.values[&PropertyKey::from(key)];
            let PropertyValue::Data { value, writable } = &property.value else {
                panic!("metadata data")
            };
            assert_eq!(value, &expected);
            assert!(!*writable && !property.enumerable && property.configurable);
        }
    }
    if published {
        assert_eq!(bag.order.last(), Some(&PropertyKey::from("try")));
        let property = &bag.values[&PropertyKey::from("try")];
        let PropertyValue::Data {
            value: Value::Native(native),
            writable,
        } = &property.value
        else {
            panic!("published method")
        };
        assert_eq!(native.name, "Promise.try");
        assert_eq!(native.properties.unwrap().get(), 756);
        assert_eq!(native.receiver, Value::Undefined);
        assert!(*writable && !property.enumerable && property.configurable);
    }
    quiet(runtime);
}

#[test]
fn promise_try_installer12_shape_guards_precede_text() {
    for bad in 0..7 {
        for work in [11, 12] {
            let (mut runtime, _, _) = before_try();
            let id = owner(&runtime);
            match bad {
                0 => (),
                1 => runtime.promises.intrinsics = None,
                2 => runtime.promises.intrinsics.as_mut().unwrap().constructor = Value::Undefined,
                3 => {
                    runtime.promises.intrinsics.as_mut().unwrap().constructor =
                        Runtime::native("Promise", Value::Window)
                }
                4 => {
                    runtime.promises.intrinsics.as_mut().unwrap().constructor =
                        Value::Native(Rc::new(Native {
                            name: "Promise".into(),
                            receiver: Value::Window,
                            properties: NonZeroUsize::new(runtime.objects.len()),
                        }))
                }
                5 => {
                    runtime.objects[id]
                        .values
                        .remove(&PropertyKey::from("length"))
                        .unwrap();
                }
                6 => {
                    runtime.objects[id].order.pop().unwrap();
                }
                _ => unreachable!(),
            }
            reset(&mut runtime, work, 0);
            assert!(failure(runtime.install_promise_try_intrinsic()).is_resource_limit());
            assert_eq!(
                (runtime.steps, runtime.allocated, runtime.objects.len()),
                (0, MAX_HEAP, 756)
            );
            assert!(
                !runtime.objects[id]
                    .values
                    .contains_key(&PropertyKey::from("try"))
            );
            quiet(&runtime);
        }
    }
    let mut runtime = installer_fixture(false);
    runtime.steps = 246;
    runtime.install_promise_try_intrinsic().unwrap();
    let allocated = runtime.allocated;
    runtime.steps = 12;
    assert!(failure(runtime.install_promise_try_intrinsic()).is_resource_limit());
    assert_eq!((runtime.steps, runtime.allocated), (0, allocated));
    state(&runtime, true, 2, true);
}

#[test]
fn promise_try_installer_work_prefixes_keep_private_metadata_unpublished() {
    for grow in [true, false] {
        let total = if grow { 268 } else { 246 };
        let cuts: &[usize] = if grow {
            &[
                31, 32, 47, 48, 61, 62, 64, 65, 72, 73, 74, 82, 83, 97, 98, 116, 117, 267, 268,
            ]
        } else {
            &[245, 246]
        };
        for &work in cuts {
            let mut runtime = installer_fixture(grow);
            let capacity = runtime.objects.capacity();
            let heap = installer_heap(grow);
            reset(&mut runtime, work, heap);
            let result = runtime.install_promise_try_intrinsic();
            if work == total {
                result.unwrap();
            } else {
                assert!(failure(result).is_resource_limit());
            }
            let spent = usize::from(work >= 32) * 88
                + usize::from(work >= 48) * 80
                + usize::from(work >= 62) * 76
                + usize::from(work >= 73) * bag_bytes()
                + usize::from(work >= 74) * 2 * size_of::<PropertyKey>()
                + usize::from(work >= 83) * (280 + node_bytes())
                + usize::from(work >= 98) * 272
                + usize::from(work >= 117) * (size_of::<Native>() + 43)
                + usize::from(work == total)
                    * (268
                        + 3 * node_bytes()
                        + if grow {
                            12 * size_of::<PropertyKey>()
                        } else {
                            0
                        });
            assert_eq!(
                (runtime.steps, runtime.allocated),
                (0, MAX_HEAP - heap + spent)
            );
            assert_eq!(runtime.objects.capacity(), capacity);
            state(
                &runtime,
                work >= 74,
                usize::from(work >= 83) + usize::from(work >= 98),
                work == total,
            );
        }
    }
}

#[test]
fn promise_try_installer_typed_heap_boundaries_and_publication() {
    let stages = [
        (32, 88),
        (48, 80),
        (62, 76),
        (73, bag_bytes()),
        (74, 2 * size_of::<PropertyKey>()),
        (83, 280 + node_bytes()),
        (98, 272),
        (117, size_of::<Native>() + 43),
        (268, 268 + 3 * node_bytes() + 12 * size_of::<PropertyKey>()),
    ];
    let mut prefix = 0;
    for (index, (work, bytes)) in stages.into_iter().enumerate() {
        prefix += bytes;
        let mut runtime = installer_fixture(true);
        reset(&mut runtime, 268, prefix - 1);
        assert!(failure(runtime.install_promise_try_intrinsic()).is_resource_limit());
        assert_eq!(
            (runtime.steps, runtime.allocated),
            (268 - work, MAX_HEAP + 1)
        );
        state(
            &runtime,
            index > 4,
            usize::from(index > 5) + usize::from(index > 6),
            false,
        );
    }
    assert_eq!(prefix, installer_heap(true));
    let mut runtime = installer_fixture(false);
    reset(&mut runtime, 246, installer_heap(false) - 1);
    assert!(failure(runtime.install_promise_try_intrinsic()).is_resource_limit());
    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP + 1));
    state(&runtime, true, 2, false);
    for grow in [false, true] {
        let mut runtime = installer_fixture(grow);
        reset(
            &mut runtime,
            if grow { 268 } else { 246 },
            installer_heap(grow),
        );
        runtime.install_promise_try_intrinsic().unwrap();
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        state(&runtime, true, 2, true);
    }
}

#[test]
fn promise_try_installer_cold_object_reserve_keeps_paid_prefix() {
    for work in [72, 73] {
        let (mut runtime, _, _) = before_try();
        exact(&mut runtime.objects);
        assert_eq!(
            (runtime.objects.len(), runtime.objects.capacity()),
            (756, 756)
        );
        reset(&mut runtime, work, 244 + bag_bytes());
        assert!(failure(runtime.install_promise_try_intrinsic()).is_resource_limit());
        assert_eq!(
            (runtime.steps, runtime.allocated),
            (0, MAX_HEAP - usize::from(work == 72) * bag_bytes())
        );
        assert_eq!(
            (runtime.objects.len(), runtime.objects.capacity()),
            (756, 756)
        );
        state(&runtime, false, 0, false);
    }
}

#[test]
fn promise_try_startup_observes_actual_capacity_and_public_reset() {
    let (mut runtime, any_work, with_resolvers_work) = before_try();
    let id = owner(&runtime);
    let before_order = (
        runtime.objects[id].order.len(),
        runtime.objects[id].order.capacity(),
    );
    let before_objects = (runtime.objects.len(), runtime.objects.capacity());
    let before_heap = runtime.allocated;
    let grow = before_order.0 == before_order.1;
    let work = if grow { 268 } else { 246 };
    runtime.steps = work;
    runtime.install_promise_try_intrinsic().unwrap();
    assert_eq!(runtime.steps, 0);
    assert_eq!(runtime.allocated - before_heap, installer_heap(grow));
    state(&runtime, true, 2, true);
    let after_order = (
        runtime.objects[id].order.len(),
        runtime.objects[id].order.capacity(),
    );
    let raw = Runtime::uninitialized().finish_bootstrap().unwrap();
    assert_eq!(
        raw.steps,
        BOOTSTRAP_STEPS - 100_797 - any_work - with_resolvers_work - work
    );
    assert_eq!(raw.allocated, runtime.allocated);
    assert_eq!(raw.objects.len(), 757);
    assert!(raw.objects.capacity() >= 757);
    let raw_id = owner(&raw);
    assert_eq!(
        (
            raw.objects[raw_id].values.len(),
            raw.objects[raw_id].order.len()
        ),
        (12, 12)
    );
    assert_eq!(
        (
            raw.native_properties.len(),
            raw.prototypes.len(),
            raw.environments[0].bindings.len()
        ),
        (300, 26, 213)
    );
    eprintln!(
        "Promise.try startup: before_order={before_order:?} after_order={after_order:?} before_objects={before_objects:?} raw_objects={:?} raw_order={:?} work={} remaining={} allocated={} maps={:?}",
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
    assert!(!public.microtasks_suspended());
    quiet(&public);
}

fn terminal_fixture(strict: bool) -> (Runtime, Document, Value, Value) {
    const SOURCE: &str = "var trace='S',holder={},resolved=()=>{},rejected=()=>{trace+='J';};\nvar queued=()=>{trace+='Q';};\nfunction C(exec){trace+='C';exec(resolved,rejected);return holder;}\nC";
    let mut runtime = Runtime::try_new().unwrap();
    let mut doc = Document::parse("");
    let constructor = if strict {
        runtime.execute_strict(SOURCE, &mut doc)
    } else {
        runtime.execute(SOURCE, &mut doc)
    }
    .unwrap();
    let queued = runtime.environments[0].bindings["queued"].value.clone();
    runtime
        .queue_microtask(&Value::Undefined, std::slice::from_ref(&queued))
        .unwrap();
    runtime.objects.try_reserve_exact(16).unwrap();
    assert!(runtime.objects.capacity() >= runtime.objects.len() + 16);
    (runtime, doc, constructor, queued)
}

#[test]
fn promise_try_natural_materialization_refusal_keeps_constructor_effect_and_queue_sticky() {
    for strict in [false, true] {
        let (mut control, mut control_doc, control_constructor, _) = terminal_fixture(strict);
        let initial_objects = control.objects.len();
        let initial_heap = control.allocated;
        let original = failure(control.call(
            Value::Bool(false),
            Vec::new(),
            Value::Undefined,
            &mut control_doc,
        ));
        assert!(matches!(original.kind, ErrorKind::Runtime("TypeError")));
        assert_eq!(original.message, "value is not callable");
        let capability = control
            .new_promise_capability(control_constructor, &mut control_doc)
            .unwrap();
        let prefix_heap = control.allocated - initial_heap;
        let prefix_objects = control.objects.len() - initial_objects;
        assert_eq!(
            capability.promise,
            control.environments[0].bindings["holder"].value
        );
        assert_eq!(
            capability.resolve,
            control.environments[0].bindings["resolved"].value
        );
        assert_eq!(
            capability.reject,
            control.environments[0].bindings["rejected"].value
        );
        assert_eq!(
            control.environments[0].bindings["trace"].value,
            Value::String("SC".into())
        );
        let reason_heap = 256 + size_of::<Option<AbortSlot>>() + size_of::<Option<f64>>() + 42;
        let (mut runtime, mut doc, constructor, queued) = terminal_fixture(strict);
        assert_eq!(runtime.objects.len(), initial_objects);
        reset(&mut runtime, MAX_STEPS, 32 + prefix_heap + reason_heap - 1);
        let error = failure(runtime.with_author_entry(&mut doc, |runtime, doc| {
            runtime.promise_native_operation("Promise.try", constructor, &[Value::Bool(false)], doc)
        }));
        assert!(error.is_resource_limit());
        assert_eq!(runtime.allocated, MAX_HEAP + 1);
        assert_eq!(runtime.objects.len(), initial_objects + prefix_objects + 1);
        let reason = &runtime.objects[initial_objects + prefix_objects];
        assert_eq!(reason.intrinsic_error, Some("TypeError"));
        assert_eq!(
            reason.prototype,
            Some(Value::Object(runtime.prototypes["Object"]))
        );
        assert!(reason.values.is_empty());
        assert!(reason.order.is_empty());
        assert_eq!(
            runtime.environments[0].bindings["trace"].value,
            Value::String("SC".into())
        );
        assert!(runtime.microtasks_suspended());
        assert_eq!(runtime.microtasks.author_depth, 0);
        assert!(!runtime.microtasks.performing_checkpoint);
        quiet(&runtime);
        let heap = runtime.allocated;
        for readback in 0..2 {
            if readback == 1 {
                runtime.steps = MAX_STEPS;
            }
            let work = runtime.steps;
            runtime.microtask_checkpoint(&mut doc).unwrap();
            assert_eq!((runtime.steps, runtime.allocated), (work, heap));
            assert_eq!(
                runtime.environments[0].bindings["trace"].value,
                Value::String("SC".into())
            );
            assert!(runtime.microtasks_suspended());
            assert_eq!(runtime.microtasks.queue.len(), 1);
            assert_eq!(
                runtime.microtasks.queue.front(),
                Some(&QueueJob::Callback(queued.clone()))
            );
        }
    }
}
