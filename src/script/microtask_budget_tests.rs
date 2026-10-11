//! Private-seam cuts, derived before implementation; no measured callback fees.
use super::*;

fn fixture() -> (Runtime, Document, Value) {
    let runtime = Runtime::new();
    let doc = Document::parse("<body></body>");
    // Already represented native: call tick1 + Object.is SameValue work4.
    let callback = Runtime::native("Object.is", Value::Undefined);
    (runtime, doc, callback)
}

fn queue(runtime: &mut Runtime, callback: &Value) -> Result<Value> {
    runtime.queue_microtask(&Value::Undefined, std::slice::from_ref(callback))
}

fn idle(runtime: &Runtime) {
    assert_eq!(runtime.microtasks.author_depth, 0);
    assert!(!runtime.microtasks.performing_checkpoint);
    assert_eq!(runtime.calls, 0);
    assert_eq!(runtime.eval_depth, 0);
    assert_eq!(runtime.stack_units, 0);
    assert!(runtime.frames.is_empty());
}

#[test]
fn microtask_cold_enqueue_exact16_and_heap_cut_precede_publication() {
    let bytes = 32 + 8 * std::mem::size_of::<Value>();
    for (work, allowance, accepted) in [
        (16, bytes, true),
        (15, bytes, false),
        (16, bytes - 1, false),
    ] {
        let (mut runtime, _, callback) = fixture();
        runtime.steps = work;
        runtime.allocated = MAX_HEAP - allowance;
        let before = runtime.allocated;
        let result = queue(&mut runtime, &callback);
        assert_eq!(result.is_ok(), accepted);
        assert_eq!(runtime.steps, 0);
        if accepted {
            assert_eq!(result.unwrap(), Value::Undefined);
            assert_eq!(runtime.microtasks.admitted_capacity, 8);
            assert_eq!(runtime.microtasks.queue.front(), Some(&callback));
            assert_eq!(runtime.microtasks.queue.len(), 1);
            assert_eq!(runtime.allocated, MAX_HEAP);
            assert!(!runtime.microtasks_suspended());
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            assert_eq!(runtime.microtasks.admitted_capacity, 0);
            assert!(runtime.microtasks.queue.is_empty());
            assert_eq!(
                runtime.allocated,
                if work == 15 { before } else { MAX_HEAP + 1 }
            );
            assert!(runtime.microtasks_suspended());
        }
        idle(&runtime);
    }
}

#[test]
fn microtask_spare8_and_full_growth32_preserve_old_fifo_on_refusal() {
    for work in [8, 7] {
        let (mut runtime, _, callback) = fixture();
        queue(&mut runtime, &callback).unwrap();
        runtime.steps = work;
        runtime.allocated = MAX_HEAP;
        let result = queue(&mut runtime, &callback);
        assert_eq!(result.is_ok(), work == 8);
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, MAX_HEAP);
        assert_eq!(runtime.microtasks.admitted_capacity, 8);
        assert_eq!(
            runtime.microtasks.queue.len(),
            if work == 8 { 2 } else { 1 }
        );
    }
    let bytes = 32 + 16 * std::mem::size_of::<Value>();
    for (work, allowance, accepted) in [
        (32, bytes, true),
        (31, bytes, false),
        (32, bytes - 1, false),
    ] {
        let (mut runtime, _, callback) = fixture();
        for _ in 0..8 {
            queue(&mut runtime, &callback).unwrap();
        }
        // Deliberately create uncharged allocator spare in fixture setup.
        // Admission must still use the separately retained logical limit8.
        runtime.microtasks.queue.try_reserve_exact(24).unwrap();
        assert!(runtime.microtasks.queue.capacity() >= 32);
        assert_eq!(runtime.microtasks.admitted_capacity, 8);
        let before_queue = runtime.microtasks.queue.clone();
        runtime.steps = work;
        runtime.allocated = MAX_HEAP - allowance;
        let before = runtime.allocated;
        let result = queue(&mut runtime, &callback);
        assert_eq!(result.is_ok(), accepted);
        assert_eq!(runtime.steps, 0);
        if accepted {
            assert_eq!(runtime.microtasks.admitted_capacity, 16);
            assert_eq!(runtime.microtasks.queue.len(), 9);
            assert_eq!(runtime.allocated, MAX_HEAP);
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            assert_eq!(runtime.microtasks.admitted_capacity, 8);
            assert_eq!(runtime.microtasks.queue, before_queue);
            assert_eq!(
                runtime.allocated,
                if work == 31 { before } else { MAX_HEAP + 1 }
            );
            assert!(runtime.microtasks_suspended());
        }
        idle(&runtime);
    }
}

#[test]
fn microtask_native_checkpoint29_28_and_unstarted23_have_literal_debits() {
    for work in [29, 28, 24, 23, 16, 7] {
        let (mut runtime, mut doc, callback) = fixture();
        queue(&mut runtime, &callback).unwrap();
        runtime.steps = work;
        let allocated = runtime.allocated;
        let result = runtime.microtask_checkpoint(&mut doc);
        assert_eq!(result.is_ok(), work == 29);
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, allocated);
        assert_eq!(runtime.microtasks.queue.len(), usize::from(work < 24));
        assert_eq!(runtime.microtasks_suspended(), work != 29);
        assert!(runtime.console.is_empty());
        idle(&runtime);
        if work != 29 {
            let remaining = runtime.microtasks.queue.clone();
            runtime.steps = MAX_STEPS;
            runtime.microtask_checkpoint(&mut doc).unwrap();
            assert_eq!(runtime.steps, MAX_STEPS);
            assert_eq!(runtime.microtasks.queue, remaining);
            assert!(runtime.microtasks_suspended());
        }
    }
}

#[test]
fn microtask_author_entry8_and_whole_native_checkpoint37_are_prepaid() {
    for work in [8, 7] {
        let (mut runtime, mut doc, _) = fixture();
        runtime.steps = work;
        let called = std::cell::Cell::new(false);
        let result = runtime.with_author_entry(&mut doc, |_, _| {
            called.set(true);
            Ok(Value::Number(42.0))
        });
        assert_eq!(result.is_ok(), work == 8);
        assert_eq!(called.get(), work == 8);
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.microtasks_suspended(), work == 7);
        idle(&runtime);
    }
    for work in [37, 36] {
        let (mut runtime, mut doc, callback) = fixture();
        queue(&mut runtime, &callback).unwrap();
        runtime.steps = work;
        let result = runtime.with_author_entry(&mut doc, |_, _| Ok(Value::Number(42.0)));
        if work == 37 {
            assert_eq!(result.unwrap(), Value::Number(42.0));
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        assert_eq!(runtime.steps, 0);
        assert!(runtime.microtasks.queue.is_empty());
        assert_eq!(runtime.microtasks_suspended(), work == 36);
        idle(&runtime);
    }
}

#[test]
fn microtask_reporter_cold88_spare80_and_heap_cuts_are_atomic() {
    for cold in [true, false] {
        let work = if cold { 88 } else { 80 };
        let bytes = 68
            + if cold {
                32 + 4 * std::mem::size_of::<String>()
            } else {
                0
            };
        for (available_work, available_heap, accepted) in [
            (work, bytes, true),
            (work - 1, bytes, false),
            (work, bytes - 1, false),
        ] {
            let (mut runtime, _, _) = fixture();
            if !cold {
                runtime.console = Vec::with_capacity(4);
            }
            runtime.steps = available_work;
            runtime.allocated = MAX_HEAP - available_heap;
            let result = runtime.report_microtask_error(ScriptError::new("marker"));
            assert_eq!(result.is_ok(), accepted);
            assert_eq!(runtime.steps, 0);
            if accepted {
                assert_eq!(runtime.console, ["Uncaught microtask exception: marker"]);
                assert_eq!(runtime.allocated, MAX_HEAP);
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert!(runtime.console.is_empty());
                assert!(runtime.microtasks_suspended());
            }
        }
    }
    // Exercise report refusal through the actual drain, not only the reporter.
    let (mut runtime, mut doc, later) = fixture();
    let throws = Runtime::native("DataView", Value::Undefined);
    queue(&mut runtime, &throws).unwrap();
    queue(&mut runtime, &later).unwrap();
    runtime.steps = MAX_STEPS;
    runtime.allocated = MAX_HEAP;
    assert!(
        runtime
            .microtask_checkpoint(&mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert!(runtime.microtasks_suspended());
    assert_eq!(runtime.microtasks.queue.len(), 1);
    assert_eq!(runtime.microtasks.queue.front(), Some(&later));
    assert!(runtime.console.is_empty());
    idle(&runtime);
}

#[test]
fn microtask_reporter_offset_digits_and_async_threshold_are_explicit() {
    for (offset, work, bytes, expected) in [
        (0, 100, 78, "Uncaught microtask exception: marker at byte 0"),
        (
            12,
            102,
            79,
            "Uncaught microtask exception: marker at byte 12",
        ),
    ] {
        let (mut runtime, _, _) = fixture();
        runtime.console = Vec::with_capacity(4);
        runtime.steps = work;
        runtime.allocated = MAX_HEAP - bytes;
        let mut error = ScriptError::new("marker");
        error.offset = Some(offset);
        runtime.report_microtask_error(error).unwrap();
        assert_eq!(runtime.console, [expected]);
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, MAX_HEAP);
    }
    for count in [1024, 1100] {
        for work in [8, 7] {
            let (mut runtime, _, _) = fixture();
            runtime.console = vec![String::new(); count];
            runtime.steps = work;
            runtime.allocated = MAX_HEAP;
            let result = runtime.report_microtask_error(ScriptError::new("marker"));
            assert_eq!(result.is_ok(), work == 8);
            assert_eq!(runtime.steps, 0);
            assert_eq!(runtime.allocated, MAX_HEAP);
            assert_eq!(runtime.console.len(), count);
            assert_eq!(runtime.microtasks_suspended(), work == 7);
        }
    }
    // These are metadata-only inputs; no impossible String/Vec is allocated.
    assert!(
        microtasks::report_plan(usize::MAX, None, 0, 0)
            .unwrap_err()
            .is_resource_limit()
    );
    assert!(
        microtasks::report_plan(usize::MAX / 2, None, 0, 0)
            .unwrap_err()
            .is_resource_limit()
    );
}

#[test]
fn microtask_self_replenishing_fifo_shares_remaining_work_and_stays_suspended() {
    let (mut runtime, mut doc, _) = fixture();
    let callback = runtime
        .execute(
            "var count=0;function again(){count++;queueMicrotask(again);}again;",
            &mut doc,
        )
        .unwrap();
    queue(&mut runtime, &callback).unwrap();
    runtime.steps = 200;
    assert!(
        runtime
            .microtask_checkpoint(&mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.steps, 0);
    assert!(runtime.allocated < MAX_HEAP);
    assert!(runtime.microtasks_suspended());
    idle(&runtime);
    let count = runtime.execute("count", &mut doc).unwrap();
    let Value::Number(value) = count else {
        panic!("numeric counter")
    };
    assert!(value > 0.0 && value < 200.0);
    let queued = runtime.microtasks.queue.len();
    for _ in 0..3 {
        runtime.steps = MAX_STEPS;
        runtime.microtask_checkpoint(&mut doc).unwrap();
        assert_eq!(
            runtime.execute("count", &mut doc).unwrap(),
            Value::Number(value)
        );
        assert_eq!(runtime.microtasks.queue.len(), queued);
        assert!(runtime.microtasks_suspended());
    }
}

#[test]
fn microtask_empty_fifo_fatal_script_suspends_direct_details_pump_before_reset() {
    for source in ["while(true){}", "/a/u"] {
        let (mut runtime, mut doc, _) = fixture();
        runtime
            .execute(
                "var d=document.createElement('details');document.body.appendChild(d);d.open=true;",
                &mut doc,
            )
            .unwrap();
        let queued = doc.peek_details_toggle().unwrap();
        let error = runtime.execute(source, &mut doc).unwrap_err();
        assert!(error.is_resource_limit() || error.is_unsupported());
        assert!(runtime.microtasks.queue.is_empty());
        assert!(runtime.microtasks_suspended());
        runtime.steps = 123;
        let allocated = runtime.allocated;
        assert!(
            runtime
                .dispatch_details_toggles(&mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.steps, 123);
        assert_eq!(runtime.allocated, allocated);
        assert_eq!(doc.peek_details_toggle(), Some(queued));
        assert!(!doc.details_toggle_running());
        idle(&runtime);
    }
}
