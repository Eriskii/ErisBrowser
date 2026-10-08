//! Candidate child of array_splice. Constants derive from the retained predecessor capture.
//! Setup and compiler costs are outside the three complete operation boundaries.
use super::*;

const KEYS: [&str; 6] = [
    "1.0",
    "+1",
    "1000000000000000000000",
    "0.0000001",
    "entries",
    "abcdefghijklmnopqrstuvwxyz",
];

fn setup(text: &str, scenario: usize) -> (Runtime, Document, Value, JsString, Value) {
    let mut runtime = Runtime::try_new().unwrap();
    let mut doc = Document::parse("<p>kept</p>");
    runtime
        .execute("var feeTarget = new Uint8Array([7,9]);", &mut doc)
        .unwrap();
    let target = runtime.environments[0].bindings["feeTarget"].value.clone();
    let key = JsString::from(text);
    runtime.steps = MAX_STEPS;
    if scenario != 0 {
        assert!(
            runtime
                .define_property_key(
                    &target,
                    &PropertyKey::String(key.clone()),
                    PropertyDescriptor::data_property(
                        Value::Number(21.0),
                        scenario == 1,
                        true,
                        true
                    ),
                    &mut doc,
                )
                .unwrap()
        );
    }
    let Value::Object(owner) = runtime.environments[0].bindings["Reflect"].value else {
        panic!("Reflect owner")
    };
    let PropertyValue::Data {
        value: function, ..
    } = &runtime.objects[owner].values[&PropertyKey::from("set")].value
    else {
        panic!("saved Reflect.set")
    };
    let function = function.clone();
    (runtime, doc, target, key, function)
}

fn operation(
    runtime: &mut Runtime,
    doc: &mut Document,
    target: &Value,
    key: &JsString,
    function: &Value,
    route: usize,
    scenario: usize,
) -> Result<()> {
    let value = Value::Number(37.0);
    match route {
        0 => {
            let result = runtime.call(
                function.clone(),
                vec![target.clone(), Value::String(key.clone()), value],
                Value::Undefined,
                doc,
            )?;
            assert_eq!(result, Value::Bool(scenario != 2));
            Ok(())
        }
        1 => runtime.set_key_strict(target.clone(), key, value, false, doc),
        2 => match runtime.splice_set(target, key.clone(), value, doc) {
            Err(error) if scenario == 2 && error.kind == ErrorKind::Runtime("TypeError") => Ok(()),
            result => result,
        },
        _ => unreachable!(),
    }
}

fn outcome(runtime: &Runtime, target: &Value, key: &JsString, scenario: usize) {
    let Value::Object(id) = target else {
        panic!("view identity")
    };
    let property = runtime.objects[*id]
        .values
        .get(&PropertyKey::String(key.clone()))
        .unwrap();
    assert!(property.enumerable && property.configurable);
    match &property.value {
        PropertyValue::Data { value, writable } => {
            assert_eq!(
                *value,
                Value::Number(if scenario == 2 { 21.0 } else { 37.0 })
            );
            assert_eq!(*writable, scenario != 2);
        }
        _ => panic!("literal data property"),
    }
    assert_eq!(
        runtime.objects[*id].order,
        vec![PropertyKey::String(key.clone())]
    );
    assert!(runtime.frames.is_empty());
    assert!(runtime.active_array_joins.is_empty());
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

// key, route, scenario, old work, old heap, classifier work, classifier heap
const BEFORE: &[(usize, usize, usize, usize, usize, usize, usize)] = &[
    (0, 0, 0, 1472, 4559, 266, 1062),
    (0, 0, 1, 1141, 4291, 266, 1062),
    (0, 0, 2, 584, 2167, 266, 1062),
    (0, 1, 0, 569, 2392, 266, 1062),
    (0, 1, 1, 542, 2124, 266, 1062),
    (0, 1, 2, 542, 2124, 266, 1062),
    (0, 2, 0, 1210, 4836, 266, 1062),
    (0, 2, 1, 1153, 4504, 266, 1062),
    (0, 2, 2, 548, 2124, 266, 1062),
    (1, 0, 0, 1322, 4547, 235, 1060),
    (1, 0, 1, 1015, 4283, 235, 1060),
    (1, 0, 2, 521, 2163, 235, 1060),
    (1, 1, 0, 507, 2384, 235, 1060),
    (1, 1, 1, 480, 2120, 235, 1060),
    (1, 1, 2, 480, 2120, 235, 1060),
    (1, 2, 0, 1077, 4824, 235, 1060),
    (1, 2, 1, 1023, 4496, 235, 1060),
    (1, 2, 2, 485, 2120, 235, 1060),
    (2, 0, 0, 4024, 4787, 780, 1100),
    (2, 0, 1, 3237, 4443, 780, 1100),
    (2, 0, 2, 1631, 2243, 780, 1100),
    (2, 1, 0, 1603, 2544, 780, 1100),
    (2, 1, 1, 1573, 2200, 780, 1100),
    (2, 1, 2, 1573, 2200, 780, 1100),
    (2, 2, 0, 3481, 5064, 780, 1100),
    (2, 2, 1, 3345, 4656, 780, 1100),
    (2, 2, 2, 1597, 2200, 780, 1100),
    (3, 0, 0, 1853, 4631, 322, 1074),
    (3, 0, 1, 1378, 4339, 322, 1074),
    (3, 0, 2, 702, 2191, 322, 1074),
    (3, 1, 0, 682, 2440, 322, 1074),
    (3, 1, 1, 655, 2148, 322, 1074),
    (3, 1, 2, 655, 2148, 322, 1074),
    (3, 2, 0, 1510, 4908, 322, 1074),
    (3, 2, 1, 1424, 4552, 322, 1074),
    (3, 2, 2, 667, 2148, 322, 1074),
    (4, 0, 0, 710, 327, 88, 0),
    (4, 0, 1, 437, 43, 88, 0),
    (4, 0, 2, 232, 43, 88, 0),
    (4, 1, 0, 204, 284, 88, 0),
    (4, 1, 1, 186, 0, 88, 0),
    (4, 1, 2, 186, 0, 88, 0),
    (4, 2, 0, 509, 604, 88, 0),
    (4, 2, 1, 465, 256, 88, 0),
    (4, 2, 2, 196, 0, 88, 0),
    (5, 0, 0, 1041, 403, 8, 0),
    (5, 0, 1, 158, 43, 8, 0),
    (5, 0, 2, 91, 43, 8, 0),
    (5, 1, 0, 60, 360, 8, 0),
    (5, 1, 1, 30, 0, 8, 0),
    (5, 1, 2, 30, 0, 8, 0),
    (5, 2, 0, 451, 680, 8, 0),
    (5, 2, 1, 292, 256, 8, 0),
    (5, 2, 2, 58, 0, 8, 0),
];

fn expected(row: &(usize, usize, usize, usize, usize, usize, usize)) -> (usize, usize) {
    let (key_id, route, scenario, work, heap, c, h) = *row;
    let removed = if route == 1 || scenario == 2 { 1 } else { 3 };
    // Preserve the retained predecessor rows and original SetKey derivation.
    // Exactly one full classification remains after immutable-key reuse.
    let reused_work = work - removed * c + 8 + removed * 4;
    let reused_heap = heap - removed * h;
    let (next_c, next_h) = [
        (274, 1062),
        (16, 0),
        (788, 1100),
        (330, 1074),
        (16, 0),
        (8, 0),
    ][key_id];
    // The four search members, fill and reverse change the shared owner13->19.
    // Only absent-own scenario0 reaches it. The existing Reflect/splice
    // lookup tariffs each traverse that owner once; assignment has no fee
    // dependent on its size. All KEYS are ASCII, so byte length is UTF16 length.
    let owner_growth = if scenario == 0 {
        match route {
            0 => 6 * (1 + KEYS[key_id].len()),
            2 => 6 * (1 + KEYS[key_id].len() / 8),
            _ => 0,
        }
    } else {
        0
    };
    (
        reused_work - c + next_c + owner_growth,
        reused_heap - h + next_h,
    )
}

#[test]
fn classified_key_full_operation_fees_match_predecessor_contract() {
    for row in BEFORE {
        let (key_id, route, scenario, _, _, _, _) = *row;
        let (work, heap) = expected(row);
        let (mut runtime, mut doc, target, key, function) = setup(KEYS[key_id], scenario);
        runtime.steps = work;
        runtime.allocated = MAX_HEAP - heap;
        operation(
            &mut runtime,
            &mut doc,
            &target,
            &key,
            &function,
            route,
            scenario,
        )
        .unwrap();
        assert_eq!(
            runtime.steps, 0,
            "key={key_id} route={route} scenario={scenario}"
        );
        assert_eq!(runtime.allocated, MAX_HEAP);
        outcome(&runtime, &target, &key, scenario);
    }
}

#[test]
fn classified_key_full_operation_one_short_endpoints() {
    for row in BEFORE {
        let (key_id, route, scenario, _, _, _, _) = *row;
        let (work, heap) = expected(row);
        for heap_cut in [false, true] {
            if heap_cut && heap == 0 {
                continue;
            } // No reached allocation to refuse.
            let (mut runtime, mut doc, target, key, function) = setup(KEYS[key_id], scenario);
            runtime.steps = if heap_cut { work } else { work - 1 };
            runtime.allocated = if heap_cut { MAX_HEAP - heap + 1 } else { 0 };
            let result = operation(
                &mut runtime,
                &mut doc,
                &target,
                &key,
                &function,
                route,
                scenario,
            );
            assert_eq!(
                result.expect_err("one-short refusal").kind,
                ErrorKind::Resource,
                "key={key_id} route={route} scenario={scenario} heap_cut={heap_cut}"
            );
            assert!(runtime.frames.is_empty());
            assert!(runtime.active_array_joins.is_empty());
            assert_eq!(
                (
                    runtime.calls,
                    runtime.eval_depth,
                    runtime.stack_units,
                    runtime.json_depth
                ),
                (0, 0, 0, 0)
            );
            // A native continuation can fail after its store. Preserve either
            // reached prefix rather than asserting whole-call rollback.
            let Value::Object(id) = target else {
                panic!("view identity")
            };
            let object = &runtime.objects[id];
            assert!(object.values.len() <= 1);
            if let Some(property) = object.values.get(&PropertyKey::String(key.clone())) {
                let PropertyValue::Data { value, writable } = &property.value else {
                    panic!("data")
                };
                assert!(property.enumerable && property.configurable);
                assert_eq!(*writable, scenario != 2);
                if scenario == 2 {
                    assert_eq!(*value, Value::Number(21.0));
                } else {
                    assert!(*value == Value::Number(21.0) || *value == Value::Number(37.0));
                }
            } else {
                assert_eq!(scenario, 0);
            }
            runtime.steps = MAX_STEPS;
            runtime.allocated = 0;
            assert_eq!(
                runtime
                    .get_key(Value::Object(id), &JsString::from("0"), &mut doc)
                    .unwrap(),
                Value::Number(7.0)
            );
            assert_eq!(
                runtime
                    .get_key(Value::Object(id), &JsString::from("1"), &mut doc)
                    .unwrap(),
                Value::Number(9.0)
            );
        }
    }
}
