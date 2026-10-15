//! Selected private signatures: import only with the borrowed Text-key implementation.
use super::*;
use std::mem::size_of;

fn leaf() -> usize {
    16 * (size_of::<PropertyKey>() + size_of::<Property>()) + 32 * size_of::<usize>() + 64
}
fn shared() -> usize {
    size_of::<Option<AbortSlot>>()
        + size_of::<Option<f64>>()
        + 2 * size_of::<PropertyKey>()
        + leaf()
        + size_of::<Native>()
}
fn named_key<'a>(runtime: &'a Runtime, owner: usize, name: &str) -> &'a PropertyKey {
    runtime.objects[owner]
        .values
        .keys()
        .find(|key| {
            key.as_string()
                .is_some_and(|text| text.units().iter().copied().eq(name.bytes().map(u16::from)))
        })
        .unwrap()
}
fn fixture(slots: usize) -> (Runtime, [PropertyKey; 2]) {
    let mut runtime = Runtime::try_new().unwrap();
    let Value::Native(constructor) = &runtime.environments[0].bindings["Text"].value else {
        panic!("Text constructor")
    };
    let owner = constructor.properties.unwrap().get();
    let keys = [
        named_key(&runtime, owner, "length").clone(),
        named_key(&runtime, owner, "name").clone(),
    ];
    // Fixture storage and original pool-handle acquisition precede measured budgets.
    runtime.objects.try_reserve_exact(slots).unwrap();
    (runtime, keys)
}
fn metadata(runtime: &Runtime, owner: usize, keys: &[PropertyKey; 2]) {
    assert_eq!(runtime.objects[owner].order, *keys);
    assert_eq!(runtime.objects[owner].values.len(), 2);
    assert_eq!(
        runtime.objects[owner].prototype,
        Some(Value::Function(runtime.function_prototype))
    );
    for (index, name) in ["length", "name"].into_iter().enumerate() {
        let key = named_key(runtime, owner, name);
        assert!(std::ptr::eq(
            key.as_string().unwrap().units().as_ptr(),
            keys[index].as_string().unwrap().units().as_ptr()
        ));
        assert!(std::ptr::eq(
            runtime.objects[owner].order[index]
                .as_string()
                .unwrap()
                .units()
                .as_ptr(),
            keys[index].as_string().unwrap().units().as_ptr()
        ));
        let property = &runtime.objects[owner].values[key];
        assert!(!property.enumerable && property.configurable);
        assert!(matches!(
            &property.value,
            PropertyValue::Data {
                writable: false,
                ..
            }
        ));
    }
}

#[test]
fn text_borrowed_factories_have_exact_fees_key_identity_and_clone_stage_prefixes() {
    // suffix, display, arity, total work, heap constant, internal-name bytes,
    // one-short clone budget, display heap, work through the display charge.
    for (
        suffix,
        display,
        arity,
        work,
        constant,
        internal,
        clone_short,
        display_heap,
        display_work,
    ) in [
        ("getWholeText", "get wholeText", 0, 98, 793, 21, 44, 116, 43),
        ("splitText", "splitText", 1, 87, 774, 18, 36, 100, 35),
    ] {
        let heap = shared() + constant;
        for (budget, bytes, accepted) in [
            (work, heap, true),
            (work - 1, heap, false),
            (work, heap - 1, false),
        ] {
            let (mut runtime, keys) = fixture(1);
            let old = runtime.objects.len();
            let capacity = runtime.objects.capacity();
            let registry = runtime.native_properties.clone();
            let globals = runtime.environments[0].bindings.len();
            let order = runtime.next_global_order;
            runtime.steps = budget;
            runtime.allocated = MAX_HEAP - bytes;
            let result =
                runtime.text_operations_function(suffix, display, arity, [&keys[0], &keys[1]]);
            assert_eq!(runtime.steps, 0, "{suffix} budget={budget} bytes={bytes}");
            assert_eq!(
                (runtime.objects.len(), runtime.objects.capacity()),
                (old + 1, capacity)
            );
            assert_eq!(runtime.native_properties, registry);
            assert_eq!(runtime.environments[0].bindings.len(), globals);
            assert_eq!(runtime.next_global_order, order);
            metadata(&runtime, old, &keys);
            assert_eq!(
                runtime.objects[old].get("length"),
                Some(&Value::Number(arity as f64))
            );
            assert_eq!(
                runtime.objects[old].get("name"),
                Some(&Value::String(display.into()))
            );
            if accepted {
                let Value::Native(native) = result.unwrap() else {
                    panic!("Text native")
                };
                assert_eq!(native.properties.unwrap().get(), old);
                assert_eq!(native.name, format!("DOM.Text.{suffix}"));
                assert_eq!(native.receiver, Value::Undefined);
                assert_eq!(runtime.allocated, MAX_HEAP);
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert_eq!(
                    runtime.allocated,
                    if bytes < heap {
                        MAX_HEAP + 1
                    } else {
                        MAX_HEAP - (size_of::<Native>() + 32 + internal)
                    }
                );
            }
        }
        let bag = 72
            + size_of::<Option<AbortSlot>>()
            + size_of::<Option<f64>>()
            + 2 * size_of::<PropertyKey>();
        // Insufficient clone work retains the completed display stage but no property.
        // Refusing that display charge must precede either clone debit.
        for heap_refusal in [false, true] {
            let (mut runtime, keys) = fixture(1);
            let old = runtime.objects.len();
            let capacity = runtime.objects.capacity();
            runtime.steps = if heap_refusal { work } else { clone_short };
            runtime.allocated = if heap_refusal {
                MAX_HEAP - (bag + display_heap - 1)
            } else {
                0
            };
            let error = runtime
                .text_operations_function(suffix, display, arity, [&keys[0], &keys[1]])
                .unwrap_err();
            assert!(error.is_resource_limit());
            assert_eq!(
                runtime.steps,
                if heap_refusal { work - display_work } else { 0 }
            );
            assert_eq!(
                runtime.allocated,
                if heap_refusal {
                    MAX_HEAP + 1
                } else {
                    bag + display_heap
                }
            );
            assert_eq!(
                (runtime.objects.len(), runtime.objects.capacity()),
                (old + 1, capacity)
            );
            assert!(runtime.objects[old].values.is_empty());
            assert!(runtime.objects[old].order.is_empty());
        }
    }
}

#[test]
fn text_borrowed_installer_pays_handoff_and_preserves_exact_owner_refusal_prefixes() {
    for work in [7, 11] {
        let (mut runtime, keys) = fixture(3);
        let owner = runtime.dom_proto_object(None, 2).unwrap();
        let old = runtime.objects.len();
        let capacity = runtime.objects.capacity();
        let owner_capacity = runtime.objects[owner].order.capacity();
        runtime.steps = work;
        runtime.allocated = 0;
        assert!(
            runtime
                .install_text_operations(owner, [&keys[0], &keys[1]])
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!((runtime.steps, runtime.allocated), (0, 0));
        assert_eq!(
            (runtime.objects.len(), runtime.objects.capacity()),
            (old, capacity)
        );
        assert!(runtime.objects[owner].values.is_empty());
        assert!(runtime.objects[owner].order.is_empty());
        assert_eq!(runtime.objects[owner].order.capacity(), owner_capacity);
    }
    // Empty owner with its two order slots already prepared. This excludes owner
    // construction and differs from the real tagged Text prototype's 300 work.
    let heap = 2 * shared() + leaf() + 2351;
    for (work, bytes, accepted, remaining) in [
        (278, heap, true, 0),
        (279, heap, true, 1),
        (277, heap, false, 0),
        (278, heap - 1, false, 0),
    ] {
        let (mut runtime, keys) = fixture(3);
        let owner = runtime.dom_proto_object(None, 2).unwrap();
        let old = runtime.objects.len();
        let capacity = runtime.objects.capacity();
        let owner_capacity = runtime.objects[owner].order.capacity();
        assert!(owner_capacity >= 2);
        assert!(capacity >= old + 2);
        let registry = runtime.native_properties.clone();
        let prototypes = runtime.prototypes.clone();
        let globals = runtime.environments[0].bindings.len();
        let order = runtime.next_global_order;
        runtime.steps = work;
        runtime.allocated = MAX_HEAP - bytes;
        let result = runtime.install_text_operations(owner, [&keys[0], &keys[1]]);
        assert_eq!(runtime.steps, remaining);
        assert_eq!(
            (runtime.objects.len(), runtime.objects.capacity()),
            (old + 2, capacity)
        );
        assert_eq!(runtime.objects[owner].order.capacity(), owner_capacity);
        assert_eq!(runtime.objects[owner].prototype, None);
        assert_eq!(runtime.native_properties, registry);
        assert_eq!(runtime.prototypes, prototypes);
        assert_eq!(runtime.environments[0].bindings.len(), globals);
        assert_eq!(runtime.next_global_order, order);
        metadata(&runtime, old, &keys);
        metadata(&runtime, old + 1, &keys);
        let whole = &runtime.objects[owner].values[&PropertyKey::from("wholeText")];
        assert!(whole.enumerable && whole.configurable);
        let PropertyValue::Accessor {
            get: Value::Native(get),
            set: Value::Undefined,
        } = &whole.value
        else {
            panic!("wholeText accessor")
        };
        assert_eq!(get.properties.unwrap().get(), old);
        assert_eq!(get.name, "DOM.Text.getWholeText");
        if accepted {
            result.unwrap();
            assert_eq!(runtime.allocated, MAX_HEAP);
            assert_eq!(
                runtime.objects[owner].order,
                [
                    PropertyKey::from("wholeText"),
                    PropertyKey::from("splitText")
                ]
            );
            assert_eq!(runtime.objects[owner].values.len(), 2);
            let split = &runtime.objects[owner].values[&PropertyKey::from("splitText")];
            assert!(split.enumerable && split.configurable);
            let PropertyValue::Data {
                value: Value::Native(split),
                writable: true,
            } = &split.value
            else {
                panic!("splitText method")
            };
            assert_eq!(split.properties.unwrap().get(), old + 1);
            assert_eq!(split.name, "DOM.Text.splitText");
            assert!(!std::rc::Rc::ptr_eq(get, split));
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            assert_eq!(
                runtime.objects[owner].order,
                [PropertyKey::from("wholeText")]
            );
            assert_eq!(runtime.objects[owner].values.len(), 1);
            assert_eq!(
                runtime.allocated,
                if bytes < heap {
                    MAX_HEAP + 1
                } else {
                    MAX_HEAP - 292
                }
            );
        }
    }
}
