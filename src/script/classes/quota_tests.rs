//! Independently selected class admissions; setup is outside each measured cut.
use super::*;
use std::mem::size_of;

fn resource<T>(result: Result<T>) {
    match result {
        Err(error) => assert!(error.is_resource_limit(), "{error:?}"),
        Ok(_) => panic!("expected resource refusal"),
    }
}
fn cut(runtime: &mut Runtime, work: usize, heap: usize) {
    runtime.steps = work;
    runtime.allocated = MAX_HEAP - heap;
}
fn fixture_plan<T>(runtime: &mut Runtime, len: usize, capacity: usize) -> Option<usize> {
    let plan = promises::growth_plan::<T>(len, capacity, 1).unwrap();
    runtime.work(plan.work).unwrap();
    runtime.charge(plan.bytes).unwrap();
    plan.target
}
fn spare_environments(runtime: &mut Runtime) {
    let (len, cap) = (runtime.environments.len(), runtime.environments.capacity());
    if let Some(target) = fixture_plan::<Environment>(runtime, len, cap) {
        runtime
            .environments
            .try_reserve_exact(target - len)
            .unwrap();
    }
    assert!(runtime.environments.capacity() > len);
}
fn spare_objects(runtime: &mut Runtime) {
    let (len, cap) = (runtime.objects.len(), runtime.objects.capacity());
    if let Some(target) = fixture_plan::<ScriptObject>(runtime, len, cap) {
        runtime.objects.try_reserve_exact(target - len).unwrap();
    }
    assert!(runtime.objects.capacity() > len);
}
fn spare_functions(runtime: &mut Runtime) {
    let (len, cap) = (runtime.functions.len(), runtime.functions.capacity());
    if let Some(target) = fixture_plan::<Function>(runtime, len, cap) {
        runtime.functions.try_reserve_exact(target - len).unwrap();
    }
    assert!(runtime.functions.capacity() > len);
}
fn spare_order(runtime: &mut Runtime, id: usize) {
    let (len, cap) = (
        runtime.objects[id].order.len(),
        runtime.objects[id].order.capacity(),
    );
    if let Some(target) = fixture_plan::<PropertyKey>(runtime, len, cap) {
        runtime.objects[id]
            .order
            .try_reserve_exact(target - len)
            .unwrap();
    }
    assert!(runtime.objects[id].order.capacity() > len);
}
fn row_heap() -> usize {
    128 + 2 * size_of::<Option<Value>>()
        + size_of::<Option<usize>>()
        + size_of::<ThisBinding>().saturating_sub(size_of::<Option<Value>>())
}
fn bag_heap() -> usize {
    72 + size_of::<Option<AbortSlot>>() + size_of::<Option<f64>>()
}
fn code(kind: Option<code::ClassKind>) -> code::FunctionRef {
    let mut unit = code::Unit::empty(true);
    let function = code::Function {
        params: Vec::new(),
        body: Vec::new(),
        name: None,
        strict: true,
        arrow: false,
        self_name: false,
        constructable: kind.is_some(),
        class_kind: kind,
    };
    let mut ledger = regexp::Budget {
        steps: MAX_STEPS,
        allocated: 0,
        heap_limit: MAX_HEAP,
        stack_limit: 16,
    };
    let id = unit.add_function(function, &mut ledger).unwrap();
    let unit = unit.finish(&mut ledger).unwrap();
    code::FunctionRef::new(&unit, id)
}

#[test]
fn class_private_name_hint_refuses_before_retaining_and_preserves_key_identity() {
    let keys = [
        PropertyKey::String(JsString::from(vec![0x03bb, 0xd800])),
        PropertyKey::Symbol(symbols::Symbol::unique(Some("slot".into()))),
    ];
    for key in &keys {
        for work in [3, 4] {
            let mut runtime = Runtime::uninitialized();
            cut(&mut runtime, work, 0);
            let result = runtime.class_name_hint(key);
            if work == 3 {
                resource(result);
            } else {
                let result = result.unwrap();
                assert_eq!(&result, key);
                if let (PropertyKey::String(before), PropertyKey::String(after)) = (key, &result) {
                    assert_eq!(before.units().as_ptr(), after.units().as_ptr());
                }
            }
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        }
    }
}

#[test]
fn class_private_identifier_utf16_has_two_work_admissions_before_heap() {
    for (source, first, second, heap, units) in [
        ("", 8, 0, 64, &[][..]),
        ("A", 9, 2, 68, &[65u16][..]),
        ("Café", 13, 8, 80, &[67u16, 97, 102, 233][..]),
        ("𐐀", 12, 4, 72, &[0xd801u16, 0xdc00][..]),
    ] {
        let mut rows = vec![
            (first - 1, heap, false, false),
            (first + second, heap - 1, false, true),
            (first + second, heap, true, true),
        ];
        if second != 0 {
            rows.push((first + second - 1, heap, false, false));
        }
        for (work, available, ok, charged) in rows {
            let mut runtime = Runtime::uninitialized();
            let counts = (
                runtime.objects.len(),
                runtime.functions.len(),
                runtime.environments.len(),
            );
            cut(&mut runtime, work, available);
            let result = runtime.class_identifier_name_hint(source);
            if ok {
                assert_eq!(result.unwrap().as_string().unwrap().units(), units);
            } else {
                resource(result);
            }
            assert_eq!(runtime.steps, 0);
            assert_eq!(
                runtime.allocated,
                MAX_HEAP - available + if charged { heap } else { 0 }
            );
            assert_eq!(
                (
                    runtime.objects.len(),
                    runtime.functions.len(),
                    runtime.environments.len()
                ),
                counts
            );
        }
    }
}

#[test]
fn class_private_environment_growth_and_named_tdz_refusals_retain_only_paid_rows() {
    for cold in [false, true] {
        for named in [false, true] {
            let mut preparation = Runtime::uninitialized();
            if !cold {
                spare_environments(&mut preparation);
            }
            let len = preparation.environments.len();
            let capacity = preparation.environments.capacity();
            assert_eq!(len, 2);
            if cold {
                assert_eq!(capacity, len);
            }
            let growth_work = if cold {
                8 + 2 * len * size_of::<Environment>().div_ceil(size_of::<Value>())
            } else {
                0
            };
            let growth_heap = if cold {
                32 + 8 * size_of::<Environment>()
            } else {
                0
            };
            let name_work = if named { 1 } else { 0 };
            let name_heap = if named { BINDING_BYTES + 1 } else { 0 };
            let work = 8 + growth_work + name_work;
            let heap = growth_heap + row_heap() + name_heap;
            let mut rows = vec![
                (7, heap, 0, 0, false),
                (
                    work,
                    growth_heap + row_heap() - 1,
                    0,
                    growth_heap + row_heap(),
                    false,
                ),
                (work, heap, 1, heap, true),
            ];
            if cold {
                rows.push((8 + growth_work - 1, heap, 0, 0, false));
            }
            if named {
                rows.push((work - 1, heap, 1, growth_heap + row_heap(), false));
                rows.push((work, heap - 1, 1, heap, false));
            }
            for (available_work, available_heap, added, charged, ok) in rows {
                let mut runtime = Runtime::uninitialized();
                if !cold {
                    spare_environments(&mut runtime);
                }
                cut(&mut runtime, available_work, available_heap);
                let result = runtime.class_environment(1, named.then_some("x"));
                if ok {
                    assert_eq!(result.unwrap(), len);
                } else {
                    resource(result);
                }
                assert_eq!(runtime.environments.len(), len + added);
                assert_eq!(runtime.allocated, MAX_HEAP - available_heap + charged);
                assert_eq!(
                    runtime.steps,
                    if added == 0 && charged == growth_heap + row_heap() {
                        name_work
                    } else {
                        0
                    }
                );
                if added == 1 {
                    let env = &runtime.environments[len];
                    assert!(env.strict && !env.function_scope);
                    assert_eq!(env.parent, Some(1));
                    assert!(matches!(env.this_binding, ThisBinding::Lexical));
                    assert!(env.new_target_binding.is_none() && env.active_constructor.is_none());
                    assert_eq!(env.bindings.len(), usize::from(ok && named));
                    if ok && named {
                        let binding = &env.bindings["x"];
                        assert!(
                            !binding.mutable
                                && !binding.initialized
                                && binding.strict_immutable
                                && !binding.global_property
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn class_private_inner_copy_and_outer_move_publish_only_after_last_admission() {
    for inner in [false, true] {
        let work = if inner { 9 } else { 8 }; // fixed6 + one-entry lookup2 + inner copy1.
        for available in [work - 1, work] {
            let mut runtime = Runtime::uninitialized();
            let env = runtime.class_environment(1, inner.then_some("x")).unwrap();
            if !inner {
                runtime.define(env, "x", Value::Undefined, true).unwrap();
                runtime.environments[env]
                    .bindings
                    .get_mut("x")
                    .unwrap()
                    .initialized = false;
            }
            let constructor = runtime.object_ordered([]).unwrap();
            let expected = constructor.clone(); // Fixture owner, before measured admission.
            cut(&mut runtime, available, 0);
            let result = if inner {
                runtime.class_initialize_inner_name(env, Some("x"), &constructor)
            } else {
                runtime.initialize_class_declaration(env, "x", constructor)
            };
            if available == work {
                result.unwrap();
            } else {
                resource(result);
            }
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            let binding = &runtime.environments[env].bindings["x"];
            assert_eq!(binding.initialized, available == work);
            assert_eq!(binding.mutable, !inner);
            assert_eq!(
                &binding.value,
                if available == work {
                    &expected
                } else {
                    &Value::Undefined
                }
            );
        }
    }
    let mut runtime = Runtime::uninitialized();
    let env = runtime.class_environment(1, None).unwrap();
    cut(&mut runtime, 6, 0);
    runtime
        .class_initialize_inner_name(env, None, &Value::Undefined)
        .unwrap();
    assert!(runtime.environments[env].bindings.is_empty());
    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    cut(&mut runtime, 6, 0);
    resource(runtime.initialize_class_declaration(env, "missing", Value::Null));
    assert!(runtime.environments[env].bindings.is_empty());
    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
}

#[test]
fn class_private_prototype_slot_and_bag_admit_before_publication() {
    for cold in [false, true] {
        let growth_work = if cold { 8 } else { 0 };
        let growth_heap = if cold {
            32 + 8 * size_of::<ScriptObject>()
        } else {
            0
        };
        let slot_heap = growth_heap + size_of::<ScriptObject>();
        let heap = slot_heap + bag_heap() + size_of::<PropertyKey>();
        let work = 17 + growth_work;
        for (available_work, available_heap, ok, charged) in [
            (7, heap, false, 0),
            (work - 1, heap, false, slot_heap + bag_heap()),
            (work, heap - 1, false, heap),
            (work, heap, true, heap),
        ] {
            let mut runtime = Runtime::uninitialized();
            if !cold {
                spare_objects(&mut runtime);
            }
            cut(&mut runtime, available_work, available_heap);
            let result = runtime.class_prototype_value(Value::Null);
            if ok {
                assert_eq!(result.unwrap(), Value::Object(0));
                assert_eq!(runtime.objects[0].prototype, Some(Value::Null));
                assert!(
                    runtime.objects[0].values.is_empty()
                        && runtime.objects[0].order.capacity() >= 1
                );
            } else {
                resource(result);
            }
            assert_eq!(runtime.objects.len(), usize::from(ok));
            assert_eq!(runtime.allocated, MAX_HEAP - available_heap + charged);
            assert_eq!(runtime.steps, 0);
        }
    }
}

#[test]
fn class_private_constructor_and_method_final_metadata_cuts_do_not_publish_members_early() {
    for constructor in [false, true] {
        let work = if constructor { 187 } else { 13 };
        let heap = size_of::<Function>()
            + size_of::<ScriptObject>()
            + bag_heap()
            + if constructor {
                1648 + 3 * size_of::<PropertyKey>()
            } else {
                680
            };
        for (available_work, available_heap, ok) in [
            (if constructor { 19 } else { 9 }, heap, false),
            (work - 1, heap, false),
            (work, heap - 1, false),
            (work, heap, true),
        ] {
            let mut runtime = Runtime::try_new().unwrap();
            let env = runtime.class_environment(1, Some("C")).unwrap();
            let prototype = runtime.class_prototype_value(Value::Null).unwrap();
            let Value::Object(prototype_id) = prototype else {
                panic!("prototype");
            };
            let function = code(constructor.then_some(code::ClassKind::DefaultBase));
            let name: PropertyKey = "C".into();
            // Paid setup owners/reservations are outside the measured operation.
            spare_functions(&mut runtime);
            spare_objects(&mut runtime);
            let old_functions = runtime.functions.len();
            let old_objects = runtime.objects.len();
            let parent = Value::Function(runtime.function_prototype);
            cut(&mut runtime, available_work, available_heap);
            let result = if constructor {
                runtime.class_constructor_value(&function, env, parent, prototype, &name)
            } else {
                runtime.class_method_value(&function, env, &name, None)
            };
            if ok {
                assert_eq!(result.unwrap(), Value::Function(old_functions));
                let bag = &runtime.objects[runtime.functions[old_functions].properties];
                assert_eq!(bag.get("name"), Some(&Value::String("C".into())));
                assert_eq!(bag.get("length"), Some(&Value::Number(0.0)));
                assert_eq!(bag.values.len(), if constructor { 3 } else { 2 });
            } else {
                resource(result);
            }
            let before_first = available_work < if constructor { 20 } else { 10 };
            let published = !before_first && (constructor || available_heap == heap);
            assert_eq!(
                runtime.functions.len(),
                old_functions + usize::from(published)
            );
            assert_eq!(runtime.objects.len(), old_objects + usize::from(published));
            let expected_constructor = Value::Function(old_functions);
            assert_eq!(
                runtime.objects[prototype_id].get("constructor"),
                if constructor && ok {
                    Some(&expected_constructor)
                } else {
                    None
                }
            );
            assert!(!runtime.environments[env].bindings["C"].initialized);
            if ok || available_heap < heap {
                assert_eq!(runtime.allocated, MAX_HEAP - available_heap + heap);
            }
            if before_first {
                assert_eq!(runtime.allocated, MAX_HEAP - available_heap);
            }
            assert_eq!(
                runtime.steps,
                usize::from(!constructor && available_heap < heap)
            );
        }
    }
}

#[test]
fn class_private_fresh_descriptor_string_symbol_and_order_growth_refusals_are_atomic() {
    for symbol in [false, true] {
        for warm in [false, true] {
            let units = if symbol { 0 } else { 1 };
            let work = if warm { 9 } else { 17 };
            let heap = 256
                + 4 * units
                + if warm {
                    0
                } else {
                    32 + 8 * size_of::<PropertyKey>()
                };
            for (available_work, available_heap, ok) in [
                (7, heap, false),
                (work - 1, heap, false),
                (work, heap - 1, false),
                (work, heap, true),
            ] {
                let mut runtime = Runtime::uninitialized();
                let owner = runtime.object_ordered([]).unwrap();
                let Value::Object(id) = owner else {
                    panic!("owner");
                };
                if warm {
                    spare_order(&mut runtime, id);
                }
                let key = if symbol {
                    PropertyKey::Symbol(symbols::Symbol::unique(Some("x".into())))
                } else {
                    "x".into()
                };
                cut(&mut runtime, available_work, available_heap);
                let result = runtime.class_define_property(
                    &owner,
                    &key,
                    PropertyDescriptor::data_property(Value::Number(7.0), true, false, true),
                );
                if ok {
                    result.unwrap();
                } else {
                    resource(result);
                }
                let bag = &runtime.objects[id];
                assert_eq!(bag.values.len(), usize::from(ok));
                assert_eq!(bag.order.len(), usize::from(ok));
                if ok {
                    assert_eq!(bag.order[0], key);
                    assert_eq!(bag.get(&key), Some(&Value::Number(7.0)));
                }
                assert_eq!(runtime.steps, 0);
                if available_work == 7 {
                    assert_eq!(runtime.allocated, MAX_HEAP - available_heap);
                } else if available_work == work {
                    assert_eq!(runtime.allocated, MAX_HEAP - available_heap + heap);
                } else {
                    assert_eq!(
                        runtime.allocated,
                        MAX_HEAP - available_heap
                            + if warm {
                                0
                            } else {
                                32 + 8 * size_of::<PropertyKey>()
                            }
                    );
                }
            }
        }
    }
}

#[test]
fn class_private_descriptor_replacement_merges_accessors_and_surfaces_false() {
    for available in [10, 11] {
        let mut runtime = Runtime::uninitialized();
        let getter = runtime.function_value(&code(None), 1).unwrap();
        let setter = runtime.function_value(&code(None), 1).unwrap();
        let owner = runtime.object_ordered([]).unwrap();
        let Value::Object(id) = owner else {
            panic!("owner");
        };
        runtime.objects[id].insert_property(
            "x".into(),
            Property {
                value: PropertyValue::Accessor {
                    get: getter.clone(),
                    set: Value::Undefined,
                },
                enumerable: false,
                configurable: true,
            },
        );
        let key: PropertyKey = "x".into();
        let descriptor = PropertyDescriptor {
            set: Some(setter.clone()),
            enumerable: Some(false),
            configurable: Some(true),
            ..PropertyDescriptor::default()
        };
        cut(&mut runtime, available, 0);
        let result = runtime.class_define_property(&owner, &key, descriptor);
        if available == 11 {
            result.unwrap();
        } else {
            resource(result);
        }
        let PropertyValue::Accessor { get, set } =
            &runtime.objects[id].values[&PropertyKey::from("x")].value
        else {
            panic!("accessor");
        };
        assert_eq!(get, &getter);
        assert_eq!(
            set,
            if available == 11 {
                &setter
            } else {
                &Value::Undefined
            }
        );
        assert_eq!(runtime.objects[id].order.len(), 1);
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    }
    let mut runtime = Runtime::uninitialized();
    let owner = runtime.object_ordered([]).unwrap();
    let Value::Object(id) = owner else {
        panic!("owner");
    };
    runtime.objects[id].insert_property(
        "prototype".into(),
        Property::data(Value::Null, false, false, false),
    );
    let key: PropertyKey = "prototype".into();
    let descriptor = PropertyDescriptor::data_property(Value::Undefined, true, false, true);
    cut(&mut runtime, 20, 0); // fixed8 + Q(1)*(1+9) + old define2.
    let error = runtime
        .class_define_property(&owner, &key, descriptor)
        .unwrap_err();
    assert_eq!(error.name(), "TypeError");
    assert!(!error.is_resource_limit() && !error.is_unsupported());
    assert_eq!(runtime.objects[id].get("prototype"), Some(&Value::Null));
    assert_eq!(runtime.objects[id].order.len(), 1);
    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    runtime.objects[id].order.clear(); // Deliberate private map/order invariant fault.
    cut(&mut runtime, 8, 0);
    resource(runtime.class_define_property(&owner, &key, PropertyDescriptor::default()));
    assert_eq!(runtime.objects[id].values.len(), 1);
    assert!(runtime.objects[id].order.is_empty());
    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
}

#[test]
fn class_private_super_capture_retains_original_target_live_parent_and_prepaid_bind() {
    for initialized in [false, true] {
        for available in [14, 16, 17] {
            let mut runtime = Runtime::uninitialized();
            let function = runtime
                .function_value(&code(Some(code::ClassKind::ExplicitDerived)), 1)
                .unwrap();
            let Value::Function(id) = function else {
                panic!("function");
            };
            let owner = runtime.class_environment(1, None).unwrap();
            runtime.environments[owner].function_scope = true;
            runtime.environments[owner].active_constructor = Some(id);
            runtime.environments[owner].new_target_binding = Some(function.clone());
            runtime.environments[owner].this_binding = if initialized {
                ThisBinding::Initialized(Value::Window)
            } else {
                ThisBinding::Uninitialized
            };
            let child = runtime.class_environment(owner, None).unwrap();
            runtime.objects[runtime.functions[id].properties].prototype = Some(Value::Number(7.0));
            let returned = runtime.object_ordered([]).unwrap();
            let expected = returned.clone();
            cut(&mut runtime, available, 0);
            let result = runtime.class_super_target(child);
            if available < 17 {
                resource(result);
            } else {
                let (actual_owner, new_target, parent) = result.unwrap();
                assert_eq!(
                    (actual_owner, new_target, parent),
                    (owner, function, Value::Number(7.0))
                );
                // The capture paid the bind clone before parent effects; no new debit is permitted here.
                let bound = runtime.class_bind_this(owner, returned);
                if initialized {
                    assert_eq!(bound.unwrap_err().name(), "ReferenceError");
                    assert!(matches!(
                        runtime.environments[owner].this_binding,
                        ThisBinding::Initialized(Value::Window)
                    ));
                } else {
                    assert_eq!(bound.unwrap(), expected);
                    assert!(
                        matches!(&runtime.environments[owner].this_binding, ThisBinding::Initialized(value) if value == &expected)
                    );
                }
            }
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            assert_eq!(
                (runtime.calls, runtime.stack_units, runtime.frames.len()),
                (0, 0, 0)
            );
        }
    }
}

#[test]
fn class_private_raw_environment_allowance_preserves_old_terms_and_adds_two_deltas() {
    let runtime = Runtime::uninitialized();
    let base = if cfg!(feature = "test262-host") {
        2048.max(size_of::<Runtime>())
    } else {
        2048
    };
    // Published source: 9 initial plus 34 native bindings; five tracked names total 34 bytes.
    let old = base
        + size_of::<DateHost>()
        + size_of::<iterators::State>()
        + size_of::<array_buffer::State>()
        + size_of::<data_view::State>()
        + size_of::<typed_array::State>()
        + size_of::<dom_prototypes::State>()
        + size_of::<microtasks::State>()
        + size_of::<promises::State>()
        + 4 * size_of::<Option<Value>>()
        + 43 * (size_of::<Option<Rc<BindingAccessor>>>() + size_of::<u64>())
        + 34 * NATIVE_METADATA_BYTES
        + 524;
    let delta = size_of::<Option<usize>>()
        + size_of::<ThisBinding>().saturating_sub(size_of::<Option<Value>>());
    assert_eq!(runtime.allocated, old + 2 * delta);
    assert_eq!(runtime.environments.len(), 2);
    assert_eq!(runtime.steps, BOOTSTRAP_STEPS);
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    {
        assert_eq!(size_of::<ThisBinding>(), 24);
        assert_eq!(size_of::<Environment>(), 112);
        assert_eq!(size_of::<Function>(), 104);
        assert_eq!(size_of::<machine::Frame>(), 184);
        assert_eq!(size_of::<code::Function>(), 80);
        assert_eq!(size_of::<code::Unit>(), 168);
        assert_eq!(ENVIRONMENT_CLASS_BYTES, 16);
        assert_eq!((delta, row_heap(), 2 * delta), (16, 192, 32));
    }
}
