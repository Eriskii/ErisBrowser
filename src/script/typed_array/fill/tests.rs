use super::*;

// Independent literal oracles from ECMA-262 TypedArray.prototype.fill.
// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.fill
// Whole saved-call admission and the copied-Record body are separate boundaries.

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

fn saved(runtime: &Runtime) -> Value {
    let owner = runtime.typed_arrays.prototype.unwrap();
    match &runtime.objects[owner].values[&JsString::from("fill").into()].value {
        PropertyValue::Data { value, .. } => value.clone(),
        _ => panic!("installed fill method"),
    }
}

fn construct(
    runtime: &mut Runtime,
    doc: &mut Document,
    kind: Kind,
    numbers: &[f64],
) -> (Value, Record) {
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
    (value, record)
}

fn bytes(runtime: &mut Runtime, record: Record) -> Vec<u8> {
    // Test-only raw inspection through the existing public-to-script buffer API.
    // Preserve the observed operation counters; no codec produces these bytes.
    let observed = (runtime.steps, runtime.allocated);
    runtime.steps = MAX_STEPS;
    let live = runtime.typed_array_live(record).unwrap().unwrap();
    let width = record.kind.width();
    let mut result = Vec::new();
    for index in 0..live.length {
        let raw = runtime
            .buffer_view_read(live.buffer, live.offset + index * width, width)
            .unwrap();
        result.extend_from_slice(&raw[..width]);
    }
    assert_eq!(runtime.allocated, observed.1);
    runtime.steps = observed.0;
    result
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
fn all_ten_kinds_fill_literal_values_and_return_the_original_receiver() {
    for kind in Kind::ALL {
        let (mut runtime, mut doc) = fresh();
        let (source, record) = construct(&mut runtime, &mut doc, kind, &[11.0, 22.0, 33.0, 44.0]);
        let function = saved(&runtime);
        let counts = (runtime.objects.len(), runtime.typed_arrays.records.len());
        assert_eq!(
            runtime
                .call(
                    function,
                    vec![
                        Value::Number(257.75),
                        Value::Number(1.0),
                        Value::Number(3.0)
                    ],
                    source.clone(),
                    &mut doc
                )
                .unwrap(),
            source
        );
        let middle = match kind {
            Kind::Int8 | Kind::Uint8 => 1.0,
            Kind::Uint8Clamped => 255.0,
            Kind::Int16 | Kind::Uint16 | Kind::Int32 | Kind::Uint32 => 257.0,
            Kind::Float16 | Kind::Float32 | Kind::Float64 => 257.75,
        };
        for (index, number) in [11.0, middle, middle, 44.0].into_iter().enumerate() {
            assert_eq!(
                runtime.typed_array_read_number(record, index).unwrap(),
                number
            );
        }
        assert_eq!(
            (runtime.objects.len(), runtime.typed_arrays.records.len()),
            counts
        );
        clean(&runtime);
    }
}

#[test]
fn integer_wrap_clamped_ties_and_floating_specials_are_literal() {
    authored(
        r#"(function(){
        var ints=[Int8Array,Uint8Array,Int16Array,Uint16Array,Int32Array,Uint32Array];
        var expected=[-1,255,-1,65535,-1,4294967295];
        for(var i=0;i<ints.length;i++){
            var t=new ints[i](1);t.fill(-1.75);if(t[0]!==expected[i])return false;
            t.fill();if(t[0]!==0)return false;
            t.fill(Infinity);if(t[0]!==0)return false;
        }
        var c=new Uint8ClampedArray(1),values=[0.5,1.5,2.5,3.5,254.5,255.5,NaN,-Infinity,Infinity];
        var results=[0,2,2,4,254,255,0,0,255];
        for(var j=0;j<values.length;j++){c.fill(values[j]);if(c[0]!==results[j])return false;}
        var floats=[Float16Array,Float32Array,Float64Array];
        for(var k=0;k<floats.length;k++){
            var f=new floats[k](2);f.fill(-0);if(1/f[0]!==-Infinity||1/f[1]!==-Infinity)return false;
            f.fill();if(f[0]===f[0]||f[1]===f[1])return false;
            f.fill(Infinity);if(f[0]!==Infinity)return false;
            f.fill(-Infinity);if(f[1]!==-Infinity)return false;
        }
        var h=new Float16Array(1);h.fill(1.00048828125);if(h[0]!==1)return false;
        h.fill(1.00146484375);if(h[0]!==1.001953125)return false;
        var s=new Float32Array(1);s.fill(1.000000059604644775390625);if(s[0]!==1)return false;
        s.fill(1.000000178813934326171875);return s[0]===1.0000002384185791015625;
    })()"#,
    );
}

#[test]
fn range_boundaries_use_captured_length_and_undefined_end() {
    for (start, end, expected) in [
        (None, None, [9, 9, 9, 9]),
        (Some(Value::Undefined), Some(Value::Undefined), [9, 9, 9, 9]),
        (
            Some(Value::Number(1.0)),
            Some(Value::Undefined),
            [1, 9, 9, 9],
        ),
        (Some(Value::Null), Some(Value::Null), [1, 2, 3, 4]),
        (
            Some(Value::Number(-3.0)),
            Some(Value::Number(-1.0)),
            [1, 9, 9, 4],
        ),
        (
            Some(Value::Number(1.9)),
            Some(Value::Number(3.9)),
            [1, 9, 9, 4],
        ),
        (
            Some(Value::Number(-1.9)),
            Some(Value::Number(f64::INFINITY)),
            [1, 2, 3, 9],
        ),
        (
            Some(Value::Number(f64::NAN)),
            Some(Value::Number(2.0)),
            [9, 9, 3, 4],
        ),
        (
            Some(Value::Number(f64::NEG_INFINITY)),
            Some(Value::Number(f64::INFINITY)),
            [9, 9, 9, 9],
        ),
        (Some(Value::Number(f64::INFINITY)), None, [1, 2, 3, 4]),
        (
            Some(Value::Number(0.0)),
            Some(Value::Number(f64::NEG_INFINITY)),
            [1, 2, 3, 4],
        ),
        (
            Some(Value::Number(3.0)),
            Some(Value::Number(1.0)),
            [1, 2, 3, 4],
        ),
    ] {
        let (mut runtime, mut doc) = fresh();
        let (source, record) =
            construct(&mut runtime, &mut doc, Kind::Uint8, &[1.0, 2.0, 3.0, 4.0]);
        let mut args = vec![Value::Number(9.0)];
        if let Some(start) = start {
            args.push(start);
        }
        if let Some(end) = end {
            args.push(end);
        }
        let function = saved(&runtime);
        assert_eq!(
            runtime
                .call(function, args, source.clone(), &mut doc)
                .unwrap(),
            source
        );
        assert_eq!(bytes(&mut runtime, record), expected);
        clean(&runtime);
    }
}

#[test]
fn conversions_are_once_in_order_even_for_empty_ranges_and_preserve_abrupt_identity() {
    authored(
        r#"(function(){
        var fill=Uint8Array.prototype.fill,trace='',marker={};
        function hook(letter,result){var o={};o[Symbol.toPrimitive]=function(h){if(h!=='number'||this!==o)throw marker;trace+=letter;return result;};return o;}
        var t=new Uint8Array([1,2,3,4]);
        if(fill.call(t,hook('v',9),hook('s',1),hook('e',3))!==t||trace!=='vse'||t[0]!==1||t[1]!==9||t[2]!==9||t[3]!==4)return false;
        trace='';var empty=new Uint8Array(0);
        if(fill.call(empty,hook('v',9),hook('s',0),hook('e',0))!==empty||trace!=='vse')return false;
        trace='';fill.call(t,{valueOf:function(){trace+='v';return {};},toString:function(){trace+='t';return '8';}},hook('s',0),hook('e',1));
        if(trace!=='vtse'||t[0]!==8)return false;
        for(var stage=0;stage<3;stage++){
            trace='';var a=[hook('v',9),hook('s',0),hook('e',0)];
            a[stage][Symbol.toPrimitive]=function(){trace+='x';throw marker;};
            try{fill.call(empty,a[0],a[1],a[2]);return false;}catch(e){if(e!==marker)return false;}
            if(trace!==['x','vx','vsx'][stage])return false;
        }
        for(var bad=0;bad<3;bad++){
            var args=[9,0,0];args[bad]=Symbol('bad');
            try{fill.call(empty,args[0],args[1],args[2]);return false;}catch(e){if(!(e instanceof TypeError))return false;}
        }
        return true;
    })()"#,
    );
}

#[test]
fn invalid_initial_receivers_reject_before_all_argument_hooks() {
    authored(
        r#"(function(){
        var fill=Uint8Array.prototype.fill,probe=new Uint8Array(0);
        if(typeof fill!=='function'||fill.call(probe,0)!==probe)return false;
        var detached=new Uint8Array(1);detached.buffer.transfer();
        var b=new ArrayBuffer(4,{maxByteLength:8}),fixed=new Uint8Array(b,0,4),tracking=new Uint8Array(b,2);b.resize(1);
        var fake=Object.create(new Uint8Array(1));
        var receivers=[{},[],null,undefined,new DataView(new ArrayBuffer(1)),fake,detached,fixed,tracking];
        var calls=0,arg={valueOf:function(){calls++;return 0;}};
        for(var i=0;i<receivers.length;i++){
            try{fill.call(receivers[i],arg,arg,arg);return false;}catch(e){if(!(e instanceof TypeError)||calls!==0)return false;}
        }
        function C(){this.ok=1;}if(Reflect.construct(C,[]).ok!==1)return false;
        try{Reflect.construct(fill,[]);return false;}catch(e){return e instanceof TypeError;}
    })()"#,
    );
}

#[test]
fn final_bounds_validation_is_unconditional_and_later_abrupt_callbacks_win() {
    authored(
        r#"(function(){
        var fill=Uint8Array.prototype.fill;
        for(var stage=0;stage<3;stage++){
            var b=new ArrayBuffer(4,{maxByteLength:8}),t=new Uint8Array(b),trace='',moved;
            t[0]=1;t[1]=2;t[2]=3;t[3]=4;
            function hook(letter,n,at){return {valueOf:function(){trace+=letter;if(stage===at)moved=b.transfer();return n;}};}
            try{fill.call(t,hook('v',9,0),hook('s',4,1),hook('e',4,2));return false;}
            catch(e){if(!(e instanceof TypeError)||trace!=='vse')return false;}
            var old=new Uint8Array(moved);if(old[0]!==1||old[1]!==2||old[2]!==3||old[3]!==4)return false;
        }
        var fb=new ArrayBuffer(4,{maxByteLength:8}),f=new Uint8Array(fb,0,4),trace='';
        try{fill.call(f,{valueOf:function(){trace+='v';fb.resize(2);return 9;}},{valueOf:function(){trace+='s';return 4;}},{valueOf:function(){trace+='e';return 4;}});return false;}
        catch(e){if(!(e instanceof TypeError)||trace!=='vse')return false;}
        var rb=new ArrayBuffer(4,{maxByteLength:8}),r=new Uint8Array(rb),marker={};
        try{fill.call(r,{valueOf:function(){rb.transfer();return 9;}},{valueOf:function(){throw marker;}},0);return false;}
        catch(e){if(e!==marker)return false;}
        for(var n=1;n<=2;n++){
            var ob=new ArrayBuffer(4,{maxByteLength:8}),o=new Uint8Array(ob,2),v={valueOf:function(){ob.resize(n);return 9;}};
            try{if(fill.call(o,v,0,0)!==o||n!==2||o.length!==0)return false;}
            catch(e){if(n!==1||!(e instanceof TypeError))return false;}
        }
        return true;
    })()"#,
    );
}

#[test]
fn tracking_shrink_growth_and_partial_element_end_use_literal_bytes() {
    authored(
        r#"(function(){
        for(var s=0;s<3;s++){
            var b=new ArrayBuffer(4,{maxByteLength:8}),t=new Uint8Array(b);t[0]=1;t[1]=2;t[2]=3;t[3]=4;
            var v={valueOf:function(){b.resize(2);return 9;}};
            var starts=[0,-3,-2],rows=[[9,9],[1,9],[1,2]];
            if(t.fill(v,starts[s])!==t||t.length!==2||t[0]!==rows[s][0]||t[1]!==rows[s][1])return false;
        }
        for(var at=0;at<2;at++){
            var gb=new ArrayBuffer(4,{maxByteLength:8}),g=new Uint8Array(gb);g[0]=1;g[1]=2;g[2]=3;g[3]=4;
            function grow(){gb.resize(8);g[6]=77;}
            var value=at===0?{valueOf:function(){grow();return 9;}}:9;
            var end=at===1?{valueOf:function(){grow();return Infinity;}}:Infinity;
            g.fill(value,-2,end);
            var expected=[1,2,9,9,0,0,77,0];
            for(var k=0;k<8;k++)if(g[k]!==expected[k])return false;
        }
        var pb=new ArrayBuffer(8,{maxByteLength:8}),raw=new Uint8Array(pb),p=new Uint16Array(pb,2);
        for(var j=0;j<8;j++)raw[j]=j+1;
        p.fill({valueOf:function(){pb.resize(5);return 4660;}});
        return p.length===1&&raw.length===5&&raw[0]===1&&raw[1]===2&&raw[2]===52&&raw[3]===18&&raw[4]===5;
    })()"#,
    );
}

#[test]
fn fixed_view_can_recover_between_conversions_without_recapturing_indices() {
    authored(
        r#"(function(){
        var b=new ArrayBuffer(4,{maxByteLength:8}),t=new Uint8Array(b,0,4),trace='';
        t[0]=1;t[1]=2;t[2]=3;t[3]=4;
        var value={valueOf:function(){trace+='v';b.resize(2);return 9;}};
        var start={valueOf:function(){trace+='s';b.resize(8);return -2;}};
        if(t.fill(value,start)!==t||trace!=='vs')return false;
        var all=new Uint8Array(b),expected=[1,2,9,9,0,0,0,0];
        for(var i=0;i<8;i++)if(all[i]!==expected[i])return false;
        return t.length===4;
    })()"#,
    );
}

#[test]
fn saved_identity_ignores_poisoned_properties_and_survives_record_growth() {
    authored(
        r#"(function(){
        var fill=Uint8Array.prototype.fill,shared=Object.getPrototypeOf(Uint8Array.prototype);
        var kinds=[Int8Array,Uint8Array,Uint8ClampedArray,Int16Array,Uint16Array,Int32Array,Uint32Array,Float16Array,Float32Array,Float64Array];
        for(var i=0;i<kinds.length;i++)if(kinds[i].prototype.fill!==fill)return false;
        var b=new ArrayBuffer(4),t=new Uint8Array(b),alias=new Uint8Array(b),calls=0;
        var poison={};Object.defineProperty(poison,'0',{get:function(){throw 1;},set:function(){throw 2;}});
        Object.defineProperty(t,'length',{get:function(){throw 3;}});
        Object.defineProperty(t,'constructor',{get:function(){throw 4;}});
        shared.fill=function(){throw 5;};
        var value={valueOf:function(){calls++;for(var j=0;j<12;j++)new Uint8Array(1);Object.setPrototypeOf(t,poison);return 9;}};
        if(fill.call(t,value)!==t||calls!==1)return false;
        return alias[0]===9&&alias[1]===9&&alias[2]===9&&alias[3]===9;
    })()"#,
    );
}

// Literal one/9 encodings; no call to the candidate codec produces the oracle.
fn encoded(kind: Kind, nine: bool) -> &'static [u8] {
    match (kind, nine) {
        (Kind::Int8 | Kind::Uint8 | Kind::Uint8Clamped, false) => &[1],
        (Kind::Int8 | Kind::Uint8 | Kind::Uint8Clamped, true) => &[9],
        (Kind::Int16 | Kind::Uint16, false) => &[1, 0],
        (Kind::Int16 | Kind::Uint16, true) => &[9, 0],
        (Kind::Int32 | Kind::Uint32, false) => &[1, 0, 0, 0],
        (Kind::Int32 | Kind::Uint32, true) => &[9, 0, 0, 0],
        (Kind::Float16, false) => &[0, 60],
        (Kind::Float16, true) => &[128, 72],
        (Kind::Float32, false) => &[0, 0, 128, 63],
        (Kind::Float32, true) => &[0, 0, 16, 65],
        (Kind::Float64, false) => &[0, 0, 0, 0, 0, 0, 240, 63],
        (Kind::Float64, true) => &[0, 0, 0, 0, 0, 0, 34, 64],
    }
}

fn element_work(kind: Kind) -> usize {
    // Loop8, live13, offset4, codec48+2w, buffer tick1, complete byte copy w.
    match kind {
        Kind::Int8 | Kind::Uint8 => 77,
        Kind::Uint8Clamped => 93,
        Kind::Int16 | Kind::Uint16 | Kind::Float16 => 80,
        Kind::Int32 | Kind::Uint32 | Kind::Float32 => 86,
        Kind::Float64 => 98,
    }
}

fn prefix_bytes(kind: Kind, complete: usize) -> Vec<u8> {
    let mut result = Vec::new();
    for index in 0..3 {
        result.extend_from_slice(encoded(kind, index < complete));
    }
    result
}

#[test]
fn literal_body_fees_prepay_complete_scalars_and_preserve_only_finished_prefixes() {
    for kind in Kind::ALL {
        for defined_end in [false, true] {
            let prelude = if defined_end { 62 } else { 58 };
            let per = element_work(kind);
            for allowance in [
                0,
                prelude - 1,
                prelude,
                prelude + per - 1,
                prelude + per,
                prelude + 2 * per - 1,
                prelude + 2 * per,
                prelude + 3 * per - 1,
                prelude + 3 * per,
            ] {
                let (mut runtime, mut doc) = fresh();
                let (source, record) = construct(&mut runtime, &mut doc, kind, &[1.0, 1.0, 1.0]);
                assert_eq!(bytes(&mut runtime, record), prefix_bytes(kind, 0));
                let mut args = vec![Value::Number(9.0)];
                if defined_end {
                    args.extend([Value::Number(0.0), Value::Number(3.0)]);
                }
                let counts = (runtime.objects.len(), runtime.typed_arrays.records.len());
                runtime.steps = allowance;
                runtime.allocated = MAX_HEAP; // The body requires no heap.
                let result = runtime.typed_array_fill(source.clone(), record, &args, &mut doc);
                if allowance == prelude + 3 * per {
                    assert_eq!(result.unwrap(), source);
                } else {
                    assert!(result.unwrap_err().is_resource_limit());
                }
                let completed = allowance.saturating_sub(prelude) / per;
                assert_eq!(bytes(&mut runtime, record), prefix_bytes(kind, completed));
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
                assert_eq!(
                    (runtime.objects.len(), runtime.typed_arrays.records.len()),
                    counts
                );
                clean(&runtime);
            }
            // Equal bytes do not permit skipping reached scalar work.
            let (mut runtime, mut doc) = fresh();
            let (source, record) = construct(&mut runtime, &mut doc, kind, &[1.0, 1.0, 1.0]);
            let mut args = vec![Value::Number(1.0)];
            if defined_end {
                args.extend([Value::Number(0.0), Value::Number(3.0)]);
            }
            runtime.steps = prelude + 3 * per;
            runtime.allocated = MAX_HEAP;
            assert_eq!(
                runtime
                    .typed_array_fill(source.clone(), record, &args, &mut doc)
                    .unwrap(),
                source
            );
            assert_eq!(bytes(&mut runtime, record), prefix_bytes(kind, 0));
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            clean(&runtime);
        }
    }
    // A valid empty range still pays the whole conversion/final-check prelude.
    for defined_end in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let (source, record) = construct(&mut runtime, &mut doc, Kind::Uint8, &[]);
        let args = if defined_end {
            vec![Value::Number(9.0), Value::Number(0.0), Value::Number(0.0)]
        } else {
            vec![Value::Number(9.0)]
        };
        runtime.steps = if defined_end { 62 } else { 58 };
        runtime.allocated = MAX_HEAP;
        assert_eq!(
            runtime
                .typed_array_fill(source.clone(), record, &args, &mut doc)
                .unwrap(),
            source
        );
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        clean(&runtime);
    }
}

#[test]
fn saved_call_exact_and_one_short_admission_has_only_the_existing_name_allocation() {
    for kind in Kind::ALL {
        let (mut measured, mut doc) = fresh();
        let (source, record) = construct(&mut measured, &mut doc, kind, &[1.0, 1.0, 1.0]);
        let function = saved(&measured);
        let before = (measured.steps, measured.allocated);
        assert_eq!(
            measured
                .call(function, vec![Value::Number(9.0)], source.clone(), &mut doc)
                .unwrap(),
            source
        );
        assert_eq!(bytes(&mut measured, record), prefix_bytes(kind, 3));
        let work = before.0 - measured.steps;
        let heap = measured.allocated - before.1;
        assert!(work > 58 + 3 * element_work(kind));
        assert_eq!(heap, 47); // 32 + fifteen bytes in TypedArray.fill.
        for (steps, room, succeeds) in [
            (work, heap, true),
            (work - 1, heap, false),
            (0, heap, false),
            (work, heap - 1, false),
            (work, 0, false),
        ] {
            let (mut runtime, mut doc) = fresh();
            let (source, record) = construct(&mut runtime, &mut doc, kind, &[1.0, 1.0, 1.0]);
            let function = saved(&runtime);
            let bag = runtime.property_object(&function).unwrap();
            let metadata = format!("{:?}", runtime.objects[bag].values);
            let counts = (runtime.objects.len(), runtime.typed_arrays.records.len());
            runtime.steps = steps;
            runtime.allocated = MAX_HEAP - room;
            let result = runtime.call(function, vec![Value::Number(9.0)], source.clone(), &mut doc);
            if succeeds {
                assert_eq!(result.unwrap(), source);
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            } else {
                assert!(result.unwrap_err().is_resource_limit());
            }
            let complete = if succeeds {
                3
            } else if steps == work - 1 {
                2
            } else {
                0
            };
            assert_eq!(bytes(&mut runtime, record), prefix_bytes(kind, complete));
            assert_eq!(
                (runtime.objects.len(), runtime.typed_arrays.records.len()),
                counts
            );
            assert_eq!(format!("{:?}", runtime.objects[bag].values), metadata);
            clean(&runtime);
        }
    }
}

fn callback_fixture() -> (Runtime, Document, Value, Record, Value, Vec<Value>) {
    let (mut runtime, mut doc) = fresh();
    runtime.execute("var effects=0;var source=new Uint16Array([1,2,3]);var value={valueOf:function(){effects++;source[2]=77;return 9;}};", &mut doc).unwrap();
    let source = runtime.lookup(1, "source").unwrap().1;
    let value = runtime.lookup(1, "value").unwrap().1;
    let record = runtime.typed_array_record(&source).unwrap().unwrap();
    let function = saved(&runtime);
    (
        runtime,
        doc,
        source,
        record,
        function,
        vec![value, Value::Number(0.0), Value::Number(2.0)],
    )
}

#[test]
fn callback_effects_survive_last_scalar_work_refusal_and_terminal_limits_clean_guards() {
    let (mut measured, mut doc, source, record, function, args) = callback_fixture();
    let before = (measured.steps, measured.allocated);
    assert_eq!(
        measured
            .call(function, args, source.clone(), &mut doc)
            .unwrap(),
        source
    );
    assert_eq!(bytes(&mut measured, record), [9, 0, 9, 0, 77, 0]);
    let (work, heap) = (before.0 - measured.steps, measured.allocated - before.1);
    let (mut runtime, mut doc, source, record, function, args) = callback_fixture();
    runtime.steps = work - 1;
    runtime.allocated = MAX_HEAP - heap;
    assert!(
        runtime
            .call(function, args, source, &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.lookup(1, "effects").unwrap().1, Value::Number(1.0));
    assert_eq!(bytes(&mut runtime, record), [9, 0, 2, 0, 77, 0]);
    clean(&runtime);
    for allocation in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let code = if allocation {
            "var effects=0,caught=0,finished=0;var source=new Uint8Array([1,2,3]);var value={valueOf:function(){effects++;source[2]=77;try{while(true){new ArrayBuffer(128);}}catch(e){caught++;}finally{finished++;}return 9;}};"
        } else {
            "var effects=0,caught=0,finished=0;var source=new Uint8Array([1,2,3]);var value={valueOf:function(){effects++;source[2]=77;try{while(true){}}catch(e){caught++;}finally{finished++;}return 9;}};"
        };
        runtime.execute(code, &mut doc).unwrap();
        let source = runtime.lookup(1, "source").unwrap().1;
        let value = runtime.lookup(1, "value").unwrap().1;
        let record = runtime.typed_array_record(&source).unwrap().unwrap();
        let function = saved(&runtime);
        runtime.steps = if allocation { MAX_STEPS } else { 10_000 };
        if allocation {
            runtime.allocated = MAX_HEAP - 16_384;
        }
        assert!(
            runtime
                .call(function, vec![value], source, &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.lookup(1, "effects").unwrap().1, Value::Number(1.0));
        assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Number(0.0));
        assert_eq!(runtime.lookup(1, "finished").unwrap().1, Value::Number(0.0));
        assert_eq!(bytes(&mut runtime, record), [1, 2, 77]);
        if allocation {
            assert!(runtime.allocated > MAX_HEAP && runtime.steps > 0);
        } else {
            assert_eq!(runtime.steps, 0);
        }
        clean(&runtime);
    }
}
