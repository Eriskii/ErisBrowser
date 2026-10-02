use super::*;

const LOCAL: &str = include_str!("../../../tests/conformance/object-is-local.js");

fn fresh() -> (Runtime, Document) {
    (Runtime::new(), Document::parse("<p>kept</p>"))
}

fn clean(runtime: &Runtime) {
    assert_eq!(runtime.calls, 0);
    assert_eq!(runtime.stack_units, 0);
    assert_eq!(runtime.eval_depth, 0);
    assert_eq!(runtime.json_depth, 0);
    assert!(runtime.frames.is_empty());
    assert!(runtime.active_array_joins.is_empty());
}

fn helpers(runtime: &mut Runtime, doc: &mut Document) {
    for source in [
        include_str!("../../../tests/upstream/test262/harness/assert.js"),
        include_str!("../../../tests/upstream/test262/harness/sta.js"),
    ] {
        runtime.execute(source, doc).unwrap();
    }
}

fn case(name: &str) -> &'static str {
    LOCAL
        .split("// CASE ")
        .skip(1)
        .find_map(|part| {
            let (header, source) = part.split_once('\n').unwrap();
            (header.split_once(' ').unwrap().0 == name).then_some(source)
        })
        .unwrap()
}

#[test]
fn object_is_number_literals_include_nan_payloads_and_zero_signs() {
    let (mut runtime, _) = fresh();
    for (a, b, expected) in [
        (0.0, 0.0, true),
        (-0.0, -0.0, true),
        (0.0, -0.0, false),
        (-0.0, 0.0, false),
        (f64::NAN, f64::from_bits(0xfff0_0000_0000_0001), true),
        (f64::NAN, 0.0, false),
        (f64::INFINITY, f64::INFINITY, true),
        (f64::INFINITY, f64::NEG_INFINITY, false),
        (f64::from_bits(1), 0.0, false),
        (f64::from_bits(1), f64::from_bits(1), true),
        (f64::MAX, f64::MAX, true),
        (1.0, 1.5, false),
    ] {
        assert_eq!(
            runtime
                .object_is_values(&Value::Number(a), &Value::Number(b))
                .unwrap(),
            expected,
            "{a:?}, {b:?}"
        );
    }
}

#[test]
fn object_is_scalar_and_arena_identity_lattice() {
    let (mut runtime, _) = fresh();
    let symbol = Symbol::unique(Some("same".into()));
    let values = [
        Value::Undefined,
        Value::Null,
        Value::Bool(false),
        Value::Bool(true),
        Value::Number(0.0),
        Value::Number(-0.0),
        Value::Number(f64::NAN),
        Value::String("0".into()),
        Value::Array(0),
        Value::Array(1),
        Value::Object(0),
        Value::Object(1),
        Value::Function(0),
        Value::Function(1),
        Value::Node(0),
        Value::Node(1),
        Value::Document,
        Value::Window,
        Value::Console,
        Value::Math,
        Value::Json,
        Value::Style(0),
        Value::Style(1),
        Value::ClassList(0),
        Value::ClassList(1),
        Value::Symbol(symbol.clone()),
        Value::Symbol(Symbol::unique(Some("same".into()))),
    ];
    for (i, a) in values.iter().enumerate() {
        for (j, b) in values.iter().enumerate() {
            assert_eq!(runtime.object_is_values(a, b).unwrap(), i == j, "{i}, {j}");
        }
    }
    assert!(
        runtime
            .object_is_values(&values[25], &Value::Symbol(symbol))
            .unwrap()
    );
}

#[test]
fn object_is_utf16_units_not_display_text_or_normalization() {
    let (mut runtime, _) = fresh();
    for (a, b, expected) in [
        (vec![], vec![], true),
        (vec![0xd800], vec![0xd800], true),
        (vec![0xd800], vec![0xdc00], false),
        (vec![0xd800], vec![0xfffd], false),
        (vec![0x61, 0, 0x62], vec![0x61, 0, 0x62], true),
        (vec![0x61, 0, 0x62], vec![0x61, 0x62], false),
        (vec![0xe9], vec![0x65, 0x301], false),
        (vec![0xd800, 0xdc00], vec![0xd800, 0xdc00], true),
    ] {
        let a = Value::String(a.into());
        let b = Value::String(b.into());
        assert_eq!(runtime.object_is_values(&a, &b).unwrap(), expected);
    }
}

#[test]
fn object_is_string_work_exact_one_short_and_full_heap() {
    for length in [0, 7, 8, 9, 4096, MAX_STRING] {
        let a = Value::String(vec![0xd800; length].into());
        let b = Value::String(vec![0xd800; length].into());
        let work = 4 + 2 + 2 * (length / 8);
        for alias in [false, true] {
            let right = if alias { &a } else { &b };
            for available in [work - 1, work] {
                let (mut runtime, _) = fresh();
                runtime.steps = available;
                runtime.allocated = MAX_HEAP;
                let result = runtime.object_is_values(&a, right);
                if available == work {
                    assert!(result.unwrap());
                } else {
                    assert!(result.unwrap_err().is_resource_limit());
                }
                assert_eq!(runtime.steps, 0);
                assert_eq!(runtime.allocated, MAX_HEAP);
                clean(&runtime);
            }
        }
    }
}

#[test]
fn object_is_unequal_length_and_type_skip_payload_work() {
    let a = Value::String(vec![0xd800; MAX_STRING].into());
    let b = Value::String(vec![0xd800; MAX_STRING - 1].into());
    for right in [b, Value::Null, Value::Object(0), Value::Number(0.0)] {
        let (mut runtime, _) = fresh();
        runtime.steps = 4;
        runtime.allocated = MAX_HEAP;
        assert!(!runtime.object_is_values(&a, &right).unwrap());
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, MAX_HEAP);
    }
}

#[test]
fn object_is_dispatch_ignores_receiver_and_extra_payloads() {
    let (mut runtime, mut doc) = fresh();
    let ignored = Value::String(vec![0xd800; MAX_STRING].into());
    let native = Native {
        name: "Object.is".into(),
        receiver: ignored.clone(),
    };
    runtime.steps = 4;
    runtime.allocated = MAX_HEAP;
    assert_eq!(
        runtime
            .native_call(
                &native,
                vec![
                    Value::Number(1.0),
                    Value::Number(1.0),
                    ignored,
                    Value::Array(usize::MAX)
                ],
                &mut doc
            )
            .unwrap(),
        Value::Bool(true)
    );
    assert_eq!(runtime.steps, 0);
    assert_eq!(runtime.allocated, MAX_HEAP);
    clean(&runtime);
}

#[test]
fn object_is_missing_arguments_and_scalar_cutpoint() {
    let (mut runtime, _) = fresh();
    for (arguments, expected) in [
        (vec![], true),
        (vec![Value::Undefined], true),
        (vec![Value::Null], false),
        (vec![Value::Null, Value::Null], true),
    ] {
        runtime.steps = 4;
        assert_eq!(
            runtime.object_is(&arguments).unwrap(),
            Value::Bool(expected)
        );
        assert_eq!(runtime.steps, 0);
        runtime.steps = 3;
        assert!(
            runtime
                .object_is(&arguments)
                .unwrap_err()
                .is_resource_limit()
        );
    }
}

#[test]
fn object_is_native_top_level_reflexivity_is_scoped_and_paid() {
    let (mut runtime, _) = fresh();
    let inner = Runtime::native("private", Value::Number(f64::NAN));
    runtime.steps = 4;
    assert!(
        runtime
            .object_is_values(&inner, &inner)
            .unwrap_err()
            .is_resource_limit()
    );
    runtime.steps = 5;
    assert!(runtime.object_is_values(&inner, &inner).unwrap());
    assert_eq!(runtime.steps, 0);
    let left = Runtime::native("outer", inner.clone());
    let right = Runtime::native("outer", inner);
    runtime.steps = MAX_STEPS;
    assert!(!runtime.object_is_values(&left, &right).unwrap());
    assert!(runtime.object_is_values(&left, &left).unwrap());
    for (a, b, expected) in [(0.0, -0.0, true), (f64::NAN, f64::NAN, false)] {
        assert_eq!(
            runtime
                .object_is_values(
                    &Runtime::native("key", Value::Number(a)),
                    &Runtime::native("key", Value::Number(b))
                )
                .unwrap(),
            expected
        );
    }
}

#[test]
fn object_is_native_names_and_receiver_strings_are_prepaid() {
    let name = "n".repeat(4096);
    let a = Runtime::native(&name, Value::String(vec![0xd800; 4096].into()));
    let b = Runtime::native(&name, Value::String(vec![0xd800; 4096].into()));
    let work = 4 + 1 + 1026 + 1026;
    for available in [4, 4 + 1 + 1026 - 1, work - 1, work] {
        let (mut runtime, _) = fresh();
        runtime.steps = available;
        runtime.allocated = MAX_HEAP;
        let result = runtime.object_is_values(&a, &b);
        if available == work {
            assert!(result.unwrap());
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, MAX_HEAP);
    }
    let (mut runtime, _) = fresh();
    runtime.steps = 5;
    assert!(
        !runtime
            .object_is_values(&a, &Runtime::native("short", Value::Undefined))
            .unwrap()
    );
    assert_eq!(runtime.steps, 0);
}

#[test]
fn object_is_native_key_walk_is_iterative_and_stops_at_budget() {
    let (mut a, mut b) = (Value::Undefined, Value::Undefined);
    for _ in 0..256 {
        a = Runtime::native("link", a);
        b = Runtime::native("link", b);
    }
    let work = 4 + 256 * 3;
    for available in [7, work - 1, work] {
        let (mut runtime, _) = fresh();
        runtime.steps = available;
        runtime.allocated = MAX_HEAP;
        let result = runtime.object_is_values(&a, &b);
        if available == work {
            assert!(result.unwrap());
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, MAX_HEAP);
        clean(&runtime);
    }
}

#[test]
fn object_is_frozen_ordinary_sources_keep_independent_expectations() {
    let mut modes = 0;
    for part in LOCAL.split("// CASE ").skip(1) {
        let (header, source) = part.split_once('\n').unwrap();
        let name = header.split_once(' ').unwrap().0;
        if header.ends_with("[resource]")
            || matches!(
                name,
                "proxy-identity-no-traps"
                    | "bigint-identity-and-number-distinction"
                    | "foreign-realm-object-identity"
            )
            || name.starts_with("dom-defining-interface-")
        {
            continue;
        }
        for strict in [false, true] {
            let (mut runtime, mut doc) = fresh();
            helpers(&mut runtime, &mut doc);
            let result = if strict {
                runtime.execute_strict(source, &mut doc)
            } else {
                runtime.execute(source, &mut doc)
            };
            assert!(result.is_ok(), "{name}, strict={strict}: {result:?}");
            clean(&runtime);
            modes += 1;
        }
    }
    assert_eq!(modes, 64);
}

#[test]
fn object_is_frozen_policy_prefix_reaches_successful_comparison() {
    for name in [
        "terminal-repeated-string-comparison",
        "terminal-string-budget-not-catchable",
    ] {
        let source = case(name);
        let success = "assert.sameValue(Object.is(a,b),true);";
        let end = source.find(success).unwrap() + success.len();
        let prefix = &source[..end];
        assert!(!prefix.contains("for(var i=0;i<200;i++)"));
        for strict in [false, true] {
            let (mut runtime, mut doc) = fresh();
            helpers(&mut runtime, &mut doc);
            let result = if strict {
                runtime.execute_strict(prefix, &mut doc)
            } else {
                runtime.execute(prefix, &mut doc)
            };
            assert!(result.is_ok(), "{name}, strict={strict}: {result:?}");
            clean(&runtime);
        }
    }
}

#[test]
fn object_is_frozen_policy_loops_keep_terminal_outcomes() {
    for name in [
        "terminal-repeated-string-comparison",
        "terminal-string-budget-not-catchable",
    ] {
        for strict in [false, true] {
            let (mut runtime, mut doc) = fresh();
            helpers(&mut runtime, &mut doc);
            let result = if strict {
                runtime.execute_strict(case(name), &mut doc)
            } else {
                runtime.execute(case(name), &mut doc)
            };
            assert!(
                result.unwrap_err().is_resource_limit(),
                "{name}, strict={strict}"
            );
            clean(&runtime);
        }
    }
}

#[test]
fn object_is_terminal_comparison_preserves_argument_effects_and_cleans_frames() {
    let (mut runtime, mut doc) = fresh();
    runtime
        .execute(
            "var seen='';var caught=false;var finalized=false;",
            &mut doc,
        )
        .unwrap();
    for key in ["a", "b"] {
        runtime
            .define(0, key, Value::String(vec![0x78; 8192].into()), true)
            .unwrap();
    }
    let unit = parser::Parser::program("try{Object.is((seen+='A',a),(seen+='B',b),(seen+='E',0));}catch(e){caught=true;}finally{finalized=true;}").unwrap();
    runtime.steps = 1000;
    let error = machine::evaluate_statements(
        &mut runtime,
        &unit,
        machine::ListOwner::Program,
        1,
        &mut doc,
    )
    .err()
    .unwrap();
    assert!(error.is_resource_limit());
    assert_eq!(
        runtime.environments[0].bindings["seen"].value,
        Value::String("ABE".into())
    );
    assert_eq!(
        runtime.environments[0].bindings["caught"].value,
        Value::Bool(false)
    );
    assert_eq!(
        runtime.environments[0].bindings["finalized"].value,
        Value::Bool(false)
    );
    clean(&runtime);
}

// Seed only the pre-install state; all measured mutations use the real helper.
fn installer_setup(grow: bool) -> (Runtime, usize) {
    let (mut runtime, _) = fresh();
    let owner = runtime.native_properties["Object"];
    runtime.native_properties.remove("Object.is").unwrap();
    runtime.objects[owner].remove(&PropertyKey::from("is"));
    if grow {
        let bag = &mut runtime.objects[owner];
        while bag.order.len() < bag.order.capacity() {
            bag.insert(format!("fill{}", bag.order.len()).into(), Value::Null);
        }
        assert_eq!(bag.order.len(), bag.order.capacity());
    } else {
        runtime.objects[owner].order.try_reserve_exact(1).unwrap();
        assert!(runtime.objects[owner].order.len() < runtime.objects[owner].order.capacity());
    }
    (runtime, owner)
}

#[test]
fn object_is_installer_full_admission_exact_and_one_short() {
    for grow in [false, true] {
        let (mut measure, _) = installer_setup(grow);
        let before = (measure.steps, measure.allocated);
        measure.install_object_is_intrinsic().unwrap();
        let work = before.0 - measure.steps;
        let heap = measure.allocated - before.1;
        println!("OBJECT_IS_INSTALL grow={grow} work={work} heap={heap}");
        for (available_work, available_heap, success) in [
            (work - 1, heap, false),
            (work, heap - 1, false),
            (work, heap, true),
        ] {
            let (mut runtime, owner) = installer_setup(grow);
            let order = runtime.objects[owner].order.clone();
            let registry = runtime.native_properties.clone();
            let objects = runtime.objects.len();
            runtime.steps = available_work;
            runtime.allocated = MAX_HEAP - available_heap;
            let result = runtime.install_object_is_intrinsic();
            if success {
                result.unwrap();
                assert_eq!(runtime.steps, 0);
                assert_eq!(runtime.allocated, MAX_HEAP);
                assert_eq!(runtime.objects.len(), objects + 1);
                assert_eq!(runtime.native_properties.len(), registry.len() + 1);
                assert_eq!(
                    runtime.objects[owner].order.last(),
                    Some(&PropertyKey::from("is"))
                );
                assert_eq!(
                    &runtime.objects[owner].order[..order.len()],
                    order.as_slice()
                );
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert_eq!(runtime.native_properties, registry);
                assert_eq!(runtime.objects.len(), objects);
                assert_eq!(runtime.objects[owner].order, order);
                assert!(!runtime.objects[owner].contains_key("is"));
            }
        }
    }
}

#[test]
fn object_is_owner_reserve_prepays_full_buffer_before_mutation() {
    let (mut measure, owner) = installer_setup(true);
    let len = measure.objects[owner].order.len();
    let requested = (2 * len).max(4).max(len + 1);
    let before = (measure.steps, measure.allocated);
    measure.object_is_reserve_order(owner).unwrap();
    assert_eq!(before.0 - measure.steps, 1 + 2 * len);
    assert_eq!(
        measure.allocated - before.1,
        requested * std::mem::size_of::<PropertyKey>()
    );
    for (work, heap) in [
        (2 * len, requested * std::mem::size_of::<PropertyKey>()),
        (
            1 + 2 * len,
            requested * std::mem::size_of::<PropertyKey>() - 1,
        ),
    ] {
        let (mut runtime, owner) = installer_setup(true);
        let order = runtime.objects[owner].order.clone();
        let capacity = runtime.objects[owner].order.capacity();
        runtime.steps = work;
        runtime.allocated = MAX_HEAP - heap;
        assert!(
            runtime
                .object_is_reserve_order(owner)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.objects[owner].order, order);
        assert_eq!(runtime.objects[owner].order.capacity(), capacity);
    }
    let (mut runtime, owner) = installer_setup(false);
    runtime.steps = 0;
    runtime.allocated = MAX_HEAP;
    runtime.object_is_reserve_order(owner).unwrap();
    assert_eq!(runtime.steps, 0);
    assert_eq!(runtime.allocated, MAX_HEAP);
}

#[test]
fn object_is_installed_descriptors_and_creation_order() {
    let (runtime, _) = fresh();
    let owner = runtime.native_properties["Object"];
    let id = runtime.native_properties["Object.is"];
    let bag = &runtime.objects[id];
    assert_eq!(
        bag.prototype,
        Some(Value::Function(runtime.function_prototype))
    );
    assert_eq!(
        bag.order,
        [PropertyKey::from("name"), PropertyKey::from("length")]
    );
    assert!(!bag.contains_key("prototype"));
    for (key, expected) in [
        ("name", Value::String("is".into())),
        ("length", Value::Number(2.0)),
    ] {
        let property = &bag.values[&PropertyKey::from(key)];
        assert!(!property.enumerable && property.configurable);
        let PropertyValue::Data { value, writable } = &property.value else {
            panic!()
        };
        assert!(!writable);
        assert_eq!(value, &expected);
    }
    let property = &runtime.objects[owner].values[&PropertyKey::from("is")];
    assert!(!property.enumerable && property.configurable);
    let PropertyValue::Data { value, writable } = &property.value else {
        panic!()
    };
    assert!(*writable);
    let Value::Native(native) = value else {
        panic!()
    };
    assert_eq!(native.name, "Object.is");
    assert_eq!(native.receiver, Value::Undefined);
    assert_eq!(
        runtime.objects[owner]
            .order
            .iter()
            .filter(|key| **key == PropertyKey::from("is"))
            .count(),
        1
    );
}

#[test]
fn object_is_repeated_generic_install_keeps_identity_bag_and_order() {
    let (mut runtime, _) = fresh();
    let owner = runtime.native_properties["Object"];
    let id = runtime.native_properties["Object.is"];
    let order = runtime.objects[owner].order.clone();
    let objects = runtime.objects.len();
    let native_count = runtime.native_properties.len();
    let saved = runtime.objects[owner].get("is").unwrap().clone();
    runtime.install_object_is_intrinsic().unwrap();
    assert_eq!(runtime.native_properties["Object.is"], id);
    assert_eq!(runtime.native_properties.len(), native_count);
    assert_eq!(runtime.objects.len(), objects);
    assert_eq!(runtime.objects[owner].order, order);
    let current = runtime.objects[owner].get("is").unwrap().clone();
    assert!(runtime.object_is_values(&saved, &current).unwrap());
}

#[test]
fn object_is_tree_and_actual_type_storage_bounds() {
    for (count, expected) in [
        (0, (0, 0)),
        (1, (1, 1)),
        (10, (10, 1)),
        (11, (11, 2)),
        (12, (12, 2)),
        (70, (22, 2)),
        (71, (33, 3)),
        (72, (33, 3)),
    ] {
        assert_eq!(tree_bound(count), expected);
        assert_eq!(insertion_work(count), (expected.1 + 1) * 28);
        assert_eq!(
            insertion_bytes::<String, usize>(count).unwrap(),
            (expected.1 + 1)
                * (16 * (std::mem::size_of::<String>() + std::mem::size_of::<usize>())
                    + 32 * std::mem::size_of::<usize>()
                    + 64)
        );
    }
    let width = std::mem::size_of::<PropertyKey>();
    assert!((2..=1024).contains(&width));
    let (runtime, _) = fresh();
    let bag = &runtime.objects[runtime.native_properties["Object.is"]];
    // Pinned RawVec's logical first request is four slots. Physical allocator
    // rounding is not asserted to equal the logical request.
    assert!(bag.order.capacity() >= 4);
}

#[test]
fn object_is_bootstrap_reports_actual_charged_budget() {
    let runtime = Runtime::uninitialized().finish_bootstrap().unwrap();
    let owner = runtime.native_properties["Object"];
    println!(
        "OBJECT_IS_BOOTSTRAP remaining_steps={} allocated={} native_properties={} prototypes={} owner_order_capacity={}",
        runtime.steps,
        runtime.allocated,
        runtime.native_properties.len(),
        runtime.prototypes.len(),
        runtime.objects[owner].order.capacity()
    );
    assert!(runtime.steps < MAX_STEPS);
    assert!(runtime.allocated < MAX_HEAP);
}
