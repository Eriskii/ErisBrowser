use super::*;

// Independent semantic oracles: ECMA-262 23.2.3.1, .16, .17 and .20.
// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.at
// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.includes
// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.indexof
// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.lastindexof
// These tests use the public saved-call boundary, not the new search helpers.

const METHODS: [&str; 4] = ["at", "includes", "indexOf", "lastIndexOf"];

fn fresh() -> (Runtime, Document) {
    (Runtime::try_new().unwrap(), Document::parse("<p>kept</p>"))
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

fn saved(runtime: &Runtime, name: &str) -> Value {
    let owner = runtime.typed_arrays.prototype.unwrap();
    match &runtime.objects[owner].values[&JsString::from(name).into()].value {
        PropertyValue::Data { value, .. } => value.clone(),
        _ => panic!("installed method"),
    }
}

fn construct(runtime: &mut Runtime, doc: &mut Document, kind: Kind, numbers: &[f64]) -> Value {
    let target = runtime.typed_arrays.constructors[kind.index()]
        .clone()
        .unwrap();
    let value = runtime
        .typed_array_constructor(kind, &[Value::Number(numbers.len() as f64)], target, doc)
        .unwrap();
    let record = runtime.typed_array_record(&value).unwrap().unwrap();
    for (index, number) in numbers.iter().enumerate() {
        runtime
            .typed_array_write_number(record, index, *number)
            .unwrap();
    }
    value
}

fn authored(source: &str) {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        assert_eq!(result.unwrap(), Value::Bool(true), "strict={strict}");
        clean(&runtime);
    }
}

#[test]
fn all_ten_number_kinds_use_real_elements_and_first_or_last_match() {
    for kind in Kind::ALL {
        let (mut runtime, mut doc) = fresh();
        let source = construct(&mut runtime, &mut doc, kind, &[1.0, 2.0, 1.0, 3.0]);
        let original = (runtime.objects.len(), runtime.typed_arrays.records.len());
        for (name, arguments, expected) in [
            ("at", vec![Value::Number(-1.0)], Value::Number(3.0)),
            ("at", vec![Value::Number(4.0)], Value::Undefined),
            ("includes", vec![Value::Number(1.0)], Value::Bool(true)),
            ("includes", vec![Value::Number(9.0)], Value::Bool(false)),
            ("indexOf", vec![Value::Number(1.0)], Value::Number(0.0)),
            ("lastIndexOf", vec![Value::Number(1.0)], Value::Number(2.0)),
        ] {
            let function = saved(&runtime, name);
            assert_eq!(
                runtime
                    .call(function, arguments, source.clone(), &mut doc)
                    .unwrap(),
                expected
            );
            assert_eq!(
                (runtime.objects.len(), runtime.typed_arrays.records.len()),
                original
            );
            clean(&runtime);
        }
    }
}

#[test]
fn floating_nan_and_signed_zero_use_the_distinct_equality_relations() {
    authored(
        r#"(function(){
        var kinds=[Float16Array,Float32Array,Float64Array];
        for(var i=0;i<kinds.length;i++){
            var C=kinds[i],t=new C([-0,NaN,0,NaN,Infinity,-Infinity]);
            if(1/t.at(0)!==-Infinity||1/t.at(2)!==Infinity)return false;
            if(t.at(1)===t.at(1)||!t.includes(NaN)||t.indexOf(NaN)!==-1||t.lastIndexOf(NaN)!==-1)return false;
            if(!t.includes(-0)||!t.includes(0)||t.indexOf(-0)!==0||t.lastIndexOf(0)!==2)return false;
            if(t.indexOf(Infinity)!==4||t.lastIndexOf(-Infinity)!==5)return false;
        }
        return true;
    })()"#,
    );
}

#[test]
fn at_relative_indices_and_search_start_boundaries_are_literal() {
    let (mut runtime, mut doc) = fresh();
    let source = construct(&mut runtime, &mut doc, Kind::Uint8, &[1.0, 2.0, 1.0, 3.0]);
    for (argument, expected) in [
        (None, Value::Number(1.0)),
        (Some(Value::Undefined), Value::Number(1.0)),
        (Some(Value::Number(f64::NAN)), Value::Number(1.0)),
        (Some(Value::Number(-0.0)), Value::Number(1.0)),
        (Some(Value::Number(1.9)), Value::Number(2.0)),
        (Some(Value::Number(-1.9)), Value::Number(3.0)),
        (Some(Value::Number(-4.0)), Value::Number(1.0)),
        (Some(Value::Number(-5.0)), Value::Undefined),
        (Some(Value::Number(4.0)), Value::Undefined),
        (Some(Value::Number(f64::INFINITY)), Value::Undefined),
        (Some(Value::Number(f64::NEG_INFINITY)), Value::Undefined),
        (Some(Value::String(JsString::from("2"))), Value::Number(1.0)),
        (Some(Value::Null), Value::Number(1.0)),
        (Some(Value::Bool(true)), Value::Number(2.0)),
    ] {
        let function = saved(&runtime, "at");
        assert_eq!(
            runtime
                .call(
                    function,
                    argument.into_iter().collect(),
                    source.clone(),
                    &mut doc
                )
                .unwrap(),
            expected
        );
    }
    // Literal rows distinguish forward clamping from lastIndexOf's lower
    // endpoint. The authored body below separately passes explicit undefined.
    for (start, includes, first, last) in [
        (None, true, 0.0, 2.0),
        (Some(f64::NAN), true, 0.0, 0.0),
        (Some(-0.0), true, 0.0, 0.0),
        (Some(1.9), true, 2.0, 0.0),
        (Some(-1.9), false, -1.0, 2.0),
        (Some(-2.0), true, 2.0, 2.0),
        (Some(-4.0), true, 0.0, 0.0),
        (Some(-5.0), true, 0.0, -1.0),
        (Some(4.0), false, -1.0, 2.0),
        (Some(f64::INFINITY), false, -1.0, 2.0),
        (Some(f64::NEG_INFINITY), true, 0.0, -1.0),
    ] {
        for (name, expected) in [
            ("includes", Value::Bool(includes)),
            ("indexOf", Value::Number(first)),
            ("lastIndexOf", Value::Number(last)),
        ] {
            let mut arguments = vec![Value::Number(1.0)];
            if let Some(start) = start {
                arguments.push(Value::Number(start));
            }
            let function = saved(&runtime, name);
            assert_eq!(
                runtime
                    .call(function, arguments, source.clone(), &mut doc)
                    .unwrap(),
                expected
            );
        }
    }
    authored(
        r#"(function(){var t=new Uint8Array([1,2,1]);
        return t.lastIndexOf(1)===2&&t.lastIndexOf(1,undefined)===0&&
            t.lastIndexOf(1,null)===0&&t.lastIndexOf(1,'2')===2&&
            t.indexOf()===-1&&t.lastIndexOf()===-1&&!t.includes();
    })()"#,
    );
    clean(&runtime);
}

#[test]
fn index_conversion_uses_number_hint_once_and_preserves_abrupt_identity() {
    for name in METHODS {
        authored(&r#"(function(){
            var method=Uint8Array.prototype.METHOD,t=new Uint8Array([5,6]),trace='',marker={};
            function invoke(x){return 'METHOD'==='at'?method.call(t,x):method.call(t,5,x);}
            var index={};Object.defineProperty(index,Symbol.toPrimitive,{get:function(){trace+='G';return function(h){
                if(h!=='number')throw marker;trace+='N';return 0;
            };}});
            invoke(index);if(trace!=='GN')return false;
            trace='';invoke({valueOf:function(){trace+='V';return {};},toString:function(){trace+='S';return '0';}});
            if(trace!=='VS')return false;
            var caught=false;try{invoke({valueOf:function(){trace+='X';throw marker;}});}catch(e){caught=e===marker;}
            if(!caught||trace!=='VSX')return false;
            caught=false;try{invoke(Symbol('index'));}catch(e){caught=e instanceof TypeError;}
            if(!caught)return false;
            trace='';var bad={};bad[Symbol.toPrimitive]=function(){trace+='B';return {};};
            bad.valueOf=function(){trace+='V';return 0;};
            caught=false;try{invoke(bad);}catch(e){caught=e instanceof TypeError;}
            return caught&&trace==='B';
        })()"#.replace("METHOD", name));
    }
}

#[test]
fn initial_brand_detach_and_oob_validation_precede_index_coercion() {
    for name in METHODS {
        authored(&r#"(function(){
            var method=Uint8Array.prototype.METHOD,calls=0;
            var index={valueOf:function(){calls++;throw new Error('index reached');}};
            function rejects(x){try{
                if('METHOD'==='at')method.call(x,index);else method.call(x,0,index);
            }catch(e){return e instanceof TypeError;}return false;}
            var b=new ArrayBuffer(4,{maxByteLength:8}),fixed=new Uint8Array(b,2,2),tracking=new Uint8Array(b,2);
            b.resize(1);var detached=new Uint8Array(0);detached.buffer.transfer();
            var ordinary=Object.create(Uint8Array.prototype);
            Object.defineProperty(ordinary,'length',{get:function(){calls++;throw new Error('length reached');}});
            return rejects(null)&&rejects({})&&rejects([])&&rejects(new DataView(new ArrayBuffer(1)))&&
                rejects(ordinary)&&rejects(fixed)&&rejects(tracking)&&rejects(detached)&&calls===0;
        })()"#.replace("METHOD", name));
    }
}

#[test]
fn valid_empty_arrays_skip_search_conversion_but_at_still_converts() {
    authored(
        r#"(function(){
        var t=new Uint8Array(0),calls=0,marker={};
        var index={valueOf:function(){calls++;throw marker;}};
        if(t.includes(undefined,index)||t.indexOf(undefined,index)!==-1||t.lastIndexOf(undefined,index)!==-1||calls!==0)return false;
        var caught=false;try{t.at(index);}catch(e){caught=e===marker;}
        return caught&&calls===1;
    })()"#,
    );
}

#[test]
fn vanished_elements_are_undefined_for_includes_and_absent_for_index_searches() {
    for name in METHODS {
        authored(&r#"(function(){
            var method=Uint8Array.prototype.METHOD;
            for(var mode=0;mode<3;mode++){
                var b=new ArrayBuffer(4,{maxByteLength:8});
                var t=mode===1?new Uint8Array(b,0,4):new Uint8Array(b);t[0]=1;t[3]=4;
                var calls=0,index={valueOf:function(){calls++;if(mode===2)b.transfer();else b.resize(1);return 'METHOD'==='at'||'METHOD'==='lastIndexOf'?3:0;}};
                var value='METHOD'==='at'?method.call(t,index):method.call(t,undefined,index);
                var expected='METHOD'==='at'?undefined:('METHOD'==='includes'?true:-1);
                if(value!==expected||calls!==1)return false;
            }
            return true;
        })()"#.replace("METHOD", name));
    }
}

#[test]
fn captured_length_ignores_growth_but_reads_current_surviving_elements() {
    for name in METHODS {
        authored(&r#"(function(){
            var method=Uint8Array.prototype.METHOD,b=new ArrayBuffer(2,{maxByteLength:8}),t=new Uint8Array(b);
            t[0]=3;t[1]=4;
            var index={valueOf:function(){b.resize(8);t[1]=7;t[7]=99;return 'METHOD'==='at'?-1:('METHOD'==='lastIndexOf'?Infinity:0);}};
            var result='METHOD'==='at'?method.call(t,index):method.call(t,99,index);
            if(result!==('METHOD'==='at'?7:('METHOD'==='includes'?false:-1))||t.length!==8)return false;
            b.resize(4);t[0]=8;t[1]=9;
            index={valueOf:function(){b.resize(2);return 'METHOD'==='at'?1:('METHOD'==='lastIndexOf'?3:0);}};
            result='METHOD'==='at'?method.call(t,index):method.call(t,9,index);
            return result===('METHOD'==='at'?9:('METHOD'==='includes'?true:1));
        })()"#.replace("METHOD", name));
    }
}

#[test]
fn search_element_is_never_coerced_or_used_as_a_predicate() {
    authored(
        r#"(function(){
        var t=new Uint8Array([0,1]),calls=0,needle={valueOf:function(){calls++;throw new Error('value');},toString:function(){calls++;throw new Error('string');}};
        needle[Symbol.toPrimitive]=function(){calls++;throw new Error('primitive');};
        var needles=[needle,'1',true,null,undefined,Symbol('needle'),function(){calls++;}];
        for(var i=0;i<needles.length;i++){
            if(t.includes(needles[i])||t.indexOf(needles[i])!==-1||t.lastIndexOf(needles[i])!==-1)return false;
        }
        return calls===0;
    })()"#,
    );
}

#[test]
fn saved_identity_ignores_poisoned_length_constructors_and_numeric_prototypes() {
    authored(
        r#"(function(){
        var shared=Object.getPrototypeOf(Uint8Array.prototype),t=new Uint8Array([2,4]),calls=0;
        var at=t.at,includes=t.includes,indexOf=t.indexOf,lastIndexOf=t.lastIndexOf;
        if(at!==Int16Array.prototype.at||includes!==Float64Array.prototype.includes||indexOf!==Uint8ClampedArray.prototype.indexOf||lastIndexOf!==Float16Array.prototype.lastIndexOf)return false;
        function poison(){calls++;throw new Error('unexpected property');}
        Object.defineProperty(t,'length',{get:poison});Object.defineProperty(t,'constructor',{get:poison});
        var proto={};Object.defineProperty(proto,'0',{get:poison});Object.defineProperty(proto,'1',{get:poison});
        Object.setPrototypeOf(t,proto);
        shared.at=0;shared.includes=0;shared.indexOf=0;shared.lastIndexOf=0;
        if(at.call(t,1)!==4||!includes.call(t,2)||indexOf.call(t,4)!==1||lastIndexOf.call(t,2)!==0)return false;
        var b=new ArrayBuffer(2,{maxByteLength:4}),view=new Uint8Array(b);
        Object.setPrototypeOf(view,proto);
        var cut={valueOf:function(){b.resize(0);return 0;}};
        return includes.call(view,undefined,cut)&&indexOf.call(view,undefined,0)===-1&&
            lastIndexOf.call(view,undefined,1)===-1&&at.call(view,0)===undefined&&calls===0;
    })()"#,
    );
}

#[test]
fn index_callbacks_grow_record_storage_without_stale_source_state() {
    for name in METHODS {
        let (mut runtime, mut doc) = fresh();
        let source = r#"var source=new Uint8Array([3,5,7]);var effects=0;
            var index={valueOf:function(){effects++;for(var i=0;i<12;i++)new Float64Array(0);source[1]=9;return 1;}};"#;
        runtime.execute(source, &mut doc).unwrap();
        let target = runtime.lookup(1, "source").unwrap().1;
        let index = runtime.lookup(1, "index").unwrap().1;
        let original = (
            runtime.typed_arrays.records.len(),
            runtime.typed_arrays.records.capacity(),
        );
        let arguments = if name == "at" {
            vec![index]
        } else {
            vec![Value::Number(9.0), index]
        };
        let function = saved(&runtime, name);
        let result = runtime.call(function, arguments, target, &mut doc).unwrap();
        let expected = match name {
            "at" => Value::Number(9.0),
            "includes" => Value::Bool(true),
            _ => Value::Number(1.0),
        };
        assert_eq!(result, expected);
        assert_eq!(runtime.lookup(1, "effects").unwrap().1, Value::Number(1.0));
        assert_eq!(runtime.typed_arrays.records.len(), original.0 + 12);
        assert!(runtime.typed_arrays.records.capacity() > original.1);
        clean(&runtime);
    }
}

fn endpoint_fixture(
    name: &str,
    kind: Kind,
) -> (Runtime, Document, Value, Value, Vec<Value>, Value) {
    let (mut runtime, mut doc) = fresh();
    let source = construct(&mut runtime, &mut doc, kind, &[1.0, 2.0, 3.0]);
    let function = saved(&runtime, name);
    let (arguments, expected) = match name {
        "at" => (vec![Value::Number(2.0)], Value::Number(3.0)),
        "includes" => (vec![Value::Number(7.0)], Value::Bool(false)),
        _ => (vec![Value::Number(7.0)], Value::Number(-1.0)),
    };
    (runtime, doc, function, source, arguments, expected)
}

#[test]
fn full_saved_call_exact_and_one_short_work_heap_boundaries_cover_every_codec() {
    for kind in Kind::ALL {
        for name in METHODS {
            let (mut measured, mut doc, function, source, arguments, expected) =
                endpoint_fixture(name, kind);
            let before = (measured.steps, measured.allocated);
            assert_eq!(
                measured
                    .call(function, arguments, source, &mut doc)
                    .unwrap(),
                expected
            );
            let (work, heap) = (before.0 - measured.steps, measured.allocated - before.1);
            // Search creates no result string or per-element scratch. The
            // existing saved-native receiver binding still owns its name copy.
            assert!(work > 0);
            assert_eq!(heap, 32 + "TypedArray.".len() + name.len());
            for (steps, room, success) in [
                (work, heap, true),
                (work - 1, heap, false),
                (work, heap - 1, false),
                (0, heap, false),
                (work, 0, false),
            ] {
                let (mut runtime, mut doc, function, source, arguments, expected) =
                    endpoint_fixture(name, kind);
                let objects = runtime.objects.len();
                let records = runtime.typed_arrays.records.len();
                let bag = runtime.property_object(&function).unwrap();
                let metadata = format!("{:?}", runtime.objects[bag].values);
                runtime.steps = steps;
                runtime.allocated = MAX_HEAP - room;
                let result = runtime.call(function, arguments, source, &mut doc);
                if success {
                    assert_eq!(result.unwrap(), expected);
                    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
                } else {
                    assert!(result.unwrap_err().is_resource_limit());
                }
                assert_eq!(
                    (runtime.objects.len(), runtime.typed_arrays.records.len()),
                    (objects, records)
                );
                assert_eq!(format!("{:?}", runtime.objects[bag].values), metadata);
                clean(&runtime);
            }
        }
    }
}

fn callback_fixture(name: &str) -> (Runtime, Document, Value, Value, Vec<Value>) {
    let (mut runtime, mut doc) = fresh();
    runtime.execute("var effects=0;var source=new Uint8Array([1,2]);var index={valueOf:function(){effects++;source[0]=9;return 0;}};", &mut doc).unwrap();
    let function = saved(&runtime, name);
    let source = runtime.lookup(1, "source").unwrap().1;
    let index = runtime.lookup(1, "index").unwrap().1;
    let arguments = if name == "at" {
        vec![index]
    } else {
        vec![Value::Number(9.0), index]
    };
    (runtime, doc, function, source, arguments)
}

#[test]
fn one_short_native_work_retains_completed_conversion_effects_and_cleans_guards() {
    for name in METHODS {
        let (mut measured, mut doc, function, source, arguments) = callback_fixture(name);
        let before = (measured.steps, measured.allocated);
        let expected = match name {
            "at" => Value::Number(9.0),
            "includes" => Value::Bool(true),
            _ => Value::Number(0.0),
        };
        assert_eq!(
            measured
                .call(function, arguments, source, &mut doc)
                .unwrap(),
            expected
        );
        let (work, heap) = (before.0 - measured.steps, measured.allocated - before.1);
        let (mut runtime, mut doc, function, source, arguments) = callback_fixture(name);
        let record = runtime.typed_array_record(&source).unwrap().unwrap();
        runtime.steps = work - 1;
        runtime.allocated = MAX_HEAP - heap;
        assert!(
            runtime
                .call(function, arguments, source, &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.lookup(1, "effects").unwrap().1, Value::Number(1.0));
        clean(&runtime);
        runtime.steps = MAX_STEPS;
        assert_eq!(runtime.typed_array_read_number(record, 0).unwrap(), 9.0);
    }
}

#[test]
fn callback_resource_is_terminal_and_vanished_searches_still_pay_loop_work() {
    for name in METHODS {
        let (mut runtime, mut doc) = fresh();
        runtime.execute("var effects=0,caught=0,finished=0;var source=new Uint8Array([1]);var index={valueOf:function(){effects++;try{while(true){}}catch(e){caught++;}finally{finished++;}return 0;}};", &mut doc).unwrap();
        let source = runtime.lookup(1, "source").unwrap().1;
        let index = runtime.lookup(1, "index").unwrap().1;
        let function = saved(&runtime, name);
        let arguments = if name == "at" {
            vec![index]
        } else {
            vec![Value::Number(1.0), index]
        };
        runtime.steps = 10_000;
        assert!(
            runtime
                .call(function, arguments, source, &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.lookup(1, "effects").unwrap().1, Value::Number(1.0));
        assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Number(0.0));
        assert_eq!(runtime.lookup(1, "finished").unwrap().1, Value::Number(0.0));
        clean(&runtime);
    }
    for name in ["includes", "indexOf", "lastIndexOf"] {
        let (mut runtime, mut doc) = fresh();
        runtime.execute("var effects=0;var source=new Uint8Array(10000);var buffer=source.buffer;var index={valueOf:function(){effects++;buffer.transfer();return 0;}};", &mut doc).unwrap();
        // LastIndexOf starts at the captured end, even though all its indices
        // disappear during conversion. Search for Number 7 never matches them.
        if name == "lastIndexOf" {
            runtime
                .execute(
                    "index.valueOf=function(){effects++;buffer.transfer();return Infinity;};",
                    &mut doc,
                )
                .unwrap();
        }
        let source = runtime.lookup(1, "source").unwrap().1;
        let record = runtime.typed_array_record(&source).unwrap().unwrap();
        let index = runtime.lookup(1, "index").unwrap().1;
        let function = saved(&runtime, name);
        runtime.steps = MAX_STEPS;
        assert!(
            runtime
                .call(function, vec![Value::Number(7.0), index], source, &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.lookup(1, "effects").unwrap().1, Value::Number(1.0));
        clean(&runtime);
        runtime.steps = MAX_STEPS;
        assert!(runtime.typed_array_live(record).unwrap().is_none());
    }
}
