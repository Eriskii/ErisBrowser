//! Capture/retention cuts frozen before finally implementation.
use super::*;

fn metadata_bytes() -> usize {
    let leaf =
        16 * (size_of::<PropertyKey>() + size_of::<Property>()) + 32 * size_of::<usize>() + 64;
    904 + size_of::<Option<AbortSlot>>()
        + size_of::<Option<f64>>()
        + 2 * size_of::<PropertyKey>()
        + leaf
        + size_of::<Native>()
}
fn backing_bytes(target: usize) -> usize {
    64 + target * (size_of::<FinallyCapture>() + size_of::<CallbackRecord>())
}
fn fixture(cold: bool) -> (Runtime, Value, Value) {
    let mut runtime = Runtime::try_new().unwrap();
    assert!(runtime.promises.finally_captures.records.is_empty());
    assert!(runtime.promises.callbacks.records.is_empty());
    runtime.objects.try_reserve_exact(4).unwrap();
    if !cold {
        runtime
            .promises
            .finally_captures
            .records
            .try_reserve_exact(8)
            .unwrap();
        runtime.promises.finally_captures.admitted_capacity = 8;
        runtime
            .promises
            .callbacks
            .records
            .try_reserve_exact(8)
            .unwrap();
        runtime.promises.callbacks.admitted_capacity = 8;
    }
    let constructor = runtime.environments[0].bindings["Promise"].value.clone();
    let callback = Runtime::native("Object.is", Value::Undefined);
    (runtime, constructor, callback)
}
fn refs(value: &Value) -> usize {
    let Value::Native(native) = value else {
        panic!("fixture Native");
    };
    Rc::strong_count(native)
}
fn reset(runtime: &mut Runtime, work: usize, bytes: usize) {
    runtime.steps = work;
    runtime.allocated = MAX_HEAP - bytes;
}
fn quiet(runtime: &Runtime) {
    assert!(runtime.microtasks.queue.is_empty());
    assert!(runtime.promises.jobs.records.is_empty());
    assert!(runtime.promises.gates.records.is_empty());
    assert!(runtime.promises.captures.records.is_empty());
    assert!(runtime.console.is_empty());
    assert!(runtime.frames.is_empty());
}
fn metadata(runtime: &Runtime, value: &Value, arity: usize) -> usize {
    let Value::Native(native) = value else {
        panic!("generated callback");
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
    for (key, expected) in [
        ("length", Value::Number(arity as f64)),
        ("name", Value::String("".into())),
    ] {
        let property = &bag.values[&PropertyKey::from(key)];
        assert!(!property.enumerable && property.configurable);
        let PropertyValue::Data { value, writable } = &property.value else {
            panic!("data");
        };
        assert!(!writable);
        assert_eq!(value, &expected);
    }
    id
}
fn same(actual: &Value, expected: &Value) {
    if let (Value::Number(a), Value::Number(b)) = (actual, expected) {
        assert_eq!(a.to_bits(), b.to_bits());
    } else {
        assert_eq!(actual, expected);
    }
}
fn late(runtime: &Runtime, work: usize, total: usize, cold: bool) {
    assert_eq!(runtime.steps, 0);
    assert_eq!(
        runtime.allocated,
        if work < total {
            MAX_HEAP - size_of::<Native>() - 48
        } else {
            MAX_HEAP + 1
        }
    );
    assert!(runtime.microtasks_suspended());
    assert!(runtime.promises.finally_captures.records.is_empty());
    assert!(runtime.promises.callbacks.records.is_empty());
    let admitted = if cold { 0 } else { 8 };
    assert_eq!(
        (
            runtime.promises.finally_captures.admitted_capacity,
            runtime.promises.callbacks.admitted_capacity
        ),
        (admitted, admitted)
    );
    quiet(runtime);
}

#[test]
fn finally_pair246_or262_admits_both_callbacks_and_one_immutable_capture_atomically() {
    for cold in [false, true] {
        let total = if cold { 262 } else { 246 };
        let required = 2 * (metadata_bytes() + size_of::<ScriptObject>())
            + if cold { backing_bytes(8) } else { 0 };
        for (work, bytes, success) in [
            (total, required, true),
            (total - 1, required, false),
            (total, required - 1, false),
        ] {
            let (mut runtime, constructor, callback) = fixture(cold);
            let old = runtime.objects.len();
            let before = (refs(&constructor), refs(&callback));
            reset(&mut runtime, work, bytes);
            let result = runtime.new_finally_functions(&constructor, &callback);
            assert_eq!(result.is_ok(), success);
            assert_eq!(runtime.objects.len(), old + 2);
            quiet(&runtime);
            if success {
                let (fulfilled, rejected) = result.unwrap();
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
                assert_eq!(
                    (
                        metadata(&runtime, &fulfilled, 1),
                        metadata(&runtime, &rejected, 1)
                    ),
                    (old, old + 1)
                );
                assert_ne!(fulfilled, rejected);
                assert_eq!(
                    (refs(&constructor), refs(&callback)),
                    (before.0 + 1, before.1 + 1)
                );
                assert_eq!(runtime.promises.finally_captures.records.len(), 1);
                let FinallyCapture::Finalizer {
                    constructor: saved,
                    on_finally,
                } = &runtime.promises.finally_captures.records[0]
                else {
                    panic!("finalizer");
                };
                assert_eq!(saved, &constructor);
                assert_eq!(on_finally, &callback);
                assert_eq!(runtime.promises.callbacks.records.len(), 2);
                assert!(matches!(
                    runtime.promises.callbacks.records[0].kind,
                    CallbackKind::ThenFinally(FinallyCaptureId(0))
                ));
                assert!(matches!(
                    runtime.promises.callbacks.records[1].kind,
                    CallbackKind::CatchFinally(FinallyCaptureId(0))
                ));
                assert_eq!(
                    (
                        runtime.promises.finally_captures.admitted_capacity,
                        runtime.promises.callbacks.admitted_capacity
                    ),
                    (8, 8)
                );
                drop((fulfilled, rejected));
                assert_eq!(
                    (refs(&constructor), refs(&callback)),
                    (before.0 + 1, before.1 + 1)
                );
                assert!(!runtime.microtasks_suspended());
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                late(&runtime, work, total, cold);
                assert_eq!((refs(&constructor), refs(&callback)), before);
            }
        }
    }
}

#[test]
fn finally_thunk127_or143_retains_exact_outcome_only_after_complete_metadata() {
    for cold in [false, true] {
        for reject in [false, true] {
            let total = if cold { 143 } else { 127 };
            let required = metadata_bytes()
                + size_of::<ScriptObject>()
                + if cold { backing_bytes(8) } else { 0 };
            for (work, bytes, success) in [
                (total, required, true),
                (total - 1, required, false),
                (total, required - 1, false),
            ] {
                let (mut runtime, _, outcome) = fixture(cold);
                let old = runtime.objects.len();
                let before = refs(&outcome);
                reset(&mut runtime, work, bytes);
                let result = runtime.new_finally_thunk(
                    &outcome,
                    if reject {
                        ReactionKind::Reject
                    } else {
                        ReactionKind::Fulfill
                    },
                );
                assert_eq!(result.is_ok(), success);
                assert_eq!(runtime.objects.len(), old + 1);
                quiet(&runtime);
                if success {
                    let callback = result.unwrap();
                    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
                    assert_eq!(metadata(&runtime, &callback, 0), old);
                    assert_eq!(refs(&outcome), before + 1);
                    assert_eq!(runtime.promises.finally_captures.records.len(), 1);
                    let FinallyCapture::Outcome(saved) =
                        &runtime.promises.finally_captures.records[0]
                    else {
                        panic!("outcome");
                    };
                    assert_eq!(saved, &outcome);
                    let expected = &runtime.promises.callbacks.records[0].kind;
                    assert!(if reject {
                        matches!(expected, CallbackKind::ThrowReason(FinallyCaptureId(0)))
                    } else {
                        matches!(expected, CallbackKind::ReturnValue(FinallyCaptureId(0)))
                    });
                    assert_eq!(
                        (
                            runtime.promises.finally_captures.admitted_capacity,
                            runtime.promises.callbacks.admitted_capacity
                        ),
                        (8, 8)
                    );
                } else {
                    assert!(result.unwrap_err().is_resource_limit());
                    late(&runtime, work, total, cold);
                    assert_eq!(refs(&outcome), before);
                }
            }
        }
    }
}

#[test]
fn finally_growth_pays_logical8_to16_despite_physical_spare_and_keeps_old_records() {
    let movement_c = size_of::<CallbackRecord>().div_ceil(size_of::<Value>());
    let movement_f = size_of::<FinallyCapture>().div_ceil(size_of::<Value>());
    let total = 262 + 16 * (movement_c + movement_f);
    let required = 2 * (metadata_bytes() + size_of::<ScriptObject>()) + backing_bytes(16);
    for (work, bytes, success) in [
        (total, required, true),
        (total - 1, required, false),
        (total, required - 1, false),
    ] {
        let (mut runtime, constructor, callback) = fixture(false);
        for index in 0..8 {
            runtime
                .promises
                .finally_captures
                .records
                .push(FinallyCapture::Outcome(Value::Number(index as f64)));
            runtime.promises.callbacks.records.push(CallbackRecord {
                object_id: index,
                kind: CallbackKind::ReturnValue(FinallyCaptureId(index)),
            });
        }
        runtime
            .promises
            .finally_captures
            .records
            .try_reserve_exact(8)
            .unwrap();
        runtime
            .promises
            .callbacks
            .records
            .try_reserve_exact(8)
            .unwrap();
        assert!(runtime.promises.finally_captures.records.capacity() >= 16);
        assert!(runtime.promises.callbacks.records.capacity() >= 16);
        let old = runtime.objects.len();
        reset(&mut runtime, work, bytes);
        let result = runtime.new_finally_functions(&constructor, &callback);
        assert_eq!(result.is_ok(), success);
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.objects.len(), old + 2);
        for index in 0..8 {
            let FinallyCapture::Outcome(saved) = &runtime.promises.finally_captures.records[index]
            else {
                panic!("old outcome");
            };
            assert_eq!(saved, &Value::Number(index as f64));
            assert_eq!(runtime.promises.callbacks.records[index].object_id, index);
            assert!(
                matches!(runtime.promises.callbacks.records[index].kind, CallbackKind::ReturnValue(FinallyCaptureId(id)) if id == index)
            );
        }
        assert_eq!(
            (
                runtime.promises.finally_captures.records.len(),
                runtime.promises.callbacks.records.len()
            ),
            if success { (9, 10) } else { (8, 8) }
        );
        assert_eq!(
            (
                runtime.promises.finally_captures.admitted_capacity,
                runtime.promises.callbacks.admitted_capacity
            ),
            if success { (16, 16) } else { (8, 8) }
        );
        if success {
            assert_eq!(runtime.allocated, MAX_HEAP);
            let (a, b) = result.unwrap();
            assert_eq!(
                (metadata(&runtime, &a, 1), metadata(&runtime, &b, 1)),
                (old, old + 1)
            );
            assert!(matches!(
                runtime.promises.callbacks.records[8].kind,
                CallbackKind::ThenFinally(FinallyCaptureId(8))
            ));
            assert!(matches!(
                runtime.promises.callbacks.records[9].kind,
                CallbackKind::CatchFinally(FinallyCaptureId(8))
            ));
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            assert!(runtime.microtasks_suspended());
            assert_eq!(
                runtime.allocated,
                if work < total {
                    MAX_HEAP - size_of::<Native>() - 48
                } else {
                    MAX_HEAP + 1
                }
            );
        }
        quiet(&runtime);
    }
}

#[test]
fn finally_initial_envelope_prepays_clones_cleanup_and_growth_before_any_payload_copy() {
    for pair in [false, true] {
        for cold in [false, true] {
            let count = if pair { 2 } else { 1 };
            let fixed = if pair { 44 } else { 26 };
            let total = fixed + if cold { 16 } else { 0 };
            let initial_bytes =
                count * size_of::<ScriptObject>() + if cold { backing_bytes(8) } else { 0 };
            for work in [total - 1, total] {
                let (mut runtime, constructor, callback) = fixture(cold);
                let old = runtime.objects.len();
                let before = (refs(&constructor), refs(&callback));
                runtime.steps = work;
                runtime.allocated = 0;
                let refused = if pair {
                    runtime
                        .new_finally_functions(&constructor, &callback)
                        .map(|_| ())
                } else {
                    runtime
                        .new_finally_thunk(&callback, ReactionKind::Fulfill)
                        .map(|_| ())
                };
                assert!(refused.unwrap_err().is_resource_limit());
                assert!(runtime.microtasks_suspended());
                assert_eq!(runtime.steps, 0);
                assert_eq!(
                    runtime.allocated,
                    if work == total { initial_bytes } else { 0 }
                );
                assert_eq!(runtime.objects.len(), old);
                assert_eq!((refs(&constructor), refs(&callback)), before);
                assert!(runtime.promises.finally_captures.records.is_empty());
                assert!(runtime.promises.callbacks.records.is_empty());
                let admitted = if cold { 0 } else { 8 };
                assert_eq!(
                    (
                        runtime.promises.finally_captures.admitted_capacity,
                        runtime.promises.callbacks.admitted_capacity
                    ),
                    (admitted, admitted)
                );
                if cold && work == total {
                    assert!(runtime.promises.finally_captures.records.capacity() >= 8);
                    assert!(runtime.promises.callbacks.records.capacity() >= 8);
                }
                quiet(&runtime);
            }
        }
    }
}

#[test]
fn finally_reads12_and10_are_repeatable_and_pay_before_id_kind_or_clone() {
    for finalizer in [false, true] {
        let total = if finalizer { 12 } else { 10 };
        for wrong in [0, 1, 2] {
            for work in [total - 1, total] {
                let (mut runtime, constructor, callback) = fixture(false);
                runtime
                    .promises
                    .finally_captures
                    .records
                    .push(if finalizer != (wrong == 1) {
                        FinallyCapture::Finalizer {
                            constructor: constructor.clone(),
                            on_finally: callback.clone(),
                        }
                    } else {
                        FinallyCapture::Outcome(callback.clone())
                    });
                let before = (refs(&constructor), refs(&callback));
                let id = FinallyCaptureId(if wrong == 2 { 1 } else { 0 });
                for _ in 0..2 {
                    runtime.steps = work;
                    runtime.allocated = 0;
                    let result = if finalizer {
                        runtime.read_finally_finalizer(id).map(|(a, b)| [a, b])
                    } else {
                        runtime
                            .read_finally_outcome(id)
                            .map(|a| [a, Value::Undefined])
                    };
                    assert_eq!(result.is_ok(), wrong == 0 && work == total);
                    assert_eq!((runtime.steps, runtime.allocated), (0, 0));
                    if wrong == 0 && work == total {
                        let pair = result.unwrap();
                        if finalizer {
                            assert_eq!(pair, [constructor.clone(), callback.clone()]);
                        } else {
                            assert_eq!(pair[0], callback);
                        }
                        drop(pair);
                    } else {
                        assert!(result.unwrap_err().is_resource_limit());
                        assert!(runtime.microtasks_suspended());
                    }
                    assert_eq!((refs(&constructor), refs(&callback)), before);
                    assert_eq!(runtime.promises.finally_captures.records.len(), 1);
                    assert!(runtime.promises.callbacks.records.is_empty());
                    quiet(&runtime);
                }
            }
        }
    }
}

#[test]
fn finally_outcome_callback20_or21_preserves_same_value_and_does_not_consume_capture() {
    for outcome in [
        Value::Undefined,
        Value::Null,
        Value::Number(-0.0),
        Value::Number(f64::NAN),
    ] {
        for reject in [false, true] {
            let total = if reject { 21 } else { 20 };
            let required = if reject { 512 } else { 0 };
            for (work, bytes, success) in [(total, required, true), (total - 1, required, false)] {
                let (mut runtime, _, _) = fixture(false);
                let callback = runtime
                    .new_finally_thunk(
                        &outcome,
                        if reject {
                            ReactionKind::Reject
                        } else {
                            ReactionKind::Fulfill
                        },
                    )
                    .unwrap();
                let Value::Native(native) = callback else {
                    panic!("native");
                };
                let objects = runtime.objects.len();
                let mut document = Document::parse("");
                reset(&mut runtime, work, bytes);
                let result =
                    runtime.promise_callback(&native, &[Value::Number(99.0)], &mut document);
                assert_eq!(runtime.steps, 0);
                assert_eq!(runtime.objects.len(), objects);
                if success {
                    assert_eq!(runtime.allocated, MAX_HEAP);
                    if reject {
                        let error = result.unwrap_err();
                        assert!(!error.is_resource_limit());
                        let ErrorKind::Thrown(saved) = error.kind else {
                            panic!("exact throw");
                        };
                        same(&saved, &outcome);
                    } else {
                        same(&result.unwrap(), &outcome);
                    }
                    assert!(!runtime.microtasks_suspended());
                } else {
                    assert!(result.unwrap_err().is_resource_limit());
                    assert!(runtime.microtasks_suspended());
                }
                let FinallyCapture::Outcome(saved) = &runtime.promises.finally_captures.records[0]
                else {
                    panic!("retained outcome");
                };
                same(saved, &outcome);
                assert_eq!(runtime.promises.callbacks.records.len(), 1);
                quiet(&runtime);
            }
        }
    }
    let (mut runtime, _, _) = fixture(false);
    let callback = runtime
        .new_finally_thunk(&Value::Number(-0.0), ReactionKind::Reject)
        .unwrap();
    let Value::Native(native) = callback else {
        panic!("native");
    };
    let mut document = Document::parse("");
    reset(&mut runtime, 21, 511);
    assert!(
        runtime
            .promise_callback(&native, &[], &mut document)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP + 1));
    assert!(runtime.microtasks_suspended());
    let FinallyCapture::Outcome(saved) = &runtime.promises.finally_captures.records[0] else {
        panic!("retained outcome");
    };
    same(saved, &Value::Number(-0.0));
    quiet(&runtime);
}

#[test]
fn finally_append_only_ids_keep_old_pair_and_outcomes_separate() {
    let (mut runtime, constructor, callback) = fixture(false);
    runtime.objects.try_reserve_exact(6).unwrap();
    let old = runtime.objects.len();
    let (a, b) = runtime
        .new_finally_functions(&constructor, &callback)
        .unwrap();
    let c = runtime
        .new_finally_thunk(&Value::Number(-0.0), ReactionKind::Fulfill)
        .unwrap();
    let (d, e) = runtime
        .new_finally_functions(&constructor, &callback)
        .unwrap();
    let f = runtime
        .new_finally_thunk(&Value::Number(f64::NAN), ReactionKind::Reject)
        .unwrap();
    let values = [a, b, c, d, e, f];
    for (index, value) in values.iter().enumerate() {
        assert_eq!(
            metadata(
                &runtime,
                value,
                if index == 2 || index == 5 { 0 } else { 1 }
            ),
            old + index
        );
        for earlier in &values[..index] {
            assert_ne!(value, earlier);
        }
    }
    assert_eq!(
        (
            runtime.promises.finally_captures.records.len(),
            runtime.promises.callbacks.records.len()
        ),
        (4, 6)
    );
    assert!(matches!(
        runtime.promises.callbacks.records[0].kind,
        CallbackKind::ThenFinally(FinallyCaptureId(0))
    ));
    assert!(matches!(
        runtime.promises.callbacks.records[1].kind,
        CallbackKind::CatchFinally(FinallyCaptureId(0))
    ));
    assert!(matches!(
        runtime.promises.callbacks.records[2].kind,
        CallbackKind::ReturnValue(FinallyCaptureId(1))
    ));
    assert!(matches!(
        runtime.promises.callbacks.records[3].kind,
        CallbackKind::ThenFinally(FinallyCaptureId(2))
    ));
    assert!(matches!(
        runtime.promises.callbacks.records[4].kind,
        CallbackKind::CatchFinally(FinallyCaptureId(2))
    ));
    assert!(matches!(
        runtime.promises.callbacks.records[5].kind,
        CallbackKind::ThrowReason(FinallyCaptureId(3))
    ));
    for _ in 0..2 {
        let (saved_constructor, saved_callback) =
            runtime.read_finally_finalizer(FinallyCaptureId(0)).unwrap();
        assert_eq!(saved_constructor, constructor);
        assert_eq!(saved_callback, callback);
        same(
            &runtime.read_finally_outcome(FinallyCaptureId(1)).unwrap(),
            &Value::Number(-0.0),
        );
        same(
            &runtime.read_finally_outcome(FinallyCaptureId(3)).unwrap(),
            &Value::Number(f64::NAN),
        );
    }
    quiet(&runtime);
}
