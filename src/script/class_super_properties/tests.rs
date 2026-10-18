//! Independent helper admissions. Published setup precedes each selected cut.
use super::*;

fn error<T>(result: Result<T>) -> ScriptError {
    match result {
        Err(error) => error,
        Ok(_) => panic!("expected refusal"),
    }
}

fn resource<T>(result: Result<T>) {
    let error = error(result);
    assert!(error.is_resource_limit(), "{error:?}");
}

fn code(
    strict: bool,
    arrow: bool,
    constructable: bool,
    class_kind: Option<code::ClassKind>,
) -> code::FunctionRef {
    let mut unit = code::Unit::empty(strict);
    let function = code::Function {
        params: Vec::new(),
        body: Vec::new(),
        name: None,
        strict,
        arrow,
        self_name: false,
        constructable,
        class_kind,
    };
    let mut budget = regexp::Budget {
        steps: MAX_STEPS,
        allocated: 0,
        heap_limit: MAX_HEAP,
        stack_limit: 16,
    };
    let id = unit.add_function(function, &mut budget).unwrap();
    let unit = unit.finish(&mut budget).unwrap();
    code::FunctionRef::new(&unit, id)
}

fn method_code() -> code::FunctionRef {
    code(true, false, false, None)
}

fn fixture() -> (Runtime, usize, usize, usize, Value) {
    let mut runtime = Runtime::uninitialized();
    let Value::Object(home) = runtime.object_ordered([]).unwrap() else {
        panic!("home");
    };
    let Value::Function(function) = runtime.function_value(&method_code(), 1).unwrap() else {
        panic!("method");
    };
    let receiver = runtime.object_ordered([]).unwrap();
    let owner = runtime.class_environment(1, None).unwrap();
    runtime.functions[function].home_object = Some(home);
    runtime.environments[owner].function_scope = true;
    runtime.environments[owner].active_function = Some(function);
    runtime.environments[owner].this_binding = ThisBinding::Initialized(receiver.clone());
    (runtime, home, function, owner, receiver)
}

fn cut(runtime: &mut Runtime, work: usize) {
    runtime.steps = work;
    runtime.allocated = MAX_HEAP;
}

fn shape(runtime: &Runtime) -> (usize, usize, usize, usize, usize, usize) {
    (
        runtime.objects.len(),
        runtime.objects.capacity(),
        runtime.functions.len(),
        runtime.functions.capacity(),
        runtime.environments.len(),
        runtime.environments.capacity(),
    )
}

fn clean(runtime: &Runtime, before: (usize, usize, usize, usize, usize, usize)) {
    assert_eq!(shape(runtime), before);
    assert_eq!(runtime.allocated, MAX_HEAP);
    assert_eq!(
        (
            runtime.frames.len(),
            runtime.calls,
            runtime.eval_depth,
            runtime.stack_units
        ),
        (0, 0, 0, 0)
    );
    assert_eq!(runtime.microtasks.author_depth, 0);
    assert!(!runtime.microtasks.performing_checkpoint);
    // Direct helper refusal is not a public terminal observation. No checkpoint
    // or public execution is invoked here, and suspension must stay unchanged.
    assert!(!runtime.microtasks_suspended());
}

#[test]
fn super_property_private_complete_layouts_preserve_top_frame_and_environment() {
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    {
        use std::mem::size_of;
        assert_eq!(size_of::<Function>(), 120);
        assert_eq!(size_of::<Reference>(), 80);
        assert_eq!(size_of::<machine::Frame>(), 184);
        assert_eq!(size_of::<Environment>(), 112);
        assert_eq!(size_of::<ThisBinding>(), 24);
        assert_eq!(ENVIRONMENT_CLASS_BYTES, 16);
    }
}

#[test]
fn super_property_private_attach_home_has_one_admission_and_single_publication() {
    for work in [7, 8] {
        let (mut runtime, home, function, _, _) = fixture();
        runtime.functions[function].home_object = None;
        let before = shape(&runtime);
        let properties = runtime.functions[function].properties;
        let environment = runtime.functions[function].environment;
        let value = Value::Function(function);
        cut(&mut runtime, work);
        let result = runtime.class_attach_home(&value, home);
        if work == 7 {
            resource(result);
        } else {
            result.unwrap();
        }
        assert_eq!(
            runtime.functions[function].home_object,
            (work == 8).then_some(home)
        );
        assert_eq!(runtime.functions[function].properties, properties);
        assert_eq!(runtime.functions[function].environment, environment);
        assert_eq!(runtime.steps, 0);
        clean(&runtime, before);
    }
    let (mut runtime, home, function, _, _) = fixture();
    let Value::Object(other) = runtime.object_ordered([]).unwrap() else {
        panic!("other");
    };
    let before = shape(&runtime);
    cut(&mut runtime, 8);
    resource(runtime.class_attach_home(&Value::Function(function), other));
    assert_eq!(runtime.functions[function].home_object, Some(home));
    assert_eq!(runtime.steps, 0);
    clean(&runtime, before);
}

#[test]
fn super_property_private_attach_rejects_ineligible_code_and_invalid_ids_without_mutation() {
    for fault in 0..9 {
        let (mut runtime, home, function, _, _) = fixture();
        runtime.functions[function].home_object = None;
        let mut value = Value::Function(function);
        let mut chosen_home = home;
        match fault {
            0 => value = Value::Undefined,
            1 => value = Value::Function(usize::MAX),
            2 => chosen_home = usize::MAX,
            3 => runtime.functions[function].code = code(false, false, false, None),
            4 => runtime.functions[function].code = code(true, true, false, None),
            5 => runtime.functions[function].code = code(true, false, true, None),
            6 => {
                runtime.functions[function].code =
                    code(true, false, true, Some(code::ClassKind::ExplicitBase))
            }
            7 => {
                runtime.functions[function].bound = Some(BoundFunction {
                    target: Value::Undefined,
                    receiver: Value::Undefined,
                    arguments: Vec::new(),
                })
            }
            8 => value = Value::Object(home),
            _ => unreachable!(),
        }
        let before = shape(&runtime);
        cut(&mut runtime, 8);
        resource(runtime.class_attach_home(&value, chosen_home));
        assert_eq!(runtime.functions[function].home_object, None);
        assert_eq!(runtime.steps, 0);
        clean(&runtime, before);
    }
}

#[test]
fn super_property_private_owner_pays_each_lexical_hop_and_retains_nearest_receiver() {
    for work in [7, 8, 9, 10, 11] {
        let (mut runtime, home, function, owner, receiver) = fixture();
        let first = runtime.class_environment(owner, None).unwrap();
        let second = runtime.class_environment(first, None).unwrap();
        // Lexical rows must not become owners even if malformed stale fields
        // are deliberately injected into their otherwise-unused owner slots.
        runtime.environments[first].active_function = Some(usize::MAX);
        runtime.environments[second].active_function = None;
        let before = shape(&runtime);
        cut(&mut runtime, work);
        let result = runtime.class_super_property_owner(second);
        if work == 11 {
            assert_eq!(result.unwrap(), (owner, receiver.clone()));
        } else {
            resource(result);
        }
        assert_eq!(runtime.functions[function].home_object, Some(home));
        assert!(
            matches!(&runtime.environments[owner].this_binding, ThisBinding::Initialized(actual) if actual == &receiver)
        );
        assert_eq!(runtime.steps, 0);
        clean(&runtime, before);
    }
    for receiver in [
        Value::Undefined,
        Value::Null,
        Value::Number(7.0),
        Value::String("receiver".into()),
    ] {
        let (mut runtime, _, _, owner, _) = fixture();
        runtime.environments[owner].this_binding = ThisBinding::Initialized(receiver.clone());
        let before = shape(&runtime);
        cut(&mut runtime, 9);
        let (actual_owner, actual_receiver) = runtime.class_super_property_owner(owner).unwrap();
        assert_eq!(actual_owner, owner);
        assert_eq!(actual_receiver, receiver);
        if let (Value::String(before), Value::String(after)) = (&receiver, &actual_receiver) {
            assert_eq!(before.units().as_ptr(), after.units().as_ptr());
        }
        assert_eq!(runtime.steps, 0);
        clean(&runtime, before);
    }
}

#[test]
fn super_property_private_owner_initialization_precedence_and_bad_state_are_bounded() {
    for fault in 0..10 {
        let (mut runtime, _, function, owner, _) = fixture();
        let mut start = owner;
        match fault {
            0 => {
                runtime.environments[owner].this_binding = ThisBinding::Uninitialized;
                runtime.environments[owner].active_function = Some(usize::MAX);
            }
            1 => runtime.environments[owner].active_function = None,
            2 => runtime.environments[owner].active_function = Some(usize::MAX),
            3 => runtime.functions[function].home_object = None,
            4 => runtime.functions[function].home_object = Some(usize::MAX),
            5 => runtime.functions[function].code = code(true, true, false, None),
            6 => {
                runtime.functions[function].bound = Some(BoundFunction {
                    target: Value::Undefined,
                    receiver: Value::Undefined,
                    arguments: Vec::new(),
                })
            }
            7 => start = usize::MAX,
            8 => {
                runtime.environments[owner].this_binding = ThisBinding::Lexical;
                runtime.environments[owner].parent = None;
            }
            9 => {
                runtime.environments[owner].this_binding = ThisBinding::Lexical;
                runtime.environments[owner].parent = Some(owner);
            }
            _ => unreachable!(),
        }
        let before = shape(&runtime);
        cut(&mut runtime, 9);
        let refusal = error(runtime.class_super_property_owner(start));
        if fault == 0 {
            assert_eq!(refusal.intrinsic_error_name(), Some("ReferenceError"));
            assert!(!refusal.is_resource_limit() && !refusal.is_unsupported());
            assert!(matches!(
                runtime.environments[owner].this_binding,
                ThisBinding::Uninitialized
            ));
        } else {
            assert!(refusal.is_resource_limit(), "fault={fault}: {refusal:?}");
        }
        assert_eq!(runtime.steps, 0);
        clean(&runtime, before);
    }
    // The nearest real this-owner with no home must fail; a valid outer home
    // cannot be borrowed through an ordinary nested function boundary.
    let (mut runtime, _, _, owner, _) = fixture();
    let inner = runtime.class_environment(owner, None).unwrap();
    runtime.environments[inner].this_binding = ThisBinding::Initialized(Value::Null);
    runtime.environments[inner].active_function = None;
    let before = shape(&runtime);
    cut(&mut runtime, 9);
    resource(runtime.class_super_property_owner(inner));
    assert_eq!(runtime.steps, 0);
    clean(&runtime, before);
}

#[test]
fn super_property_private_completion_keeps_raw_key_receiver_and_live_captured_base() {
    for strict in [false, true] {
        for work in [7, 8] {
            for key_kind in 0..3 {
                let (mut runtime, home, _, owner, receiver) = fixture();
                let first = runtime.object_ordered([]).unwrap();
                let second = runtime.object_ordered([]).unwrap();
                let key = match key_kind {
                    0 => Value::String(vec![0x96ea, 0xd800].into()),
                    1 => runtime.object_ordered([]).unwrap(),
                    2 => Value::Symbol(symbols::Symbol::unique(Some("raw".into()))),
                    _ => unreachable!(),
                };
                // Owner capture precedes key effects; the live home prototype
                // is deliberately changed between the two selected helpers.
                runtime.objects[home].prototype = Some(first);
                let (captured_owner, captured_receiver) =
                    runtime.class_super_property_owner(owner).unwrap();
                runtime.objects[home].prototype = Some(second.clone());
                let raw_key = key.clone();
                let before = shape(&runtime);
                cut(&mut runtime, work);
                let result = runtime.class_super_property_reference(
                    captured_owner,
                    captured_receiver,
                    key,
                    strict,
                );
                if work == 7 {
                    resource(result);
                } else {
                    let Reference::SuperProperty {
                        base,
                        name,
                        receiver: actual_receiver,
                        strict: actual_strict,
                    } = result.unwrap()
                    else {
                        panic!("super reference");
                    };
                    assert_eq!(base, second);
                    assert_eq!(name, raw_key);
                    assert_eq!(actual_receiver, receiver);
                    assert_eq!(actual_strict, strict);
                    if let (Value::String(before), Value::String(after)) = (&raw_key, &name) {
                        assert_eq!(before.units().as_ptr(), after.units().as_ptr());
                    }
                    runtime.objects[home].prototype = None;
                    assert_eq!(base, second); // Capture cannot be retargeted after creation.
                }
                assert_eq!(runtime.steps, 0);
                clean(&runtime, before);
            }
        }
    }
    let (mut runtime, home, _, owner, _) = fixture();
    runtime.objects[home].prototype = None;
    let before = shape(&runtime);
    cut(&mut runtime, 8);
    let result = runtime
        .class_super_property_reference(owner, Value::Null, Value::Number(9.0), true)
        .unwrap();
    assert!(matches!(
        result,
        Reference::SuperProperty {
            base: Value::Null,
            name: Value::Number(9.0),
            receiver: Value::Null,
            strict: true
        }
    ));
    assert_eq!(runtime.steps, 0);
    clean(&runtime, before);
}

#[test]
fn super_property_private_completion_revalidates_owner_home_and_parent_before_publication() {
    for fault in 0..9 {
        let (mut runtime, home, function, owner, receiver) = fixture();
        let mut captured_owner = owner;
        match fault {
            0 => captured_owner = usize::MAX,
            1 => runtime.environments[owner].active_function = None,
            2 => runtime.environments[owner].active_function = Some(usize::MAX),
            3 => runtime.functions[function].home_object = None,
            4 => runtime.functions[function].home_object = Some(usize::MAX),
            5 => runtime.functions[function].code = code(true, true, false, None),
            6 => {
                runtime.functions[function].bound = Some(BoundFunction {
                    target: Value::Undefined,
                    receiver: Value::Undefined,
                    arguments: Vec::new(),
                })
            }
            7 => runtime.objects[home].prototype = Some(Value::Bool(false)),
            8 => runtime.objects[home].prototype = Some(Value::Object(usize::MAX)),
            _ => unreachable!(),
        }
        let old_home = runtime.functions[function].home_object;
        let old_parent = runtime.objects[home].prototype.clone();
        let before = shape(&runtime);
        cut(&mut runtime, 8);
        resource(runtime.class_super_property_reference(
            captured_owner,
            receiver,
            Value::Undefined,
            true,
        ));
        assert_eq!(runtime.functions[function].home_object, old_home);
        assert_eq!(runtime.objects[home].prototype, old_parent);
        assert_eq!(runtime.steps, 0);
        clean(&runtime, before);
    }
}

// Additive completion-kind controls. All fixtures and snapshots are prepared
// outside the measured helper call; the original seven groups stay unchanged.
fn completion_storage(runtime: &Runtime) -> Vec<String> {
    let mut snapshot = vec![format!(
        "arrays={:?}; properties={:?}; holes={:?}; lengths={:?}; capacities={:?}",
        runtime.arrays,
        runtime.array_properties,
        runtime.array_holes,
        runtime
            .array_lengths
            .iter()
            .map(|length| (length.value, length.writable))
            .collect::<Vec<_>>(),
        (
            runtime.arrays.capacity(),
            runtime.array_properties.capacity(),
            runtime.array_holes.capacity(),
            runtime.array_lengths.capacity(),
            runtime.arrays.iter().map(Vec::capacity).collect::<Vec<_>>()
        ),
    )];
    snapshot.push(format!(
        "native={:?}; prototypes={:?}",
        runtime.native_properties, runtime.prototypes
    ));
    for object in &runtime.objects {
        snapshot.push(format!(
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
        snapshot.push(format!(
            "function={:?}|{:?}|{}|{}|{:?}|{:?}",
            std::rc::Rc::as_ptr(&function.code.unit),
            function.code.id(),
            function.environment,
            function.properties,
            function.home_object,
            function
                .bound
                .as_ref()
                .map(|bound| (&bound.target, &bound.receiver, &bound.arguments)),
        ));
    }
    for environment in &runtime.environments {
        let this = match &environment.this_binding {
            ThisBinding::Lexical => "lexical".to_owned(),
            ThisBinding::Uninitialized => "uninitialized".to_owned(),
            ThisBinding::Initialized(value) => format!("initialized:{value:?}"),
        };
        snapshot.push(format!(
            "environment={this}|{:?}|{:?}|{:?}|{}|{}",
            environment.active_function,
            environment.new_target_binding,
            environment.parent,
            environment.function_scope,
            environment.strict,
        ));
        for (name, binding) in &environment.bindings {
            snapshot.push(format!(
                "binding={name:?}|{:?}|{:?}|{}|{}|{}|{}|{}|{}|{}",
                binding.value,
                binding
                    .accessor
                    .as_ref()
                    .map(|accessor| (&accessor.get, &accessor.set)),
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
    snapshot
}

fn completion_base_fixture() -> (Runtime, usize, usize, Value, usize, usize, usize) {
    let (mut runtime, home, _, owner, receiver) = fixture();
    let Value::Function(parent_function) = runtime.function_value(&method_code(), 1).unwrap()
    else {
        panic!("parent function");
    };
    let Value::Array(parent_array) = runtime.array(vec![Value::Number(17.0)]).unwrap() else {
        panic!("parent array");
    };
    let Value::Object(native_bag) = runtime
        .object_ordered([("sentinel".into(), Value::Number(23.0))])
        .unwrap()
    else {
        panic!("native bag");
    };
    // An explicit Native bag cannot use zero, which is the legacy sentinel.
    assert_ne!(native_bag, 0);
    runtime
        .native_properties
        .insert("capture.legacy".into(), native_bag);
    (
        runtime,
        home,
        owner,
        receiver,
        parent_function,
        parent_array,
        native_bag,
    )
}

fn completion_native(properties: Option<std::num::NonZeroUsize>) -> Value {
    Value::Native(std::rc::Rc::new(Native {
        name: "capture.legacy".into(),
        receiver: Value::Window,
        properties,
    }))
}

#[test]
fn super_property_private_completion_rejects_malformed_base_ids_and_bags() {
    // Function ID/bag, Array ID/missing-or-invalid bag, and explicit Native bag.
    // Both first-out-of-range and maximal IDs must refuse without indexing.
    for fault in 0..11 {
        for work in [7, 8] {
            let (mut runtime, home, owner, receiver, function, array, _) =
                completion_base_fixture();
            let base = match fault {
                0 => Value::Function(runtime.functions.len()),
                1 => Value::Function(usize::MAX),
                2 => {
                    runtime.functions[function].properties = runtime.objects.len();
                    Value::Function(function)
                }
                3 => {
                    runtime.functions[function].properties = usize::MAX;
                    Value::Function(function)
                }
                4 => Value::Array(runtime.arrays.len()),
                5 => Value::Array(usize::MAX),
                6 => {
                    runtime.array_properties.truncate(array);
                    Value::Array(array)
                }
                7 => {
                    runtime.array_properties[array] = runtime.objects.len();
                    Value::Array(array)
                }
                8 => {
                    runtime.array_properties[array] = usize::MAX;
                    Value::Array(array)
                }
                9 => completion_native(std::num::NonZeroUsize::new(runtime.objects.len())),
                10 => completion_native(std::num::NonZeroUsize::new(usize::MAX)),
                _ => unreachable!(),
            };
            runtime.objects[home].prototype = Some(base.clone());
            let key = Value::String(vec![0x96ea, 0xd800].into());
            let before = shape(&runtime);
            let storage = completion_storage(&runtime);
            cut(&mut runtime, work);
            let refusal = error(runtime.class_super_property_reference(owner, receiver, key, true));
            assert!(
                refusal.is_resource_limit(),
                "fault={fault} work={work}: {refusal:?}"
            );
            assert_eq!(
                refusal.message,
                if work == 7 {
                    "script instruction limit exceeded"
                } else {
                    "invalid super property runtime state"
                }
            );
            assert_eq!(runtime.steps, 0);
            assert_eq!(
                completion_storage(&runtime),
                storage,
                "fault={fault} work={work}"
            );
            assert_eq!(runtime.objects[home].prototype.as_ref(), Some(&base));
            if let Value::Native(expected) = &base {
                let Some(Value::Native(actual)) = &runtime.objects[home].prototype else {
                    panic!("native base");
                };
                assert!(std::rc::Rc::ptr_eq(expected, actual));
            }
            clean(&runtime, before);
        }
    }
}

#[test]
fn super_property_private_completion_accepts_valid_function_array_and_native_bases() {
    for kind in 0..4 {
        for work in [7, 8] {
            let (mut runtime, home, owner, receiver, function, array, native_bag) =
                completion_base_fixture();
            let base = match kind {
                0 => Value::Function(function),
                1 => Value::Array(array),
                2 => completion_native(std::num::NonZeroUsize::new(native_bag)),
                3 => completion_native(None),
                _ => unreachable!(),
            };
            runtime.objects[home].prototype = Some(base.clone());
            let key = Value::String(vec![0x96ea, 0xd800].into());
            let raw_key = key.clone();
            let actual_receiver = receiver.clone();
            let strict = kind % 2 == 0;
            let before = shape(&runtime);
            let storage = completion_storage(&runtime);
            cut(&mut runtime, work);
            let result = runtime.class_super_property_reference(owner, receiver, key, strict);
            if work == 7 {
                let refusal = error(result);
                assert!(refusal.is_resource_limit());
                assert_eq!(refusal.message, "script instruction limit exceeded");
            } else {
                let Reference::SuperProperty {
                    base: captured,
                    name,
                    receiver,
                    strict: captured_strict,
                } = result.unwrap()
                else {
                    panic!("super reference");
                };
                assert_eq!(captured, base);
                assert_eq!(name, raw_key);
                assert_eq!(receiver, actual_receiver);
                assert_eq!(captured_strict, strict);
                let (Value::String(before), Value::String(after)) = (&raw_key, &name) else {
                    panic!("raw key");
                };
                assert_eq!(before.units().as_ptr(), after.units().as_ptr());
                if let (Value::Native(expected), Value::Native(actual)) = (&base, &captured) {
                    assert!(std::rc::Rc::ptr_eq(expected, actual));
                    assert_eq!(
                        actual.properties,
                        if kind == 2 {
                            std::num::NonZeroUsize::new(native_bag)
                        } else {
                            None
                        }
                    );
                }
            }
            assert_eq!(runtime.steps, 0);
            assert_eq!(
                completion_storage(&runtime),
                storage,
                "kind={kind} work={work}"
            );
            clean(&runtime, before);
        }
    }
}
