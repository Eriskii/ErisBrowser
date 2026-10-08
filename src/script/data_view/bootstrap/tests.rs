use super::*;

// The literal registry roster comes from the published pre-DataView stage.
// Reconstruct that stage from a completed realm without using candidate output
// to define either the roster or the expected owner descriptors.
fn stage() -> Runtime {
    let mut runtime = Runtime::new();
    let owner = runtime.prototypes["DataView"];
    let PropertyValue::Accessor {
        get: Value::Native(native),
        ..
    } = &runtime.objects[owner].values[&PropertyKey::from("buffer")].value
    else {
        panic!()
    };
    let first = native.properties.unwrap().get();
    runtime
        .native_properties
        .retain(|name, _| BEFORE_REGISTRY.contains(&name.as_str()));
    assert_eq!(runtime.native_properties.len(), 188);
    assert!(runtime.native_properties.values().all(|id| *id < first));
    runtime.objects[owner]
        .values
        .retain(|key, _| key == &PropertyKey::from("constructor"));
    let mut order = Vec::with_capacity(4);
    order.push(PropertyKey::from("constructor"));
    runtime.objects[owner].order = order;
    runtime.objects.truncate(first);
    runtime.data_views = super::super::State::default();
    runtime.steps = MAX_STEPS;
    runtime
}

struct Snapshot {
    registry: BTreeMap<String, usize>,
    owner: usize,
    values: BTreeMap<PropertyKey, Property>,
    order: Vec<PropertyKey>,
    order_capacity: usize,
    objects: usize,
    capacity: usize,
    prototype: Option<usize>,
}

fn snapshot(runtime: &Runtime) -> Snapshot {
    // The object's stable ID remains observable even in a malformed table.
    let owner = runtime
        .data_views
        .prototype
        .unwrap_or_else(|| runtime.prototypes.get("DataView").copied().unwrap_or(0));
    let object = &runtime.objects[owner];
    Snapshot {
        registry: runtime.native_properties.clone(),
        owner,
        values: object.values.clone(),
        order: object.order.clone(),
        order_capacity: object.order.capacity(),
        objects: runtime.objects.len(),
        capacity: runtime.objects.capacity(),
        prototype: runtime.data_views.prototype,
    }
}

fn same_value(actual: &Value, expected: &Value) {
    assert_eq!(actual, expected);
    if let (Value::Native(a), Value::Native(b)) = (actual, expected) {
        assert!(Rc::ptr_eq(a, b));
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
    assert_eq!(runtime.data_views.prototype, before.prototype);
    assert!(runtime.data_views.records.is_empty());
    let owner = &runtime.objects[before.owner];
    assert_eq!(owner.order, before.order);
    assert_eq!(owner.order.capacity(), before.order_capacity);
    assert_eq!(owner.values.len(), before.values.len());
    for (key, property) in &before.values {
        same_property(&owner.values[key], property);
    }
}

fn assert_data(property: &Property, value: Value, writable: bool) {
    assert!(!property.enumerable && property.configurable);
    let PropertyValue::Data {
        value: actual,
        writable: actual_writable,
    } = &property.value
    else {
        panic!()
    };
    assert_eq!(actual, &value);
    assert_eq!(*actual_writable, writable);
}

#[test]
fn data_view_bootstrap_retains_all_literal_metadata_ids_order_and_flags() {
    let mut runtime = stage();
    let before = snapshot(&runtime);
    let constructor_bag = runtime.native_properties["DataView"];
    let constructor_order = runtime.objects[constructor_bag].order.clone();
    let constructor_values = runtime.objects[constructor_bag].values.clone();
    runtime.install_data_view_intrinsics().unwrap();
    assert_eq!(runtime.native_properties, before.registry);
    assert_eq!(
        (runtime.objects.len(), runtime.objects.capacity()),
        (before.objects + 21, before.capacity)
    );
    assert_eq!(runtime.data_views.prototype, Some(before.owner));
    let owner = &runtime.objects[before.owner];
    assert_eq!(owner.values.len(), 23);
    assert_eq!(owner.order.capacity(), 23);
    same_property(
        &owner.values[&PropertyKey::from("constructor")],
        &before.values[&PropertyKey::from("constructor")],
    );
    assert_eq!(runtime.objects[constructor_bag].order, constructor_order);
    for (key, value) in &constructor_values {
        same_property(&runtime.objects[constructor_bag].values[key], value);
    }
    // Literal rows are independent of METHODS, TEXT, Codec::NAMES and permutations.
    let expected = [
        ("buffer", "DataView.getBuffer", "get buffer", 0, true),
        (
            "byteLength",
            "DataView.getByteLength",
            "get byteLength",
            0,
            true,
        ),
        (
            "byteOffset",
            "DataView.getByteOffset",
            "get byteOffset",
            0,
            true,
        ),
        ("getInt8", "DataView.getInt8", "getInt8", 1, false),
        ("setInt8", "DataView.setInt8", "setInt8", 2, false),
        ("getUint8", "DataView.getUint8", "getUint8", 1, false),
        ("setUint8", "DataView.setUint8", "setUint8", 2, false),
        ("getInt16", "DataView.getInt16", "getInt16", 1, false),
        ("setInt16", "DataView.setInt16", "setInt16", 2, false),
        ("getUint16", "DataView.getUint16", "getUint16", 1, false),
        ("setUint16", "DataView.setUint16", "setUint16", 2, false),
        ("getInt32", "DataView.getInt32", "getInt32", 1, false),
        ("setInt32", "DataView.setInt32", "setInt32", 2, false),
        ("getUint32", "DataView.getUint32", "getUint32", 1, false),
        ("setUint32", "DataView.setUint32", "setUint32", 2, false),
        ("getFloat16", "DataView.getFloat16", "getFloat16", 1, false),
        ("setFloat16", "DataView.setFloat16", "setFloat16", 2, false),
        ("getFloat32", "DataView.getFloat32", "getFloat32", 1, false),
        ("setFloat32", "DataView.setFloat32", "setFloat32", 2, false),
        ("getFloat64", "DataView.getFloat64", "getFloat64", 1, false),
        ("setFloat64", "DataView.setFloat64", "setFloat64", 2, false),
    ];
    let order: Vec<PropertyKey> = std::iter::once(PropertyKey::from("constructor"))
        .chain(expected.iter().map(|row| PropertyKey::from(row.0)))
        .chain([runtime.well_known_key("toStringTag")])
        .collect();
    assert_eq!(owner.order, order);
    for (ordinal, (key, full, display, length, getter)) in expected.into_iter().enumerate() {
        let property = &owner.values[&PropertyKey::from(key)];
        assert!(!property.enumerable && property.configurable);
        let function = match &property.value {
            PropertyValue::Accessor { get, set } => {
                assert!(getter);
                assert_eq!(set, &Value::Undefined);
                get
            }
            PropertyValue::Data { value, writable } => {
                assert!(!getter && *writable);
                value
            }
        };
        let Value::Native(native) = function else {
            panic!()
        };
        assert_eq!(native.name, full);
        assert_eq!(native.receiver, Value::Undefined);
        assert_eq!(native.properties.unwrap().get(), before.objects + ordinal);
        assert!(!runtime.native_properties.contains_key(full));
        let bag = &runtime.objects[before.objects + ordinal];
        assert_eq!(
            bag.prototype,
            Some(Value::Function(runtime.function_prototype))
        );
        assert_eq!(
            bag.order,
            [PropertyKey::from("name"), PropertyKey::from("length")]
        );
        assert_eq!(bag.order.capacity(), 2);
        assert_eq!(bag.values.len(), 2);
        assert_data(
            &bag.values[&PropertyKey::from("name")],
            Value::String(display.into()),
            false,
        );
        assert_data(
            &bag.values[&PropertyKey::from("length")],
            Value::Number(length as f64),
            false,
        );
    }
    assert_data(
        &owner.values[&runtime.well_known_key("toStringTag")],
        Value::String("DataView".into()),
        false,
    );
}

#[test]
fn data_view_bootstrap_exact_work_and_all_admission_cuts() {
    let mut measure = stage();
    let allocated = measure.allocated;
    measure.install_data_view_intrinsics().unwrap();
    assert_eq!(MAX_STEPS - measure.steps, 4_553);
    let bytes = measure.allocated - allocated;
    println!(
        "DATAVIEW_DIRECT work={} bytes={bytes}",
        MAX_STEPS - measure.steps
    );
    for work in [0, 1, 235, 236, 4_552, 4_553] {
        let mut runtime = stage();
        let before = snapshot(&runtime);
        let allocated = runtime.allocated;
        runtime.steps = work;
        let result = runtime.install_data_view_intrinsics();
        if work == 4_553 {
            result.unwrap();
            assert_eq!(runtime.steps, 0);
            assert_eq!(runtime.allocated - allocated, bytes);
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            assert_eq!(runtime.allocated, allocated);
            unchanged(&runtime, &before);
        }
    }
}

#[test]
fn data_view_bootstrap_typed_storage_exact_and_one_short() {
    let bytes = install_bytes().unwrap();
    let leaf = 16 * (std::mem::size_of::<PropertyKey>() + std::mem::size_of::<Property>())
        + 32 * std::mem::size_of::<usize>()
        + 64;
    let expected = 2_688
        + 21 * (72 + std::mem::size_of::<Option<AbortSlot>>() + std::mem::size_of::<Option<f64>>())
        + 21 * leaf
        + 42 * std::mem::size_of::<PropertyKey>()
        + 21 * (std::mem::size_of::<Native>() + 32)
        + 382
        + 3 * leaf
        + 23 * std::mem::size_of::<PropertyKey>()
        + (23 + 48) * std::mem::size_of::<(PropertyKey, Property)>();
    assert_eq!(bytes, expected);
    for enough in [false, true] {
        let mut runtime = stage();
        let before = snapshot(&runtime);
        runtime.allocated = MAX_HEAP - bytes + usize::from(!enough);
        let result = runtime.install_data_view_intrinsics();
        assert_eq!(MAX_STEPS - runtime.steps, 4_553);
        assert_eq!(runtime.allocated, MAX_HEAP + usize::from(!enough));
        if enough {
            result.unwrap();
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            unchanged(&runtime, &before);
        }
    }
}

#[test]
fn data_view_bootstrap_closed_stage_refusals_preserve_owner_and_registry() {
    for case in 0..9 {
        let mut runtime = stage();
        let owner = runtime.prototypes["DataView"];
        match case {
            0 => {
                runtime.native_properties.remove("Math");
            }
            1 => {
                runtime.prototypes.remove("Object");
            }
            2 => {
                runtime.function_prototype = usize::MAX;
            }
            3 => {
                runtime.objects.shrink_to_fit();
            }
            4 => {
                runtime.objects[owner].order.reserve_exact(4);
            }
            5 => {
                runtime.objects[owner].order[0] = PropertyKey::from("constructoz");
            }
            6 => {
                let property = runtime.objects[owner]
                    .values
                    .remove(&PropertyKey::from("constructor"))
                    .unwrap();
                runtime.objects[owner]
                    .values
                    .insert(PropertyKey::from("constructoz"), property);
            }
            7 => {
                runtime.objects[owner]
                    .values
                    .get_mut(&PropertyKey::from("constructor"))
                    .unwrap()
                    .configurable = false;
            }
            8 => {
                runtime.objects[owner]
                    .values
                    .get_mut(&PropertyKey::from("constructor"))
                    .unwrap()
                    .value = PropertyValue::Data {
                    value: Value::Undefined,
                    writable: true,
                };
            }
            _ => unreachable!(),
        }
        let before = snapshot(&runtime);
        let allocated = runtime.allocated;
        assert!(
            runtime
                .install_data_view_intrinsics()
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(MAX_STEPS - runtime.steps, 236);
        assert_eq!(runtime.allocated, allocated);
        unchanged(&runtime, &before);
    }
}

#[test]
fn data_view_bootstrap_reentry_preserves_completed_direct_metadata() {
    let mut runtime = stage();
    runtime.install_data_view_intrinsics().unwrap();
    let before = snapshot(&runtime);
    let allocated = runtime.allocated;
    let steps = runtime.steps;
    assert!(
        runtime
            .install_data_view_intrinsics()
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(steps - runtime.steps, 236);
    assert_eq!(runtime.allocated, allocated);
    unchanged(&runtime, &before);
}

#[test]
fn data_view_bootstrap_function_bags_do_not_consult_object_anchor() {
    for anchor in ["normal", "absent", "misdirected"] {
        let mut runtime = stage();
        match anchor {
            "normal" => {}
            "absent" => {
                let old = runtime.prototypes.remove("Object").unwrap();
                runtime.prototypes.insert("UnusedAnchor", old);
            }
            "misdirected" => {
                runtime.prototypes.insert("Object", usize::MAX);
            }
            _ => unreachable!(),
        }
        let first = runtime.objects.len();
        let allocated = runtime.allocated;
        runtime.install_data_view_intrinsics().unwrap();
        assert_eq!(MAX_STEPS - runtime.steps, 4_553);
        assert_eq!(runtime.allocated - allocated, install_bytes().unwrap());
        for bag in &runtime.objects[first..] {
            assert_eq!(
                bag.prototype,
                Some(Value::Function(runtime.function_prototype))
            );
        }
        runtime.objects[first].insert_hidden("name".into(), Value::String("changed".into()));
        assert_eq!(
            runtime.objects[first + 1].get("name"),
            Some(&Value::String("get byteLength".into()))
        );
    }
}

#[test]
fn data_view_direct_saved_methods_use_ordinary_metadata_and_bound_calls() {
    // Each run gets a fresh realm; immutable function identity and mutable
    // metadata are checked separately from the owner's replacement property.
    for strict in [false, true] {
        let mut runtime = Runtime::new();
        let mut doc = Document::parse("<p>kept</p>");
        let source = r#"
function require(ok) { if (!ok) throw new Error('direct DataView metadata'); }
var view = new DataView(new ArrayBuffer(4));
var get = DataView.prototype.getUint8, set = DataView.prototype.setUint8;
var other = DataView.prototype.getInt8;
var call = Function.prototype.call, apply = Function.prototype.apply;
var bound = set.bind(view, 0);
require(get !== other && get.name === 'getUint8' && get.length === 1);
require(Object.getPrototypeOf(get) === Function.prototype);
require(Object.getOwnPropertyNames(get).join(',') === 'name,length');
Object.defineProperty(get, 'name', {value:'saved', configurable:true});
require(get.name === 'saved' && other.name === 'getInt8');
require(delete get.length && !Object.hasOwn(get, 'length'));
get.marker = 7;
Object.setPrototypeOf(get, {inherited:9});
require(get.inherited === 9 && get.marker === 7);
Object.defineProperty(DataView.prototype, 'getUint8', {value:17});
require(DataView.prototype.getUint8 === 17 && get.name === 'saved');
bound(43);
require(call.call(get, view, 0) === 43);
require(apply.call(get, view, [0]) === 43);
Object.preventExtensions(get);
require(!Object.isExtensible(get) && !Reflect.defineProperty(get, 'newKey', {value:1}));
Object.freeze(get);
require(Object.isFrozen(get) && get.marker === 7);
Object.seal(set);
require(Object.isSealed(set));
var bufferGetter = Object.getOwnPropertyDescriptor(DataView.prototype, 'buffer').get;
require(bufferGetter.call(view) === view.buffer);
"#;
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        assert!(result.is_ok(), "strict={strict}: {result:?}");
        assert_eq!(runtime.calls, 0);
        assert_eq!(runtime.stack_units, 0);
        assert!(runtime.frames.is_empty());
    }
}

const BEFORE_REGISTRY: [&str; 188] = [
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
    "ArrayBuffer.getByteLength",
    "ArrayBuffer.getDetached",
    "ArrayBuffer.getMaxByteLength",
    "ArrayBuffer.getResizable",
    "ArrayBuffer.isView",
    "ArrayBuffer.resize",
    "ArrayBuffer.slice",
    "ArrayBuffer.species",
    "ArrayBuffer.transfer",
    "ArrayBuffer.transferToFixedLength",
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
