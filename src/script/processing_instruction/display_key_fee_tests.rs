//! Private tuple-return controls; import only with the selected PI factory.
use super::key_cache_tests::{CASES, cached_keys};
use super::*;
use std::mem::size_of;

fn leaf() -> usize {
    16 * (size_of::<PropertyKey>() + size_of::<Property>()) + 32 * size_of::<usize>() + 64
}
fn bag() -> usize {
    72 + size_of::<Option<AbortSlot>>() + size_of::<Option<f64>>() + 2 * size_of::<PropertyKey>()
}
fn shared() -> usize {
    bag() - 72 + leaf() + size_of::<Native>()
}
fn fixture(slots: usize) -> (Runtime, [PropertyKey; 2]) {
    let mut runtime = Runtime::try_new().unwrap();
    let keys = cached_keys(&runtime);
    runtime.objects.try_reserve_exact(slots).unwrap();
    assert!(runtime.objects.capacity() - runtime.objects.len() >= slots);
    (runtime, keys)
}

#[test]
fn retained_display_factory_prepays_one_clone_and_returns_the_same_immutable_payload() {
    for (case, work) in CASES.iter().zip([119, 85, 79, 82, 85, 98, 89, 89, 89, 92]) {
        assert_eq!(case.work, work); // Explicit selected migration, not a measured oracle.
        let display_work = 17 + 2 * case.display.len();
        let display_heap = bag() + 64 + 4 * case.display.len();
        // The final pair separates the one-short first insertion from its exact
        // admission, including the preceding new1 and existing2 clone debits.
        for (budget, properties) in [
            (display_work, 0),
            (display_work + 1, 0),
            (display_work + 11, 0),
            (display_work + 12, 1),
        ] {
            let (mut runtime, keys) = fixture(1);
            let old = runtime.objects.len();
            let capacity = runtime.objects.capacity();
            let registry = runtime.native_properties.clone();
            runtime.steps = budget;
            runtime.allocated = 0;
            assert!(
                runtime
                    .pi_function(case.suffix, case.display, case.arity, [&keys[0], &keys[1]])
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(runtime.steps, 0);
            assert_eq!(
                runtime.allocated,
                display_heap + if properties == 0 { 0 } else { 280 + leaf() }
            );
            assert_eq!(
                (runtime.objects.len(), runtime.objects.capacity()),
                (old + 1, capacity)
            );
            assert_eq!(runtime.objects[old].values.len(), properties);
            assert_eq!(runtime.objects[old].order, keys[..properties]);
            assert_eq!(runtime.native_properties, registry);
            if properties == 1 {
                assert_eq!(
                    runtime.objects[old].get("length"),
                    Some(&Value::Number(case.arity as f64))
                );
            }
        }
        // Display heap refusal precedes the new clone debit.
        let (mut runtime, keys) = fixture(1);
        let old = runtime.objects.len();
        runtime.steps = work;
        runtime.allocated = MAX_HEAP - (display_heap - 1);
        assert!(
            runtime
                .pi_function(case.suffix, case.display, case.arity, [&keys[0], &keys[1]])
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(
            (runtime.steps, runtime.allocated),
            (work - display_work, MAX_HEAP + 1)
        );
        assert_eq!(runtime.objects.len(), old + 1);
        assert!(runtime.objects[old].values.is_empty());
        assert!(runtime.objects[old].order.is_empty());

        let (mut runtime, keys) = fixture(1);
        let old = runtime.objects.len();
        let capacity = runtime.objects.capacity();
        let registry = runtime.native_properties.clone();
        let globals = runtime.environments[0].bindings.len();
        let global_order = runtime.next_global_order;
        runtime.steps = work;
        runtime.allocated = MAX_HEAP - (shared() + case.heap_constant);
        let (value, key) = runtime
            .pi_function(case.suffix, case.display, case.arity, [&keys[0], &keys[1]])
            .unwrap();
        let Value::Native(native) = value else {
            panic!("PI native");
        };
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        assert_eq!(
            (runtime.objects.len(), runtime.objects.capacity()),
            (old + 1, capacity)
        );
        assert_eq!(native.properties.unwrap().get(), old);
        assert_eq!(native.name, format!("DOM.PI.{}", case.suffix));
        assert_eq!(native.receiver, Value::Undefined);
        assert_eq!(runtime.native_properties, registry);
        assert_eq!(runtime.environments[0].bindings.len(), globals);
        assert_eq!(runtime.next_global_order, global_order);
        assert_eq!(runtime.objects[old].order, keys);
        let Some(Value::String(display)) = runtime.objects[old].get("name") else {
            panic!("display");
        };
        assert_eq!(key, PropertyKey::from(case.display));
        assert!(std::ptr::eq(
            key.as_string().unwrap().units().as_ptr(),
            display.units().as_ptr()
        ));
        // Replacing the mutable metadata value cannot mutate the retained key.
        runtime.objects[old]
            .values
            .get_mut(&PropertyKey::from("name"))
            .unwrap()
            .value = PropertyValue::Data {
            value: Value::String("changed".into()),
            writable: false,
        };
        assert_eq!(key, PropertyKey::from(case.display));
        assert_eq!(native.properties.unwrap().get(), old);
    }
}

#[test]
fn retained_owner_insertions_and_pooled_length_clone_keep_the_reached_prefix() {
    // Empty Document fixture: entry4 + create119 + owner insertion9 =132.
    for (work, accepted) in [(123, false), (131, false), (132, true)] {
        let (mut runtime, keys) = fixture(2);
        let owner = runtime.dom_proto_object(None, 1).unwrap();
        let old = runtime.objects.len();
        let capacity = runtime.objects.capacity();
        let order_capacity = runtime.objects[owner].order.capacity();
        let registry = runtime.native_properties.clone();
        runtime.steps = work;
        runtime.allocated = 0;
        let result = runtime.install_pi_members("Document", owner, [&keys[0], &keys[1]]);
        assert_eq!(runtime.steps, 0);
        assert_eq!(
            (runtime.objects.len(), runtime.objects.capacity()),
            (old + 1, capacity)
        );
        assert_eq!(runtime.objects[owner].order.capacity(), order_capacity);
        assert_eq!(runtime.native_properties, registry);
        assert_eq!(runtime.objects[old].order, keys);
        assert_eq!(
            runtime.allocated,
            shared() + 841 + if accepted { leaf() + 364 } else { 0 }
        );
        if accepted {
            result.unwrap();
            assert_eq!(
                runtime.objects[owner].order,
                [PropertyKey::from("createProcessingInstruction")]
            );
            let key = &runtime.objects[owner].order[0];
            let PropertyValue::Data {
                value: Value::Native(native),
                writable: true,
            } = &runtime.objects[owner].values[key].value
            else {
                panic!("method");
            };
            assert_eq!(native.properties.unwrap().get(), old);
            let Some(Value::String(display)) = runtime.objects[old].get("name") else {
                panic!("display");
            };
            assert!(std::ptr::eq(
                key.as_string().unwrap().units().as_ptr(),
                display.units().as_ptr()
            ));
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            assert!(runtime.objects[owner].values.is_empty());
            assert!(runtime.objects[owner].order.is_empty());
        }
    }
    // Empty CharacterData owner with all order capacity prepared. Before length
    // clone:4+79+82+16+9+85=275; clone1; insertion17; then loop debit5.
    for (work, length_published) in [(275, false), (276, false), (292, false), (293, true)] {
        let (mut runtime, keys) = fixture(4);
        let owner = runtime.dom_proto_object(None, 7).unwrap();
        let old = runtime.objects.len();
        let capacity = runtime.objects.capacity();
        let order_capacity = runtime.objects[owner].order.capacity();
        assert!(order_capacity >= 7);
        let registry = runtime.native_properties.clone();
        runtime.steps = work;
        runtime.allocated = 0;
        assert!(
            runtime
                .install_pi_members("CharacterData", owner, [&keys[0], &keys[1]])
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.steps, 0);
        assert_eq!(
            (runtime.objects.len(), runtime.objects.capacity()),
            (old + 3, capacity)
        );
        assert_eq!(runtime.objects[owner].order.capacity(), order_capacity);
        assert_eq!(runtime.native_properties, registry);
        assert_eq!(
            runtime.allocated,
            3 * shared() + leaf() + 2654 + if length_published { 280 } else { 0 }
        );
        let PropertyValue::Accessor {
            get: Value::Native(get),
            set: Value::Native(set),
        } = &runtime.objects[owner].values[&PropertyKey::from("data")].value
        else {
            panic!("data accessor");
        };
        assert_eq!(get.properties.unwrap().get(), old);
        assert_eq!(set.properties.unwrap().get(), old + 1);
        if length_published {
            assert_eq!(
                runtime.objects[owner].order,
                [PropertyKey::from("data"), PropertyKey::from("length")]
            );
            let key = &runtime.objects[owner].order[1];
            assert!(std::ptr::eq(
                key.as_string().unwrap().units().as_ptr(),
                keys[0].as_string().unwrap().units().as_ptr()
            ));
            let PropertyValue::Accessor {
                get: Value::Native(get),
                set: Value::Undefined,
            } = &runtime.objects[owner].values[key].value
            else {
                panic!("length accessor");
            };
            assert_eq!(get.properties.unwrap().get(), old + 2);
        } else {
            assert_eq!(runtime.objects[owner].order, [PropertyKey::from("data")]);
            assert!(
                !runtime.objects[owner]
                    .values
                    .contains_key(&PropertyKey::from("length"))
            );
        }
    }
}
