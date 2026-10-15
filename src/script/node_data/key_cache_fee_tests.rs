//! Future private borrowed-key API controls; never import before the selected API.
use super::key_cache_tests::{CASES, cached_keys, named_key};
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
fn fixture(slots: usize) -> (Runtime, [PropertyKey; 2]) {
    let mut runtime = Runtime::try_new().unwrap();
    let keys = cached_keys(&runtime);
    runtime.objects.try_reserve_exact(slots).unwrap();
    (runtime, keys)
}

#[test]
fn node_borrowed_factory_exact_and_one_short_work_and_heap_for_all_four_accessors() {
    for case in &CASES {
        let heap = shared() + case.heap_constant;
        for (work, bytes, accepted) in [
            (case.work, heap, true),
            (case.work - 1, heap, false),
            (case.work, heap - 1, false),
        ] {
            let (mut runtime, keys) = fixture(1);
            let old = runtime.objects.len();
            let registry = runtime.native_properties.clone();
            let globals = runtime.environments[0].bindings.len();
            let order = runtime.next_global_order;
            runtime.steps = work;
            runtime.allocated = MAX_HEAP - bytes;
            let result = runtime.node_data_function_with_keys(
                case.suffix,
                case.display,
                case.arity,
                [&keys[0], &keys[1]],
            );
            assert_eq!(runtime.steps, 0);
            assert_eq!(runtime.objects.len(), old + 1);
            assert_eq!(runtime.objects[old].order, keys);
            assert_eq!(runtime.objects[old].values.len(), 2);
            assert_eq!(runtime.native_properties, registry);
            assert_eq!(runtime.environments[0].bindings.len(), globals);
            assert_eq!(runtime.next_global_order, order);
            if accepted {
                let Value::Native(native) = result.unwrap() else {
                    panic!("native")
                };
                assert_eq!(native.properties.unwrap().get(), old);
                assert_eq!(native.name, format!("DOM.Node.{}", case.suffix));
                assert_eq!(runtime.allocated, MAX_HEAP);
                for (i, name) in ["length", "name"].into_iter().enumerate() {
                    assert!(std::ptr::eq(
                        named_key(&runtime, old, name)
                            .as_string()
                            .unwrap()
                            .units()
                            .as_ptr(),
                        keys[i].as_string().unwrap().units().as_ptr()
                    ));
                }
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert_eq!(
                    runtime.allocated,
                    if bytes < heap {
                        MAX_HEAP + 1
                    } else {
                        MAX_HEAP - (size_of::<Native>() + 32 + 9 + case.suffix.len())
                    }
                );
            }
        }
    }
}

#[test]
fn node_borrowed_factory_staged_work_refusals_preserve_the_exact_private_prefix() {
    let base = 72 + size_of::<Option<AbortSlot>>() + size_of::<Option<f64>>();
    let bag = base + 2 * size_of::<PropertyKey>();
    let display = bag + 116;
    let first = display + 280 + leaf();
    let second = first + 272;
    for (work, bags, properties, charged) in [
        (7, 0, 0, 0),
        (8, 0, 0, base),
        (42, 1, 0, bag),
        (44, 1, 0, display),
        (53, 1, 0, display),
        (68, 1, 1, first),
        (97, 1, 2, second),
    ] {
        let (mut runtime, keys) = fixture(1);
        let old = runtime.objects.len();
        runtime.steps = work;
        runtime.allocated = 0;
        let error = runtime
            .node_data_function_with_keys("getNodeValue", "get nodeValue", 0, [&keys[0], &keys[1]])
            .unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, charged);
        assert_eq!(runtime.objects.len(), old + bags);
        if bags != 0 {
            assert_eq!(runtime.objects[old].values.len(), properties);
            assert_eq!(runtime.objects[old].order.len(), properties);
        }
        assert_eq!(runtime.environments[0].bindings.len(), 212);
        assert_eq!(runtime.native_properties.len(), 299);
    }
}

#[test]
fn node_borrowed_factory_staged_heap_refusals_never_publish_a_callable() {
    let base = 72 + size_of::<Option<AbortSlot>>() + size_of::<Option<f64>>();
    let bag = base + 2 * size_of::<PropertyKey>();
    let display = bag + 116;
    let first = display + 280 + leaf();
    let second = first + 272;
    let total = shared() + 793;
    for (boundary, used, bags, properties) in [
        (base, 8, 0, 0),
        (bag, 9, 0, 0),
        (display, 43, 1, 0),
        (first, 54, 1, 0),
        (second, 69, 1, 1),
        (total, 98, 1, 2),
    ] {
        let (mut runtime, keys) = fixture(1);
        let old = runtime.objects.len();
        runtime.steps = 98;
        runtime.allocated = MAX_HEAP - (boundary - 1);
        let error = runtime
            .node_data_function_with_keys("getNodeValue", "get nodeValue", 0, [&keys[0], &keys[1]])
            .unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(runtime.steps, 98 - used);
        assert_eq!(runtime.allocated, MAX_HEAP + 1);
        assert_eq!(runtime.objects.len(), old + bags);
        if bags != 0 {
            assert_eq!(runtime.objects[old].values.len(), properties);
            assert_eq!(runtime.objects[old].order.len(), properties);
        }
    }
}

#[test]
fn node_key_handoff_is_paid_once_before_the_existing_installer_and_four_factories() {
    for work in [7, 8, 11, 12] {
        let (mut runtime, keys) = fixture(5);
        let owner = runtime.dom_proto_object(None, 2).unwrap();
        let old = runtime.objects.len();
        runtime.steps = work;
        runtime.allocated = 0;
        assert!(
            runtime
                .install_node_data_members(owner, [&keys[0], &keys[1]])
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, 0);
        assert_eq!(runtime.objects.len(), old);
        assert!(runtime.objects[owner].values.is_empty());
    }
    let heap = 3992 + 4 * shared() + leaf();
    for (work, bytes, accepted, remaining) in [
        (503, heap, true, 0),
        (504, heap, true, 1),
        (502, heap, false, 0),
        (503, heap - 1, false, 0),
    ] {
        let (mut runtime, keys) = fixture(5);
        let owner = runtime.dom_proto_object(None, 2).unwrap();
        let old = runtime.objects.len();
        runtime.steps = work;
        runtime.allocated = MAX_HEAP - bytes;
        let result = runtime.install_node_data_members(owner, [&keys[0], &keys[1]]);
        assert_eq!(runtime.steps, remaining);
        assert_eq!(runtime.objects.len(), old + 4);
        assert_eq!(runtime.native_properties.len(), 299);
        assert_eq!(runtime.environments[0].bindings.len(), 212);
        if accepted {
            result.unwrap();
            assert_eq!(runtime.allocated, MAX_HEAP);
            assert_eq!(
                runtime.objects[owner].order,
                [
                    PropertyKey::from("nodeValue"),
                    PropertyKey::from("textContent")
                ]
            );
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            assert_eq!(
                runtime.objects[owner].order,
                [PropertyKey::from("nodeValue")]
            );
            assert_eq!(
                runtime.allocated,
                if bytes < heap {
                    MAX_HEAP + 1
                } else {
                    MAX_HEAP - 300
                }
            );
        }
    }
}

#[test]
fn node_unselected_original_factory_keeps_its_exact_fees_and_private_key_payloads() {
    let heap = shared() + 952;
    for (work, bytes, accepted) in [
        (127, heap, true),
        (126, heap, false),
        (127, heap - 1, false),
    ] {
        let (mut runtime, keys) = fixture(1);
        let old = runtime.objects.len();
        runtime.steps = work;
        runtime.allocated = MAX_HEAP - bytes;
        let result = runtime.node_data_function("getRootNode", "getRootNode", 0);
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.objects.len(), old + 1);
        assert_eq!(runtime.objects[old].order, keys);
        if accepted {
            let Value::Native(native) = result.unwrap() else {
                panic!("native")
            };
            assert_eq!(native.name, "DOM.Node.getRootNode");
            assert_eq!(native.properties.unwrap().get(), old);
            assert_eq!(runtime.allocated, MAX_HEAP);
            for (i, name) in ["length", "name"].into_iter().enumerate() {
                assert!(!std::ptr::eq(
                    named_key(&runtime, old, name)
                        .as_string()
                        .unwrap()
                        .units()
                        .as_ptr(),
                    keys[i].as_string().unwrap().units().as_ptr()
                ));
            }
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            assert_eq!(
                runtime.allocated,
                if bytes < heap {
                    MAX_HEAP + 1
                } else {
                    MAX_HEAP - (size_of::<Native>() + 52)
                }
            );
        }
    }
}

#[test]
fn node_borrowed_factory_keeps_the_existing_eager_object_reservation_guard() {
    let mut runtime = Runtime::try_new().unwrap();
    let keys = cached_keys(&runtime);
    assert_eq!(
        (runtime.objects.len(), runtime.objects.capacity()),
        (749, 749)
    );
    runtime.steps = 98;
    runtime.allocated = 0;
    let error = runtime
        .node_data_function_with_keys("getNodeValue", "get nodeValue", 0, [&keys[0], &keys[1]])
        .unwrap_err();
    assert!(error.is_resource_limit());
    assert_eq!(runtime.steps, 90);
    assert_eq!(
        runtime.allocated,
        72 + size_of::<Option<AbortSlot>>() + size_of::<Option<f64>>()
    );
    assert_eq!(
        (runtime.objects.len(), runtime.objects.capacity()),
        (749, 749)
    );
    assert_eq!(runtime.native_properties.len(), 299);
    assert_eq!(runtime.environments[0].bindings.len(), 212);
}
