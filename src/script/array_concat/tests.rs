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
fn concat_metadata_saved_aliases_and_separate_string_method() {
    check(
        r#"
        var c=Array.prototype.concat;
        verifyProperty(Array.prototype,'concat',{value:c,writable:true,enumerable:false,configurable:true},{restore:true});
        verifyProperty(c,'name',{value:'concat',writable:false,enumerable:false,configurable:true},{restore:true});
        verifyProperty(c,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});
        assert.sameValue(c.hasOwnProperty('prototype'),false);
        assert.throws(TypeError,function(){new c();});
        assert.throws(TypeError,function(){new (c.bind([]))();});
        assert.throws(TypeError,function(){c.call(null);});assert.throws(TypeError,function(){c.call(undefined);});
        Array.prototype.concat=undefined;var a=[1],bound=c.bind(a,2);
        assert.sameValue(c.call(a,3).join(','),'1,3');assert.sameValue(c.apply(a,[[4],5]).join(','),'1,4,5');
        assert.sameValue(bound(6).join(','),'1,2,6');assert.sameValue(a.length,1);
        assert.sameValue('ab'.concat('c',7),'abc7');
    "#,
    );
}

#[test]
fn concat_primitive_arguments_skip_hooks_and_flags_use_boolean() {
    check(
        r#"
        var c=Array.prototype.concat,sp=Symbol.isConcatSpreadable,n=0;
        var ps=[String.prototype,Number.prototype,Boolean.prototype,Symbol.prototype];
        for(var i=0;i<ps.length;i++)Object.defineProperty(ps[i],sp,{get:function(){n++;throw 'hook';},configurable:true});
        try{var s=Symbol('s'),r=c.call([],'ab',7,false,s,null,undefined);
          assert.sameValue(r.length,6);assert.sameValue(r[0],'ab');assert.sameValue(r[3],s);assert.sameValue(n,0);
        }finally{for(var j=0;j<ps.length;j++)delete ps[j][sp];}
        var boxed=c.call(7)[0];assert.sameValue(typeof boxed,'object');assert.sameValue(boxed.valueOf(),7);
        var text=Object('A\uD83D\uDE00\uD800');text[sp]=true;r=c.call(text);
        assert.sameValue(r.length,4);assert.sameValue(r[1],'\uD83D');assert.sameValue(r[2],'\uDE00');assert.sameValue(r[3],'\uD800');
        var flag={valueOf:function(){throw 'coercion';},toString:function(){throw 'coercion';}};
        var o={0:8,length:1};o[sp]=flag;assert.sameValue(c.call([],o)[0],8);
        o[sp]=null;Object.defineProperty(o,'length',{get:function(){throw 'length';}});assert.sameValue(c.call([],o)[0],o);
        var a=[2];a[sp]=undefined;assert.sameValue(c.call(a)[0],2);a[sp]=false;assert.sameValue(c.call(a)[0],a);
    "#,
    );
}

#[test]
fn concat_species_precedes_spreading_and_keeps_constructor_identity() {
    check(
        r#"
        var c=Array.prototype.concat,log='',a=[7],h={},out={};
        Object.defineProperty(a,'constructor',{get:function(){log+='C';assert.sameValue(this,a);return h;}});
        Object.defineProperty(h,Symbol.species,{get:function(){log+='S';assert.sameValue(this,h);return C;}});
        function C(n){log+='N';assert.sameValue(arguments.length,1);assert.sameValue(n,0);assert.sameValue(new.target,C);return out;}
        Object.defineProperty(a,Symbol.isConcatSpreadable,{get:function(){log+='B';return true;}});
        Object.defineProperty(a,'0',{get:function(){log+='G';return 7;}});
        Object.defineProperty(out,'length',{set:function(n){log+='L';assert.sameValue(n,1);assert.sameValue(this,out);assert.sameValue(out[0],7);}});
        assert.sameValue(c.call(a),out);assert.sameValue(log,'CSNBGL');
        var A=Array,b=[3];b.constructor=undefined;Array=function(){throw 'global';};
        var r=c.call(b);assert.sameValue(Object.getPrototypeOf(r),A.prototype);Array=A;
        function Bound(prefix,n){assert.sameValue(prefix,9);assert.sameValue(n,0);assert.sameValue(new.target,Bound);}
        var d=[4],holder={};holder[Symbol.species]=Bound.bind(null,9);d.constructor=holder;
        r=c.call(d);assert.sameValue(Object.getPrototypeOf(r),Bound.prototype);assert.sameValue(r[0],4);assert.sameValue(r.length,1);
        holder[Symbol.species]=Object;r=c.call(d);assert.sameValue(r.valueOf(),0);assert.sameValue(r[0],4);
        d.constructor=null;assert.throws(TypeError,function(){c.call(d);});
        d.constructor={};d.constructor[Symbol.species]=()=>{};assert.throws(TypeError,function(){c.call(d);});
        var generic={0:5,length:1};generic[Symbol.isConcatSpreadable]=true;
        Object.defineProperty(generic,'constructor',{get:function(){throw 'ignored';}});assert.sameValue(c.call(generic)[0],5);
    "#,
    );
}

#[test]
fn concat_live_holes_per_item_lengths_and_result_aliases() {
    check(
        r#"
        var c=Array.prototype.concat,p={1:6},o=Object.create(p),later={0:2,length:1};
        o.length=3;o[1]=8;o[Symbol.isConcatSpreadable]=true;
        Object.defineProperty(o,'0',{get:function(){assert.sameValue(this,o);delete o[1];o.length=1;later[Symbol.isConcatSpreadable]=true;later[1]=9;later.length=2;return 4;}});
        var r=c.call([],o,later);assert.sameValue(r.length,5);assert.sameValue(r[0],4);assert.sameValue(r[1],6);
        assert.sameValue(r.hasOwnProperty('2'),false);assert.sameValue(r[3],2);assert.sameValue(r[4],9);
        var a=[1],b=[2,3],h={};h[Symbol.species]=function(){return b;};a.constructor=h;
        assert.sameValue(c.call(a,b),b);assert.sameValue(b.join(','),'1,1,1');
        var self=[1,2];h={};h[Symbol.species]=function(){return self;};self.constructor=h;
        assert.sameValue(c.call(self,self),self);assert.sameValue(self.join(','),'1,2,1,2');
        var sparse=[7,,9],out={1:'kept',5:'beyond'};h={};h[Symbol.species]=function(){return out;};sparse.constructor=h;
        assert.sameValue(c.call(sparse),out);assert.sameValue(out[1],'kept');assert.sameValue(out[5],'beyond');assert.sameValue(out.length,3);
    "#,
    );
}

#[test]
fn concat_own_definitions_and_late_strict_length_preserve_partial_effects() {
    check(
        r#"
        var c=Array.prototype.concat,p={},out=Object.create(p),a=[7,8],h={},hits=0;
        Object.defineProperty(p,'0',{set:function(){hits++;throw 'setter';}});
        Object.defineProperty(out,'1',{get:function(){throw 'getter';},configurable:true});
        h[Symbol.species]=function(){return out;};a.constructor=h;c.call(a);
        assert.sameValue(hits,0);verifyProperty(out,'1',{value:8,writable:true,enumerable:true,configurable:true},{restore:true});
        var bad={},source=[4,5],reads=0;Object.defineProperty(bad,'1',{value:9,writable:false,configurable:false});
        Object.defineProperty(source,'1',{get:function(){reads++;return 5;}});h={};h[Symbol.species]=function(){return bad;};source.constructor=h;
        assert.throws(TypeError,function(){c.call(source);});assert.sameValue(reads,1);assert.sameValue(bad[0],4);assert.sameValue(bad.hasOwnProperty('length'),false);
        var array=[0,1,2,3,4,5,6],one=[10];Object.defineProperty(array,'4',{configurable:false});
        h={};h[Symbol.species]=function(){return array;};one.constructor=h;
        assert.throws(TypeError,function(){c.call(one);});assert.sameValue(array.length,5);assert.sameValue(array[0],10);assert.sameValue(array[3],3);
        assert.sameValue(array.hasOwnProperty('5'),false);assert.sameValue(array.hasOwnProperty('6'),false);
        var empty=[],locked=[];Object.defineProperty(locked,'length',{writable:false});h={};h[Symbol.species]=function(){return locked;};empty.constructor=h;
        assert.throws(TypeError,function(){c.call(empty);});
        var token={},q={},s=[8];Object.defineProperty(q,'length',{set:function(n){assert.sameValue(n,1);throw token;}});
        h={};h[Symbol.species]=function(){return q;};s.constructor=h;var seen;try{c.call(s);}catch(e){seen=e;}
        assert.sameValue(seen,token);assert.sameValue(q[0],8);
    "#,
    );
}

#[test]
fn concat_mapped_output_and_ignored_iterator_protocol() {
    check(
        r#"
        var f=Function('a','b','var out=arguments,s=[7,8],h={};h[Symbol.species]=function(){return out;};s.constructor=h;Array.prototype.concat.call(s);return [a,b,out.length];');
        assert.sameValue(f(1,2).join(','),'7,8,2');
        var c=Array.prototype.concat,a=[1],o={0:2,length:1},out={},h={};o[Symbol.isConcatSpreadable]=true;
        function poison(){throw 'iterator';}Object.defineProperty(a,Symbol.iterator,{get:poison});Object.defineProperty(o,Symbol.iterator,{get:poison});Object.defineProperty(o,'return',{get:poison});
        h[Symbol.species]=function(){return out;};a.constructor=h;assert.sameValue(c.call(a,o),out);assert.sameValue(out[0],1);assert.sameValue(out[1],2);assert.sameValue(out.length,2);
        var box=Object('x'),s=[9];h={};h[Symbol.species]=function(){return box;};s.constructor=h;
        assert.throws(TypeError,function(){c.call(s);});assert.sameValue(box[0],'x');
    "#,
    );
}

#[test]
fn concat_overflow_is_after_conversion_and_before_first_index() {
    check(
        r#"
        var c=Array.prototype.concat,log='',o={};
        Object.defineProperty(o,Symbol.isConcatSpreadable,{get:function(){log+='S';return true;}});
        Object.defineProperty(o,'length',{get:function(){log+='L';return {valueOf:function(){log+='V';return Infinity;}};}});
        Object.defineProperty(o,'0',{get:function(){throw 'index';}});
        assert.throws(TypeError,function(){c.call({},o);});assert.sameValue(log,'SLV');
        var reason={},seen,n={length:9007199254740991};n[Symbol.isConcatSpreadable]=true;
        Object.defineProperty(n,'0',{get:function(){throw reason;}});try{c.call([],n);}catch(e){seen=e;}
        assert.sameValue(seen,reason);
        var later=0,t={};Object.defineProperty(t,Symbol.isConcatSpreadable,{get:function(){later++;return true;}});
        seen=undefined;try{c.call([],n,t);}catch(e){seen=e;}assert.sameValue(seen,reason);assert.sameValue(later,0);
    "#,
    );
}

#[test]
fn concat_seeded_safe_counters_and_late_array_length_boundary() {
    let (mut runtime, mut doc) = fresh();
    let out = runtime.object_ordered([]).unwrap();
    let mut next = MAX_LENGTH - 1;
    runtime
        .concat_append(&out, &Value::Number(7.0), &mut next, &mut doc)
        .unwrap();
    assert_eq!(next, MAX_LENGTH);
    assert_eq!(data(&runtime, &out, "9007199254740990"), Value::Number(7.0));
    let allocation = runtime.allocated;
    assert_eq!(
        runtime
            .concat_append(&out, &Value::Number(8.0), &mut next, &mut doc)
            .unwrap_err()
            .name(),
        "TypeError"
    );
    assert_eq!(runtime.allocated, allocation);
    runtime.splice_length(&out, next, &mut doc).unwrap();
    assert_eq!(
        data(&runtime, &out, "length"),
        Value::Number(MAX_LENGTH as f64)
    );

    let array = runtime.array(Vec::new()).unwrap();
    let mut next = u64::from(u32::MAX);
    runtime
        .concat_append(&array, &Value::Number(9.0), &mut next, &mut doc)
        .unwrap();
    assert_eq!(data(&runtime, &array, "4294967295"), Value::Number(9.0));
    assert_eq!(data(&runtime, &array, "length"), Value::Number(0.0));
    assert_eq!(
        runtime
            .splice_length(&array, next, &mut doc)
            .unwrap_err()
            .name(),
        "RangeError"
    );
    assert_eq!(data(&runtime, &array, "4294967295"), Value::Number(9.0));
    clean(&runtime);
}

#[test]
fn concat_symbol_search_is_prepaid_and_primitive_items_skip_it() {
    let (mut runtime, mut doc) = fresh();
    let item = runtime
        .execute(
            "var seen=0;var item={};Object.defineProperty(item,Symbol.isConcatSpreadable,{get:function(){seen++;return false;}});item",
            &mut doc,
        )
        .unwrap();
    // The first bounded well-known-symbol search is charged before the hook.
    runtime.steps = search_work(15, "isConcatSpreadable".len()) - 1;
    assert!(
        runtime
            .concat_spreadable(&item, &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.lookup(1, "seen").unwrap().1, Value::Number(0.0));
    runtime.steps = 0;
    assert!(
        !runtime
            .concat_spreadable(&Value::Number(7.0), &mut doc)
            .unwrap()
    );
    assert_eq!(runtime.steps, 0);
    clean(&runtime);
}

#[test]
fn concat_heap_cut_keeps_getter_effect_and_written_prefix_without_final_setter() {
    let mut witnessed = false;
    // Bounded cutpoints around two reached definitions, not a range-sized test.
    for allowance in (256..=4096).step_by(64) {
        let (mut runtime, mut doc) = fresh();
        let source = runtime
            .execute(
                "var reads=0,finalized=0;var out={set length(n){finalized++;}};var source=[7,8];Object.defineProperty(source,'1',{get:function(){reads++;return 8;},configurable:true});var h={};h[Symbol.species]=function(){return out;};source.constructor=h;source",
                &mut doc,
            )
            .unwrap();
        let out = runtime.lookup(1, "out").unwrap().1;
        runtime.allocated = MAX_HEAP - allowance;
        let result = runtime.array_concat(source, &[], &mut doc);
        if result
            .as_ref()
            .is_err_and(|error| error.is_resource_limit())
            && runtime.lookup(1, "reads").unwrap().1 == Value::Number(1.0)
            && runtime.own_property(&out, &"0".into()).is_some()
            && runtime.own_property(&out, &"1".into()).is_none()
        {
            assert_eq!(data(&runtime, &out, "0"), Value::Number(7.0));
            assert_eq!(
                runtime.lookup(1, "finalized").unwrap().1,
                Value::Number(0.0)
            );
            witnessed = true;
        }
        clean(&runtime);
        if witnessed {
            break;
        }
    }
    assert!(
        witnessed,
        "a reached getter must precede a later definition heap cut"
    );
}

#[test]
fn concat_lazy_host_boundaries_and_prototype_cycles() {
    let (mut runtime, mut doc) = fresh();
    let item = runtime
        .execute(
            "var o={length:0};o[Symbol.isConcatSpreadable]=true;o",
            &mut doc,
        )
        .unwrap();
    let id = runtime.property_object(&item).unwrap();
    runtime.objects[id].prototype = Some(Value::Window);
    let result = runtime.array_concat(item.clone(), &[], &mut doc).unwrap();
    assert_eq!(data(&runtime, &result, "length"), Value::Number(0.0));
    runtime.objects[id].insert("length".into(), Value::Number(1.0));
    assert!(
        runtime
            .array_concat(item, &[], &mut doc)
            .unwrap_err()
            .is_unsupported()
    );
    let cycle = runtime.object_ordered([]).unwrap();
    let id = runtime.property_object(&cycle).unwrap();
    runtime.objects[id].prototype = Some(cycle.clone());
    assert!(
        runtime
            .array_concat(cycle, &[], &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    clean(&runtime);
}

#[test]
fn concat_terminal_recursion_and_huge_holes_skip_author_cleanup() {
    for body in [
        "var a=[],h={};h[Symbol.species]=function(){return a.concat();};a.constructor=h;a.concat();",
        "var o={length:4294967296};o[Symbol.isConcatSpreadable]=true;[].concat(o);",
    ] {
        let (mut runtime, mut doc) = fresh();
        let error = runtime
            .execute(
                &format!("var caught=0,finalized=0;try{{{body}}}catch(e){{caught++;}}finally{{finalized++;}}"),
                &mut doc,
            )
            .unwrap_err();
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
fn concat_repeated_calls_retain_cumulative_work_and_heap() {
    let (mut runtime, mut doc) = fresh();
    let source = runtime.object_ordered([]).unwrap();
    let before = (runtime.steps, runtime.allocated);
    runtime.array_concat(source.clone(), &[], &mut doc).unwrap();
    let cost = (before.0 - runtime.steps, runtime.allocated - before.1);
    assert!(cost.0 > 0 && cost.1 > 0);
    runtime.steps = MAX_STEPS;
    runtime.allocated = MAX_HEAP - cost.1 * 2;
    runtime.array_concat(source.clone(), &[], &mut doc).unwrap();
    runtime.array_concat(source.clone(), &[], &mut doc).unwrap();
    assert_eq!(runtime.allocated, MAX_HEAP);
    assert!(
        runtime
            .array_concat(source, &[], &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert!(runtime.steps < MAX_STEPS - cost.0);
    clean(&runtime);
}
