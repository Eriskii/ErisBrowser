use super::*;

fn bag(value: &Value) -> usize {
    let Value::Object(id) = value else {
        panic!("ordinary fixture")
    };
    *id
}

fn fresh() -> (Runtime, Document, Value, Value, PropertyKey) {
    let mut runtime = Runtime::try_new().unwrap();
    let base = runtime
        .object_ordered([("x".into(), Value::Number(1.0))])
        .unwrap();
    let receiver = runtime
        .object_ordered([("x".into(), Value::Number(2.0))])
        .unwrap();
    for value in [&base, &receiver] {
        runtime.objects[bag(value)].prototype = None;
        assert_eq!(runtime.objects[bag(value)].values.len(), 1);
        assert!(runtime.objects[bag(value)].parameter_map.is_empty());
        let before = runtime.steps;
        assert!(matches!(
            runtime
                .typed_array_own_property(value, &"x".into(), false)
                .unwrap(),
            typed_array::Exotic::Ordinary
        ));
        // A nonempty TypedArray registry would charge a binary-search probe.
        assert_eq!(runtime.steps, before);
    }
    (
        runtime,
        Document::parse(""),
        base,
        receiver,
        PropertyKey::from("x"),
    )
}

fn clean(runtime: &Runtime) {
    assert!(runtime.frames.is_empty());
    assert_eq!(
        (
            runtime.calls,
            runtime.stack_units,
            runtime.eval_depth,
            runtime.json_depth
        ),
        (0, 0, 0, 0)
    );
}

fn global(runtime: &Runtime, name: &str) -> Value {
    runtime.environments[0].bindings[name].value.clone()
}

fn accessor(runtime: &mut Runtime, owner: &Value, key: &PropertyKey, get: Value, set: Value) {
    runtime.objects[bag(owner)]
        .values
        .get_mut(key)
        .unwrap()
        .value = PropertyValue::Accessor { get, set };
}

#[test]
fn super_property_wrapper_entry_and_data_cuts_preserve_distinct_objects() {
    for steps in [5, 21, 22] {
        let (mut runtime, mut doc, base, receiver, key) = fresh();
        let heap = runtime.allocated;
        runtime.steps = steps;
        let result = runtime.get_super_property_key(&base, &key, &receiver, &mut doc);
        if steps == 22 {
            assert_eq!(result.unwrap(), Value::Number(1.0));
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, heap);
        assert_eq!(
            runtime.objects[bag(&receiver)].get("x"),
            Some(&Value::Number(2.0))
        );
        clean(&runtime);
    }
    for steps in [5, 6] {
        let (mut runtime, mut doc, _, receiver, key) = fresh();
        runtime.steps = steps;
        let result = runtime.get_super_property_key(&Value::Null, &key, &receiver, &mut doc);
        if steps == 5 {
            assert!(result.unwrap_err().is_resource_limit());
        } else {
            assert_eq!(result.unwrap_err().name(), "TypeError");
        }
        assert_eq!(runtime.steps, 0);
    }
    for steps in [6, 39, 40] {
        let (mut runtime, mut doc, base, receiver, key) = fresh();
        let heap = runtime.allocated;
        runtime.steps = steps;
        let result = runtime.set_super_property_key(
            &base,
            &key,
            &receiver,
            Value::Number(7.0),
            true,
            &mut doc,
        );
        if steps == 40 {
            result.unwrap();
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, heap);
        assert_eq!(
            runtime.objects[bag(&base)].get("x"),
            Some(&Value::Number(1.0))
        );
        assert_eq!(
            runtime.objects[bag(&receiver)].get("x"),
            Some(&Value::Number(if steps == 40 { 7.0 } else { 2.0 }))
        );
        clean(&runtime);
    }
}

#[test]
fn super_property_getter_copy_cut_and_primitive_receiver_are_explicit() {
    for steps in [22, 23] {
        let (mut runtime, mut doc, base, receiver, key) = fresh();
        let getter = runtime
            .execute(
                "var helperCalls=0; (function(){'use strict';helperCalls++;return this;});",
                &mut doc,
            )
            .unwrap();
        accessor(&mut runtime, &base, &key, getter, Value::Undefined);
        let heap = runtime.allocated;
        runtime.steps = steps;
        assert!(
            runtime
                .get_super_property_key(&base, &key, &receiver, &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, heap);
        assert_eq!(global(&runtime, "helperCalls"), Value::Number(0.0));
        clean(&runtime);
    }
    for receiver in [Value::Undefined, Value::Null, Value::Number(9.0)] {
        let (mut runtime, mut doc, base, _, key) = fresh();
        let getter = runtime
            .execute("(function(){'use strict';return this;});", &mut doc)
            .unwrap();
        accessor(&mut runtime, &base, &key, getter, Value::Undefined);
        assert_eq!(
            runtime
                .get_super_property_key(&base, &key, &receiver, &mut doc)
                .unwrap(),
            receiver
        );
        clean(&runtime);
    }
}

#[test]
fn super_property_data_set_never_calls_receiver_accessors_or_prototype_setters() {
    for (steps, strict) in [(38, true), (39, true), (39, false)] {
        let (mut runtime, mut doc, base, receiver, key) = fresh();
        let setter = runtime
            .execute(
                "var helperCalls=0; (function(v){helperCalls++;});",
                &mut doc,
            )
            .unwrap();
        accessor(
            &mut runtime,
            &receiver,
            &key,
            Value::Undefined,
            setter.clone(),
        );
        let heap = runtime.allocated;
        runtime.steps = steps;
        let result = runtime.set_super_property_key(
            &base,
            &key,
            &receiver,
            Value::Number(7.0),
            strict,
            &mut doc,
        );
        if steps == 38 {
            assert!(result.unwrap_err().is_resource_limit());
        } else if strict {
            assert_eq!(result.unwrap_err().name(), "TypeError");
        } else {
            result.unwrap();
        }
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, heap);
        assert_eq!(global(&runtime, "helperCalls"), Value::Number(0.0));
        let PropertyValue::Accessor { set, .. } =
            &runtime.objects[bag(&receiver)].values[&key].value
        else {
            panic!("receiver descriptor changed")
        };
        assert_eq!(set, &setter);
        clean(&runtime);
    }
    let (mut runtime, mut doc, base, receiver, key) = fresh();
    let setter = runtime
        .execute(
            "var helperCalls=0; (function(v){helperCalls++;});",
            &mut doc,
        )
        .unwrap();
    let prototype = runtime
        .object_ordered([("x".into(), Value::Undefined)])
        .unwrap();
    accessor(&mut runtime, &prototype, &key, Value::Undefined, setter);
    runtime.objects[bag(&receiver)].remove(&key);
    runtime.objects[bag(&receiver)].prototype = Some(prototype);
    runtime
        .set_super_property_key(&base, &key, &receiver, Value::Number(7.0), true, &mut doc)
        .unwrap();
    assert_eq!(global(&runtime, "helperCalls"), Value::Number(0.0));
    assert_eq!(
        runtime.objects[bag(&receiver)].get("x"),
        Some(&Value::Number(7.0))
    );
}

#[test]
fn super_property_base_setter_keeps_receiver_payload_and_abrupt_value() {
    for receiver in [Value::Undefined, Value::Null, Value::Number(9.0)] {
        let (mut runtime, mut doc, base, _, key) = fresh();
        let setter = runtime.execute("var helperSeen,helperPayload; (function(v){'use strict';helperSeen=this;helperPayload=v;});", &mut doc).unwrap();
        accessor(&mut runtime, &base, &key, Value::Undefined, setter);
        let payload = runtime.object_ordered([]).unwrap();
        runtime
            .set_super_property_key(&base, &key, &receiver, payload.clone(), true, &mut doc)
            .unwrap();
        assert_eq!(global(&runtime, "helperSeen"), receiver);
        assert_eq!(global(&runtime, "helperPayload"), payload);
        clean(&runtime);
    }
    let (mut runtime, mut doc, base, receiver, key) = fresh();
    let setter = runtime
        .execute(
            "var helperReason={}; (function(v){throw helperReason;});",
            &mut doc,
        )
        .unwrap();
    accessor(&mut runtime, &base, &key, Value::Undefined, setter);
    let error = runtime
        .set_super_property_key(&base, &key, &receiver, Value::Number(7.0), false, &mut doc)
        .unwrap_err();
    let ErrorKind::Thrown(value) = error.kind else {
        panic!("original author throw")
    };
    assert_eq!(value, global(&runtime, "helperReason"));
    clean(&runtime);
    for strict in [false, true] {
        let (mut runtime, mut doc, base, _, key) = fresh();
        runtime.steps = 25;
        let result = runtime.set_super_property_key(
            &base,
            &key,
            &Value::Number(9.0),
            Value::Number(7.0),
            strict,
            &mut doc,
        );
        if strict {
            assert_eq!(result.unwrap_err().name(), "TypeError");
        } else {
            result.unwrap();
        }
        assert_eq!(runtime.steps, 0);
        assert_eq!(
            runtime.objects[bag(&base)].get("x"),
            Some(&Value::Number(1.0))
        );
    }
}

#[test]
fn super_property_setter_argument_work_and_heap_refuse_before_author_entry() {
    let bytes = 32 + std::mem::size_of::<Value>();
    for (steps, room, added) in [(28, 0, 0), (29, bytes - 1, bytes), (29, bytes, bytes)] {
        let (mut runtime, mut doc, base, receiver, key) = fresh();
        let setter = runtime
            .execute(
                "var helperCalls=0; (function(v){helperCalls++;});",
                &mut doc,
            )
            .unwrap();
        accessor(&mut runtime, &base, &key, Value::Undefined, setter);
        runtime.steps = steps;
        runtime.allocated = MAX_HEAP - room;
        let heap = runtime.allocated;
        assert!(
            runtime
                .set_super_property_key(&base, &key, &receiver, Value::Number(7.0), true, &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, heap + added);
        assert_eq!(global(&runtime, "helperCalls"), Value::Number(0.0));
        assert_eq!(
            runtime.objects[bag(&receiver)].get("x"),
            Some(&Value::Number(2.0))
        );
        clean(&runtime);
    }
}

#[test]
fn super_property_typed_numeric_targets_keep_actual_receiver_rules() {
    let (mut runtime, mut doc, _, _, _) = fresh();
    runtime.execute("var helperCalls=0, view=new Uint8Array(1), targetReceiver={}, rhs={valueOf:function(){helperCalls++;return 9;}};view[0]=17;Object.setPrototypeOf(view,{'2':99});", &mut doc).unwrap();
    let view = global(&runtime, "view");
    let receiver = global(&runtime, "targetReceiver");
    let rhs = global(&runtime, "rhs");
    for key in ["-0", "1"] {
        runtime
            .set_super_property_key(
                &view,
                &PropertyKey::from(key),
                &receiver,
                rhs.clone(),
                true,
                &mut doc,
            )
            .unwrap();
    }
    assert_eq!(global(&runtime, "helperCalls"), Value::Number(0.0));
    assert!(runtime.objects[bag(&receiver)].values.is_empty());
    assert_eq!(
        runtime
            .get_super_property_key(&view, &PropertyKey::from("2"), &receiver, &mut doc)
            .unwrap(),
        Value::Undefined
    );
    runtime
        .set_super_property_key(
            &view,
            &PropertyKey::from("0"),
            &receiver,
            rhs.clone(),
            true,
            &mut doc,
        )
        .unwrap();
    assert_eq!(runtime.objects[bag(&receiver)].get("0"), Some(&rhs));
    assert_eq!(global(&runtime, "helperCalls"), Value::Number(0.0));
    assert_eq!(
        runtime
            .get_super_property_key(&view, &PropertyKey::from("0"), &receiver, &mut doc)
            .unwrap(),
        Value::Number(17.0)
    );
    runtime
        .set_super_property_key(&view, &PropertyKey::from("1"), &view, rhs, true, &mut doc)
        .unwrap();
    assert_eq!(global(&runtime, "helperCalls"), Value::Number(1.0));
    clean(&runtime);
}

#[test]
fn super_property_dom_branding_and_explicit_fallback_are_preserved() {
    let (mut runtime, mut doc, _, receiver, _) = fresh();
    let target = Value::Node(doc.create_text_node("target"));
    let actual = Value::Node(doc.create_text_node("receiver"));
    let key = PropertyKey::from("nodeValue");
    assert_eq!(
        runtime
            .get_super_property_key(&target, &key, &actual, &mut doc)
            .unwrap(),
        Value::String("receiver".into())
    );
    assert_eq!(
        runtime
            .get_super_property_key(&target, &key, &receiver, &mut doc)
            .unwrap_err()
            .name(),
        "TypeError"
    );
    runtime
        .set_super_property_key(
            &target,
            &key,
            &actual,
            Value::String("changed".into()),
            true,
            &mut doc,
        )
        .unwrap();
    assert_eq!(
        runtime
            .get_property_key(target.clone(), &key, &mut doc)
            .unwrap(),
        Value::String("target".into())
    );
    assert_eq!(
        runtime.get_property_key(actual, &key, &mut doc).unwrap(),
        Value::String("changed".into())
    );
    assert!(
        runtime
            .get_super_property_key(&target, &PropertyKey::from("nodeType"), &receiver, &mut doc)
            .unwrap_err()
            .is_unsupported()
    );
    clean(&runtime);
}

#[test]
fn super_property_window_own_storage_omits_lexicals_and_preserves_receiver() {
    let (mut runtime, mut doc, _, receiver, _) = fresh();
    runtime.execute("var helperVisible=7;let helperLexical=9;Object.defineProperty(globalThis,'helperAccessor',{get:function(){'use strict';return this;},configurable:true});", &mut doc).unwrap();
    assert_eq!(
        runtime
            .get_super_property_key(
                &Value::Window,
                &PropertyKey::from("helperVisible"),
                &receiver,
                &mut doc
            )
            .unwrap(),
        Value::Number(7.0)
    );
    assert_eq!(
        runtime
            .get_super_property_key(
                &Value::Window,
                &PropertyKey::from("helperLexical"),
                &receiver,
                &mut doc
            )
            .unwrap(),
        Value::Undefined
    );
    assert_eq!(
        runtime
            .get_super_property_key(
                &Value::Window,
                &PropertyKey::from("helperAccessor"),
                &receiver,
                &mut doc
            )
            .unwrap(),
        receiver
    );
    assert!(
        runtime
            .get_super_property_key(
                &Value::Window,
                &PropertyKey::from("onclick"),
                &receiver,
                &mut doc
            )
            .unwrap_err()
            .is_unsupported()
    );
    clean(&runtime);
}
