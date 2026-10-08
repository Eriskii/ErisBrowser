use super::*;

// Reconstruct the real earlier metadata from a completed realm. The held
// published roster determines what to retain; no candidate-generated roster
// defines the expectation. BTree shape need not equal the original shape.
fn stage() -> Runtime {
    let mut runtime = Runtime::new();
    let first = runtime.native_properties["ArrayBuffer.isView"];
    let constructor = runtime.native_properties["ArrayBuffer"];
    let prototype = runtime.prototypes["ArrayBuffer"];
    runtime
        .native_properties
        .retain(|name, _| BEFORE_REGISTRY.contains(&name.as_str()));
    assert_eq!(runtime.native_properties.len(), 178);
    assert!(runtime.native_properties.values().all(|id| *id < first));
    for (id, keys) in [
        (constructor, &["length", "name", "prototype"][..]),
        (prototype, &["constructor"][..]),
    ] {
        runtime.objects[id].values.retain(|key, _| {
            key.as_string().is_some_and(|key| {
                keys.iter()
                    .any(|text| key.units().iter().copied().eq(text.encode_utf16()))
            })
        });
        let mut order = Vec::with_capacity(4);
        order.extend(keys.iter().map(|text| PropertyKey::from(*text)));
        runtime.objects[id].order = order;
    }
    runtime.objects.truncate(first);
    runtime.array_buffers = super::super::State::default();
    assert_eq!(runtime.prototypes.len(), 25);
    runtime.steps = MAX_STEPS;
    runtime
}

struct OwnerSnapshot {
    id: usize,
    values: BTreeMap<PropertyKey, Property>,
    order: Vec<PropertyKey>,
    capacity: usize,
}

struct Snapshot {
    registry: BTreeMap<String, usize>,
    owners: Vec<OwnerSnapshot>,
    objects: usize,
    capacity: usize,
}

fn snapshot(runtime: &Runtime) -> Snapshot {
    Snapshot {
        registry: runtime.native_properties.clone(),
        owners: [
            runtime.native_properties["ArrayBuffer"],
            runtime.prototypes["ArrayBuffer"],
        ]
        .into_iter()
        .map(|id| OwnerSnapshot {
            id,
            values: runtime.objects[id].values.clone(),
            order: runtime.objects[id].order.clone(),
            capacity: runtime.objects[id].order.capacity(),
        })
        .collect(),
        objects: runtime.objects.len(),
        capacity: runtime.objects.capacity(),
    }
}

fn same_value(actual: &Value, expected: &Value) {
    assert_eq!(actual, expected);
    if let (Value::Native(actual), Value::Native(expected)) = (actual, expected) {
        assert!(Rc::ptr_eq(actual, expected));
    }
}

fn same_property(actual: &Property, expected: &Property) {
    assert_eq!(
        (actual.enumerable, actual.configurable),
        (expected.enumerable, expected.configurable)
    );
    match (&actual.value, &expected.value) {
        (
            PropertyValue::Data {
                value: a,
                writable: aw,
            },
            PropertyValue::Data {
                value: b,
                writable: bw,
            },
        ) => {
            assert_eq!(aw, bw);
            same_value(a, b);
        }
        (
            PropertyValue::Accessor { get: ag, set: as_ },
            PropertyValue::Accessor { get: bg, set: bs },
        ) => {
            same_value(ag, bg);
            same_value(as_, bs);
        }
        _ => panic!("descriptor kind changed"),
    }
}

fn unchanged(runtime: &Runtime, before: &Snapshot) {
    assert_eq!(runtime.native_properties, before.registry);
    assert_eq!(
        (runtime.objects.len(), runtime.objects.capacity()),
        (before.objects, before.capacity)
    );
    for saved in &before.owners {
        let owner = &runtime.objects[saved.id];
        assert_eq!(owner.order, saved.order);
        assert_eq!(owner.order.capacity(), saved.capacity);
        assert_eq!(owner.values.len(), saved.values.len());
        for (key, property) in &saved.values {
            same_property(&owner.values[key], property);
        }
    }
    assert!(runtime.array_buffers.intrinsic.is_none());
    assert!(runtime.array_buffers.prototype.is_none());
    assert!(runtime.array_buffers.records.is_empty());
}

fn assert_data(property: &Property, value: Value, writable: bool, configurable: bool) {
    assert!(!property.enumerable);
    assert_eq!(property.configurable, configurable);
    let PropertyValue::Data {
        value: actual,
        writable: actual_writable,
    } = &property.value
    else {
        panic!("not data")
    };
    assert_eq!(actual, &value);
    assert_eq!(*actual_writable, writable);
}

#[test]
fn array_buffer_bootstrap_retains_literal_metadata_order_and_old_identities() {
    let mut runtime = stage();
    let before = snapshot(&runtime);
    let constructor = runtime.native_properties["ArrayBuffer"];
    let prototype = runtime.prototypes["ArrayBuffer"];
    let species = runtime.well_known_key("species");
    let tag = runtime.well_known_key("toStringTag");
    runtime.install_array_buffer_intrinsics().unwrap();
    assert_eq!(runtime.objects.len(), before.objects + 10);
    assert_eq!(runtime.objects.capacity(), before.capacity);
    assert_eq!(runtime.native_properties.len(), 188);
    for (name, id) in &before.registry {
        assert_eq!(runtime.native_properties[name], *id);
    }
    for saved in &before.owners {
        for (key, property) in &saved.values {
            same_property(&runtime.objects[saved.id].values[key], property);
        }
    }
    let constructor_order: Vec<PropertyKey> = ["length", "name", "prototype", "isView"]
        .into_iter()
        .map(Into::into)
        .chain([species.clone()])
        .collect();
    let prototype_order: Vec<PropertyKey> = [
        "constructor",
        "byteLength",
        "maxByteLength",
        "resizable",
        "detached",
        "resize",
        "slice",
        "transfer",
        "transferToFixedLength",
    ]
    .into_iter()
    .map(Into::into)
    .chain([tag.clone()])
    .collect();
    assert_eq!(runtime.objects[constructor].order, constructor_order);
    assert_eq!(runtime.objects[prototype].order, prototype_order);
    assert_eq!(runtime.objects[constructor].order.capacity(), 8);
    assert_eq!(runtime.objects[prototype].order.capacity(), 16);
    // Literal expectations, independent of METHODS and its permutation.
    let expected = [
        ("ArrayBuffer.isView", "isView", "isView", 1, true, false),
        (
            "ArrayBuffer.getByteLength",
            "get byteLength",
            "byteLength",
            0,
            false,
            true,
        ),
        (
            "ArrayBuffer.getMaxByteLength",
            "get maxByteLength",
            "maxByteLength",
            0,
            false,
            true,
        ),
        (
            "ArrayBuffer.getResizable",
            "get resizable",
            "resizable",
            0,
            false,
            true,
        ),
        (
            "ArrayBuffer.getDetached",
            "get detached",
            "detached",
            0,
            false,
            true,
        ),
        ("ArrayBuffer.resize", "resize", "resize", 1, false, false),
        ("ArrayBuffer.slice", "slice", "slice", 2, false, false),
        (
            "ArrayBuffer.transfer",
            "transfer",
            "transfer",
            0,
            false,
            false,
        ),
        (
            "ArrayBuffer.transferToFixedLength",
            "transferToFixedLength",
            "transferToFixedLength",
            0,
            false,
            false,
        ),
        (
            "ArrayBuffer.species",
            "get [Symbol.species]",
            "",
            0,
            true,
            true,
        ),
    ];
    for (ordinal, (full, name, key, length, on_constructor, accessor)) in
        expected.into_iter().enumerate()
    {
        let id = runtime.native_properties[full];
        assert_eq!(id, before.objects + ordinal);
        let bag = &runtime.objects[id];
        assert_eq!(
            bag.prototype,
            Some(Value::Function(runtime.function_prototype))
        );
        assert_eq!(
            bag.order,
            vec![PropertyKey::from("name"), PropertyKey::from("length")]
        );
        assert_eq!(bag.order.capacity(), 4);
        assert_eq!(bag.values.len(), 2);
        assert_data(
            &bag.values[&PropertyKey::from("name")],
            Value::String(name.into()),
            false,
            true,
        );
        assert_data(
            &bag.values[&PropertyKey::from("length")],
            Value::Number(f64::from(length)),
            false,
            true,
        );
        let owner = if on_constructor {
            constructor
        } else {
            prototype
        };
        let key = if key.is_empty() {
            species.clone()
        } else {
            key.into()
        };
        let property = &runtime.objects[owner].values[&key];
        assert!(!property.enumerable);
        assert!(property.configurable);
        let function = match &property.value {
            PropertyValue::Accessor { get, set } => {
                assert!(accessor);
                assert_eq!(set, &Value::Undefined);
                get
            }
            PropertyValue::Data { value, writable } => {
                assert!(!accessor);
                assert!(*writable);
                value
            }
        };
        let Value::Native(native) = function else {
            panic!("not native")
        };
        assert_eq!(native.name, full);
        assert_eq!(native.receiver, Value::Undefined);
        assert!(native.properties.is_none());
    }
    assert_data(
        &runtime.objects[prototype].values[&tag],
        Value::String("ArrayBuffer".into()),
        false,
        true,
    );
    let Value::Native(saved) = runtime.array_buffers.intrinsic.as_ref().unwrap() else {
        panic!("not native")
    };
    assert_eq!(saved.name, "ArrayBuffer");
    assert_eq!(saved.receiver, Value::Window);
    assert!(saved.properties.is_none());
    assert_eq!(runtime.array_buffers.prototype, Some(prototype));
}

#[test]
fn array_buffer_bootstrap_exact_and_one_short_work_precedes_publication() {
    let mut exact = stage();
    let start = exact.allocated;
    exact.steps = 8_150;
    exact.install_array_buffer_intrinsics().unwrap();
    assert_eq!(exact.steps, 0);
    let bytes = exact.allocated - start;
    assert_eq!(bytes, install_bytes().unwrap());
    for budget in [0, 301, 302, 557, 558, 8_149] {
        let mut runtime = stage();
        let before = snapshot(&runtime);
        let allocated = runtime.allocated;
        runtime.steps = budget;
        assert!(
            runtime
                .install_array_buffer_intrinsics()
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, allocated);
        unchanged(&runtime, &before);
    }
}

#[test]
fn array_buffer_bootstrap_exact_and_one_short_heap_precedes_publication() {
    let mut measured = stage();
    let before = measured.allocated;
    measured.install_array_buffer_intrinsics().unwrap();
    let bytes = measured.allocated - before;
    let independent = 2_281
        + 10 * (72 + std::mem::size_of::<Option<AbortSlot>>() + std::mem::size_of::<Option<f64>>())
        + 10 * (16 * (std::mem::size_of::<PropertyKey>() + std::mem::size_of::<Property>())
            + 32 * std::mem::size_of::<usize>()
            + 64)
        + 72 * std::mem::size_of::<PropertyKey>()
        + 11 * (std::mem::size_of::<Native>() + 32)
        + 3 * (16 * (std::mem::size_of::<String>() + std::mem::size_of::<usize>())
            + 32 * std::mem::size_of::<usize>()
            + 64);
    assert_eq!(bytes, independent);
    let mut exact = stage();
    exact.allocated = MAX_HEAP - bytes;
    exact.install_array_buffer_intrinsics().unwrap();
    assert_eq!(exact.allocated, MAX_HEAP);
    let mut short = stage();
    let before = snapshot(&short);
    short.allocated = MAX_HEAP - bytes + 1;
    assert!(
        short
            .install_array_buffer_intrinsics()
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(short.steps, MAX_STEPS - 8_150);
    assert_eq!(short.allocated, MAX_HEAP + 1);
    unchanged(&short, &before);
}

#[test]
fn array_buffer_bootstrap_closed_stage_and_gap_refusals_are_unpublished() {
    for case in 0..5 {
        let mut runtime = stage();
        let ctor = runtime.native_properties["ArrayBuffer"];
        match case {
            0 => {
                runtime.native_properties.remove("Math");
            }
            1 => {
                let id = runtime.native_properties.remove("Math").unwrap();
                runtime
                    .native_properties
                    .insert("ArrayBuffer.collision".into(), id);
            }
            2 => {
                let bag = &mut runtime.objects[ctor];
                let property = bag.values.remove(&PropertyKey::from("length")).unwrap();
                bag.values.insert("lengti".into(), property);
            }
            3 => {
                runtime.objects[ctor].order.reserve_exact(2);
            }
            4 => {
                runtime.objects.shrink_to_fit();
            }
            _ => unreachable!(),
        }
        let before = snapshot(&runtime);
        let allocated = runtime.allocated;
        let error = runtime.install_array_buffer_intrinsics().unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(runtime.allocated, allocated);
        unchanged(&runtime, &before);
        assert_eq!(MAX_STEPS - runtime.steps, if case == 1 { 558 } else { 302 });
    }
}

#[test]
fn array_buffer_bootstrap_reentry_preserves_completed_metadata() {
    let mut runtime = stage();
    runtime.install_array_buffer_intrinsics().unwrap();
    let before = snapshot(&runtime);
    let saved = runtime.array_buffers.intrinsic.clone();
    let prototype = runtime.array_buffers.prototype;
    assert!(
        runtime
            .install_array_buffer_intrinsics()
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.native_properties, before.registry);
    assert_eq!(runtime.objects.len(), before.objects);
    assert_eq!(runtime.array_buffers.prototype, prototype);
    same_value(
        runtime.array_buffers.intrinsic.as_ref().unwrap(),
        saved.as_ref().unwrap(),
    );
    for saved in before.owners {
        assert_eq!(runtime.objects[saved.id].order, saved.order);
        assert_eq!(runtime.objects[saved.id].order.capacity(), saved.capacity);
        for (key, property) in saved.values {
            same_property(&runtime.objects[saved.id].values[&key], &property);
        }
    }
}

#[test]
fn array_buffer_bootstrap_sorted_gap_preserves_bindings_across_tree_shapes() {
    // Twelve nearby gap positions plus a parent-scale position and both ends.
    // Ascending, descending and interleaved initial orders exercise distinct
    // standard BTree shapes. This checks contents, not private library heights.
    let names = [
        "getByteLength",
        "getDetached",
        "getMaxByteLength",
        "getResizable",
        "isView",
        "resize",
        "slice",
        "species",
        "transfer",
        "transferToFixedLength",
    ];
    for left in (0..12).chain([70, 178]) {
        let pairs: Vec<_> = (0..178)
            .map(|i| (format!("{}-{i:03}", if i < left { "A" } else { "Z" }), i))
            .collect();
        for order in 0..3 {
            let indices: Vec<_> = match order {
                0 => (0..178).collect(),
                1 => (0..178).rev().collect(),
                _ => (0..89).flat_map(|i| [i, 177 - i]).collect(),
            };
            let mut map = BTreeMap::new();
            for i in indices {
                map.insert(pairs[i].0.clone(), pairs[i].1);
            }
            assert_eq!(map.len(), 178);
            assert!(
                map.range::<str, _>((Included("ArrayBuffer."), Excluded("ArrayBuffer/")))
                    .next()
                    .is_none()
            );
            for (i, name) in names.iter().enumerate() {
                let Entry::Vacant(entry) = map.entry(format!("ArrayBuffer.{name}")) else {
                    panic!("collision")
                };
                entry.insert(178 + i);
            }
            assert_eq!(map.len(), 188);
            for (key, value) in &pairs {
                assert_eq!(map[key], *value);
            }
            for (i, name) in names.iter().enumerate() {
                assert_eq!(map[&format!("ArrayBuffer.{name}")], 178 + i);
            }
        }
    }
}

#[test]
fn array_buffer_bootstrap_reports_raw_combined_initialization() {
    let runtime = Runtime::uninitialized().finish_bootstrap().unwrap();
    println!(
        "ARRAY_BUFFER_BOOTSTRAP remaining={} allocated={} objects={} capacity={} native={} prototypes={}",
        runtime.steps,
        runtime.allocated,
        runtime.objects.len(),
        runtime.objects.capacity(),
        runtime.native_properties.len(),
        runtime.prototypes.len()
    );
    assert!(runtime.steps > 0);
    assert!(runtime.steps < MAX_STEPS);
    assert_eq!(runtime.native_properties.len(), 320);
    assert_eq!(runtime.prototypes.len(), 25);
    assert!(runtime.array_buffers.intrinsic.is_some());
}

// Literal published roster, independently frozen before implementation.
const BEFORE_REGISTRY: [&str; 178] = [
    "AbortController",
    "AbortController.abort",
    "AbortController.get.signal",
    "AbortSignal",
    "AbortSignal.get.aborted",
    "AbortSignal.get.onabort",
    "AbortSignal.get.reason",
    "AbortSignal.set.onabort",
    "AbortSignal.static.abort",
    "AbortSignal.throwIfAborted",
    "Array",
    "Array.concat",
    "Array.every",
    "Array.filter",
    "Array.find",
    "Array.findIndex",
    "Array.findLast",
    "Array.findLastIndex",
    "Array.forEach",
    "Array.from",
    "Array.includes",
    "Array.indexOf",
    "Array.isArray",
    "Array.lastIndexOf",
    "Array.pop",
    "Array.reduce",
    "Array.reduceRight",
    "Array.reverse",
    "Array.shift",
    "Array.slice",
    "Array.some",
    "Array.sort",
    "Array.splice",
    "Array.unshift",
    "ArrayBuffer",
    "Boolean",
    "CustomEvent",
    "CustomEvent.initCustomEvent",
    "DOM.Document.append",
    "DOM.Document.querySelector",
    "DOM.Document.querySelectorAll",
    "DOM.DocumentFragment.append",
    "DOM.DocumentFragment.querySelector",
    "DOM.DocumentFragment.querySelectorAll",
    "DOM.Element.append",
    "DOM.Element.querySelector",
    "DOM.Element.querySelectorAll",
    "DOMException",
    "DataView",
    "Date",
    "Error",
    "EvalError",
    "Event",
    "Event.composedPath",
    "Event.get.bubbles",
    "Event.get.cancelBubble",
    "Event.get.cancelable",
    "Event.get.composed",
    "Event.get.currentTarget",
    "Event.get.defaultPrevented",
    "Event.get.detail",
    "Event.get.eventPhase",
    "Event.get.isTrusted",
    "Event.get.newState",
    "Event.get.oldState",
    "Event.get.returnValue",
    "Event.get.source",
    "Event.get.srcElement",
    "Event.get.target",
    "Event.get.timeStamp",
    "Event.get.type",
    "Event.initEvent",
    "Event.preventDefault",
    "Event.set.cancelBubble",
    "Event.set.returnValue",
    "Event.stopImmediatePropagation",
    "Event.stopPropagation",
    "EventTarget",
    "EventTarget.addEventListener",
    "EventTarget.dispatchEvent",
    "EventTarget.removeEventListener",
    "Function",
    "Function.hasInstance",
    "JSON",
    "JSON.parse",
    "JSON.stringify",
    "Math",
    "Math.abs",
    "Math.ceil",
    "Math.cos",
    "Math.exp",
    "Math.floor",
    "Math.log",
    "Math.max",
    "Math.min",
    "Math.pow",
    "Math.round",
    "Math.sign",
    "Math.sin",
    "Math.sqrt",
    "Math.tan",
    "Math.trunc",
    "Number",
    "Number.isFinite",
    "Number.isInteger",
    "Number.isNaN",
    "Number.isSafeInteger",
    "Number.toString",
    "Object",
    "Object.freeze",
    "Object.getOwnPropertySymbols",
    "Object.isFrozen",
    "Object.isSealed",
    "Object.seal",
    "RangeError",
    "ReferenceError",
    "Reflect.apply",
    "Reflect.construct",
    "Reflect.defineProperty",
    "Reflect.ownKeys",
    "RegExp",
    "RegExp.exec",
    "RegExp.get.dotAll",
    "RegExp.get.flags",
    "RegExp.get.global",
    "RegExp.get.hasIndices",
    "RegExp.get.ignoreCase",
    "RegExp.get.multiline",
    "RegExp.get.source",
    "RegExp.get.sticky",
    "RegExp.get.unicode",
    "RegExp.get.unicodeSets",
    "RegExp.species",
    "RegExp.symbolMatch",
    "RegExp.symbolSearch",
    "RegExp.symbolSplit",
    "RegExp.test",
    "RegExp.toString",
    "String",
    "String.charAt",
    "String.charCodeAt",
    "String.codePointAt",
    "String.concat",
    "String.endsWith",
    "String.fromCharCode",
    "String.fromCodePoint",
    "String.includes",
    "String.indexOf",
    "String.lastIndexOf",
    "String.match",
    "String.replace",
    "String.search",
    "String.slice",
    "String.split",
    "String.startsWith",
    "String.substring",
    "String.toLowerCase",
    "String.toUpperCase",
    "String.trim",
    "Symbol",
    "Symbol.description",
    "Symbol.for",
    "Symbol.keyFor",
    "Symbol.toPrimitive",
    "Symbol.toString",
    "Symbol.valueOf",
    "SyntaxError",
    "ToggleEvent",
    "TypeError",
    "URIError",
    "decodeURI",
    "decodeURIComponent",
    "encodeURI",
    "encodeURIComponent",
    "isFinite",
    "isNaN",
    "parseFloat",
    "parseInt",
];
