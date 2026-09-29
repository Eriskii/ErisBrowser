use super::*;

fn fresh() -> (Runtime, Document) {
    (
        Runtime::new(),
        Document::parse("<p id='kept'>unchanged</p>"),
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

fn writable_data(runtime: &Runtime, object: &Value, key: &str) -> Value {
    let PropertyValue::Data {
        value,
        writable: true,
    } = runtime.own_property(object, &key.into()).unwrap().value
    else {
        panic!("writable data")
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
fn array_from_metadata_aliases_and_ignored_hooks() {
    check(
        r#"
        var from=Array.from;
        verifyProperty(Array,'from',{value:from,writable:true,enumerable:false,configurable:true},{restore:true});
        verifyProperty(from,'name',{value:'from',writable:false,enumerable:false,configurable:true},{restore:true});
        verifyProperty(from,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});
        assert.sameValue(from.hasOwnProperty('prototype'),false);
        assert.throws(TypeError,function(){new from([]);});
        var source=[3];Object.defineProperty(source,'constructor',{get:function(){throw 'constructor';}});
        Array.from=null;
        assert.sameValue(from.call(null,source)[0],3);
        assert.sameValue(from.apply(undefined,[{0:4,length:1}])[0],4);
        assert.sameValue(from.bind(null)('ab').length,2);
        var poison={valueOf:function(){throw 'coerce';},toString:function(){throw 'coerce';}};
        assert.sameValue(from.call(null,{length:0},undefined,poison,poison).length,0);
    "#,
    );
}

#[test]
fn array_from_constructor_acquisition_and_fallback_order() {
    check(
        r#"
        var log='',input={},n=0,it={next:function(){log+='n';return n++?{done:true}:{value:6};}};
        Object.defineProperty(input,Symbol.iterator,{get:function(){log+='g';return function(){log+='i';assert.sameValue(this,input);return it;};}});
        function C(){log+='c';assert.sameValue(arguments.length,0);assert.sameValue(new.target,C);}
        var output=Array.from.call(C,input);assert.sameValue(log,'gcinn');assert.sameValue(output[0],6);
        log='';var like={get length(){log+='l';return {valueOf:function(){log+='v';return 1;}};},get 0(){log+='0';return 5;}};
        function D(prefix,len){log+='d';assert.sameValue(prefix,8);assert.sameValue(len,1);assert.sameValue(new.target,D);}
        var d=Array.from.call(D.bind(null,8),like);assert.sameValue(log,'lvd0');assert.sameValue(d[0],5);
        var boxed=Array.from.call(Object,{0:9,length:1});assert.sameValue(boxed.valueOf(),1);assert.sameValue(boxed[0],9);
        var oldArray=Array,proto=Array.prototype,from=Array.from;Array=function(){throw 'global';};
        var a=from.call(()=>{},[]);assert.sameValue(Object.getPrototypeOf(a),proto);assert.sameValue(oldArray.isArray(a),true);
    "#,
    );
}

#[test]
fn array_from_protocol_receivers_step_cache_and_live_values() {
    check(
        r#"
        var gets=0,n=0,it={},input={};input[Symbol.iterator]=function(){return it;};
        Object.defineProperty(it,'next',{get:function(){gets++;return function(){assert.sameValue(this,it);assert.sameValue(arguments.length,0);return n++===0?{get done(){return false;},get value(){return 4;}}:{done:true,get value(){throw 'unused';}};};},configurable:true});
        var context={},out=Array.from(input,function(v,k){'use strict';assert.sameValue(this,context);assert.sameValue(arguments.length,2);assert.sameValue(k,0);Object.defineProperty(it,'next',{value:function(){throw 'new';}});return v+1;},context);
        assert.sameValue(gets,1);assert.sameValue(n,2);assert.sameValue(out[0],5);
        Object.defineProperty(Number.prototype,Symbol.iterator,{get:function(){'use strict';assert.sameValue(this,7);return function(){'use strict';assert.sameValue(this,7);return {next:function(){return {done:true};}};};}});
        assert.sameValue(Array.from(7).length,0);
        var a=[1];var b=Array.from(a,function(v){if(v===1)a.push(2);return v;});assert.sameValue(b.length,2);
    "#,
    );
}

#[test]
fn array_from_close_boundaries_and_throw_identity() {
    check(
        r#"
        function iterable(it){var o={};o[Symbol.iterator]=function(){return it;};return o;}
        var token={},seen,closed=0,it={next:function(){return {value:1};},return:function(){closed++;throw 'close';}};
        try{Array.from(iterable(it),function(){throw token;});}catch(e){seen=e;}
        assert.sameValue(seen,token);assert.sameValue(closed,1);
        for(var stage=0;stage<4;stage++){
          closed=0;seen=undefined;
          it={return:function(){closed++;return {};}};
          if(stage===0)Object.defineProperty(it,'next',{get:function(){throw token;}});
          if(stage===1)it.next=function(){throw token;};
          if(stage===2)it.next=function(){return {get done(){throw token;}};};
          if(stage===3)it.next=function(){return {get value(){throw token;}};};
          try{Array.from(iterable(it));}catch(e){seen=e;}
          assert.sameValue(seen,token);assert.sameValue(closed,0);
        }
        closed=0;it={next:function(){return {done:true};},return:function(){closed++;return {};}};
        var out={};Object.defineProperty(out,'length',{set:function(){throw token;}});function C(){return out;}
        try{Array.from.call(C,iterable(it));}catch(e){seen=e;}
        assert.sameValue(seen,token);assert.sameValue(closed,0);
        it.next=function(){return {value:1};};Object.preventExtensions(out);
        assert.throws(TypeError,function(){Array.from.call(C,iterable(it));});assert.sameValue(closed,1);
    "#,
    );
}

#[test]
fn array_from_live_aliases_descriptors_and_arguments() {
    check(
        r#"
        var a={0:2,1:3,length:2};function C(){return a;}
        var b=Array.from.call(C,a,function(v,k){if(k===0)a[1]=8;return v*2;});
        assert.sameValue(a,b);assert.sameValue(a[0],4);assert.sameValue(a[1],16);
        var setters=0;function D(){}Object.defineProperty(D.prototype,'0',{set:function(){setters++;}});
        var d=Array.from.call(D,[6]);assert.sameValue(setters,0);
        verifyProperty(d,'0',{value:6,writable:true,enumerable:true,configurable:true},{restore:true});
        var f=Function('a','b','function C(){return saved;}var saved=arguments;Array.from.call(C,{0:7,1:9,length:2});return a+b;');
        assert.sameValue(f(1,2),16);
        var mapped=Function('a','b','return Array.from(arguments,function(v,k){if(k===0)b=8;return v;});');
        assert.sameValue(mapped(1,2)[1],8);
        function strict(a,b){'use strict';return Array.from(arguments,function(v,k){if(k===0)b=8;return v;});}
        assert.sameValue(strict(1,2)[1],2);
        var holder={},writes=0;Object.defineProperty(holder,'length',{set:function(v){assert.sameValue(this[0],3);writes+=v;}});
        function E(){return holder;}Array.from.call(E,{0:3,length:1});assert.sameValue(writes,1);
    "#,
    );
}

#[test]
fn array_from_string_units_sparse_inputs_and_large_logical_lengths() {
    check(
        r#"
        var s=Array.from('A\uD800\uD801\uDC00');assert.sameValue(s.length,3);assert.sameValue(s[1],'\uD800');assert.sameValue(s[2].length,2);
        String.prototype[Symbol.iterator]=null;assert.sameValue(Array.from('\uD801\uDC00').length,2);
        var a=Array.from(new Array(2));assert.sameValue(a.hasOwnProperty('0'),true);assert.sameValue(a[0],undefined);
        var marker={},caught,reads=0,like={length:Infinity,get 0(){reads++;throw marker;}};
        function C(n){assert.sameValue(n,9007199254740991);return {};}
        try{Array.from.call(C,like);}catch(e){caught=e;}assert.sameValue(caught,marker);assert.sameValue(reads,1);
        reads=0;assert.throws(RangeError,function(){Array.from(like);});assert.sameValue(reads,0);
        assert.throws(RangeError,function(){Array.from.call(null,like);});assert.sameValue(reads,0);
    "#,
    );
}

#[test]
fn array_from_invalid_mapper_has_no_heap_or_property_prefix() {
    let (mut runtime, mut doc) = fresh();
    let text = Value::String("x".repeat(20_000).into());
    let arguments = [Value::Null, Value::Null, text.clone(), text];
    let arenas = (runtime.objects.len(), runtime.arrays.len());
    runtime.steps = 1;
    runtime.allocated = MAX_HEAP;
    let error = runtime
        .array_from(Value::Undefined, &arguments, &mut doc)
        .unwrap_err();
    assert_eq!(error.name(), "TypeError");
    assert_eq!(runtime.steps, 0);
    assert_eq!(runtime.allocated, MAX_HEAP);
    assert_eq!(arenas, (runtime.objects.len(), runtime.arrays.len()));
    clean(&runtime);
}

#[test]
fn array_from_fallback_range_precedes_allocation_and_sparse_length_has_no_gap_storage() {
    let (mut runtime, mut doc) = fresh();
    let before = (
        runtime.allocated,
        runtime.objects.len(),
        runtime.arrays.len(),
    );
    let error = runtime
        .array_from_result(Value::Null, Some(u64::from(u32::MAX) + 1), &mut doc)
        .unwrap_err();
    assert_eq!(error.name(), "RangeError");
    assert_eq!(
        before,
        (
            runtime.allocated,
            runtime.objects.len(),
            runtime.arrays.len()
        )
    );
    let result = runtime
        .array_from_result(Value::Null, Some(u64::from(u32::MAX)), &mut doc)
        .unwrap();
    let Value::Array(id) = result else {
        panic!("array")
    };
    assert!(runtime.arrays[id].is_empty());
    assert!(runtime.array_holes[id].is_empty());
    assert_eq!(runtime.array_lengths[id].value, u32::MAX);
    assert!(runtime.allocated - before.0 < 1024);
}

#[test]
fn array_from_mapper_precharge_preserves_prior_getter_effects_without_close() {
    let (mut runtime, mut doc) = fresh();
    let source = runtime.execute("var reads=0,maps=0,closed=0;var it={return:function(){closed++;return {};}};var source={get value(){reads++;return 7;}};function mapper(){maps++;}source", &mut doc).unwrap();
    let value = runtime.array_from_get(&source, "value", &mut doc).unwrap();
    let mapper = runtime.lookup(1, "mapper").unwrap().1;
    let iterator = runtime.lookup(1, "it").unwrap().1;
    runtime.allocated = MAX_HEAP - 2 * std::mem::size_of::<Value>() + 1;
    let error = runtime
        .array_from_map(&mapper, &Value::Null, value, 0, &mut doc)
        .unwrap_err();
    let error = runtime.array_from_close(&iterator, error, &mut doc);
    assert!(error.is_resource_limit());
    assert_eq!(runtime.allocated, MAX_HEAP + 1);
    assert_eq!(runtime.lookup(1, "reads").unwrap().1, Value::Number(1.0));
    assert_eq!(runtime.lookup(1, "maps").unwrap().1, Value::Number(0.0));
    assert_eq!(runtime.lookup(1, "closed").unwrap().1, Value::Number(0.0));
    clean(&runtime);
}

fn mapped(name: &str) -> (Runtime, Document, Value, usize, usize) {
    let (mut runtime, mut doc) = fresh();
    let target = runtime
        .execute(
            &format!("function capture({name}){{{name}=17;return arguments;}}capture(1)"),
            &mut doc,
        )
        .unwrap();
    let object = runtime.property_object(&target).unwrap();
    let env = runtime.objects[object].parameter_map[&JsString::from("0")].0;
    (runtime, doc, target, object, env)
}

#[test]
fn array_from_mapped_output_names_charge_before_alias_mutation() {
    let short = "p";
    let long = "p".repeat(1024);
    let mut costs = Vec::new();
    for name in [short, long.as_str()] {
        let (mut runtime, mut doc, target, object, env) = mapped(name);
        let before = (runtime.steps, runtime.allocated);
        runtime
            .array_from_define(&target, "0".into(), Value::Number(8.0), &mut doc)
            .unwrap();
        costs.push((before.0 - runtime.steps, runtime.allocated - before.1));
        assert_eq!(
            runtime.environments[env].bindings[name].value,
            Value::Number(8.0)
        );
        assert!(
            runtime.objects[object]
                .parameter_map
                .contains_key(&JsString::from("0"))
        );
    }
    assert!(costs[1].0 > costs[0].0 + long.len());
    assert_eq!(costs[0].1, costs[1].1);
    let (mut runtime, mut doc, target, object, env) = mapped(&long);
    runtime.steps = costs[0].0;
    let error = runtime
        .array_from_define(&target, "0".into(), Value::Number(8.0), &mut doc)
        .unwrap_err();
    assert!(error.is_resource_limit());
    assert_eq!(
        runtime.environments[env].bindings[&long].value,
        Value::Number(17.0)
    );
    assert_eq!(runtime.objects[object].parameter_map.len(), 1);
    assert_eq!(writable_data(&runtime, &target, "0"), Value::Number(17.0));
    clean(&runtime);
}

#[test]
fn array_from_output_budget_failure_preserves_prefix_and_defers_length_setter() {
    let (mut runtime, mut doc) = fresh();
    let out = runtime
        .execute(
            "var writes=0;var out={set length(n){writes++;}};out",
            &mut doc,
        )
        .unwrap();
    runtime
        .array_from_define(&out, "0".into(), Value::Number(7.0), &mut doc)
        .unwrap();
    runtime.allocated = MAX_HEAP;
    let error = runtime
        .array_from_define(&out, "1".into(), Value::Number(8.0), &mut doc)
        .unwrap_err();
    assert!(error.is_resource_limit());
    assert_eq!(writable_data(&runtime, &out, "0"), Value::Number(7.0));
    assert!(runtime.own_property(&out, &"1".into()).is_none());
    let error = runtime
        .array_from_set_length(&out, 1, &mut doc)
        .unwrap_err();
    assert!(error.is_resource_limit());
    assert_eq!(runtime.lookup(1, "writes").unwrap().1, Value::Number(0.0));
    clean(&runtime);
}

#[test]
fn array_from_symbol_walk_and_final_write_charge_tree_shape_and_bound_cycles() {
    let (mut runtime, mut doc) = fresh();
    let small = runtime.object_ordered([]).unwrap();
    let large = runtime
        .object_ordered((0..256).map(|i| (format!("property{i}").into(), Value::Number(0.0))))
        .unwrap();
    let before = runtime.steps;
    assert!(
        runtime
            .array_from_iterator_method(&small, &mut doc)
            .unwrap()
            .is_none()
    );
    let small_cost = before - runtime.steps;
    let before = runtime.steps;
    assert!(
        runtime
            .array_from_iterator_method(&large, &mut doc)
            .unwrap()
            .is_none()
    );
    assert!(before - runtime.steps > small_cost);
    let before = runtime.steps;
    runtime.array_from_set_length(&small, 0, &mut doc).unwrap();
    let small_cost = before - runtime.steps;
    let before = runtime.steps;
    runtime.array_from_set_length(&large, 0, &mut doc).unwrap();
    assert!(before - runtime.steps > small_cost);
    let Value::Object(a) = small else {
        panic!("object")
    };
    let Value::Object(b) = large else {
        panic!("object")
    };
    runtime.objects[a].prototype = Some(Value::Object(b));
    runtime.objects[b].prototype = Some(Value::Object(a));
    let error = runtime
        .array_from_iterator_method(&Value::Object(a), &mut doc)
        .unwrap_err();
    assert!(error.is_resource_limit());
    clean(&runtime);
}

#[test]
fn array_from_host_edges_are_lazy_and_safe_integer_guard_closes_before_next() {
    let (mut runtime, mut doc) = fresh();
    let source=runtime.execute("var closed=0,nextCalls=0;var it={next:function(){nextCalls++;return {done:true};},return:function(){closed++;throw 'close';}};var source={};source[Symbol.iterator]=function(){return it;};source",&mut doc).unwrap();
    let object = runtime.property_object(&source).unwrap();
    runtime.objects[object].prototype = Some(Value::Window);
    assert!(
        runtime
            .array_from_iterator_method(&source, &mut doc)
            .unwrap()
            .is_some()
    );
    let key = runtime.well_known_key("iterator");
    runtime.objects[object].remove(&key);
    assert!(
        runtime
            .array_from_iterator_method(&source, &mut doc)
            .unwrap_err()
            .is_unsupported()
    );
    let iterator = runtime.lookup(1, "it").unwrap().1;
    runtime
        .array_from_count(&iterator, MAX_LENGTH - 1, &mut doc)
        .unwrap();
    let error = runtime
        .array_from_count(&iterator, MAX_LENGTH, &mut doc)
        .unwrap_err();
    assert_eq!(error.name(), "TypeError");
    assert_eq!(runtime.lookup(1, "closed").unwrap().1, Value::Number(1.0));
    assert_eq!(
        runtime.lookup(1, "nextCalls").unwrap().1,
        Value::Number(0.0)
    );
    for index in [u64::from(u32::MAX), u64::from(u32::MAX) + 1, MAX_LENGTH] {
        assert_eq!(
            runtime.reduce_index_key(index).unwrap().to_utf8().unwrap(),
            index.to_string()
        );
    }
    clean(&runtime);
}

#[test]
fn array_from_terminal_stops_skip_author_close_catch_finally_and_cleanup() {
    for body in [
        "var n=0;var it={next:function(){n++;return {value:n};},return:function(){closed++;return {};}};var o={};o[Symbol.iterator]=function(){return it;};Array.from(o);",
        "function recursive(){return Array.from([1],recursive);}recursive();",
        "var it={next:function(){return {value:1};},return:function(){closed++;while(true){}}};var o={};o[Symbol.iterator]=function(){return it;};Array.from(o,function(){throw 1;});",
    ] {
        let (mut runtime, mut doc) = fresh();
        let error=runtime.execute(&format!("var closed=0,caught=0,finalized=0;try{{{body}}}catch(e){{caught++;}}finally{{finalized++;}}"),&mut doc).unwrap_err();
        assert!(error.is_resource_limit(), "{error:?}");
        assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Number(0.0));
        assert_eq!(
            runtime.lookup(1, "finalized").unwrap().1,
            Value::Number(0.0)
        );
        assert_eq!(
            runtime.lookup(1, "closed").unwrap().1,
            Value::Number(if body.contains("closed++;while") {
                1.0
            } else {
                0.0
            })
        );
        clean(&runtime);
    }
}

#[test]
fn array_from_repeated_native_calls_share_allocation_and_work() {
    let (mut runtime, mut doc) = fresh();
    let source = runtime
        .object_ordered([
            ("0".into(), Value::Number(1.0)),
            ("length".into(), Value::Number(1.0)),
        ])
        .unwrap();
    let mapper = runtime.lookup(1, "Boolean").unwrap().1;
    let args = [source, mapper];
    let before = (runtime.steps, runtime.allocated);
    let output = runtime.array_from(Value::Null, &args, &mut doc).unwrap();
    assert_eq!(writable_data(&runtime, &output, "0"), Value::Bool(true));
    let cost = (before.0 - runtime.steps, runtime.allocated - before.1);
    runtime.steps = MAX_STEPS;
    runtime.allocated = MAX_HEAP - cost.1 * 2;
    runtime.array_from(Value::Null, &args, &mut doc).unwrap();
    runtime.array_from(Value::Null, &args, &mut doc).unwrap();
    assert_eq!(runtime.allocated, MAX_HEAP);
    assert!(
        runtime
            .array_from(Value::Null, &args, &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert!(runtime.steps < MAX_STEPS);
    clean(&runtime);
}

#[test]
fn array_from_shrink_preserves_names_without_full_name_dedup_work() {
    // The former long-name exhaustion oracle is retained in the before
    // evidence. Ranked snapshot dedup removes those actual comparisons.
    let mut costs = Vec::new();
    for prefix in ["x".to_owned(), "x".repeat(512)] {
        let prepare = || {
            let (mut runtime, doc) = fresh();
            let array = runtime.array(vec![Value::Number(7.0)]).unwrap();
            let object = runtime.property_object(&array).unwrap();
            for i in 0..24 {
                runtime.objects[object].insert(format!("{prefix}{i}").into(), Value::Bool(true));
            }
            (runtime, doc, array, object)
        };
        let (mut runtime, mut doc, array, object) = prepare();
        let before = runtime.steps;
        runtime.array_from_shrink_budget(&array, 0).unwrap();
        let preflight = before - runtime.steps;
        let before = (runtime.steps, runtime.allocated);
        runtime.array_from_set_length(&array, 0, &mut doc).unwrap();
        costs.push((
            preflight,
            before.0 - runtime.steps,
            runtime.allocated - before.1,
        ));
        let Value::Array(id) = array else {
            panic!("array")
        };
        assert_eq!(runtime.array_lengths[id].value, 0);
        assert!(runtime.arrays[id].is_empty());
        assert_eq!(runtime.objects[object].order.len(), 24);
        for i in 0..24 {
            assert_eq!(
                writable_data(&runtime, &array, &format!("{prefix}{i}")),
                Value::Bool(true)
            );
        }
        let (mut runtime, _, array, _) = prepare();
        runtime.steps = preflight - 1;
        assert!(
            runtime
                .array_from_shrink_budget(&array, 0)
                .unwrap_err()
                .is_resource_limit()
        );
        let Value::Array(id) = array else {
            panic!("array")
        };
        assert_eq!(runtime.array_lengths[id].value, 1);
        assert_eq!(runtime.arrays[id], vec![Value::Number(7.0)]);
        runtime.steps = 1;
        runtime.array_from_shrink_budget(&array, 1).unwrap();
        runtime.array_lengths[id].writable = false;
        runtime.array_from_shrink_budget(&array, 0).unwrap();
        assert_eq!(runtime.steps, 1);
        clean(&runtime);
    }
    assert_eq!(costs[0], costs[1]);
}

#[test]
fn array_from_unsupported_output_is_terminal_after_prior_iterator_effects() {
    let (mut runtime, mut doc) = fresh();
    let error = runtime
        .execute(
            r#"
        var nextCalls=0,closed=0,caught=0,finalized=0;
        var input={};input[Symbol.iterator]=function(){return {
            next:function(){nextCalls++;return {value:7};},
            return:function(){closed++;return {};}
        };};
        function C(){return document;}
        try{Array.from.call(C,input);}catch(e){caught++;}finally{finalized++;}
    "#,
            &mut doc,
        )
        .unwrap_err();
    assert!(error.is_unsupported());
    assert_eq!(
        runtime.lookup(1, "nextCalls").unwrap().1,
        Value::Number(1.0)
    );
    for name in ["closed", "caught", "finalized"] {
        assert_eq!(runtime.lookup(1, name).unwrap().1, Value::Number(0.0));
    }
    clean(&runtime);
}
