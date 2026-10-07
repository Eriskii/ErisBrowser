use super::*;

fn fresh() -> (Runtime, Document) {
    (Runtime::try_new().unwrap(), Document::parse(""))
}
fn clean(runtime: &Runtime) {
    assert!(runtime.frames.is_empty());
    assert_eq!(
        (runtime.calls, runtime.eval_depth, runtime.stack_units),
        (0, 0, 0)
    );
}
fn invoke(runtime: &mut Runtime, doc: &mut Document, target: Value, key: Value) -> Result<Value> {
    runtime.object_has_own(&[target, key], doc)
}

#[test]
fn object_has_own_bootstrap_reports_actual_admission_and_types() {
    let runtime = Runtime::uninitialized().finish_bootstrap().unwrap();
    println!(
        "OBJECT_HAS_OWN_BOOTSTRAP steps={} allocated={} objects={} capacity={} native={} prototypes={}",
        runtime.steps,
        runtime.allocated,
        runtime.objects.len(),
        runtime.objects.capacity(),
        runtime.native_properties.len(),
        runtime.prototypes.len()
    );
    println!(
        "OBJECT_HAS_OWN_TYPES native={} legacy_metadata={} key={} entry={} block={} object={}",
        std::mem::size_of::<Native>(),
        NATIVE_METADATA_BYTES,
        std::mem::size_of::<PropertyKey>(),
        std::mem::size_of::<(PropertyKey, Property)>(),
        16 * (std::mem::size_of::<PropertyKey>() + std::mem::size_of::<Property>())
            + 32 * std::mem::size_of::<usize>()
            + 64,
        std::mem::size_of::<ScriptObject>()
    );
    assert_eq!(runtime.objects.len(), dom_prototypes::BOOTSTRAP_OBJECTS);
    assert!(!runtime.native_properties.contains_key("Object.hasOwn"));
    let owner = runtime.native_properties["Object"];
    println!(
        "OBJECT_HAS_OWN_OWNER entries={} order={} capacity={}",
        runtime.objects[owner].values.len(),
        runtime.objects[owner].order.len(),
        runtime.objects[owner].order.capacity()
    );
    let PropertyValue::Data {
        value: Value::Native(method),
        writable: true,
    } = &runtime.objects[owner].values[&PropertyKey::from("hasOwn")].value
    else {
        panic!("ordinary static method")
    };
    assert_eq!(method.name, "Object.hasOwn");
    assert!(method.properties.is_some());
    assert!(runtime.steps > 0 && runtime.allocated < MAX_HEAP);
    clean(&runtime);
}

#[test]
fn object_has_own_direct_exotic_presence_and_heap_free_object_paths() {
    let (mut runtime, mut doc) = fresh();
    let array = runtime.array(vec![Value::Undefined, Value::Null]).unwrap();
    let Value::Array(a) = array else {
        unreachable!()
    };
    runtime.array_holes[a].insert(0);
    let boxed = runtime
        .coerce_object(Value::String(vec![0xd800, 65].into()))
        .unwrap();
    let symbol = Symbol::unique(Some("same".into()));
    let ordinary = runtime.object_ordered([]).unwrap();
    let Value::Object(o) = ordinary else {
        unreachable!()
    };
    runtime.objects[o].insert_property(
        PropertyKey::Symbol(symbol.clone()),
        Property::data(Value::Undefined, true, true, true),
    );
    runtime.allocated = MAX_HEAP;
    for (target, key, expected) in [
        (array.clone(), Value::String("0".into()), false),
        (array.clone(), Value::String("1".into()), true),
        (array, Value::String("length".into()), true),
        (boxed.clone(), Value::String("0".into()), true),
        (boxed.clone(), Value::String("1".into()), true),
        (boxed.clone(), Value::String("2".into()), false),
        (boxed, Value::String("-0".into()), false),
        (ordinary.clone(), Value::Symbol(symbol), true),
        (
            ordinary,
            Value::Symbol(Symbol::unique(Some("same".into()))),
            false,
        ),
    ] {
        runtime.steps = MAX_STEPS;
        assert_eq!(
            invoke(&mut runtime, &mut doc, target, key).unwrap(),
            Value::Bool(expected)
        );
        assert_eq!(runtime.allocated, MAX_HEAP);
    }
    clean(&runtime);
}

#[test]
fn object_has_own_small_body_exhaustive_work_cuts_and_exact_heap() {
    for primitive in [false, true] {
        let (mut measured, mut doc) = fresh();
        let target = if primitive {
            Value::String("xy".into())
        } else {
            measured
                .object_ordered([("x".into(), Value::Undefined)])
                .unwrap()
        };
        let key = Value::String(if primitive { "1" } else { "x" }.into());
        measured.steps = MAX_STEPS;
        let start = measured.allocated;
        assert_eq!(
            invoke(&mut measured, &mut doc, target, key).unwrap(),
            Value::Bool(true)
        );
        let work = MAX_STEPS - measured.steps;
        let heap = measured.allocated - start;
        assert!(work > 0 && work < 4000);
        let cuts = if primitive {
            vec![work - 1, work]
        } else {
            (0..=work).collect()
        };
        for available in cuts {
            let (mut runtime, mut doc) = fresh();
            let target = if primitive {
                Value::String("xy".into())
            } else {
                runtime
                    .object_ordered([("x".into(), Value::Undefined)])
                    .unwrap()
            };
            runtime.steps = available;
            let result = invoke(
                &mut runtime,
                &mut doc,
                target,
                Value::String(if primitive { "1" } else { "x" }.into()),
            );
            if available == work {
                assert_eq!(result.unwrap(), Value::Bool(true));
            } else {
                assert!(result.unwrap_err().is_resource_limit());
            }
            assert_eq!(runtime.steps, 0);
            clean(&runtime);
        }
        for short in 0..=usize::from(heap > 0) {
            let (mut runtime, mut doc) = fresh();
            let target = if primitive {
                Value::String("xy".into())
            } else {
                runtime
                    .object_ordered([("x".into(), Value::Undefined)])
                    .unwrap()
            };
            runtime.steps = MAX_STEPS;
            runtime.allocated = MAX_HEAP - heap + short;
            let result = invoke(
                &mut runtime,
                &mut doc,
                target,
                Value::String(if primitive { "1" } else { "x" }.into()),
            );
            if short == 0 {
                assert_eq!(result.unwrap(), Value::Bool(true));
                assert_eq!(runtime.allocated, MAX_HEAP);
            } else {
                assert!(result.unwrap_err().is_resource_limit());
            }
            clean(&runtime);
        }
    }
}

#[test]
fn object_has_own_primitive_growth_admission_and_nullish_key_suppression() {
    let (mut runtime, mut doc) = fresh();
    runtime.objects.shrink_to_fit();
    let count = runtime.objects.len();
    let cap = runtime.objects.capacity();
    runtime.allocated = MAX_HEAP;
    assert!(
        invoke(
            &mut runtime,
            &mut doc,
            Value::Bool(true),
            Value::String("x".into())
        )
        .unwrap_err()
        .is_resource_limit()
    );
    assert_eq!(
        (runtime.objects.len(), runtime.objects.capacity()),
        (count, cap)
    );
    for target in [Value::Null, Value::Undefined] {
        runtime.steps = MAX_STEPS;
        runtime.allocated = MAX_HEAP;
        let error = invoke(&mut runtime, &mut doc, target, Value::Object(usize::MAX)).unwrap_err();
        assert_eq!(error.name(), "TypeError");
        assert_eq!(runtime.allocated, MAX_HEAP);
    }
    clean(&runtime);
}

#[test]
fn object_has_own_saved_native_boundary_and_cleanup() {
    let (mut measured, mut doc) = fresh();
    let method = measured.execute("Object.hasOwn", &mut doc).unwrap();
    let target = measured
        .object_ordered([("x".into(), Value::Null)])
        .unwrap();
    measured.steps = MAX_STEPS;
    let initial = measured.allocated;
    assert_eq!(
        measured
            .call(
                method,
                vec![target, Value::String("x".into())],
                Value::Null,
                &mut doc
            )
            .unwrap(),
        Value::Bool(true)
    );
    let work = MAX_STEPS - measured.steps;
    let heap = measured.allocated - initial;
    for short in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let method = runtime.execute("Object.hasOwn", &mut doc).unwrap();
        let target = runtime.object_ordered([("x".into(), Value::Null)]).unwrap();
        runtime.steps = work - usize::from(short);
        runtime.allocated = MAX_HEAP - heap;
        let result = runtime.call(
            method,
            vec![target, Value::String("x".into())],
            Value::Undefined,
            &mut doc,
        );
        if short {
            assert!(result.unwrap_err().is_resource_limit());
        } else {
            assert_eq!(result.unwrap(), Value::Bool(true));
        }
        assert_eq!(runtime.steps, 0);
        clean(&runtime);
    }
}

#[test]
fn object_has_own_terminal_key_effects_share_budget_and_suppress_handlers() {
    let (mut runtime, mut doc) = fresh();
    let source = "var hasOwnTrace='';var hasOwnKey={};hasOwnKey[Symbol.toPrimitive]=function(hint){hasOwnTrace+='k';document.title='prefix';while(true){};};try{Object.hasOwn({},hasOwnKey);}catch(e){hasOwnTrace+='c';}finally{hasOwnTrace+='f';}";
    assert!(
        runtime
            .execute(source, &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    clean(&runtime);
    assert_eq!(
        runtime.execute("hasOwnTrace", &mut doc).unwrap(),
        Value::String("k".into())
    );
    assert_eq!(doc.title(), "prefix");
}

#[test]
fn object_has_own_two_live_realms_keep_independent_method_bags() {
    let (mut a, mut da) = fresh();
    let (mut b, mut db) = fresh();
    assert_eq!(a.execute("Object.hasOwn.marker=41;Object.defineProperty(Object.hasOwn,'length',{value:7});Object.hasOwn.marker",&mut da).unwrap(),Value::Number(41.0));
    assert_eq!(
        b.execute(
            "Object.hasOwn.length===2&&Object.hasOwn.marker===undefined",
            &mut db
        )
        .unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        b.execute("Object.hasOwn.marker=72;Object.hasOwn.marker", &mut db)
            .unwrap(),
        Value::Number(72.0)
    );
    assert_eq!(
        a.execute(
            "Object.hasOwn.length===7&&Object.hasOwn.marker===41",
            &mut da
        )
        .unwrap(),
        Value::Bool(true)
    );
}

const CASES: &str = include_str!("../../../tests/fixtures/object-has-own.js");
fn independent(name: &str) {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = format!("{CASES}\nobjectHasOwnCases.{name}();");
        let value = if strict {
            runtime.execute_strict(&source, &mut doc)
        } else {
            runtime.execute(&source, &mut doc)
        };
        assert_eq!(value.unwrap(), Value::Bool(true), "{name} strict={strict}");
        clean(&runtime);
    }
}
macro_rules! cases { ($($name:ident),* $(,)?) => { $(#[test] fn $name() { independent(stringify!($name)); })* }; }
cases!(
    ordinary_metadata_saved_method_and_ignored_this,
    target_before_key_conversion_and_fresh_lookup,
    descriptor_presence_without_getters_or_target_methods,
    primitive_wrappers_and_exact_string_indices,
    array_holes_accessors_and_noncanonical_names,
    symbols_exact_keys_and_function_metadata,
    dom_expandos_are_separate_from_inherited_members,
    window_global_properties_are_not_lexical_bindings,
);

#[test]
fn object_has_own_final_initializer_exact_and_one_short_admission() {
    let before = Runtime::uninitialized();
    let initial_steps = before.steps;
    let initial_heap = before.allocated;
    let measured = before.finish_bootstrap().unwrap();
    let work = initial_steps - measured.steps;
    let heap = measured.allocated - initial_heap;
    for short_work in [false, true] {
        let mut runtime = Runtime::uninitialized();
        runtime.steps = work - usize::from(short_work);
        runtime.allocated = MAX_HEAP - heap;
        let result = runtime.finish_bootstrap();
        if short_work {
            assert!(result.err().unwrap().is_resource_limit());
        } else {
            let runtime = result.unwrap();
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            let owner = runtime.native_properties["Object"];
            assert!(
                runtime.objects[owner]
                    .values
                    .contains_key(&PropertyKey::from("hasOwn"))
            );
        }
    }
    let mut runtime = Runtime::uninitialized();
    runtime.allocated = MAX_HEAP - heap + 1;
    assert!(
        runtime
            .finish_bootstrap()
            .err()
            .unwrap()
            .is_resource_limit()
    );
}

#[test]
fn object_has_own_large_nonindex_key_skips_unbounded_numeric_scan() {
    let (mut runtime, mut doc) = fresh();
    let boxed = runtime.coerce_object(Value::String("x".into())).unwrap();
    let key = Value::String(vec![49; MAX_STRING].into());
    runtime.steps = 47;
    runtime.allocated = MAX_HEAP;
    assert_eq!(
        invoke(&mut runtime, &mut doc, boxed.clone(), key.clone()).unwrap(),
        Value::Bool(false)
    );
    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    runtime.steps = 46;
    assert!(
        invoke(&mut runtime, &mut doc, boxed, key)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    clean(&runtime);
}
