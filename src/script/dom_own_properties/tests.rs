use super::*;

fn fresh() -> (Runtime, Document) {
    (
        Runtime::new(),
        Document::parse(
            "<!doctype html><html><head><title>before</title></head><body></body></html>",
        ),
    )
}
fn clean(runtime: &Runtime) {
    assert_eq!(runtime.calls, 0);
    assert_eq!(runtime.stack_units, 0);
    assert_eq!(runtime.eval_depth, 0);
    assert!(runtime.frames.is_empty());
}
fn check(source: &str) {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        result.unwrap_or_else(|error| panic!("strict={strict}: {error:?}"));
        clean(&runtime);
    }
}
fn data(value: f64) -> PropertyDescriptor {
    PropertyDescriptor::data_property(Value::Number(value), true, true, true)
}
fn bag(runtime: &Runtime, receiver: &Value) -> Option<usize> {
    runtime
        .host_symbol_objects
        .get(&host(receiver).unwrap())
        .copied()
}
fn populated(count: usize) -> Runtime {
    let mut runtime = Runtime::new();
    for n in 0..count {
        runtime
            .dom_define_own(
                &Value::Node(123),
                &PropertyKey::from(format!("k{n}")),
                data(n as f64),
            )
            .unwrap();
    }
    runtime.steps = MAX_STEPS;
    runtime
}

#[test]
fn independent_method_shadow_descriptor_and_key_fixture_both_modes() {
    check(include_str!(
        "../../../tests/fixtures/dom-own-properties.js"
    ));
}

#[test]
fn fresh_reads_missing_deletes_and_node_enumeration_allocate_no_bag() {
    let (mut runtime, mut doc) = fresh();
    let node = Value::Node(doc.create_document_fragment());
    let objects = runtime.objects.len();
    let bytes = runtime.allocated;
    for target in [&Value::Document, &node] {
        assert!(
            runtime
                .read_own_property(target, &"missing".into())
                .unwrap()
                .is_none()
        );
        assert!(runtime.dom_delete_own(target, &"missing".into()).unwrap());
        assert!(bag(&runtime, target).is_none());
    }
    assert_eq!(runtime.allocated, bytes);
    assert!(runtime.own_keys(&node).unwrap().is_empty());
    assert!(runtime.own_symbol_keys(&node).unwrap().is_empty());
    assert_eq!(runtime.objects.len(), objects);
    assert!(runtime.host_symbol_objects.is_empty());
}

#[test]
fn string_symbol_storage_is_shared_and_runtime_isolated() {
    let (mut runtime, _) = fresh();
    let target = Value::Node(5);
    let symbol = PropertyKey::Symbol(Symbol::unique(None));
    runtime.dom_define_own(&target, &symbol, data(9.0)).unwrap();
    let original = bag(&runtime, &target).unwrap();
    runtime
        .dom_define_own(&target, &"text".into(), data(4.0))
        .unwrap();
    assert_eq!(bag(&runtime, &target), Some(original));
    assert_eq!(
        runtime.objects[original].order,
        [symbol.clone(), "text".into()]
    );
    assert!(
        runtime
            .read_own_property_key(&Value::Node(6), &symbol)
            .unwrap()
            .is_none()
    );
    assert!(
        Runtime::new()
            .read_own_property_key(&target, &symbol)
            .unwrap()
            .is_none()
    );
}

#[test]
fn first_bag_and_property_growth_exact_and_one_short_admission() {
    for count in [0, 1, 4, 11, 12] {
        let target = Value::Node(123);
        let key = PropertyKey::from("next");
        let mut measured = populated(count);
        let before = measured.allocated;
        measured.dom_define_own(&target, &key, data(77.0)).unwrap();
        let work = MAX_STEPS - measured.steps;
        let bytes = measured.allocated - before;
        assert!(work > 0 && bytes > 0);
        for short_work in [true, false] {
            let mut runtime = populated(count);
            let objects = runtime.objects.len();
            let entries = runtime.host_symbol_objects.len();
            let old_order = bag(&runtime, &target).map(|id| runtime.objects[id].order.clone());
            runtime.steps = if short_work { work - 1 } else { work };
            runtime.allocated = MAX_HEAP - bytes + usize::from(!short_work);
            let error = runtime
                .dom_define_own(&target, &key, data(77.0))
                .unwrap_err();
            assert!(error.is_resource_limit());
            assert_eq!(runtime.objects.len(), objects);
            assert_eq!(runtime.host_symbol_objects.len(), entries);
            assert_eq!(
                bag(&runtime, &target).map(|id| runtime.objects[id].order.clone()),
                old_order
            );
            if let Some(id) = bag(&runtime, &target) {
                assert!(!runtime.objects[id].values.contains_key(&key));
            }
        }
        let mut exact = populated(count);
        exact.steps = work;
        exact.allocated = MAX_HEAP - bytes;
        assert!(exact.dom_define_own(&target, &key, data(77.0)).unwrap());
        assert_eq!(exact.steps, 0);
        assert_eq!(exact.allocated, MAX_HEAP);
    }
}

#[test]
fn full_object_arena_reserve_refusal_publishes_no_host_bag() {
    fn full() -> Runtime {
        let mut runtime = Runtime::new();
        runtime.objects.shrink_to_fit();
        assert_eq!(runtime.objects.len(), runtime.objects.capacity());
        runtime.steps = MAX_STEPS;
        runtime
    }
    let mut measured = full();
    let before = measured.allocated;
    measured
        .dom_define_own(&Value::Document, &"x".into(), data(1.0))
        .unwrap();
    let bytes = measured.allocated - before;
    let mut failed = full();
    let old_capacity = failed.objects.capacity();
    failed.allocated = MAX_HEAP - bytes + 1;
    assert!(
        failed
            .dom_define_own(&Value::Document, &"x".into(), data(1.0))
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(failed.objects.capacity(), old_capacity);
    assert!(failed.host_symbol_objects.is_empty());
}

#[test]
fn existing_descriptor_update_is_heap_free_and_refuses_before_mutation() {
    let target = Value::Node(123);
    let key = PropertyKey::from("k0");
    let mut measured = populated(1);
    let before = measured.allocated;
    measured.dom_define_own(&target, &key, data(9.0)).unwrap();
    let work = MAX_STEPS - measured.steps;
    assert_eq!(measured.allocated, before);
    let mut runtime = populated(1);
    runtime.steps = work - 1;
    runtime.allocated = MAX_HEAP;
    assert!(
        runtime
            .dom_define_own(&target, &key, data(9.0))
            .unwrap_err()
            .is_resource_limit()
    );
    let id = bag(&runtime, &target).unwrap();
    assert_eq!(runtime.objects[id].get(key), Some(&Value::Number(0.0)));
}

#[test]
fn nonconfigurable_descriptors_use_paid_same_value_and_no_coercion() {
    check(
        r#"
      var node=document.createDocumentFragment(), hooks=0;
      Object.defineProperty(node,'n',{value:NaN});
      Object.defineProperty(node,'n',{value:NaN});
      Object.defineProperty(node,'z',{value:-0});
      if(Reflect.defineProperty(node,'z',{value:0})) throw new Error('signed zero');
      var value={valueOf:function(){hooks++;return 1;}};
      Object.defineProperty(node,'v',{value:value});
      if(Reflect.defineProperty(node,'v',{value:{}})) throw new Error('object identity');
      if(hooks!==0) throw new Error('coercion');
      var get=function(){return this;};
      Object.defineProperty(node,'a',{get:get});
      if(!Reflect.defineProperty(node,'a',{get:get}) || Reflect.defineProperty(node,'a',{get:function(){}})) throw new Error('getter identity');
      if(node.a!==node) throw new Error('receiver');
    "#,
    );
    let (mut runtime, _) = fresh();
    let value = Value::String(JsString::from(vec![65; 8192]));
    runtime
        .dom_define_own(
            &Value::Node(0),
            &"long".into(),
            PropertyDescriptor::data_property(value.clone(), false, true, false),
        )
        .unwrap();
    runtime.steps = 100;
    assert!(
        runtime
            .dom_define_own(
                &Value::Node(0),
                &"long".into(),
                PropertyDescriptor {
                    value: Some(value),
                    ..PropertyDescriptor::default()
                }
            )
            .unwrap_err()
            .is_resource_limit()
    );
}

#[test]
fn descriptor_conversion_observes_callback_mutation_before_definition() {
    check(
        r#"
      var node=document.createDocumentFragment(), effects=0;
      node.x=1;
      var descriptor={get value(){effects++;Object.defineProperty(node,'x',{value:7,writable:false,configurable:false});return 9;}};
      if(Reflect.defineProperty(node,'x',descriptor)) throw new Error('stale descriptor');
      if(effects!==1 || node.x!==7) throw new Error('lost prior effect');
      var peer=document.createDocumentFragment(), calls=0;
      Object.defineProperty(node,'r',{configurable:true,get:function(){calls++;delete this.r;peer.y=5;this.r=8;return 3;}});
      if(node.r!==3 || node.r!==8 || peer.y!==5 || calls!==1) throw new Error('reentrant getter');
      Object.defineProperty(node,'s',{configurable:true,set:function(v){delete this.s;this.s=v;peer.y++;}});
      node.s=12;
      if(node.s!==12 || peer.y!==6) throw new Error('reentrant setter');
    "#,
    );
}

#[test]
fn own_native_attribute_shadows_preserve_underlying_mutations() {
    check(
        r#"
      var node=document.createElement('div');node.textContent='under';node.extra=1;
      if(node.textContent!=='under') throw new Error('native text');
      Object.defineProperty(node,'textContent',{value:undefined,writable:true,configurable:true});
      if(node.textContent!==undefined) throw new Error('undefined shadow');
      node.textContent='over';delete node.textContent;
      if(node.textContent!=='under') throw new Error('shadow leaked into tree');
      node.textContent='after';if(node.textContent!=='after') throw new Error('restored setter');
      var before=document.title;Object.defineProperty(document,'title',{value:'shadow',writable:true,configurable:true});
      document.title='shadow2';delete document.title;
      if(document.title!==before) throw new Error('title shadow leaked');
      document.title='native-after';if(document.title!=='native-after') throw new Error('title setter');
      Object.defineProperty(document,'URL',{value:41,writable:true,configurable:true});document.URL=42;
      if(document.URL!==42) throw new Error('readonly fallback took precedence');delete document.URL;
      if(typeof document.URL!=='string') throw new Error('URL restore');
    "#,
    );
}

#[test]
fn own_event_handler_shadow_does_not_change_registered_listener() {
    check(
        r#"
      var node=document.createElement('button'), calls=0;
      var handler=function(){calls++;};node.onclick=handler;
      Object.defineProperty(node,'onclick',{value:99,writable:true,configurable:true});
      if(node.onclick!==99) throw new Error('get precedence');node.onclick=100;
      node.dispatchEvent(new Event('click'));
      if(calls!==1 || node.onclick!==100) throw new Error('hidden handler lost');
      delete node.onclick;if(node.onclick!==handler) throw new Error('handler restore');
      node.dispatchEvent(new Event('click'));if(calls!==2) throw new Error('restored dispatch');
    "#,
    );
}

#[test]
fn snapshots_and_for_in_read_live_descriptors_without_invoking_accessors() {
    check(
        r#"
      var node=document.createDocumentFragment(), gets=0;
      node.a=1;node.b=2;
      Object.defineProperty(node,'hidden',{get:function(){gets++;return 3;},enumerable:false});
      var names=Object.getOwnPropertyNames(node), keys=Object.keys(node);
      if(names.join(',')!=='a,b,hidden' || keys.join(',')!=='a,b' || gets!==0) throw new Error('own enumeration');
      var seen='';for(var key in node){if(key==='a'){delete node.b;node.c=3;}if(key==='a'||key==='b'||key==='c'||key==='hidden')seen+=key;}
      if(seen!=='a' || gets!==0) throw new Error('live descriptor walk');
      var values=Object.values(node);if(values.join(',')!=='1,3') throw new Error('values');
    "#,
    );
}

#[test]
fn detach_reinsert_retains_descriptors_clone_does_not_copy_them() {
    check(
        r#"
      var node=document.createElement('div');node.x=7;document.body.appendChild(node);node.remove();document.body.appendChild(node);
      if(node.x!==7) throw new Error('detach identity');
      var copy=node.cloneNode(true);
      if(copy.x!==undefined || Object.getOwnPropertyNames(copy).length!==0) throw new Error('clone copied expando');
    "#,
    );
}

#[test]
fn prototype_and_unsupported_integrity_boundaries_do_not_depend_on_storage() {
    let (mut runtime, mut doc) = fresh();
    let node = Value::Node(doc.create_document_fragment());
    for target in [&Value::Document, &node] {
        let original = runtime.prototype_of(target);
        for populated in [false, true] {
            if populated {
                runtime
                    .dom_define_own(target, &"x".into(), data(1.0))
                    .unwrap();
            }
            assert_eq!(runtime.prototype_of(target), original);
            assert!(runtime.property_object(target).is_none());
            let error = runtime
                .set_object_prototype(target, Value::Null)
                .unwrap_err();
            assert!(error.is_unsupported());
        }
    }
    for code in [
        "Object.preventExtensions(document)",
        "Object.freeze(document)",
        "Object.isExtensible(document)",
        "Object.getOwnPropertyNames(document)",
        "Object.getOwnPropertyDescriptor(document,'location')",
    ] {
        let error = runtime.execute(code, &mut doc).unwrap_err();
        assert!(error.is_unsupported(), "{code}: {error:?}");
    }
}

#[test]
fn ordinary_names_follow_receiver_interface_without_value_coercion() {
    check(
        r#"
      var fragment=document.createDocumentFragment(),text=document.createTextNode('text'),div=document.createElement('div');
      var hooks=0,value={toString:function(){hooks++;throw new Error('coerced');}};
      if(fragment.id!==undefined || text.innerHTML!==undefined || div.value!==undefined) throw new Error('inapplicable native getter');
      fragment.id=value;fragment.onclick=value;text.innerHTML=value;text.onclick=value;div.value=value;
      if(fragment.id!==value || fragment.onclick!==value || text.innerHTML!==value || text.onclick!==value || div.value!==value || hooks!==0) throw new Error('wrong interface');
      delete fragment.id;delete text.innerHTML;delete div.value;
      if(fragment.id!==undefined || text.innerHTML!==undefined || div.value!==undefined) throw new Error('inapplicable fallback revealed');
      var input=document.createElement('input');input.value='native';
      if(input.value!=='native' || Object.getOwnPropertyDescriptor(input,'value')!==undefined) throw new Error('native input setter');
    "#,
    );
}

#[test]
fn readonly_native_fields_reject_before_conversion_and_allow_explicit_shadows() {
    check(
        r#"
      var text=document.createTextNode('x'),hooks=0,value={toString:function(){hooks++;return 'oops';}};
      var failed=false;
      try{(function(){'use strict';text.nodeType=value;})();}catch(e){failed=e instanceof TypeError;}
      if(!failed || text.nodeType!==3 || hooks!==0) throw new Error('readonly strict conversion');
      try{text.parentNode=value;}catch(e){if(!(e instanceof TypeError))throw e;}
      if(text.parentNode!==null || hooks!==0) throw new Error('readonly sloppy conversion');
      Object.defineProperty(text,'nodeType',{value:value,writable:true,configurable:true});
      if(text.nodeType!==value) throw new Error('own readonly shadow');
      delete text.nodeType;if(text.nodeType!==3) throw new Error('native readonly restored');
    "#,
    );
}

#[test]
fn terminal_accessor_keeps_prior_effects_and_cleans_machine_state() {
    let (mut runtime, mut doc) = fresh();
    runtime.execute("var node=document.createDocumentFragment();Object.defineProperty(node,'stop',{get:function(){node.prior=1;while(true){};}});", &mut doc).unwrap();
    let error = runtime
        .execute(
            "try{node.stop;}catch(e){node.caught=1;}finally{node.after=1;}",
            &mut doc,
        )
        .unwrap_err();
    assert!(error.is_resource_limit());
    clean(&runtime);
    assert_eq!(
        runtime
            .execute(
                "node.prior===1 && node.caught===undefined && node.after===undefined",
                &mut doc
            )
            .unwrap(),
        Value::Bool(true)
    );
}

#[test]
fn setter_argument_admission_precedes_author_code_for_strings_and_symbols() {
    for symbol in [false, true] {
        let (mut runtime, mut doc) = fresh();
        runtime.execute("var node=document.createDocumentFragment();var setter=function(v){this.called=v;};", &mut doc).unwrap();
        let node = runtime.execute("node", &mut doc).unwrap();
        let setter = runtime.execute("setter", &mut doc).unwrap();
        let key = if symbol {
            PropertyKey::Symbol(Symbol::unique(None))
        } else {
            "setter".into()
        };
        runtime
            .dom_define_own(
                &node,
                &key,
                PropertyDescriptor {
                    set: Some(setter),
                    configurable: Some(true),
                    ..PropertyDescriptor::default()
                },
            )
            .unwrap();
        runtime.allocated = MAX_HEAP - std::mem::size_of::<Value>() + 1;
        let error = runtime
            .set_property_key(node.clone(), &key, Value::Number(71.0), true, &mut doc)
            .unwrap_err();
        assert!(error.is_resource_limit());
        let id = bag(&runtime, &node).unwrap();
        assert!(!runtime.objects[id].contains_key("called"));
        clean(&runtime);
    }
}

#[test]
fn direct_get_key_copy_is_paid_and_empty_bags_copy_nothing() {
    let (mut runtime, mut doc) = fresh();
    let node = Value::Node(doc.create_element("div"));
    let key = "x".repeat(4096);
    let before = runtime.allocated;
    assert!(
        runtime
            .dom_own_get_utf8(&node, &key, &mut doc)
            .unwrap()
            .is_none()
    );
    assert_eq!(runtime.allocated, before);
    runtime
        .dom_define_own(&node, &"present".into(), data(1.0))
        .unwrap();
    let before = runtime.allocated;
    runtime.dom_own_get_utf8(&node, &key, &mut doc).unwrap();
    let bytes = runtime.allocated - before;
    assert!(bytes > key.len() * 2);
    runtime.allocated = MAX_HEAP - bytes + 1;
    assert!(
        runtime
            .dom_own_get_utf8(&node, &key, &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(
        runtime.objects[bag(&runtime, &node).unwrap()].values.len(),
        1
    );
    runtime.allocated = before;
    runtime.dom_delete_own(&node, &"present".into()).unwrap();
    let before = runtime.allocated;
    assert!(
        runtime
            .dom_own_get_utf8(&node, &key, &mut doc)
            .unwrap()
            .is_none()
    );
    assert_eq!(runtime.allocated, before);
}

#[test]
fn ordinary_inherited_values_on_inapplicable_names_use_both_get_routes() {
    let (mut runtime, mut doc) = fresh();
    runtime.execute("Object.defineProperty(Object.prototype,'onclick',{configurable:true,get:function(){return this;}});", &mut doc).unwrap();
    let node = Value::Node(doc.create_text_node("x"));
    assert_eq!(
        runtime.get(node.clone(), "onclick", &mut doc).unwrap(),
        node
    );
    assert_eq!(
        runtime
            .get_key(node.clone(), &"onclick".into(), &mut doc)
            .unwrap(),
        node
    );
    assert!(bag(&runtime, &node).is_none());
}
