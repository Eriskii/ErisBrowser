//! Selected AggregateError cuts; all fixture preparation precedes the operation.
use super::*;
use std::{mem::size_of, num::NonZeroUsize, rc::Rc};

fn resource<T>(result: Result<T>) {
    match result {
        Err(error) => assert!(error.is_resource_limit(), "{error:?}"),
        Ok(_) => panic!("expected resource refusal"),
    }
}

fn allowance(runtime: &mut Runtime, work: usize, heap: usize) {
    runtime.steps = work;
    runtime.allocated = MAX_HEAP - heap;
}

fn refs(value: &Value) -> usize {
    let Value::Native(native) = value else {
        panic!("Native ownership fixture")
    };
    Rc::strong_count(native)
}

fn quiet(runtime: &Runtime) {
    assert!(runtime.frames.is_empty());
    assert!(runtime.console.is_empty());
    assert!(runtime.microtasks.queue.is_empty());
    assert_eq!(
        (runtime.calls, runtime.eval_depth, runtime.stack_units),
        (0, 0, 0)
    );
}

fn wrapper_fixture() -> (Runtime, Document, Value, Rc<Native>) {
    let mut runtime = Runtime::try_new().unwrap();
    let mut document = Document::parse("");
    let target = runtime.execute(
        "var entered=0;var savedPrototype={};var N=(function(){}).bind(null);Object.defineProperty(N,'prototype',{get:function(){entered++;return savedPrototype;}});N",
        &mut document,
    ).unwrap();
    assert!(matches!(target, Value::Function(_)));
    // This ordinary bound constructor already owns a real accessor descriptor.
    // An inert native fixture uses the same bag, so both wrappers face that getter.
    let bag = runtime.property_object(&target).unwrap();
    let native = Rc::new(Native {
        name: "AggregateError".into(),
        receiver: Value::Window,
        properties: Some(NonZeroUsize::new(bag).unwrap()),
    });
    (runtime, document, target, native)
}

#[test]
fn aggregate_error_wrappers_prepaid_target_and_body_stop_before_getter() {
    for callable in [false, true] {
        for (work, heap) in [
            (1, 100),
            (2, 100),
            (17, 100),
            (18, 100),
            (19, 100),
            (20, 100),
            (45, 100),
            (46, 100),
            (46, 99),
        ] {
            let (mut runtime, mut document, target, native) = wrapper_fixture();
            let receiver = if callable {
                Value::Native(native.clone())
            } else {
                target.clone()
            };
            let key: PropertyKey = "prototype".into();
            let prototype = runtime
                .get_property_key(receiver, &key, &mut document)
                .unwrap();
            assert!(matches!(prototype, Value::Object(_)));
            assert_eq!(
                runtime.environments[0].bindings["entered"].value,
                Value::Number(1.0)
            );
            let objects = runtime.objects.len();
            let arrays = runtime.arrays.len();
            let retained = Rc::strong_count(&native);
            allowance(&mut runtime, work, heap);
            let result = if callable {
                runtime.aggregate_error_call(&native, &[], &mut document)
            } else {
                runtime.aggregate_error_construct(&[], &target, &mut document)
            };
            resource(result);
            assert_eq!(runtime.steps, 0);
            assert_eq!(
                runtime.allocated,
                MAX_HEAP - heap + if work == 46 { 100 } else { 0 }
            );
            assert_eq!(
                (runtime.objects.len(), runtime.arrays.len()),
                (objects, arrays)
            );
            assert_eq!(
                runtime.environments[0].bindings["entered"].value,
                Value::Number(1.0)
            );
            assert_eq!(Rc::strong_count(&native), retained);
            assert!(runtime.microtasks_suspended());
            quiet(&runtime);
        }
    }
}

#[test]
fn aggregate_error_instance_warm_cold_work_and_heap_prefixes() {
    let base_bag = 72 + size_of::<Option<AbortSlot>>() + size_of::<Option<f64>>();
    let order = 3 * size_of::<PropertyKey>();
    for cold in [false, true] {
        let work = if cold { 25 } else { 17 };
        let slots = if cold {
            32 + 9 * size_of::<ScriptObject>()
        } else {
            size_of::<ScriptObject>()
        };
        let bytes = slots + base_bag + order;
        for (given_work, given_heap, success) in [
            (work - 1, bytes, false),
            (work, bytes, true),
            (work, bytes - 1, false),
        ] {
            let mut runtime = Runtime::uninitialized();
            if !cold {
                runtime.objects.try_reserve_exact(1).unwrap();
                assert!(runtime.objects.capacity() >= 1);
            } else {
                assert_eq!((runtime.objects.len(), runtime.objects.capacity()), (0, 0));
            }
            let prototype = Runtime::native("Object.is", Value::Undefined);
            runtime.work(1).unwrap();
            let incoming = prototype.clone();
            assert_eq!(refs(&prototype), 2);
            allowance(&mut runtime, given_work, given_heap);
            let result = runtime.new_aggregate_error_object(incoming);
            assert_eq!(runtime.steps, 0);
            if success {
                let id = result.unwrap();
                assert_eq!(id, 0);
                assert_eq!(runtime.allocated, MAX_HEAP);
                assert_eq!(runtime.objects.len(), 1);
                let object = &runtime.objects[id];
                assert_eq!(object.intrinsic_error, Some("AggregateError"));
                assert_eq!(object.prototype.as_ref(), Some(&prototype));
                assert!(object.values.is_empty());
                assert!(object.order.is_empty());
                assert!(object.order.capacity() >= 3);
                assert_eq!(refs(&prototype), 2);
            } else {
                resource(result);
                assert!(runtime.objects.is_empty());
                assert_eq!(refs(&prototype), 1);
                assert_eq!(
                    runtime.allocated,
                    if given_work < work {
                        MAX_HEAP - order
                    } else {
                        MAX_HEAP + 1
                    }
                );
            }
            // Both selected refusal rows occur after object-slot reservation.
            assert!(runtime.objects.capacity() >= 1);
            quiet(&runtime);
        }
    }
}

fn property_fixture(name: &str, count: usize) -> (Runtime, usize, Value) {
    let mut runtime = Runtime::try_new().unwrap();
    runtime.objects.try_reserve_exact(2).unwrap();
    let owner = runtime.new_aggregate_error_object(Value::Window).unwrap();
    if count >= 1 {
        runtime
            .define_aggregate_error_data(owner, "message", Value::String("earlier".into()))
            .unwrap();
    }
    if count == 2 {
        runtime
            .define_aggregate_error_data(owner, "cause", Value::Undefined)
            .unwrap();
    }
    let payload = match name {
        "message" => Value::String("converted message".into()),
        "cause" => Runtime::native("Object.is", Value::Undefined),
        "errors" => runtime.new_all_values_array(Vec::new()).unwrap(),
        _ => panic!("fixed property name"),
    };
    assert_eq!(runtime.objects[owner].values.len(), count);
    assert_eq!(runtime.objects[owner].order.len(), count);
    assert!(runtime.objects[owner].order.capacity() >= 3);
    (runtime, owner, payload)
}

#[test]
fn aggregate_error_reachable_own_data_rows_and_invalid_owners() {
    let leaf =
        16 * (size_of::<PropertyKey>() + size_of::<Property>()) + 32 * size_of::<usize>() + 64;
    for (name, count, work, fixed_bytes) in [
        ("message", 0, 35, 376),
        ("cause", 0, 31, 360),
        ("cause", 1, 38, 360),
        ("errors", 0, 33, 368),
        ("errors", 1, 41, 368),
        ("errors", 2, 49, 368),
    ] {
        let bytes = fixed_bytes + if count == 0 { leaf } else { 0 };
        let key_bytes = 64 + 4 * name.len();
        for (given_work, given_heap, success) in [
            (work - 1, bytes, false),
            (work, bytes, true),
            (work, bytes - 1, false),
        ] {
            let (mut runtime, owner, payload) = property_fixture(name, count);
            let old_order = runtime.objects[owner].order.clone();
            let old_capacity = runtime.objects[owner].order.capacity();
            // Caller-owned input and release credit exist before isolated helper admission.
            runtime.work(17).unwrap();
            let incoming = payload.clone();
            let retained = if name == "cause" {
                Some(refs(&payload))
            } else {
                None
            };
            allowance(&mut runtime, given_work, given_heap);
            let result = runtime.define_aggregate_error_data(owner, name, incoming);
            assert_eq!(runtime.steps, 0);
            let key: PropertyKey = name.into();
            let object = &runtime.objects[owner];
            assert_eq!(object.order.capacity(), old_capacity);
            assert_eq!(object.intrinsic_error, Some("AggregateError"));
            if success {
                result.unwrap();
                assert_eq!(runtime.allocated, MAX_HEAP);
                assert_eq!(object.values.len(), count + 1);
                assert_eq!(&object.order[..count], old_order.as_slice());
                assert_eq!(object.order.last(), Some(&key));
                let property = &object.values[&key];
                assert!(!property.enumerable);
                assert!(property.configurable);
                let PropertyValue::Data { value, writable } = &property.value else {
                    panic!("own data")
                };
                assert!(*writable);
                assert_eq!(value, &payload);
                if let (Value::String(actual), Value::String(expected)) = (value, &payload) {
                    assert_eq!(actual.units().as_ptr(), expected.units().as_ptr());
                }
                if let Some(before) = retained {
                    assert_eq!(refs(&payload), before);
                }
            } else {
                resource(result);
                assert_eq!(object.values.len(), count);
                assert_eq!(object.order, old_order);
                assert!(!object.values.contains_key(&key));
                assert_eq!(
                    runtime.allocated,
                    if given_work < work {
                        MAX_HEAP - bytes + key_bytes
                    } else {
                        MAX_HEAP + 1
                    }
                );
                if let Some(before) = retained {
                    assert_eq!(refs(&payload), before - 1);
                }
            }
            quiet(&runtime);
        }
    }
    for missing in [false, true] {
        let mut runtime = Runtime::uninitialized();
        runtime.objects.try_reserve_exact(1).unwrap();
        let ordinary = runtime.dom_proto_object(None, 0).unwrap();
        let owner = if missing {
            runtime.objects.len()
        } else {
            ordinary
        };
        let payload = Runtime::native("Object.is", Value::Undefined);
        let incoming = payload.clone();
        allowance(&mut runtime, 4, 0);
        resource(runtime.define_aggregate_error_data(owner, "cause", incoming));
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        assert_eq!(refs(&payload), 1);
        assert_eq!(runtime.objects.len(), 1);
        assert!(runtime.objects[ordinary].values.is_empty());
        assert!(runtime.objects[ordinary].order.is_empty());
        quiet(&runtime);
    }
}

fn list_fixture(count: usize) -> (Runtime, AggregateList, ReceivedAggregateValue, Value) {
    let mut runtime = Runtime::try_new().unwrap();
    let mut document = Document::parse("");
    let source = runtime.execute(
        "var payload=Object.is;var items={};items[Symbol.iterator]=function(){return {next:function(){return {done:false,value:payload};}};};items",
        &mut document,
    ).unwrap();
    let marker = runtime.environments[0].bindings["payload"].value.clone();
    assert!(matches!(marker, Value::Native(_)));
    runtime.work(8).unwrap();
    let mut list = AggregateList::empty_prepaid();
    let iterator = AggregateIterator::acquire(&mut runtime, &source, &mut document).unwrap();
    for _ in 0..count {
        let received = iterator
            .next_value(&mut runtime, &mut document)
            .unwrap()
            .unwrap();
        list.append_received(&mut runtime, received).unwrap();
    }
    let received = iterator
        .next_value(&mut runtime, &mut document)
        .unwrap()
        .unwrap();
    assert_eq!(list.values.len(), count);
    assert!(list.values.iter().all(|value| value == &marker));
    (runtime, list, received, marker)
}

#[test]
fn aggregate_error_list_first_spare_growth_and_received_ownership() {
    for (count, work, bytes, logical_after) in [
        (0, 16, 32 + 8 * size_of::<Value>(), 8),
        (7, 8, 0, 8),
        (8, 32, 32 + 16 * size_of::<Value>(), 16),
    ] {
        let mut cases = vec![(work - 1, bytes, false), (work, bytes, true)];
        if bytes != 0 {
            cases.push((work, bytes - 1, false));
        }
        for (given_work, given_heap, success) in cases {
            let (mut runtime, mut list, received, marker) = list_fixture(count);
            if count == 0 {
                assert_eq!((list.admitted_capacity, list.values.capacity()), (0, 0));
            } else {
                assert_eq!(list.admitted_capacity, 8);
                assert!(list.values.capacity() >= 8);
            }
            if count == 8 {
                list.values.try_reserve_exact(16 - count).unwrap();
                assert!(list.values.capacity() >= 16);
            }
            let old_physical = list.values.capacity();
            let old_logical = list.admitted_capacity;
            let retained = refs(&marker);
            allowance(&mut runtime, given_work, given_heap);
            let result = list.append_received(&mut runtime, received);
            assert_eq!(runtime.steps, 0);
            if success {
                result.unwrap();
                assert_eq!(runtime.allocated, MAX_HEAP);
                assert_eq!(list.values.len(), count + 1);
                assert_eq!(list.admitted_capacity, logical_after);
                assert!(list.values.capacity() >= logical_after);
                assert_eq!(refs(&marker), retained);
            } else {
                resource(result);
                assert_eq!(
                    runtime.allocated,
                    if given_work < work {
                        MAX_HEAP - given_heap
                    } else {
                        MAX_HEAP + 1
                    }
                );
                assert_eq!(list.values.len(), count);
                assert_eq!(list.admitted_capacity, old_logical);
                assert_eq!(list.values.capacity(), old_physical);
                assert_eq!(refs(&marker), retained - 1);
            }
            assert!(list.values.iter().all(|value| value == &marker));
            let pointer = list.values.as_ptr();
            let final_len = list.values.len();
            let before_drop = refs(&marker);
            let values = list.into_values();
            assert_eq!(values.as_ptr(), pointer);
            assert_eq!(refs(&marker), before_drop);
            drop(values);
            assert_eq!(refs(&marker), before_drop - final_len);
            quiet(&runtime);
        }
    }
}

#[test]
fn aggregate_error_list_rejects_malformed_logical_and_physical_state() {
    for invalid in 0..4 {
        let (mut runtime, mut list, received, marker) = list_fixture(usize::from(invalid == 3));
        match invalid {
            0 => {
                assert_eq!(list.values.capacity(), 0);
                list.admitted_capacity = 8;
            }
            1 => {
                list.values.try_reserve_exact(1).unwrap();
                list.admitted_capacity = 1;
            }
            2 => {
                list.values.try_reserve_exact(9).unwrap();
                list.admitted_capacity = 9;
            }
            3 => {
                assert_eq!(list.values.len(), 1);
                list.admitted_capacity = 0;
            }
            _ => unreachable!(),
        }
        let old = (
            list.values.len(),
            list.values.capacity(),
            list.admitted_capacity,
        );
        let retained = refs(&marker);
        allowance(&mut runtime, 8, 0);
        resource(list.append_received(&mut runtime, received));
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        assert_eq!(
            (
                list.values.len(),
                list.values.capacity(),
                list.admitted_capacity
            ),
            old
        );
        assert_eq!(refs(&marker), retained - 1);
        assert!(list.values.iter().all(|value| value == &marker));
        quiet(&runtime);
    }
}
