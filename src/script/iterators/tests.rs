use super::*;

fn prepared() -> (Runtime, Document) {
    (Runtime::new(), Document::parse("<p>unchanged</p>"))
}
fn clean(runtime: &Runtime) {
    assert_eq!(
        (
            runtime.calls,
            runtime.eval_depth,
            runtime.stack_units,
            runtime.json_depth
        ),
        (0, 0, 0, 0)
    );
    assert!(runtime.frames.is_empty());
    assert!(runtime.active_array_joins.is_empty());
}
fn result(runtime: &Runtime, value: Value) -> (Value, bool) {
    let Value::Object(id) = value else {
        panic!("ordinary iterator result expected")
    };
    let value = runtime.objects[id].get("value").unwrap().clone();
    let done = runtime.objects[id].get("done").unwrap();
    assert_eq!(runtime.objects[id].order.len(), 2);
    (value, *done == Value::Bool(true))
}
fn next(runtime: &mut Runtime, doc: &mut Document, iterator: &Value, kind: &str) -> (Value, bool) {
    let value = runtime
        .iterator_native(kind, iterator.clone(), &[], doc)
        .unwrap();
    result(runtime, value)
}
fn check(source: &str) {
    for strict in [false, true] {
        let (mut runtime, mut doc) = prepared();
        for helper in [
            include_str!("../../../tests/upstream/test262/harness/sta.js"),
            include_str!("../../../tests/upstream/test262/harness/assert.js"),
            include_str!("../../../tests/upstream/test262/harness/propertyHelper.js"),
        ] {
            runtime.execute(helper, &mut doc).unwrap();
        }
        let value = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        assert!(value.is_ok(), "strict={strict}: {value:?}");
        clean(&runtime);
    }
}

#[test]
fn native_iterator_metadata_aliases_brands_and_self_are_exact() {
    check(
        r#"
        var values=Array.prototype.values, tag=Symbol.toStringTag, key=Symbol.iterator;
        assert.sameValue(values,Array.prototype[key]);
        var a=values.call([2]), s=String.prototype[key].call('x');
        var ap=Object.getPrototypeOf(a),sp=Object.getPrototypeOf(s),common=Object.getPrototypeOf(ap);
        assert.sameValue(common,Object.getPrototypeOf(sp));
        assert.sameValue(Object.getPrototypeOf(common),Object.prototype);
        var self=common[key], token={};
        assert.sameValue(self.call(token),token);assert.sameValue(self.call(null),null);assert.sameValue(self.call(undefined),undefined);assert.sameValue(self.call(7),7);
        verifyProperty(common,key,{value:self,writable:true,enumerable:false,configurable:true},{restore:true});
        verifyProperty(ap,tag,{value:'Array Iterator',writable:false,enumerable:false,configurable:true},{restore:true});
        verifyProperty(sp,tag,{value:'String Iterator',writable:false,enumerable:false,configurable:true},{restore:true});
        assert.sameValue(Object.prototype.toString.call(a),'[object Array Iterator]');
        assert.sameValue(Object.prototype.toString.call(s),'[object String Iterator]');
        var methods=[values,Array.prototype.keys,Array.prototype.entries,a.next,s.next,String.prototype[key],self];
        var names=['values','keys','entries','next','next','[Symbol.iterator]','[Symbol.iterator]'];
        for(var i=0;i<methods.length;i++) {
            verifyProperty(methods[i],'name',{value:names[i],writable:false,enumerable:false,configurable:true},{restore:true});
            verifyProperty(methods[i],'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});
            assert.sameValue(Object.getOwnPropertyDescriptor(methods[i],'prototype'),undefined);
            (function(m){assert.throws(TypeError,function(){new m();});})(methods[i]);
        }
        assert.throws(TypeError,function(){a.next.call(s);});assert.throws(TypeError,function(){s.next.call(a);});
        assert.throws(TypeError,function(){a.next.call(Object.create(ap));});
        assert.throws(TypeError,function(){s.next.call(Object.create(sp));});
        var r=a.next(),r2=a.next(),r3=a.next();
        assert.sameValue(r.value,2);assert.sameValue(r.done,false);assert.sameValue(r2.done,true);assert.sameValue(r3.done,true);assert.notSameValue(r2,r3);
        assert.compareArray(Object.keys(r),['value','done']);
        verifyProperty(r,'value',{value:2,writable:true,enumerable:true,configurable:true},{restore:true});
        verifyProperty(r,'done',{value:false,writable:true,enumerable:true,configurable:true},{restore:true});
    "#,
    );
}
#[test]
fn native_iterator_arrays_live_lengths_holes_keys_entries_and_saved_arguments_alias() {
    check(r#"
        var a=[,2],it=a.values();assert.sameValue(it.next().value,undefined);a.push(3);assert.sameValue(it.next().value,2);assert.sameValue(it.next().value,3);assert.sameValue(it.next().done,true);a.push(4);assert.sameValue(it.next().done,true);
        var proto={0:8},o=Object.create(proto);o.length=1;var generic=Array.prototype.values.call(o);assert.sameValue(generic.next().value,8);
        var keys=Array.prototype.keys.call({length:2,get 0(){throw 'keys must not Get';}});assert.sameValue(keys.next().value,0);assert.sameValue(keys.next().value,1);assert.sameValue(keys.next().done,true);
        var e=[5,6].entries(),p=e.next().value,q=e.next().value;assert.compareArray(p,[0,5]);assert.compareArray(q,[1,6]);assert.notSameValue(p,q);
        var old=Array.prototype[Symbol.iterator];Array.prototype[Symbol.iterator]=7;
        function f(x){x=9;assert.sameValue(arguments[Symbol.iterator],old);verifyProperty(arguments,Symbol.iterator,{value:old,writable:true,enumerable:false,configurable:true},{restore:true});return arguments[Symbol.iterator]().next().value;}
        var v=f(3);assert.sameValue(v,9);
    "#.replace("assert.sameValue(v,9);","assert.sameValue(v,(function(){return this===undefined;})()?3:9);").as_str());
}
#[test]
fn native_iterator_length_reentrancy_preserves_saved_index_and_permanent_completion() {
    check(
        r#"
        var active=false,nested,o={0:'zero',1:'one',2:'two',get length(){if(!active){active=true;nested=it.next();active=false;}return 3;}},it=Array.prototype.values.call(o);
        assert.sameValue(it.next().value,'zero');assert.sameValue(nested.value,'zero');
        Object.defineProperty(o,'length',{value:3});assert.sameValue(it.next().value,'one');
        var stage=0,done,o2={0:'kept',get length(){if(stage===0){stage=1;done=it2.next();return 1;}return 0;}},it2=Array.prototype.values.call(o2);
        assert.sameValue(it2.next().value,'kept');assert.sameValue(done.done,true);assert.sameValue(it2.next().done,true);assert.sameValue(stage,1);
        var reason={},reads=0,o3={0:5,get length(){if(reads++===0)throw reason;return 1;}},it3=Array.prototype.values.call(o3);
        try{it3.next();throw 'missing';}catch(e){assert.sameValue(e,reason);}assert.sameValue(it3.next().value,5);
    "#,
    );
}
#[test]
fn native_iterator_index_get_reentrancy_and_throw_observe_already_advanced_index() {
    check(
        r#"
        var nested,it,o={length:3,get 0(){nested=it.next();return 'outer';},1:'inner',2:'last'};it=Array.prototype.values.call(o);
        assert.sameValue(it.next().value,'outer');assert.sameValue(nested.value,'inner');assert.sameValue(it.next().value,'last');
        var reason={},o2={length:2,get 0(){throw reason;},1:8},it2=Array.prototype.values.call(o2);
        try{it2.next();throw 'missing';}catch(e){assert.sameValue(e,reason);}assert.sameValue(it2.next().value,8);
        var o3={length:1,get 0(){assert.sameValue(it3.next().done,true);return 3;}},it3=Array.prototype.values.call(o3);
        assert.sameValue(it3.next().value,3);assert.sameValue(it3.next().done,true);
    "#,
    );
}
#[test]
fn native_iterator_string_coercion_exact_utf16_and_frozen_progress() {
    check(
        r#"
        var calls=0,source={toString:function(){calls++;return 'A\uD83D\uDE00\uD800B\uDC00';}},it=String.prototype[Symbol.iterator].call(source);
        source.toString=function(){throw 'must not reconvert';};Object.freeze(it);
        assert.sameValue(calls,1);assert.sameValue(it.next().value,'A');var pair=it.next().value;assert.sameValue(pair.length,2);assert.sameValue(pair.charCodeAt(0),0xD83D);assert.sameValue(pair.charCodeAt(1),0xDE00);
        assert.sameValue(it.next().value.charCodeAt(0),0xD800);assert.sameValue(it.next().value,'B');assert.sameValue(it.next().value.charCodeAt(0),0xDC00);assert.sameValue(it.next().done,true);assert.sameValue(it.next().done,true);
        assert.throws(TypeError,function(){String.prototype[Symbol.iterator].call(null);});assert.throws(TypeError,function(){String.prototype[Symbol.iterator].call(undefined);});assert.throws(TypeError,function(){String.prototype[Symbol.iterator].call(Symbol('x'));});
        var ai=[1,2].values();Object.freeze(ai);assert.sameValue(ai.next().value,1);assert.sameValue(ai.next().value,2);
    "#,
    );
}
#[test]
fn native_iterator_private_creation_refusal_is_prepublication_and_lookup_work_is_shared() {
    for method in ["array.values", "string.create"] {
        let (mut runtime, mut doc) = prepared();
        let value = if method == "array.values" {
            runtime.array(vec![Value::Number(2.)]).unwrap()
        } else {
            Value::String("ab".into())
        };
        let counts = (runtime.objects.len(), runtime.iterators.entries.len());
        runtime.allocated = MAX_HEAP;
        assert!(
            runtime
                .iterator_native(method, value, &[], &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(
            (runtime.objects.len(), runtime.iterators.entries.len()),
            counts
        );
    }
    let (mut runtime, mut doc) = prepared();
    let array = runtime.array(vec![Value::Number(1.)]).unwrap();
    let it = runtime
        .iterator_native("array.values", array, &[], &mut doc)
        .unwrap();
    let allocated = runtime.allocated;
    runtime.steps = 0;
    assert!(
        runtime
            .iterator_native("array.next", it.clone(), &[], &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.allocated, allocated);
    let Value::Object(id) = it else {
        unreachable!()
    };
    assert!(matches!(
        runtime.iterators.entries[&id],
        IteratorState::Array { index: 0, .. }
    ));
    clean(&runtime);
}
#[test]
fn native_iterator_private_done_releases_sources_but_preserves_brand_and_fresh_results() {
    let (mut runtime, mut doc) = prepared();
    for (create, next_method, value) in [
        ("array.values", "array.next", Value::Undefined),
        ("string.create", "string.next", Value::String("".into())),
    ] {
        let value = if create == "array.values" {
            runtime.array(vec![]).unwrap()
        } else {
            value
        };
        let it = runtime
            .iterator_native(create, value, &[], &mut doc)
            .unwrap();
        let one = runtime
            .iterator_native(next_method, it.clone(), &[], &mut doc)
            .unwrap();
        let two = runtime
            .iterator_native(next_method, it.clone(), &[], &mut doc)
            .unwrap();
        assert_ne!(one, two);
        assert_eq!(result(&runtime, one), (Value::Undefined, true));
        let Value::Object(id) = it else {
            unreachable!()
        };
        assert!(matches!(
            &runtime.iterators.entries[&id],
            IteratorState::Array { object: None, .. } | IteratorState::String { text: None, .. }
        ));
    }
}
#[test]
fn native_iterator_private_huge_logical_indices_are_lazy_and_hosts_fail_only_when_reached() {
    let (mut runtime, mut doc) = prepared();
    let target = runtime
        .execute("({length:Infinity,9007199254740990:7})", &mut doc)
        .unwrap();
    let it = runtime
        .iterator_native("array.values", target, &[], &mut doc)
        .unwrap();
    let Value::Object(id) = it else {
        unreachable!()
    };
    let IteratorState::Array { index, .. } = runtime.iterators.entries.get_mut(&id).unwrap() else {
        unreachable!()
    };
    *index = MAX_LENGTH - 1;
    assert_eq!(
        next(&mut runtime, &mut doc, &Value::Object(id), "array.next"),
        (Value::Number(7.), false)
    );
    assert_eq!(
        next(&mut runtime, &mut doc, &Value::Object(id), "array.next"),
        (Value::Undefined, true)
    );
    let target = runtime
        .object_ordered([
            ("length".into(), Value::Number(1.)),
            ("0".into(), Value::Number(3.)),
        ])
        .unwrap();
    let Value::Object(target_id) = target else {
        unreachable!()
    };
    runtime.objects[target_id].prototype = Some(Value::Window);
    let it = runtime
        .iterator_native("array.values", target, &[], &mut doc)
        .unwrap();
    assert_eq!(
        next(&mut runtime, &mut doc, &it, "array.next"),
        (Value::Number(3.), false)
    );
    let host = runtime
        .iterator_native("array.values", Value::Window, &[], &mut doc)
        .unwrap();
    assert!(
        runtime
            .iterator_native("array.next", host, &[], &mut doc)
            .unwrap_err()
            .is_unsupported()
    );
    clean(&runtime);
}

#[test]
fn native_iterator_private_result_and_entry_preflight_keep_getter_effects_without_partial_publication()
 {
    for creation in ["values", "entries"] {
        let source = format!(
            "var reads=0,target={{length:1,get 0(){{reads++;target.touched=7;return 5;}}}};Array.prototype.{creation}.call(target)"
        );
        let (mut measure, mut doc) = prepared();
        let it = measure.execute(&source, &mut doc).unwrap();
        let before = measure.allocated;
        measure
            .iterator_native("array.next", it, &[], &mut doc)
            .unwrap();
        let staging = 32 + 2 * std::mem::size_of::<Value>();
        let suffix = RESULT_BYTES
            + if creation == "entries" {
                staging * 2 + OBJECT_BYTES
            } else {
                0
            };
        let prefix = measure.allocated - before - suffix;
        let prefix_objects = measure.objects.len() - if creation == "entries" { 2 } else { 1 };
        let (mut runtime, mut doc) = prepared();
        let it = runtime.execute(&source, &mut doc).unwrap();
        let arrays = runtime.arrays.len();
        runtime.allocated = MAX_HEAP - prefix - suffix + 1;
        let error = runtime
            .iterator_native("array.next", it.clone(), &[], &mut doc)
            .unwrap_err();
        assert!(error.is_resource_limit(), "{creation}: {error}");
        assert_eq!(runtime.lookup(1, "reads").unwrap().1, Value::Number(1.));
        let Value::Object(target) = runtime.lookup(1, "target").unwrap().1 else {
            unreachable!()
        };
        assert_eq!(
            runtime.objects[target].get("touched"),
            Some(&Value::Number(7.))
        );
        assert_eq!(runtime.arrays.len(), arrays, "no partial entry array");
        assert_eq!(
            runtime.objects.len(),
            prefix_objects,
            "no partial result or pair property bag"
        );
        let Value::Object(id) = it else {
            unreachable!()
        };
        assert!(matches!(
            runtime.iterators.entries[&id],
            IteratorState::Array {
                index: 1,
                object: Some(_),
                ..
            }
        ));
        clean(&runtime);
    }
    let (mut runtime, mut doc) = prepared();
    let it = runtime
        .iterator_native(
            "string.create",
            Value::String(JsString::from(&[0xd83d, 0xde00][..])),
            &[],
            &mut doc,
        )
        .unwrap();
    let objects = runtime.objects.len();
    runtime.allocated = MAX_HEAP - RESULT_BYTES - 36 + 1;
    assert!(
        runtime
            .iterator_native("string.next", it.clone(), &[], &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.objects.len(), objects);
    let Value::Object(id) = it else {
        unreachable!()
    };
    assert!(matches!(
        runtime.iterators.entries[&id],
        IteratorState::String {
            index: 0,
            text: Some(_)
        }
    ));
    clean(&runtime);
}

#[test]
fn native_iterator_private_tree_lookup_cycles_reentrancy_and_cumulative_limits_are_bounded() {
    let (mut runtime, mut doc) = prepared();
    let array = runtime.array(vec![]).unwrap();
    let it = runtime
        .iterator_native("array.values", array.clone(), &[], &mut doc)
        .unwrap();
    let before = runtime.steps;
    let allocated = runtime.allocated;
    runtime.iterator_snapshot(it.clone()).unwrap();
    let small = before - runtime.steps;
    assert_eq!(runtime.allocated, allocated);
    for _ in 0..31 {
        runtime
            .iterator_native("array.values", array.clone(), &[], &mut doc)
            .unwrap();
    }
    let before = runtime.steps;
    let allocated = runtime.allocated;
    runtime.iterator_snapshot(it).unwrap();
    assert!(before - runtime.steps > small);
    assert_eq!(runtime.allocated, allocated);
    let (mut runtime, mut doc) = prepared();
    let target = runtime.object_ordered([]).unwrap();
    let Value::Object(id) = target else {
        unreachable!()
    };
    runtime.objects[id].prototype = Some(target.clone());
    let it = runtime
        .iterator_native("array.values", target, &[], &mut doc)
        .unwrap();
    assert!(
        runtime
            .iterator_native("array.next", it, &[], &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    clean(&runtime);
    let (mut runtime, mut doc) = prepared();
    let it=runtime.execute("var it;var source={get length(){return it.next().value;}};it=Array.prototype.values.call(source);it",&mut doc).unwrap();
    assert!(
        runtime
            .iterator_native("array.next", it, &[], &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    clean(&runtime);
    let (mut runtime, mut doc) = prepared();
    let it = runtime
        .iterator_native("string.create", Value::String("".into()), &[], &mut doc)
        .unwrap();
    let mut stopped = false;
    for _ in 0..MAX_STEPS {
        match runtime.iterator_native("string.next", it.clone(), &[], &mut doc) {
            Ok(value) => assert_eq!(result(&runtime, value), (Value::Undefined, true)),
            Err(error) => {
                assert!(error.is_resource_limit());
                stopped = true;
                break;
            }
        }
    }
    assert!(stopped, "fresh done results cannot reset shared budgets");
    clean(&runtime);
}
