//! Additive source-derived witnesses; the original eight groups stay unchanged.
use super::*;
use std::{mem::size_of, rc::Rc};

fn leaf_bytes() -> usize {
    16 * (size_of::<PropertyKey>() + size_of::<Property>()) + 32 * size_of::<usize>() + 64
}
fn metadata_bytes() -> usize {
    904 + size_of::<Option<AbortSlot>>()
        + size_of::<Option<f64>>()
        + 2 * size_of::<PropertyKey>()
        + leaf_bytes()
        + size_of::<Native>()
}
fn reset(runtime: &mut Runtime, work: usize, heap: usize) {
    runtime.steps = work;
    runtime.allocated = MAX_HEAP - heap;
}
fn handler() -> Value {
    Runtime::native("Object.is", Value::Undefined)
}
fn rejector() -> Value {
    Runtime::native("Number.isNaN", Value::Undefined)
}
fn object(runtime: &mut Runtime) -> Value {
    runtime.object_ordered([]).unwrap()
}
fn pending() -> (Runtime, PromiseId) {
    let mut runtime = Runtime::new();
    assert!(runtime.promises.promises.records.is_empty());
    assert!(runtime.promises.gates.records.is_empty());
    assert!(runtime.promises.callbacks.records.is_empty());
    assert!(runtime.promises.captures.records.is_empty());
    let Value::Object(object_id) = object(&mut runtime) else {
        panic!("object");
    };
    runtime.promises.promises.records.push(PromiseRecord {
        object_id,
        settlement: Settlement::Pending,
        handled: false,
        reactions: Arena {
            records: Vec::new(),
            admitted_capacity: 0,
        },
    });
    runtime.promises.promises.admitted_capacity = 1;
    // Isolate the promised factory envelope from unrelated ordinary-arena
    // capacity growth. This is Rust fixture setup, outside each budget cut.
    runtime.objects.try_reserve_exact(4).unwrap();
    (runtime, PromiseId(0))
}
fn callable_metadata(runtime: &Runtime, value: &Value, arity: usize) -> usize {
    let Value::Native(native) = value else {
        panic!("native callback");
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
        ("length", Value::Number(arity as f64)),
        ("name", Value::String("".into())),
    ] {
        let property = bag.values.get(&PropertyKey::from(key)).unwrap();
        assert!(!property.enumerable && property.configurable);
        let PropertyValue::Data { value, writable } = &property.value else {
            panic!("data property");
        };
        assert_eq!(value, &expected);
        assert!(!writable);
    }
    id
}
fn late_refusal(runtime: &Runtime, work: usize, total: usize, heap: usize, required: usize) {
    assert_eq!(runtime.steps, 0);
    assert!(runtime.microtasks_suspended());
    assert_eq!(
        runtime.allocated,
        if work < total {
            MAX_HEAP - (size_of::<Native>() + 48)
        } else {
            assert_eq!(heap + 1, required);
            MAX_HEAP + 1
        }
    );
    assert!(runtime.microtasks.queue.is_empty());
    assert!(runtime.promises.jobs.records.is_empty());
    assert!(runtime.console.is_empty());
}

#[test]
fn promise_runtime_object_slot_plan_charges_each_new_slot_and_growth() {
    let zero = object_slot_plan(8, 8, 0).unwrap();
    assert_eq!((zero.target, zero.work, zero.bytes), (None, 0, 0));
    let spare = object_slot_plan(7, 8, 1).unwrap();
    assert_eq!(
        (spare.target, spare.work, spare.bytes),
        (None, 0, size_of::<ScriptObject>())
    );
    let movement = 1 + (size_of::<ScriptObject>() - 1) / size_of::<Value>();
    let grow = object_slot_plan(8, 8, 2).unwrap();
    assert_eq!(
        (grow.target, grow.work, grow.bytes),
        (
            Some(16),
            8 + 16 * movement,
            32 + 18 * size_of::<ScriptObject>()
        )
    );
    assert!(object_slot_plan(9, 8, 1).is_err());
    assert!(object_slot_plan(usize::MAX, usize::MAX, 1).is_err());
    assert!(object_slot_plan(0, 0, MAX_HEAP / size_of::<ScriptObject>() + 1).is_err());
}

#[test]
fn promise_generated_callback_116_or124_has_exact_late_cuts() {
    for cold in [false, true] {
        let total = if cold { 124 } else { 116 };
        let required = metadata_bytes()
            + size_of::<ScriptObject>()
            + if cold {
                32 + 8 * size_of::<CallbackRecord>()
            } else {
                0
            };
        for (work, heap, success) in [
            (total, required, true),
            (total - 1, required, false),
            (total, required - 1, false),
        ] {
            let (mut runtime, promise) = pending();
            runtime.promises.gates.records.push(ResolveGate {
                target: Some(promise),
            });
            runtime.promises.gates.admitted_capacity = 1;
            if !cold {
                runtime
                    .promises
                    .callbacks
                    .records
                    .try_reserve_exact(8)
                    .unwrap();
                runtime.promises.callbacks.admitted_capacity = 8;
            }
            let old_objects = runtime.objects.len();
            let old_registry = runtime.native_properties.clone();
            let old_globals = runtime.environments[0].bindings.len();
            let old_order = runtime.next_global_order;
            reset(&mut runtime, work, heap);
            let result = runtime.new_promise_callback(CallbackKind::Resolve(GateId(0)));
            assert_eq!(result.is_ok(), success);
            assert_eq!(runtime.objects.len(), old_objects + 1);
            assert_eq!(runtime.native_properties, old_registry);
            assert_eq!(runtime.environments[0].bindings.len(), old_globals);
            assert_eq!(runtime.next_global_order, old_order);
            assert_eq!(
                runtime.promises.gates.records[0].target.unwrap().0,
                promise.0
            );
            assert!(runtime.promises.captures.records.is_empty());
            if success {
                let value = result.unwrap();
                assert_eq!(callable_metadata(&runtime, &value, 1), old_objects);
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
                assert_eq!(runtime.promises.callbacks.records.len(), 1);
                assert_eq!(runtime.promises.callbacks.records[0].object_id, old_objects);
                assert!(matches!(
                    runtime.promises.callbacks.records[0].kind,
                    CallbackKind::Resolve(GateId(0))
                ));
                assert_eq!(runtime.promises.callbacks.admitted_capacity, 8);
                assert!(!runtime.microtasks_suspended());
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                late_refusal(&runtime, work, total, heap, required);
                assert!(runtime.promises.callbacks.records.is_empty());
                assert_eq!(
                    runtime.promises.callbacks.admitted_capacity,
                    if cold { 0 } else { 8 }
                );
            }
        }
    }
}

#[test]
fn promise_resolving_pair_241_or257_commits_one_gate_and_distinct_callbacks() {
    for cold in [false, true] {
        let total = if cold { 257 } else { 241 };
        let required = 2 * (metadata_bytes() + size_of::<ScriptObject>())
            + if cold {
                64 + 8 * size_of::<ResolveGate>() + 8 * size_of::<CallbackRecord>()
            } else {
                0
            };
        for (work, heap, success) in [
            (total, required, true),
            (total - 1, required, false),
            (total, required - 1, false),
        ] {
            let (mut runtime, promise) = pending();
            if !cold {
                runtime.promises.gates.records.try_reserve_exact(8).unwrap();
                runtime.promises.gates.admitted_capacity = 8;
                runtime
                    .promises
                    .callbacks
                    .records
                    .try_reserve_exact(8)
                    .unwrap();
                runtime.promises.callbacks.admitted_capacity = 8;
            }
            let old_objects = runtime.objects.len();
            let old_registry = runtime.native_properties.clone();
            reset(&mut runtime, work, heap);
            let result = runtime.new_resolving_functions(promise);
            assert_eq!(result.is_ok(), success);
            assert_eq!(runtime.objects.len(), old_objects + 2);
            assert_eq!(runtime.native_properties, old_registry);
            assert!(runtime.promises.captures.records.is_empty());
            if success {
                let (resolve, reject) = result.unwrap();
                assert_ne!(resolve, reject);
                assert_eq!(callable_metadata(&runtime, &resolve, 1), old_objects);
                assert_eq!(callable_metadata(&runtime, &reject, 1), old_objects + 1);
                assert_eq!(runtime.promises.gates.records.len(), 1);
                assert_eq!(
                    runtime.promises.gates.records[0].target.unwrap().0,
                    promise.0
                );
                assert_eq!(runtime.promises.callbacks.records.len(), 2);
                assert!(matches!(
                    runtime.promises.callbacks.records[0].kind,
                    CallbackKind::Resolve(GateId(0))
                ));
                assert!(matches!(
                    runtime.promises.callbacks.records[1].kind,
                    CallbackKind::Reject(GateId(0))
                ));
                assert_eq!(
                    (
                        runtime.promises.gates.admitted_capacity,
                        runtime.promises.callbacks.admitted_capacity
                    ),
                    (8, 8)
                );
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                late_refusal(&runtime, work, total, heap, required);
                assert!(runtime.promises.gates.records.is_empty());
                assert!(runtime.promises.callbacks.records.is_empty());
                let old = if cold { 0 } else { 8 };
                assert_eq!(
                    (
                        runtime.promises.gates.admitted_capacity,
                        runtime.promises.callbacks.admitted_capacity
                    ),
                    (old, old)
                );
            }
        }
    }
}

#[test]
fn promise_capability_executor_126_or142_keeps_capture_unpublished_on_refusal() {
    for cold in [false, true] {
        let total = if cold { 142 } else { 126 };
        let required = metadata_bytes()
            + size_of::<ScriptObject>()
            + if cold {
                64 + 8 * size_of::<CapabilityCapture>() + 8 * size_of::<CallbackRecord>()
            } else {
                0
            };
        for (work, heap, success) in [
            (total, required, true),
            (total - 1, required, false),
            (total, required - 1, false),
        ] {
            let (mut runtime, _) = pending();
            if !cold {
                runtime
                    .promises
                    .captures
                    .records
                    .try_reserve_exact(8)
                    .unwrap();
                runtime.promises.captures.admitted_capacity = 8;
                runtime
                    .promises
                    .callbacks
                    .records
                    .try_reserve_exact(8)
                    .unwrap();
                runtime.promises.callbacks.admitted_capacity = 8;
            }
            let old_objects = runtime.objects.len();
            reset(&mut runtime, work, heap);
            let result = runtime.new_capability_executor();
            assert_eq!(result.is_ok(), success);
            assert_eq!(runtime.objects.len(), old_objects + 1);
            assert!(runtime.promises.gates.records.is_empty());
            if success {
                let (id, callback) = result.unwrap();
                assert_eq!(id.0, 0);
                assert_eq!(callable_metadata(&runtime, &callback, 2), old_objects);
                assert_eq!(runtime.promises.captures.records.len(), 1);
                assert_eq!(
                    runtime.promises.captures.records[0].resolve,
                    Value::Undefined
                );
                assert_eq!(
                    runtime.promises.captures.records[0].reject,
                    Value::Undefined
                );
                assert_eq!(runtime.promises.callbacks.records.len(), 1);
                assert!(matches!(
                    runtime.promises.callbacks.records[0].kind,
                    CallbackKind::CapabilityExecutor(CaptureId(0))
                ));
                assert_eq!(
                    (
                        runtime.promises.captures.admitted_capacity,
                        runtime.promises.callbacks.admitted_capacity
                    ),
                    (8, 8)
                );
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                late_refusal(&runtime, work, total, heap, required);
                assert!(runtime.promises.captures.records.is_empty());
                assert!(runtime.promises.callbacks.records.is_empty());
                let old = if cold { 0 } else { 8 };
                assert_eq!(
                    (
                        runtime.promises.captures.admitted_capacity,
                        runtime.promises.callbacks.admitted_capacity
                    ),
                    (old, old)
                );
            }
        }
    }
}

#[test]
fn promise_argument_vectors_10_12_pay_before_allocate_and_drop_without_steps() {
    enum Arguments {
        One([Value; 1]),
        Two([Value; 2]),
    }
    for (n, total) in [(1, 10), (2, 12)] {
        let required = 32 + n * size_of::<Value>();
        for (work, heap, success) in [
            (total, required, true),
            (total - 1, required, false),
            (total, required - 1, false),
        ] {
            let (mut runtime, _) = pending();
            let Value::Native(marker) = handler() else {
                panic!("native");
            };
            let args = if n == 1 {
                Arguments::One([Value::Native(marker.clone())])
            } else {
                Arguments::Two([Value::Native(marker.clone()), Value::Native(marker.clone())])
            };
            reset(&mut runtime, work, heap);
            let before_heap = runtime.allocated;
            let result = match args {
                Arguments::One(values) => runtime.promise_arguments(values),
                Arguments::Two(values) => runtime.promise_arguments(values),
            };
            assert_eq!(result.is_ok(), success);
            assert_eq!(runtime.steps, 0);
            if success {
                let values = result.unwrap();
                assert_eq!(values.len(), n);
                for value in &values {
                    let Value::Native(found) = value else {
                        panic!("retained native");
                    };
                    assert!(Rc::ptr_eq(found, &marker));
                }
                assert_eq!(Rc::strong_count(&marker), n + 1);
                assert_eq!(runtime.allocated, MAX_HEAP);
                drop(values);
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert_eq!(
                    runtime.allocated,
                    if work < total {
                        before_heap
                    } else {
                        MAX_HEAP + 1
                    }
                );
                assert!(runtime.microtasks_suspended());
            }
            assert_eq!(Rc::strong_count(&marker), 1);
            assert_eq!(runtime.steps, 0);
            assert_eq!(
                (runtime.calls, runtime.eval_depth, runtime.stack_units),
                (0, 0, 0)
            );
            assert!(runtime.console.is_empty());
        }
    }
}

#[test]
fn promise_report_offset_literal104_106_and_heap_cuts() {
    for (offset, total, required, expected) in [
        (
            0,
            104,
            80,
            "Uncaught Promise job exception: marker at byte 0",
        ),
        (
            12,
            106,
            81,
            "Uncaught Promise job exception: marker at byte 12",
        ),
    ] {
        for (work, heap, success) in [
            (total, required, true),
            (total - 1, required, false),
            (total, required - 1, false),
        ] {
            let mut runtime = Runtime::new();
            runtime.console = Vec::with_capacity(4);
            let mut error = ScriptError::new("marker");
            error.offset = Some(offset);
            reset(&mut runtime, work, heap);
            let before_heap = runtime.allocated;
            let result = runtime.report_promise_job_error(error);
            assert_eq!(result.is_ok(), success);
            assert_eq!(runtime.steps, 0);
            if success {
                assert_eq!(runtime.console, [expected]);
                assert_eq!(runtime.allocated, MAX_HEAP);
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert!(runtime.console.is_empty());
                assert!(runtime.microtasks_suspended());
                assert_eq!(
                    runtime.allocated,
                    if work < total {
                        before_heap
                    } else {
                        MAX_HEAP + 1
                    }
                );
            }
        }
    }
}

#[test]
fn promise_report_last_visible_line_and_suppression_exact8_short7() {
    let mut runtime = Runtime::new();
    runtime.console = Vec::with_capacity(1024);
    runtime.console.resize(1023, String::from("kept"));
    reset(&mut runtime, 84, 70);
    runtime
        .report_promise_job_error(ScriptError::new("marker"))
        .unwrap();
    assert_eq!(runtime.console.len(), 1024);
    assert_eq!(
        runtime.console.last().unwrap(),
        "Uncaught Promise job exception: marker"
    );
    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    for count in [1024, 1100] {
        for work in [7, 8] {
            let mut runtime = Runtime::new();
            runtime.console = vec![String::from("kept"); count];
            let before_console = runtime.console.clone();
            reset(&mut runtime, work, 0);
            let result = runtime.report_promise_job_error(ScriptError::at("marker", usize::MAX));
            assert_eq!(result.is_ok(), work == 8);
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            assert_eq!(runtime.console, before_console);
            assert_eq!(runtime.microtasks_suspended(), work == 7);
            if work == 7 {
                assert!(result.unwrap_err().is_resource_limit());
            }
        }
    }
}

#[test]
fn promise_report_fatal_inputs_preserve_identity_without_any_debit() {
    for full in [false, true] {
        for mut error in [
            ScriptError::resource("resource marker"),
            ScriptError::unsupported("unsupported marker"),
        ] {
            let mut runtime = Runtime::new();
            if full {
                runtime.console = vec![String::from("kept"); 1024];
            }
            runtime
                .microtasks
                .queue
                .push_back(QueueJob::Callback(handler()));
            runtime.microtasks.admitted_capacity = 1;
            error.offset = Some(12);
            let expected = error.clone();
            let before_console = runtime.console.clone();
            reset(&mut runtime, 0, 0);
            let result = runtime.report_promise_job_error(error);
            assert_eq!(result.unwrap_err(), expected);
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            assert_eq!(runtime.console, before_console);
            assert_eq!(runtime.microtasks.queue.len(), 1);
            let QueueJob::Callback(value) = &runtime.microtasks.queue[0] else {
                panic!("old callback");
            };
            assert_eq!(value, &handler());
            assert!(runtime.microtasks_suspended());
        }
    }
}

#[test]
fn promise_settlement_second_reserve_failure_keeps_exact_old_state() {
    use allocation_probe::ReserveSite::{SettlementPayload, SettlementQueue};
    let required = 64 + 8 * size_of::<Option<PromiseJob>>() + 8 * size_of::<QueueJob>();
    for site in [SettlementPayload, SettlementQueue] {
        let (mut runtime, id) = pending();
        let child = object(&mut runtime);
        runtime.promises.promises.records[0]
            .reactions
            .records
            .push(ReactionPair {
                fulfilled: Some(handler()),
                rejected: Some(rejector()),
                capability: Capability {
                    promise: child.clone(),
                    resolve: handler(),
                    reject: rejector(),
                },
            });
        runtime.promises.promises.records[0]
            .reactions
            .admitted_capacity = 1;
        runtime.promises.promises.records[0].handled = true;
        runtime
            .promises
            .gates
            .records
            .push(ResolveGate { target: None });
        runtime.promises.gates.admitted_capacity = 1;
        reset(&mut runtime, 48, required);
        let guard = allocation_probe::fail_once(site);
        let result = runtime.settle_promise(id, ReactionKind::Fulfill, Value::Number(9.0));
        let observed = guard.observation();
        drop(guard);
        assert!(result.unwrap_err().is_resource_limit());
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        assert!(runtime.microtasks_suspended());
        let record = &runtime.promises.promises.records[0];
        assert!(matches!(record.settlement, Settlement::Pending));
        assert!(record.handled);
        assert_eq!(record.reactions.admitted_capacity, 1);
        assert_eq!(record.reactions.records.len(), 1);
        let pair = &record.reactions.records[0];
        assert_eq!(pair.fulfilled.as_ref(), Some(&handler()));
        assert_eq!(pair.rejected.as_ref(), Some(&rejector()));
        assert_eq!(pair.capability.promise, child);
        assert_eq!(pair.capability.resolve, handler());
        assert_eq!(pair.capability.reject, rejector());
        assert!(runtime.promises.gates.records[0].target.is_none());
        assert!(runtime.promises.jobs.records.is_empty());
        assert!(runtime.microtasks.queue.is_empty());
        assert_eq!(
            (
                runtime.promises.jobs.admitted_capacity,
                runtime.microtasks.admitted_capacity
            ),
            (0, 0)
        );
        assert_eq!(observed.payload_attempts, 1);
        assert_eq!(observed.queue_completed, 0);
        match site {
            SettlementPayload => {
                assert_eq!(
                    (observed.payload_completed, observed.queue_attempts),
                    (0, 0)
                );
                assert_eq!(runtime.promises.jobs.records.capacity(), 0);
            }
            SettlementQueue => {
                assert_eq!(
                    (observed.payload_completed, observed.queue_attempts),
                    (1, 1)
                );
                assert!(runtime.promises.jobs.records.capacity() >= 8);
            }
        }
        // A fresh Runtime is unaffected after guard restoration; no global
        // allocator state or leftover armed failure can poison this reserve.
        let (mut next, next_id) = pending();
        next.promises.promises.records[0]
            .reactions
            .records
            .push(ReactionPair {
                fulfilled: None,
                rejected: None,
                capability: Capability {
                    promise: Value::Object(0),
                    resolve: handler(),
                    reject: rejector(),
                },
            });
        next.promises.promises.records[0]
            .reactions
            .admitted_capacity = 1;
        reset(&mut next, 48, required);
        next.settle_promise(next_id, ReactionKind::Fulfill, Value::Number(9.0))
            .unwrap();
        assert_eq!(next.microtasks.queue.len(), 1);
        assert!(!next.microtasks_suspended());
    }
    let unwound = std::panic::catch_unwind(|| {
        let _guard = allocation_probe::fail_once(SettlementQueue);
        panic!("Promise reserve probe restoration witness");
    });
    assert!(unwound.is_err());
    let (mut restored, id) = pending();
    restored.promises.promises.records[0]
        .reactions
        .records
        .push(ReactionPair {
            fulfilled: None,
            rejected: None,
            capability: Capability {
                promise: Value::Object(0),
                resolve: handler(),
                reject: rejector(),
            },
        });
    restored.promises.promises.records[0]
        .reactions
        .admitted_capacity = 1;
    reset(&mut restored, 48, required);
    restored
        .settle_promise(id, ReactionKind::Fulfill, Value::Number(9.0))
        .unwrap();
    assert_eq!(restored.microtasks.queue.len(), 1);
}

#[test]
fn promise_direct_reaction32_and_thenable34_share_atomic_cold_admission() {
    let required = 64 + 8 * size_of::<Option<PromiseJob>>() + 8 * size_of::<QueueJob>();
    for thenable_job in [false, true] {
        let total = if thenable_job { 34 } else { 32 };
        for (work, heap, success) in [
            (total, required, true),
            (total - 1, required, false),
            (total, required - 1, false),
        ] {
            let (mut runtime, id) = pending();
            let target = object(&mut runtime);
            let then = handler();
            let capability = Capability {
                promise: target.clone(),
                resolve: handler(),
                reject: rejector(),
            };
            reset(&mut runtime, work, heap);
            let before_heap = runtime.allocated;
            let result = if thenable_job {
                runtime.enqueue_resolve_thenable(id, &target, &then)
            } else {
                runtime.enqueue_settled_reaction(ReactionKind::Reject, None, capability, &target)
            };
            assert_eq!(result.is_ok(), success);
            assert_eq!(runtime.steps, 0);
            if success {
                assert_eq!(runtime.allocated, MAX_HEAP);
                assert_eq!(
                    (
                        runtime.promises.jobs.records.len(),
                        runtime.microtasks.queue.len()
                    ),
                    (1, 1)
                );
                let QueueJob::Promise(job) = &runtime.microtasks.queue[0] else {
                    panic!("Promise FIFO entry");
                };
                assert_eq!(job.0, 0);
                match runtime.promises.jobs.records[0].as_ref().unwrap() {
                    PromiseJob::ResolveThenable {
                        target: promise,
                        thenable,
                        then: captured,
                    } => {
                        assert!(thenable_job);
                        assert_eq!(promise.0, id.0);
                        assert_eq!(thenable, &target);
                        assert_eq!(captured, &then);
                    }
                    PromiseJob::Reaction {
                        handler: selected,
                        capability,
                        argument,
                        kind,
                    } => {
                        assert!(!thenable_job);
                        assert!(selected.is_none());
                        assert_eq!(capability.promise, target);
                        assert_eq!(argument, &target);
                        assert!(matches!(kind, ReactionKind::Reject));
                    }
                }
                assert_eq!(
                    (
                        runtime.promises.jobs.admitted_capacity,
                        runtime.microtasks.admitted_capacity
                    ),
                    (8, 8)
                );
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert!(runtime.promises.jobs.records.is_empty());
                assert!(runtime.microtasks.queue.is_empty());
                assert_eq!(
                    (
                        runtime.promises.jobs.admitted_capacity,
                        runtime.microtasks.admitted_capacity
                    ),
                    (0, 0)
                );
                assert_eq!(
                    runtime.allocated,
                    if work < total {
                        before_heap
                    } else {
                        MAX_HEAP + 1
                    }
                );
                assert!(runtime.microtasks_suspended());
            }
        }
    }
}
