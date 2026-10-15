//! Prospective selected cuts. Setup payments are real but isolated-stage resets
//! are not a claim that the complete fixture fits one public transaction.
use super::*;
use crate::script::{MAX_HEAP, MAX_STEPS};
use std::mem::size_of;
use std::rc::Rc;

fn reset(runtime: &mut Runtime, work: usize, heap: usize) {
    runtime.steps = work;
    runtime.allocated = MAX_HEAP - heap;
}
fn resource<T>(result: Result<T>) {
    match result {
        Err(error) => assert!(error.is_resource_limit(), "{error:?}"),
        Ok(_) => panic!("expected ResourceLimit"),
    }
}
fn marker() -> Value {
    Runtime::native("Object.is", Value::Undefined)
}
fn refs(value: &Value) -> usize {
    let Value::Native(native) = value else {
        panic!("native marker")
    };
    Rc::strong_count(native)
}
fn entry(runtime: &mut Runtime) -> Arguments {
    let before = runtime.steps;
    prepay_entry(runtime).unwrap();
    assert_eq!(before - runtime.steps, 8);
    Arguments::empty_prepaid()
}
fn incoming(runtime: &mut Runtime, value: Value) -> ReceivedValue {
    let before = runtime.steps;
    prepay_item(runtime).unwrap();
    assert_eq!(before - runtime.steps, 6);
    receive_prepaid(value)
}
fn prefix(runtime: &mut Runtime, n: usize) -> Arguments {
    let mut values = entry(runtime);
    for i in 0..n {
        let value = incoming(runtime, Value::Number(i as f64));
        values.append_received(runtime, value).unwrap();
    }
    values
}
fn assert_prefix(values: &Arguments, n: usize, admitted: usize) {
    assert_eq!(
        (values.values.len(), values.admitted_capacity),
        (n, admitted)
    );
    for (i, value) in values.values.iter().enumerate() {
        assert_eq!(value, &Value::Number(i as f64));
    }
}
fn clean(runtime: &Runtime) {
    assert_eq!(
        (
            runtime.frames.len(),
            runtime.calls,
            runtime.eval_depth,
            runtime.stack_units
        ),
        (0, 0, 0, 0)
    );
    assert!(runtime.microtasks.queue.is_empty());
    assert!(runtime.console.is_empty());
}
fn iterator_fixture(done: &str, value: &str) -> (Runtime, Document, IteratorState) {
    let mut runtime = Runtime::try_new().unwrap();
    let mut document = Document::parse("");
    let source = format!(
        "var trace='';var count=0;var token={{}};var iterable={{get return(){{trace+='R';throw token;}},[Symbol.iterator]:function(){{trace+='I';return this;}},get next(){{trace+='G';return function(){{trace+='N';count++;return {{get done(){{trace+='D';{done}}},get value(){{trace+='V';{value}}}}};}};}}}};iterable"
    );
    let operand = runtime.execute(&source, &mut document).unwrap();
    let _list = entry(&mut runtime);
    let operand = incoming(&mut runtime, operand);
    let iterator = IteratorState::acquire(&mut runtime, operand, &mut document).unwrap();
    assert_eq!(
        runtime.execute("trace", &mut document).unwrap(),
        Value::String("IG".into())
    );
    (runtime, document, iterator)
}
fn log(runtime: &mut Runtime, document: &mut Document) -> Value {
    // Observation starts a separate synchronous execution only after fee/heap
    // and cleanup assertions for the measured stage have been captured.
    runtime.allocated = 0;
    runtime.execute("trace", document).unwrap()
}

#[test]
fn entry_and_item_cut_receipts_precede_owned_value_receipt() {
    for (budget, ok) in [(7, false), (8, true)] {
        let mut runtime = Runtime::try_new().unwrap();
        reset(&mut runtime, budget, 0);
        let result = prepay_entry(&mut runtime);
        assert_eq!(result.is_ok(), ok);
        if !ok {
            resource(result);
        }
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    }
    for (budget, ok) in [(5, false), (6, true)] {
        let mut runtime = Runtime::try_new().unwrap();
        let values = entry(&mut runtime);
        reset(&mut runtime, budget, 0);
        let result = prepay_item(&mut runtime);
        assert_eq!(result.is_ok(), ok);
        if !ok {
            resource(result);
        }
        assert_prefix(&values, 0, 0);
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    }
}

#[test]
fn ordinary_incoming_handle_survives_one_short_warm_append_without_publication() {
    for (budget, ok) in [(7, false), (8, true)] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut values = prefix(&mut runtime, 1);
        let retained = marker();
        let value = incoming(&mut runtime, retained.clone());
        assert_eq!(refs(&retained), 2);
        reset(&mut runtime, budget, 0);
        let result = values.append_received(&mut runtime, value);
        assert_eq!(result.is_ok(), ok);
        if ok {
            assert_eq!((values.values.len(), values.admitted_capacity), (2, 8));
            assert_eq!(&values.values[1], &retained);
            assert_eq!(refs(&retained), 2);
        } else {
            resource(result);
            assert_prefix(&values, 1, 8);
            assert_eq!(refs(&retained), 1);
        }
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        drop(values);
        assert_eq!(refs(&retained), 1);
        clean(&runtime);
    }
}

#[test]
fn acquisition_first_fee_stops_before_iterator_author_lookup() {
    for budget in [7, 8] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        let source = runtime
            .execute(
                "var trace='';var iterable={get [Symbol.iterator](){trace+='G';throw 9;}};iterable",
                &mut document,
            )
            .unwrap();
        let _values = entry(&mut runtime);
        let source = incoming(&mut runtime, source);
        reset(&mut runtime, budget, MAX_HEAP);
        resource(IteratorState::acquire(&mut runtime, source, &mut document));
        // At8 acquisition succeeds, but the existing GetMethod64 refuses.
        assert_eq!((runtime.steps, runtime.allocated), (0, 0));
        clean(&runtime);
        assert_eq!(log(&mut runtime, &mut document), Value::String("".into()));
    }
}

#[test]
fn retained_iterator_and_valid_step_clones_pay_before_named_gets() {
    for budget in [0, 1] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        let source=runtime.execute("var trace='';var source={[Symbol.iterator]:function(){trace+='I';return this;},get next(){trace+='G';return function(){};}};source",&mut document).unwrap();
        let _values = entry(&mut runtime);
        let ReceivedValue(source) = incoming(&mut runtime, source);
        // Real selected acquisition payment and existing GetMethod/Call reach
        // the returned object. This isolated fixture stops before its next Get.
        let paid = runtime.steps;
        runtime.work(8).unwrap();
        assert_eq!(paid - runtime.steps, 8);
        let method = runtime
            .array_from_iterator_method(&source, &mut document)
            .unwrap()
            .unwrap();
        let iterator = runtime
            .call(method, Vec::new(), source, &mut document)
            .unwrap();
        assert!(matches!(iterator, Value::Object(_)));
        let before = runtime.allocated;
        runtime.steps = budget;
        let result = clone_handle(&mut runtime, &iterator);
        if budget == 0 {
            resource(result);
        } else {
            assert_eq!(result.unwrap(), iterator);
        }
        assert_eq!((runtime.steps, runtime.allocated), (0, before));
        assert_eq!(log(&mut runtime, &mut document), Value::String("I".into()));
    }
    for budget in [0, 1] {
        let (mut runtime, mut document, state) = iterator_fixture("return false;", "return 17;");
        let before_attempt = runtime.steps;
        prepay_attempt(&mut runtime).unwrap();
        assert_eq!(before_attempt - runtime.steps, 8);
        runtime.work(2).unwrap(); // Actual selected cached-next/iterator copy stage.
        let step = runtime
            .call(
                state.next.clone(),
                Vec::new(),
                state.iterator.clone(),
                &mut document,
            )
            .unwrap();
        assert!(matches!(step, Value::Object(_)));
        let before = runtime.allocated;
        runtime.steps = budget;
        let result = clone_handle(&mut runtime, &step);
        if budget == 0 {
            resource(result);
        } else {
            assert_eq!(result.unwrap(), step);
        }
        assert_eq!((runtime.steps, runtime.allocated), (0, before));
        assert_eq!(
            log(&mut runtime, &mut document),
            Value::String("IGN".into())
        );
    }
}

#[test]
fn attempt_and_cached_copy_cuts_cannot_invoke_next() {
    for budget in [7, 8] {
        let (mut runtime, mut document, _state) = iterator_fixture("return true;", "throw 19;");
        runtime.steps = budget;
        let result = prepay_attempt(&mut runtime);
        assert_eq!(result.is_ok(), budget == 8);
        if budget == 7 {
            resource(result);
        }
        assert_eq!(runtime.steps, 0);
        assert_eq!(log(&mut runtime, &mut document), Value::String("IG".into()));
    }
    for budget in [7, 8, 9, 10] {
        let (mut runtime, mut document, mut state) = iterator_fixture("return true;", "throw 19;");
        let before = runtime.allocated;
        runtime.steps = budget;
        resource(state.next_value(&mut runtime, &mut document));
        //10 admits8+2, then the unchanged Call tick refuses before author code.
        assert_eq!((runtime.steps, runtime.allocated), (0, before));
        clean(&runtime);
        assert_eq!(log(&mut runtime, &mut document), Value::String("IG".into()));
    }
}

#[test]
fn fixed_name_work_heap_boundaries_leave_getters_unreached() {
    for (name, work, heap) in [("next", 16, 80), ("done", 16, 80), ("value", 18, 84)] {
        for (available_work, available_heap, ok) in [
            (work - 1, heap, false),
            (work, heap - 1, false),
            (work, heap, true),
        ] {
            let (mut runtime, mut document, _state) = iterator_fixture("return true;", "throw 23;");
            reset(&mut runtime, available_work, available_heap);
            let result = fixed_key(&mut runtime, name);
            assert_eq!(result.is_ok(), ok);
            if ok {
                assert_eq!(result.unwrap(), PropertyKey::from(name));
            } else {
                resource(result);
            }
            assert_eq!(runtime.steps, 0);
            assert_eq!(
                runtime.allocated,
                if available_work < work {
                    MAX_HEAP - available_heap
                } else if available_heap < heap {
                    MAX_HEAP + 1
                } else {
                    MAX_HEAP
                }
            );
            assert_eq!(log(&mut runtime, &mut document), Value::String("IG".into()));
        }
    }
}

#[test]
fn cold_and_ninth_growth_have_exact_work_heap_and_prefix_boundaries() {
    for (n, work, target) in [(0, 16, 8), (8, 32, 16)] {
        let heap = 32 + target * size_of::<Value>();
        for (available_work, available_heap, ok) in [
            (work - 1, heap, false),
            (work, heap - 1, false),
            (work, heap, true),
        ] {
            let mut runtime = Runtime::try_new().unwrap();
            let mut values = prefix(&mut runtime, n);
            let value = incoming(&mut runtime, Value::Number(n as f64));
            let capacity = values.values.capacity();
            reset(&mut runtime, available_work, available_heap);
            let result = values.append_received(&mut runtime, value);
            assert_eq!(result.is_ok(), ok);
            if ok {
                assert_prefix(&values, n + 1, target);
            } else {
                resource(result);
                assert_prefix(&values, n, if n == 0 { 0 } else { 8 });
                assert_eq!(values.values.capacity(), capacity);
            }
            assert_eq!(runtime.steps, 0);
            assert_eq!(
                runtime.allocated,
                if available_work < work {
                    MAX_HEAP - heap
                } else if available_heap < heap {
                    MAX_HEAP + 1
                } else {
                    MAX_HEAP
                }
            );
        }
    }
}

#[test]
fn physical_spare_does_not_waive_logical_growth_or_cumulative_full_buffer_charge() {
    let mut runtime = Runtime::try_new().unwrap();
    let mut values = prefix(&mut runtime, 8);
    // Explicit Rust fixture spare is not logical permission or a measured fee.
    values.values.try_reserve_exact(24).unwrap();
    assert!(values.values.capacity() >= 32);
    let cap = values.values.capacity();
    let value = incoming(&mut runtime, Value::Number(8.0));
    reset(&mut runtime, 31, 32 + 16 * size_of::<Value>());
    resource(values.append_received(&mut runtime, value));
    assert_prefix(&values, 8, 8);
    assert_eq!(values.values.capacity(), cap);
    runtime.steps = MAX_STEPS;
    let value = incoming(&mut runtime, Value::Number(8.0));
    reset(&mut runtime, 32, 32 + 16 * size_of::<Value>());
    values.append_received(&mut runtime, value).unwrap();
    assert_prefix(&values, 9, 16);
    assert_eq!(
        (runtime.steps, runtime.allocated, values.values.capacity()),
        (0, MAX_HEAP, cap)
    );

    let mut runtime = Runtime::try_new().unwrap();
    let before_work = runtime.steps;
    let before_heap = runtime.allocated;
    let mut values = entry(&mut runtime);
    for i in 0..9 {
        let value = incoming(&mut runtime, Value::Number(i as f64));
        values.append_received(&mut runtime, value).unwrap();
        if i == 7 {
            assert_eq!(before_work - runtime.steps, 80 + 8 * 6);
            assert_eq!(runtime.allocated - before_heap, 32 + 8 * size_of::<Value>());
        }
    }
    assert_eq!(before_work - runtime.steps, 112 + 9 * 6);
    assert_eq!(
        runtime.allocated - before_heap,
        64 + 24 * size_of::<Value>()
    );
    assert_prefix(&values, 9, 16);
}

#[test]
fn malformed_logical_or_physical_admission_and_output_limit_are_inert() {
    for (len, admitted, physical) in [
        (0, 8, 0),
        (1, 0, 1),
        (0, 1, 1),
        (0, 7, 8),
        (0, 12, 16),
        (9, 8, 9),
        (0, 131072, 131072),
        (0, usize::MAX, 0),
        (65536, 65536, 65536),
    ] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut values = entry(&mut runtime);
        // Deliberate raw primitive fixture: not a public-limit reachability claim.
        if physical != 0 {
            values.values.try_reserve_exact(physical).unwrap();
        }
        values.values.resize(len, Value::Undefined);
        values.admitted_capacity = admitted;
        let before = (
            values.values.len(),
            values.admitted_capacity,
            values.values.capacity(),
        );
        let retained = marker();
        let value = incoming(&mut runtime, retained.clone());
        reset(&mut runtime, 8, 0);
        resource(values.append_received(&mut runtime, value));
        assert_eq!(
            (
                values.values.len(),
                values.admitted_capacity,
                values.values.capacity()
            ),
            before
        );
        assert!(values.values.iter().all(|v| matches!(v, Value::Undefined)));
        assert_eq!(
            (runtime.steps, runtime.allocated, refs(&retained)),
            (0, MAX_HEAP, 1)
        );
    }
    let mut runtime = Runtime::try_new().unwrap();
    let mut values = entry(&mut runtime);
    let value = incoming(&mut runtime, Value::Undefined);
    runtime.steps = 16;
    runtime.allocated = usize::MAX;
    resource(values.append_received(&mut runtime, value));
    assert_prefix(&values, 0, 0);
    assert_eq!(
        (values.values.capacity(), runtime.steps, runtime.allocated),
        (0, 0, usize::MAX)
    );
}

#[test]
fn reserve_failure_occurs_after_full_admission_once_and_restores_scoped_probe() {
    use allocation_probe::fail_once;
    let mut runtime = Runtime::try_new().unwrap();
    let mut values = entry(&mut runtime);
    let value = incoming(&mut runtime, Value::Number(0.0));
    let guard = fail_once();
    reset(&mut runtime, 16, 32 + 8 * size_of::<Value>());
    resource(values.append_received(&mut runtime, value));
    assert_prefix(&values, 0, 0);
    assert_eq!(values.values.capacity(), 0);
    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    assert_eq!(
        (guard.observation().attempts, guard.observation().completed),
        (1, 0)
    );
    runtime.steps = MAX_STEPS;
    runtime.allocated = 0;
    let value = incoming(&mut runtime, Value::Number(0.0));
    values.append_received(&mut runtime, value).unwrap();
    assert_eq!(
        (guard.observation().attempts, guard.observation().completed),
        (2, 1)
    );
    assert_prefix(&values, 1, 8);
    let outer = guard.observation();
    let unwind = std::panic::catch_unwind(|| {
        let _inner = fail_once();
        panic!("fixture unwind");
    });
    assert!(unwind.is_err());
    assert_eq!(guard.observation(), outer);
    drop(guard);
    clean(&runtime);
}

#[test]
fn reached_step_fee_sum_skips_value_after_done_and_never_prepays_future_attempts() {
    // Reference operations use the existing Call/Get/text APIs and the held
    // literal11 fixed units. No new collector planner supplies expectations.
    for (done, value, expected_trace, throws) in [
        ("return true;", "throw 29;", "IGND", false),
        ("return false;", "return 17;", "IGNDV", false),
        ("throw token;", "return 17;", "IGND", true),
        ("return false;", "throw token;", "IGNDV", true),
    ] {
        let (mut actual, mut actual_doc, mut state) = iterator_fixture(done, value);
        let (mut expected, mut expected_doc, reference) = iterator_fixture(done, value);
        actual.steps = MAX_STEPS;
        expected.steps = MAX_STEPS;
        let a_heap = actual.allocated;
        let e_heap = expected.allocated;
        let observed = state.next_value(&mut actual, &mut actual_doc);
        let oracle = (|| -> Result<Option<Value>> {
            expected.work(11)?;
            let step = expected.call(
                reference.next.clone(),
                Vec::new(),
                reference.iterator.clone(),
                &mut expected_doc,
            )?;
            let done_key: PropertyKey = expected.dom_proto_text("done")?.into();
            let finished = expected.get_property_key(step.clone(), &done_key, &mut expected_doc)?;
            if finished.truthy() {
                return Ok(None);
            }
            let value_key: PropertyKey = expected.dom_proto_text("value")?.into();
            expected
                .get_property_key(step, &value_key, &mut expected_doc)
                .map(Some)
        })();
        assert_eq!(observed.is_err(), throws);
        assert_eq!(oracle.is_err(), throws);
        match (observed, oracle) {
            (Ok(None), Ok(None)) => {}
            (Ok(Some(ReceivedValue(a))), Ok(Some(b))) => assert_eq!(a, b),
            (Err(a), Err(b)) => {
                assert_eq!(a.kind, b.kind);
                assert!(!a.is_resource_limit());
            }
            _ => panic!("step outcome differs"),
        }
        assert_eq!(actual.steps, expected.steps);
        assert_eq!(actual.allocated - a_heap, expected.allocated - e_heap);
        clean(&actual);
        assert_eq!(
            log(&mut actual, &mut actual_doc),
            Value::String(expected_trace.into())
        );
    }
}

#[test]
fn cap_completion_reads_done_or_value_without_closing_or_speculative_next() {
    for (done, value, trace, ok) in [
        ("return true;", "throw token;", "IGND", true),
        ("return false;", "return 31;", "IGNDV", false),
    ] {
        let (mut runtime, mut document, iterator) = iterator_fixture(done, value);
        drop(iterator); // The setup acquisition paid these cached releases.
        let source = runtime.execute("trace='';iterable", &mut document).unwrap();
        let mut values = entry(&mut runtime);
        values.values.resize(65536, Value::Undefined);
        values.admitted_capacity = 65536;
        let source = incoming(&mut runtime, source);
        let result = values.spread_received(&mut runtime, source, &mut document);
        assert_eq!(result.is_ok(), ok);
        if !ok {
            resource(result);
        }
        assert_eq!(
            (values.values.len(), values.admitted_capacity),
            (65536, 65536)
        );
        assert!(values.values.iter().all(|v| matches!(v, Value::Undefined)));
        assert_eq!(
            log(&mut runtime, &mut document),
            Value::String(trace.into())
        );
    }
}

#[test]
fn whole_spread_loop_matches_only_its_known_reached_attempts_and_appends() {
    for (done, yields, trace) in [
        ("return true;", 0, "IGND"),
        ("return count>1;", 1, "IGNDVND"),
    ] {
        let (mut actual, mut actual_doc, old) = iterator_fixture(done, "return 17;");
        drop(old);
        let (mut expected, mut expected_doc, old) = iterator_fixture(done, "return 17;");
        drop(old);
        let actual_source = actual
            .execute("trace='';iterable", &mut actual_doc)
            .unwrap();
        let expected_source = expected
            .execute("trace='';iterable", &mut expected_doc)
            .unwrap();
        let mut actual_values = entry(&mut actual);
        let mut expected_values = entry(&mut expected);
        let actual_source = incoming(&mut actual, actual_source);
        let expected_source = incoming(&mut expected, expected_source);
        let before_actual = (actual.steps, actual.allocated);
        let before_expected = (expected.steps, expected.allocated);
        actual_values
            .spread_received(&mut actual, actual_source, &mut actual_doc)
            .unwrap();
        let mut expected_iterator =
            IteratorState::acquire(&mut expected, expected_source, &mut expected_doc).unwrap();
        // Literal known count from the fixture, not an observed collector result.
        for _ in 0..yields {
            let value = expected_iterator
                .next_value(&mut expected, &mut expected_doc)
                .unwrap()
                .unwrap();
            expected_values
                .append_received(&mut expected, value)
                .unwrap();
        }
        assert!(
            expected_iterator
                .next_value(&mut expected, &mut expected_doc)
                .unwrap()
                .is_none()
        );
        assert_eq!(
            (
                before_actual.0 - actual.steps,
                actual.allocated - before_actual.1
            ),
            (
                before_expected.0 - expected.steps,
                expected.allocated - before_expected.1
            )
        );
        assert_eq!(actual_values.values, expected_values.values);
        assert_eq!(actual_values.values.len(), yields);
        assert_eq!(
            actual_values.admitted_capacity,
            if yields == 0 { 0 } else { 8 }
        );
        assert_eq!(
            log(&mut actual, &mut actual_doc),
            Value::String(trace.into())
        );
    }
}
