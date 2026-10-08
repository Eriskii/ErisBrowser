use super::*;

fn fresh() -> (Runtime, Document) {
    (
        Runtime::try_new().unwrap(),
        Document::parse("<p>retained</p>"),
    )
}

fn clean(runtime: &Runtime) {
    assert!(runtime.frames.is_empty());
    assert!(runtime.active_array_joins.is_empty());
    assert_eq!(
        (
            runtime.calls,
            runtime.eval_depth,
            runtime.stack_units,
            runtime.json_depth
        ),
        (0, 0, 0, 0)
    );
}

fn check(source: &str) {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        assert_eq!(
            result.unwrap_or_else(|e| panic!("strict={strict}: {e}")),
            Value::Bool(true)
        );
        clean(&runtime);
    }
}

fn saved(runtime: &Runtime, name: &str) -> Value {
    let Value::Object(owner) = runtime.environments[0].bindings["Reflect"].value else {
        panic!("installed Reflect object")
    };
    let key = PropertyKey::from(name);
    let PropertyValue::Data { value, .. } = &runtime.objects[owner].values[&key].value else {
        panic!("installed method")
    };
    value.clone()
}

#[test]
fn window_presence_omits_lexicals_and_descriptor_reads_do_not_invoke_accessors() {
    check(
        r#"
        var reflectVisible=7; let reflectLexical=9;
        var calls=0, reason={}, key=Symbol('own');
        function getter(){calls++;throw reason;}
        Object.defineProperty(globalThis,'reflectAccessor',{get:getter,configurable:true});
        globalThis[key]=13;
        var d=Reflect.getOwnPropertyDescriptor(globalThis,'reflectAccessor');
        if(!Reflect.has(globalThis,'reflectVisible')||Reflect.has(globalThis,'reflectLexical')||
           Reflect.get(globalThis,'reflectLexical')!==undefined||d.get!==getter||calls!==0||
           Reflect.get(globalThis,key)!==13)throw new Error('global presence');
        var caught;try{Reflect.get(globalThis,'reflectAccessor');}catch(e){caught=e;}
        if(caught!==reason||calls!==1)throw new Error('global getter identity');
        true;
    "#,
    );
}

#[test]
fn array_length_refusal_preserves_required_partial_deletion() {
    check(
        r#"
        var a=[0,1,2,3,4];
        Object.defineProperty(a,'3',{configurable:false});
        if(Reflect.set(a,'length',1)!==false||a.length!==4||Reflect.has(a,'4')||
           a[3]!==3||a[0]!==0)throw new Error('partial array length refusal');
        var receiver=[], target={length:7};
        if(!Reflect.set(target,'length',3,receiver)||receiver.length!==3||target.length!==7)
            throw new Error('Receiver exotic length');
        var calls=0, value={valueOf:function(){calls++;return 2;}};
        if(!Reflect.set(receiver,'length',value)||calls!==2||receiver.length!==2)
            throw new Error('two length conversions');
        true;
    "#,
    );
}

#[test]
fn receiver_accessor_refusal_and_target_callback_keep_distinct_identities() {
    check(
        r#"
        var key=Symbol('x'), target={}, receiver={}, calls=0, seen, payload={};
        target[key]=1;
        Object.defineProperty(receiver,key,{set:function(){calls++;},configurable:true});
        if(Reflect.set(target,key,payload,receiver)!==false||calls!==0||target[key]!==1)
            throw new Error('Receiver accessor is not target setter');
        Object.defineProperty(target,'setter',{set:function(v){'use strict';seen=this;if(v!==payload)throw new Error('value');}});
        if(!Reflect.set(target,'setter',payload,undefined)||seen!==undefined||
           !Reflect.set(target,'setter',payload,5)||seen!==5)throw new Error('primitive Receiver');
        var reason=new TypeError('exact'), caught;
        Object.defineProperty(target,'abrupt',{set:function(){throw reason;}});
        try{Reflect.set(target,'abrupt',0,receiver);}catch(e){caught=e;}
        if(caught!==reason)throw new Error('TypeError cannot be Boolean refusal');
        true;
    "#,
    );
}

#[test]
fn arguments_mapping_and_boxed_string_virtual_descriptors_remain_exact() {
    let (mut runtime, mut doc) = fresh();
    let source = r#"
        function f(a){
            if(!Reflect.set({'0':1},'0',7,arguments)||a!==7)throw new Error('mapped argument');
            Object.defineProperty(arguments,'0',{writable:false});
            if(Reflect.set({'0':1},'0',8,arguments)!==false||a!==7)throw new Error('readonly mapped argument');
        }
        f(1);
        var s=Object('A\uD800'), d=Reflect.getOwnPropertyDescriptor(s,'1');
        if(d.value.charCodeAt(0)!==55296||d.writable||!d.enumerable||d.configurable||
           Reflect.set(s,'1','x')!==false||Reflect.deleteProperty(s,'1')!==false)
            throw new Error('exact virtual string descriptor');
        true;
    "#;
    assert_eq!(
        runtime.execute(source, &mut doc).unwrap(),
        Value::Bool(true)
    );
    clean(&runtime);
}

#[test]
fn typed_array_receiver_definitions_and_live_key_callbacks_use_fresh_state() {
    check(
        r#"
        var a=new Uint8Array(1), b=new Uint8Array(0), ordinary={'0':1}, calls=0;
        a[0]=17;
        var rhs={valueOf:function(){calls++;return 9;}};
        if(!Reflect.set(a,'1',rhs,{})||Reflect.set(ordinary,'0',rhs,b)!==false||calls!==0)
            throw new Error('invalid target versus Receiver');
        var receiver={};
        if(!Reflect.set(a,'0',rhs,receiver)||receiver[0]!==rhs||calls!==0||a[0]!==17)
            throw new Error('distinct ordinary Receiver');
        var rab=new ArrayBuffer(0,{maxByteLength:4}), view=new Uint8Array(rab);
        rhs.valueOf=function(){calls++;rab.resize(1);return 23;};
        if(!Reflect.set(view,'0',rhs)||view[0]!==23||calls!==1)throw new Error('same Receiver growth');
        var key={toString:function(){rab.resize(0);return '0';}};
        if(Reflect.getOwnPropertyDescriptor(view,key)!==undefined||Reflect.has(view,'0'))
            throw new Error('key callback fresh bound');
        if(Reflect.preventExtensions(view)!==false||!Reflect.isExtensible(view))
            throw new Error('resizable refusal');
        true;
    "#,
    );
}

#[test]
fn represented_dom_descriptors_keep_receiver_while_legacy_fallback_is_explicit() {
    let (mut runtime, mut doc) = fresh();
    let node = Value::Node(doc.create_text_node("text"));
    let key = Value::String("nodeValue".into());
    assert_eq!(
        runtime
            .reflect_property_call("Reflect.get", &[node.clone(), key.clone()], &mut doc)
            .unwrap(),
        Value::String("text".into())
    );
    let other = runtime.object_ordered([]).unwrap();
    assert_eq!(
        runtime
            .reflect_property_call("Reflect.get", &[node.clone(), key, other], &mut doc)
            .unwrap_err()
            .name(),
        "TypeError"
    );
    for name in ["Reflect.has", "Reflect.get", "Reflect.set"] {
        let error = runtime
            .reflect_property_call(
                name,
                &[
                    node.clone(),
                    Value::String("nodeType".into()),
                    Value::Number(7.0),
                ],
                &mut doc,
            )
            .unwrap_err();
        assert!(error.is_unsupported());
    }
    assert!(
        runtime
            .read_own_property_key(&node, &PropertyKey::from("nodeType"))
            .unwrap()
            .is_none()
    );
    assert!(
        runtime
            .reflect_property_call("Reflect.isExtensible", &[Value::Window], &mut doc)
            .unwrap_err()
            .is_unsupported()
    );
    clean(&runtime);
}

fn prototype_body() -> (Runtime, Document, Value, Value) {
    let (mut runtime, doc) = fresh();
    let target = runtime.object_ordered([]).unwrap();
    let next = runtime.object_ordered([]).unwrap();
    let Value::Object(next_id) = next else {
        unreachable!()
    };
    runtime.objects[next_id].prototype = None;
    (runtime, doc, target, next)
}

#[test]
fn prototype_body_all_work_cuts_precede_mutation_and_require_no_heap() {
    let (mut measured, mut doc, target, next) = prototype_body();
    let initial = measured.steps;
    measured.allocated = MAX_HEAP;
    assert_eq!(
        measured
            .reflect_property_call("Reflect.setPrototypeOf", &[target, next], &mut doc)
            .unwrap(),
        Value::Bool(true)
    );
    let work = initial - measured.steps;
    // Original walk is 20 work; the intrinsic identity guard adds one paid lookup.
    assert_eq!(work, 20 + 2 + search(measured.prototypes.len(), 6));
    assert_eq!(measured.allocated, MAX_HEAP);
    for cut in 0..=work {
        let (mut runtime, mut doc, target, next) = prototype_body();
        let Value::Object(id) = target else {
            unreachable!()
        };
        let old = runtime.objects[id].prototype.clone();
        runtime.steps = cut;
        runtime.allocated = MAX_HEAP;
        let result = runtime.reflect_property_call(
            "Reflect.setPrototypeOf",
            &[target, next.clone()],
            &mut doc,
        );
        if cut == work {
            assert_eq!(result.unwrap(), Value::Bool(true));
            assert_eq!(runtime.objects[id].prototype, Some(next));
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            assert_eq!(runtime.objects[id].prototype, old);
        }
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        clean(&runtime);
    }
}

#[test]
fn saved_nine_native_preflight_boundaries_preserve_metadata_and_cleanup() {
    for name in [
        "has",
        "get",
        "set",
        "deleteProperty",
        "getOwnPropertyDescriptor",
        "getPrototypeOf",
        "setPrototypeOf",
        "isExtensible",
        "preventExtensions",
    ] {
        let (mut runtime, mut doc) = fresh();
        let function = saved(&runtime, name);
        let Value::Native(native) = &function else {
            panic!("installed Native")
        };
        assert_eq!(native.name, format!("Reflect.{name}"));
        assert_eq!(native.receiver, Value::Undefined);
        assert!(native.properties.is_some());
        let identity = native.clone();
        let work = 4 + native.name.len();
        let heap = 32 + native.name.len();
        let bag = runtime.property_object(&function).unwrap();
        let metadata = format!("{:?}", runtime.objects[bag].values);
        let objects = runtime.objects.len();
        for (steps, room, charged) in [
            (work, heap, 0),
            (1 + work, heap, heap),
            (1 + work, heap - 1, heap),
        ] {
            runtime.steps = steps;
            runtime.allocated = MAX_HEAP - room;
            let before = runtime.allocated;
            let error = runtime
                .call(function.clone(), Vec::new(), Value::Null, &mut doc)
                .unwrap_err();
            assert!(error.is_resource_limit(), "{name}");
            assert_eq!(runtime.steps, 0, "{name}");
            assert_eq!(runtime.allocated, before + charged, "{name}");
            assert_eq!(runtime.objects.len(), objects);
            assert_eq!(format!("{:?}", runtime.objects[bag].values), metadata);
            let Value::Native(current) = &function else {
                unreachable!()
            };
            assert!(Rc::ptr_eq(current, &identity));
            clean(&runtime);
        }
    }
}

#[test]
fn saved_native_full_call_exact_and_one_short_have_stable_boolean_and_guards() {
    let (mut measured, mut doc) = fresh();
    let function = saved(&measured, "isExtensible");
    let target = measured.object_ordered([]).unwrap();
    let start = (measured.steps, measured.allocated);
    assert_eq!(
        measured
            .call(function, vec![target], Value::Null, &mut doc)
            .unwrap(),
        Value::Bool(true)
    );
    let (work, heap) = (start.0 - measured.steps, measured.allocated - start.1);
    assert!(work > 8 && heap > 0);
    for (steps, room, success) in [
        (work, heap, true),
        (work - 1, heap, false),
        (work, heap - 1, false),
    ] {
        let (mut runtime, mut doc) = fresh();
        let function = saved(&runtime, "isExtensible");
        let target = runtime.object_ordered([]).unwrap();
        let Value::Object(id) = target else {
            unreachable!()
        };
        runtime.steps = steps;
        runtime.allocated = MAX_HEAP - room;
        let result = runtime.call(function, vec![target], Value::Bool(false), &mut doc);
        if success {
            assert_eq!(result.unwrap(), Value::Bool(true));
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        assert!(!runtime.objects[id].non_extensible);
        clean(&runtime);
    }
}

#[test]
fn descriptor_heap_endpoint_retains_completed_key_callback_without_target_mutation() {
    fn setup() -> (Runtime, Document, Value, Value) {
        let (mut runtime, mut doc) = fresh();
        runtime.execute("var effect=0;var target={x:19};var key={toString:function(){effect=1;return 'x';}};", &mut doc).unwrap();
        let target = runtime.environments[0].bindings["target"].value.clone();
        let key = runtime.environments[0].bindings["key"].value.clone();
        (runtime, doc, target, key)
    }
    let (mut measured, mut doc, target, key) = setup();
    let before = measured.allocated;
    let result = measured
        .reflect_property_call("Reflect.getOwnPropertyDescriptor", &[target, key], &mut doc)
        .unwrap();
    assert!(matches!(result, Value::Object(_)));
    let heap = measured.allocated - before;
    assert!(heap > 1000);
    for success in [true, false] {
        let (mut runtime, mut doc, target, key) = setup();
        runtime.allocated = MAX_HEAP - heap + usize::from(!success);
        let result = runtime.reflect_property_call(
            "Reflect.getOwnPropertyDescriptor",
            &[target.clone(), key],
            &mut doc,
        );
        if success {
            assert!(matches!(result.unwrap(), Value::Object(_)));
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        assert_eq!(
            runtime.environments[0].bindings["effect"].value,
            Value::Number(1.0)
        );
        match runtime
            .own_property(&target, &JsString::from("x"))
            .unwrap()
            .value
        {
            PropertyValue::Data { value, writable } => {
                assert_eq!(value, Value::Number(19.0));
                assert!(writable);
            }
            _ => panic!("target data property retained"),
        }
        clean(&runtime);
    }
}

#[test]
fn intrinsic_object_prototype_is_immutable_even_after_global_rename() {
    check(
        r#"
        var originalObject=Object, p=Object.prototype, replacement={};
        if(Object.getPrototypeOf(p)!==null||!Object.isExtensible(p))throw new Error('initial intrinsic');
        p.reflectOwnWitness=17;
        if(p.reflectOwnWitness!==17||!delete p.reflectOwnWitness)throw new Error('own extensibility');
        if(!Reflect.setPrototypeOf(p,null)||Object.setPrototypeOf(p,null)!==p)
            throw new Error('same null');
        Object={prototype:replacement};
        if(Reflect.setPrototypeOf(p,replacement)!==false)throw new Error('renamed intrinsic identity');
        var caught;try{originalObject.setPrototypeOf(p,replacement);}catch(e){caught=e;}
        if(!(caught instanceof TypeError)||originalObject.getPrototypeOf(p)!==null)
            throw new Error('Object immutable refusal');
        if(!Reflect.setPrototypeOf(Object.prototype,null))throw new Error('replacement is ordinary');
        originalObject.preventExtensions(p);
        if(!Reflect.setPrototypeOf(p,null)||originalObject.setPrototypeOf(p,null)!==p||
           Reflect.setPrototypeOf(p,replacement)!==false)throw new Error('nonextensible intrinsic no-op');
        var ordinary=originalObject.create(replacement);
        originalObject.preventExtensions(ordinary);
        if(!Reflect.setPrototypeOf(ordinary,replacement)||
           originalObject.setPrototypeOf(ordinary,replacement)!==ordinary||
           Reflect.setPrototypeOf(ordinary,null)!==false)throw new Error('ordinary nonextensible');
        caught=undefined;try{originalObject.setPrototypeOf(ordinary,null);}catch(e){caught=e;}
        if(!(caught instanceof TypeError))throw new Error('ordinary Object refusal');
        Object=originalObject;
        true;
    "#,
    );
}

#[test]
fn immutable_prototype_identity_lookup_is_prepaid_and_heap_free() {
    for reflect in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let id = runtime.prototypes["Object"];
        let target = Value::Object(id);
        let next = runtime.object_ordered([]).unwrap();
        let prefix = if reflect { 8 + 6 } else { 2 };
        let work = prefix + 2 + search(runtime.prototypes.len(), 6);
        for budget in [work - 1, work] {
            runtime.steps = budget;
            runtime.allocated = MAX_HEAP;
            let result = if reflect {
                runtime.reflect_property_call(
                    "Reflect.setPrototypeOf",
                    &[target.clone(), next.clone()],
                    &mut doc,
                )
            } else {
                runtime
                    .set_object_prototype_in(&target, next.clone(), &doc)
                    .map(|()| Value::Bool(true))
            };
            if budget < work {
                assert!(result.unwrap_err().is_resource_limit());
            } else if reflect {
                assert_eq!(result.unwrap(), Value::Bool(false));
            } else {
                assert!(matches!(
                    result.unwrap_err().kind,
                    ErrorKind::Runtime("TypeError")
                ));
            }
            assert_eq!(runtime.objects[id].prototype, None);
            assert!(!runtime.objects[id].non_extensible);
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            clean(&runtime);
        }
    }
}
