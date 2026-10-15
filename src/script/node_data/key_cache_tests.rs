//! Parent-compatible identity and public behavior witnesses, frozen before production.
use super::*;

pub(super) struct Case {
    pub(super) member: &'static str,
    pub(super) setter: bool,
    pub(super) suffix: &'static str,
    pub(super) display: &'static str,
    pub(super) arity: usize,
    pub(super) work: usize,
    pub(super) heap_constant: usize,
}
pub(super) const CASES: [Case; 4] = [
    Case {
        member: "nodeValue",
        setter: false,
        suffix: "getNodeValue",
        display: "get nodeValue",
        arity: 0,
        work: 98,
        heap_constant: 793,
    },
    Case {
        member: "nodeValue",
        setter: true,
        suffix: "setNodeValue",
        display: "set nodeValue",
        arity: 1,
        work: 98,
        heap_constant: 793,
    },
    Case {
        member: "textContent",
        setter: false,
        suffix: "getTextContent",
        display: "get textContent",
        arity: 0,
        work: 104,
        heap_constant: 803,
    },
    Case {
        member: "textContent",
        setter: true,
        suffix: "setTextContent",
        display: "set textContent",
        arity: 1,
        work: 104,
        heap_constant: 803,
    },
];

fn constructor_bag(runtime: &Runtime) -> usize {
    let Value::Native(native) = &runtime.environments[0].bindings["Node"].value else {
        panic!("Node constructor")
    };
    native.properties.unwrap().get()
}
pub(super) fn named_key<'a>(runtime: &'a Runtime, owner: usize, name: &str) -> &'a PropertyKey {
    runtime.objects[owner]
        .values
        .keys()
        .find(|key| {
            key.as_string()
                .is_some_and(|text| text.units().iter().copied().eq(name.bytes().map(u16::from)))
        })
        .unwrap()
}
pub(super) fn cached_keys(runtime: &Runtime) -> [PropertyKey; 2] {
    let owner = constructor_bag(runtime);
    [
        named_key(runtime, owner, "length").clone(),
        named_key(runtime, owner, "name").clone(),
    ]
}

#[test]
fn node_accessor_keys_share_existing_constructor_payloads_with_four_distinct_bags() {
    let runtime = Runtime::try_new().unwrap();
    let keys = cached_keys(&runtime);
    let Some(Value::Object(prototype)) =
        runtime.objects[constructor_bag(&runtime)].get("prototype")
    else {
        panic!("Node prototype")
    };
    let mut natives = Vec::new();
    let mut bags = Vec::new();
    for case in &CASES {
        let property = runtime.objects[*prototype]
            .values
            .get(&PropertyKey::from(case.member))
            .unwrap();
        let PropertyValue::Accessor { get, set } = &property.value else {
            panic!("accessor")
        };
        let Value::Native(native) = (if case.setter { set } else { get }) else {
            panic!("native")
        };
        assert_eq!(native.name, format!("DOM.Node.{}", case.suffix));
        assert_eq!(native.receiver, Value::Undefined);
        assert!(property.enumerable && property.configurable);
        let owner = native.properties.unwrap().get();
        assert!(!bags.contains(&owner));
        assert!(natives.iter().all(|prior| !Rc::ptr_eq(prior, native)));
        bags.push(owner);
        natives.push(native.clone());
        assert_eq!(runtime.objects[owner].order, keys);
        assert_eq!(runtime.objects[owner].values.len(), 2);
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
            assert!(std::ptr::eq(stored, expected), "{} {name}", case.suffix);
            assert!(
                std::ptr::eq(ordered, expected),
                "order {} {name}",
                case.suffix
            );
        }
    }
    assert_eq!(bags.len(), 4);
    assert_eq!(runtime.objects.len(), 755);
    assert_eq!(runtime.native_properties.len(), 300);
    assert_eq!(runtime.prototypes.len(), 26);
    assert_eq!(runtime.environments[0].bindings.len(), 213);
}

#[test]
fn node_accessor_keys_preserve_descriptors_order_and_saved_behavior_in_both_modes() {
    const SOURCE: &str = r#"
(function () {
    function check(v) { if (!v) throw new Error('Node accessor key cache'); }
    var names = Object.getOwnPropertyNames(Node.prototype);
    check(names.slice(0, 11).join(',') === 'isConnected,nodeValue,textContent,getRootNode,hasChildNodes,normalize,cloneNode,isEqualNode,isSameNode,compareDocumentPosition,contains');
    check(names[names.length - 1] === 'constructor');
    var rows = [['nodeValue', 'get', 'get nodeValue', 0], ['nodeValue', 'set', 'set nodeValue', 1], ['textContent', 'get', 'get textContent', 0], ['textContent', 'set', 'set textContent', 1]];
    var fs = [];
    for (var i = 0; i < rows.length; i++) {
        var row = rows[i], descriptor = Object.getOwnPropertyDescriptor(Node.prototype, row[0]), f = descriptor[row[1]];
        check(descriptor.enumerable && descriptor.configurable && descriptor.writable === undefined);
        check(typeof f === 'function' && f === Object.getOwnPropertyDescriptor(Node.prototype, row[0])[row[1]]);
        check(Object.getPrototypeOf(f) === Function.prototype);
        check(Object.getOwnPropertyNames(f).join(',') === 'length,name');
        check(Object.getOwnPropertySymbols(f).length === 0 && Object.getOwnPropertyDescriptor(f, 'prototype') === undefined);
        var n = Object.getOwnPropertyDescriptor(f, 'name'), l = Object.getOwnPropertyDescriptor(f, 'length');
        check(n.value === row[2] && !n.writable && !n.enumerable && n.configurable);
        check(l.value === row[3] && !l.writable && !l.enumerable && l.configurable);
        var rejected = false; try { Reflect.construct(f, []); } catch (e) { rejected = e instanceof TypeError; }
        check(rejected);
        for (var j = 0; j < fs.length; j++) check(f !== fs[j]);
        fs.push(f);
    }
    Object.defineProperty(fs[0], 'name', {value: 'changed'});
    Object.defineProperty(fs[0], 'length', {value: 99});
    check(fs[0].name === 'changed' && fs[0].length === 99);
    for (var k = 1; k < rows.length; k++) check(fs[k].name === rows[k][2] && fs[k].length === rows[k][3]);
    check(Node.prototype.getRootNode.name === 'getRootNode' && Node.prototype.getRootNode.length === 0);
    var text = new Text('old'), box = document.createElement('div');
    Object.defineProperty(text, 'nodeValue', {value: 'shadow'});
    Object.defineProperty(text, 'textContent', {value: 'shadow'});
    check(fs[1].call(text, 'A\uD800') === undefined);
    check(fs[0].call(text) === 'A\uD800' && fs[2].call(text) === 'A\uD800');
    check(fs[3].call(text, null) === undefined && fs[0].call(text) === '');
    check(fs[3].call(box, 'Z') === undefined && fs[2].call(box) === 'Z' && fs[0].call(box) === null);
    var converted = 0;
    check(fs[1].call(box, {toString: function () { converted++; return 'ignored'; }}) === undefined);
    check(converted === 1 && fs[2].call(box) === 'Z');
    var bad = false; try { fs[1].call({}, {toString: function () { converted++; return 'bad'; }}); } catch (e) { bad = e instanceof TypeError; }
    check(bad && converted === 1);
    delete Node.prototype.nodeValue; delete Node.prototype.textContent;
    check(fs[1].call(text, 'saved') === undefined && fs[0].call(text) === 'saved');
    check(fs[3].call(box, 'after') === undefined && fs[2].call(box) === 'after');
    return true;
})()
"#;
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        let result = if strict {
            runtime.execute_strict(SOURCE, &mut document)
        } else {
            runtime.execute(SOURCE, &mut document)
        };
        assert_eq!(result.unwrap(), Value::Bool(true), "strict={strict}");
        assert!(runtime.frames.is_empty());
        assert_eq!(
            (runtime.calls, runtime.eval_depth, runtime.stack_units),
            (0, 0, 0)
        );
        assert!(runtime.console.is_empty());
    }
}
