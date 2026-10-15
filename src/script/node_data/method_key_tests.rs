//! Parent-compatible Node method metadata and public behavior controls.
use super::key_cache_tests::{cached_keys, named_key};
use super::*;

pub(super) struct Case {
    pub(super) member: &'static str,
    pub(super) suffix: &'static str,
    pub(super) display: &'static str,
    pub(super) arity: usize,
    pub(super) accessor: bool,
    public_row: &'static str,
    exercise: &'static str,
}

pub(super) const CASES: [Case; 9] = [
    Case {
        member: "isConnected",
        suffix: "getIsConnected",
        display: "get isConnected",
        arity: 0,
        accessor: true,
        public_row: "var selected = 'isConnected', display = 'get isConnected', arity = 0, accessor = true;",
        exercise: "check(f.call(text) === false && f.call(document) === true);",
    },
    Case {
        member: "getRootNode",
        suffix: "getRootNode",
        display: "getRootNode",
        arity: 0,
        accessor: false,
        public_row: "var selected = 'getRootNode', display = 'getRootNode', arity = 0, accessor = false;",
        exercise: "check(f.call(box) === box && f.call(text) === box);",
    },
    Case {
        member: "hasChildNodes",
        suffix: "hasChildNodes",
        display: "hasChildNodes",
        arity: 0,
        accessor: false,
        public_row: "var selected = 'hasChildNodes', display = 'hasChildNodes', arity = 0, accessor = false;",
        exercise: "check(f.call(box) === true && f.call(text) === false);",
    },
    Case {
        member: "normalize",
        suffix: "normalize",
        display: "normalize",
        arity: 0,
        accessor: false,
        public_row: "var selected = 'normalize', display = 'normalize', arity = 0, accessor = false;",
        exercise: "box.appendChild(new Text('B')); check(f.call(box) === undefined && box.childNodes.length === 1 && box.firstChild.data === 'AB');",
    },
    Case {
        member: "cloneNode",
        suffix: "cloneNode",
        display: "cloneNode",
        arity: 0,
        accessor: false,
        public_row: "var selected = 'cloneNode', display = 'cloneNode', arity = 0, accessor = false;",
        exercise: "var copy = f.call(box, true); check(copy !== box && copy.firstChild !== text && copy.textContent === 'A');",
    },
    Case {
        member: "isEqualNode",
        suffix: "isEqualNode",
        display: "isEqualNode",
        arity: 1,
        accessor: false,
        public_row: "var selected = 'isEqualNode', display = 'isEqualNode', arity = 1, accessor = false;",
        exercise: "check(f.call(text, new Text('A')) === true && f.call(text, new Text('B')) === false);",
    },
    Case {
        member: "isSameNode",
        suffix: "isSameNode",
        display: "isSameNode",
        arity: 1,
        accessor: false,
        public_row: "var selected = 'isSameNode', display = 'isSameNode', arity = 1, accessor = false;",
        exercise: "check(f.call(text, text) === true && f.call(text, box) === false);",
    },
    Case {
        member: "compareDocumentPosition",
        suffix: "compareDocumentPosition",
        display: "compareDocumentPosition",
        arity: 1,
        accessor: false,
        public_row: "var selected = 'compareDocumentPosition', display = 'compareDocumentPosition', arity = 1, accessor = false;",
        exercise: "check(f.call(box, text) === 20 && f.call(text, box) === 10);",
    },
    Case {
        member: "contains",
        suffix: "contains",
        display: "contains",
        arity: 1,
        accessor: false,
        public_row: "var selected = 'contains', display = 'contains', arity = 1, accessor = false;",
        exercise: "check(f.call(box, text) === true && f.call(text, box) === false);",
    },
];

fn node_prototype(runtime: &Runtime) -> usize {
    let Value::Native(constructor) = &runtime.environments[0].bindings["Node"].value else {
        panic!("Node constructor")
    };
    let Some(Value::Object(prototype)) =
        runtime.objects[constructor.properties.unwrap().get()].get("prototype")
    else {
        panic!("Node prototype")
    };
    *prototype
}

#[test]
fn node_method_keys_share_constructor_payloads_after_preserving_all_nine_bags() {
    let runtime = Runtime::try_new().unwrap();
    let keys = cached_keys(&runtime);
    let prototype = node_prototype(&runtime);
    let expected_order: Vec<PropertyKey> = [
        "isConnected",
        "nodeValue",
        "textContent",
        "getRootNode",
        "hasChildNodes",
        "normalize",
        "cloneNode",
        "isEqualNode",
        "isSameNode",
        "compareDocumentPosition",
        "contains",
    ]
    .into_iter()
    .map(PropertyKey::from)
    .collect();
    assert_eq!(
        runtime.objects[prototype].order.first(),
        Some(&runtime.well_known_key("toStringTag"))
    );
    assert_eq!(
        &runtime.objects[prototype].order[1..12],
        expected_order.as_slice()
    );
    assert_eq!(
        runtime.objects[prototype].order.last(),
        Some(&PropertyKey::from("constructor"))
    );
    let mut bags = Vec::new();
    let mut natives: Vec<Rc<Native>> = Vec::new();
    // Check the complete old roster before the new physical-sharing requirement.
    for case in &CASES {
        let property = &runtime.objects[prototype].values[&PropertyKey::from(case.member)];
        assert!(property.enumerable && property.configurable);
        let value = if case.accessor {
            let PropertyValue::Accessor { get, set } = &property.value else {
                panic!("accessor")
            };
            assert_eq!(*set, Value::Undefined);
            get
        } else {
            let PropertyValue::Data { value, writable } = &property.value else {
                panic!("method")
            };
            assert!(*writable);
            value
        };
        let Value::Native(native) = value else {
            panic!("native")
        };
        assert_eq!(native.name, format!("DOM.Node.{}", case.suffix));
        assert_eq!(native.receiver, Value::Undefined);
        assert!(natives.iter().all(|prior| !Rc::ptr_eq(prior, native)));
        natives.push(native.clone());
        let owner = native.properties.unwrap().get();
        assert!(!bags.contains(&owner));
        bags.push(owner);
        assert_eq!(
            runtime.objects[owner].prototype,
            Some(Value::Function(runtime.function_prototype))
        );
        assert_eq!(runtime.objects[owner].order, keys);
        assert_eq!(runtime.objects[owner].values.len(), 2);
        for (name, expected) in [
            ("length", Value::Number(case.arity as f64)),
            ("name", Value::String(case.display.into())),
        ] {
            let descriptor = &runtime.objects[owner].values[&PropertyKey::from(name)];
            let PropertyValue::Data { value, writable } = &descriptor.value else {
                panic!("metadata data")
            };
            assert_eq!(*value, expected);
            assert!(!*writable && !descriptor.enumerable && descriptor.configurable);
        }
    }
    assert_eq!(bags.len(), 9);
    assert_eq!(runtime.objects.len(), 755);
    assert_eq!(runtime.native_properties.len(), 300);
    assert_eq!(runtime.prototypes.len(), 26);
    assert_eq!(runtime.environments[0].bindings.len(), 213);
    for (case, owner) in CASES.iter().zip(bags) {
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
            assert!(std::ptr::eq(stored, expected), "{} {name}", case.member);
            assert!(
                std::ptr::eq(ordered, expected),
                "order {} {name}",
                case.member
            );
        }
    }
}

#[test]
fn node_method_keys_preserve_public_descriptors_mutation_and_saved_calls_in_both_modes() {
    const PREFIX: &str =
        "(function () { function check(v) { if (!v) throw new Error('Node method keys'); }";
    const BODY: &str = r#"
    var names = Object.getOwnPropertyNames(Node.prototype);
    check(names.slice(0, 11).join(',') === 'isConnected,nodeValue,textContent,getRootNode,hasChildNodes,normalize,cloneNode,isEqualNode,isSameNode,compareDocumentPosition,contains');
    check(names[names.length - 1] === 'constructor');
    var descriptor = Object.getOwnPropertyDescriptor(Node.prototype, selected);
    var f = accessor ? descriptor.get : descriptor.value;
    check(descriptor.enumerable && descriptor.configurable);
    check(accessor ? descriptor.set === undefined && descriptor.writable === undefined : descriptor.writable === true);
    check(f === (accessor ? Object.getOwnPropertyDescriptor(Node.prototype, selected).get : Node.prototype[selected]));
    check(Object.getPrototypeOf(f) === Function.prototype);
    check(Object.getOwnPropertyNames(f).join(',') === 'length,name');
    check(Object.getOwnPropertySymbols(f).length === 0 && Object.getOwnPropertyDescriptor(f, 'prototype') === undefined);
    var n = Object.getOwnPropertyDescriptor(f, 'name'), l = Object.getOwnPropertyDescriptor(f, 'length');
    check(n.value === display && !n.writable && !n.enumerable && n.configurable);
    check(l.value === arity && !l.writable && !l.enumerable && l.configurable);
    var rejected = false; try { Reflect.construct(f, []); } catch (e) { rejected = e instanceof TypeError; }
    check(rejected);
    Object.defineProperty(f, 'name', {value: 'changed'});
    Object.defineProperty(f, 'length', {value: 99});
    check(f.name === 'changed' && f.length === 99);
    var rows = [['isConnected','get isConnected',0],['getRootNode','getRootNode',0],['hasChildNodes','hasChildNodes',0],['normalize','normalize',0],['cloneNode','cloneNode',0],['isEqualNode','isEqualNode',1],['isSameNode','isSameNode',1],['compareDocumentPosition','compareDocumentPosition',1],['contains','contains',1]];
    for (var i = 0; i < rows.length; i++) {
        var row = rows[i];
        if (row[0] !== selected) {
            var d = Object.getOwnPropertyDescriptor(Node.prototype, row[0]);
            var other = row[0] === 'isConnected' ? d.get : d.value;
            check(other !== f && other.name === row[1] && other.length === row[2]);
        }
    }
    check(Node.name === 'Node' && Node.length === 0);
    var untouched = Object.getOwnPropertyDescriptor(Node.prototype, 'nodeValue');
    check(untouched.get.name === 'get nodeValue' && untouched.get.length === 0 && untouched.set.name === 'set nodeValue' && untouched.set.length === 1);
    check(delete f.name && delete f.length);
    check(Object.getOwnPropertyNames(f).length === 0);
    check(delete Node.prototype[selected]);
    check(Object.getOwnPropertyDescriptor(Node.prototype, selected) === undefined);
    var text = new Text('A'), box = document.createElement('div'); box.appendChild(text);
"#;
    for case in &CASES {
        let source = [
            PREFIX,
            case.public_row,
            BODY,
            case.exercise,
            "return true; })()",
        ]
        .concat();
        for strict in [false, true] {
            let mut runtime = Runtime::try_new().unwrap();
            let mut document = Document::parse("");
            let result = if strict {
                runtime.execute_strict(&source, &mut document)
            } else {
                runtime.execute(&source, &mut document)
            };
            assert_eq!(
                result.unwrap(),
                Value::Bool(true),
                "{} strict={strict}",
                case.member
            );
            assert!(runtime.frames.is_empty());
            assert_eq!(
                (runtime.calls, runtime.eval_depth, runtime.stack_units),
                (0, 0, 0)
            );
            assert!(runtime.console.is_empty());
        }
    }
}
