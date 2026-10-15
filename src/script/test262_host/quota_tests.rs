//! Isolated host admission cuts, frozen before production. Feature-only APIs.
use super::*;
use std::cell::Cell;
use std::mem::{align_of, size_of};

const DONE: &str = include_str!(
    "../../../tests/upstream/test262-promise-methods-async/harness/doneprintHandle.js"
);

fn metadata_heap() -> usize {
    let k = size_of::<PropertyKey>();
    let property_block = 16 * (k + size_of::<Property>()) + 32 * size_of::<usize>() + 64;
    let binding_block =
        16 * (size_of::<String>() + size_of::<Binding>()) + 32 * size_of::<usize>() + 64;
    1878 + 2 * size_of::<Option<AbortSlot>>()
        + 2 * size_of::<Option<f64>>()
        + 4 * k
        + 2 * property_block
        + 2 * size_of::<Native>()
        + 4 * binding_block
        + 2 * size_of::<ScriptObject>()
}

fn warm() -> Runtime {
    let mut runtime = Runtime::new();
    runtime.objects.try_reserve_exact(2).unwrap();
    runtime.steps = MAX_STEPS;
    runtime.allocated = 0;
    runtime
}

fn hook(runtime: &Runtime, name: &str) -> Rc<Native> {
    let Value::Native(native) = &runtime.environments[0].bindings[name].value else {
        panic!("private host hook must be a Native");
    };
    native.clone()
}

fn harnessed(policy: Test262Policy, arm: bool) -> (Runtime, Document) {
    let mut runtime = Runtime::new();
    let mut doc = Document::parse("");
    runtime.enable_test262_host(policy).unwrap();
    if policy == Test262Policy::AsyncDone {
        runtime.execute(DONE, &mut doc).unwrap();
        runtime.bind_test262_done_from_harness().unwrap();
    }
    if arm {
        runtime.arm_test262_body().unwrap();
    }
    (runtime, doc)
}

#[test]
fn host_setup_exact_work_and_heap_never_publish_partial_identity() {
    for (work, good) in [(405, true), (404, false)] {
        let mut runtime = warm();
        let objects = runtime.objects.len();
        let globals = runtime.environments[0].bindings.len();
        let order = runtime.next_global_order;
        runtime.steps = work;
        let result = runtime.enable_test262_host(Test262Policy::AsyncDone);
        assert_eq!(result.is_ok(), good);
        assert_eq!(runtime.steps, 0);
        assert_eq!(
            runtime.objects.len(),
            objects + 2,
            "staged bags may remain unexposed"
        );
        assert_eq!(
            runtime.environments[0].bindings.len(),
            globals + usize::from(good)
        );
        assert_eq!(runtime.next_global_order, order + u64::from(good));
        assert_eq!(runtime.test262_session.is_some(), good);
        assert_eq!(runtime.environments[0].bindings.contains_key("print"), good);
        if good {
            assert_eq!(runtime.allocated, metadata_heap());
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
    }
    for short in [false, true] {
        let mut runtime = warm();
        runtime.steps = 405;
        runtime.allocated = MAX_HEAP - metadata_heap() + usize::from(short);
        let globals = runtime.environments[0].bindings.len();
        let result = runtime.enable_test262_host(Test262Policy::CheckedSync);
        assert_eq!(result.is_ok(), !short);
        assert_eq!(runtime.allocated, MAX_HEAP + usize::from(short));
        assert_eq!(
            runtime.environments[0].bindings.len(),
            globals + usize::from(!short)
        );
        assert_eq!(runtime.test262_session.is_some(), !short);
        if short {
            assert!(result.unwrap_err().is_resource_limit());
        }
    }
}

#[test]
fn host_slot_growth_and_injected_reserve_failure_precede_all_metadata() {
    let plan = object_plan(749, 749).unwrap();
    assert_eq!(plan.target, Some(1498));
    assert_eq!(
        plan.work,
        8 + 1498 * size_of::<ScriptObject>().div_ceil(size_of::<Value>())
    );
    assert_eq!(plan.bytes, 32 + 1500 * size_of::<ScriptObject>());
    let spare = object_plan(749, 751).unwrap();
    assert_eq!(
        (spare.target, spare.work, spare.bytes),
        (None, 0, 2 * size_of::<ScriptObject>())
    );
    let mut runtime = Runtime::new();
    let before = runtime.objects.len();
    let globals = runtime.environments[0].bindings.len();
    let growth = object_plan(before, runtime.objects.capacity()).unwrap();
    assert!(
        growth.target.is_some(),
        "parent eager arena has no host slots"
    );
    runtime.steps = 32 + growth.work;
    runtime.allocated = 0;
    let calls = Cell::new(0);
    let error = runtime
        .enable_test262_host_with(Test262Policy::AsyncDone, |_, target| {
            calls.set(calls.get() + 1);
            assert_eq!(Some(target), growth.target);
            Err(ScriptError::resource("fixture reserve refusal"))
        })
        .unwrap_err();
    assert!(error.is_resource_limit());
    assert_eq!(
        (calls.get(), runtime.steps, runtime.allocated),
        (1, 0, growth.bytes)
    );
    assert_eq!(runtime.objects.len(), before);
    assert_eq!(runtime.environments[0].bindings.len(), globals);
    assert!(runtime.test262_session.is_none());
    assert!(!runtime.environments[0].bindings.contains_key("print"));
}

#[test]
fn host_binding_and_arm_cuts_preserve_original_harness_until_admitted() {
    for work in [49, 48] {
        let mut runtime = Runtime::new();
        let mut doc = Document::parse("");
        runtime
            .enable_test262_host(Test262Policy::AsyncDone)
            .unwrap();
        runtime.execute(DONE, &mut doc).unwrap();
        let original = runtime.environments[0].bindings["$DONE"].value.clone();
        assert!(matches!(original, Value::Function(_)));
        let order = runtime.environments[0].bindings["$DONE"].global_order;
        runtime.steps = work;
        let allocated = runtime.allocated;
        let result = runtime.bind_test262_done_from_harness();
        assert_eq!(result.is_ok(), work == 49);
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, allocated);
        assert_eq!(
            runtime.environments[0].bindings["$DONE"].global_order,
            order
        );
        if work == 48 {
            assert_eq!(runtime.environments[0].bindings["$DONE"].value, original);
            assert!(
                runtime
                    .test262_session
                    .as_ref()
                    .unwrap()
                    .pending_done
                    .is_some()
            );
            assert!(
                runtime
                    .test262_session
                    .as_ref()
                    .unwrap()
                    .original_done
                    .is_none()
            );
        } else {
            assert!(matches!(
                runtime.environments[0].bindings["$DONE"].value,
                Value::Native(_)
            ));
            assert!(
                runtime
                    .test262_session
                    .as_ref()
                    .unwrap()
                    .pending_done
                    .is_none()
            );
            assert!(
                runtime
                    .test262_session
                    .as_ref()
                    .unwrap()
                    .original_done
                    .is_some()
            );
        }
    }
    for work in [16, 15] {
        let (mut runtime, _) = harnessed(Test262Policy::AsyncDone, false);
        runtime.steps = work;
        let allocated = runtime.allocated;
        assert_eq!(runtime.arm_test262_body().is_ok(), work == 16);
        assert_eq!((runtime.steps, runtime.allocated), (0, allocated));
        assert_eq!(
            matches!(
                runtime.test262_session.as_ref().unwrap().phase,
                Phase::Armed
            ),
            work == 16
        );
    }
}

#[test]
fn host_done_begin_is_prepaid_and_cleanup_restores_nested_scope_without_calls() {
    for work in [24, 23] {
        let (mut runtime, _) = harnessed(Test262Policy::AsyncDone, true);
        let done = hook(&runtime, "$DONE");
        runtime.steps = work;
        let allocated = runtime.allocated;
        let result = runtime.test262_done_begin(&done, &[Value::Undefined]);
        assert_eq!(result.is_ok(), work == 24);
        assert_eq!((runtime.steps, runtime.allocated), (0, allocated));
        let state = runtime.test262_session.as_ref().unwrap();
        assert_eq!(
            (state.done_calls, state.active_done_depth),
            if work == 24 { (1, 1) } else { (0, 0) }
        );
        if let Ok(token) = result {
            runtime.test262_done_end(token, &Ok(Value::Undefined));
            assert_eq!(
                runtime.test262_session.as_ref().unwrap().active_done_depth,
                0
            );
        }
    }
    // Manual Rust scope fixture, not a delegated JS completion or budget reset.
    let (mut runtime, _) = harnessed(Test262Policy::AsyncDone, true);
    let done = hook(&runtime, "$DONE");
    runtime.test262_session.as_mut().unwrap().active_done_depth = 2;
    runtime.steps = 24;
    let token = runtime
        .test262_done_begin(&done, &[Value::Bool(true)])
        .unwrap();
    assert_eq!(
        runtime.test262_session.as_ref().unwrap().active_done_depth,
        3
    );
    runtime.test262_done_end(token, &Err(ScriptError::thrown(Value::Number(7.0))));
    let state = runtime.test262_session.as_ref().unwrap();
    assert_eq!(state.active_done_depth, 2);
    assert_eq!(state.flags & 0x41, 0x41);
}

#[test]
fn host_print_admission_never_formats_or_accepts_public_sentinels() {
    for work in [66, 65] {
        let (mut runtime, _) = harnessed(Test262Policy::AsyncDone, true);
        let print = hook(&runtime, "print");
        let args = [Value::String("Test262:AsyncTestComplete".into())];
        runtime.test262_session.as_mut().unwrap().active_done_depth = 1;
        runtime.steps = work;
        let allocated = runtime.allocated;
        assert_eq!(runtime.test262_print(&print, &args).is_ok(), work == 66);
        assert_eq!((runtime.steps, runtime.allocated), (0, allocated));
        assert_eq!(
            runtime.test262_session.as_ref().unwrap().success_prints,
            u8::from(work == 66)
        );
        assert!(runtime.console.is_empty());
    }
    let (mut runtime, _) = harnessed(Test262Policy::AsyncDone, true);
    let print = hook(&runtime, "print");
    runtime.steps = 16;
    let allocated = runtime.allocated;
    runtime
        .test262_print(&print, &[Value::String("Test262:AsyncTestComplete".into())])
        .unwrap();
    assert_eq!((runtime.steps, runtime.allocated), (0, allocated));
    let state = runtime.test262_session.as_ref().unwrap();
    assert_eq!(
        (state.done_calls, state.success_prints, state.failure_prints),
        (0, 0, 0)
    );
    assert_eq!(state.flags & 0x10, 0x10);
    assert!(runtime.console.is_empty());
}

#[test]
fn host_final_audit_shares_remaining_work_and_never_reads_before_admission() {
    for (source, n, unhandled, exact) in [
        ("1", 0, 0, 24),
        ("Promise.reject(7);", 1, 1, 27),
        (
            "Promise.reject(7);Promise.resolve(1);new Promise(function(){});Promise.reject('x');",
            4,
            2,
            36,
        ),
    ] {
        for short in [false, true] {
            let (mut runtime, mut doc) = harnessed(Test262Policy::CheckedSync, true);
            let body = runtime.execute(source, &mut doc);
            assert!(body.is_ok());
            runtime.steps = exact - usize::from(short);
            let allocated = runtime.allocated;
            let result = runtime.finish_test262_body(&body);
            assert_eq!(result.is_ok(), !short);
            assert_eq!((runtime.steps, runtime.allocated), (0, allocated));
            let observation = runtime.test262_host_observation().unwrap();
            assert_eq!(observation.audited, !short);
            assert_eq!(observation.unhandled, if short { 0 } else { unhandled });
            assert!(matches!(
                runtime.test262_session.as_ref().unwrap().phase,
                Phase::Finished
            ));
            if short {
                assert!(result.unwrap_err().is_resource_limit());
            }
        }
        for short in [false, true] {
            let (mut runtime, mut doc) = harnessed(Test262Policy::CheckedSync, true);
            runtime.execute(source, &mut doc).unwrap();
            runtime.steps = 8 + 3 * n - usize::from(short);
            let reads = Cell::new(0);
            let result = runtime.count_test262_unhandled_with(|| reads.set(reads.get() + 1));
            assert_eq!(result.is_ok(), !short);
            assert_eq!(reads.get(), if short { 0 } else { n });
            assert_eq!(runtime.steps, 0);
            if let Ok(count) = result {
                assert_eq!(count, unhandled);
            }
        }
    }
}

#[test]
fn host_ordinary_job_failure_is_sticky_before_capped_reporter_or_console_clear() {
    for capped in [false, true] {
        let (mut runtime, mut doc) = harnessed(Test262Policy::AsyncDone, true);
        if capped {
            runtime.console.resize(1024, "existing".into());
        }
        let body = runtime.execute(
            "queueMicrotask(function(){throw 7;});queueMicrotask(function(){$DONE();});",
            &mut doc,
        );
        assert!(body.is_ok());
        assert_eq!(runtime.console.len(), if capped { 1024 } else { 1 });
        runtime.console.clear();
        let observation = runtime.finish_test262_body(&body).unwrap();
        assert_eq!(
            (
                observation.done_calls,
                observation.success_prints,
                observation.failure_prints
            ),
            (1, 1, 0)
        );
        assert_eq!(observation.flags & 4, 4);
        assert!(observation.quiescent && observation.audited);
        assert_eq!(observation.unhandled, 0);
    }
}

#[test]
fn host_actual_inline_layout_and_inactive_bootstrap_are_observed_without_new_allowance() {
    println!(
        "TEST262_HOST_LAYOUT Session bytes={} align={}",
        size_of::<Session>(),
        align_of::<Session>()
    );
    println!(
        "TEST262_HOST_LAYOUT OptionalSession bytes={} align={}",
        size_of::<Option<Session>>(),
        align_of::<Option<Session>>()
    );
    println!(
        "TEST262_HOST_LAYOUT Observation bytes={} align={}",
        size_of::<Test262Observation>(),
        align_of::<Test262Observation>()
    );
    println!(
        "TEST262_HOST_LAYOUT Runtime bytes={} align={}",
        size_of::<Runtime>(),
        align_of::<Runtime>()
    );
    let raw = Runtime::uninitialized();
    println!(
        "TEST262_HOST_LAYOUT initial_allocated={} header_allowance={}",
        raw.allocated,
        2048.max(size_of::<Runtime>())
    );
    let runtime = raw.finish_bootstrap().unwrap();
    assert_eq!(runtime.steps, 5);
    assert_eq!(runtime.objects.len(), 749);
    assert_eq!(runtime.environments[0].bindings.len(), 212);
    assert_eq!(runtime.native_properties.len(), 299);
    assert!(runtime.test262_session.is_none());
    assert!(!runtime.environments[0].bindings.contains_key("print"));
    assert!(!runtime.environments[0].bindings.contains_key("$DONE"));
    assert_eq!((MAX_STEPS, MAX_HEAP), (100_000, 8 * 1024 * 1024));
}
