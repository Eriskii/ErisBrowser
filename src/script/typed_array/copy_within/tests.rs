use super::*;

// Independently authored before reading the candidate copyWithin implementation.
// ECMA-262 23.2.3.6: https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.copywithin
// Root registers this as typed_array/copy_within/tests.rs and owns execution.
// Quota-cut tests are deliberately a separate root-authored task.

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
    match &runtime.objects[owner].values[&JsString::from("copyWithin").into()].value {
        PropertyValue::Data { value, .. } => value.clone(),
        _ => panic!("installed copyWithin method"),
    }
}

fn construct(
    runtime: &mut Runtime,
    doc: &mut Document,
    kind: Kind,
    values: &[f64],
) -> (Value, Record) {
    let target = runtime.typed_arrays.constructors[kind.index()]
        .clone()
        .unwrap();
    let value = runtime
        .typed_array_constructor(kind, &[Value::Number(values.len() as f64)], target, doc)
        .unwrap();
    let record = runtime.typed_array_record(&value).unwrap().unwrap();
    for (index, number) in values.iter().enumerate() {
        runtime
            .typed_array_write_number(record, index, *number)
            .unwrap();
    }
    (value, record)
}

fn bytes(runtime: &mut Runtime, record: Record) -> Vec<u8> {
    // Existing admitted raw-buffer reads only; no sibling-private state and no
    // codec/candidate operation generates the expected bit patterns.
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

fn raw_fixture(
    kind: Kind,
    payload: &[u8],
    offset: usize,
    length: usize,
) -> (Runtime, Document, Value, Record) {
    let (mut runtime, mut doc) = fresh();
    runtime.execute(&format!(
        "var buffer=new ArrayBuffer({});var raw=new Uint8Array(buffer);var source=new {}(buffer,{offset},{length});",
        payload.len(), kind.name()
    ), &mut doc).unwrap();
    let source = runtime.lookup(1, "source").unwrap().1;
    let raw = runtime.lookup(1, "raw").unwrap().1;
    let raw_record = runtime.typed_array_record(&raw).unwrap().unwrap();
    let live = runtime.typed_array_live(raw_record).unwrap().unwrap();
    for (index, chunk) in payload.chunks(8).enumerate() {
        runtime
            .buffer_view_write(live.buffer, index * 8, chunk)
            .unwrap();
    }
    assert_eq!(bytes(&mut runtime, raw_record), payload);
    (runtime, doc, source, raw_record)
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
fn all_ten_kinds_copy_overlaps_in_both_directions_and_return_the_receiver() {
    for kind in Kind::ALL {
        for (args, expected) in [
            ([1.0, 0.0, 4.0], [1.0, 1.0, 2.0, 3.0, 4.0]),
            ([0.0, 1.0, 5.0], [2.0, 3.0, 4.0, 5.0, 5.0]),
            ([3.0, 0.0, 2.0], [1.0, 2.0, 3.0, 1.0, 2.0]),
            ([1.0, 1.0, 4.0], [1.0, 2.0, 3.0, 4.0, 5.0]),
        ] {
            let (mut runtime, mut doc) = fresh();
            let (source, record) =
                construct(&mut runtime, &mut doc, kind, &[1.0, 2.0, 3.0, 4.0, 5.0]);
            let function = saved(&runtime);
            let counts = (runtime.objects.len(), runtime.typed_arrays.records.len());
            assert_eq!(
                runtime
                    .call(
                        function,
                        args.into_iter().map(Value::Number).collect(),
                        source.clone(),
                        &mut doc
                    )
                    .unwrap(),
                source
            );
            for (index, number) in expected.into_iter().enumerate() {
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
        for initial in [&[][..], &[7.0][..]] {
            let (mut runtime, mut doc) = fresh();
            let (source, record) = construct(&mut runtime, &mut doc, kind, initial);
            let function = saved(&runtime);
            assert_eq!(
                runtime
                    .call(function, vec![], source.clone(), &mut doc)
                    .unwrap(),
                source
            );
            for (index, number) in initial.iter().enumerate() {
                assert_eq!(
                    runtime.typed_array_read_number(record, index).unwrap(),
                    *number
                );
            }
            clean(&runtime);
        }
    }
}

#[test]
fn primitive_clamping_omission_and_fractional_indices_have_literal_results() {
    let n = Value::Number;
    let cases = [
        (vec![], [1, 2, 3, 4, 5]),
        (vec![n(1.0)], [1, 1, 2, 3, 4]),
        (vec![n(0.0), n(2.0)], [3, 4, 5, 4, 5]),
        (vec![n(0.0), n(2.0), Value::Undefined], [3, 4, 5, 4, 5]),
        (vec![n(0.0), n(2.0), Value::Null], [1, 2, 3, 4, 5]),
        (vec![n(-2.0), n(0.0), n(2.0)], [1, 2, 3, 1, 2]),
        (vec![n(0.0), n(-3.0), n(-1.0)], [3, 4, 3, 4, 5]),
        (vec![n(1.9), n(2.9), n(4.9)], [1, 3, 4, 4, 5]),
        (vec![n(-1.9), n(-3.9), n(-1.9)], [1, 2, 3, 4, 3]),
        (vec![n(f64::NAN), n(3.0)], [4, 5, 3, 4, 5]),
        (
            vec![n(f64::NEG_INFINITY), n(3.0), n(f64::INFINITY)],
            [4, 5, 3, 4, 5],
        ),
        (vec![n(f64::INFINITY), n(0.0)], [1, 2, 3, 4, 5]),
        (vec![n(0.0), n(f64::INFINITY)], [1, 2, 3, 4, 5]),
        (vec![n(0.0), n(0.0), n(f64::NEG_INFINITY)], [1, 2, 3, 4, 5]),
        (vec![n(0.0), n(4.0), n(1.0)], [1, 2, 3, 4, 5]),
        (
            vec![Value::Bool(true), Value::Null, n(2.0)],
            [1, 1, 2, 4, 5],
        ),
        (vec![n(-0.0), n(1.0), n(3.0)], [2, 3, 3, 4, 5]),
        (vec![n(-100.0), n(-2.0), n(100.0)], [4, 5, 3, 4, 5]),
    ];
    for (args, expected) in cases {
        let (mut runtime, mut doc) = fresh();
        let (source, record) = construct(
            &mut runtime,
            &mut doc,
            Kind::Uint8,
            &[1.0, 2.0, 3.0, 4.0, 5.0],
        );
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

// Arbitrary integer encodings and literal floating encodings: -0, a positive
// signaling NaN payload, a negative quiet NaN payload, +Infinity, -Infinity.
// Expected order below is fixed independently, with no floating decode/encode.
fn raw_elements(kind: Kind) -> [&'static [u8]; 5] {
    match kind {
        Kind::Int8 | Kind::Uint8 | Kind::Uint8Clamped => [&[0x80], &[0x81], &[0xfe], &[0x7f], &[0]],
        Kind::Int16 | Kind::Uint16 => [
            &[0, 0x80],
            &[0x13, 0x81],
            &[0x37, 0xfe],
            &[0xff, 0x7f],
            &[0, 0],
        ],
        Kind::Int32 | Kind::Uint32 => [
            &[0, 0, 0, 0x80],
            &[0x13, 0, 1, 0x81],
            &[0x37, 0, 2, 0xfe],
            &[0xff, 0xff, 0xff, 0x7f],
            &[0, 0, 0, 0],
        ],
        Kind::Float16 => [
            &[0, 0x80],
            &[0x15, 0x7c],
            &[0x35, 0xfe],
            &[0, 0x7c],
            &[0, 0xfc],
        ],
        Kind::Float32 => [
            &[0, 0, 0, 0x80],
            &[0x15, 0, 0x80, 0x7f],
            &[0x35, 0x12, 0xc0, 0xff],
            &[0, 0, 0x80, 0x7f],
            &[0, 0, 0x80, 0xff],
        ],
        Kind::Float64 => [
            &[0, 0, 0, 0, 0, 0, 0, 0x80],
            &[0x15, 0, 0, 0, 0, 0, 0xf0, 0x7f],
            &[0x35, 0x12, 0, 0, 0, 0, 0xf8, 0xff],
            &[0, 0, 0, 0, 0, 0, 0xf0, 0x7f],
            &[0, 0, 0, 0, 0, 0, 0xf0, 0xff],
        ],
    }
}

#[test]
fn raw_payloads_overlap_without_nan_canonicalization_or_sentinel_changes() {
    for kind in Kind::ALL {
        let width = kind.width();
        let elements = raw_elements(kind);
        for (args, order) in [
            ([1.0, 0.0, 4.0], [0, 0, 1, 2, 3]),
            ([0.0, 1.0, 5.0], [1, 2, 3, 4, 4]),
            ([0.0, 0.0, 5.0], [0, 1, 2, 3, 4]),
        ] {
            let mut payload = vec![0xa5; width];
            for element in elements {
                payload.extend_from_slice(element);
            }
            payload.resize(payload.len() + width, 0x5a);
            let mut expected = vec![0xa5; width];
            for index in order {
                expected.extend_from_slice(elements[index]);
            }
            expected.resize(expected.len() + width, 0x5a);
            let (mut runtime, mut doc, source, raw) = raw_fixture(kind, &payload, width, 5);
            let function = saved(&runtime);
            assert_eq!(
                runtime
                    .call(
                        function,
                        args.into_iter().map(Value::Number).collect(),
                        source.clone(),
                        &mut doc
                    )
                    .unwrap(),
                source
            );
            assert_eq!(bytes(&mut runtime, raw), expected, "{}", kind.name());
            clean(&runtime);
        }
    }
}

#[test]
fn invalid_brands_and_initial_invalid_views_throw_before_any_index_coercion() {
    authored(
        r#"(function(){
        var f=Uint8Array.prototype.copyWithin,good=new Uint8Array([1,2]);
        if(typeof f!=='function'||f.call(good,0,1)!==good||good[0]!==2)return false;
        var detached=new Uint8Array(1);detached.buffer.transfer();
        var detachedEmpty=new Uint8Array(0);detachedEmpty.buffer.transfer();
        var b=new ArrayBuffer(4,{maxByteLength:8}),fixed=new Uint8Array(b,0,4),tracking=new Uint8Array(b,2);b.resize(1);
        var calls=0,arg={valueOf:function(){calls++;return 0;}},fake=Object.create(good);
        var bad=[{},[],null,undefined,1,'x',new DataView(new ArrayBuffer(1)),fake,detached,detachedEmpty,fixed,tracking];
        for(var i=0;i<bad.length;i++){
            try{f.call(bad[i],arg,arg,arg);return false;}catch(e){if(!(e instanceof TypeError)||calls!==0)return false;}
        }
        b.resize(2);if(f.call(tracking,arg,arg,arg)!==tracking||tracking.length!==0)return false;
        return calls===3;
    })()"#,
    );
}

#[test]
fn target_start_end_coerce_once_in_order_even_when_the_range_is_empty() {
    authored(
        r#"(function(){
        var f=Uint8Array.prototype.copyWithin,t=new Uint8Array([1,2,3,4]),trace='';
        function arg(label,n){var x={valueOf:function(){throw 1;},toString:function(){throw 2;}};
            x[Symbol.toPrimitive]=function(hint){if(this!==x||hint!=='number')throw 3;trace+=label;return n;};return x;}
        if(f.call(t,arg('t',1),arg('s',0),arg('e',3))!==t||trace!=='tse')return false;
        if(t[0]!==1||t[1]!==1||t[2]!==2||t[3]!==3)return false;
        trace='';if(f.call(t,arg('t',4),arg('s',0),arg('e',4))!==t||trace!=='tse')return false;
        var empty=new Uint8Array(0);trace='';
        return f.call(empty,arg('t',0),arg('s',0),arg('e',0))===empty&&trace==='tse';
    })()"#,
    );
}

#[test]
fn abrupt_index_conversions_preserve_identity_order_and_unchanged_bytes() {
    authored(
        r#"(function(){
        var f=Uint8Array.prototype.copyWithin,control=new Uint8Array([1,2]);
        if(f.call(control,0,1)!==control||control[0]!==2)return false;
        for(var stage=0;stage<3;stage++){
            var t=new Uint8Array([1,2,3,4]),trace='',marker={};
            var a={valueOf:function(){trace+='t';if(stage===0)throw marker;return 0;}};
            var b={valueOf:function(){trace+='s';if(stage===1)throw marker;return 1;}};
            var c={valueOf:function(){trace+='e';if(stage===2)throw marker;return 4;}};
            try{f.call(t,a,b,c);return false;}catch(e){if(e!==marker)return false;}
            var expected=['t','ts','tse'];if(trace!==expected[stage])return false;
            for(var i=0;i<4;i++)if(t[i]!==i+1)return false;
        }
        for(var which=0;which<3;which++){
            var x=new Uint8Array([1,2]),seen='',symbol=Symbol('index');
            var first={valueOf:function(){seen+='t';return 0;}},next={valueOf:function(){seen+='s';return 1;}},last={valueOf:function(){seen+='e';return 2;}};
            var args=[first,next,last];args[which]=symbol;
            try{f.call(x,args[0],args[1],args[2]);return false;}catch(e){if(!(e instanceof TypeError))return false;}
            var prefixes=['','t','ts'];if(seen!==prefixes[which]||x[0]!==1||x[1]!==2)return false;
        }
        return true;
    })()"#,
    );
}

#[test]
fn omitted_and_undefined_end_differ_from_an_object_coercing_to_undefined() {
    authored(
        r#"(function(){
        var f=Uint8Array.prototype.copyWithin,a=new Uint8Array([1,2,3,4]),b=new Uint8Array([1,2,3,4]),c=new Uint8Array([1,2,3,4]),calls=0;
        if(f.call(a,0,2)!==a||f.call(b,0,2,undefined)!==b)return false;
        var end={valueOf:function(){calls++;return undefined;}};
        if(f.call(c,0,2,end)!==c||calls!==1)return false;
        var expected=[3,4,3,4];for(var i=0;i<4;i++)if(a[i]!==expected[i]||b[i]!==expected[i]||c[i]!==i+1)return false;
        var trace='',poison={valueOf:function(){throw 1;},toString:function(){throw 2;}};
        poison[Symbol.toPrimitive]=function(){throw 3;};
        function evaluated(){trace+='x';a[3]=9;return poison;}
        return f.call(a,0,2,undefined,evaluated())===a&&trace==='x'&&a[0]===3&&a[1]===9&&a[2]===3&&a[3]===9;
    })()"#,
    );
}

#[test]
fn initially_positive_count_revalidates_after_all_callbacks_even_for_self_copy() {
    authored(
        r#"(function(){
        var f=Uint8Array.prototype.copyWithin,control=new Uint8Array([1,2]);
        if(f.call(control,0,1)!==control||control[0]!==2)return false;
        for(var stage=0;stage<3;stage++){
            var t=new Uint8Array([1,2,3,4]),trace='',moved;
            function touch(label,n){trace+=label;if((stage===0&&label==='t')||(stage===1&&label==='s')||(stage===2&&label==='e'))moved=t.buffer.transfer();return n;}
            var target={valueOf:function(){return touch('t',0);}},start={valueOf:function(){return touch('s',0);}},end={valueOf:function(){return touch('e',4);}};
            try{f.call(t,target,start,end);return false;}catch(e){if(!(e instanceof TypeError))return false;}
            if(trace!=='tse'||t.length!==0)return false;
            var raw=new Uint8Array(moved);for(var i=0;i<4;i++)if(raw[i]!==i+1)return false;
        }
        var b=new ArrayBuffer(4,{maxByteLength:8}),fixed=new Uint8Array(b,0,4);fixed[0]=1;fixed[1]=2;
        var end={valueOf:function(){b.resize(2);return 4;}};
        try{f.call(fixed,0,0,end);return false;}catch(e){if(!(e instanceof TypeError))return false;}
        var raw=new Uint8Array(b);return raw.length===2&&raw[0]===1&&raw[1]===2;
    })()"#,
    );
}

#[test]
fn later_abrupt_conversion_precedes_final_detach_or_oob_validation() {
    authored(
        r#"(function(){
        var f=Uint8Array.prototype.copyWithin,control=new Uint8Array([1,2]);
        if(f.call(control,0,1)!==control||control[0]!==2)return false;
        var t=new Uint8Array([1,2,3]),trace='',marker={},moved;
        var target={valueOf:function(){trace+='t';moved=t.buffer.transfer();return 0;}};
        var start={valueOf:function(){trace+='s';throw marker;}},end={valueOf:function(){trace+='e';return 3;}};
        try{f.call(t,target,start,end);return false;}catch(e){if(e!==marker||trace!=='ts')return false;}
        var raw=new Uint8Array(moved);if(raw[0]!==1||raw[1]!==2||raw[2]!==3)return false;
        var b=new ArrayBuffer(4,{maxByteLength:8}),fixed=new Uint8Array(b,0,4);trace='';
        var first={valueOf:function(){trace+='t';b.resize(0);return 0;}};
        var next={valueOf:function(){trace+='s';return 1;}},last={valueOf:function(){trace+='e';throw marker;}};
        try{f.call(fixed,first,next,last);return false;}catch(e){if(e!==marker||trace!=='tse')return false;}
        return fixed.length===0;
    })()"#,
    );
}

#[test]
fn initially_nonpositive_count_skips_final_detached_and_oob_validation() {
    authored(
        r#"(function(){
        var f=Uint8Array.prototype.copyWithin;
        for(var mode=0;mode<3;mode++){
            var t=new Uint8Array([1,2,3,4]),moved,trace='';
            var target={valueOf:function(){trace+='t';moved=t.buffer.transfer();return mode===0?4:0;}};
            var start={valueOf:function(){trace+='s';return mode===2?3:2;}};
            var end={valueOf:function(){trace+='e';return mode===0?4:mode===1?2:1;}};
            if(f.call(t,target,start,end)!==t||trace!=='tse'||t.length!==0)return false;
            var raw=new Uint8Array(moved);for(var i=0;i<4;i++)if(raw[i]!==i+1)return false;
        }
        var b=new ArrayBuffer(4,{maxByteLength:8}),fixed=new Uint8Array(b,0,4),trace='';
        var target={valueOf:function(){trace+='t';b.resize(0);return 4;}},start={valueOf:function(){trace+='s';return 0;}},end={valueOf:function(){trace+='e';return 4;}};
        if(f.call(fixed,target,start,end)!==fixed||trace!=='tse'||fixed.length!==0)return false;
        var empty=new Uint8Array(0),count=0;
        var first={valueOf:function(){count++;empty.buffer.transfer();return 0;}},next={valueOf:function(){count++;return 0;}};
        return f.call(empty,first,next,next)===empty&&count===3;
    })()"#,
    );
}

#[test]
fn tracking_shrink_clips_both_source_and_destination_without_reclamping_indices() {
    authored(
        r#"(function(){
        var f=Uint8Array.prototype.copyWithin;
        var b=new ArrayBuffer(6,{maxByteLength:8}),t=new Uint8Array(b);for(var i=0;i<6;i++)t[i]=i+1;
        var target={valueOf:function(){b.resize(4);return 1;}};
        if(f.call(t,target,0,5)!==t||t.length!==4)return false;
        var a=[1,1,2,3];for(var j=0;j<4;j++)if(t[j]!==a[j])return false;
        var c=new ArrayBuffer(6,{maxByteLength:8}),u=new Uint8Array(c);for(var k=0;k<6;k++)u[k]=k+1;
        var end={valueOf:function(){c.resize(4);return 6;}};
        if(f.call(u,0,2,end)!==u)return false;
        var q=[3,4,3,4];for(var m=0;m<4;m++)if(u[m]!==q[m])return false;
        var d=new ArrayBuffer(6,{maxByteLength:8}),v=new Uint8Array(d);for(var n=0;n<6;n++)v[n]=n+1;
        var negative={valueOf:function(){d.resize(3);return -2;}};
        if(f.call(v,negative,0,2)!==v||v.length!==3)return false;
        return v[0]===1&&v[1]===2&&v[2]===3;
    })()"#,
    );
}

#[test]
fn growth_does_not_expand_captured_range_or_reinterpret_negative_indices() {
    authored(
        r#"(function(){
        var f=Uint8Array.prototype.copyWithin,b=new ArrayBuffer(4,{maxByteLength:8}),t=new Uint8Array(b);
        for(var i=0;i<4;i++)t[i]=i+1;
        var target={valueOf:function(){b.resize(8);for(var j=4;j<8;j++)t[j]=j+1;return -2;}};
        if(f.call(t,target,0)!==t||t.length!==8)return false;
        var expected=[1,2,1,2,5,6,7,8];for(var k=0;k<8;k++)if(t[k]!==expected[k])return false;
        var c=new ArrayBuffer(4,{maxByteLength:8}),u=new Uint8Array(c);for(var n=0;n<4;n++)u[n]=n+1;
        var start={valueOf:function(){c.resize(8);for(var p=4;p<8;p++)u[p]=p+1;return 1;}};
        if(f.call(u,0,start)!==u)return false;
        var second=[2,3,4,4,5,6,7,8];for(var q=0;q<8;q++)if(u[q]!==second[q])return false;
        return true;
    })()"#,
    );
}

#[test]
fn transient_fixed_oob_recovers_after_later_callbacks_and_record_reallocation() {
    authored(
        r#"(function(){
        var f=Uint8Array.prototype.copyWithin,b=new ArrayBuffer(4,{maxByteLength:8}),t=new Uint8Array(b,0,4),kept=[],trace='';
        for(var i=0;i<4;i++)t[i]=i+1;
        function grow(label){trace+=label;for(var j=0;j<4;j++){var buffer=new ArrayBuffer(2),view=new Uint8Array(buffer);view[0]=kept.length+10;kept[kept.length]=view;}}
        var target={valueOf:function(){grow('t');b.resize(2);return 1;}};
        var start={valueOf:function(){grow('s');b.resize(8);return 0;}};
        var end={valueOf:function(){grow('e');b.resize(4);return 4;}};
        if(f.call(t,target,start,end)!==t||trace!=='tse'||kept.length!==12)return false;
        var expected=[1,1,2,0];for(var k=0;k<4;k++)if(t[k]!==expected[k])return false;
        for(var n=0;n<12;n++)if(kept[n][0]!==n+10||kept[n].byteLength!==2)return false;
        return b.byteLength===4&&t.length===4;
    })()"#,
    );
}

#[test]
fn offset_tracking_views_preserve_partial_trailing_bytes_and_full_sentinels() {
    authored(
        r#"(function(){
        var f=Uint16Array.prototype.copyWithin,b=new ArrayBuffer(10,{maxByteLength:12}),raw=new Uint8Array(b),t=new Uint16Array(b,2);
        for(var i=0;i<10;i++)raw[i]=i+1;
        var end={valueOf:function(){b.resize(7);return 4;}};
        if(f.call(t,1,0,end)!==t||t.length!==2)return false;
        var expected=[1,2,3,4,3,4,7];for(var k=0;k<7;k++)if(raw[k]!==expected[k])return false;
        var c=new ArrayBuffer(10,{maxByteLength:12}),other=new Uint8Array(c),u=new Uint16Array(c,2);for(var j=0;j<10;j++)other[j]=j+1;
        var shorter={valueOf:function(){c.resize(7);return 4;}};
        if(f.call(u,0,1,shorter)!==u||u.length!==2)return false;
        var second=[1,2,5,6,5,6,7];for(var n=0;n<7;n++)if(other[n]!==second[n])return false;
        return true;
    })()"#,
    );
}

#[test]
fn saved_method_ignores_public_metadata_species_globals_and_numeric_prototypes() {
    authored(
        r#"(function(){
        var C=Uint16Array,f=C.prototype.copyWithin,b=new ArrayBuffer(8),t=new C(b),alias=new C(b),calls=0,poison={};
        for(var i=0;i<4;i++)t[i]=i+1;
        function fail(){calls++;throw 1;}
        Object.defineProperty(t,'length',{get:fail});Object.defineProperty(t,'byteLength',{get:fail});Object.defineProperty(t,'byteOffset',{get:fail});Object.defineProperty(t,'buffer',{get:fail});Object.defineProperty(t,'constructor',{get:fail});
        Object.defineProperty(C,Symbol.species,{get:fail});
        Object.defineProperty(poison,'0',{get:fail,set:fail});Object.defineProperty(poison,'1',{get:fail,set:fail});
        Object.setPrototypeOf(t,poison);t.note='kept';Uint16Array=fail;
        if(f.call(t,1,0,3)!==t||calls!==0||t.note!=='kept')return false;
        return alias[0]===1&&alias[1]===1&&alias[2]===2&&alias[3]===3&&b.byteLength===8;
    })()"#,
    );
}

#[test]
fn shared_metadata_saved_identity_and_nonconstructibility_are_literal() {
    authored(
        r#"(function(){
        var f=Uint8Array.prototype.copyWithin,shared=Object.getPrototypeOf(Uint8Array.prototype);
        var kinds=[Int8Array,Uint8Array,Uint8ClampedArray,Int16Array,Uint16Array,Int32Array,Uint32Array,Float16Array,Float32Array,Float64Array];
        for(var i=0;i<kinds.length;i++)if(kinds[i].prototype.copyWithin!==f)return false;
        var d=Object.getOwnPropertyDescriptor(shared,'copyWithin');if(d.value!==f||!d.writable||d.enumerable||!d.configurable)return false;
        var n=Object.getOwnPropertyDescriptor(f,'name'),l=Object.getOwnPropertyDescriptor(f,'length');
        if(n.value!=='copyWithin'||n.writable||n.enumerable||!n.configurable)return false;
        if(l.value!==2||l.writable||l.enumerable||!l.configurable||Object.hasOwn(f,'prototype'))return false;
        if(Object.getPrototypeOf(f)!==Function.prototype)return false;
        function C(){this.ok=1;}if(Reflect.construct(C,[]).ok!==1)return false;
        try{Reflect.construct(f,[]);return false;}catch(e){if(!(e instanceof TypeError))return false;}
        shared.copyWithin=function(){throw 1;};var t=new Uint8Array([1,2]);
        return f.call(t,0,1)===t&&t[0]===2&&t[1]===2;
    })()"#,
    );
}
