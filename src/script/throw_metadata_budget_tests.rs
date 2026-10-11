//! Independent literal fees for stored-descriptor diagnostics; fixture setup is unmetered Rust.
use super::*;
use std::num::NonZeroUsize;

#[derive(Clone, Copy, Debug)]
enum Fixture {
    Primitive,
    Host,
    Empty,
    ConstructorAccessor,
    Direct,
    Inherited,
    Legacy,
    RegistryTwelve,
    PropertyTwelve,
    NameTwelve,
    NameAccessor,
    Branded,
    BrandedNameAccessor,
    BrandedNameNumber,
    BrandedNonCallable,
    LargeName,
    Math,
    Json,
    MissingRegistry,
}

fn bag(runtime: &mut Runtime) -> usize {
    let id = runtime.objects.len();
    runtime.objects.push(ScriptObject::default());
    id
}
fn data(runtime: &mut Runtime, id: usize, key: &str, value: Value) {
    runtime.objects[id].insert(key.into(), value);
}
fn accessor(runtime: &mut Runtime, id: usize, key: &str) {
    runtime.objects[id].insert_property(
        key.into(),
        Property {
            value: PropertyValue::Accessor {
                get: Value::Undefined,
                set: Value::Undefined,
            },
            enumerable: true,
            configurable: true,
        },
    );
}
fn fixture(kind: Fixture) -> (Runtime, Value) {
    let mut runtime = Runtime::new();
    runtime.native_properties.clear();
    if matches!(kind, Fixture::Primitive) {
        return (runtime, Value::Number(7.0));
    }
    if matches!(kind, Fixture::Host) {
        return (runtime, Value::Document);
    }
    if matches!(kind, Fixture::MissingRegistry) {
        return (runtime, Value::Math);
    }
    let root = bag(&mut runtime);
    if matches!(kind, Fixture::Empty) {
        return (runtime, Value::Object(root));
    }
    if matches!(kind, Fixture::ConstructorAccessor) {
        accessor(&mut runtime, root, "constructor");
        // A blocking descriptor must not fall through to this otherwise valid data.
        let parent = bag(&mut runtime);
        data(&mut runtime, parent, "constructor", Value::Number(7.0));
        runtime.objects[root].prototype = Some(Value::Object(parent));
        return (runtime, Value::Object(root));
    }
    let branded = matches!(
        kind,
        Fixture::Branded
            | Fixture::BrandedNameAccessor
            | Fixture::BrandedNameNumber
            | Fixture::BrandedNonCallable
    );
    if branded {
        runtime.objects[root].intrinsic_error = Some("TypeError");
    }
    if matches!(kind, Fixture::BrandedNonCallable) {
        data(&mut runtime, root, "constructor", Value::Number(7.0));
        return (runtime, Value::Object(root));
    }
    let names = bag(&mut runtime);
    if matches!(kind, Fixture::NameAccessor | Fixture::BrandedNameAccessor) {
        accessor(&mut runtime, names, "name");
    } else if matches!(kind, Fixture::BrandedNameNumber) {
        data(&mut runtime, names, "name", Value::Number(7.0));
    } else {
        let text = if matches!(kind, Fixture::LargeName) {
            "x".repeat(32768)
        } else {
            "marker".to_owned()
        };
        data(&mut runtime, names, "name", Value::String(text.into()));
    }
    if matches!(kind, Fixture::NameTwelve) {
        for i in 0..11 {
            data(&mut runtime, names, &format!("z{i}"), Value::Null);
        }
    }
    let legacy = matches!(kind, Fixture::Legacy | Fixture::RegistryTwelve);
    let native_name = if branded { "TypeError" } else { "Fixture" };
    let constructor = Value::Native(Rc::new(Native {
        name: native_name.to_owned(),
        receiver: Value::Window,
        properties: if legacy {
            None
        } else {
            NonZeroUsize::new(names)
        },
    }));
    if legacy {
        runtime
            .native_properties
            .insert(native_name.to_owned(), names);
    }
    if matches!(kind, Fixture::RegistryTwelve) {
        for i in 0..11 {
            runtime.native_properties.insert(format!("Z{i}"), names);
        }
    }
    data(&mut runtime, root, "constructor", constructor);
    if matches!(kind, Fixture::PropertyTwelve) {
        for i in 0..11 {
            data(&mut runtime, root, &format!("z{i}"), Value::Null);
        }
    }
    if matches!(kind, Fixture::Inherited) {
        let child = bag(&mut runtime);
        runtime.objects[child].prototype = Some(Value::Object(root));
        return (runtime, Value::Object(child));
    }
    if matches!(kind, Fixture::Math | Fixture::Json) {
        let (key, value) = if matches!(kind, Fixture::Math) {
            ("Math", Value::Math)
        } else {
            ("JSON", Value::Json)
        };
        runtime.native_properties.insert(key.to_owned(), root);
        return (runtime, value);
    }
    (runtime, Value::Object(root))
}

#[test]
fn throw_completion_metadata_exact_and_one_short_cuts_precede_every_reached_operation() {
    // (fixture, metadata work, metadata heap, display, private canonical identity)
    for (kind, work, heap, display, intrinsic) in [
        (Fixture::Primitive, 8, 0, None, None),
        (Fixture::Host, 8, 0, None, None),
        (Fixture::Empty, 46, 108, None, None),
        (Fixture::ConstructorAccessor, 48, 108, None, None),
        (Fixture::Direct, 74, 230, Some("marker"), None),
        (Fixture::Inherited, 82, 230, Some("marker"), None),
        (Fixture::Legacy, 75, 230, Some("marker"), None),
        (Fixture::RegistryTwelve, 86, 230, Some("marker"), None),
        (Fixture::PropertyTwelve, 96, 230, Some("marker"), None),
        (Fixture::NameTwelve, 85, 230, Some("marker"), None),
        (Fixture::NameAccessor, 73, 188, None, None),
        (Fixture::Branded, 91, 230, Some("marker"), Some("TypeError")),
        (
            Fixture::BrandedNameAccessor,
            90,
            188,
            None,
            Some("TypeError"),
        ),
        (Fixture::BrandedNameNumber, 90, 188, None, Some("TypeError")),
        (Fixture::BrandedNonCallable, 65, 108, None, None),
        (Fixture::Math, 75, 230, Some("marker"), None),
        (Fixture::Json, 75, 230, Some("marker"), None),
        (Fixture::MissingRegistry, 46, 108, None, None),
    ] {
        let (mut runtime, value) = fixture(kind);
        let retained = (runtime.objects.len(), runtime.native_properties.len());
        runtime.steps = work;
        runtime.allocated = MAX_HEAP - heap;
        let metadata = runtime.throw_metadata(&value).unwrap();
        assert_eq!(metadata.name.as_deref(), display, "{kind:?}");
        assert_eq!(metadata.intrinsic, intrinsic, "{kind:?}");
        assert_eq!(
            (runtime.steps, runtime.allocated),
            (0, MAX_HEAP),
            "{kind:?}"
        );
        assert_eq!(
            (runtime.objects.len(), runtime.native_properties.len()),
            retained
        );
        assert!(runtime.console.is_empty());

        let (mut runtime, value) = fixture(kind);
        runtime.steps = work - 1;
        runtime.allocated = 0;
        let Err(error) = runtime.throw_metadata(&value) else {
            panic!("work refusal {kind:?}")
        };
        assert!(error.is_resource_limit());
        assert_eq!(runtime.steps, 0);
        assert!(runtime.console.is_empty());

        if heap != 0 {
            let (mut runtime, value) = fixture(kind);
            runtime.steps = work;
            runtime.allocated = MAX_HEAP - heap + 1;
            let Err(error) = runtime.throw_metadata(&value) else {
                panic!("heap refusal {kind:?}")
            };
            assert!(error.is_resource_limit());
            assert_eq!(runtime.allocated, MAX_HEAP + 1);
            assert!(runtime.console.is_empty());
        }
    }
    for (work, heap, succeeds) in [
        (2122, 98516, true),
        (2121, 98516, false),
        (2122, 98515, false),
    ] {
        let (mut runtime, value) = fixture(Fixture::LargeName);
        runtime.steps = work;
        runtime.allocated = MAX_HEAP - heap;
        let result = runtime.throw_metadata(&value);
        if succeeds {
            let metadata = result.unwrap();
            assert_eq!(metadata.name.as_deref(), Some("x".repeat(32768).as_str()));
            assert_eq!(metadata.intrinsic, None);
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        } else {
            let Err(error) = result else {
                panic!("large copy refusal")
            };
            assert!(error.is_resource_limit());
        }
    }
    // Existing throw-formatting fee is an additional1 work/512 heap for an object.
    let (mut runtime, value) = fixture(Fixture::Empty);
    runtime.steps = 47;
    runtime.allocated = MAX_HEAP - 620;
    let metadata = runtime.throw_metadata(&value).unwrap();
    assert!(metadata.name.is_none() && metadata.intrinsic.is_none());
    let error = runtime.thrown_error(value.clone()).unwrap();
    let ErrorKind::Thrown(actual) = error.kind else {
        panic!("original payload")
    };
    assert_eq!(actual, value);
    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    // A private cyclic fixture uses the existing96-edge ceiling; it invokes no author code.
    let (mut runtime, value) = fixture(Fixture::Empty);
    let Value::Object(id) = value else {
        unreachable!()
    };
    runtime.objects[id].prototype = Some(Value::Object(id));
    runtime.steps = 806;
    runtime.allocated = MAX_HEAP - 108;
    let Err(error) = runtime.throw_metadata(&Value::Object(id)) else {
        panic!("cycle limit")
    };
    assert!(error.is_resource_limit());
    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
}
