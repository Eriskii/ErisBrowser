//! Source-derived race cuts. Setup finishes before each measured operation.
use super::*;
use std::{mem::size_of, rc::Rc};

fn error<T>(result: Result<T>) -> ScriptError {
    match result {
        Ok(_) => panic!("expected abrupt completion"),
        Err(error) => error,
    }
}
fn quiet(runtime: &Runtime) {
    assert!(runtime.frames.is_empty());
    assert!(runtime.console.is_empty());
    assert_eq!(
        (runtime.calls, runtime.eval_depth, runtime.stack_units),
        (0, 0, 0)
    );
    assert!(runtime.microtasks.queue.is_empty());
    assert_eq!(runtime.microtasks.queue.capacity(), 0);
    assert_eq!(runtime.microtasks.admitted_capacity, 0);
}
fn no_race_storage(runtime: &Runtime) {
    macro_rules! empty {
        ($field:ident) => {
            assert!(runtime.promises.$field.records.is_empty());
            assert_eq!(runtime.promises.$field.records.capacity(), 0);
            assert_eq!(runtime.promises.$field.admitted_capacity, 0);
        };
    }
    empty!(promises);
    empty!(gates);
    empty!(jobs);
    empty!(finally_captures);
    empty!(all_captures);
}
fn no_capability(runtime: &Runtime) {
    assert!(runtime.promises.captures.records.is_empty());
    assert!(runtime.promises.callbacks.records.is_empty());
}
fn native_refs(value: &Value) -> usize {
    let Value::Native(native) = value else {
        panic!("native fixture")
    };
    Rc::strong_count(native)
}

#[test]
fn promise_race_body17_and_constructor20_refuse_before_allocation_or_author_work() {
    for work in [16, 17, 18, 19, 20] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        let retained = Runtime::native("Object.is", Value::Undefined);
        // Inert caller-owned input: its copy exists before the measured entry.
        let input = retained.clone();
        assert_eq!(native_refs(&retained), 2);
        let objects = runtime.objects.len();
        runtime.steps = work;
        runtime.allocated = MAX_HEAP;
        let failure = error(runtime.promise_race(input, &[], &mut document));
        if work == 20 {
            assert!(matches!(failure.kind, ErrorKind::Runtime("TypeError")));
            assert_eq!(
                failure.message,
                "Promise capability target is not a constructor"
            );
        } else {
            assert!(failure.is_resource_limit());
        }
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, MAX_HEAP);
        assert_eq!(runtime.objects.len(), objects);
        assert_eq!(native_refs(&retained), 1);
        no_capability(&runtime);
        no_race_storage(&runtime);
        quiet(&runtime);
    }
}

#[test]
fn promise_race_public_wrapper8_observes_entry_refusal_without_new_storage() {
    for work in [7, 8, 24, 25, 26, 27, 28] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        let objects = runtime.objects.len();
        runtime.steps = work;
        runtime.allocated = MAX_HEAP;
        let failure = error(runtime.promise_native_operation(
            "Promise.race",
            Value::Bool(false),
            &[],
            &mut document,
        ));
        if work == 28 {
            assert!(matches!(failure.kind, ErrorKind::Runtime("TypeError")));
            assert!(!runtime.microtasks_suspended());
        } else {
            assert!(failure.is_resource_limit());
            assert!(runtime.microtasks_suspended());
        }
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, MAX_HEAP);
        assert_eq!(runtime.objects.len(), objects);
        no_capability(&runtime);
        no_race_storage(&runtime);
        quiet(&runtime);
    }
}

fn executor_bytes(cold: bool) -> usize {
    let leaf =
        16 * (size_of::<PropertyKey>() + size_of::<Property>()) + 32 * size_of::<usize>() + 64;
    let metadata = 904
        + size_of::<Option<AbortSlot>>()
        + size_of::<Option<f64>>()
        + 2 * size_of::<PropertyKey>()
        + leaf
        + size_of::<Native>();
    metadata
        + size_of::<ScriptObject>()
        + if cold {
            64 + 8 * size_of::<CapabilityCapture>() + 8 * size_of::<CallbackRecord>()
        } else {
            0
        }
}
fn constructor_fixture(cold: bool) -> (Runtime, Document, Value) {
    let mut runtime = Runtime::try_new().unwrap();
    let mut document = Document::parse("");
    let constructor = runtime
        .execute(
            "var entered=0; function C(executor) { entered++; } C",
            &mut document,
        )
        .unwrap();
    assert!(matches!(constructor, Value::Function(_)));
    no_capability(&runtime);
    no_race_storage(&runtime);
    // Inert physical preparation, matching the existing isolated factory cuts.
    // These admitted records have no values or author callbacks to execute.
    runtime.objects.try_reserve_exact(1).unwrap();
    assert!(runtime.objects.capacity() > runtime.objects.len());
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
    (runtime, document, constructor)
}
fn capability_prefix(runtime: &Runtime, published: bool, cold: bool, objects: usize) {
    assert_eq!(runtime.objects.len(), objects + 1);
    assert_eq!(
        runtime.promises.captures.records.len(),
        usize::from(published)
    );
    assert_eq!(
        runtime.promises.callbacks.records.len(),
        usize::from(published)
    );
    let admitted = if published || !cold { 8 } else { 0 };
    assert_eq!(runtime.promises.captures.admitted_capacity, admitted);
    assert_eq!(runtime.promises.callbacks.admitted_capacity, admitted);
    if published {
        assert_eq!(
            runtime.promises.captures.records[0].resolve,
            Value::Undefined
        );
        assert_eq!(
            runtime.promises.captures.records[0].reject,
            Value::Undefined
        );
        assert!(matches!(
            runtime.promises.callbacks.records[0].kind,
            CallbackKind::CapabilityExecutor(CaptureId(0))
        ));
    }
    assert_eq!(
        runtime.environments[0].bindings["entered"].value,
        Value::Number(0.0)
    );
    no_race_storage(runtime);
    quiet(runtime);
}

#[test]
fn promise_race_capability_then_argument_admission_keeps_author_constructor_unentered() {
    for cold in [false, true] {
        //20 pays body17, both retained-C copies and the IsConstructor tick.
        // The existing executor factory is126 warm or142 cold, including101 metadata.
        let executor_end = if cold { 162 } else { 146 };
        let bytes = executor_bytes(cold);
        let vector = 32 + size_of::<Value>();
        for (work, allowance, published, allocated) in [
            (
                executor_end - 1,
                bytes + vector,
                false,
                bytes - size_of::<Native>() - 48,
            ),
            (executor_end, bytes + vector, true, bytes),
            (executor_end + 9, bytes + vector, true, bytes),
            (executor_end + 10, bytes + vector, true, bytes + vector),
            (executor_end + 10, bytes + vector - 1, true, bytes + vector),
        ] {
            let (mut runtime, mut document, constructor) = constructor_fixture(cold);
            let objects = runtime.objects.len();
            let capacity = runtime.objects.capacity();
            runtime.steps = work;
            runtime.allocated = MAX_HEAP - allowance;
            let failure = error(runtime.promise_race(constructor, &[], &mut document));
            assert!(
                failure.is_resource_limit(),
                "cold={cold} work={work}: {failure:?}"
            );
            assert_eq!(runtime.steps, 0);
            assert_eq!(runtime.allocated, MAX_HEAP - allowance + allocated);
            assert_eq!(runtime.objects.capacity(), capacity);
            capability_prefix(&runtime, published, cold, objects);
        }
    }
}

#[test]
fn promise_race_generic_inputs_use_one_capability_and_no_aggregate_storage() {
    const SETUP: &str = r#"
var chosen={},entered=0,visits=0,thens=0,fulfilled,rejected;
function C(executor){entered++;executor(function(v){fulfilled=v;},function(v){rejected=v;});return chosen;}
C.resolve=function(v){visits++;return {then:function(f,r){thens++;f(v);r(v);}};};
C;
"#;
    for strict in [false, true] {
        for (input, count) in [("[]", 0), ("[1]", 1), ("[1,2,3]", 3)] {
            let mut runtime = Runtime::try_new().unwrap();
            let mut document = Document::parse("");
            let constructor = if strict {
                runtime.execute_strict(SETUP, &mut document)
            } else {
                runtime.execute(SETUP, &mut document)
            }
            .unwrap();
            let items = if strict {
                runtime.execute_strict(input, &mut document)
            } else {
                runtime.execute(input, &mut document)
            }
            .unwrap();
            let chosen = runtime.environments[0].bindings["chosen"].value.clone();
            no_capability(&runtime);
            no_race_storage(&runtime);
            let result = runtime
                .promise_race(constructor, &[items], &mut document)
                .unwrap();
            assert_eq!(result, chosen);
            assert_eq!(
                runtime.environments[0].bindings["entered"].value,
                Value::Number(1.0)
            );
            for name in ["visits", "thens"] {
                assert_eq!(
                    runtime.environments[0].bindings[name].value,
                    Value::Number(f64::from(count))
                );
            }
            let expected = if count == 0 {
                Value::Undefined
            } else {
                Value::Number(f64::from(count))
            };
            assert_eq!(
                runtime.environments[0].bindings["fulfilled"].value,
                expected
            );
            assert_eq!(runtime.environments[0].bindings["rejected"].value, expected);
            assert_eq!(runtime.promises.captures.records.len(), 1);
            assert_eq!(runtime.promises.callbacks.records.len(), 1);
            assert!(matches!(
                runtime.promises.callbacks.records[0].kind,
                CallbackKind::CapabilityExecutor(CaptureId(0))
            ));
            assert!(!runtime.microtasks_suspended());
            no_race_storage(&runtime);
            quiet(&runtime);
        }
    }
}
