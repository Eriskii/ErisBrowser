//! Parent-compatible Text metadata preservation and immutable-key sharing controls.
use super::*;

fn constructor_bag(runtime: &Runtime) -> usize {
    let Value::Native(native) = &runtime.environments[0].bindings["Text"].value else {
        panic!("Text constructor")
    };
    native.properties.unwrap().get()
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

fn cached_keys(runtime: &Runtime) -> [PropertyKey; 2] {
    let owner = constructor_bag(runtime);
    [
        named_key(runtime, owner, "length").clone(),
        named_key(runtime, owner, "name").clone(),
    ]
}

fn clean(runtime: &Runtime) {
    assert!(runtime.frames.is_empty());
    assert_eq!(
        (runtime.calls, runtime.eval_depth, runtime.stack_units),
        (0, 0, 0)
    );
    assert!(runtime.console.is_empty());
}

#[test]
fn text_metadata_keys_preserve_both_bags_before_sharing_constructor_payloads() {
    let runtime = Runtime::try_new().unwrap();
    let constructor = constructor_bag(&runtime);
    let keys = cached_keys(&runtime);
    let Some(Value::Object(prototype)) = runtime.objects[constructor].get("prototype") else {
        panic!("Text prototype")
    };
    let prototype_bag = &runtime.objects[*prototype];
    assert_eq!(prototype_bag.order.len(), 4);
    assert!(prototype_bag.order[0].as_string().is_none());
    assert_eq!(
        &prototype_bag.order[1..],
        &[
            PropertyKey::from("wholeText"),
            PropertyKey::from("splitText"),
            PropertyKey::from("constructor")
        ]
    );
    assert_eq!(prototype_bag.values.len(), 4);
    let whole = prototype_bag
        .values
        .get(&PropertyKey::from("wholeText"))
        .unwrap();
    let PropertyValue::Accessor {
        get: Value::Native(get),
        set: Value::Undefined,
    } = &whole.value
    else {
        panic!("wholeText getter without setter")
    };
    assert!(whole.enumerable && whole.configurable);
    let method = prototype_bag
        .values
        .get(&PropertyKey::from("splitText"))
        .unwrap();
    let PropertyValue::Data {
        value: Value::Native(split),
        writable: true,
    } = &method.value
    else {
        panic!("writable splitText method")
    };
    assert!(method.enumerable && method.configurable);
    assert!(!Rc::ptr_eq(get, split));
    let getter_bag = get.properties.unwrap().get();
    let split_bag = split.properties.unwrap().get();
    assert_eq!(split_bag, getter_bag + 1);
    for (native, suffix, display, arity) in [
        (get, "getWholeText", "get wholeText", 0),
        (split, "splitText", "splitText", 1),
    ] {
        assert_eq!(native.name, format!("DOM.Text.{suffix}"));
        assert_eq!(native.receiver, Value::Undefined);
        let owner = native.properties.unwrap().get();
        assert_ne!(owner, constructor);
        assert_ne!(owner, *prototype);
        let bag = &runtime.objects[owner];
        assert_eq!(
            bag.prototype,
            Some(Value::Function(runtime.function_prototype))
        );
        assert!(!bag.non_extensible);
        assert_eq!(bag.values.len(), 2);
        assert_eq!(bag.order, keys);
        for (name, expected) in [
            ("length", Value::Number(arity as f64)),
            ("name", Value::String(display.into())),
        ] {
            let property = bag.values.get(&PropertyKey::from(name)).unwrap();
            let PropertyValue::Data { value, writable } = &property.value else {
                panic!("Text function metadata data property")
            };
            assert_eq!(value, &expected);
            assert!(!*writable && !property.enumerable && property.configurable);
        }
    }
    assert_eq!(runtime.objects.len(), 755);
    assert_eq!(runtime.objects.capacity(), 755);
    assert_eq!(runtime.native_properties.len(), 300);
    assert_eq!(runtime.prototypes.len(), 26);
    assert_eq!(runtime.environments[0].bindings.len(), 213);
    clean(&runtime);

    // All preservation controls above precede the first intended parent failure.
    // Compare backing allocations, not merely equal UTF-16 key contents.
    for native in [get, split] {
        let owner = native.properties.unwrap().get();
        for (index, name) in ["length", "name"].into_iter().enumerate() {
            let expected = keys[index].as_string().unwrap().units().as_ptr();
            let stored = named_key(&runtime, owner, name)
                .as_string()
                .unwrap()
                .units()
                .as_ptr();
            let ordered = runtime.objects[owner].order[index]
                .as_string()
                .unwrap()
                .units()
                .as_ptr();
            assert!(
                std::ptr::eq(stored, expected),
                "{} stored {name}",
                native.name
            );
            assert!(
                std::ptr::eq(ordered, expected),
                "{} ordered {name}",
                native.name
            );
        }
    }
}

#[test]
fn text_metadata_key_sharing_preserves_saved_behavior_and_mutation_isolation_in_both_modes() {
    const SOURCE: &str = r#"
(function () {
    function check(value) { if (!value) throw new Error('Text metadata key cache'); }
    var whole = Object.getOwnPropertyDescriptor(Text.prototype, 'wholeText');
    var method = Object.getOwnPropertyDescriptor(Text.prototype, 'splitText');
    var get = whole.get, split = method.value;
    check(whole.set === undefined && whole.enumerable && whole.configurable && whole.writable === undefined);
    check(method.writable && method.enumerable && method.configurable);
    check(Object.getOwnPropertyNames(Text.prototype).join(',') === 'wholeText,splitText,constructor');
    var fs = [get, split], names = ['get wholeText', 'splitText'], lengths = [0, 1];
    check(get !== split && get === Object.getOwnPropertyDescriptor(Text.prototype, 'wholeText').get && split === Text.prototype.splitText);
    for (var i = 0; i < fs.length; i++) {
        var f = fs[i], n = Object.getOwnPropertyDescriptor(f, 'name'), l = Object.getOwnPropertyDescriptor(f, 'length');
        check(typeof f === 'function' && Object.getPrototypeOf(f) === Function.prototype);
        check(Object.getOwnPropertyNames(f).join(',') === 'length,name' && Object.getOwnPropertySymbols(f).length === 0);
        check(Object.getOwnPropertyDescriptor(f, 'prototype') === undefined);
        check(n.value === names[i] && !n.writable && !n.enumerable && n.configurable);
        check(l.value === lengths[i] && !l.writable && !l.enumerable && l.configurable);
        var rejected = false;
        try { Reflect.construct(f, []); } catch (error) { rejected = error instanceof TypeError; }
        check(rejected);
    }
    Object.defineProperty(get, 'name', {value: 'changed'});
    Object.defineProperty(get, 'length', {value: 99});
    check(get.name === 'changed' && get.length === 99);
    check(split.name === 'splitText' && split.length === 1);
    check(Text.name === 'Text' && Text.length === 0);
    check(delete get.name); check(delete get.length);
    check(Object.getOwnPropertyNames(get).length === 0);
    check(Object.getOwnPropertyDescriptor(get, 'name') === undefined && Object.getOwnPropertyDescriptor(get, 'length') === undefined);
    check(Object.getOwnPropertyNames(split).join(',') === 'length,name' && split.name === 'splitText' && split.length === 1);
    Object.defineProperty(split, 'name', {value: 'renamed'});
    Object.defineProperty(split, 'length', {value: 7});
    check(split.name === 'renamed' && split.length === 7 && Object.getOwnPropertyNames(get).length === 0);
    check(CharacterData.prototype.substringData.name === 'substringData' && CharacterData.prototype.substringData.length === 2);
    var text = new Text('ab'), tail = split.call(text, 1);
    check(tail !== text && text.data === 'a' && tail.data === 'b');
    check(get.call(text) === 'a' && get.call(tail) === 'b');
    var converted = 0, bad = false;
    try { split.call({}, {valueOf: function () { converted++; return 0; }}); } catch (error) { bad = error instanceof TypeError; }
    check(bad && converted === 0);
    bad = false; try { get.call({}); } catch (error) { bad = error instanceof TypeError; }
    check(bad);
    Object.defineProperty(Text.prototype, 'wholeText', {value: 'shadow', writable: true, enumerable: true, configurable: true});
    Object.defineProperty(Text.prototype, 'splitText', {value: function () { return 17; }, writable: true, enumerable: true, configurable: true});
    check(text.wholeText === 'shadow' && text.splitText(0) === 17 && get.call(text) === 'a');
    check(delete Text.prototype.wholeText); check(delete Text.prototype.splitText);
    check(text.wholeText === undefined && text.splitText === undefined);
    check(delete split.name); check(delete split.length);
    check(Object.getOwnPropertyNames(split).length === 0);
    var last = split.call(tail, 0);
    check(last !== tail && tail.data === '' && last.data === 'b');
    check(get.call(tail) === '' && get.call(last) === 'b');
    check(Text.name === 'Text' && Text.length === 0);
    return true;
})()
"#;
    // Each mode gets a fresh realm; the second must retain original descriptors
    // despite all metadata and owner deletions in the first realm.
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        let result = if strict {
            runtime.execute_strict(SOURCE, &mut document)
        } else {
            runtime.execute(SOURCE, &mut document)
        };
        assert_eq!(result.unwrap(), Value::Bool(true), "strict={strict}");
        clean(&runtime);
    }
}

#[test]
fn text_metadata_keys_raw_bootstrap_uses_100986_work_and_preserves_public_reset() {
    // Two removed key pairs save72 work and336 bytes; handoff8 plus clones4
    // leave a net60 work saving. AggregateError subsequently adds1103 startup
    // work and two bags; late Promise.any adds189 work and one bag.
    // Preserve the former raw inputs below as refusals.
    assert_eq!(MAX_STEPS, 100_000);
    let mut complete_heap = None;
    for (allowance, success, remaining) in [
        (99_693, false, 0),
        (99_694, false, 0),
        (100_000, false, 0),
        (100_796, false, 0),
        (100_797, false, 0),
        (100_985, false, 0),
        (100_986, true, 0),
        (1_000_000, true, 899_014),
    ] {
        let mut runtime = Runtime::uninitialized();
        runtime.steps = allowance;
        let result = runtime.finish_bootstrap();
        if !success {
            let error = match result {
                Ok(_) => panic!("under-budget Text key-cache bootstrap succeeded"),
                Err(error) => error,
            };
            assert!(error.is_resource_limit());
            continue;
        }
        let runtime = result.unwrap();
        assert_eq!(runtime.steps, remaining);
        assert_eq!(runtime.objects.len(), 755);
        assert_eq!(runtime.objects.capacity(), 755);
        assert_eq!(runtime.native_properties.len(), 300);
        assert_eq!(runtime.prototypes.len(), 26);
        assert_eq!(runtime.environments[0].bindings.len(), 213);
        assert!(runtime.environments[0].bindings.contains_key("Text"));
        assert!(runtime.environments[0].bindings.contains_key("Promise"));
        clean(&runtime);
        if let Some(heap) = complete_heap {
            assert_eq!(runtime.allocated, heap);
        }
        complete_heap = Some(runtime.allocated);
    }
    let runtime = Runtime::try_new().unwrap();
    assert_eq!(runtime.steps, 100_000);
    assert_eq!(Some(runtime.allocated), complete_heap);
    assert_eq!(runtime.objects.len(), 755);
    clean(&runtime);
}
