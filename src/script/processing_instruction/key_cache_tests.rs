//! Cache-only metadata and raw-bootstrap controls, frozen before production.
use super::*;

pub(super) struct Case {
    pub(super) owner: &'static str,
    pub(super) member: &'static str,
    pub(super) part: &'static str,
    pub(super) suffix: &'static str,
    pub(super) display: &'static str,
    pub(super) arity: usize,
    pub(super) work: usize,
    pub(super) heap_constant: usize,
}
pub(super) const CASES: [Case; 10] = [
    Case {
        owner: "Document",
        member: "createProcessingInstruction",
        part: "value",
        suffix: "create",
        display: "createProcessingInstruction",
        arity: 2,
        work: 118,
        heap_constant: 841,
    },
    Case {
        owner: "ProcessingInstruction",
        member: "target",
        part: "get",
        suffix: "target",
        display: "get target",
        arity: 0,
        work: 84,
        heap_constant: 773,
    },
    Case {
        owner: "CharacterData",
        member: "data",
        part: "get",
        suffix: "data",
        display: "get data",
        arity: 0,
        work: 78,
        heap_constant: 763,
    },
    Case {
        owner: "CharacterData",
        member: "data",
        part: "set",
        suffix: "setData",
        display: "set data",
        arity: 1,
        work: 81,
        heap_constant: 766,
    },
    Case {
        owner: "CharacterData",
        member: "length",
        part: "get",
        suffix: "length",
        display: "get length",
        arity: 0,
        work: 84,
        heap_constant: 773,
    },
    Case {
        owner: "CharacterData",
        member: "substringData",
        part: "value",
        suffix: "substringData",
        display: "substringData",
        arity: 2,
        work: 97,
        heap_constant: 792,
    },
    Case {
        owner: "CharacterData",
        member: "appendData",
        part: "value",
        suffix: "appendData",
        display: "appendData",
        arity: 1,
        work: 88,
        heap_constant: 777,
    },
    Case {
        owner: "CharacterData",
        member: "insertData",
        part: "value",
        suffix: "insertData",
        display: "insertData",
        arity: 2,
        work: 88,
        heap_constant: 777,
    },
    Case {
        owner: "CharacterData",
        member: "deleteData",
        part: "value",
        suffix: "deleteData",
        display: "deleteData",
        arity: 2,
        work: 88,
        heap_constant: 777,
    },
    Case {
        owner: "CharacterData",
        member: "replaceData",
        part: "value",
        suffix: "replaceData",
        display: "replaceData",
        arity: 3,
        work: 91,
        heap_constant: 782,
    },
];

fn constructor_bag(runtime: &Runtime, name: &str) -> usize {
    let Value::Native(native) = &runtime.environments[0].bindings[name].value else {
        panic!("interface constructor");
    };
    native.properties.unwrap().get()
}
fn prototype(runtime: &Runtime, name: &str) -> usize {
    let bag = constructor_bag(runtime, name);
    let Some(Value::Object(id)) = runtime.objects[bag].get("prototype") else {
        panic!("interface prototype");
    };
    *id
}
fn named_key<'a>(runtime: &'a Runtime, owner: usize, name: &str) -> &'a PropertyKey {
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
    let bag = constructor_bag(runtime, "ProcessingInstruction");
    [
        named_key(runtime, bag, "length").clone(),
        named_key(runtime, bag, "name").clone(),
    ]
}
fn function(runtime: &Runtime, case: &Case) -> Value {
    let owner = prototype(runtime, case.owner);
    let property = runtime.objects[owner]
        .values
        .get(&PropertyKey::from(case.member))
        .unwrap();
    match (&property.value, case.part) {
        (PropertyValue::Data { value, .. }, "value") => value.clone(),
        (PropertyValue::Accessor { get, .. }, "get") => get.clone(),
        (PropertyValue::Accessor { set, .. }, "set") => set.clone(),
        _ => panic!("PI function descriptor"),
    }
}

#[test]
fn cached_pi_keys_preserve_all_ten_public_descriptors_and_identities() {
    const SOURCE: &str = r#"
(function () {
    function check(v) { if (!v) throw new Error('PI key cache descriptor'); }
    var rows = [
        [Document.prototype, 'createProcessingInstruction', 'value', 'createProcessingInstruction', 2],
        [ProcessingInstruction.prototype, 'target', 'get', 'get target', 0],
        [CharacterData.prototype, 'data', 'get', 'get data', 0],
        [CharacterData.prototype, 'data', 'set', 'set data', 1],
        [CharacterData.prototype, 'length', 'get', 'get length', 0],
        [CharacterData.prototype, 'substringData', 'value', 'substringData', 2],
        [CharacterData.prototype, 'appendData', 'value', 'appendData', 1],
        [CharacterData.prototype, 'insertData', 'value', 'insertData', 2],
        [CharacterData.prototype, 'deleteData', 'value', 'deleteData', 2],
        [CharacterData.prototype, 'replaceData', 'value', 'replaceData', 3]
    ];
    var functions = [];
    for (var i = 0; i < rows.length; i++) {
        var r = rows[i], owner = Object.getOwnPropertyDescriptor(r[0], r[1]), f = owner[r[2]];
        check(owner.enumerable === true && owner.configurable === true);
        check(owner.writable === (r[2] === 'value' ? true : undefined));
        check(typeof f === 'function' && f === Object.getOwnPropertyDescriptor(r[0], r[1])[r[2]]);
        check(Object.getPrototypeOf(f) === Function.prototype);
        check(Object.getOwnPropertyNames(f).join(',') === 'length,name');
        check(Object.getOwnPropertySymbols(f).length === 0);
        check(Object.getOwnPropertyDescriptor(f, 'prototype') === undefined);
        var n = Object.getOwnPropertyDescriptor(f, 'name'), l = Object.getOwnPropertyDescriptor(f, 'length');
        check(n.value === r[3] && n.writable === false && n.enumerable === false && n.configurable === true);
        check(l.value === r[4] && l.writable === false && l.enumerable === false && l.configurable === true);
        for (var j = 0; j < functions.length; j++) check(f !== functions[j]);
        functions.push(f);
    }
    Object.defineProperty(functions[0], 'name', {value: 'changed'});
    Object.defineProperty(functions[0], 'length', {value: 99});
    check(functions[0].name === 'changed' && functions[0].length === 99);
    for (var k = 1; k < rows.length; k++) check(functions[k].name === rows[k][3] && functions[k].length === rows[k][4]);
    return true;
})()
"#;
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut doc = Document::parse("");
        let result = if strict {
            runtime.execute_strict(SOURCE, &mut doc)
        } else {
            runtime.execute(SOURCE, &mut doc)
        };
        assert_eq!(result.unwrap(), Value::Bool(true));
        assert!(runtime.frames.is_empty());
        assert_eq!(
            (runtime.calls, runtime.eval_depth, runtime.stack_units),
            (0, 0, 0)
        );
        assert!(runtime.console.is_empty());
    }
}

#[test]
fn cached_pi_keys_share_the_existing_immutable_payload_for_every_bag() {
    let runtime = Runtime::try_new().unwrap();
    let keys = cached_keys(&runtime);
    let mut ids = Vec::new();
    for case in &CASES {
        let Value::Native(native) = function(&runtime, case) else {
            panic!("native");
        };
        assert_eq!(native.name, format!("DOM.PI.{}", case.suffix));
        assert_eq!(native.receiver, Value::Undefined);
        let owner = native.properties.unwrap().get();
        assert!(!ids.contains(&owner));
        ids.push(owner);
        assert_eq!(runtime.objects[owner].order, keys);
        assert_eq!(runtime.objects[owner].values.len(), 2);
        for (index, key_name) in ["length", "name"].into_iter().enumerate() {
            let actual = named_key(&runtime, owner, key_name).as_string().unwrap();
            let cached = keys[index].as_string().unwrap();
            assert!(
                std::ptr::eq(actual.units().as_ptr(), cached.units().as_ptr()),
                "{} {key_name}",
                case.suffix
            );
        }
    }
    assert_eq!(ids.len(), 10);
    // The ordinary CharacterData.length accessor remains a separate property;
    // only each function's metadata keys participate in the saving.
    let owner = prototype(&runtime, "CharacterData");
    let property = runtime.objects[owner]
        .values
        .get(&PropertyKey::from("length"))
        .unwrap();
    assert!(matches!(
        &property.value,
        PropertyValue::Accessor {
            set: Value::Undefined,
            ..
        }
    ));
    assert!(property.enumerable && property.configurable);
}

#[test]
fn cached_pi_keys_raw_bootstrap_with_promise_uses_the_frozen_99846_work_boundary() {
    // Cached PI keys still save332; Promise with finally standalone installation adds1154.
    // Shared adjacent job keys independently save34 work without changing either limit.
    // Four Node accessor key pairs independently save128 work and672 bytes.
    // Promise.all independently adds107 metadata work and one arena guard.
    for (allowance, success, remaining) in
        [(99_845, false, 0), (99_846, true, 0), (100_000, true, 154)]
    {
        let mut runtime = Runtime::uninitialized();
        runtime.steps = allowance;
        let result = runtime.finish_bootstrap();
        if !success {
            let error = match result {
                Ok(_) => panic!("one-short bootstrap succeeded"),
                Err(error) => error,
            };
            assert!(error.is_resource_limit());
            continue;
        }
        let runtime = result.unwrap();
        assert_eq!(runtime.steps, remaining);
        assert_eq!(runtime.objects.len(), 751);
        assert_eq!(runtime.native_properties.len(), 299);
        assert_eq!(runtime.prototypes.len(), 25);
        assert_eq!(runtime.environments[0].bindings.len(), 212);
        assert!(runtime.environments[0].bindings.contains_key("Promise"));
        assert!(runtime.frames.is_empty());
        assert!(runtime.console.is_empty());
    }
    let runtime = Runtime::try_new().unwrap();
    assert_eq!(runtime.steps, 100_000);
}
