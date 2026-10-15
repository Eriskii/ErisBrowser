//! Selected withResolvers fees and refusal order, frozen before implementation.
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
fn node_bytes() -> usize {
    16 * (size_of::<PropertyKey>() + size_of::<Property>()) + 32 * size_of::<usize>() + 64
}
fn bag_bytes() -> usize {
    72 + size_of::<Option<AbortSlot>>() + size_of::<Option<f64>>()
}
fn result_heap() -> usize {
    size_of::<ScriptObject>()
        + size_of::<Option<AbortSlot>>()
        + size_of::<Option<f64>>()
        + 3 * size_of::<PropertyKey>()
        + node_bytes()
        + 1192
}
fn installer_heap(grow: bool) -> usize {
    size_of::<Option<AbortSlot>>()
        + size_of::<Option<f64>>()
        + (if grow { 13 } else { 2 }) * size_of::<PropertyKey>()
        + node_bytes()
        + size_of::<Native>()
        + 1269
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

// Existing bootstrap reaches the Any entry with no remaining work. Pay that
// unchanged installer separately, selecting its physical order branch.
fn before_with_resolvers() -> (Runtime, usize) {
    let mut runtime = Runtime::uninitialized();
    runtime.steps = 100_797;
    assert!(failure(runtime.initialize_bootstrap()).is_resource_limit());
    assert_eq!(runtime.steps, 0);
    assert_eq!(runtime.objects.len(), 754);
    assert!(runtime.objects.capacity() >= 756);
    assert_eq!(runtime.prototypes["AggregateError"], 752);
    assert_eq!(runtime.native_properties["AggregateError"], 753);
    let id = owner(&runtime);
    let bag = &runtime.objects[id];
    assert_eq!((bag.values.len(), bag.order.len()), (9, 9));
    let any_work = if bag.order.len() == bag.order.capacity() {
        189
    } else {
        171
    };
    runtime.steps = any_work;
    runtime.install_promise_any_intrinsic().unwrap();
    assert_eq!(runtime.steps, 0);
    assert_eq!(runtime.objects.len(), 755);
    let bag = &runtime.objects[id];
    assert_eq!((bag.values.len(), bag.order.len()), (10, 10));
    assert!(!bag.values.contains_key(&PropertyKey::from("withResolvers")));
    (runtime, any_work)
}
fn installer_fixture(grow: bool) -> Runtime {
    let (mut runtime, _) = before_with_resolvers();
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
fn installed(runtime: &Runtime, success: bool) {
    assert_eq!(runtime.objects.len(), 756);
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
        if success { (11, 11) } else { (10, 10) }
    );
    assert_eq!(
        bag.values.contains_key(&PropertyKey::from("withResolvers")),
        success
    );
    let PropertyValue::Data {
        value: Value::Native(any),
        ..
    } = &bag.values[&PropertyKey::from("any")].value
    else {
        panic!("old Any method")
    };
    assert_eq!(any.properties.unwrap().get(), 754);
    if success {
        assert_eq!(bag.order.last(), Some(&PropertyKey::from("withResolvers")));
        let property = &bag.values[&PropertyKey::from("withResolvers")];
        let PropertyValue::Data {
            value: Value::Native(native),
            writable,
        } = &property.value
        else {
            panic!("new method")
        };
        assert!(*writable && !property.enumerable && property.configurable);
        assert_eq!(native.name, "Promise.withResolvers");
        assert_eq!(native.properties.unwrap().get(), 755);
        assert_eq!(native.receiver, Value::Undefined);
        let metadata = &runtime.objects[755];
        assert_eq!(
            metadata.prototype,
            Some(Value::Function(runtime.function_prototype))
        );
        assert_eq!(
            metadata.order,
            [PropertyKey::from("length"), PropertyKey::from("name")]
        );
        for (key, expected) in [
            ("length", Value::Number(0.0)),
            ("name", Value::String("withResolvers".into())),
        ] {
            let property = &metadata.values[&PropertyKey::from(key)];
            let PropertyValue::Data { value, writable } = &property.value else {
                panic!("metadata data")
            };
            assert_eq!(value, &expected);
            assert!(!*writable && !property.enumerable && property.configurable);
        }
    }
    quiet(runtime);
}

#[test]
fn with_resolvers_installer12_guards_precede_all_allocations() {
    for bad in 0..7 {
        for work in [11, 12] {
            let (mut runtime, _) = before_with_resolvers();
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
            assert!(
                failure(runtime.install_promise_with_resolvers_intrinsic()).is_resource_limit()
            );
            assert_eq!(
                (runtime.steps, runtime.allocated, runtime.objects.len()),
                (0, MAX_HEAP, 755)
            );
            assert!(
                !runtime.objects[id]
                    .values
                    .contains_key(&PropertyKey::from("withResolvers"))
            );
            quiet(&runtime);
        }
    }
    let mut runtime = installer_fixture(false);
    runtime.steps = 306;
    runtime.install_promise_with_resolvers_intrinsic().unwrap();
    let allocated = runtime.allocated;
    runtime.steps = 12;
    assert!(failure(runtime.install_promise_with_resolvers_intrinsic()).is_resource_limit());
    assert_eq!((runtime.steps, runtime.allocated), (0, allocated));
    installed(&runtime, true);
}

#[test]
fn with_resolvers_installer306_or326_exact_work_and_heap_publish_last() {
    for grow in [false, true] {
        let total = if grow { 326 } else { 306 };
        let heap = installer_heap(grow);
        for (work, bytes, accepted) in [
            (total - 1, heap, false),
            (total, heap - 1, false),
            (total, heap, true),
        ] {
            let mut runtime = installer_fixture(grow);
            let capacity = runtime.objects.capacity();
            reset(&mut runtime, work, bytes);
            let result = runtime.install_promise_with_resolvers_intrinsic();
            if accepted {
                result.unwrap();
            } else {
                assert!(failure(result).is_resource_limit());
            }
            assert_eq!(runtime.steps, 0);
            assert_eq!(runtime.objects.capacity(), capacity);
            let final_charge = 308
                + if grow {
                    11 * size_of::<PropertyKey>()
                } else {
                    0
                };
            assert_eq!(
                runtime.allocated,
                if work < total {
                    MAX_HEAP - final_charge
                } else if accepted {
                    MAX_HEAP
                } else {
                    MAX_HEAP + 1
                }
            );
            installed(&runtime, accepted);
        }
    }
}

#[test]
fn with_resolvers_startup_records_physical_growth_and_public_budget_reset() {
    let (mut runtime, any_work) = before_with_resolvers();
    let id = owner(&runtime);
    let before_order = (
        runtime.objects[id].order.len(),
        runtime.objects[id].order.capacity(),
    );
    let before_objects = (runtime.objects.len(), runtime.objects.capacity());
    let before_heap = runtime.allocated;
    let grow = before_order.0 == before_order.1;
    let work = if grow { 326 } else { 306 };
    runtime.steps = work;
    runtime.install_promise_with_resolvers_intrinsic().unwrap();
    assert_eq!(runtime.steps, 0);
    assert_eq!(runtime.allocated - before_heap, installer_heap(grow));
    installed(&runtime, true);
    let after_order = (
        runtime.objects[id].order.len(),
        runtime.objects[id].order.capacity(),
    );
    // Complete the later Try installer only after the old isolated measurements.
    // The initial bootstrap preparation already paid its extra ScriptObject slot.
    let try_before_order = (
        runtime.objects[id].order.len(),
        runtime.objects[id].order.capacity(),
    );
    assert_eq!(try_before_order.0, 11);
    let try_grow = try_before_order.0 == try_before_order.1;
    let try_work = if try_grow { 268 } else { 246 };
    let try_heap = size_of::<Option<AbortSlot>>()
        + size_of::<Option<f64>>()
        + (if try_grow { 14 } else { 2 }) * size_of::<PropertyKey>()
        + 4 * node_bytes()
        + size_of::<Native>()
        + 1179;
    let try_before_heap = runtime.allocated;
    runtime.steps = try_work;
    runtime.install_promise_try_intrinsic().unwrap();
    assert_eq!(runtime.steps, 0);
    assert_eq!(runtime.allocated - try_before_heap, try_heap);
    assert_eq!(runtime.objects[id].order.len(), 12);
    let raw = Runtime::uninitialized().finish_bootstrap().unwrap();
    assert_eq!(
        raw.steps,
        BOOTSTRAP_STEPS - 100_797 - any_work - work - try_work
    );
    assert_eq!(raw.allocated, runtime.allocated);
    assert_eq!(raw.objects.len(), 757);
    assert!(raw.objects.capacity() >= 757);
    eprintln!(
        "withResolvers startup: before_order={before_order:?} after_order={after_order:?} before_objects={before_objects:?} raw_objects={:?} work={} remaining={} allocated={} maps={:?}",
        (raw.objects.len(), raw.objects.capacity()),
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

fn refs(value: &Value) -> usize {
    let Value::Native(native) = value else {
        panic!("Native identity marker")
    };
    Rc::strong_count(native)
}
fn result_fixture() -> (Runtime, Capability, [Value; 3]) {
    let mut runtime = Runtime::try_new().unwrap();
    let retained: [Value; 3] =
        std::array::from_fn(|_| Runtime::native("Object.is", Value::Undefined));
    // Real outer3 incoming cleanup plus three fixture-only retained copies.
    // Both are paid before the measured factory reset, including entry refusal.
    runtime.work(6).unwrap();
    let capability = Capability {
        promise: retained[0].clone(),
        resolve: retained[1].clone(),
        reject: retained[2].clone(),
    };
    assert!(retained.iter().all(|value| refs(value) == 2));
    runtime.objects.try_reserve_exact(1).unwrap();
    assert!(runtime.objects.capacity() > runtime.objects.len());
    (runtime, capability, retained)
}
fn partial_result(runtime: &Runtime, objects: usize, published: usize, retained: &[Value; 3]) {
    let keys = ["promise", "resolve", "reject"];
    let bag = &runtime.objects[objects];
    assert_eq!(
        bag.prototype,
        Some(Value::Object(runtime.prototypes["Object"]))
    );
    assert!(bag.intrinsic_error.is_none());
    assert_eq!(
        bag.order,
        keys[..published]
            .iter()
            .map(|key| PropertyKey::from(*key))
            .collect::<Vec<_>>()
    );
    assert_eq!(bag.values.len(), published);
    for (index, key) in keys.iter().enumerate() {
        assert_eq!(
            refs(&retained[index]),
            if index < published { 2 } else { 1 }
        );
        if index < published {
            let property = &bag.values[&PropertyKey::from(*key)];
            let PropertyValue::Data { value, writable } = &property.value else {
                panic!("own data")
            };
            assert!(*writable && property.enumerable && property.configurable);
            let (Value::Native(actual), Value::Native(expected)) = (value, &retained[index]) else {
                panic!("identity")
            };
            assert!(Rc::ptr_eq(actual, expected));
        }
    }
}

#[test]
fn with_resolvers_result156_work_stages_keep_order_identity_and_partial_state() {
    for work in [
        8, 9, 30, 31, 38, 39, 40, 61, 62, 83, 84, 103, 104, 112, 113, 130, 131, 155, 156,
    ] {
        let (mut runtime, capability, retained) = result_fixture();
        let objects = runtime.objects.len();
        reset(&mut runtime, work, result_heap());
        let result = runtime.new_promise_capability_result(capability);
        assert_eq!(runtime.steps, 0);
        let spent = usize::from(work >= 31) * size_of::<ScriptObject>()
            + usize::from(work >= 39) * bag_bytes()
            + usize::from(work >= 40) * 3 * size_of::<PropertyKey>()
            + usize::from(work >= 62) * 92
            + usize::from(work >= 84) * 92
            + usize::from(work >= 104) * 88
            + usize::from(work >= 113) * (284 + node_bytes())
            + usize::from(work >= 131) * 284
            + usize::from(work >= 156) * 280;
        assert_eq!(runtime.allocated, MAX_HEAP - result_heap() + spent);
        assert_eq!(runtime.objects.len(), objects + usize::from(work >= 40));
        if work >= 40 {
            partial_result(
                &runtime,
                objects,
                usize::from(work >= 113) + usize::from(work >= 131) + usize::from(work >= 156),
                &retained,
            );
        } else {
            assert!(retained.iter().all(|value| refs(value) == 1));
        }
        if work == 156 {
            assert_eq!(result.unwrap(), Value::Object(objects));
            assert!(!runtime.microtasks_suspended());
        } else {
            assert!(failure(result).is_resource_limit());
            assert!(runtime.microtasks_suspended());
        }
        quiet(&runtime);
    }
}

#[test]
fn with_resolvers_result_heap_stages_refuse_before_each_commit() {
    let stages = [
        (31, size_of::<ScriptObject>()),
        (39, bag_bytes()),
        (40, 3 * size_of::<PropertyKey>()),
        (62, 92),
        (84, 92),
        (104, 88),
        (113, 284 + node_bytes()),
        (131, 284),
        (156, 280),
    ];
    let mut prefix = 0;
    for (index, (work, bytes)) in stages.into_iter().enumerate() {
        prefix += bytes;
        let (mut runtime, capability, retained) = result_fixture();
        let objects = runtime.objects.len();
        reset(&mut runtime, 156, prefix - 1);
        assert!(failure(runtime.new_promise_capability_result(capability)).is_resource_limit());
        assert_eq!(
            (runtime.steps, runtime.allocated),
            (156 - work, MAX_HEAP + 1)
        );
        assert_eq!(runtime.objects.len(), objects + usize::from(index > 2));
        if index > 2 {
            partial_result(&runtime, objects, index.saturating_sub(6), &retained);
        } else {
            assert!(retained.iter().all(|value| refs(value) == 1));
        }
        assert!(runtime.microtasks_suspended());
        quiet(&runtime);
    }
    assert_eq!(prefix, result_heap());
    let (mut runtime, capability, retained) = result_fixture();
    let objects = runtime.objects.len();
    reset(&mut runtime, 156, prefix);
    assert_eq!(
        runtime.new_promise_capability_result(capability).unwrap(),
        Value::Object(objects)
    );
    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    partial_result(&runtime, objects, 3, &retained);
}

#[test]
fn with_resolvers_result_uses_saved_object_and_checks_private_lookup_before_slot() {
    for malformed in [false, true] {
        let (mut runtime, capability, retained) = result_fixture();
        let objects = runtime.objects.len();
        if malformed {
            runtime.prototypes.insert("Object", objects);
        } else {
            runtime.prototypes.remove("Object").unwrap();
        }
        reset(&mut runtime, 31, 0);
        assert!(failure(runtime.new_promise_capability_result(capability)).is_resource_limit());
        assert_eq!(
            (runtime.steps, runtime.allocated, runtime.objects.len()),
            (0, MAX_HEAP, objects)
        );
        assert!(retained.iter().all(|value| refs(value) == 1));
        assert!(runtime.microtasks_suspended());
    }
    let (mut runtime, capability, retained) = result_fixture();
    let objects = runtime.objects.len();
    runtime.environments[0]
        .bindings
        .get_mut("Object")
        .unwrap()
        .value = Value::Undefined;
    reset(&mut runtime, 156, result_heap());
    assert_eq!(
        runtime.new_promise_capability_result(capability).unwrap(),
        Value::Object(objects)
    );
    partial_result(&runtime, objects, 3, &retained);
    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
}

#[test]
fn with_resolvers_result_cold_object_plan_uses_actual_capacity_and_retains_charges() {
    for mode in 0..5 {
        let (mut runtime, capability, retained) = result_fixture();
        exact(&mut runtime.objects);
        let objects = runtime.objects.len();
        let limit = MAX_HEAP / size_of::<ScriptObject>();
        let target = (2 * objects)
            .max(8)
            .max((objects + 1).next_power_of_two())
            .min(limit);
        assert!(target > objects);
        let growth_work = 8 + 2 * objects * size_of::<ScriptObject>().div_ceil(size_of::<Value>());
        let growth_bytes = 32 + target * size_of::<ScriptObject>();
        let total = 156 + growth_work;
        let heap = result_heap() + growth_bytes;
        let (work, allowance) = match mode {
            0 => (31 + growth_work - 1, heap),
            1 => (total, size_of::<ScriptObject>() + growth_bytes - 1),
            2 => (total - 1, heap),
            3 => (total, heap - 1),
            _ => (total, heap),
        };
        reset(&mut runtime, work, allowance);
        let result = runtime.new_promise_capability_result(capability);
        if mode < 2 {
            assert!(failure(result).is_resource_limit());
            assert_eq!(
                (runtime.objects.len(), runtime.objects.capacity()),
                (objects, objects)
            );
            assert_eq!(runtime.steps, if mode == 0 { 0 } else { 125 });
            assert_eq!(
                runtime.allocated,
                if mode == 0 {
                    MAX_HEAP - heap
                } else {
                    MAX_HEAP + 1
                }
            );
            assert!(retained.iter().all(|value| refs(value) == 1));
            assert!(runtime.microtasks_suspended());
            quiet(&runtime);
            continue;
        }
        assert_eq!(runtime.objects.len(), objects + 1);
        assert!(runtime.objects.capacity() >= target);
        assert_eq!(runtime.steps, 0);
        assert_eq!(
            runtime.allocated,
            match mode {
                2 => MAX_HEAP - 280,
                3 => MAX_HEAP + 1,
                _ => MAX_HEAP,
            }
        );
        if mode == 4 {
            assert_eq!(result.unwrap(), Value::Object(objects));
        } else {
            assert!(failure(result).is_resource_limit());
            assert!(runtime.microtasks_suspended());
        }
        partial_result(&runtime, objects, if mode == 4 { 3 } else { 2 }, &retained);
        quiet(&runtime);
    }
}

#[test]
fn with_resolvers_public8_outer3_and_constructor2_cuts_precede_result_allocation() {
    for work in [7, 8, 10, 11, 12, 13] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        let objects = runtime.objects.len();
        reset(&mut runtime, work, 0);
        let error = failure(runtime.promise_native_operation(
            "Promise.withResolvers",
            Value::Bool(false),
            &[],
            &mut document,
        ));
        if work == 13 {
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
        assert!(runtime.microtasks.queue.is_empty());
        quiet(&runtime);
    }
}

fn terminal_fixture(strict: bool) -> (Runtime, Document, Value, Value) {
    const SOURCE: &str = r#"
var trace='S',holder={},resolved=()=>{},rejected=()=>{};
var queued=()=>{trace+='Q';};
function C(exec){trace+='C';exec(resolved,rejected);return holder;}
C
"#;
    let mut runtime = Runtime::try_new().unwrap();
    let mut document = Document::parse("");
    let constructor = if strict {
        runtime.execute_strict(SOURCE, &mut document)
    } else {
        runtime.execute(SOURCE, &mut document)
    }
    .unwrap();
    let queued = runtime.environments[0].bindings["queued"].value.clone();
    runtime
        .queue_microtask(&Value::Undefined, std::slice::from_ref(&queued))
        .unwrap();
    runtime.objects.try_reserve_exact(16).unwrap();
    assert!(runtime.objects.capacity() >= runtime.objects.len() + 16);
    (runtime, document, constructor, queued)
}

#[test]
fn with_resolvers_natural_result_heap_refusal_preserves_constructor_effects_and_sticky_queue() {
    for strict in [false, true] {
        // Independently compose the unchanged NPC prefix; do not measure the
        // new method to set its own expected budget or reset inside its call.
        let (mut control, mut control_doc, control_constructor, _) = terminal_fixture(strict);
        let initial_objects = control.objects.len();
        let initial_heap = control.allocated;
        let capability = control
            .new_promise_capability(control_constructor, &mut control_doc)
            .unwrap();
        let npc_heap = control.allocated - initial_heap;
        let npc_objects = control.objects.len() - initial_objects;
        assert!(control.objects.capacity() > control.objects.len());
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
        let (mut runtime, mut document, constructor, queued) = terminal_fixture(strict);
        assert_eq!(runtime.objects.len(), initial_objects);
        reset(&mut runtime, MAX_STEPS, npc_heap + result_heap() - 1);
        let error = runtime
            .with_author_entry(&mut document, |runtime, document| {
                runtime.promise_native_operation(
                    "Promise.withResolvers",
                    constructor,
                    &[],
                    document,
                )
            })
            .unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(runtime.allocated, MAX_HEAP + 1);
        assert_eq!(runtime.objects.len(), initial_objects + npc_objects + 1);
        let bag = &runtime.objects[initial_objects + npc_objects];
        assert_eq!(
            bag.order,
            [PropertyKey::from("promise"), PropertyKey::from("resolve")]
        );
        for (key, name) in [("promise", "holder"), ("resolve", "resolved")] {
            let PropertyValue::Data { value, writable } =
                &bag.values[&PropertyKey::from(key)].value
            else {
                panic!("partial result")
            };
            assert_eq!(value, &runtime.environments[0].bindings[name].value);
            assert!(*writable);
        }
        assert!(!bag.values.contains_key(&PropertyKey::from("reject")));
        assert!(runtime.microtasks_suspended());
        assert_eq!(runtime.microtasks.author_depth, 0);
        assert!(!runtime.microtasks.performing_checkpoint);
        quiet(&runtime);
        let heap = runtime.allocated;
        for readback in 0..2 {
            // An explicit post-failure work reset cannot revive queued work.
            // Heap and suspension are never reset; no reset occurs in the call.
            if readback == 1 {
                runtime.steps = MAX_STEPS;
            }
            let work = runtime.steps;
            runtime.microtask_checkpoint(&mut document).unwrap();
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
