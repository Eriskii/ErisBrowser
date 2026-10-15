//! Borrowed-title-key signatures; fixtures precede isolated measured admissions.
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
    let Value::Native(constructor) = &runtime.environments[0].bindings["Document"].value else {
        panic!("Document constructor");
    };
    let owner = constructor.properties.unwrap().get();
    let keys = [
        named_key(&runtime, owner, "length").clone(),
        named_key(&runtime, owner, "name").clone(),
    ];
    runtime.objects.try_reserve_exact(slots).unwrap();
    assert!(runtime.objects.capacity() - runtime.objects.len() >= slots);
    (runtime, keys)
}
fn metadata(runtime: &Runtime, owner: usize, keys: &[PropertyKey; 2], display: &str, arity: usize) {
    let object = &runtime.objects[owner];
    assert_eq!(object.order, *keys);
    assert_eq!(object.values.len(), 2);
    assert_eq!(
        object.prototype,
        Some(Value::Function(runtime.function_prototype))
    );
    assert_eq!(object.get("length"), Some(&Value::Number(arity as f64)));
    assert_eq!(object.get("name"), Some(&Value::String(display.into())));
    for (index, name) in ["length", "name"].into_iter().enumerate() {
        let key = named_key(runtime, owner, name);
        let pointer = keys[index].as_string().unwrap().units().as_ptr();
        assert!(std::ptr::eq(
            key.as_string().unwrap().units().as_ptr(),
            pointer
        ));
        assert!(std::ptr::eq(
            object.order[index].as_string().unwrap().units().as_ptr(),
            pointer
        ));
        let property = &object.values[key];
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
fn title_borrowed_factories_keep_exact_fees_and_prepaid_key_clone_prefixes() {
    // Both nine-unit display strings and thirteen-byte dispatch names have the
    // same source-derived fee:9+26+2+9+15+21=82, T+769 accounted bytes.
    let heap = shared() + 769;
    for (suffix, display, arity) in [("get", "get title", 0), ("set", "set title", 1)] {
        for (work, bytes, accepted) in [(82, heap, true), (81, heap, false), (82, heap - 1, false)]
        {
            let (mut runtime, keys) = fixture(1);
            let old = runtime.objects.len();
            let capacity = runtime.objects.capacity();
            let registry = runtime.native_properties.clone();
            let globals = runtime.environments[0].bindings.len();
            let order = runtime.next_global_order;
            runtime.steps = work;
            runtime.allocated = MAX_HEAP - bytes;
            let result =
                runtime.document_title_function(suffix, display, arity, [&keys[0], &keys[1]]);
            assert_eq!(runtime.steps, 0);
            assert_eq!(
                (runtime.objects.len(), runtime.objects.capacity()),
                (old + 1, capacity)
            );
            assert_eq!(runtime.native_properties, registry);
            assert_eq!(runtime.environments[0].bindings.len(), globals);
            assert_eq!(runtime.next_global_order, order);
            metadata(&runtime, old, &keys, display, arity);
            if accepted {
                let Value::Native(native) = result.unwrap() else {
                    panic!("title native");
                };
                assert_eq!(native.properties.unwrap().get(), old);
                assert_eq!(native.name, format!("DOM.Title.{suffix}"));
                assert_eq!(native.receiver, Value::Undefined);
                assert_eq!(runtime.allocated, MAX_HEAP);
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert_eq!(
                    runtime.allocated,
                    if bytes < heap {
                        MAX_HEAP + 1
                    } else {
                        MAX_HEAP - (size_of::<Native>() + 45)
                    }
                );
            }
        }
        //36 refuses the paired clone2;37 admits it before property9 refuses.
        // A heap refusal at the display charge must still leave47 work.
        for (work, heap_refusal) in [(36, false), (37, false), (82, true)] {
            let (mut runtime, keys) = fixture(1);
            let old = runtime.objects.len();
            let capacity = runtime.objects.capacity();
            runtime.steps = work;
            runtime.allocated = if heap_refusal {
                MAX_HEAP - (bag() + 100 - 1)
            } else {
                0
            };
            assert!(
                runtime
                    .document_title_function(suffix, display, arity, [&keys[0], &keys[1]])
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(runtime.steps, if heap_refusal { 47 } else { 0 });
            assert_eq!(
                runtime.allocated,
                if heap_refusal {
                    MAX_HEAP + 1
                } else {
                    bag() + 100
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
fn title_borrowed_installer_prepays_handoff_and_keeps_owner_publication_atomic() {
    for work in [7, 11] {
        let (mut runtime, keys) = fixture(3);
        let owner = runtime.dom_proto_object(None, 1).unwrap();
        let old = runtime.objects.len();
        let capacity = runtime.objects.capacity();
        let order_capacity = runtime.objects[owner].order.capacity();
        runtime.steps = work;
        runtime.allocated = 0;
        assert!(
            runtime
                .install_document_title(owner, [&keys[0], &keys[1]])
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!((runtime.steps, runtime.allocated), (0, 0));
        assert_eq!(
            (runtime.objects.len(), runtime.objects.capacity()),
            (old, capacity)
        );
        assert_eq!(runtime.objects[owner].order.capacity(), order_capacity);
        assert!(runtime.objects[owner].order.is_empty());
        assert!(runtime.objects[owner].values.is_empty());
    }
    // Empty owner with one physical order slot and two remaining arena slots:
    // handoff8 + old entry4 +2*82 +title text18 +owner insert9 =203.
    let heap = 2 * shared() + leaf() + 1898;
    for (work, bytes, accepted) in [
        (203, heap, true),
        (202, heap, false),
        (203, heap - 1, false),
    ] {
        let (mut runtime, keys) = fixture(3);
        let owner = runtime.dom_proto_object(None, 1).unwrap();
        let old = runtime.objects.len();
        let capacity = runtime.objects.capacity();
        let order_capacity = runtime.objects[owner].order.capacity();
        assert!(order_capacity >= 1);
        assert!(capacity >= old + 2);
        let registry = runtime.native_properties.clone();
        let prototypes = runtime.prototypes.clone();
        let globals = runtime.environments[0].bindings.len();
        let global_order = runtime.next_global_order;
        runtime.steps = work;
        runtime.allocated = MAX_HEAP - bytes;
        let result = runtime.install_document_title(owner, [&keys[0], &keys[1]]);
        assert_eq!(runtime.steps, 0);
        assert_eq!(
            (runtime.objects.len(), runtime.objects.capacity()),
            (old + 2, capacity)
        );
        assert_eq!(runtime.objects[owner].order.capacity(), order_capacity);
        assert_eq!(runtime.objects[owner].prototype, None);
        assert_eq!(runtime.native_properties, registry);
        assert_eq!(runtime.prototypes, prototypes);
        assert_eq!(runtime.environments[0].bindings.len(), globals);
        assert_eq!(runtime.next_global_order, global_order);
        metadata(&runtime, old, &keys, "get title", 0);
        metadata(&runtime, old + 1, &keys, "set title", 1);
        if accepted {
            result.unwrap();
            assert_eq!(runtime.allocated, MAX_HEAP);
            assert_eq!(runtime.objects[owner].order, [PropertyKey::from("title")]);
            assert_eq!(runtime.objects[owner].values.len(), 1);
            let property = &runtime.objects[owner].values[&PropertyKey::from("title")];
            assert!(property.enumerable && property.configurable);
            let PropertyValue::Accessor {
                get: Value::Native(get),
                set: Value::Native(set),
            } = &property.value
            else {
                panic!("title accessor");
            };
            assert_eq!(get.properties.unwrap().get(), old);
            assert_eq!(set.properties.unwrap().get(), old + 1);
            assert_eq!(get.name, "DOM.Title.get");
            assert_eq!(set.name, "DOM.Title.set");
            assert!(!Rc::ptr_eq(get, set));
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            assert!(runtime.objects[owner].order.is_empty());
            assert!(runtime.objects[owner].values.is_empty());
            assert_eq!(
                runtime.allocated,
                if bytes < heap {
                    MAX_HEAP + 1
                } else {
                    MAX_HEAP - (leaf() + 276)
                }
            );
        }
    }
}
