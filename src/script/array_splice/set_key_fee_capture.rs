//! Root registers this unchanged as a child of array_splice on predecessor and candidate.
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

#[test]
fn classified_key_complete_operation_fee_capture() {
    for (key_id, text) in KEYS.iter().enumerate() {
        let (mut classifier, _, _, key, _) = setup(text, 0);
        classifier.steps = MAX_STEPS;
        classifier.allocated = 0;
        let _ = classifier.typed_array_index(&key).unwrap();
        let class_work = MAX_STEPS - classifier.steps;
        let class_heap = classifier.allocated;
        for route in 0..3 {
            for scenario in 0..3 {
                let (mut runtime, mut doc, target, key, function) = setup(text, scenario);
                runtime.steps = MAX_STEPS;
                runtime.allocated = 0;
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
                let work = MAX_STEPS - runtime.steps;
                let heap = runtime.allocated;
                outcome(&runtime, &target, &key, scenario);
                println!(
                    "SET_KEY_FEES key={key_id} route={route} scenario={scenario} work={work} heap={heap} classifier_work={class_work} classifier_heap={class_heap}"
                );
                // The element payload is independent of the measured expando operation.
                runtime.steps = MAX_STEPS;
                assert_eq!(
                    runtime
                        .get_key(target.clone(), &JsString::from("0"), &mut doc)
                        .unwrap(),
                    Value::Number(7.0)
                );
                assert_eq!(
                    runtime
                        .get_key(target.clone(), &JsString::from("1"), &mut doc)
                        .unwrap(),
                    Value::Number(9.0)
                );
            }
        }
    }
}
