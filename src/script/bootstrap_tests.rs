use super::*;

fn failed(result: Result<Runtime>) -> ScriptError {
    match result {
        Ok(_) => panic!("bootstrap unexpectedly succeeded"),
        Err(error) => {
            assert!(error.is_resource_limit());
            assert_eq!(error.name(), "ResourceLimit");
            assert_eq!(error.offset, None);
            assert_eq!(error.intrinsic_error_name(), None);
            error
        }
    }
}

fn idle(runtime: &Runtime) {
    assert_eq!(runtime.calls, 0);
    assert_eq!(runtime.eval_depth, 0);
    assert_eq!(runtime.json_depth, 0);
    assert_eq!(runtime.stack_units, 0);
    assert!(runtime.frames.is_empty());
    assert!(runtime.active_array_joins.is_empty());
    assert!(runtime.console.is_empty());
    assert!(!runtime.readiness_fired);
}

// Measure only the two unchanged preparation stages, without initializing any
// intrinsic. This locates real failure boundaries without a production hook.
fn preparation_debits() -> (usize, usize) {
    let mut runtime = Runtime::uninitialized();
    let initial = runtime.allocated;
    runtime.reserve_bootstrap_objects().unwrap();
    let arena = runtime.allocated - initial;
    let before_machine = runtime.allocated;
    machine::initialize(&mut runtime).unwrap();
    assert!(runtime.objects.is_empty());
    assert!(runtime.native_properties.is_empty());
    (arena, runtime.allocated - before_machine)
}

#[test]
fn bootstrap_initial_reserve_work_failure_is_returned_unchanged() {
    let mut witness = Runtime::uninitialized();
    witness.steps = 0;
    let allocated = witness.allocated;
    let error = witness.reserve_bootstrap_objects().unwrap_err();
    assert_eq!(witness.objects.capacity(), 0);
    assert_eq!(witness.frames.capacity(), 0);
    assert_eq!(witness.allocated, allocated);
    assert!(witness.native_properties.is_empty());

    let mut runtime = Runtime::uninitialized();
    runtime.steps = 0;
    assert_eq!(failed(runtime.finish_bootstrap()), error);
}

#[test]
fn bootstrap_initial_reserve_heap_failure_is_returned_unchanged() {
    let mut witness = Runtime::uninitialized();
    witness.allocated = MAX_HEAP;
    let error = witness.reserve_bootstrap_objects().unwrap_err();
    assert_eq!(witness.steps, MAX_STEPS - 1);
    assert_eq!(witness.objects.capacity(), 0);
    assert_eq!(witness.frames.capacity(), 0);
    assert!(witness.native_properties.is_empty());

    let mut runtime = Runtime::uninitialized();
    runtime.allocated = MAX_HEAP;
    assert_eq!(failed(runtime.finish_bootstrap()), error);
}

#[test]
fn bootstrap_machine_heap_failure_follows_successful_arena_reserve() {
    let (arena, machine) = preparation_debits();
    assert!(arena > 0 && machine > 0);
    let initial = MAX_HEAP - arena - machine + 1;
    let mut witness = Runtime::uninitialized();
    witness.allocated = initial;
    witness.reserve_bootstrap_objects().unwrap();
    assert!(witness.objects.capacity() >= 512);
    let error = machine::initialize(&mut witness).unwrap_err();
    assert_eq!(witness.frames.capacity(), 0);
    assert!(witness.prototypes.is_empty());
    assert!(witness.native_properties.is_empty());

    let mut runtime = Runtime::uninitialized();
    runtime.allocated = initial;
    assert_eq!(failed(runtime.finish_bootstrap()), error);
}

#[test]
fn bootstrap_first_intrinsic_heap_failure_follows_both_preparation_stages() {
    let (arena, machine) = preparation_debits();
    let initial = MAX_HEAP - arena - machine;
    let mut witness = Runtime::uninitialized();
    witness.allocated = initial;
    witness.reserve_bootstrap_objects().unwrap();
    machine::initialize(&mut witness).unwrap();
    assert_eq!(witness.allocated, MAX_HEAP);
    assert!(witness.frames.capacity() > 0);
    let error = witness.initialize_intrinsics().unwrap_err();
    assert!(witness.objects.is_empty());
    assert!(witness.prototypes.is_empty());
    assert!(witness.native_properties.is_empty());

    let mut runtime = Runtime::uninitialized();
    runtime.allocated = initial;
    assert_eq!(failed(runtime.finish_bootstrap()), error);
}

#[test]
fn bootstrap_intrinsic_work_failure_preserves_the_original_resource_error() {
    let mut witness = Runtime::uninitialized();
    witness.steps = 1;
    witness.reserve_bootstrap_objects().unwrap();
    machine::initialize(&mut witness).unwrap();
    let error = witness.initialize_intrinsics().unwrap_err();
    // Prototype construction preceded the first checked DOM batch work. The
    // partially initialized realm must never become the constructor's result.
    assert_eq!(witness.prototypes.len(), 25);
    assert_eq!(witness.functions.len(), 1);
    assert!(witness.native_properties.is_empty());
    idle(&witness);

    let mut runtime = Runtime::uninitialized();
    runtime.steps = 1;
    assert_eq!(failed(runtime.finish_bootstrap()), error);
}

#[test]
fn bootstrap_late_work_boundary_returns_error_or_the_complete_realm() {
    let complete = Runtime::uninitialized().finish_bootstrap().unwrap();
    let consumed = MAX_STEPS - complete.steps;
    let mut witness = Runtime::uninitialized();
    witness.steps = consumed - 1;
    witness.reserve_bootstrap_objects().unwrap();
    machine::initialize(&mut witness).unwrap();
    let error = witness.initialize_intrinsics().unwrap_err();
    assert!(witness.native_properties.len() > 8);
    assert!(witness.native_properties.contains_key("DataView"));
    assert_eq!(witness.steps, 0);
    idle(&witness);

    let mut runtime = Runtime::uninitialized();
    runtime.steps = consumed - 1;
    assert_eq!(failed(runtime.finish_bootstrap()), error);
    let mut runtime = Runtime::uninitialized();
    runtime.steps = consumed;
    let runtime = runtime.finish_bootstrap().unwrap();
    assert_eq!(runtime.steps, 0);
    assert_eq!(runtime.allocated, complete.allocated);
    assert_eq!(runtime.native_properties, complete.native_properties);
    assert_eq!(runtime.objects.len(), complete.objects.len());
    idle(&runtime);
}

#[test]
fn bootstrap_late_heap_boundary_returns_error_or_the_complete_realm() {
    let complete = Runtime::uninitialized().finish_bootstrap().unwrap();
    let consumed = complete.allocated - Runtime::uninitialized().allocated;
    let initial = MAX_HEAP - consumed;
    let mut witness = Runtime::uninitialized();
    witness.allocated = initial + 1;
    witness.reserve_bootstrap_objects().unwrap();
    machine::initialize(&mut witness).unwrap();
    let error = witness.initialize_intrinsics().unwrap_err();
    assert!(witness.native_properties.len() > 8);
    assert!(witness.native_properties.contains_key("DataView"));
    idle(&witness);

    let mut runtime = Runtime::uninitialized();
    runtime.allocated = initial + 1;
    assert_eq!(failed(runtime.finish_bootstrap()), error);
    let mut runtime = Runtime::uninitialized();
    runtime.allocated = initial;
    let runtime = runtime.finish_bootstrap().unwrap();
    assert_eq!(runtime.allocated, MAX_HEAP);
    assert_eq!(runtime.steps, complete.steps);
    assert_eq!(runtime.native_properties, complete.native_properties);
    assert_eq!(runtime.objects.len(), complete.objects.len());
    idle(&runtime);
}

#[test]
fn bootstrap_convenience_and_try_paths_separate_work_and_retain_heap() {
    let reference = Runtime::uninitialized().finish_bootstrap().unwrap();
    // Eager represented DOM interface metadata is included in this raw
    // initialization witness. No author-entry reset has happened.
    assert_eq!(reference.steps, 10_636);
    assert_eq!(reference.objects.len(), 681);
    assert_eq!(reference.native_properties.len(), 321);
    assert_eq!(reference.prototypes.len(), 25);
    for mut runtime in [
        Runtime::try_new().unwrap(),
        Runtime::new(),
        Runtime::default(),
        Runtime::try_with_date_host(DateHost::unconfigured()).unwrap(),
        Runtime::with_date_host(DateHost::unconfigured()),
    ] {
        assert_eq!(runtime.steps, MAX_STEPS);
        assert_eq!(runtime.allocated, reference.allocated);
        assert_eq!(runtime.objects.capacity(), reference.objects.capacity());
        assert_eq!(runtime.frames.capacity(), reference.frames.capacity());
        assert_eq!(runtime.native_properties, reference.native_properties);
        assert_eq!(runtime.prototypes, reference.prototypes);
        assert!(runtime.date_host.zone().is_err());
        assert_eq!(
            runtime
                .object_is(&[Value::Number(f64::NAN), Value::Number(f64::NAN)])
                .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(runtime.steps, MAX_STEPS - 4);
        assert_eq!(runtime.allocated, reference.allocated);
        idle(&runtime);
    }
}

#[test]
fn bootstrap_try_realms_have_intrinsics_and_isolated_mutable_metadata() {
    let mut first = Runtime::try_new().unwrap();
    let mut second = Runtime::try_new().unwrap();
    let mut doc = Document::parse("<p>kept</p>");
    let source = "var a={}, b={}; var bytes=new ArrayBuffer(2); \
        var view=new DataView(bytes); view.setUint16(0,4660); \
        Object.is(NaN,NaN) && !Object.is(0,-0) && Object.is(a,a) && \
        !Object.is(a,b) && view.getUint16(0)===4660 && ArrayBuffer.isView(view) && \
        document.querySelector===document.querySelector && \
        document.querySelector!==document.createElement('p').querySelector";
    assert_eq!(first.execute(source, &mut doc).unwrap(), Value::Bool(true));
    assert_eq!(second.execute(source, &mut doc).unwrap(), Value::Bool(true));
    assert_eq!(
        first
            .execute(
                "Object.defineProperty(Object.is,'length',{value:99}); \
                 Object.prototype.bootstrapMarker=7; Object.is.length",
                &mut doc,
            )
            .unwrap(),
        Value::Number(99.0)
    );
    assert_eq!(
        second
            .execute(
                "Object.is.length===2 && ({}).bootstrapMarker===undefined",
                &mut doc,
            )
            .unwrap(),
        Value::Bool(true)
    );
    idle(&first);
    idle(&second);
}

#[test]
fn bootstrap_try_date_host_is_installed_without_clock_reads() {
    use crate::date_host::{TimeZoneSnapshot, ZonePayload, ZonePayloadKind};
    use std::sync::Arc;

    let payload = ZonePayload::new(ZonePayloadKind::Posix2024, b"UTC0").unwrap();
    let zone = Arc::new(TimeZoneSnapshot::parse(&payload, &mut Default::default()).unwrap());
    let host = DateHost::sequence_for_test(Some(zone.clone()), Arc::from([0, 1_234_000_000]));
    let mut runtime = Runtime::try_with_date_host(host).unwrap();
    assert!(std::ptr::eq(
        runtime.date_host.zone().unwrap(),
        zone.as_ref()
    ));
    assert_eq!(runtime.steps, MAX_STEPS);
    let mut doc = Document::parse("");
    for (source, expected) in [
        ("Date.now()", Value::Number(0.0)),
        ("new Date().getTime()", Value::Number(1234.0)),
        ("new Date(0).getHours()", Value::Number(0.0)),
    ] {
        assert_eq!(runtime.execute(source, &mut doc).unwrap(), expected);
    }
    assert!(
        runtime
            .execute("Date.now()", &mut doc)
            .unwrap_err()
            .is_unsupported()
    );
    idle(&runtime);
}

#[test]
fn bootstrap_try_path_does_not_move_the_execute_budget_reset() {
    let mut runtime = Runtime::try_new().unwrap();
    let mut doc = Document::parse("");
    runtime.steps = 0;
    assert!(runtime.object_is(&[]).unwrap_err().is_resource_limit());
    assert_eq!(
        runtime.execute("Object.is(NaN,NaN)", &mut doc).unwrap(),
        Value::Bool(true)
    );
    assert!(runtime.steps > 0);
    runtime.steps = 123;
    runtime.allocated = MAX_HEAP;
    assert!(
        runtime
            .execute("1", &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    // execute_program still charges source/compiled storage before resetting.
    assert_eq!(runtime.steps, 123);
    idle(&runtime);
}

#[test]
fn bootstrap_first_public_script_entry_has_the_same_work_and_heap() {
    const SOURCE: &str = "var trace='';var value={valueOf:function(){trace+='v';return 7;}};\
        var holder={get x(){trace+='g';return value;}};\
        var result=+holder.x;console.log(trace);trace+':'+result;";
    for strict in [false, true] {
        let mut observation = None;
        for mut runtime in [
            Runtime::uninitialized().finish_bootstrap().unwrap(),
            Runtime::try_new().unwrap(),
        ] {
            let mut doc = Document::parse("");
            let result = if strict {
                runtime.execute_strict(SOURCE, &mut doc)
            } else {
                runtime.execute(SOURCE, &mut doc)
            }
            .unwrap();
            assert_eq!(result, Value::String("gv:7".into()));
            assert_eq!(runtime.console, ["gv"]);
            let actual = (
                result,
                runtime.steps,
                runtime.allocated,
                runtime.objects.len(),
                runtime.functions.len(),
            );
            if let Some(expected) = &observation {
                assert_eq!(&actual, expected);
            } else {
                observation = Some(actual);
            }
            assert_eq!(runtime.calls, 0);
            assert_eq!(runtime.stack_units, 0);
            assert!(runtime.frames.is_empty());
        }
    }
}

#[test]
fn bootstrap_callbacks_and_nested_dispatch_cannot_refresh_author_work() {
    let mut runtime = Runtime::try_new().unwrap();
    let mut doc = Document::parse("");
    runtime
        .execute(
            "var target=new EventTarget(),event=new Event('pulse');\
             var getters=0,conversions=0,listeners=0,caught=false,finished=false;\
             target.addEventListener('pulse',function(){listeners++;});\
             var argument={valueOf:function(){conversions++;target.dispatchEvent(event);return 1;}};\
             var holder={get value(){getters++;return argument;}};",
            &mut doc,
        )
        .unwrap();
    // Enter an already admitted author computation with a smaller remaining
    // allowance. Getters, ToPrimitive and nested synthetic dispatch must all
    // consume it; none is a new host entry.
    let unit = parser::Parser::program(
        "try{for(var i=0;i<200;i++){+holder.value;}}\
         catch(error){caught=true;}finally{finished=true;}",
    )
    .unwrap();
    runtime.steps = 2_000;
    let error = match machine::evaluate_statements(
        &mut runtime,
        &unit,
        machine::ListOwner::Program,
        1,
        &mut doc,
    ) {
        Ok(_) => panic!("callbacks refreshed the author work allowance"),
        Err(error) => error,
    };
    assert!(error.is_resource_limit());
    assert_eq!(error.message, "script instruction limit exceeded");
    assert_eq!(runtime.steps, 0);
    assert!(runtime.allocated < MAX_HEAP);
    for name in ["getters", "conversions", "listeners"] {
        let Value::Number(count) = runtime.lookup(0, name).unwrap().1 else {
            panic!("callback counter is not numeric");
        };
        assert!(count > 0.0 && count < 200.0, "{name}: {count}");
    }
    assert_eq!(runtime.lookup(0, "caught").unwrap().1, Value::Bool(false));
    assert_eq!(runtime.lookup(0, "finished").unwrap().1, Value::Bool(false));
    let event = runtime.lookup(0, "event").unwrap().1;
    let event = &runtime.events[runtime.event_index(&event).unwrap()];
    assert!(!event.dispatching);
    assert!(event.path.is_empty());
    assert_eq!(event.current_target, Value::Null);
    idle(&runtime);
}
