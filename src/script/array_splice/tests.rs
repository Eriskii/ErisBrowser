use super::*;

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

fn data(runtime: &Runtime, object: &Value, key: &str) -> Value {
    let PropertyValue::Data { value, .. } =
        runtime.own_property(object, &key.into()).unwrap().value
    else {
        panic!("data property")
    };
    value
}

fn check(source: &str) {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        for helper in [
            include_str!("../../../tests/upstream/test262/harness/sta.js"),
            include_str!("../../../tests/upstream/test262/harness/assert.js"),
            include_str!("../../../tests/upstream/test262/harness/propertyHelper.js"),
        ] {
            runtime.execute(helper, &mut doc).unwrap();
        }
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        assert!(result.is_ok(), "strict={strict}: {result:?}");
        clean(&runtime);
    }
}

#[test]
fn splice_metadata_saved_aliases_and_nonconstructor() {
    check(
        r#"
        var s=Array.prototype.splice;
        verifyProperty(Array.prototype,'splice',{value:s,writable:true,enumerable:false,configurable:true},{restore:true});
        verifyProperty(s,'name',{value:'splice',writable:false,enumerable:false,configurable:true},{restore:true});
        verifyProperty(s,'length',{value:2,writable:false,enumerable:false,configurable:true},{restore:true});
        assert.sameValue(s.hasOwnProperty('prototype'),false);
        assert.throws(TypeError,function(){new s();});
        Array.prototype.splice=undefined;
        var a=[1,2,3];assert.sameValue(s.call(a,1,1,8)[0],2);assert.sameValue(a.join(','),'1,8,3');
        assert.sameValue(s.apply(a,[0,1])[0],1);assert.sameValue(s.bind(a)(1)[0],3);
        assert.sameValue(a.join(','),'8');
        assert.throws(TypeError,function(){s.call(null);});assert.throws(TypeError,function(){s.call(undefined);});
    "#,
    );
}

#[test]
fn splice_conversion_presence_order_and_overflow() {
    check(
        r#"
        var s=Array.prototype.splice,a=[1,2,3];assert.sameValue(s.call(a).length,0);assert.sameValue(a.length,3);
        assert.sameValue(s.call(a,1,undefined).length,0);assert.sameValue(a.length,3);
        assert.sameValue(s.call(a,1).join(','),'2,3');
        var log='',o={get length(){log+='l';return {valueOf:function(){log+='n';return 3;}};},set length(n){log+='L';assert.sameValue(n,3);}};
        var start={valueOf:function(){log+='s';return -2;}},count={valueOf:function(){log+='d';return 0;}};
        s.call(o,start,count);assert.sameValue(log,'lnsdL');
        var n={};n[Symbol.toPrimitive]=function(h){assert.sameValue(h,'number');return 1;};
        assert.sameValue(s.call([3],n,0).length,0);
        assert.throws(TypeError,function(){s.call({length:Symbol()},0,0);});
        var huge={length:9007199254740991};Object.defineProperty(huge,'9007199254740990',{get:function(){throw 'index';}});
        assert.throws(TypeError,function(){s.call(huge,9007199254740990,0,7);});
        assert.sameValue(huge.length,9007199254740991);
    "#,
    );
}

#[test]
fn splice_species_order_original_receivers_and_result_length() {
    check(
        r#"
        var s=Array.prototype.splice,log='',stored=0,a=[7],out={},holder={};
        Object.defineProperty(a,'0',{get:function(){log+='g';return 7;},set:function(v){log+='w';stored=v;},configurable:true});
        Object.defineProperty(a,'constructor',{get:function(){assert.sameValue(this,a);log+='c';return holder;}});
        Object.defineProperty(holder,Symbol.species,{get:function(){assert.sameValue(this,holder);log+='p';return C;}});
        function C(n){log+='n';assert.sameValue(arguments.length,1);assert.sameValue(n,1);assert.sameValue(new.target,C);return out;}
        Object.defineProperty(out,'length',{set:function(n){assert.sameValue(this,out);assert.sameValue(this[0],7);assert.sameValue(n,1);log+='l';}});
        assert.sameValue(s.call(a,0,1,8),out);assert.sameValue(log,'cpnglw');assert.sameValue(stored,8);
        var generic={0:5,length:1};Object.defineProperty(generic,'constructor',{get:function(){throw 'ignored';}});
        assert.sameValue(s.call(generic,0,1)[0],5);
    "#,
    );
}

#[test]
fn splice_species_defaults_bound_native_and_invalid() {
    check(
        r#"
        var s=Array.prototype.splice,Original=Array,proto=Array.prototype;
        var a=[1];a.constructor=undefined;Array=function(){throw 'global';};
        var r=s.call(a,0,1);assert.sameValue(Object.getPrototypeOf(r),proto);assert.sameValue(r[0],1);Array=Original;
        function C(prefix,n){assert.sameValue(prefix,4);assert.sameValue(n,1);assert.sameValue(new.target,C);}
        var b=[6],h={};h[Symbol.species]=C.bind(null,4);b.constructor=h;
        r=s.call(b,0,1);assert.sameValue(Object.getPrototypeOf(r),C.prototype);assert.sameValue(r[0],6);
        var c=[8];h={};h[Symbol.species]=Object;c.constructor=h;r=s.call(c,0,1);assert.sameValue(r.valueOf(),1);assert.sameValue(r[0],8);
        var d=[9];d.constructor=null;assert.throws(TypeError,function(){s.call(d,0,1);});assert.sameValue(d[0],9);
        d.constructor={};d.constructor[Symbol.species]=()=>{};assert.throws(TypeError,function(){s.call(d,0,1);});
    "#,
    );
}

#[test]
fn splice_alias_and_sparse_result_definitions_are_live() {
    check(
        r#"
        var s=Array.prototype.splice,a=[0,1,2,3],h={};h[Symbol.species]=function(){return a;};a.constructor=h;
        assert.sameValue(s.call(a,1,2,'x'),a);assert.sameValue(a.length,3);assert.sameValue(a[0],1);assert.sameValue(a[1],'x');assert.sameValue(a.hasOwnProperty('2'),false);
        var out={1:'kept'},b=[7,,9];h={};h[Symbol.species]=function(){return out;};b.constructor=h;
        assert.sameValue(s.call(b,0,2),out);assert.sameValue(out[0],7);assert.sameValue(out[1],'kept');assert.sameValue(out.length,2);
        var hits=0,p={};Object.defineProperty(p,'0',{set:function(){hits++;throw 'setter';}});
        var c=[4];h={};h[Symbol.species]=function(){return Object.create(p);};c.constructor=h;
        var r=s.call(c,0,1);assert.sameValue(hits,0);verifyProperty(r,'0',{value:4,writable:true,enumerable:true,configurable:true},{restore:true});
    "#,
    );
}

#[test]
fn splice_has_get_sparse_prototype_and_captured_length() {
    check(
        r#"
        var s=Array.prototype.splice,p={},o=Object.create(p),calls=0;
        Object.defineProperty(p,'1',{get:function(){assert.sameValue(this,o);calls++;return 42;},configurable:true});o.length=3;o[2]=9;
        var r=s.call(o,0,2);assert.sameValue(r.hasOwnProperty('0'),false);assert.sameValue(r[1],42);assert.sameValue(calls,1);assert.sameValue(o[0],9);assert.sameValue(o.length,1);
        var a=[0,1,2,3],start={valueOf:function(){a.length=1;a[2]=9;return 1;}};
        r=s.call(a,start,1);assert.sameValue(r.hasOwnProperty('0'),false);assert.sameValue(a.length,3);assert.sameValue(a[1],9);assert.sameValue(a.hasOwnProperty('2'),false);
        var b={1:20,length:3};Object.defineProperty(b,'0',{get:function(){delete b[1];b[2]=99;return 10;},configurable:true});
        r=s.call(b,0,3);assert.sameValue(r[0],10);assert.sameValue(r.hasOwnProperty('1'),false);assert.sameValue(r[2],99);
    "#,
    );
}

#[test]
fn splice_partial_strict_set_delete_and_length_failures() {
    check(
        r#"
        var s=Array.prototype.splice,token={},seen,o={0:'a',1:'b',length:3},reads=0;
        Object.defineProperty(o,'2',{get:function(){reads++;return 'c';},set:function(){throw token;},configurable:true});
        try{s.call(o,0,0,'x');}catch(e){seen=e;}assert.sameValue(seen,token);assert.sameValue(reads,1);assert.sameValue(o[3],'c');assert.sameValue(o[1],'b');assert.sameValue(o.length,3);
        var a=[1,2,3];Object.defineProperty(a,'2',{configurable:false});assert.throws(TypeError,function(){s.call(a,0,1);});
        assert.sameValue(a[0],2);assert.sameValue(a[1],3);assert.sameValue(a.length,3);
        var b={0:8,length:1};Object.defineProperty(b,'length',{writable:false});assert.throws(TypeError,function(){s.call(b,0,1);});
        assert.sameValue(b.hasOwnProperty('0'),false);assert.sameValue(b.length,1);
        var c=[4],out={},h={};Object.defineProperty(out,'0',{value:1});h[Symbol.species]=function(){return out;};c.constructor=h;
        assert.throws(TypeError,function(){s.call(c,0,1);});assert.sameValue(c[0],4);assert.sameValue(out[0],1);
    "#,
    );
}

#[test]
fn splice_mapped_arguments_and_boxed_string_partial_growth() {
    check(
        r#"
        var mapped=Function('x','y','Array.prototype.splice.call(arguments,0,1);return [x,y,arguments.length,arguments[0]];')(3,4);
        assert.sameValue(mapped.join(','),'4,4,1,4');
        var unmapped=(function(x,y){'use strict';Array.prototype.splice.call(arguments,0,1);return [x,y,arguments.length,arguments[0]];})(3,4);
        assert.sameValue(unmapped.join(','),'3,4,1,4');
        var out=Function('x','return {args:arguments,read:function(){return x;}};')(2),a=[9],h={};h[Symbol.species]=function(){return out.args;};a.constructor=h;
        assert.sameValue(a.splice(0,1),out.args);assert.sameValue(out.read(),9);
        var boxed=new String('ab');assert.throws(TypeError,function(){Array.prototype.splice.call(boxed,0,0,'x');});
        assert.sameValue(boxed[2],'b');assert.sameValue(boxed[0],'a');assert.sameValue(boxed.length,2);
    "#,
    );
}

#[test]
fn splice_full_safe_lengths_and_late_array_range_error() {
    check(
        r#"
        var s=Array.prototype.splice,o={length:9007199254740991};o[9007199254740990]=7;
        var r=s.call(o,9007199254740990,1);assert.sameValue(r[0],7);assert.sameValue(o.length,9007199254740990);assert.sameValue(o.hasOwnProperty('9007199254740990'),false);
        o={length:4294967296};o[4294967295]=9;r=s.call(o,4294967295,1);assert.sameValue(r[0],9);assert.sameValue(o.length,4294967295);
        var a=new Array(4294967295);assert.throws(RangeError,function(){s.call(a,4294967295,0,8);});
        assert.sameValue(a[4294967295],8);assert.sameValue(a.length,4294967295);
    "#,
    );
}

#[test]
fn splice_fallback_preflight_and_sparse_storage_do_not_allocate_gaps() {
    let (mut runtime, _) = fresh();
    let before = (
        runtime.arrays.len(),
        runtime.objects.len(),
        runtime.allocated,
    );
    assert_eq!(
        runtime
            .splice_array_create(u64::from(u32::MAX) + 1)
            .unwrap_err()
            .name(),
        "RangeError"
    );
    assert_eq!(
        (
            runtime.arrays.len(),
            runtime.objects.len(),
            runtime.allocated
        ),
        before
    );
    runtime.allocated = MAX_HEAP;
    assert!(
        runtime
            .splice_array_create(1)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(
        (runtime.arrays.len(), runtime.objects.len()),
        (before.0, before.1)
    );
    runtime.allocated = before.2;
    let Value::Array(id) = runtime.splice_array_create(u32::MAX.into()).unwrap() else {
        panic!("array")
    };
    assert!(runtime.arrays[id].is_empty());
    assert_eq!(runtime.array_lengths[id].value, u32::MAX);
    clean(&runtime);
}

#[test]
fn splice_heap_stop_keeps_written_prefix_and_defers_final_setter() {
    let (mut runtime, mut doc) = fresh();
    let out = runtime
        .execute(
            "var writes=0;var out={set length(n){writes++;}};out",
            &mut doc,
        )
        .unwrap();
    runtime
        .splice_define(&out, "0".into(), Value::Number(7.0), true, &mut doc)
        .unwrap();
    runtime.allocated = MAX_HEAP;
    assert!(
        runtime
            .splice_define(&out, "1".into(), Value::Number(8.0), true, &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(data(&runtime, &out, "0"), Value::Number(7.0));
    assert!(runtime.own_property(&out, &"1".into()).is_none());
    assert!(
        runtime
            .splice_length(&out, 1, &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.lookup(1, "writes").unwrap().1, Value::Number(0.0));
    clean(&runtime);
}

#[test]
fn splice_mapped_name_search_is_borrowed_and_prepaid_before_write() {
    let mut costs = Vec::new();
    for size in [1, 2048] {
        let (mut runtime, mut doc) = fresh();
        let name = "m".repeat(size);
        let env = runtime.environment(1).unwrap();
        runtime
            .define(env, &name, Value::Number(3.0), true)
            .unwrap();
        let object = runtime
            .object_ordered([("0".into(), Value::Number(3.0))])
            .unwrap();
        let id = runtime.property_object(&object).unwrap();
        runtime.objects[id]
            .parameter_map
            .insert("0".into(), (env, name.clone()));
        let before = (runtime.steps, runtime.allocated);
        runtime
            .splice_define(&object, "0".into(), Value::Number(4.0), true, &mut doc)
            .unwrap();
        costs.push((before.0 - runtime.steps, runtime.allocated - before.1));
        assert_eq!(
            runtime.environments[env].bindings[&name].value,
            Value::Number(4.0)
        );
        runtime.environments[env]
            .bindings
            .get_mut(&name)
            .unwrap()
            .value = Value::Number(3.0);
        // The initial environment-search allowance must fail before the shared write.
        runtime.steps = 1 + search_work(1, 1) * 8 + search_work(1, 1) * 9;
        assert!(
            runtime
                .splice_define(&object, "0".into(), Value::Number(8.0), true, &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(
            runtime.environments[env].bindings[&name].value,
            Value::Number(3.0)
        );
        clean(&runtime);
    }
    assert!(costs[1].0 > costs[0].0);
    assert_eq!(costs[0].1, costs[1].1);
}

#[test]
fn splice_delete_charges_retained_handles_without_scanning_long_names() {
    let mut costs = Vec::new();
    for size in [1, 2048] {
        let (mut runtime, _) = fresh();
        let prefix = "x".repeat(size);
        let object = runtime
            .object_ordered(
                std::iter::once(("0".into(), Value::Number(8.0)))
                    .chain((0..24).map(|i| (format!("{prefix}{i}").into(), Value::Bool(true)))),
            )
            .unwrap();
        let before = (runtime.steps, runtime.allocated);
        runtime.splice_delete(&object, &"0".into()).unwrap();
        costs.push((before.0 - runtime.steps, runtime.allocated - before.1));
        assert!(runtime.own_property(&object, &"0".into()).is_none());
        assert_eq!(
            runtime.objects[runtime.property_object(&object).unwrap()]
                .order
                .len(),
            24
        );
        clean(&runtime);
    }
    assert_eq!(costs[0], costs[1]);
}

#[test]
fn splice_vector_relocation_is_prepaid_before_dense_or_order_growth() {
    for dense in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let object = if dense {
            runtime.array(vec![Value::Number(1.0)]).unwrap()
        } else {
            runtime
                .object_ordered([("0".into(), Value::Number(1.0))])
                .unwrap()
        };
        let id = runtime.property_object(&object).unwrap();
        let (slots, normal_insertion, slot_bytes) = if let Value::Array(array) = &object {
            runtime.arrays[*array].shrink_to_fit();
            assert_eq!(
                runtime.arrays[*array].len(),
                runtime.arrays[*array].capacity()
            );
            (
                runtime.arrays[*array].len(),
                std::mem::size_of::<Value>(),
                std::mem::size_of::<Value>(),
            )
        } else {
            runtime.objects[id].order.shrink_to_fit();
            assert_eq!(
                runtime.objects[id].order.len(),
                runtime.objects[id].order.capacity()
            );
            (
                runtime.objects[id].order.len(),
                256 + 4,
                std::mem::size_of::<PropertyKey>(),
            )
        };
        let buffer = (2 * slots).max(4) * slot_bytes;
        // One less than fixed scratch + new buffer + shared insertion. Without
        // the relocation charge the ordinary insertion would succeed, so this
        // specifically detects an omitted buffer preflight.
        runtime.allocated = MAX_HEAP - (256 + buffer + normal_insertion - 1);
        assert!(
            runtime
                .splice_define(&object, "1".into(), Value::Number(2.0), true, &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(data(&runtime, &object, "0"), Value::Number(1.0));
        assert!(runtime.own_property(&object, &"1".into()).is_none());
        clean(&runtime);
    }
}

#[test]
fn splice_shrink_budget_precedes_shared_deletions_and_skips_unreached_work() {
    let prepare = || {
        let (mut runtime, doc) = fresh();
        let array = runtime
            .array(vec![Value::Number(7.0), Value::Number(8.0)])
            .unwrap();
        let id = runtime.property_object(&array).unwrap();
        for i in 0..24 {
            runtime.objects[id].insert(format!("tag{i}").into(), Value::Bool(true));
        }
        (runtime, doc, array)
    };
    let (mut runtime, mut doc, array) = prepare();
    let Value::Array(id) = array else {
        panic!("array")
    };
    let before = runtime.steps;
    runtime.splice_shrink_budget(id, 0.0).unwrap();
    let cost = before - runtime.steps;
    runtime
        .splice_length(&Value::Array(id), 0, &mut doc)
        .unwrap();
    assert_eq!(runtime.array_lengths[id].value, 0);
    assert!(runtime.arrays[id].is_empty());
    let (mut runtime, _, array) = prepare();
    let Value::Array(id) = array else {
        panic!("array")
    };
    runtime.steps = cost - 1;
    assert!(
        runtime
            .splice_shrink_budget(id, 0.0)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.array_lengths[id].value, 2);
    assert_eq!(runtime.arrays[id][0], Value::Number(7.0));
    runtime.steps = 1;
    runtime.splice_shrink_budget(id, 2.0).unwrap();
    runtime.array_lengths[id].writable = false;
    runtime.splice_shrink_budget(id, 0.0).unwrap();
    assert_eq!(runtime.steps, 1);
    clean(&runtime);
}

#[test]
fn splice_lazy_host_and_prototype_cycles_are_bounded() {
    let (mut runtime, mut doc) = fresh();
    let object = runtime
        .object_ordered([("0".into(), Value::Number(7.0))])
        .unwrap();
    let id = runtime.property_object(&object).unwrap();
    runtime.objects[id].prototype = Some(Value::Window);
    assert_eq!(
        runtime.splice_get(&object, &"0".into(), &mut doc).unwrap(),
        Value::Number(7.0)
    );
    assert!(
        runtime
            .splice_get(&object, &"1".into(), &mut doc)
            .unwrap_err()
            .is_unsupported()
    );
    runtime.objects[id].prototype = Some(object.clone());
    assert!(
        runtime
            .splice_symbol_get(&object, "species", &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    clean(&runtime);
}

#[test]
fn splice_terminal_recursion_and_wide_scan_skip_author_cleanup() {
    for body in [
        "var a=[1],h={};h[Symbol.species]=function(){return a.splice(0,1);};a.constructor=h;a.splice(0,1);",
        "Array.prototype.splice.call({length:9007199254740990},0,0,7);",
        "Array.prototype.splice.call({length:9007199254740990},0,1);",
    ] {
        let (mut runtime, mut doc) = fresh();
        let error=runtime.execute(&format!("var caught=0,finalized=0;try{{{body}}}catch(e){{caught++;}}finally{{finalized++;}}"),&mut doc).unwrap_err();
        assert!(error.is_resource_limit(), "{error:?}");
        assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Number(0.0));
        assert_eq!(
            runtime.lookup(1, "finalized").unwrap().1,
            Value::Number(0.0)
        );
        clean(&runtime);
    }
}

#[test]
fn splice_repeated_calls_share_heap_and_work_ledgers() {
    let (mut runtime, mut doc) = fresh();
    let source = runtime
        .object_ordered([("length".into(), Value::Number(0.0))])
        .unwrap();
    let args = [Value::Number(0.0), Value::Number(0.0)];
    let before = (runtime.steps, runtime.allocated);
    runtime
        .array_splice(source.clone(), &args, &mut doc)
        .unwrap();
    let cost = (before.0 - runtime.steps, runtime.allocated - before.1);
    runtime.steps = MAX_STEPS;
    runtime.allocated = MAX_HEAP - cost.1 * 2;
    runtime
        .array_splice(source.clone(), &args, &mut doc)
        .unwrap();
    runtime
        .array_splice(source.clone(), &args, &mut doc)
        .unwrap();
    assert_eq!(runtime.allocated, MAX_HEAP);
    assert!(
        runtime
            .array_splice(source, &args, &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert!(runtime.steps < MAX_STEPS);
    clean(&runtime);
}
