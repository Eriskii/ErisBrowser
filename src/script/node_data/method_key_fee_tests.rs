//! Private signature and quota controls; import only with the selected implementation.
use super::key_cache_tests::{cached_keys, named_key};
use super::method_key_tests::CASES;
use super::*;
use std::mem::size_of;

const FACTORY_FEES: [(usize, usize); 9] = [
    (104, 803),
    (93, 784),
    (99, 794),
    (87, 774),
    (87, 774),
    (93, 784),
    (90, 779),
    (129, 844),
    (84, 769),
];

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
fn node_method_borrowed_factory_has_nine_exact_work_and_heap_endpoints() {
    for (case, (work, constant)) in CASES.iter().zip(FACTORY_FEES) {
        let heap = shared() + constant;
        for (budget, bytes, accepted) in [
            (work, heap, true),
            (work - 1, heap, false),
            (work, heap - 1, false),
        ] {
            let (mut runtime, keys) = fixture(1);
            let old = runtime.objects.len();
            let registry = runtime.native_properties.clone();
            let globals = runtime.environments[0].bindings.len();
            let order = runtime.next_global_order;
            runtime.steps = budget;
            runtime.allocated = MAX_HEAP - bytes;
            let result = runtime.node_data_function_with_keys(
                case.suffix,
                case.display,
                case.arity,
                [&keys[0], &keys[1]],
            );
            assert_eq!(runtime.steps, 0, "{} budget={budget}", case.member);
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
                assert_eq!(native.receiver, Value::Undefined);
                assert_eq!(runtime.allocated, MAX_HEAP);
                for (index, name) in ["length", "name"].into_iter().enumerate() {
                    assert!(std::ptr::eq(
                        named_key(&runtime, old, name)
                            .as_string()
                            .unwrap()
                            .units()
                            .as_ptr(),
                        keys[index].as_string().unwrap().units().as_ptr()
                    ));
                }
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert_eq!(
                    runtime.allocated,
                    if bytes < heap {
                        MAX_HEAP + 1
                    } else {
                        MAX_HEAP - (size_of::<Native>() + 41 + case.suffix.len())
                    }
                );
            }
        }
    }
}

#[test]
fn node_method_factory_work_refusal_preserves_the_longest_name_stages() {
    let base = 72 + size_of::<Option<AbortSlot>>() + size_of::<Option<f64>>();
    let bag = base + 2 * size_of::<PropertyKey>();
    let display = bag + 156;
    let first = display + 280 + leaf();
    let second = first + 272;
    for (work, bags, properties, charged) in [
        (7, 0, 0, 0),
        (8, 0, 0, base),
        (62, 1, 0, bag),
        (64, 1, 0, display),
        (73, 1, 0, display),
        (88, 1, 1, first),
        (128, 1, 2, second),
    ] {
        let (mut runtime, keys) = fixture(1);
        let old = runtime.objects.len();
        runtime.steps = work;
        runtime.allocated = 0;
        let error = runtime
            .node_data_function_with_keys(
                "compareDocumentPosition",
                "compareDocumentPosition",
                1,
                [&keys[0], &keys[1]],
            )
            .unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, charged);
        assert_eq!(runtime.objects.len(), old + bags);
        if bags != 0 {
            assert_eq!(runtime.objects[old].values.len(), properties);
            assert_eq!(runtime.objects[old].order.len(), properties);
        }
        assert_eq!(runtime.environments[0].bindings.len(), 213);
        assert_eq!(runtime.native_properties.len(), 300);
    }
}

#[test]
fn node_method_factory_heap_refusal_preserves_the_longest_name_stages() {
    let base = 72 + size_of::<Option<AbortSlot>>() + size_of::<Option<f64>>();
    let bag = base + 2 * size_of::<PropertyKey>();
    let display = bag + 156;
    let first = display + 280 + leaf();
    let second = first + 272;
    let total = shared() + 844;
    for (boundary, consumed, bags, properties) in [
        (base, 8, 0, 0),
        (bag, 9, 0, 0),
        (display, 63, 1, 0),
        (first, 74, 1, 0),
        (second, 89, 1, 1),
        (total, 129, 1, 2),
    ] {
        let (mut runtime, keys) = fixture(1);
        let old = runtime.objects.len();
        runtime.steps = 129;
        runtime.allocated = MAX_HEAP - (boundary - 1);
        let error = runtime
            .node_data_function_with_keys(
                "compareDocumentPosition",
                "compareDocumentPosition",
                1,
                [&keys[0], &keys[1]],
            )
            .unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(runtime.steps, 129 - consumed);
        assert_eq!(runtime.allocated, MAX_HEAP + 1);
        assert_eq!(runtime.objects.len(), old + bags);
        if bags != 0 {
            assert_eq!(runtime.objects[old].values.len(), properties);
            assert_eq!(runtime.objects[old].order.len(), properties);
        }
    }
}

struct Installer {
    call: for<'a> fn(&mut Runtime, usize, [&'a PropertyKey; 2]) -> Result<()>,
    names: &'static [&'static str],
    work: usize,
    constant: usize,
    final_property_heap: usize,
}
fn installers() -> [Installer; 7] {
    [
        Installer {
            call: Runtime::install_node_connected,
            names: &["isConnected"],
            work: 155,
            constant: 1211,
            final_property_heap: 300,
        },
        Installer {
            call: Runtime::install_node_root,
            names: &["getRootNode"],
            work: 144,
            constant: 1192,
            final_property_heap: 300,
        },
        Installer {
            call: Runtime::install_node_has_child_nodes,
            names: &["hasChildNodes"],
            work: 154,
            constant: 1218,
            final_property_heap: 308,
        },
        Installer {
            call: Runtime::install_node_normalize,
            names: &["normalize"],
            work: 134,
            constant: 1166,
            final_property_heap: 292,
        },
        Installer {
            call: Runtime::install_node_clone,
            names: &["cloneNode"],
            work: 134,
            constant: 1166,
            final_property_heap: 292,
        },
        Installer {
            call: Runtime::install_node_equality,
            names: &["isEqualNode"],
            work: 144,
            constant: 1192,
            final_property_heap: 300,
        },
        Installer {
            call: Runtime::install_node_identity_members,
            names: &["isSameNode", "compareDocumentPosition", "contains"],
            work: 493,
            constant: 3680,
            final_property_heap: 288,
        },
    ]
}

#[test]
fn node_method_seven_handoffs_precede_old_entry_work_and_any_metadata() {
    for installer in installers() {
        for work in [7, 8, 11, 12] {
            let (mut runtime, keys) = fixture(installer.names.len() + 1);
            let owner = runtime
                .dom_proto_object(None, installer.names.len())
                .unwrap();
            let old = runtime.objects.len();
            let order = runtime.next_global_order;
            runtime.steps = work;
            runtime.allocated = 0;
            assert!(
                (installer.call)(&mut runtime, owner, [&keys[0], &keys[1]])
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(runtime.steps, 0);
            assert_eq!(runtime.allocated, 0);
            assert_eq!(runtime.objects.len(), old);
            assert!(runtime.objects[owner].values.is_empty());
            assert!(runtime.objects[owner].order.is_empty());
            assert_eq!(runtime.next_global_order, order);
            assert_eq!(runtime.native_properties.len(), 300);
            assert_eq!(runtime.environments[0].bindings.len(), 213);
        }
    }
}

#[test]
fn node_method_complete_installer_endpoints_include_owner_publication_and_one_handoff() {
    for installer in installers() {
        let count = installer.names.len();
        let heap = count * shared() + installer.constant + leaf();
        for (work, bytes, accepted, remaining) in [
            (installer.work, heap, true, 0),
            (installer.work + 1, heap, true, 1),
            (installer.work - 1, heap, false, 0),
            (installer.work, heap - 1, false, 0),
        ] {
            let (mut runtime, keys) = fixture(count + 1);
            let owner = runtime.dom_proto_object(None, count).unwrap();
            assert!(runtime.objects[owner].order.capacity() >= count);
            let old = runtime.objects.len();
            let registry = runtime.native_properties.clone();
            let global_order = runtime.next_global_order;
            runtime.steps = work;
            runtime.allocated = MAX_HEAP - bytes;
            let result = (installer.call)(&mut runtime, owner, [&keys[0], &keys[1]]);
            assert_eq!(
                runtime.steps, remaining,
                "{:?} work={work}",
                installer.names
            );
            assert_eq!(runtime.objects.len(), old + count);
            assert_eq!(runtime.native_properties, registry);
            assert_eq!(runtime.next_global_order, global_order);
            assert_eq!(runtime.environments[0].bindings.len(), 213);
            let published = if accepted { count } else { count - 1 };
            let expected: Vec<PropertyKey> = installer.names[..published]
                .iter()
                .copied()
                .map(PropertyKey::from)
                .collect();
            assert_eq!(runtime.objects[owner].order, expected);
            assert_eq!(runtime.objects[owner].values.len(), published);
            for bag in old..old + count {
                assert_eq!(runtime.objects[bag].order, keys);
                assert_eq!(runtime.objects[bag].values.len(), 2);
                for (index, name) in ["length", "name"].into_iter().enumerate() {
                    assert!(std::ptr::eq(
                        named_key(&runtime, bag, name)
                            .as_string()
                            .unwrap()
                            .units()
                            .as_ptr(),
                        keys[index].as_string().unwrap().units().as_ptr()
                    ));
                }
            }
            if accepted {
                result.unwrap();
                assert_eq!(runtime.allocated, MAX_HEAP);
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                let unpaid = installer.final_property_heap + if count == 1 { leaf() } else { 0 };
                assert_eq!(
                    runtime.allocated,
                    if bytes < heap {
                        MAX_HEAP + 1
                    } else {
                        MAX_HEAP - unpaid
                    }
                );
            }
        }
    }
}

#[test]
fn node_method_owner_growth_uses_actual_capacity_without_charging_spare_slots_twice() {
    for requested in [0, 1, 4] {
        let heap = shared()
            + 1192
            + leaf()
            + if requested == 0 {
                size_of::<PropertyKey>()
            } else {
                0
            };
        for short in [false, true] {
            // Every attempt starts with an empty owner and fresh factory bag.
            let (mut runtime, keys) = fixture(2);
            let owner = runtime.dom_proto_object(None, requested).unwrap();
            let capacity = runtime.objects[owner].order.capacity();
            if requested == 0 {
                assert_eq!(capacity, 0);
            } else {
                assert!(capacity >= requested);
            }
            let old = runtime.objects.len();
            runtime.steps = 144;
            runtime.allocated = MAX_HEAP - heap + usize::from(short);
            let result = runtime.install_node_root(owner, [&keys[0], &keys[1]]);
            assert_eq!(runtime.steps, 0);
            assert_eq!(runtime.objects.len(), old + 1);
            assert_eq!(runtime.objects[old].order, keys);
            if short {
                assert!(result.unwrap_err().is_resource_limit());
                assert_eq!(runtime.allocated, MAX_HEAP + 1);
                assert!(runtime.objects[owner].values.is_empty());
                assert!(runtime.objects[owner].order.is_empty());
            } else {
                result.unwrap();
                assert_eq!(runtime.allocated, MAX_HEAP);
                assert_eq!(
                    runtime.objects[owner].order,
                    [PropertyKey::from("getRootNode")]
                );
            }
        }
    }
}
