//! Independent shared-key cuts. Fixture setup is inert Rust, never author work.
use super::*;

fn producer_bytes() -> usize {
    let a = size_of::<Option<AbortSlot>>();
    let d = size_of::<Option<f64>>();
    let k = size_of::<PropertyKey>();
    let n = size_of::<Native>();
    let l = 16 * (k + size_of::<Property>()) + 32 * size_of::<usize>() + 64;
    let b = 16 * (size_of::<String>() + size_of::<Binding>()) + 32 * size_of::<usize>() + 64;
    1004 + a + d + 2 * k + l + n + 4 * b
}
fn consumer_bytes() -> usize {
    let a = size_of::<Option<AbortSlot>>();
    let d = size_of::<Option<f64>>();
    let k = size_of::<PropertyKey>();
    let n = size_of::<Native>();
    let l = 16 * (k + size_of::<Property>()) + 32 * size_of::<usize>() + 64;
    let b = 16 * (size_of::<String>() + size_of::<Binding>()) + 32 * size_of::<usize>() + 64;
    8 * a + 8 * d + 23 * k + 8 * l + 7 * n + 4 * b + 8177
}
fn error<T>(result: Result<T>) -> ScriptError {
    match result {
        Ok(_) => panic!("refused installer succeeded"),
        Err(error) => error,
    }
}
fn empty(runtime: &Runtime) {
    macro_rules! arena {
        ($field:ident) => {
            assert!(runtime.promises.$field.records.is_empty());
            assert_eq!(runtime.promises.$field.records.capacity(), 0);
            assert_eq!(runtime.promises.$field.admitted_capacity, 0);
        };
    }
    arena!(promises);
    arena!(gates);
    arena!(callbacks);
    arena!(captures);
    arena!(jobs);
    arena!(finally_captures);
    assert!(runtime.microtasks.queue.is_empty());
    assert_eq!(runtime.microtasks.queue.capacity(), 0);
    assert_eq!(runtime.microtasks.admitted_capacity, 0);
    assert!(runtime.frames.is_empty());
    assert!(runtime.console.is_empty());
}
fn replay() -> Runtime {
    let mut runtime = Runtime::try_new().unwrap();
    empty(&runtime);
    assert_eq!(runtime.objects.len(), 749);
    assert!(
        runtime.environments[0]
            .bindings
            .remove("queueMicrotask")
            .is_some()
    );
    assert!(runtime.environments[0].bindings.remove("Promise").is_some());
    assert!(runtime.promises.intrinsics.take().is_some());
    // Retain all old bags. Physical preparation and restoration of the two
    // last global order positions are outside the measured installer call.
    runtime.objects.try_reserve_exact(9).unwrap();
    assert!(runtime.objects.capacity() >= 758);
    assert_eq!(runtime.environments[0].bindings.len(), 210);
    runtime.next_global_order = 210;
    runtime.steps = MAX_STEPS;
    runtime.allocated = 0;
    runtime
}
fn consumer_fixture() -> (Runtime, [JsString; 2]) {
    let mut runtime = replay();
    let keys = runtime.install_microtask_intrinsic().unwrap();
    assert_eq!(runtime.steps, MAX_STEPS - 331);
    assert_eq!(runtime.allocated, producer_bytes());
    assert_eq!(runtime.objects.len(), 750);
    // The producer has already paid the keys' construction and future cleanup.
    // Resetting fixture counters isolates the consumer's additional charges.
    runtime.steps = 0;
    runtime.allocated = 0;
    (runtime, keys)
}
fn native_bag(runtime: &Runtime, name: &str) -> usize {
    let Value::Native(native) = &runtime.environments[0].bindings[name].value else {
        panic!("global is not native");
    };
    native.properties.unwrap().get()
}
fn named_key<'a>(runtime: &'a Runtime, owner: usize, name: &str) -> &'a JsString {
    runtime.objects[owner]
        .values
        .keys()
        .filter_map(PropertyKey::as_string)
        .find(|key| key.units().iter().copied().eq(name.bytes().map(u16::from)))
        .unwrap()
}
fn promise_coherent(runtime: &Runtime) {
    let intrinsics = runtime.promises.intrinsics.as_ref().unwrap();
    let Value::Native(private) = &intrinsics.constructor else {
        panic!("private Promise constructor");
    };
    let Value::Native(global) = &runtime.environments[0].bindings["Promise"].value else {
        panic!("global Promise constructor");
    };
    assert!(Rc::ptr_eq(private, global));
    assert!(!runtime.environments[0].bindings["Promise"].enumerable);
    assert!(runtime.environments[0].bindings["queueMicrotask"].enumerable);
    assert_eq!(
        runtime.environments[0].bindings["queueMicrotask"].global_order,
        210
    );
    assert_eq!(
        runtime.environments[0].bindings["Promise"].global_order,
        211
    );
    assert_eq!(runtime.native_properties.len(), 299);
    assert_eq!(runtime.prototypes.len(), 25);
    assert_eq!(runtime.environments[0].bindings.len(), 212);
}

#[test]
fn shared_key_producer_exact331_and_heap_publish_only_after_full_admission() {
    let bytes = producer_bytes();
    for (work, allowance, accepted) in [
        (331, bytes, true),
        (330, bytes, false),
        (331, bytes - 1, false),
    ] {
        let mut runtime = replay();
        let capacity = runtime.objects.capacity();
        runtime.steps = work;
        runtime.allocated = MAX_HEAP - allowance;
        let result = runtime.install_microtask_intrinsic();
        assert_eq!(result.is_ok(), accepted);
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.objects.capacity(), capacity);
        assert!(!runtime.environments[0].bindings.contains_key("Promise"));
        assert!(runtime.promises.intrinsics.is_none());
        empty(&runtime);
        if accepted {
            let keys = result.unwrap();
            assert_eq!(runtime.allocated, MAX_HEAP);
            assert_eq!(runtime.objects.len(), 750);
            assert_eq!(runtime.environments[0].bindings.len(), 211);
            assert_eq!(runtime.next_global_order, 211);
            let owner = native_bag(&runtime, "queueMicrotask");
            assert_eq!(owner, 749);
            for (index, name) in ["length", "name"].into_iter().enumerate() {
                assert!(std::ptr::eq(
                    keys[index].units().as_ptr(),
                    named_key(&runtime, owner, name).units().as_ptr()
                ));
            }
        } else {
            assert!(error(result).is_resource_limit());
            assert!(
                !runtime.environments[0]
                    .bindings
                    .contains_key("queueMicrotask")
            );
            assert_eq!(runtime.environments[0].bindings.len(), 210);
            assert_eq!(runtime.next_global_order, 210);
            if allowance == bytes - 1 {
                assert_eq!(runtime.allocated, MAX_HEAP + 1);
            } else {
                assert!(runtime.allocated <= MAX_HEAP);
            }
        }
    }
}

#[test]
fn shared_key_consumer_exact1116_and_heap_preserve_the_published_producer_on_refusal() {
    let bytes = consumer_bytes();
    for (work, allowance, accepted) in [
        (1116, bytes, true),
        (1115, bytes, false),
        (1116, bytes - 1, false),
    ] {
        let (mut runtime, keys) = consumer_fixture();
        let capacity = runtime.objects.capacity();
        let producer_owner = native_bag(&runtime, "queueMicrotask");
        runtime.steps = work;
        runtime.allocated = MAX_HEAP - allowance;
        let result = runtime.install_promise_intrinsics_with_keys(keys);
        assert_eq!(result.is_ok(), accepted);
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.objects.capacity(), capacity);
        assert_eq!(native_bag(&runtime, "queueMicrotask"), producer_owner);
        empty(&runtime);
        if accepted {
            result.unwrap();
            assert_eq!(runtime.allocated, MAX_HEAP);
            assert_eq!(runtime.objects.len(), 758);
            assert_eq!(runtime.next_global_order, 212);
            promise_coherent(&runtime);
        } else {
            assert!(error(result).is_resource_limit());
            assert!(!runtime.environments[0].bindings.contains_key("Promise"));
            assert!(runtime.promises.intrinsics.is_none());
            assert_eq!(runtime.environments[0].bindings.len(), 211);
            assert_eq!(runtime.next_global_order, 211);
            if allowance == bytes - 1 {
                assert_eq!(runtime.allocated, MAX_HEAP + 1);
            } else {
                assert!(runtime.allocated <= MAX_HEAP);
            }
        }
    }
}

#[test]
fn shared_key_pair_exact1447_and_heap_keep_late_partial_state_unreturned() {
    let bytes = producer_bytes() + consumer_bytes();
    for (work, allowance, accepted) in [
        (1447, bytes, true),
        (1446, bytes, false),
        (1447, bytes - 1, false),
    ] {
        let mut runtime = replay();
        let capacity = runtime.objects.capacity();
        runtime.steps = work;
        runtime.allocated = MAX_HEAP - allowance;
        let result = runtime
            .install_microtask_intrinsic()
            .and_then(|keys| runtime.install_promise_intrinsics_with_keys(keys));
        assert_eq!(result.is_ok(), accepted);
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.objects.capacity(), capacity);
        assert!(
            runtime.environments[0]
                .bindings
                .contains_key("queueMicrotask")
        );
        empty(&runtime);
        if accepted {
            result.unwrap();
            assert_eq!(runtime.allocated, MAX_HEAP);
            promise_coherent(&runtime);
        } else {
            assert!(error(result).is_resource_limit());
            assert!(!runtime.environments[0].bindings.contains_key("Promise"));
            assert!(runtime.promises.intrinsics.is_none());
            assert_eq!(runtime.environments[0].bindings.len(), 211);
            if allowance == bytes - 1 {
                assert_eq!(runtime.allocated, MAX_HEAP + 1);
            }
        }
    }
    // Actual whole-bootstrap Err/success and public fresh-budget behavior are
    // the independently migrated99866/99867/100000 witnesses, not this replay.
}

#[test]
fn shared_key_producer_prepays_retained_cleanup_before_either_key_can_exist() {
    let bag = 72
        + size_of::<Option<AbortSlot>>()
        + size_of::<Option<f64>>()
        + 2 * size_of::<PropertyKey>();
    let leaf =
        16 * (size_of::<PropertyKey>() + size_of::<Property>()) + 32 * size_of::<usize>() + 64;
    for (work, added, bytes, properties) in [
        (1, 0, 0, 0),
        (2, 0, 0, 0),
        (10, 0, bag - 2 * size_of::<PropertyKey>(), 0),
        (11, 1, bag, 0),
        (47, 1, bag + 120, 0),
        (67, 1, bag + 120 + 88, 0),
        (93, 1, bag + 120 + 88 + 280 + leaf + 80, 1),
    ] {
        let mut runtime = replay();
        runtime.steps = work;
        let failure = error(runtime.install_microtask_intrinsic());
        assert!(failure.is_resource_limit());
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, bytes);
        assert_eq!(runtime.objects.len(), 749 + added);
        if added != 0 {
            assert_eq!(runtime.objects[749].values.len(), properties);
        }
        assert!(
            !runtime.environments[0]
                .bindings
                .contains_key("queueMicrotask")
        );
        assert!(!runtime.environments[0].bindings.contains_key("Promise"));
        assert!(runtime.promises.intrinsics.is_none());
        empty(&runtime);
    }
}

fn inject_invalid_install_state(runtime: &mut Runtime, which: usize) {
    // Manual Rust-only invalid-state fixtures. No invalid record is executed.
    match which {
        0 => runtime.promises.promises.records.push(PromiseRecord {
            object_id: 0,
            settlement: Settlement::Pending,
            reactions: Arena::default(),
            handled: false,
        }),
        1 => runtime
            .promises
            .gates
            .records
            .push(ResolveGate { target: None }),
        2 => runtime.promises.callbacks.records.push(CallbackRecord {
            object_id: 0,
            kind: CallbackKind::Resolve(GateId(0)),
        }),
        3 => runtime.promises.captures.records.push(CapabilityCapture {
            resolve: Value::Undefined,
            reject: Value::Undefined,
        }),
        4 => runtime.promises.jobs.records.push(None),
        5 => {
            runtime.promises.intrinsics = Some(Intrinsics {
                constructor: Value::Undefined,
                prototype: 0,
            })
        }
        6 => runtime
            .promises
            .finally_captures
            .records
            .push(FinallyCapture::Outcome(Value::Undefined)),
        _ => unreachable!(),
    }
}
#[test]
fn shared_and_standalone_guards_refuse_all_seven_invalid_states_before_allocation() {
    for shared in [false, true] {
        let fixed = if shared { 17 } else { 19 };
        for which in 0..7 {
            for work in [fixed - 1, fixed] {
                let (mut runtime, keys) = consumer_fixture();
                inject_invalid_install_state(&mut runtime, which);
                let objects = runtime.objects.len();
                runtime.steps = work;
                let result = if shared {
                    runtime.install_promise_intrinsics_with_keys(keys)
                } else {
                    drop(keys); // Producer already prepaid this fixture pair's cleanup.
                    runtime.install_promise_intrinsics()
                };
                let failure = error(result);
                assert!(failure.is_resource_limit());
                if work == fixed {
                    assert_eq!(failure.message, "Promise state already initialized");
                }
                assert_eq!(runtime.steps, 0);
                assert_eq!(runtime.allocated, 0);
                assert_eq!(runtime.objects.len(), objects);
                assert_eq!(runtime.environments[0].bindings.len(), 211);
                assert!(
                    runtime.environments[0]
                        .bindings
                        .contains_key("queueMicrotask")
                );
                assert!(!runtime.environments[0].bindings.contains_key("Promise"));
                assert!(runtime.frames.is_empty());
                assert!(runtime.microtasks.queue.is_empty());
                assert!(runtime.console.is_empty());
            }
        }
    }
}

#[test]
fn standalone_guard_then_key_order_and_shared_move_have_distinct_exact_prefixes() {
    for (shared, work, bytes) in [
        (false, 19, 0),
        (false, 38, 0),
        (false, 39, 88),
        (false, 54, 88),
        (false, 55, 168),
        (true, 17, 0),
        (true, 38, 0),
        (true, 39, 92),
    ] {
        let (mut runtime, keys) = consumer_fixture();
        runtime.steps = work;
        let result = if shared {
            runtime.install_promise_intrinsics_with_keys(keys)
        } else {
            drop(keys);
            runtime.install_promise_intrinsics()
        };
        assert!(error(result).is_resource_limit());
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, bytes);
        assert_eq!(runtime.objects.len(), 750);
        assert!(runtime.promises.intrinsics.is_none());
        assert!(!runtime.environments[0].bindings.contains_key("Promise"));
        empty(&runtime);
    }
}
