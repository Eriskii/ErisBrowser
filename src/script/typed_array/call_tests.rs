use super::*;

fn fresh() -> (Runtime, Document) {
    (Runtime::try_new().unwrap(), Document::parse("<p>kept</p>"))
}

fn clean(runtime: &Runtime) {
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

// Read genuine installed descriptors during test setup. No forged Native or
// direct typed_array_native call can bypass the machine dispatch under test.
fn saved(runtime: &Runtime, name: &str) -> Value {
    let (bag, key) = if name == "species" {
        (
            runtime
                .property_object(runtime.typed_arrays.intrinsic.as_ref().unwrap())
                .unwrap(),
            runtime.well_known_key("species"),
        )
    } else {
        (
            runtime.typed_arrays.prototype.unwrap(),
            if name == "tag" {
                runtime.well_known_key("toStringTag")
            } else {
                JsString::from(name).into()
            },
        )
    };
    match &runtime.objects[bag].values[&key].value {
        PropertyValue::Accessor { get, .. } => get.clone(),
        PropertyValue::Data { value, .. } => value.clone(),
    }
}

#[test]
fn typed_array_saved_native_copy_admission_precedes_dispatch_and_cleans_guards() {
    for name in [
        "buffer",
        "byteLength",
        "byteOffset",
        "length",
        "tag",
        "species",
        "keys",
        "values",
        "entries",
    ] {
        // These prices are the independent public call-admission contract, not
        // a measurement of the helper being checked. The Runtime entry tick is
        // separate; an exact preflight leaves no step for intrinsic dispatch.
        let (mut runtime, mut doc) = fresh();
        let function = saved(&runtime, name);
        let Value::Native(native) = &function else {
            panic!("installed Native")
        };
        assert!(native.name.starts_with("TypedArray."));
        assert_eq!(native.receiver, Value::Undefined);
        assert!(native.properties.is_some());
        let work = 4 + native.name.len();
        let heap = 32 + native.name.len();
        let bag = runtime.property_object(&function).unwrap();
        let metadata = format!("{:?}", runtime.objects[bag].values);
        let original = (runtime.objects.len(), runtime.typed_arrays.records.len());
        assert!(runtime.frames.capacity() > 0);
        let identity = native.clone();
        for (steps, room, charged) in [
            (work, heap, 0),
            (1 + work, heap, heap),
            (1 + work, heap - 1, heap),
        ] {
            runtime.steps = steps;
            runtime.allocated = MAX_HEAP - room;
            let before = runtime.allocated;
            let error = runtime
                .call(function.clone(), Vec::new(), Value::Null, &mut doc)
                .unwrap_err();
            assert!(error.is_resource_limit(), "{name}");
            assert_eq!(runtime.steps, 0, "{name}");
            assert_eq!(runtime.allocated, before + charged, "{name}");
            assert_eq!(
                (runtime.objects.len(), runtime.typed_arrays.records.len()),
                original
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

#[test]
fn typed_array_species_machine_call_exact_and_one_short_preserves_receiver_identity() {
    let (mut measured, mut doc) = fresh();
    let function = saved(&measured, "species");
    let receiver = measured.typed_arrays.constructors[Kind::Uint8.index()]
        .clone()
        .unwrap();
    let start = (measured.steps, measured.allocated);
    assert_eq!(
        measured
            .call(function, Vec::new(), receiver.clone(), &mut doc)
            .unwrap(),
        receiver
    );
    let (work, heap) = (start.0 - measured.steps, measured.allocated - start.1);
    assert!(work > 1 && heap > 0);
    clean(&measured);
    for (steps, room, success) in [
        (work, heap, true),
        (work - 1, heap, false),
        (work, heap - 1, false),
    ] {
        let (mut runtime, mut doc) = fresh();
        let function = saved(&runtime, "species");
        let receiver = runtime.typed_arrays.constructors[Kind::Uint8.index()]
            .clone()
            .unwrap();
        let objects = runtime.objects.len();
        runtime.steps = steps;
        runtime.allocated = MAX_HEAP - room;
        let result = runtime.call(function, Vec::new(), receiver.clone(), &mut doc);
        if success {
            assert_eq!(result.unwrap(), receiver);
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        assert_eq!(runtime.objects.len(), objects);
        assert!(runtime.typed_arrays.records.is_empty());
        clean(&runtime);
    }
}

fn authored(strict: bool) -> (Runtime, Document, Value, Value) {
    let (mut runtime, mut doc) = fresh();
    let source = format!(
        "var effect=0;var holder={{getter:Object.getOwnPropertyDescriptor(Object.getPrototypeOf(Uint8Array),Symbol.species).get}};(function(){{{}effect=1;return holder.getter();}})",
        if strict { "'use strict';" } else { "" }
    );
    let function = runtime.execute(&source, &mut doc).unwrap();
    let receiver = runtime.environments[0].bindings["holder"].value.clone();
    assert!(matches!(function, Value::Function(_)));
    clean(&runtime);
    (runtime, doc, function, receiver)
}

#[test]
fn typed_array_authored_call_heap_refusal_retains_effect_and_unwinds_machine_guard() {
    for strict in [false, true] {
        let (mut measured, mut doc, function, receiver) = authored(strict);
        let start = (measured.steps, measured.allocated);
        assert_eq!(
            measured
                .call(function, Vec::new(), Value::Undefined, &mut doc)
                .unwrap(),
            receiver
        );
        let (work, heap) = (start.0 - measured.steps, measured.allocated - start.1);
        assert!(heap >= 32 + "TypedArray.getSpecies".len());
        clean(&measured);
        for (room, success) in [(heap, true), (heap - 1, false)] {
            let (mut runtime, mut doc, function, receiver) = authored(strict);
            let native = saved(&runtime, "species");
            let bag = runtime.property_object(&native).unwrap();
            let metadata = format!("{:?}", runtime.objects[bag].values);
            runtime.steps = work;
            runtime.allocated = MAX_HEAP - room;
            let result = runtime.call(function, Vec::new(), Value::Undefined, &mut doc);
            if success {
                assert_eq!(result.unwrap(), receiver);
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert_eq!(runtime.allocated, MAX_HEAP + 1);
            }
            assert_eq!(
                runtime.environments[0].bindings["effect"].value,
                Value::Number(1.0)
            );
            assert_eq!(format!("{:?}", runtime.objects[bag].values), metadata);
            assert!(runtime.typed_arrays.records.is_empty());
            clean(&runtime);
        }
    }
}
