//! Independent object-home admissions; all setup precedes the measured call.
use super::*;

fn method_code(
    strict: bool,
    arrow: bool,
    constructable: bool,
    class_kind: Option<code::ClassKind>,
) -> code::FunctionRef {
    let mut unit = code::Unit::empty(strict);
    let mut budget = regexp::Budget {
        steps: MAX_STEPS,
        allocated: 0,
        heap_limit: MAX_HEAP,
        stack_limit: 16,
    };
    let id = unit
        .add_function(
            code::Function {
                params: Vec::new(),
                body: Vec::new(),
                name: None,
                strict,
                arrow,
                self_name: false,
                constructable,
                class_kind,
            },
            &mut budget,
        )
        .unwrap();
    let unit = unit.finish(&mut budget).unwrap();
    code::FunctionRef::new(&unit, id)
}

fn fixture(strict: bool) -> (Runtime, usize, usize, usize, usize) {
    let mut runtime = Runtime::uninitialized();
    let Value::Object(home) = runtime
        .object_ordered([("marker".into(), Value::String(vec![0x96ea, 0xd800].into()))])
        .unwrap()
    else {
        panic!("home");
    };
    let Value::Object(other) = runtime
        .object_ordered([("other".into(), Value::Number(23.0))])
        .unwrap()
    else {
        panic!("other home");
    };
    // A home is an identity, not a new definition on the home object.
    runtime.objects[home].prototype = Some(Value::Object(other));
    runtime.objects[home].non_extensible = true;
    let Value::Function(function) = runtime
        .function_value(&method_code(strict, false, false, None), 1)
        .unwrap()
    else {
        panic!("method");
    };
    let queued = runtime
        .function_value(&method_code(!strict, false, false, None), 1)
        .unwrap();
    let Value::Array(array) = runtime
        .array(vec![queued.clone(), Value::Number(17.0)])
        .unwrap()
    else {
        panic!("array sentinel");
    };
    runtime
        .queue_microtask(&Value::Window, std::slice::from_ref(&queued))
        .unwrap();
    runtime.console.push("untouched".into());
    (runtime, home, other, function, array)
}

#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    storage: Vec<String>,
    homes: Vec<Option<usize>>,
}

fn snapshot(runtime: &Runtime) -> Snapshot {
    let mut storage = vec![format!(
        "arenas={:?}; guards={:?}; allocated={}; queue={:?}; console={:?}",
        (
            runtime.objects.len(),
            runtime.objects.capacity(),
            runtime.functions.len(),
            runtime.functions.capacity(),
            runtime.environments.len(),
            runtime.environments.capacity(),
            runtime.arrays.len(),
            runtime.arrays.capacity(),
        ),
        (
            runtime.frames.len(),
            runtime.frames.capacity(),
            runtime.calls,
            runtime.eval_depth,
            runtime.json_depth,
            runtime.stack_units,
            runtime.microtasks.author_depth,
            runtime.microtasks.performing_checkpoint,
            runtime.microtasks.suspended,
        ),
        runtime.allocated,
        (
            runtime.microtasks.queue.capacity(),
            runtime.microtasks.admitted_capacity,
            runtime
                .microtasks
                .queue
                .iter()
                .map(|job| match job {
                    promises::QueueJob::Callback(value) => format!("callback:{value:?}"),
                    promises::QueueJob::Promise(id) => format!("promise:{id:?}"),
                })
                .collect::<Vec<_>>(),
        ),
        runtime.console,
    )];
    storage.push(format!(
        "arrays={:?}|{:?}|{:?}|{:?}|{:?}; native={:?}; prototypes={:?}",
        runtime.arrays,
        runtime.array_properties,
        runtime.array_holes,
        runtime
            .array_lengths
            .iter()
            .map(|row| (row.value, row.writable))
            .collect::<Vec<_>>(),
        (
            runtime.array_properties.capacity(),
            runtime.array_holes.capacity(),
            runtime.array_lengths.capacity(),
            runtime.arrays.iter().map(Vec::capacity).collect::<Vec<_>>(),
        ),
        runtime.native_properties,
        runtime.prototypes,
    ));
    for object in &runtime.objects {
        storage.push(format!(
            "object={:?}|{:?}|{}|{:?}|{:?}|{}|{:?}|{:?}|{}|{:?}|{:?}|{:?}|{}|{:?}|{:?}",
            object.values,
            object.order,
            object.order.capacity(),
            object.prototype,
            object.boxed,
            object.non_extensible,
            object.intrinsic_error,
            object.parameter_map,
            object.arguments,
            object.regexp.as_ref().map(std::rc::Rc::as_ptr),
            object.date_value,
            object.event,
            object.event_target,
            object.abort.map(|slot| match slot {
                AbortSlot::Controller(id) => (false, id),
                AbortSlot::Signal(id) => (true, id),
            }),
            object.namespace,
        ));
    }
    for function in &runtime.functions {
        storage.push(format!(
            "function={:?}|{}|{:?}|{}|{}|{:?}",
            std::rc::Rc::as_ptr(&function.code.unit),
            std::rc::Rc::strong_count(&function.code.unit),
            function.code.id(),
            function.environment,
            function.properties,
            function.bound.as_ref().map(|bound| (
                &bound.target,
                &bound.receiver,
                &bound.arguments,
                bound.arguments.capacity(),
            )),
        ));
    }
    for environment in &runtime.environments {
        let this = match &environment.this_binding {
            ThisBinding::Lexical => "lexical".to_owned(),
            ThisBinding::Uninitialized => "uninitialized".to_owned(),
            ThisBinding::Initialized(value) => format!("initialized:{value:?}"),
        };
        storage.push(format!(
            "environment={this}|{:?}|{:?}|{:?}|{}|{}",
            environment.active_function,
            environment.new_target_binding,
            environment.parent,
            environment.function_scope,
            environment.strict,
        ));
        for (name, binding) in &environment.bindings {
            storage.push(format!(
                "binding={name:?}|{:?}|{:?}|{}|{}|{}|{}|{}|{}|{}",
                binding.value,
                binding.accessor.as_ref().map(|row| (&row.get, &row.set)),
                binding.mutable,
                binding.initialized,
                binding.strict_immutable,
                binding.global_property,
                binding.global_order,
                binding.enumerable,
                binding.deletable,
            ));
        }
    }
    Snapshot {
        storage,
        homes: runtime
            .functions
            .iter()
            .map(|row| row.home_object)
            .collect(),
    }
}

fn measure(
    runtime: &mut Runtime,
    method: &Value,
    home: &Value,
    work: usize,
    publication: Option<(usize, usize)>,
) {
    runtime.steps = work;
    runtime.allocated = MAX_HEAP;
    let mut expected = snapshot(runtime);
    let result = runtime.object_attach_home(method, home);
    if let Some((function, home)) = publication {
        assert_eq!(work, 8);
        result.unwrap();
        expected.homes[function] = Some(home);
    } else {
        let error = result.unwrap_err();
        assert!(error.is_resource_limit(), "{error:?}");
        assert_eq!(
            error.message,
            if work == 7 {
                "script instruction limit exceeded"
            } else {
                "invalid object method home state"
            }
        );
    }
    assert_eq!(runtime.steps, 0);
    assert_eq!(snapshot(runtime), expected);
    assert_eq!(runtime.allocated, MAX_HEAP);
    assert_eq!(runtime.microtasks.queue.len(), 1);
    // Direct helper refusal is not an author-turn epilogue or a checkpoint.
    assert!(!runtime.microtasks_suspended());
}

#[test]
fn object_super_private_home_admits_strict_and_sloppy_at_exact_work_without_heap() {
    for strict in [false, true] {
        for choose_other in [false, true] {
            for work in [7, 8] {
                let (mut runtime, home, other, function, _) = fixture(strict);
                let chosen = if choose_other { other } else { home };
                assert_eq!(runtime.functions[function].home_object, None);
                measure(
                    &mut runtime,
                    &Value::Function(function),
                    &Value::Object(chosen),
                    work,
                    (work == 8).then_some((function, chosen)),
                );
            }
        }
    }
}

#[test]
fn object_super_private_home_rejects_wrong_kinds_and_out_of_range_ids_atomically() {
    for fault in 0..13 {
        for work in [7, 8] {
            let (mut runtime, home, _, function, array) = fixture(fault % 2 == 0);
            let mut method = Value::Function(function);
            let mut chosen = Value::Object(home);
            match fault {
                0 => method = Value::Undefined,
                1 => method = Value::Object(home),
                2 => method = Value::Array(array),
                3 => method = Runtime::native("Object", Value::Window),
                4 => method = Value::Function(runtime.functions.len()),
                5 => method = Value::Function(usize::MAX),
                6 => chosen = Value::Undefined,
                7 => chosen = Value::Null,
                8 => chosen = Value::Function(function),
                9 => chosen = Value::Array(array),
                10 => chosen = Runtime::native("Object", Value::Window),
                11 => chosen = Value::Object(runtime.objects.len()),
                12 => chosen = Value::Object(usize::MAX),
                _ => unreachable!(),
            }
            measure(&mut runtime, &method, &chosen, work, None);
        }
    }
}

#[test]
fn object_super_private_home_checks_each_eligibility_condition_before_publication() {
    for fault in 0..8 {
        for work in [7, 8] {
            let strict = fault % 2 == 0;
            let (mut runtime, home, _, function, _) = fixture(strict);
            match fault {
                0 => runtime.functions[function].code = method_code(strict, true, false, None),
                1 => runtime.functions[function].code = method_code(strict, false, true, None),
                2 => {
                    runtime.functions[function].bound = Some(BoundFunction {
                        target: Value::Function(function),
                        receiver: Value::Object(home),
                        arguments: vec![Value::String(vec![0x96ea, 0xd800].into())],
                    });
                }
                3..=6 => {
                    let kind = [
                        code::ClassKind::ExplicitBase,
                        code::ClassKind::ExplicitDerived,
                        code::ClassKind::DefaultBase,
                        code::ClassKind::DefaultDerived,
                    ][fault - 3];
                    // Keep constructable false to isolate the class-kind guard.
                    runtime.functions[function].code =
                        method_code(strict, false, false, Some(kind));
                }
                7 => runtime.functions[function].home_object = Some(usize::MAX),
                _ => unreachable!(),
            }
            measure(
                &mut runtime,
                &Value::Function(function),
                &Value::Object(home),
                work,
                None,
            );
        }
    }
}

#[test]
fn object_super_private_home_second_attachment_never_replaces_even_the_same_home() {
    for choose_other in [false, true] {
        for work in [7, 8] {
            let (mut runtime, home, other, function, _) = fixture(choose_other);
            let method = Value::Function(function);
            measure(
                &mut runtime,
                &method,
                &Value::Object(home),
                8,
                Some((function, home)),
            );
            let chosen = if choose_other { other } else { home };
            measure(&mut runtime, &method, &Value::Object(chosen), work, None);
            assert_eq!(runtime.functions[function].home_object, Some(home));
        }
    }
}
