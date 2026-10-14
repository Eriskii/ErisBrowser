//! Candidate private API witnesses; freeze before production, never run on AH.
use super::*;
use std::mem::size_of;

fn handler() -> Value {
    Runtime::native("Object.is", Value::Undefined)
}
fn rejection() -> Value {
    Runtime::native("Number.isNaN", Value::Undefined)
}
fn object(runtime: &mut Runtime) -> Value {
    runtime.object_ordered([]).unwrap()
}
fn fixture(
    n: usize,
    jlen: usize,
    jcap: usize,
    qlen: usize,
    qcap: usize,
) -> (Runtime, PromiseId, Vec<Value>) {
    let mut runtime = Runtime::new();
    assert!(runtime.promises.promises.records.is_empty());
    assert!(runtime.promises.jobs.records.is_empty());
    assert!(runtime.microtasks.queue.is_empty());
    let Value::Object(object_id) = object(&mut runtime) else {
        panic!("object");
    };
    let mut reactions = Vec::new();
    let mut capabilities = Vec::new();
    for _ in 0..n {
        let promise = object(&mut runtime);
        capabilities.push(promise.clone());
        reactions.push(ReactionPair {
            fulfilled: Some(handler()),
            rejected: Some(rejection()),
            capability: Capability {
                promise,
                resolve: handler(),
                reject: rejection(),
            },
        });
    }
    runtime.promises.promises.records.push(PromiseRecord {
        object_id,
        settlement: Settlement::Pending,
        reactions: Arena {
            admitted_capacity: n,
            records: reactions,
        },
        handled: false,
    });
    runtime.promises.promises.admitted_capacity = 1;
    runtime.promises.jobs.records = (0..jlen).map(|_| None).collect();
    runtime
        .promises
        .jobs
        .records
        .try_reserve_exact(jcap - jlen)
        .unwrap();
    runtime.promises.jobs.admitted_capacity = jcap;
    runtime
        .microtasks
        .queue
        .extend((0..qlen).map(|_| QueueJob::Callback(handler())));
    runtime
        .microtasks
        .queue
        .try_reserve_exact(qcap - qlen)
        .unwrap();
    runtime.microtasks.admitted_capacity = qcap;
    // All fixture allocation is outside the cut; the operation gets exactly
    // the selected remaining work/heap below. No fake production credit.
    (runtime, PromiseId(0), capabilities)
}
fn cut(runtime: &mut Runtime, work: usize, bytes: usize) {
    runtime.steps = work;
    runtime.allocated = MAX_HEAP - bytes;
}
fn is_pending(runtime: &Runtime, count: usize) {
    let record = &runtime.promises.promises.records[0];
    assert!(matches!(&record.settlement, Settlement::Pending));
    assert_eq!(record.reactions.records.len(), count);
}
fn published(runtime: &Runtime, start: usize, queue_prefix: usize, capabilities: &[Value]) {
    assert!(matches!(&runtime.promises.promises.records[0].settlement,
        Settlement::Fulfilled(Value::Number(value)) if *value == 9.0));
    assert!(
        runtime.promises.promises.records[0]
            .reactions
            .records
            .is_empty()
    );
    assert_eq!(
        runtime.promises.promises.records[0]
            .reactions
            .admitted_capacity,
        0
    );
    for (i, expected) in capabilities.iter().enumerate() {
        let QueueJob::Promise(id) = &runtime.microtasks.queue[queue_prefix + i] else {
            panic!("job ID");
        };
        assert_eq!(id.0, start + i);
        let Some(PromiseJob::Reaction {
            handler: selected,
            capability,
            argument,
            kind,
        }) = &runtime.promises.jobs.records[start + i]
        else {
            panic!("reaction payload");
        };
        assert_eq!(selected.as_ref(), Some(&handler()));
        assert_eq!(&capability.promise, expected);
        assert_eq!(argument, &Value::Number(9.0));
        assert!(matches!(kind, ReactionKind::Fulfill));
    }
}

#[test]
fn promise_growth_plan_zero_cold_batch_spare_and_overflow() {
    let zero = growth_plan::<Option<PromiseJob>>(0, 0, 0).unwrap();
    assert_eq!((zero.target, zero.work, zero.bytes), (None, 0, 0));
    let cold = growth_plan::<Option<PromiseJob>>(0, 0, 1).unwrap();
    assert_eq!(
        (cold.target, cold.work, cold.bytes),
        (Some(8), 8, 32 + 8 * size_of::<Option<PromiseJob>>())
    );
    let batch = growth_plan::<Option<PromiseJob>>(0, 0, 11).unwrap();
    assert_eq!(
        (batch.target, batch.work, batch.bytes),
        (Some(16), 8, 32 + 16 * size_of::<Option<PromiseJob>>())
    );
    let spare = growth_plan::<QueueJob>(7, 8, 1).unwrap();
    assert_eq!((spare.target, spare.work, spare.bytes), (None, 0, 0));
    assert!(growth_plan::<QueueJob>(1, 0, 1).is_err());
    assert!(growth_plan::<QueueJob>(usize::MAX, usize::MAX, 1).is_err());
    assert!(growth_plan::<QueueJob>(0, 0, MAX_HEAP / size_of::<QueueJob>() + 1).is_err());
}

#[test]
fn promise_settlement_cold_literal_16_48_80_cuts_are_atomic() {
    for (n, work) in [(0, 16), (1, 48), (3, 80)] {
        let bytes = if n == 0 {
            0
        } else {
            64 + 8 * size_of::<Option<PromiseJob>>() + 8 * size_of::<QueueJob>()
        };
        let mut cuts = vec![(work, bytes, true), (work - 1, bytes, false)];
        if bytes != 0 {
            cuts.push((work, bytes - 1, false));
        }
        for (allow_work, allow_bytes, success) in cuts {
            let (mut runtime, id, capabilities) = fixture(n, 0, 0, 0, 0);
            cut(&mut runtime, allow_work, allow_bytes);
            let before_heap = runtime.allocated;
            let result = runtime.settle_promise(id, ReactionKind::Fulfill, Value::Number(9.0));
            assert_eq!(result.is_ok(), success);
            assert_eq!(runtime.steps, 0);
            if success {
                published(&runtime, 0, 0, &capabilities);
                assert_eq!(runtime.allocated, MAX_HEAP);
                assert_eq!(runtime.promises.jobs.records.len(), n);
                assert_eq!(runtime.microtasks.queue.len(), n);
                assert_eq!(
                    runtime.promises.jobs.admitted_capacity,
                    if n == 0 { 0 } else { 8 }
                );
                assert_eq!(
                    runtime.microtasks.admitted_capacity,
                    if n == 0 { 0 } else { 8 }
                );
                assert!(!runtime.microtasks_suspended());
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                is_pending(&runtime, n);
                assert!(runtime.promises.jobs.records.is_empty());
                assert!(runtime.microtasks.queue.is_empty());
                assert_eq!(runtime.promises.jobs.admitted_capacity, 0);
                assert_eq!(runtime.microtasks.admitted_capacity, 0);
                assert_eq!(
                    runtime.allocated,
                    if allow_work < work {
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

#[test]
fn promise_settlement_admitted_spare_and_unpaid_physical_spare_differ() {
    let (mut runtime, id, capabilities) = fixture(1, 0, 8, 0, 8);
    cut(&mut runtime, 32, 0);
    runtime
        .settle_promise(id, ReactionKind::Fulfill, Value::Number(9.0))
        .unwrap();
    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    published(&runtime, 0, 0, &capabilities);
    let bytes = 64 + 8 * size_of::<Option<PromiseJob>>() + 8 * size_of::<QueueJob>();
    let (mut runtime, id, _) = fixture(1, 0, 0, 0, 0);
    runtime.promises.jobs.records.try_reserve_exact(32).unwrap();
    runtime.microtasks.queue.try_reserve_exact(32).unwrap();
    cut(&mut runtime, 48, bytes - 1);
    assert!(
        runtime
            .settle_promise(id, ReactionKind::Fulfill, Value::Number(9.0))
            .unwrap_err()
            .is_resource_limit()
    );
    is_pending(&runtime, 1);
    assert_eq!(
        (
            runtime.promises.jobs.admitted_capacity,
            runtime.microtasks.admitted_capacity
        ),
        (0, 0)
    );
    assert!(runtime.promises.jobs.records.is_empty());
    assert!(runtime.microtasks.queue.is_empty());
    assert_eq!(runtime.allocated, MAX_HEAP + 1);
}

#[test]
fn promise_settlement_preserves_prefix_tombstones_and_charges_final_targets() {
    let jm = 1 + (size_of::<Option<PromiseJob>>() - 1) / size_of::<Value>();
    let qm = 1 + (size_of::<QueueJob>() - 1) / size_of::<Value>();
    let work = 64 + 14 * (jm + qm);
    let bytes = 64 + 16 * size_of::<Option<PromiseJob>>() + 16 * size_of::<QueueJob>();
    for success in [false, true] {
        let (mut runtime, id, capabilities) = fixture(2, 7, 8, 7, 8);
        cut(&mut runtime, work - usize::from(!success), bytes);
        let before_heap = runtime.allocated;
        let result = runtime.settle_promise(id, ReactionKind::Fulfill, Value::Number(9.0));
        assert_eq!(result.is_ok(), success);
        assert_eq!(runtime.steps, 0);
        assert!(
            runtime.promises.jobs.records[..7]
                .iter()
                .all(Option::is_none)
        );
        for i in 0..7 {
            let QueueJob::Callback(value) = &runtime.microtasks.queue[i] else {
                panic!("old callback");
            };
            assert_eq!(value, &handler());
        }
        if success {
            published(&runtime, 7, 7, &capabilities);
            assert_eq!(
                (
                    runtime.promises.jobs.records.len(),
                    runtime.microtasks.queue.len()
                ),
                (9, 9)
            );
            assert_eq!(
                (
                    runtime.promises.jobs.admitted_capacity,
                    runtime.microtasks.admitted_capacity
                ),
                (16, 16)
            );
            assert_eq!(runtime.allocated, MAX_HEAP);
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            is_pending(&runtime, 2);
            assert_eq!(
                (
                    runtime.promises.jobs.records.len(),
                    runtime.microtasks.queue.len()
                ),
                (7, 7)
            );
            assert_eq!(
                (
                    runtime.promises.jobs.admitted_capacity,
                    runtime.microtasks.admitted_capacity
                ),
                (8, 8)
            );
            assert_eq!(runtime.allocated, before_heap);
        }
    }
}

#[test]
fn promise_brand_lookup_prepays_literal_search_bounds() {
    for (count, work) in [(0, 8), (1, 10), (7, 14), (8, 16)] {
        for enough in [false, true] {
            let mut runtime = Runtime::new();
            let mut last = Value::Object(usize::MAX);
            for _ in 0..count {
                let Value::Object(object_id) = object(&mut runtime) else {
                    panic!("object");
                };
                last = Value::Object(object_id);
                runtime.promises.promises.records.push(PromiseRecord {
                    object_id,
                    settlement: Settlement::Pending,
                    reactions: Arena {
                        records: Vec::new(),
                        admitted_capacity: 0,
                    },
                    handled: false,
                });
            }
            runtime.promises.promises.admitted_capacity = count;
            cut(&mut runtime, work - usize::from(!enough), 0);
            let result = runtime.promise_id(&last);
            assert_eq!(runtime.steps, 0);
            assert_eq!(runtime.allocated, MAX_HEAP);
            if enough {
                let id = result.unwrap();
                if count == 0 {
                    assert!(id.is_none());
                } else {
                    assert_eq!(id.unwrap().0, count - 1);
                }
            } else {
                assert!(result.unwrap_err().is_resource_limit());
            }
        }
    }
}

#[test]
fn promise_pending_reaction_cold26_precedes_copies_and_publication() {
    let bytes = 32 + 8 * size_of::<ReactionPair>();
    for (work, heap, success) in [
        (26, bytes, true),
        (25, bytes, false),
        (26, bytes - 1, false),
    ] {
        let (mut runtime, id, _) = fixture(0, 0, 0, 0, 0);
        let capability = Capability {
            promise: object(&mut runtime),
            resolve: handler(),
            reject: rejection(),
        };
        let on_fulfilled = handler();
        let on_rejected = rejection();
        cut(&mut runtime, work, heap);
        let before_heap = runtime.allocated;
        let result = runtime.append_promise_reaction(
            id,
            Some(&on_fulfilled),
            Some(&on_rejected),
            &capability,
        );
        assert_eq!(result.is_ok(), success);
        assert_eq!(runtime.steps, 0);
        let record = &runtime.promises.promises.records[0];
        assert_eq!(record.reactions.records.len(), usize::from(success));
        assert_eq!(
            record.reactions.admitted_capacity,
            if success { 8 } else { 0 }
        );
        assert_eq!(record.handled, success);
        assert!(runtime.promises.jobs.records.is_empty());
        assert!(runtime.microtasks.queue.is_empty());
        assert_eq!(
            runtime.allocated,
            if success {
                MAX_HEAP
            } else if work == 25 {
                before_heap
            } else {
                MAX_HEAP + 1
            }
        );
        if !success {
            assert!(result.unwrap_err().is_resource_limit());
        }
    }
}

#[test]
fn promise_job_prepop32_and_sticky_failure_do_not_replay() {
    for work in [7, 23, 31, 32] {
        let (mut runtime, id, _) = fixture(1, 0, 0, 0, 0);
        // No handler: first body action is the separately prepaid one-argument
        // builder for the generic capability resolve call.
        runtime.promises.promises.records[0].reactions.records[0].fulfilled = None;
        runtime
            .settle_promise(id, ReactionKind::Fulfill, Value::Number(9.0))
            .unwrap();
        let mut doc = Document::parse("");
        cut(&mut runtime, work, 0);
        let result = runtime.microtask_checkpoint(&mut doc);
        assert!(result.unwrap_err().is_resource_limit());
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.microtasks.queue.len(), usize::from(work < 32));
        assert_eq!(runtime.promises.jobs.records[0].is_some(), work < 32);
        assert!(runtime.microtasks_suspended());
        assert_eq!(runtime.microtasks.author_depth, 0);
        assert!(!runtime.microtasks.performing_checkpoint);
        assert!(runtime.console.is_empty());
        runtime.steps = MAX_STEPS;
        runtime.microtask_checkpoint(&mut doc).unwrap();
        assert_eq!(runtime.steps, MAX_STEPS);
        assert_eq!(runtime.microtasks.queue.len(), usize::from(work < 32));
        assert_eq!(runtime.promises.jobs.records[0].is_some(), work < 32);
    }
}

#[test]
fn promise_abrupt_job_report_uses_literal32_byte_prefix_and_cuts() {
    for cold in [false, true] {
        let work = if cold { 92 } else { 84 };
        let bytes = 70
            + if cold {
                32 + 4 * size_of::<String>()
            } else {
                0
            };
        for (w, h, success) in [
            (work, bytes, true),
            (work - 1, bytes, false),
            (work, bytes - 1, false),
        ] {
            let mut runtime = Runtime::new();
            if !cold {
                runtime.console = Vec::with_capacity(4);
            }
            cut(&mut runtime, w, h);
            let before_heap = runtime.allocated;
            let result = runtime.report_promise_job_error(ScriptError::new("marker"));
            assert_eq!(result.is_ok(), success);
            assert_eq!(runtime.steps, 0);
            if success {
                assert_eq!(runtime.console, ["Uncaught Promise job exception: marker"]);
                assert_eq!(runtime.allocated, MAX_HEAP);
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert!(runtime.console.is_empty());
                assert_eq!(
                    runtime.allocated,
                    if w < work { before_heap } else { MAX_HEAP + 1 }
                );
            }
        }
    }
    let mut runtime = Runtime::new();
    runtime.console = vec![String::from("kept"); 1024];
    cut(&mut runtime, 8, 0);
    runtime
        .report_promise_job_error(ScriptError::new("marker"))
        .unwrap();
    assert_eq!(
        (runtime.steps, runtime.allocated, runtime.console.len()),
        (0, MAX_HEAP, 1024)
    );
}
