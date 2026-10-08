use super::*;

// Independent numeric/order/byte oracles for ECMA-262 23.2.3.25.
// https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.reverse
// This source belongs under typed_array/reverse/tests.rs after root integration.

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
    match &runtime.objects[owner].values[&JsString::from("reverse").into()].value {
        PropertyValue::Data { value, .. } => value.clone(),
        _ => panic!("installed reverse method"),
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
    // Inspect raw storage through the existing admitted interface. No sibling
    // private fields, candidate codec oracle, or observed-budget mutation.
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
) -> (Runtime, Document, Value, Record, Record) {
    let (mut runtime, mut doc) = fresh();
    runtime.execute(&format!(
        "var buffer=new ArrayBuffer({});var raw=new Uint8Array(buffer);var source=new {}(buffer,{offset},{length});",
        payload.len(), kind.name()
    ), &mut doc).unwrap();
    let source = runtime.lookup(1, "source").unwrap().1;
    let raw = runtime.lookup(1, "raw").unwrap().1;
    let record = runtime.typed_array_record(&source).unwrap().unwrap();
    let raw_record = runtime.typed_array_record(&raw).unwrap().unwrap();
    let live = runtime.typed_array_live(raw_record).unwrap().unwrap();
    for (index, chunk) in payload.chunks(8).enumerate() {
        runtime
            .buffer_view_write(live.buffer, index * 8, chunk)
            .unwrap();
    }
    assert_eq!(bytes(&mut runtime, raw_record), payload);
    (runtime, doc, source, record, raw_record)
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
fn all_ten_kinds_reverse_even_odd_empty_and_single_views_with_identity() {
    for kind in Kind::ALL {
        for (initial, expected) in [
            (&[][..], &[][..]),
            (&[1.0][..], &[1.0][..]),
            (&[1.0, 2.0, 3.0, 4.0][..], &[4.0, 3.0, 2.0, 1.0][..]),
            (&[1.0, 2.0, 3.0][..], &[3.0, 2.0, 1.0][..]),
        ] {
            let (mut runtime, mut doc) = fresh();
            let (source, record) = construct(&mut runtime, &mut doc, kind, initial);
            let function = saved(&runtime);
            let counts = (runtime.objects.len(), runtime.typed_arrays.records.len());
            assert_eq!(
                runtime
                    .call(function.clone(), vec![], source.clone(), &mut doc)
                    .unwrap(),
                source
            );
            for (index, number) in expected.iter().enumerate() {
                assert_eq!(
                    runtime.typed_array_read_number(record, index).unwrap(),
                    *number
                );
            }
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
            assert_eq!(
                (runtime.objects.len(), runtime.typed_arrays.records.len()),
                counts
            );
            clean(&runtime);
        }
    }
}

#[test]
fn signed_unsigned_extremes_and_clamped_values_are_preserved_numerically() {
    authored(
        r#"(function(){
        var kinds=[Int8Array,Uint8Array,Uint8ClampedArray,Int16Array,Uint16Array,Int32Array,Uint32Array];
        var a=[-128,0,0,-32768,0,-2147483648,0],b=[127,255,255,32767,65535,2147483647,4294967295];
        for(var i=0;i<kinds.length;i++){
            var t=new kinds[i]([a[i],2,b[i]]);
            if(t.reverse()!==t||t[0]!==b[i]||t[1]!==2||t[2]!==a[i])return false;
        }
        return true;
    })()"#,
    );
}

#[test]
fn arguments_are_evaluated_but_never_coerced_and_record_growth_precedes_dispatch() {
    authored(
        r#"(function(){
        var reverse=Uint8Array.prototype.reverse,t=new Uint8Array([1,2,3]),trace='',calls=0,marker={};
        var poison={valueOf:function(){calls++;throw marker;},toString:function(){calls++;throw marker;}};
        poison[Symbol.toPrimitive]=function(){calls++;throw marker;};
        function evaluated(){trace+='e';for(var i=0;i<12;i++)new Uint8Array(1);t[1]=7;return poison;}
        if(reverse.call(t,evaluated(),Symbol('ignored'),undefined)!==t||trace!=='e'||calls!==0)return false;
        if(t[0]!==3||t[1]!==7||t[2]!==1)return false;
        function abrupt(){trace+='x';throw marker;}
        try{reverse.call(t,abrupt());return false;}catch(e){if(e!==marker)return false;}
        return trace==='ex'&&calls===0&&t[0]===3&&t[1]===7&&t[2]===1;
    })()"#,
    );
}

#[test]
fn initial_brand_detached_and_out_of_bounds_checks_include_empty_views() {
    authored(
        r#"(function(){
        var reverse=Uint8Array.prototype.reverse,ok=new Uint8Array(0);
        if(typeof reverse!=='function'||reverse.call(ok)!==ok)return false;
        var detached=new Uint8Array(1);detached.buffer.transfer();
        var detachedEmpty=new Uint8Array(0);detachedEmpty.buffer.transfer();
        var b=new ArrayBuffer(4,{maxByteLength:8}),fixed=new Uint8Array(b,0,4),tracking=new Uint8Array(b,2);b.resize(1);
        var fake=Object.create(new Uint8Array(1));
        var bad=[{},[],null,undefined,1,'x',new DataView(new ArrayBuffer(1)),fake,detached,detachedEmpty,fixed,tracking];
        var calls=0,arg={valueOf:function(){calls++;return 0;}};
        for(var i=0;i<bad.length;i++){
            try{reverse.call(bad[i],arg);return false;}catch(e){if(!(e instanceof TypeError)||calls!==0)return false;}
        }
        b.resize(2);if(reverse.call(tracking)!==tracking||tracking.length!==0)return false;
        return calls===0;
    })()"#,
    );
}

#[test]
fn fixed_and_tracking_views_use_current_resized_bounds_and_whole_elements() {
    authored(
        r#"(function(){
        var reverse=Uint8Array.prototype.reverse;
        var b=new ArrayBuffer(4,{maxByteLength:8}),fixed=new Uint8Array(b,0,4),raw=new Uint8Array(b);
        raw[0]=1;raw[1]=2;raw[2]=3;raw[3]=4;b.resize(2);
        try{reverse.call(fixed);return false;}catch(e){if(!(e instanceof TypeError))return false;}
        if(raw[0]!==1||raw[1]!==2)return false;
        b.resize(6);if(reverse.call(fixed)!==fixed)return false;
        var first=[0,0,2,1,0,0];for(var i=0;i<6;i++)if(raw[i]!==first[i])return false;
        var tb=new ArrayBuffer(4,{maxByteLength:8}),t=new Uint8Array(tb);t[0]=1;t[1]=2;t[2]=3;t[3]=4;
        tb.resize(2);if(t.reverse()!==t||t[0]!==2||t[1]!==1)return false;
        tb.resize(6);t.reverse();var second=[0,0,0,0,1,2];for(var j=0;j<6;j++)if(t[j]!==second[j])return false;
        var pb=new ArrayBuffer(8,{maxByteLength:8}),praw=new Uint8Array(pb),p=new Uint16Array(pb,2);
        for(var k=0;k<8;k++)praw[k]=k+1;
        pb.resize(5);if(p.reverse()!==p||p.length!==1)return false;
        for(var q=0;q<5;q++)if(praw[q]!==q+1)return false;
        pb.resize(8);p.reverse();var third=[1,2,0,0,5,0,3,4];
        for(var z=0;z<8;z++)if(praw[z]!==third[z])return false;
        return true;
    })()"#,
    );
}

#[test]
fn shared_metadata_saved_identity_and_nonconstructibility_are_literal() {
    authored(
        r#"(function(){
        var reverse=Uint8Array.prototype.reverse,shared=Object.getPrototypeOf(Uint8Array.prototype);
        var kinds=[Int8Array,Uint8Array,Uint8ClampedArray,Int16Array,Uint16Array,Int32Array,Uint32Array,Float16Array,Float32Array,Float64Array];
        for(var i=0;i<kinds.length;i++)if(kinds[i].prototype.reverse!==reverse)return false;
        var d=Object.getOwnPropertyDescriptor(shared,'reverse');
        if(d.value!==reverse||!d.writable||d.enumerable||!d.configurable)return false;
        var n=Object.getOwnPropertyDescriptor(reverse,'name'),l=Object.getOwnPropertyDescriptor(reverse,'length');
        if(n.value!=='reverse'||n.writable||n.enumerable||!n.configurable)return false;
        if(l.value!==0||l.writable||l.enumerable||!l.configurable||Object.hasOwn(reverse,'prototype'))return false;
        if(Object.getPrototypeOf(reverse)!==Function.prototype)return false;
        function C(){this.ok=1;}if(Reflect.construct(C,[]).ok!==1)return false;
        try{Reflect.construct(reverse,[]);return false;}catch(e){if(!(e instanceof TypeError))return false;}
        shared.reverse=function(){throw 1;};var t=new Uint8Array([1,2]);
        return reverse.call(t)===t&&t[0]===2&&t[1]===1;
    })()"#,
    );
}

#[test]
fn poisoned_public_length_constructor_and_numeric_prototypes_are_not_observed() {
    authored(
        r#"(function(){
        var reverse=Uint16Array.prototype.reverse,b=new ArrayBuffer(6),t=new Uint16Array(b),alias=new Uint16Array(b);
        t[0]=1;t[1]=2;t[2]=3;var calls=0,poison={};
        Object.defineProperty(t,'length',{get:function(){calls++;throw 1;}});
        Object.defineProperty(t,'constructor',{get:function(){calls++;throw 2;}});
        Object.defineProperty(poison,'0',{get:function(){calls++;throw 3;},set:function(){calls++;throw 4;}});
        Object.defineProperty(poison,'1',{get:function(){calls++;throw 5;},set:function(){calls++;throw 6;}});
        Object.setPrototypeOf(t,poison);t.note='kept';
        return reverse.call(t)===t&&alias[0]===3&&alias[1]===2&&alias[2]===1&&t.note==='kept'&&calls===0;
    })()"#,
    );
}

// Literal little-endian encodings of1,2,3,4. No codec or candidate operation
// generates expected bytes, including the floating encodings.
fn scalar(kind: Kind, value: usize) -> &'static [u8] {
    match kind {
        Kind::Int8 | Kind::Uint8 | Kind::Uint8Clamped => [&[1][..], &[2], &[3], &[4]][value - 1],
        Kind::Int16 | Kind::Uint16 => [&[1, 0][..], &[2, 0], &[3, 0], &[4, 0]][value - 1],
        Kind::Int32 | Kind::Uint32 => [
            &[1, 0, 0, 0][..],
            &[2, 0, 0, 0],
            &[3, 0, 0, 0],
            &[4, 0, 0, 0],
        ][value - 1],
        Kind::Float16 => [&[0, 60][..], &[0, 64], &[0, 66], &[0, 68]][value - 1],
        Kind::Float32 => [
            &[0, 0, 128, 63][..],
            &[0, 0, 0, 64],
            &[0, 0, 64, 64],
            &[0, 0, 128, 64],
        ][value - 1],
        Kind::Float64 => [
            &[0, 0, 0, 0, 0, 0, 240, 63][..],
            &[0, 0, 0, 0, 0, 0, 0, 64],
            &[0, 0, 0, 0, 0, 0, 8, 64],
            &[0, 0, 0, 0, 0, 0, 16, 64],
        ][value - 1],
    }
}

fn literal_bytes(kind: Kind, values: &[usize]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| scalar(kind, *value).iter().copied())
        .collect()
}

fn prefix(kind: Kind, stores: usize) -> Vec<u8> {
    let order = match stores {
        0 => [1, 2, 3, 4],
        1 => [4, 2, 3, 4],
        2 => [4, 2, 3, 1],
        3 => [4, 3, 3, 1],
        4 => [4, 3, 2, 1],
        _ => panic!("four ordered stores"),
    };
    literal_bytes(kind, &order)
}

// (one read, one write, one pair, first lower store within the pair).
// Independent helper tariff sum: R=74+3w, W=66+3w+clamp;
// pair=8+2R+2W. These literals are not obtained by observing reverse.
fn fees(kind: Kind) -> (usize, usize, usize, usize) {
    match kind {
        Kind::Int8 | Kind::Uint8 => (77, 69, 300, 231),
        Kind::Uint8Clamped => (77, 85, 332, 247),
        Kind::Int16 | Kind::Uint16 | Kind::Float16 => (80, 72, 312, 240),
        Kind::Int32 | Kind::Uint32 | Kind::Float32 => (86, 78, 336, 258),
        Kind::Float64 => (98, 90, 384, 294),
    }
}

#[test]
fn floating_outer_values_and_untouched_odd_midpoint_have_literal_bytes() {
    type FloatByteRow<'a> = (Kind, &'a [u8], &'a [u8], &'a [u8], &'a [u8]);
    let rows: &[FloatByteRow<'_>] = &[
        (Kind::Float16, &[0, 128], &[1, 252], &[0, 124], &[0, 126]),
        (
            Kind::Float32,
            &[0, 0, 0, 128],
            &[1, 0, 128, 255],
            &[0, 0, 128, 127],
            &[0, 0, 192, 127],
        ),
        (
            Kind::Float64,
            &[0, 0, 0, 0, 0, 0, 0, 128],
            &[1, 0, 0, 0, 0, 0, 240, 255],
            &[0, 0, 0, 0, 0, 0, 240, 127],
            &[0, 0, 0, 0, 0, 0, 248, 127],
        ),
    ];
    for &(kind, minus_zero, raw_nan, infinity, canonical_nan) in rows {
        // A signaling NaN at the odd midpoint is never read/re-encoded.
        let payload = [minus_zero, raw_nan, infinity].concat();
        let expected = [infinity, raw_nan, minus_zero].concat();
        let (mut runtime, _doc, source, record, raw) = raw_fixture(kind, &payload, 0, 3);
        runtime.steps = 25 + fees(kind).2;
        runtime.allocated = MAX_HEAP;
        assert_eq!(
            runtime.typed_array_reverse(source.clone(), record).unwrap(),
            source
        );
        assert_eq!(bytes(&mut runtime, raw), expected);
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        clean(&runtime);
        // Visited NaNs follow numeric read/encode semantics, not a raw swap.
        let payload = [raw_nan, minus_zero].concat();
        let expected = [minus_zero, canonical_nan].concat();
        let (mut runtime, _doc, source, record, raw) = raw_fixture(kind, &payload, 0, 2);
        runtime.steps = 25 + fees(kind).2;
        runtime.allocated = MAX_HEAP;
        assert_eq!(
            runtime.typed_array_reverse(source.clone(), record).unwrap(),
            source
        );
        assert_eq!(bytes(&mut runtime, raw), expected);
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        clean(&runtime);
    }
}

#[test]
fn literal_read_and_ordered_store_boundaries_retain_only_completed_scalars() {
    for kind in Kind::ALL {
        let (read, write, pair, lower) = fees(kind);
        let mut cuts = vec![0, 7, 8, 20, 21, 24, 25];
        for completed_pairs in 0..2 {
            let begin = 25 + completed_pairs * pair;
            for endpoint in [8, 8 + read, 8 + 2 * read, 8 + 2 * read + write, pair] {
                cuts.extend([begin + endpoint - 1, begin + endpoint]);
            }
        }
        cuts.sort_unstable();
        cuts.dedup();
        let endpoints = [25 + lower, 25 + pair, 25 + pair + lower, 25 + 2 * pair];
        for allowance in cuts {
            let (mut runtime, mut doc) = fresh();
            let (source, record) = construct(&mut runtime, &mut doc, kind, &[1.0, 2.0, 3.0, 4.0]);
            assert_eq!(bytes(&mut runtime, record), prefix(kind, 0));
            let counts = (runtime.objects.len(), runtime.typed_arrays.records.len());
            runtime.steps = allowance;
            runtime.allocated = MAX_HEAP;
            let result = runtime.typed_array_reverse(source.clone(), record);
            if allowance == 25 + 2 * pair {
                assert_eq!(result.unwrap(), source);
            } else {
                assert!(result.unwrap_err().is_resource_limit());
            }
            let stores = endpoints
                .iter()
                .filter(|endpoint| allowance >= **endpoint)
                .count();
            assert_eq!(
                bytes(&mut runtime, record),
                prefix(kind, stores),
                "{} cut={allowance}",
                kind.name()
            );
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            assert_eq!(
                (runtime.objects.len(), runtime.typed_arrays.records.len()),
                counts
            );
            clean(&runtime);
        }
    }
}

#[test]
fn aligned_views_keep_backing_sentinels_at_partial_and_complete_pairs() {
    for kind in Kind::ALL {
        let width = scalar(kind, 1).len();
        let (_, _, pair, lower) = fees(kind);
        let payload = [vec![165; width], prefix(kind, 0), vec![90; width]].concat();
        for (allowance, stores) in [
            (25 + lower - 1, 0),
            (25 + lower, 1),
            (25 + pair, 2),
            (25 + pair + lower, 3),
            (25 + 2 * pair - 1, 3),
            (25 + 2 * pair, 4),
        ] {
            let (mut runtime, _doc, source, record, raw) = raw_fixture(kind, &payload, width, 4);
            runtime.steps = allowance;
            runtime.allocated = MAX_HEAP;
            let result = runtime.typed_array_reverse(source.clone(), record);
            if stores == 4 {
                assert_eq!(result.unwrap(), source);
            } else {
                assert!(result.unwrap_err().is_resource_limit());
            }
            assert_eq!(
                bytes(&mut runtime, raw),
                [vec![165; width], prefix(kind, stores), vec![90; width]].concat()
            );
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            clean(&runtime);
        }
    }
}

#[test]
fn empty_single_and_equal_pairs_keep_exact_work_and_require_no_body_heap() {
    for kind in Kind::ALL {
        for initial in [&[][..], &[1.0][..], &[1.0, 1.0][..]] {
            let work = 25 + (initial.len() / 2) * fees(kind).2;
            for allowance in [0, work - 1, work] {
                let (mut runtime, mut doc) = fresh();
                let (source, record) = construct(&mut runtime, &mut doc, kind, initial);
                let original = scalar(kind, 1).repeat(initial.len());
                runtime.steps = allowance;
                runtime.allocated = MAX_HEAP;
                let result = runtime.typed_array_reverse(source.clone(), record);
                if allowance == work {
                    assert_eq!(result.unwrap(), source);
                } else {
                    assert!(result.unwrap_err().is_resource_limit());
                }
                assert_eq!(bytes(&mut runtime, record), original);
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
                clean(&runtime);
            }
        }
    }
}

#[test]
fn saved_call_exact_and_one_short_work_heap_and_cleanup_are_complete_boundaries() {
    for kind in Kind::ALL {
        let (mut measured, mut doc) = fresh();
        let (source, record) = construct(&mut measured, &mut doc, kind, &[1.0, 2.0, 3.0, 4.0]);
        let function = saved(&measured);
        let before = (measured.steps, measured.allocated);
        assert_eq!(
            measured
                .call(function, vec![], source.clone(), &mut doc)
                .unwrap(),
            source
        );
        let work = before.0 - measured.steps;
        let heap = measured.allocated - before.1;
        assert_eq!(bytes(&mut measured, record), prefix(kind, 4));
        assert!(work > 25 + 2 * fees(kind).2);
        assert_eq!(heap, 50); // Existing preflight32 + TypedArray.reverse's18 bytes.
        clean(&measured);
        for (allowance, room, succeeds, stores) in [
            (work, 50, true, 4),
            (work - 1, 50, false, 3),
            (0, 50, false, 0),
            (work, 49, false, 0),
            (work, 0, false, 0),
        ] {
            let (mut runtime, mut doc) = fresh();
            let (source, record) = construct(&mut runtime, &mut doc, kind, &[1.0, 2.0, 3.0, 4.0]);
            let function = saved(&runtime);
            let bag = runtime.property_object(&function).unwrap();
            let metadata = format!("{:?}", runtime.objects[bag].values);
            let counts = (runtime.objects.len(), runtime.typed_arrays.records.len());
            runtime.steps = allowance;
            runtime.allocated = MAX_HEAP - room;
            let result = runtime.call(function, vec![], source.clone(), &mut doc);
            if succeeds {
                assert_eq!(result.unwrap(), source);
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                if room == 50 {
                    assert_eq!(runtime.steps, 0);
                } else {
                    assert!(runtime.allocated > MAX_HEAP);
                }
            }
            assert_eq!(bytes(&mut runtime, record), prefix(kind, stores));
            assert_eq!(
                (runtime.objects.len(), runtime.typed_arrays.records.len()),
                counts
            );
            assert_eq!(format!("{:?}", runtime.objects[bag].values), metadata);
            clean(&runtime);
        }
    }
}
