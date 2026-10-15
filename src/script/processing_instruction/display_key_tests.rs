//! Parent-compatible retained-display and borrowed-title-key controls.
use super::key_cache_tests::CASES;
use super::*;

fn constructor_bag(runtime: &Runtime, name: &str) -> usize {
    let Value::Native(native) = &runtime.environments[0].bindings[name].value else {
        panic!("interface constructor");
    };
    native.properties.unwrap().get()
}
fn prototype(runtime: &Runtime, name: &str) -> usize {
    let Some(Value::Object(id)) = runtime.objects[constructor_bag(runtime, name)].get("prototype")
    else {
        panic!("interface prototype");
    };
    *id
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
fn text(value: &Value) -> &JsString {
    let Value::String(text) = value else {
        panic!("string metadata");
    };
    text
}
fn metadata(runtime: &Runtime, native: &Native, display: &str, arity: usize) {
    let bag = &runtime.objects[native.properties.unwrap().get()];
    assert_eq!(
        bag.prototype,
        Some(Value::Function(runtime.function_prototype))
    );
    assert_eq!(
        bag.order,
        [PropertyKey::from("length"), PropertyKey::from("name")]
    );
    assert_eq!(bag.values.len(), 2);
    assert_eq!(bag.get("length"), Some(&Value::Number(arity as f64)));
    assert_eq!(bag.get("name"), Some(&Value::String(display.into())));
    assert_eq!(native.receiver, Value::Undefined);
    for property in bag.values.values() {
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
fn retained_display_keys_share_six_owner_names_and_the_character_data_length_key() {
    let runtime = Runtime::try_new().unwrap();
    let mut natives = Vec::new();
    let mut bags = Vec::new();
    let mut sharing = Vec::new();
    // Complete all ten preservation checks before any new sharing assertion.
    for case in &CASES {
        let owner = prototype(&runtime, case.owner);
        let key = named_key(&runtime, owner, case.member);
        let property = &runtime.objects[owner].values[key];
        assert!(property.enumerable && property.configurable);
        let value = match (&property.value, case.part) {
            (
                PropertyValue::Data {
                    value,
                    writable: true,
                },
                "value",
            ) => value,
            (PropertyValue::Accessor { get, .. }, "get") => get,
            (PropertyValue::Accessor { set, .. }, "set") => set,
            _ => panic!("PI owner descriptor"),
        };
        let Value::Native(native) = value else {
            panic!("PI native");
        };
        assert_eq!(native.name, format!("DOM.PI.{}", case.suffix));
        metadata(&runtime, native, case.display, case.arity);
        let bag = native.properties.unwrap().get();
        assert!(!bags.contains(&bag));
        assert!(natives.iter().all(|old| !Rc::ptr_eq(old, native)));
        bags.push(bag);
        natives.push(native.clone());
        if case.part == "value" {
            let order_key = runtime.objects[owner]
                .order
                .iter()
                .find(|item| *item == key)
                .unwrap();
            sharing.push((
                case.member,
                key.as_string().unwrap().clone(),
                order_key.as_string().unwrap().clone(),
                text(runtime.objects[bag].get("name").unwrap()).clone(),
            ));
        }
    }
    assert_eq!((bags.len(), sharing.len()), (10, 6));
    let owner = prototype(&runtime, "CharacterData");
    let length_key = named_key(&runtime, owner, "length");
    let property = &runtime.objects[owner].values[length_key];
    assert!(matches!(
        &property.value,
        PropertyValue::Accessor {
            set: Value::Undefined,
            ..
        }
    ));
    let cached_length = named_key(
        &runtime,
        constructor_bag(&runtime, "CharacterData"),
        "length",
    );
    assert_eq!(length_key, cached_length);
    for (name, map, order, display) in sharing {
        assert!(
            std::ptr::eq(map.units().as_ptr(), display.units().as_ptr()),
            "{name} map/display"
        );
        assert!(
            std::ptr::eq(order.units().as_ptr(), display.units().as_ptr()),
            "{name} order/display"
        );
    }
    assert!(std::ptr::eq(
        length_key.as_string().unwrap().units().as_ptr(),
        cached_length.as_string().unwrap().units().as_ptr()
    ));
    let order_key = runtime.objects[owner]
        .order
        .iter()
        .find(|item| *item == length_key)
        .unwrap();
    assert!(std::ptr::eq(
        order_key.as_string().unwrap().units().as_ptr(),
        cached_length.as_string().unwrap().units().as_ptr()
    ));
}

#[test]
fn title_borrowed_metadata_keys_preserve_two_distinct_accessor_bags() {
    let runtime = Runtime::try_new().unwrap();
    let constructor = constructor_bag(&runtime, "Document");
    let owner = prototype(&runtime, "Document");
    let property = &runtime.objects[owner].values[named_key(&runtime, owner, "title")];
    assert!(property.enumerable && property.configurable);
    let PropertyValue::Accessor {
        get: Value::Native(get),
        set: Value::Native(set),
    } = &property.value
    else {
        panic!("title accessors");
    };
    assert!(!Rc::ptr_eq(get, set));
    assert_ne!(get.properties, set.properties);
    assert_eq!(get.name, "DOM.Title.get");
    assert_eq!(set.name, "DOM.Title.set");
    metadata(&runtime, get, "get title", 0);
    metadata(&runtime, set, "set title", 1);
    let names: Vec<_> = runtime.objects[owner]
        .order
        .iter()
        .filter_map(PropertyKey::as_string)
        .collect();
    assert_eq!(
        names.first().unwrap().units(),
        &"title".bytes().map(u16::from).collect::<Vec<_>>()
    );
    // Both bags and their identities are checked before the first pointer test.
    for native in [get, set] {
        let bag = native.properties.unwrap().get();
        for (index, name) in ["length", "name"].into_iter().enumerate() {
            let cached = named_key(&runtime, constructor, name).as_string().unwrap();
            let key = named_key(&runtime, bag, name).as_string().unwrap();
            assert!(
                std::ptr::eq(key.units().as_ptr(), cached.units().as_ptr()),
                "{} {name}",
                native.name
            );
            assert!(std::ptr::eq(
                runtime.objects[bag].order[index]
                    .as_string()
                    .unwrap()
                    .units()
                    .as_ptr(),
                cached.units().as_ptr()
            ));
        }
    }
}

#[test]
fn startup_headroom_raw_observation_precedes_the_frozen_99694_work_witness() {
    let runtime = Runtime::uninitialized().finish_bootstrap().unwrap();
    println!(
        "STARTUP_HEADROOM raw_work={} remaining={} allocated={} objects={} capacity={} globals={} natives={} prototypes={}",
        100_000 - runtime.steps,
        runtime.steps,
        runtime.allocated,
        runtime.objects.len(),
        runtime.objects.capacity(),
        runtime.environments[0].bindings.len(),
        runtime.native_properties.len(),
        runtime.prototypes.len()
    );
    assert_eq!(runtime.objects.len(), 752);
    assert!(runtime.objects.capacity() >= 752);
    assert_eq!(runtime.environments[0].bindings.len(), 212);
    assert_eq!(runtime.native_properties.len(), 299);
    assert_eq!(runtime.prototypes.len(), 25);
    assert!(runtime.frames.is_empty());
    assert!(runtime.console.is_empty());
    assert_eq!(
        (runtime.calls, runtime.eval_depth, runtime.stack_units),
        (0, 0, 0)
    );
    assert_eq!(100_000 - runtime.steps, 99_694);
    assert_eq!(runtime.steps, 306);
}
