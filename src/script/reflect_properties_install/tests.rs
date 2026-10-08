use super::*;

// Recover only the private pre-extension owner shape from a fresh realm. The
// published constructors/prototypes and registry are unchanged; the final nine
// metadata bags are removed without altering the reserved arena capacity.
fn before_extension() -> (Runtime, usize) {
    let mut runtime = Runtime::try_new().unwrap();
    let Value::Object(owner) = runtime.environments[0].bindings["Reflect"].value else {
        panic!("Reflect owner");
    };
    let old = runtime.objects[owner].order[..5].to_vec();
    runtime.objects[owner]
        .values
        .retain(|key, _| old.contains(key));
    runtime.objects[owner].order = old;
    runtime.objects.truncate(runtime.objects.len() - 9);
    runtime.steps = MAX_STEPS;
    runtime.allocated = 0;
    (runtime, owner)
}

fn owner_snapshot(runtime: &Runtime, owner: usize) -> String {
    format!(
        "{:?}/{:?}",
        runtime.objects[owner].values, runtime.objects[owner].order
    )
}

#[test]
fn reflect_installation_matches_typed_ledger_and_preserves_old_identity() {
    let (mut runtime, owner) = before_extension();
    let old = runtime.objects[owner].values.clone();
    let registry = runtime.native_properties.clone();
    let objects = runtime.objects.len();
    let capacity = runtime.objects.capacity();
    runtime.initialize_reflect_property_intrinsics().unwrap();
    let work = MAX_STEPS - runtime.steps;
    let heap = runtime.allocated;
    let expected_heap = 12 * leaf_bytes()
        + 32 * std::mem::size_of::<PropertyKey>()
        + 62 * std::mem::size_of::<(PropertyKey, Property)>()
        + 9 * (72 + std::mem::size_of::<Option<AbortSlot>>() + std::mem::size_of::<Option<f64>>())
        + 1160
        + 9 * (std::mem::size_of::<Native>() + 32)
        + 176;
    println!(
        "REFLECT_INSTALL work={work} heap={heap} arena_added={} objects={} capacity={capacity}",
        9 * std::mem::size_of::<ScriptObject>(),
        runtime.objects.len()
    );
    assert_eq!(work, 3278);
    assert_eq!(heap, expected_heap);
    assert_eq!(runtime.objects.len(), objects + 9);
    assert_eq!(runtime.objects.capacity(), capacity);
    assert_eq!(runtime.native_properties, registry);
    for (key, property) in old {
        let after = &runtime.objects[owner].values[&key];
        match (property.value, &after.value) {
            (
                PropertyValue::Data { value, writable },
                PropertyValue::Data {
                    value: actual,
                    writable: w,
                },
            ) => {
                assert_eq!(&value, actual);
                assert_eq!(&writable, w);
            }
            _ => panic!("old Reflect property shape changed"),
        }
        assert_eq!(property.enumerable, after.enumerable);
        assert_eq!(property.configurable, after.configurable);
    }
    let mut doc = Document::parse("");
    runtime.steps = MAX_STEPS;
    assert_eq!(runtime.execute(
        "Object.getOwnPropertyNames(Reflect).join(',') === 'ownKeys,apply,construct,defineProperty,has,get,set,deleteProperty,getOwnPropertyDescriptor,getPrototypeOf,setPrototypeOf,isExtensible,preventExtensions'",
        &mut doc).unwrap(), Value::Bool(true));
}

#[test]
fn reflect_installation_exact_and_short_admissions_precede_owner_publication() {
    let (mut measured, _) = before_extension();
    measured.initialize_reflect_property_intrinsics().unwrap();
    let work = MAX_STEPS - measured.steps;
    let heap = measured.allocated;
    for short in [false, true] {
        let (mut runtime, owner) = before_extension();
        let before = owner_snapshot(&runtime, owner);
        runtime.steps = work - usize::from(short);
        let result = runtime.initialize_reflect_property_intrinsics();
        if short {
            assert!(result.unwrap_err().is_resource_limit());
            assert_eq!(owner_snapshot(&runtime, owner), before);
        } else {
            result.unwrap();
            assert_eq!(runtime.objects[owner].values.len(), 14);
        }
        assert_eq!(runtime.steps, 0);

        let (mut runtime, owner) = before_extension();
        let before = owner_snapshot(&runtime, owner);
        runtime.allocated = MAX_HEAP - heap + usize::from(short);
        let result = runtime.initialize_reflect_property_intrinsics();
        if short {
            assert!(result.unwrap_err().is_resource_limit());
            assert_eq!(owner_snapshot(&runtime, owner), before);
        } else {
            result.unwrap();
            assert_eq!(runtime.allocated, MAX_HEAP);
        }
    }
}

#[test]
fn reflect_installation_rejects_missing_capacity_and_unexpected_roster() {
    let (mut runtime, owner) = before_extension();
    runtime.objects.shrink_to_fit();
    let before = owner_snapshot(&runtime, owner);
    let count = runtime.objects.len();
    assert!(
        runtime
            .initialize_reflect_property_intrinsics()
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(
        (runtime.objects.len(), runtime.objects.capacity()),
        (count, count)
    );
    assert_eq!(owner_snapshot(&runtime, owner), before);

    let (mut runtime, owner) = before_extension();
    runtime.objects[owner].order.swap(0, 1);
    let before = owner_snapshot(&runtime, owner);
    let count = runtime.objects.len();
    assert!(
        runtime
            .initialize_reflect_property_intrinsics()
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.objects.len(), count);
    assert_eq!(runtime.allocated, 0);
    assert_eq!(owner_snapshot(&runtime, owner), before);
}
