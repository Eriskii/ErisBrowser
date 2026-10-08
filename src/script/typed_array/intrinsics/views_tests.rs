use super::*;
use std::mem::size_of;

// These are literal graph expectations, independent of Kind::ALL and LITERALS.
const KINDS: [(&str, usize); 10] = [
    ("Int8Array", 1),
    ("Uint8Array", 1),
    ("Uint8ClampedArray", 1),
    ("Int16Array", 2),
    ("Uint16Array", 2),
    ("Int32Array", 4),
    ("Uint32Array", 4),
    ("Float16Array", 2),
    ("Float32Array", 4),
    ("Float64Array", 8),
];

fn before_installation() -> Runtime {
    let mut runtime = Runtime::uninitialized();
    runtime.reserve_bootstrap_objects().unwrap();
    machine::initialize(&mut runtime).unwrap();
    runtime.initialize_intrinsics_before_typed_array().unwrap();
    unpublished(&runtime);
    for (name, _) in KINDS {
        assert!(!runtime.environments[0].bindings.contains_key(name));
    }
    assert!(runtime.array_prototype.is_some());
    assert!(runtime.native_properties.contains_key("Array.toString"));
    // Preserve the real preceding stages; only the two test counters reset.
    runtime.steps = MAX_STEPS;
    runtime.allocated = 0;
    runtime
}

fn unpublished(runtime: &Runtime) {
    assert!(runtime.typed_arrays.intrinsic.is_none());
    assert!(runtime.typed_arrays.prototype.is_none());
    assert!(
        runtime
            .typed_arrays
            .constructors
            .iter()
            .all(Option::is_none)
    );
    assert!(runtime.typed_arrays.prototypes.iter().all(Option::is_none));
    assert!(runtime.typed_arrays.records.is_empty());
}

fn native(value: &Value) -> &Rc<Native> {
    match value {
        Value::Native(value) => value,
        _ => panic!("expected a genuine cached native"),
    }
}

fn data(runtime: &Runtime, owner: usize, key: impl Into<PropertyKey>) -> &Value {
    runtime.objects[owner].get(key).unwrap()
}

fn descriptor(
    runtime: &Runtime,
    owner: usize,
    key: impl Into<PropertyKey>,
    value: &Value,
    writable: bool,
    configurable: bool,
) {
    let property = &runtime.objects[owner].values[&key.into()];
    match &property.value {
        PropertyValue::Data {
            value: actual,
            writable: actual_writable,
        } => {
            assert_eq!(actual, value);
            assert_eq!(*actual_writable, writable);
        }
        _ => panic!("unexpected accessor"),
    }
    assert!(!property.enumerable);
    assert_eq!(property.configurable, configurable);
}

fn bag_snapshot(runtime: &Runtime, owner: usize) -> String {
    let bag = &runtime.objects[owner];
    format!(
        "{:?}/{:?}/{:?}/{}",
        bag.values, bag.order, bag.prototype, bag.non_extensible
    )
}

fn prefix_snapshot(runtime: &Runtime, end: usize) -> Vec<String> {
    (0..end).map(|owner| bag_snapshot(runtime, owner)).collect()
}

fn expected_site_heap() -> usize {
    // Thirty-six small maps plus three shared-owner nodes. The original
    // 31-leaf graph had92 order handles; the views/search graph has111.
    // The65 pairs pay17 staged and48 conservatively reserved sort scratch
    // pairs before the bulk collection.
    let block =
        16 * (size_of::<PropertyKey>() + size_of::<Property>()) + 32 * size_of::<usize>() + 64;
    39 * block
        + 65 * size_of::<(PropertyKey, Property)>()
        + 37 * (72 + size_of::<Option<AbortSlot>>() + size_of::<Option<f64>>())
        + 111 * size_of::<PropertyKey>()
        + 3656
        + 26 * (size_of::<Native>() + 32)
        + 406
}

fn old_site_heap() -> usize {
    let block =
        16 * (size_of::<PropertyKey>() + size_of::<Property>()) + 32 * size_of::<usize>() + 64;
    31 * block
        + 31 * (72 + size_of::<Option<AbortSlot>>() + size_of::<Option<f64>>())
        + 92 * size_of::<PropertyKey>()
        + 3016
        + 20 * (size_of::<Native>() + 32)
        + 300
}

fn expected_site_work() -> usize {
    // Retained old4742, replace the ten-key owner416; two new leaves116,
    // thirteen-key owner1370, fixed64, pooled text64, two bags18,
    // native names50, and bounded Array alias704. No production constants.
    // Search additions: four58-work leaves,384 shared-owner work,64 fixed,
    // four strings88, four bags36, and four native names104.
    4742 - 416 + 116 + 1370 + 64 + 64 + 18 + 50 + 704 + 232 + 384 + 64 + 88 + 36 + 104
}

#[test]
fn typed_array_views_metadata_preserves_old_31_bag_ordinals_and_cached_handles() {
    let mut runtime = before_installation();
    let base = runtime.objects.len();
    let capacity = runtime.objects.capacity();
    let prefix = prefix_snapshot(&runtime, base);
    let registry = runtime.native_properties.clone();
    let old_globals = format!(
        "{:?}",
        runtime.environments[0].bindings.keys().collect::<Vec<_>>()
    );
    runtime.initialize_typed_array_intrinsics().unwrap();
    assert_eq!(runtime.objects.len(), base + 37);
    assert_eq!(runtime.objects.capacity(), capacity);
    assert_eq!(runtime.native_properties, registry);
    assert_eq!(prefix_snapshot(&runtime, base), prefix);
    assert_eq!(
        format!(
            "{:?}",
            runtime.environments[0].bindings.keys().collect::<Vec<_>>()
        ),
        old_globals
    );
    assert_eq!(runtime.typed_arrays.prototype, Some(base));
    let shared = runtime.typed_arrays.intrinsic.as_ref().unwrap();
    assert_eq!(native(shared).name, "TypedArray");
    assert_eq!(native(shared).properties.unwrap().get(), base + 1);
    assert!(matches!(native(shared).receiver, Value::Undefined));
    descriptor(
        &runtime,
        base + 1,
        "length",
        &Value::Number(0.0),
        false,
        true,
    );
    descriptor(
        &runtime,
        base + 1,
        "prototype",
        &Value::Object(base),
        false,
        false,
    );
    assert_eq!(
        runtime.objects[base].prototype,
        Some(Value::Object(runtime.prototypes["Object"]))
    );

    for (ordinal, key, expected_name, accessor, display) in [
        (
            2,
            PropertyKey::from("buffer"),
            "TypedArray.getBuffer",
            true,
            "get buffer",
        ),
        (
            3,
            PropertyKey::from("byteLength"),
            "TypedArray.getByteLength",
            true,
            "get byteLength",
        ),
        (
            4,
            PropertyKey::from("byteOffset"),
            "TypedArray.getByteOffset",
            true,
            "get byteOffset",
        ),
        (
            5,
            PropertyKey::from("length"),
            "TypedArray.getLength",
            true,
            "get length",
        ),
        (
            6,
            runtime.well_known_key("toStringTag"),
            "TypedArray.getTag",
            true,
            "get [Symbol.toStringTag]",
        ),
        (
            7,
            runtime.well_known_key("species"),
            "TypedArray.getSpecies",
            true,
            "get [Symbol.species]",
        ),
        (
            8,
            PropertyKey::from("keys"),
            "TypedArray.keys",
            false,
            "keys",
        ),
        (
            9,
            PropertyKey::from("values"),
            "TypedArray.values",
            false,
            "values",
        ),
        (
            10,
            PropertyKey::from("entries"),
            "TypedArray.entries",
            false,
            "entries",
        ),
    ] {
        let owner = if ordinal == 7 { base + 1 } else { base };
        let property = &runtime.objects[owner].values[&key];
        assert!(!property.enumerable);
        assert!(property.configurable);
        let value = match (&property.value, accessor) {
            (PropertyValue::Accessor { get, set }, true) => {
                assert!(matches!(set, Value::Undefined));
                get
            }
            (
                PropertyValue::Data {
                    value,
                    writable: true,
                },
                false,
            ) => value,
            _ => panic!("old descriptor shape changed"),
        };
        let function = native(value);
        assert_eq!(function.name, expected_name);
        assert_eq!(function.properties.unwrap().get(), base + ordinal);
        assert!(matches!(function.receiver, Value::Undefined));
        descriptor(
            &runtime,
            base + ordinal,
            "length",
            &Value::Number(0.0),
            false,
            true,
        );
        descriptor(
            &runtime,
            base + ordinal,
            "name",
            &Value::String(display.into()),
            false,
            true,
        );
        assert_eq!(
            runtime.objects[base + ordinal].order,
            [PropertyKey::from("length"), PropertyKey::from("name")]
        );
        assert!(!runtime.objects[base + ordinal].contains_key("prototype"));
    }
    for (index, (name, width)) in KINDS.into_iter().enumerate() {
        let prototype = base + 11 + 2 * index;
        let bag = prototype + 1;
        let constructor = runtime.typed_arrays.constructors[index].as_ref().unwrap();
        assert_eq!(runtime.typed_arrays.prototypes[index], Some(prototype));
        assert_eq!(native(constructor).name, name);
        assert_eq!(native(constructor).properties.unwrap().get(), bag);
        assert!(matches!(native(constructor).receiver, Value::Undefined));
        assert_eq!(
            runtime.objects[prototype].prototype,
            Some(Value::Object(base))
        );
        assert!(Rc::ptr_eq(
            native(runtime.objects[bag].prototype.as_ref().unwrap()),
            native(shared)
        ));
        assert!(Rc::ptr_eq(
            native(data(&runtime, prototype, "constructor")),
            native(constructor)
        ));
        descriptor(&runtime, bag, "length", &Value::Number(3.0), false, true);
        descriptor(
            &runtime,
            bag,
            "name",
            &Value::String(name.into()),
            false,
            true,
        );
        descriptor(
            &runtime,
            bag,
            "prototype",
            &Value::Object(prototype),
            false,
            false,
        );
        descriptor(
            &runtime,
            bag,
            "BYTES_PER_ELEMENT",
            &Value::Number(width as f64),
            false,
            false,
        );
        descriptor(
            &runtime,
            prototype,
            "BYTES_PER_ELEMENT",
            &Value::Number(width as f64),
            false,
            false,
        );
        assert_eq!(
            runtime.objects[bag].order,
            ["length", "name", "prototype", "BYTES_PER_ELEMENT"].map(PropertyKey::from)
        );
        assert_eq!(
            runtime.objects[prototype].order,
            ["constructor", "BYTES_PER_ELEMENT"].map(PropertyKey::from)
        );
    }
    assert_eq!(
        runtime.objects[base..]
            .iter()
            .map(|bag| bag.values.len())
            .sum::<usize>(),
        111
    );
    assert_eq!(
        runtime.objects[base..]
            .iter()
            .map(|bag| bag.order.len())
            .sum::<usize>(),
        111
    );
    assert!(runtime.typed_arrays.records.is_empty());
}

#[test]
fn typed_array_views_and_search_metadata_append_properties_and_exact_array_alias() {
    let mut runtime = before_installation();
    let array_owner = runtime.array_properties[runtime.array_prototype.unwrap()];
    let alias = native(data(&runtime, array_owner, "toString")).clone();
    let array_before = bag_snapshot(&runtime, array_owner);
    let base = runtime.objects.len();
    runtime.initialize_typed_array_intrinsics().unwrap();
    let expected = vec![
        "constructor".into(),
        "buffer".into(),
        "byteLength".into(),
        "byteOffset".into(),
        "length".into(),
        runtime.well_known_key("toStringTag"),
        "keys".into(),
        "values".into(),
        "entries".into(),
        runtime.well_known_key("iterator"),
        "subarray".into(),
        "join".into(),
        "toString".into(),
        "at".into(),
        "includes".into(),
        "indexOf".into(),
        "lastIndexOf".into(),
    ];
    assert_eq!(runtime.objects[base].order, expected);
    assert_eq!(runtime.objects[base].values.len(), 17);
    assert_eq!(bag_snapshot(&runtime, array_owner), array_before);
    for (name, full, ordinal, length) in [
        ("subarray", "TypedArray.subarray", 31, 2.0),
        ("join", "TypedArray.join", 32, 1.0),
        ("at", "TypedArray.at", 33, 1.0),
        ("includes", "TypedArray.includes", 34, 1.0),
        ("indexOf", "TypedArray.indexOf", 35, 1.0),
        ("lastIndexOf", "TypedArray.lastIndexOf", 36, 1.0),
    ] {
        let value = data(&runtime, base, name);
        descriptor(&runtime, base, name, value, true, true);
        assert_eq!(native(value).name, full);
        assert_eq!(native(value).properties.unwrap().get(), base + ordinal);
        assert!(matches!(native(value).receiver, Value::Undefined));
        descriptor(
            &runtime,
            base + ordinal,
            "length",
            &Value::Number(length),
            false,
            true,
        );
        descriptor(
            &runtime,
            base + ordinal,
            "name",
            &Value::String(name.into()),
            false,
            true,
        );
        assert_eq!(runtime.objects[base + ordinal].values.len(), 2);
        assert_eq!(
            runtime.objects[base + ordinal].order,
            ["length", "name"].map(PropertyKey::from)
        );
        assert_eq!(
            runtime.objects[base + ordinal].prototype,
            Some(Value::Function(runtime.function_prototype))
        );
        assert!(!runtime.objects[base + ordinal].contains_key("prototype"));
    }
    let saved_alias = data(&runtime, base, "toString");
    descriptor(&runtime, base, "toString", saved_alias, true, true);
    assert!(Rc::ptr_eq(native(saved_alias), &alias));
    assert!(Rc::ptr_eq(
        native(data(&runtime, base, "values")),
        native(data(&runtime, base, runtime.well_known_key("iterator")))
    ));
}

#[test]
fn typed_array_views_metadata_literal_ledger_and_exact_one_short_site_admission() {
    let work = expected_site_work();
    let heap = expected_site_heap();
    assert_eq!(work, 7620);
    assert_eq!(work - 4742, 2878);
    let delta = heap - old_site_heap() + 6 * size_of::<ScriptObject>();
    println!(
        "TYPED_VIEWS_SITE work={work} heap={heap} reserved_arena_delta={} marginal_total={delta}",
        6 * size_of::<ScriptObject>()
    );
    assert_eq!(delta, 19954);
    for short in [false, true] {
        let mut runtime = before_installation();
        let base = runtime.objects.len();
        let old = prefix_snapshot(&runtime, base);
        let registry = runtime.native_properties.clone();
        runtime.steps = work - usize::from(short);
        let result = runtime.initialize_typed_array_intrinsics();
        if short {
            assert!(result.unwrap_err().is_resource_limit());
            unpublished(&runtime);
            assert!(runtime.objects[base].values.is_empty());
            assert!(runtime.objects[base].order.is_empty());
        } else {
            result.unwrap();
            assert_eq!(runtime.allocated, heap);
            assert_eq!(runtime.typed_arrays.prototype, Some(base));
        }
        assert_eq!(runtime.steps, 0);
        assert_eq!(prefix_snapshot(&runtime, base), old);
        assert_eq!(runtime.native_properties, registry);

        let mut runtime = before_installation();
        let base = runtime.objects.len();
        let old = prefix_snapshot(&runtime, base);
        let registry = runtime.native_properties.clone();
        runtime.allocated = MAX_HEAP - heap + usize::from(short);
        let result = runtime.initialize_typed_array_intrinsics();
        if short {
            assert!(result.unwrap_err().is_resource_limit());
            unpublished(&runtime);
            assert!(runtime.objects[base].values.is_empty());
            assert!(runtime.objects[base].order.is_empty());
        } else {
            result.unwrap();
            assert_eq!(runtime.allocated, MAX_HEAP);
            assert_eq!(MAX_STEPS - runtime.steps, work);
        }
        assert_eq!(prefix_snapshot(&runtime, base), old);
        assert_eq!(runtime.native_properties, registry);
    }
}

#[test]
fn typed_array_views_metadata_initial_work_and_heap_admission_precede_new_objects() {
    let initial_work = 2870 - 416 + 348 + 192 + 1754 + 64 + 64;
    let block =
        16 * (size_of::<PropertyKey>() + size_of::<Property>()) + 32 * size_of::<usize>() + 64;
    let initial_heap = 39 * block + 65 * size_of::<(PropertyKey, Property)>();
    for heap_cut in [false, true] {
        let mut runtime = before_installation();
        let base = runtime.objects.len();
        let old = prefix_snapshot(&runtime, base);
        let registry = runtime.native_properties.clone();
        if heap_cut {
            runtime.allocated = MAX_HEAP - initial_heap + 1;
        } else {
            runtime.steps = initial_work - 1;
        }
        assert!(
            runtime
                .initialize_typed_array_intrinsics()
                .unwrap_err()
                .is_resource_limit()
        );
        if heap_cut {
            assert_eq!(runtime.steps, MAX_STEPS - initial_work);
        } else {
            assert_eq!(runtime.steps, 0);
            assert_eq!(runtime.allocated, 0);
        }
        assert_eq!(runtime.objects.len(), base);
        assert_eq!(prefix_snapshot(&runtime, base), old);
        assert_eq!(runtime.native_properties, registry);
        unpublished(&runtime);
    }
}

#[test]
fn typed_array_views_metadata_duplicate_installation_keeps_saved_handles_and_graph() {
    for partial in [0, 1, 2] {
        let mut runtime = before_installation();
        if partial == 0 {
            runtime.typed_arrays.intrinsic = Some(Value::Undefined);
        } else if partial == 1 {
            runtime.typed_arrays.prototype = Some(0);
        } else {
            runtime.initialize_typed_array_intrinsics().unwrap();
        }
        let base = runtime.objects.len();
        let before = prefix_snapshot(&runtime, base);
        let saved = runtime.typed_arrays.intrinsic.clone();
        let prototype = runtime.typed_arrays.prototype;
        let constructors = runtime.typed_arrays.constructors.clone();
        let prototypes = runtime.typed_arrays.prototypes;
        let registry = runtime.native_properties.clone();
        runtime.steps = MAX_STEPS;
        runtime.allocated = 0;
        let error = runtime.initialize_typed_array_intrinsics().unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(error.message, "TypedArray intrinsics initialized twice");
        assert_eq!(runtime.objects.len(), base);
        assert_eq!(prefix_snapshot(&runtime, base), before);
        assert_eq!(runtime.native_properties, registry);
        assert_eq!(runtime.typed_arrays.intrinsic, saved);
        assert_eq!(runtime.typed_arrays.prototype, prototype);
        assert_eq!(runtime.typed_arrays.prototypes, prototypes);
        for (actual, previous) in runtime.typed_arrays.constructors.iter().zip(&constructors) {
            assert_eq!(actual, previous);
            if let (Some(actual), Some(previous)) = (actual, previous) {
                assert!(Rc::ptr_eq(native(actual), native(previous)));
            }
        }
        if partial == 2 {
            assert!(Rc::ptr_eq(
                native(runtime.typed_arrays.intrinsic.as_ref().unwrap()),
                native(saved.as_ref().unwrap())
            ));
        }
    }
}

#[test]
fn typed_array_views_metadata_alias_guards_and_exact_bounded_lookup() {
    let name = JsString::from("toString");
    for broken in 0..8 {
        let mut runtime = before_installation();
        let array = runtime.array_prototype.unwrap();
        let owner = runtime.array_properties[array];
        match broken {
            0 => runtime.array_prototype = None,
            1 => runtime.array_prototype = Some(runtime.array_properties.len()),
            2 => runtime.array_properties[array] = runtime.objects.len(),
            3 => {
                for number in 0..129 {
                    runtime.objects[owner].values.insert(
                        format!("extra{number}").into(),
                        Property::data(Value::Null, false, false, false),
                    );
                }
            }
            4 => {
                runtime.objects[owner]
                    .values
                    .remove(&PropertyKey::from("toString"));
            }
            5 => {
                runtime.objects[owner]
                    .values
                    .insert("toString".into(), getter(Value::Undefined));
            }
            6 => {
                runtime.objects[owner].values.insert(
                    "toString".into(),
                    Property::data(Value::Number(1.0), true, false, true),
                );
            }
            7 => {
                let wrong = Runtime::native("Array.join", Value::Undefined);
                runtime.objects[owner]
                    .values
                    .insert("toString".into(), Property::data(wrong, true, false, true));
            }
            _ => unreachable!(),
        }
        let before = prefix_snapshot(&runtime, runtime.objects.len());
        let registry = runtime.native_properties.clone();
        let error = runtime.typed_array_to_string_alias(&name).unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(error.message, "TypedArray Array.toString alias missing");
        assert_eq!(MAX_STEPS - runtime.steps, if broken < 4 { 16 } else { 704 });
        assert_eq!(runtime.allocated, 0);
        assert_eq!(prefix_snapshot(&runtime, runtime.objects.len()), before);
        assert_eq!(runtime.native_properties, registry);
        unpublished(&runtime);
    }
    for short in [false, true] {
        let mut runtime = before_installation();
        let owner = runtime.array_properties[runtime.array_prototype.unwrap()];
        let original = native(data(&runtime, owner, "toString")).clone();
        // Exercise the admitted maximum, not only the current small Array map.
        let mut suffix = 0;
        while runtime.objects[owner].values.len() < 128 {
            runtime.objects[owner].values.insert(
                format!("extra{suffix}").into(),
                Property::data(Value::Null, false, false, false),
            );
            suffix += 1;
        }
        runtime.steps = 704 - usize::from(short);
        let result = runtime.typed_array_to_string_alias(&name);
        if short {
            assert!(result.unwrap_err().is_resource_limit());
        } else {
            assert!(Rc::ptr_eq(native(&result.unwrap()), &original));
        }
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, 0);
        unpublished(&runtime);
    }
}

#[test]
fn typed_array_views_metadata_shared_owner_shape_guards_leave_owner_unchanged() {
    for broken in 0..5 {
        let mut runtime = before_installation();
        let owner = runtime.objects.len();
        let mut bag = ScriptObject {
            order: Vec::with_capacity(if broken == 2 { 16 } else { 17 }),
            ..ScriptObject::default()
        };
        if broken == 0 {
            bag.values.insert(
                "old".into(),
                Property::data(Value::Bool(true), true, false, true),
            );
        } else if broken == 1 {
            bag.order.push("old".into());
        }
        runtime.objects.push(bag);
        let mut entries: [(PropertyKey, Property); 17] = std::array::from_fn(|index| {
            (
                format!("key{index:02}").into(),
                Property::data(Value::Number(index as f64), true, false, true),
            )
        });
        if broken == 3 {
            entries[7].0 = entries[6].0.clone();
        } else if broken == 4 {
            entries.swap(5, 6);
        }
        let before = bag_snapshot(&runtime, owner);
        let error = runtime.typed_array_fill_shared(owner, entries).unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(error.message, "unexpected TypedArray shared metadata shape");
        assert_eq!(bag_snapshot(&runtime, owner), before);
        unpublished(&runtime);
    }
}

#[test]
fn typed_array_views_metadata_raw_bootstrap_diagnostic_and_public_reset() {
    let raw = Runtime::uninitialized().finish_bootstrap().unwrap();
    println!(
        "TYPED_VIEWS_TYPES S={} A={} D={} K={} N={} Pair={} Property={} State={}",
        size_of::<ScriptObject>(),
        size_of::<Option<AbortSlot>>(),
        size_of::<Option<f64>>(),
        size_of::<PropertyKey>(),
        size_of::<Native>(),
        size_of::<(PropertyKey, Property)>(),
        size_of::<Property>(),
        size_of::<State>(),
    );
    println!(
        "TYPED_VIEWS_BOOTSTRAP remaining={} allocated={} objects={} capacity={} native={} prototypes={} globals={}",
        raw.steps,
        raw.allocated,
        raw.objects.len(),
        raw.objects.capacity(),
        raw.native_properties.len(),
        raw.prototypes.len(),
        raw.environments[0].bindings.len(),
    );
    // Raw totals are measured here before any existing descriptive snapshots
    // are migrated; the exact site contract is asserted in its separate test.
    assert!(raw.steps < MAX_STEPS);
    assert_eq!(raw.objects.len(), raw.objects.capacity());
    assert!(raw.typed_arrays.records.is_empty());
    for (index, (name, _)) in KINDS.into_iter().enumerate() {
        let saved = raw.typed_arrays.constructors[index].as_ref().unwrap();
        assert!(Rc::ptr_eq(
            native(saved),
            native(&raw.environments[0].bindings[name].value)
        ));
        assert_eq!(
            data(&raw, native(saved).properties.unwrap().get(), "prototype"),
            &Value::Object(raw.typed_arrays.prototypes[index].unwrap())
        );
    }
    let public = Runtime::try_new().unwrap();
    assert_eq!(public.steps, MAX_STEPS);
    assert_eq!(public.allocated, raw.allocated);
    assert_eq!(public.objects.len(), raw.objects.len());
    assert_eq!(public.native_properties, raw.native_properties);
    assert!(public.frames.is_empty());
    assert!(public.active_array_joins.is_empty());
    assert_eq!(public.calls, 0);
    assert_eq!(public.eval_depth, 0);
    assert_eq!(public.stack_units, 0);
    assert_eq!(public.json_depth, 0);
}
