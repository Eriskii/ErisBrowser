//! Independent exact boundaries from the admitted native-call fee schedule.
use super::*;

fn fixture(kind: Kind) -> (Runtime, Document, Value, Record, Record) {
    let mut runtime = Runtime::try_new().unwrap();
    let mut doc = Document::parse("<p>kept</p>");
    let width = kind.width();
    runtime
        .execute(
            &format!(
                "var b=new ArrayBuffer({});var raw=new Uint8Array(b);var t=new {}(b,{width},4);",
                width * 6,
                kind.name()
            ),
            &mut doc,
        )
        .unwrap();
    let source = runtime.lookup(1, "t").unwrap().1;
    let raw = runtime.lookup(1, "raw").unwrap().1;
    let record = runtime.typed_array_record(&source).unwrap().unwrap();
    let raw_record = runtime.typed_array_record(&raw).unwrap().unwrap();
    let live = runtime.typed_array_live(raw_record).unwrap().unwrap();
    for (i, byte) in [165, 17, 34, 51, 68, 90].iter().enumerate() {
        runtime
            .buffer_view_write(live.buffer, i * width, &vec![*byte; width])
            .unwrap();
    }
    (runtime, doc, source, record, raw_record)
}

fn raw_bytes(runtime: &mut Runtime, record: Record) -> Vec<u8> {
    let remaining = runtime.steps;
    runtime.steps = MAX_STEPS;
    let live = runtime.typed_array_live(record).unwrap().unwrap();
    let bytes = (0..live.length)
        .map(|i| runtime.buffer_view_read(live.buffer, i, 1).unwrap()[0])
        .collect();
    runtime.steps = remaining;
    bytes
}

fn expected(width: usize, right: bool, stores: usize) -> Vec<u8> {
    let values = if right {
        match stores {
            0 => [17, 34, 51, 68],
            1 => [17, 34, 51, 51],
            2 => [17, 34, 34, 51],
            3 => [17, 17, 34, 51],
            _ => panic!("three complete stores"),
        }
    } else {
        match stores {
            0 => [17, 34, 51, 68],
            1 => [34, 34, 51, 68],
            2 => [34, 51, 51, 68],
            3 => [34, 51, 68, 68],
            _ => panic!("three complete stores"),
        }
    };
    [165, values[0], values[1], values[2], values[3], 90]
        .iter()
        .flat_map(|value| std::iter::repeat_n(*value, width))
        .collect()
}

fn clean(runtime: &Runtime) {
    assert!(runtime.frames.is_empty() && runtime.active_array_joins.is_empty());
    assert_eq!(
        (
            runtime.calls,
            runtime.eval_depth,
            runtime.stack_units,
            runtime.json_depth
        ),
        (0, 0, 0, 0)
    );
}

#[test]
fn every_work_cut_preserves_complete_scalars_and_overlap_direction_without_heap() {
    for kind in Kind::ALL {
        let width = kind.width();
        // Independent sum: loop8 + two checked offsets8 + raw read9+w
        // + whole raw write1+w. Defined-end setup is82, no author callback.
        let element = 26 + 2 * width;
        let total = 82 + 3 * element;
        for right in [false, true] {
            for allowance in 0..=total {
                let (mut runtime, mut doc, source, record, raw) = fixture(kind);
                let counts = (runtime.objects.len(), runtime.typed_arrays.records.len());
                let args = if right {
                    [Value::Number(1.0), Value::Number(0.0), Value::Number(3.0)]
                } else {
                    [Value::Number(0.0), Value::Number(1.0), Value::Number(4.0)]
                };
                runtime.steps = allowance;
                runtime.allocated = MAX_HEAP;
                let result =
                    runtime.typed_array_copy_within(source.clone(), record, &args, &mut doc);
                if allowance == total {
                    assert_eq!(result.unwrap(), source);
                } else {
                    assert!(result.unwrap_err().is_resource_limit());
                }
                let stores = allowance.saturating_sub(82) / element;
                assert_eq!(
                    raw_bytes(&mut runtime, raw),
                    expected(width, right, stores),
                    "{} right={right} cut={allowance}",
                    kind.name()
                );
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
                assert_eq!(
                    (runtime.objects.len(), runtime.typed_arrays.records.len()),
                    counts
                );
                clean(&runtime);
            }
        }
    }
}

#[test]
fn empty_range_and_undefined_end_have_separate_literal_endpoints() {
    for (args, total) in [
        (vec![Value::Number(4.0), Value::Number(0.0)], 49),
        (
            vec![Value::Number(0.0), Value::Number(3.0), Value::Number(2.0)],
            53,
        ),
        (vec![Value::Number(0.0), Value::Number(3.0)], 106), //78+28, one byte
    ] {
        for allowance in [total - 1, total, total + 7] {
            let (mut runtime, mut doc, source, record, raw) = fixture(Kind::Uint8);
            runtime.steps = allowance;
            runtime.allocated = MAX_HEAP;
            let result = runtime.typed_array_copy_within(source.clone(), record, &args, &mut doc);
            if allowance < total {
                assert!(result.unwrap_err().is_resource_limit());
            } else {
                assert_eq!(result.unwrap(), source);
            }
            let expected = if total == 106 && allowance >= total {
                vec![165, 68, 34, 51, 68, 90]
            } else {
                vec![165, 17, 34, 51, 68, 90]
            };
            assert_eq!(raw_bytes(&mut runtime, raw), expected);
            assert_eq!(
                (runtime.steps, runtime.allocated),
                (allowance.saturating_sub(total), MAX_HEAP)
            );
            clean(&runtime);
        }
    }
}

#[test]
fn copying_to_the_same_indices_still_admits_each_raw_store() {
    for kind in Kind::ALL {
        let total = 82 + 3 * (26 + 2 * kind.width());
        for allowance in [82, total - 1, total, total + 7] {
            let (mut runtime, mut doc, source, record, raw) = fixture(kind);
            runtime.steps = allowance;
            runtime.allocated = MAX_HEAP;
            let result = runtime.typed_array_copy_within(
                source.clone(),
                record,
                &[Value::Number(1.0), Value::Number(1.0), Value::Number(4.0)],
                &mut doc,
            );
            if allowance < total {
                assert!(result.unwrap_err().is_resource_limit());
            } else {
                assert_eq!(result.unwrap(), source);
            }
            assert_eq!(
                raw_bytes(&mut runtime, raw),
                expected(kind.width(), true, 0)
            );
            assert_eq!(
                (runtime.steps, runtime.allocated),
                (allowance.saturating_sub(total), MAX_HEAP)
            );
            clean(&runtime);
        }
    }
}

fn saved(runtime: &Runtime) -> Value {
    let owner = runtime.typed_arrays.prototype.unwrap();
    match &runtime.objects[owner].values[&PropertyKey::from("copyWithin")].value {
        PropertyValue::Data { value, .. } => value.clone(),
        _ => panic!("installed saved method"),
    }
}

#[test]
fn saved_native_name_preflight_precedes_authentication_and_keeps_metadata() {
    let (mut runtime, mut doc, _, _, _) = fixture(Kind::Uint8);
    let function = saved(&runtime);
    let Value::Native(identity) = function.clone() else {
        panic!("native method")
    };
    let bag = runtime.property_object(&function).unwrap();
    let metadata = format!("{:?}", runtime.objects[bag].values);
    let counts = (runtime.objects.len(), runtime.typed_arrays.records.len());
    for (allowance, room, charge) in [(25, 53, 0), (26, 53, 53), (26, 52, 53)] {
        runtime.steps = allowance;
        runtime.allocated = MAX_HEAP - room;
        let initial = runtime.allocated;
        let error = runtime
            .call(function.clone(), vec![], Value::Null, &mut doc)
            .unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!((runtime.steps, runtime.allocated), (0, initial + charge));
        assert_eq!(
            (runtime.objects.len(), runtime.typed_arrays.records.len()),
            counts
        );
        assert_eq!(format!("{:?}", runtime.objects[bag].values), metadata);
        let Value::Native(current) = &function else {
            unreachable!()
        };
        assert!(Rc::ptr_eq(current, &identity));
        assert_eq!(current.receiver, Value::Undefined);
        clean(&runtime);
    }
}

#[test]
fn complete_saved_call_has_literal_work_heap_and_partial_copy_boundaries() {
    for kind in Kind::ALL {
        // Fixture has raw then source brands: one4-work lookup reaches source.
        // call tick1 + name preflight25 + native tick1 + lookup4 + body82.
        let total = 113 + 3 * (26 + 2 * kind.width());
        for (allowance, room, success, stores) in [
            (total, 53, true, 3),
            (total - 1, 53, false, 2),
            (0, 53, false, 0),
            (total, 52, false, 0),
            (total, 0, false, 0),
        ] {
            let (mut runtime, mut doc, source, _, raw) = fixture(kind);
            assert_eq!(runtime.typed_arrays.records.len(), 2);
            assert!(runtime.frames.capacity() > 0);
            let function = saved(&runtime);
            let Value::Native(identity) = function.clone() else {
                panic!("native method")
            };
            let bag = runtime.property_object(&function).unwrap();
            let metadata = format!("{:?}", runtime.objects[bag].values);
            let counts = (runtime.objects.len(), runtime.typed_arrays.records.len());
            runtime.steps = allowance;
            runtime.allocated = MAX_HEAP - room;
            let result = runtime.call(
                function.clone(),
                vec![Value::Number(1.0), Value::Number(0.0), Value::Number(3.0)],
                source.clone(),
                &mut doc,
            );
            if success {
                assert_eq!(result.unwrap(), source);
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                if room == 53 {
                    assert_eq!(runtime.steps, 0);
                } else {
                    assert!(runtime.allocated > MAX_HEAP);
                }
            }
            assert_eq!(
                raw_bytes(&mut runtime, raw),
                expected(kind.width(), true, stores)
            );
            assert_eq!(
                (runtime.objects.len(), runtime.typed_arrays.records.len()),
                counts
            );
            assert_eq!(format!("{:?}", runtime.objects[bag].values), metadata);
            let Value::Native(current) = &function else {
                unreachable!()
            };
            assert!(Rc::ptr_eq(current, &identity));
            assert_eq!(current.receiver, Value::Undefined);
            clean(&runtime);
        }
    }
}
