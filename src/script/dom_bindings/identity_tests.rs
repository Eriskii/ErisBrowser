use super::*;

const LOCAL: &str = include_str!("../../../tests/conformance/dom-method-identity-local.js");

fn fresh() -> (Runtime, Document) {
    (
        Runtime::new(),
        Document::parse("<!doctype html><html><head></head><body></body></html>"),
    )
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
        include_str!("../../../tests/upstream/test262/harness/propertyHelper.js"),
    ] {
        runtime.execute(source, doc).unwrap();
    }
}

fn source(name: &str) -> &'static str {
    LOCAL
        .split("// ")
        .skip(1)
        .find_map(|part| {
            let (header, body) = part.split_once('\n').unwrap();
            (header.split_once(' ').unwrap().0 == name).then_some(body)
        })
        .unwrap()
}

fn run(runtime: &mut Runtime, unit: &Rc<code::Unit>, doc: &mut Document) -> Result<Value> {
    match machine::evaluate_statements(runtime, unit, machine::ListOwner::Program, 1, doc)? {
        Flow::Normal(value) => Ok(value.unwrap_or(Value::Undefined)),
        _ => panic!("private fixture left abrupt control flow"),
    }
}

fn receiver(interface: ParentInterface, doc: &mut Document) -> Value {
    match interface {
        ParentInterface::Document => Value::Document,
        ParentInterface::Element => Value::Node(doc.create_element("section")),
        ParentInterface::DocumentFragment => Value::Node(doc.create_document_fragment()),
    }
}

fn metadata(runtime: &Runtime, full: &str, name: &str, length: usize) {
    let bag = &runtime.objects[runtime.native_properties[full]];
    assert_eq!(
        bag.prototype,
        Some(Value::Function(runtime.function_prototype))
    );
    assert_eq!(
        bag.order,
        [PropertyKey::from("name"), PropertyKey::from("length")]
    );
    assert_eq!(bag.values.len(), 2);
    assert!(!bag.contains_key("prototype"));
    for (key, expected) in [
        ("name", Value::String(name.into())),
        ("length", Value::Number(length as f64)),
    ] {
        let property = &bag.values[&PropertyKey::from(key)];
        assert!(!property.enumerable && property.configurable);
        let PropertyValue::Data { value, writable } = &property.value else {
            panic!()
        };
        assert!(!writable);
        assert_eq!(value, &expected);
    }
}

#[test]
fn dom_parent_nine_keys_metadata_and_legacy_inventory_are_exact() {
    let (runtime, _) = fresh();
    let legacy = [
        ("DOM.getElementById", 1),
        ("DOM.getElementsByTagName", 1),
        ("DOM.getElementsByClassName", 1),
        ("DOM.createElement", 1),
        ("DOM.createTextNode", 1),
        ("DOM.createDocumentFragment", 0),
        ("DOM.getAttribute", 1),
        ("DOM.hasAttribute", 1),
        ("DOM.setAttribute", 2),
        ("DOM.removeAttribute", 1),
        ("DOM.appendChild", 1),
        ("DOM.removeChild", 1),
        ("DOM.remove", 0),
        ("DOM.cloneNode", 0),
        ("DOMTokenList.add", 0),
        ("DOMTokenList.remove", 0),
        ("DOMTokenList.toggle", 1),
        ("DOMTokenList.contains", 1),
    ];
    assert_eq!(METHODS, legacy);
    let mut keys = std::collections::BTreeSet::new();
    let mut ids = std::collections::BTreeSet::new();
    for row in &PARENT_METHODS {
        assert!(keys.insert(row.full));
        assert!(ids.insert(runtime.native_properties[row.full]));
        assert_eq!(row.full.strip_prefix("DOM."), Some(row.suffix));
        assert_eq!(
            parent_method(row.interface, row.operation).unwrap().full,
            row.full
        );
        metadata(&runtime, row.full, row.name, row.length);
    }
    for (full, length) in legacy {
        metadata(&runtime, full, full.rsplit('.').next().unwrap(), length);
    }
    for stale in ["DOM.querySelector", "DOM.querySelectorAll", "DOM.append"] {
        assert!(!runtime.native_properties.contains_key(stale));
    }
    assert_eq!(
        runtime
            .native_properties
            .keys()
            .filter(|name| name.starts_with("DOM.") || name.starts_with("DOMTokenList."))
            .count(),
        27
    );
}

#[test]
fn dom_parent_pinned_leaf_and_payload_bounds_use_actual_types() {
    let names: Vec<_> = PARENT_INSTALL_ORDER
        .iter()
        .map(|&i| PARENT_METHODS[i].full)
        .collect();
    assert!(names.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(names.len(), 9);
    assert!(names.len() < 11); // Pinned B=6 leaf capacity.
    let mut registry = BTreeMap::new();
    for (ordinal, &index) in PARENT_INSTALL_ORDER.iter().enumerate() {
        let row = &PARENT_METHODS[index];
        assert!(row.full.is_ascii() && row.name.is_ascii());
        assert_eq!(
            parent_install_work(ordinal, row),
            64 + row.full.len() + ordinal * (1 + row.full.len() / 8) + 8 + 16
        );
        let Entry::Vacant(entry) = registry.entry(row.full.to_owned()) else {
            panic!()
        };
        entry.insert(ordinal);
        assert_eq!(registry.len(), ordinal + 1);
        assert_eq!(registry.last_key_value().unwrap().0, row.full);
        assert_eq!(
            parent_install_bytes(row),
            1536 + 8 * (row.full.len() + row.name.len())
                + parent_node_bytes::<PropertyKey, Property>()
                + 4 * std::mem::size_of::<PropertyKey>()
                + 72
                + std::mem::size_of::<Option<AbortSlot>>()
                + std::mem::size_of::<Option<f64>>()
                + 552
        );
    }
    assert_eq!(
        parent_node_bytes::<String, usize>(),
        16 * (std::mem::size_of::<String>() + std::mem::size_of::<usize>())
            + 32 * std::mem::size_of::<usize>()
            + 64
    );
    // Five reserved Vec payloads and five final Rc slices, including fixed
    // handles, Vec headers and strong/weak counts. Allocator rounding excluded.
    let shared = 4 * 45
        + 5 * (std::mem::size_of::<JsString>()
            + std::mem::size_of::<Vec<u16>>()
            + 2 * std::mem::size_of::<usize>());
    assert!(shared <= 512);
    for literal in [
        "name",
        "length",
        "querySelector",
        "querySelectorAll",
        "append",
    ] {
        assert!(literal.is_ascii());
        assert_eq!(parent_ascii(literal).unwrap().to_utf8().unwrap(), literal);
    }
    let (runtime, _) = fresh();
    for row in &PARENT_METHODS {
        assert!(
            runtime.objects[runtime.native_properties[row.full]]
                .order
                .capacity()
                >= 4
        );
    }
}

fn install_setup(ordinal: usize) -> Runtime {
    let (mut runtime, _) = fresh();
    runtime.native_properties.clear();
    // Replay the early installer after a completed bootstrap. Its production
    // slots are prepaid before installation; prepare equivalent spare slots
    // here, outside the work and heap measurements below.
    runtime
        .objects
        .try_reserve_exact(PARENT_INSTALL_ORDER.len())
        .unwrap();
    assert!(runtime.objects.capacity() - runtime.objects.len() >= PARENT_INSTALL_ORDER.len());
    let keys = [PropertyKey::from("name"), PropertyKey::from("length")];
    for (n, &index) in PARENT_INSTALL_ORDER[..ordinal].iter().enumerate() {
        let row = &PARENT_METHODS[index];
        runtime
            .install_dom_parent_method(n, row, &row.name.into(), &keys)
            .unwrap();
    }
    runtime.steps = MAX_STEPS;
    runtime.allocated = 0;
    runtime
}

#[test]
fn dom_parent_installer_exact_and_one_short_admission_for_every_row() {
    for (ordinal, &index) in PARENT_INSTALL_ORDER.iter().enumerate() {
        let row = &PARENT_METHODS[index];
        let keys = [PropertyKey::from("name"), PropertyKey::from("length")];
        let display: JsString = row.name.into();
        let work = parent_install_work(ordinal, row);
        let heap = parent_install_bytes(row);
        let mut measured = install_setup(ordinal);
        measured
            .install_dom_parent_method(ordinal, row, &display, &keys)
            .unwrap();
        assert_eq!(
            (MAX_STEPS - measured.steps, measured.allocated),
            (work, heap)
        );
        for (available_work, available_heap, success) in [
            (work - 1, heap, false),
            (work, heap - 1, false),
            (work, heap, true),
        ] {
            let mut runtime = install_setup(ordinal);
            let registry = runtime.native_properties.clone();
            let len = runtime.objects.len();
            let capacity = runtime.objects.capacity();
            runtime.steps = available_work;
            runtime.allocated = MAX_HEAP - available_heap;
            let result = runtime.install_dom_parent_method(ordinal, row, &display, &keys);
            if success {
                result.unwrap();
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
                assert_eq!(runtime.objects.len(), len + 1);
                assert_eq!(runtime.native_properties.len(), registry.len() + 1);
                metadata(&runtime, row.full, row.name, row.length);
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert_eq!(runtime.native_properties, registry);
                assert_eq!(runtime.objects.len(), len);
            }
            assert_eq!(runtime.objects.capacity(), capacity);
        }
    }
}

fn reserve_setup(length: usize) -> Runtime {
    let (mut runtime, _) = fresh();
    runtime.objects = (0..length)
        .map(|_| ScriptObject::default())
        .collect::<Vec<_>>()
        .into_boxed_slice()
        .into_vec();
    assert_eq!(runtime.objects.capacity(), length);
    runtime.steps = MAX_STEPS;
    runtime.allocated = 0;
    runtime
}

#[test]
fn dom_parent_bootstrap_reserve_is_prepaid_and_spare_capacity_is_free() {
    let bytes = BOOTSTRAP_OBJECT_CAPACITY * std::mem::size_of::<ScriptObject>();
    for length in [0, 25, 256] {
        let work = 1 + 2 * length;
        for (available_work, available_heap, success) in [
            (work - 1, bytes, false),
            (work, bytes - 1, false),
            (work, bytes, true),
        ] {
            let mut runtime = reserve_setup(length);
            let registry = runtime.native_properties.clone();
            runtime.steps = available_work;
            runtime.allocated = MAX_HEAP - available_heap;
            let result = runtime.reserve_bootstrap_objects();
            assert_eq!(runtime.objects.len(), length);
            assert_eq!(runtime.native_properties, registry);
            if success {
                result.unwrap();
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
                assert!(runtime.objects.capacity() >= BOOTSTRAP_OBJECT_CAPACITY);
                runtime.reserve_bootstrap_objects().unwrap();
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert_eq!(runtime.objects.capacity(), length);
            }
        }
    }
}

#[test]
fn dom_parent_batch_refusal_retains_only_prepaid_bootstrap_capacity() {
    let mut runtime = reserve_setup(25);
    runtime.native_properties.clear();
    runtime.reserve_bootstrap_objects().unwrap();
    let capacity = runtime.objects.capacity();
    let row = &PARENT_METHODS[PARENT_INSTALL_ORDER[0]];
    let shared = 512 + parent_node_bytes::<String, usize>();
    runtime.allocated = MAX_HEAP - (shared + parent_install_bytes(row) - 1);
    assert!(
        runtime
            .initialize_dom_parent_bindings()
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.objects.len(), 25);
    assert_eq!(runtime.objects.capacity(), capacity);
    assert!(runtime.native_properties.is_empty());
    assert!(runtime.allocated > MAX_HEAP);
}

#[test]
fn dom_parent_fresh_batch_guard_total_admission_and_partial_publication() {
    let work = PARENT_SHARED_WORK
        + PARENT_INSTALL_ORDER
            .iter()
            .enumerate()
            .map(|(n, &i)| parent_install_work(n, &PARENT_METHODS[i]))
            .sum::<usize>();
    let heap = 512
        + parent_node_bytes::<String, usize>()
        + PARENT_METHODS
            .iter()
            .map(parent_install_bytes)
            .sum::<usize>();
    assert_eq!(work, 1284);
    for (available_work, available_heap, success) in [
        (work - 1, heap, false),
        (work, heap - 1, false),
        (work, heap, true),
    ] {
        let mut runtime = install_setup(0);
        let len = runtime.objects.len();
        runtime.steps = available_work;
        runtime.allocated = MAX_HEAP - available_heap;
        let result = runtime.initialize_dom_parent_bindings();
        let rows = if success { 9 } else { 8 };
        if success {
            result.unwrap();
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        assert_eq!(runtime.objects.len(), len + rows);
        assert_eq!(runtime.native_properties.len(), rows);
        for &i in &PARENT_INSTALL_ORDER[..rows] {
            let row = &PARENT_METHODS[i];
            metadata(&runtime, row.full, row.name, row.length);
        }
        assert_eq!(
            runtime
                .native_properties
                .contains_key(PARENT_METHODS[4].full),
            success
        );
    }
    for invalid_function in [false, true] {
        let mut runtime = install_setup(0);
        if invalid_function {
            runtime.function_prototype = usize::MAX;
        } else {
            runtime.native_properties.insert("occupied".into(), 0);
        }
        let registry = runtime.native_properties.clone();
        let len = runtime.objects.len();
        runtime.steps = 8;
        assert!(
            runtime
                .initialize_dom_parent_bindings()
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!((runtime.steps, runtime.allocated), (0, 0));
        assert_eq!(runtime.native_properties, registry);
        assert_eq!(runtime.objects.len(), len);
    }
}

#[test]
fn dom_parent_row_guard_and_occupied_entry_never_replace_metadata() {
    let keys = [PropertyKey::from("name"), PropertyKey::from("length")];
    let row = &PARENT_METHODS[PARENT_INSTALL_ORDER[0]];
    for wrong_count in [false, true] {
        let mut runtime = install_setup(0);
        if wrong_count {
            runtime.native_properties.insert("occupied".into(), 0);
        }
        let registry = runtime.native_properties.clone();
        let len = runtime.objects.len();
        let selected = if wrong_count { row } else { &PARENT_METHODS[1] };
        assert!(
            runtime
                .install_dom_parent_method(0, selected, &selected.name.into(), &keys)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.native_properties, registry);
        assert_eq!(runtime.objects.len(), len);
        assert_eq!(runtime.allocated, 0);
    }
    let ordinal = 1;
    let row = &PARENT_METHODS[PARENT_INSTALL_ORDER[ordinal]];
    let mut runtime = install_setup(ordinal);
    let old = runtime.native_properties.pop_first().unwrap().1;
    runtime.native_properties.insert(row.full.into(), old);
    let registry = runtime.native_properties.clone();
    let len = runtime.objects.len();
    assert!(
        runtime
            .install_dom_parent_method(ordinal, row, &row.name.into(), &keys)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.native_properties, registry);
    assert_eq!(runtime.objects.len(), len);
    assert_eq!(runtime.allocated, parent_install_bytes(row));
    let mut runtime = install_setup(0);
    runtime.objects = runtime.objects.into_boxed_slice().into_vec();
    let len = runtime.objects.len();
    let row = &PARENT_METHODS[PARENT_INSTALL_ORDER[0]];
    assert!(
        runtime
            .install_dom_parent_method(0, row, &row.name.into(), &keys)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.objects.len(), len);
    assert!(runtime.native_properties.is_empty());
    assert_eq!(runtime.allocated, 0);
}

#[test]
fn dom_parent_direct_bags_preserve_defaults_and_share_only_immutable_payloads() {
    let (mut runtime, _) = fresh();
    let first = runtime.native_properties[PARENT_METHODS[0].full];
    for row in &PARENT_METHODS {
        let bag = &runtime.objects[runtime.native_properties[row.full]];
        assert!(bag.boxed.is_none() && !bag.non_extensible && bag.intrinsic_error.is_none());
        assert!(bag.parameter_map.is_empty() && !bag.arguments && bag.regexp.is_none());
        assert!(bag.date_value.is_none() && bag.event.is_none() && !bag.event_target);
        assert!(bag.abort.is_none() && bag.namespace.is_none());
        for key in 0..2 {
            assert_eq!(
                bag.order[key].as_string().unwrap().units().as_ptr(),
                runtime.objects[first].order[key]
                    .as_string()
                    .unwrap()
                    .units()
                    .as_ptr()
            );
        }
    }
    for indices in [
        [0, 1, 2].as_slice(),
        [3, 4, 5].as_slice(),
        [6, 7, 8].as_slice(),
    ] {
        let pointers: Vec<_> = indices
            .iter()
            .map(|&i| {
                let bag = &runtime.objects[runtime.native_properties[PARENT_METHODS[i].full]];
                let Some(Value::String(name)) = bag.get("name") else {
                    panic!()
                };
                name.units().as_ptr()
            })
            .collect();
        assert!(pointers.windows(2).all(|pair| pair[0] == pair[1]));
    }
    runtime.objects[first].insert("name".into(), Value::String("changed".into()));
    metadata(&runtime, PARENT_METHODS[1].full, "querySelector", 1);
    metadata(&runtime, PARENT_METHODS[2].full, "querySelector", 1);
}

#[test]
fn dom_parent_getter_exact_one_short_and_no_new_bags() {
    for row in &PARENT_METHODS {
        let work = 4 + 2 + 8 + row.full.len();
        let heap =
            32 + std::mem::size_of::<Native>() + 2 * std::mem::size_of::<usize>() + row.full.len();
        for (available_work, available_heap, success) in [
            (work - 1, heap, false),
            (work, heap - 1, false),
            (work, heap, true),
        ] {
            let (mut runtime, mut doc) = fresh();
            let recv = receiver(row.interface, &mut doc);
            let registry = runtime.native_properties.clone();
            let objects = runtime.objects.len();
            let nodes = doc.nodes.len();
            runtime.steps = available_work;
            runtime.allocated = MAX_HEAP - available_heap;
            let result = runtime.dom_parent_method(&recv, row.operation, &doc);
            if success {
                let Value::Native(native) = result.unwrap() else {
                    panic!()
                };
                assert_eq!(native.name, row.full);
                assert_eq!(native.receiver, Value::Undefined);
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            } else {
                assert!(result.unwrap_err().is_resource_limit());
            }
            assert_eq!(runtime.native_properties, registry);
            assert_eq!(runtime.objects.len(), objects);
            assert_eq!(doc.nodes.len(), nodes);
        }
    }
}

#[test]
fn dom_parent_classifier_rejects_wrong_kinds_and_invalid_ids_before_allocation() {
    let (mut runtime, mut doc) = fresh();
    let text = Value::Node(doc.create_text_node("x"));
    for bad in [
        text,
        Value::Node(usize::MAX),
        Value::Null,
        Value::Undefined,
        Value::Window,
        Value::Object(0),
    ] {
        runtime.steps = 4;
        runtime.allocated = MAX_HEAP;
        assert_eq!(
            runtime
                .dom_parent_method(&bad, ParentOperation::QuerySelector, &doc)
                .unwrap(),
            Value::Undefined
        );
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    }
    // Document.append is recognized; its wrapper still needs admission after
    // classification instead of returning the old missing-method value.
    runtime.steps = 6;
    assert!(
        runtime
            .dom_parent_method(&Value::Document, ParentOperation::Append, &doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
}

fn resolution_work(name: &str, full: bool) -> usize {
    let mut work = 0;
    for row in &PARENT_METHODS {
        let candidate = if full { row.full } else { row.suffix };
        work += 1;
        if name.len() == candidate.len() {
            work += 1 + name.len() / 8;
        }
        if name == candidate {
            break;
        }
    }
    work
}

#[test]
fn dom_parent_resolver_pays_each_reached_length_and_text_comparison() {
    for full in [false, true] {
        for row in &PARENT_METHODS {
            let name = if full { row.full } else { row.suffix };
            let work = resolution_work(name, full);
            for available in [work - 1, work] {
                let (mut runtime, _) = fresh();
                runtime.steps = available;
                runtime.allocated = MAX_HEAP;
                let result = runtime.dom_parent_resolve(name, full);
                if available == work {
                    assert_eq!(result.unwrap().full, row.full);
                } else {
                    assert!(result.unwrap_err().is_resource_limit());
                }
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            }
        }
    }
}

#[test]
fn dom_parent_preclone_exact_one_short_and_unknown_namespace_admission() {
    for row in &PARENT_METHODS {
        assert!(is_parent_method_name(row.full));
        let work = 4 + resolution_work(row.full, true) + 4 + row.full.len();
        let heap = 32 + row.full.len();
        for (w, h, success) in [
            (work - 1, heap, false),
            (work, heap - 1, false),
            (work, heap, true),
        ] {
            let (mut runtime, _) = fresh();
            let len = runtime.objects.len();
            runtime.steps = w;
            runtime.allocated = MAX_HEAP - h;
            let result = runtime.dom_parent_call_preflight(row.full);
            if success {
                result.unwrap();
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            } else {
                assert!(result.unwrap_err().is_resource_limit());
            }
            assert_eq!(runtime.objects.len(), len);
            clean(&runtime);
        }
    }
    for name in [
        "",
        "DOM",
        "DOM.",
        "DOM.querySelector",
        "DOMTokenList.add",
        "Object.is",
    ] {
        assert!(!is_parent_method_name(name));
    }
    for name in [
        "DOM.Document.invalid",
        "DOM.Element.invalid",
        "DOM.DocumentFragment.querySelectox",
        "DOM.E",
    ] {
        assert!(is_parent_method_name(name));
        let (mut runtime, _) = fresh();
        runtime.steps = 4 + resolution_work(name, true);
        runtime.allocated = MAX_HEAP;
        let error = runtime.dom_parent_call_preflight(name).unwrap_err();
        assert!(matches!(error.kind, ErrorKind::Runtime("TypeError")));
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    }
}

fn saved_call_setup() -> (Runtime, Document, Value, Value) {
    let (mut runtime, mut doc) = fresh();
    runtime
        .execute(
            "var touched=0;var arg={toString:function(){touched++;return 'i';}};",
            &mut doc,
        )
        .unwrap();
    let element = Value::Node(doc.create_element("section"));
    let alias = runtime
        .dom_parent_method(&element, ParentOperation::QuerySelector, &doc)
        .unwrap();
    let arg = runtime.lookup(0, "arg").unwrap().1;
    runtime.define(0, "saved", alias.clone(), true).unwrap();
    // Test scaffolding, outside admission: isolate the name copy from a possible
    // first execution-frame allocation. Calls still use the production drive.
    runtime.frames.try_reserve_exact(32).unwrap();
    runtime.steps = MAX_STEPS;
    runtime.allocated = 0;
    (runtime, doc, alias, arg)
}

#[test]
fn dom_parent_saved_preentered_call_cuts_preclone_and_restores_outer_stack() {
    let row = &PARENT_METHODS[1];
    let preflight = 4 + resolution_work(row.full, true) + 4 + row.full.len();
    let dispatch = 1 + resolution_work(row.suffix, false) + 4;
    let work = 1 + preflight + dispatch;
    let heap = 32 + row.full.len();
    for (w, h, resource) in [
        (1 + preflight - 1, heap, true),
        (work, heap - 1, true),
        (work, heap, false),
    ] {
        let (mut runtime, mut doc, alias, arg) = saved_call_setup();
        runtime.stack_units = 8;
        runtime.steps = w;
        runtime.allocated = MAX_HEAP - h;
        // Wrong interface must not reach the supplied toString. Full allowance
        // reaches the actual branded TypeError; one-short copy cuts are terminal.
        let error = runtime
            .call(alias, vec![arg], Value::Document, &mut doc)
            .unwrap_err();
        if resource {
            assert!(error.is_resource_limit());
        } else {
            assert!(matches!(error.kind, ErrorKind::Runtime("TypeError")));
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        }
        assert_eq!(runtime.lookup(0, "touched").unwrap().1, Value::Number(0.0));
        assert_eq!(runtime.stack_units, 8);
        runtime.stack_units = 0;
        clean(&runtime);
    }
}

#[test]
fn dom_parent_saved_expression_call_cuts_preclone_and_unwinds_native_guard() {
    let row = &PARENT_METHODS[1];
    let unit = parser::Parser::program("saved(arg);").unwrap();
    let (mut measured, mut doc, _, _) = saved_call_setup();
    let error = run(&mut measured, &unit, &mut doc).unwrap_err();
    assert!(matches!(error.kind, ErrorKind::Runtime("TypeError")));
    let work = MAX_STEPS - measured.steps;
    let heap = measured.allocated;
    // The final reached operation is the branded receiver check. Removing its
    // complete suffix-dispatch work locates the paid pre-clone boundary.
    let before_dispatch = work - (1 + resolution_work(row.suffix, false) + 4);
    assert_eq!(
        heap,
        32 + std::mem::size_of::<Value>() + 32 + row.full.len()
    );
    for (w, h, resource) in [
        (before_dispatch - 1, heap, true),
        (work, heap - 1, true),
        (work, heap, false),
    ] {
        let (mut runtime, mut doc, _, _) = saved_call_setup();
        runtime.steps = w;
        runtime.allocated = MAX_HEAP - h;
        let error = run(&mut runtime, &unit, &mut doc).unwrap_err();
        if resource {
            assert!(error.is_resource_limit());
        } else {
            assert!(matches!(error.kind, ErrorKind::Runtime("TypeError")));
        }
        assert_eq!(runtime.lookup(0, "touched").unwrap().1, Value::Number(0.0));
        clean(&runtime);
    }
}

#[test]
fn dom_parent_wrong_brand_precedes_arity_and_unusable_argument_payload() {
    for row in &PARENT_METHODS {
        let (mut runtime, mut doc) = fresh();
        for interface in [
            ParentInterface::Document,
            ParentInterface::Element,
            ParentInterface::DocumentFragment,
        ] {
            if interface == row.interface {
                continue;
            }
            let recv = receiver(interface, &mut doc);
            for args in [vec![], vec![Value::Node(usize::MAX)]] {
                runtime.steps = 1 + resolution_work(row.suffix, false) + 4;
                runtime.allocated = MAX_HEAP;
                let error = runtime
                    .dom_native(row.suffix, recv.clone(), &args, &mut doc)
                    .unwrap_err();
                assert!(matches!(error.kind, ErrorKind::Runtime("TypeError")));
                assert_eq!(error.message, "incompatible DOM method receiver");
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            }
        }
    }
}

#[test]
fn dom_parent_getter_aliases_share_only_their_interface_bag() {
    let (mut runtime, mut doc) = fresh();
    let mut aliases = Vec::new();
    for row in &PARENT_METHODS {
        let first = receiver(row.interface, &mut doc);
        let second = receiver(row.interface, &mut doc);
        let a = runtime
            .dom_parent_method(&first, row.operation, &doc)
            .unwrap();
        let b = runtime
            .dom_parent_method(&second, row.operation, &doc)
            .unwrap();
        assert_eq!(a, b);
        assert_eq!(runtime.property_object(&a), runtime.property_object(&b));
        aliases.push(a);
    }
    for (i, a) in aliases.iter().enumerate() {
        let id = runtime.property_object(a).unwrap();
        runtime.objects[id].insert("marker".into(), Value::Number(i as f64));
        for (j, b) in aliases.iter().enumerate() {
            assert_eq!(a == b, i == j);
            if i != j {
                assert_ne!(runtime.property_object(a), runtime.property_object(b));
            }
        }
    }
    for (i, a) in aliases.iter().enumerate() {
        assert_eq!(
            runtime.objects[runtime.property_object(a).unwrap()].get("marker"),
            Some(&Value::Number(i as f64))
        );
    }
}

#[test]
fn dom_parent_frozen_ordinary_sources_pass_in_both_modes() {
    let mut modes = 0;
    for part in LOCAL.split("// ").skip(1).take(29) {
        let (header, body) = part.split_once('\n').unwrap();
        let name = header.split_once(' ').unwrap().0;
        if matches!(
            name,
            "real-interface-prototypes-standard-prerequisite"
                | "host-method-replacement-standard-prerequisite"
        ) || header.ends_with("[resource]")
        {
            continue;
        }
        for strict in [false, true] {
            let (mut runtime, mut doc) = fresh();
            helpers(&mut runtime, &mut doc);
            let result = if strict {
                runtime.execute_strict(body, &mut doc)
            } else {
                runtime.execute(body, &mut doc)
            };
            assert!(result.is_ok(), "{name}, strict={strict}: {result:?}");
            clean(&runtime);
            modes += 1;
        }
    }
    assert_eq!(modes, 50);
}

#[test]
fn dom_parent_frozen_feature_controls_reach_only_the_paired_name_assertion() {
    let mut modes = 0;
    for part in LOCAL.split("// ").skip(30) {
        let (header, body) = part.split_once('\n').unwrap();
        for strict in [false, true] {
            let (mut runtime, mut doc) = fresh();
            helpers(&mut runtime, &mut doc);
            let result = if strict {
                runtime.execute_strict(body, &mut doc)
            } else {
                runtime.execute(body, &mut doc)
            };
            if header.ends_with("[pass]") {
                assert!(result.is_ok(), "{header}: {result:?}");
            } else {
                assert_eq!(result.unwrap_err().name(), "Test262Error", "{header}");
            }
            clean(&runtime);
            modes += 1;
        }
    }
    assert_eq!(modes, 32);
}

#[test]
fn dom_parent_frozen_policy_prefixes_complete_before_terminal_loops() {
    for (name, loop_start) in [
        ("repeated-method-acquisition-terminal-policy", "for(;;)"),
        (
            "cumulative-function-property-growth-terminal-policy",
            "for(var n=0;;n++)",
        ),
    ] {
        let body = source(name);
        let prefix = &body[..body.find(loop_start).unwrap()];
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
            let (mut runtime, mut doc) = fresh();
            helpers(&mut runtime, &mut doc);
            let result = if strict {
                runtime.execute_strict(body, &mut doc)
            } else {
                runtime.execute(body, &mut doc)
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
fn dom_parent_append_terminal_conversion_keeps_author_effect_but_not_operation_mutation() {
    for fragment in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let create = if fragment {
            "document.createDocumentFragment()"
        } else {
            "document.createElement('section')"
        };
        let setup = format!(
            "var caught=false,finalized=false,destination={create},holder=document.createElement('aside'),child=document.createElement('b');holder.appendChild(child);var append=destination.append;var first={{toString:function(){{destination.textContent='author';return 'prepared';}}}},second={{toString:function(){{while(true){{}}}}}};"
        );
        runtime.execute(&setup, &mut doc).unwrap();
        let result=runtime.execute("try{append.call(destination,child,first,second);}catch(e){caught=true;}finally{finalized=true;}",&mut doc);
        assert!(result.unwrap_err().is_resource_limit());
        assert_eq!(runtime.lookup(0, "caught").unwrap().1, Value::Bool(false));
        assert_eq!(
            runtime.lookup(0, "finalized").unwrap().1,
            Value::Bool(false)
        );
        let Value::Node(destination) = runtime.lookup(0, "destination").unwrap().1 else {
            panic!()
        };
        let Value::Node(holder) = runtime.lookup(0, "holder").unwrap().1 else {
            panic!()
        };
        let Value::Node(child) = runtime.lookup(0, "child").unwrap().1 else {
            panic!()
        };
        assert_eq!(doc.text_content(destination), "author");
        assert_eq!(doc.nodes[child].parent, Some(holder));
        clean(&runtime);
    }
}

#[test]
fn dom_parent_bootstrap_reports_actual_remaining_budget_and_capacities() {
    let runtime = Runtime::uninitialized().finish_bootstrap().unwrap();
    println!(
        "DOM_PARENT_BOOTSTRAP remaining_steps={} allocated={} native_properties={} prototypes={} objects={} object_capacity={}",
        runtime.steps,
        runtime.allocated,
        runtime.native_properties.len(),
        runtime.prototypes.len(),
        runtime.objects.len(),
        runtime.objects.capacity()
    );
    assert!(runtime.steps < MAX_STEPS);
    assert!(runtime.allocated < MAX_HEAP);
    assert_eq!(runtime.objects.len(), 687);
    assert!(runtime.objects.capacity() >= BOOTSTRAP_OBJECT_CAPACITY);
    // Runtime::new additionally compares this capacity to its actual immediate
    // post-reserve capacity on every test build, detecting any later growth.
    clean(&runtime);
}
