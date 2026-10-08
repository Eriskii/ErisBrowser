use super::*;

// Independent tests for ECMA-262 23.2.3.32 and 23.2.4.2.
// Authored without reading the toReversed production draft.
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
    match &runtime.objects[owner].values[&PropertyKey::from("toReversed")].value {
        PropertyValue::Data { value, .. } => value.clone(),
        _ => panic!("installed saved toReversed"),
    }
}

fn construct(
    runtime: &mut Runtime,
    doc: &mut Document,
    kind: Kind,
    values: &[f64],
) -> (Value, Record) {
    let ctor = runtime.typed_arrays.constructors[kind.index()]
        .clone()
        .unwrap();
    let value = runtime
        .typed_array_constructor(kind, &[Value::Number(values.len() as f64)], ctor, doc)
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
    // Test inspection only: preserve the outcome's counters and use the
    // existing opaque buffer accessor, never sibling-private buffer records.
    let before = (runtime.steps, runtime.allocated);
    runtime.steps = MAX_STEPS;
    let live = runtime.typed_array_live(record).unwrap().unwrap();
    let width = record.kind.width();
    let mut out = Vec::new();
    for index in 0..live.length {
        let raw = runtime
            .buffer_view_read(live.buffer, live.offset + index * width, width)
            .unwrap();
        out.extend_from_slice(&raw[..width]);
    }
    assert_eq!(runtime.allocated, before.1);
    runtime.steps = before.0;
    out
}

fn result_record(runtime: &mut Runtime, value: &Value) -> Record {
    let before = runtime.steps;
    runtime.steps = MAX_STEPS;
    let record = runtime.typed_array_record(value).unwrap().unwrap();
    runtime.steps = before;
    record
}

fn buffer_value(runtime: &mut Runtime, record: Record) -> Value {
    let before = runtime.steps;
    runtime.steps = MAX_STEPS;
    let value = runtime
        .buffer_view_value(record.backing.unwrap().buffer)
        .unwrap();
    runtime.steps = before;
    value
}

fn assert_fixed(runtime: &mut Runtime, record: Record, kind: Kind, length: usize) {
    assert_eq!(record.kind, kind);
    let backing = record.backing.unwrap();
    assert_eq!(backing.offset, 0);
    assert!(matches!(backing.length, ViewLength::Fixed(n) if n == length));
    let before = runtime.steps;
    runtime.steps = MAX_STEPS;
    let metadata = runtime.buffer_view_metadata(backing.buffer).unwrap();
    assert!(!metadata.resizable);
    assert_eq!(metadata.byte_length, Some(length * kind.width()));
    runtime.steps = before;
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

// Literal little-endian encodings. No candidate operation or codec generates
// expected bytes. Zero blocks are used only for newly allocated unwritten data.
fn scalar(kind: Kind, number: usize) -> &'static [u8] {
    match kind {
        Kind::Int8 | Kind::Uint8 | Kind::Uint8Clamped => [&[1][..], &[2], &[3], &[4]][number - 1],
        Kind::Int16 | Kind::Uint16 => [&[1, 0][..], &[2, 0], &[3, 0], &[4, 0]][number - 1],
        Kind::Int32 | Kind::Uint32 => [
            &[1, 0, 0, 0][..],
            &[2, 0, 0, 0],
            &[3, 0, 0, 0],
            &[4, 0, 0, 0],
        ][number - 1],
        Kind::Float16 => [&[0, 60][..], &[0, 64], &[0, 66], &[0, 68]][number - 1],
        Kind::Float32 => [
            &[0, 0, 128, 63][..],
            &[0, 0, 0, 64],
            &[0, 0, 64, 64],
            &[0, 0, 128, 64],
        ][number - 1],
        Kind::Float64 => [
            &[0, 0, 0, 0, 0, 0, 240, 63][..],
            &[0, 0, 0, 0, 0, 0, 0, 64],
            &[0, 0, 0, 0, 0, 0, 8, 64],
            &[0, 0, 0, 0, 0, 0, 16, 64],
        ][number - 1],
    }
}

fn literals(kind: Kind, values: &[usize]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|v| scalar(kind, *v).iter().copied())
        .collect()
}

fn prefix(kind: Kind, length: usize, written: usize) -> Vec<u8> {
    let mut out = Vec::new();
    for index in 0..length {
        if index < written {
            out.extend_from_slice(scalar(kind, length - index));
        } else {
            out.extend(std::iter::repeat_n(0, kind.width()));
        }
    }
    out
}

// (read, write, whole reached element). Independent retained scalar tariffs.
fn fees(kind: Kind) -> (usize, usize, usize) {
    match kind {
        Kind::Int8 | Kind::Uint8 => (77, 69, 154),
        Kind::Uint8Clamped => (77, 85, 170),
        Kind::Int16 | Kind::Uint16 | Kind::Float16 => (80, 72, 160),
        Kind::Int32 | Kind::Uint32 | Kind::Float32 => (86, 78, 172),
        Kind::Float64 => (98, 90, 196),
    }
}

fn fixture(
    kind: Kind,
    length: usize,
    grow: bool,
    expandos: usize,
) -> (Runtime, Document, Value, Record) {
    let (mut runtime, mut doc) = fresh();
    let values: Vec<_> = (1..=length).map(|n| n as f64).collect();
    let (source, record) = construct(&mut runtime, &mut doc, kind, &values);
    if grow {
        // Authentic construction fills the existing typed and buffer tables.
        // Every zero-length dummy creates its own genuine buffer and view.
        while runtime.typed_arrays.records.len() < runtime.typed_arrays.records.capacity() {
            construct(&mut runtime, &mut doc, Kind::Uint8, &[]);
        }
    }
    if expandos != 0 {
        let ctor = runtime.typed_arrays.constructors[kind.index()]
            .as_ref()
            .unwrap();
        let owner = runtime.property_object(ctor).unwrap();
        // Private setup shapes the genuine immutable-prototype constructor bag;
        // production constructor lookup, including its dynamic fee, is measured.
        for index in 0..expandos {
            runtime.objects[owner].insert(
                format!("safe{index:02}").into(),
                Value::Number(index as f64),
            );
        }
    }
    (runtime, doc, source, record)
}

fn allocation_contract(
    kind: Kind,
    length: usize,
    grow: bool,
    expandos: usize,
) -> (usize, usize, usize) {
    let (mut runtime, mut doc, _source, source_record) = fixture(kind, length, grow, expandos);
    let original = bytes(&mut runtime, source_record);
    let ctor = runtime.typed_arrays.constructors[kind.index()]
        .clone()
        .unwrap();
    runtime.steps = MAX_STEPS;
    runtime.allocated = 0;
    // This independent control calls only the existing constructor, never
    // toReversed or a candidate helper. It captures inherited allocation A.
    let target = runtime
        .typed_array_constructor(kind, &[Value::Number(length as f64)], ctor, &mut doc)
        .unwrap();
    let allocation_work = MAX_STEPS - runtime.steps;
    let allocation_heap = runtime.allocated;
    let start = runtime.steps;
    let target_record = runtime.typed_array_record(&target).unwrap().unwrap();
    let lookup = start - runtime.steps;
    assert_eq!(
        bytes(&mut runtime, target_record),
        vec![0; length * kind.width()]
    );
    assert_eq!(bytes(&mut runtime, source_record), original);
    assert_fixed(&mut runtime, target_record, kind, length);
    let count = length * kind.width();
    let zero_work = if count == 0 { 0 } else { 1 + count.div_ceil(8) };
    if !grow && expandos == 0 {
        assert_eq!(allocation_work, 107 + zero_work);
        let fields =
            72 + std::mem::size_of::<Option<AbortSlot>>() + std::mem::size_of::<Option<f64>>();
        assert_eq!(allocation_heap, 118 + 2 * fields + count);
        assert_eq!(lookup, 4); // Two genuine records, target is middle index1.
    }
    if grow {
        assert!(allocation_work > 107 + zero_work);
        assert!(allocation_heap > 326 + count);
    }
    clean(&runtime);
    (allocation_work, allocation_heap, lookup)
}

#[test]
fn every_number_kind_returns_a_distinct_fixed_same_kind_result_even_when_empty() {
    for kind in Kind::ALL {
        for length in [0, 1, 3, 4] {
            let (mut runtime, mut doc, source, record) = fixture(kind, length, false, 0);
            let original = bytes(&mut runtime, record);
            let source_buffer = buffer_value(&mut runtime, record);
            let function = saved(&runtime);
            let first = runtime
                .call(function.clone(), vec![], source.clone(), &mut doc)
                .unwrap();
            let target = result_record(&mut runtime, &first);
            assert_ne!(first, source);
            assert_fixed(&mut runtime, target, kind, length);
            assert_ne!(buffer_value(&mut runtime, target), source_buffer);
            assert_eq!(
                runtime.objects[target.object_id].prototype,
                Some(Value::Object(
                    runtime.typed_arrays.prototypes[kind.index()].unwrap()
                ))
            );
            assert_eq!(bytes(&mut runtime, target), prefix(kind, length, length));
            let second = runtime
                .call(function, vec![], source.clone(), &mut doc)
                .unwrap();
            let other = result_record(&mut runtime, &second);
            assert_ne!(first, second);
            assert_ne!(
                buffer_value(&mut runtime, target),
                buffer_value(&mut runtime, other)
            );
            if length != 0 {
                runtime.typed_array_write_number(target, 0, 0.0).unwrap();
                assert_eq!(bytes(&mut runtime, other), prefix(kind, length, length));
            }
            assert_eq!(bytes(&mut runtime, record), original);
            clean(&runtime);
        }
    }
}

#[test]
fn integer_extremes_clamping_and_source_values_have_independent_literals() {
    authored(
        r#"(function(){
        var kinds=[Int8Array,Uint8Array,Uint8ClampedArray,Int16Array,Uint16Array,Int32Array,Uint32Array];
        var lo=[-128,0,0,-32768,0,-2147483648,0],hi=[127,255,255,32767,65535,2147483647,4294967295];
        for(var i=0;i<kinds.length;i++){
            var a=new kinds[i]([lo[i],2,hi[i]]),b=a.toReversed();
            if(a===b||b[0]!==hi[i]||b[1]!==2||b[2]!==lo[i]||a[0]!==lo[i]||a[2]!==hi[i])return false;
        }
        var c=new Uint8ClampedArray([0,2,254,255]),d=c.toReversed();
        return d[0]===255&&d[1]===254&&d[2]===2&&d[3]===0&&c[0]===0&&c[3]===255;
    })()"#,
    );
}

fn raw_fixture(
    kind: Kind,
    payload: &[u8],
    offset: usize,
    length: usize,
) -> (Runtime, Document, Value, Record, Record) {
    let (mut runtime, mut doc) = fresh();
    runtime.execute(&format!("var b=new ArrayBuffer({});var raw=new Uint8Array(b);var source=new {}(b,{offset},{length});",payload.len(),kind.name()), &mut doc).unwrap();
    let source = runtime.lookup(1, "source").unwrap().1;
    let raw = runtime.lookup(1, "raw").unwrap().1;
    let record = runtime.typed_array_record(&source).unwrap().unwrap();
    let raw_record = runtime.typed_array_record(&raw).unwrap().unwrap();
    let backing = raw_record.backing.unwrap();
    for (index, chunk) in payload.chunks(8).enumerate() {
        runtime
            .buffer_view_write(backing.buffer, index * 8, chunk)
            .unwrap();
    }
    (runtime, doc, source, record, raw_record)
}

#[test]
fn aligned_source_sentinels_remain_exact_and_result_starts_at_zero() {
    for kind in Kind::ALL {
        let w = kind.width();
        let payload = [vec![165; w], literals(kind, &[1, 2, 3]), vec![90; w]].concat();
        let (mut runtime, mut doc, source, record, raw) = raw_fixture(kind, &payload, w, 3);
        let function = saved(&runtime);
        let result = runtime.call(function, vec![], source, &mut doc).unwrap();
        let target = result_record(&mut runtime, &result);
        assert_fixed(&mut runtime, target, kind, 3);
        assert_eq!(bytes(&mut runtime, target), literals(kind, &[3, 2, 1]));
        assert_eq!(bytes(&mut runtime, raw), payload);
        assert_eq!(bytes(&mut runtime, record), literals(kind, &[1, 2, 3]));
        clean(&runtime);
    }
}

type FloatRow = (
    Kind,
    &'static [u8],
    &'static [u8],
    &'static [u8],
    &'static [u8],
);

#[test]
fn every_float_index_is_numeric_including_nan_midpoint_while_source_payload_is_exact() {
    let rows: [FloatRow; 3] = [
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
    for (kind, negative_zero, signaling_nan, infinity, canonical_nan) in rows {
        for (payload, expected, length) in [
            (
                [negative_zero, signaling_nan, infinity].concat(),
                [infinity, canonical_nan, negative_zero].concat(),
                3,
            ),
            (
                [signaling_nan, negative_zero].concat(),
                [negative_zero, canonical_nan].concat(),
                2,
            ),
            (signaling_nan.to_vec(), canonical_nan.to_vec(), 1),
        ] {
            let (mut runtime, mut doc, source, _record, raw) =
                raw_fixture(kind, &payload, 0, length);
            let function = saved(&runtime);
            let result = runtime.call(function, vec![], source, &mut doc).unwrap();
            let target = result_record(&mut runtime, &result);
            assert_eq!(bytes(&mut runtime, target), expected);
            assert_eq!(bytes(&mut runtime, raw), payload);
            clean(&runtime);
        }
    }
}

#[test]
fn intrinsic_creation_ignores_poisoned_constructor_species_globals_and_prototypes() {
    authored(
        r#"(function(){
        var C=Uint16Array,proto=C.prototype,f=proto.toReversed,a=new C([1,2,3]),calls=0;
        function poison(){calls++;throw 1;}
        Object.defineProperty(a,'length',{get:poison});Object.defineProperty(a,'constructor',{get:poison});
        Object.defineProperty(C,Symbol.species,{get:poison});Object.defineProperty(C,'name',{get:poison});
        var p={};Object.defineProperty(p,'0',{get:poison,set:poison});Object.setPrototypeOf(a,p);
        proto.toReversed=poison;Uint16Array=poison;
        var r=f.call(a);
        return calls===0&&r!==a&&Object.getPrototypeOf(r)===proto&&ArrayBuffer.isView(r)&&r.length===3&&r[0]===3&&r[1]===2&&r[2]===1&&a[0]===1&&a[2]===3;
    })()"#,
    );
}

#[test]
fn ignored_arguments_evaluate_before_entry_but_values_are_never_coerced() {
    authored(
        r#"(function(){
        var f=Uint8Array.prototype.toReversed,b=new ArrayBuffer(4,{maxByteLength:8}),a=new Uint8Array(b),trace='',calls=0;
        a[0]=1;a[1]=2;a[2]=3;a[3]=4;
        var p={valueOf:function(){calls++;throw 1;},toString:function(){calls++;throw 2;}};
        p[Symbol.toPrimitive]=function(){calls++;throw 3;};
        function arg(){trace+='a';b.resize(2);for(var i=0;i<12;i++)new Uint8Array(1);return p;}
        var r=f.call(a,arg(),Symbol('unused'));
        if(trace!=='a'||calls!==0||r.length!==2||r[0]!==2||r[1]!==1||a[0]!==1||a[1]!==2)return false;
        var marker={};function abrupt(){trace+='x';throw marker;}
        try{f.call(a,abrupt());return false;}catch(e){if(e!==marker)return false;}
        function detach(){b.transfer();return p;}
        try{f.call(a,detach());return false;}catch(e){if(!(e instanceof TypeError))return false;}
        return trace==='ax'&&calls===0&&r[0]===2&&r[1]===1;
    })()"#,
    );
}

#[test]
fn genuine_invalid_receivers_detached_empty_and_oob_views_reject() {
    authored(
        r#"(function(){
        var f=Uint8Array.prototype.toReversed,ok=new Uint8Array(0);
        if(typeof f!=='function'||f.call(ok).length!==0)return false;
        var detached=new Uint8Array(0);detached.buffer.transfer();
        var b=new ArrayBuffer(4,{maxByteLength:8}),fixed=new Uint8Array(b,0,4),tracking=new Uint8Array(b,2);b.resize(1);
        var bad=[null,undefined,1,'x',{},[],new ArrayBuffer(0),new DataView(new ArrayBuffer(0)),Object.create(Uint8Array.prototype),detached,fixed,tracking];
        for(var i=0;i<bad.length;i++){try{f.call(bad[i]);return false;}catch(e){if(!(e instanceof TypeError))return false;}}
        b.resize(2);var r=f.call(tracking);
        return r!==tracking&&r.length===0&&r.buffer!==b&&!r.buffer.resizable;
    })()"#,
    );
}

#[test]
fn fixed_tracking_growth_recovery_and_partial_elements_copy_current_bounds_only() {
    authored(
        r#"(function(){
        var f=Uint8Array.prototype.toReversed,b=new ArrayBuffer(4,{maxByteLength:8}),fixed=new Uint8Array(b,0,4),a=new Uint8Array(b);
        a[0]=1;a[1]=2;a[2]=3;a[3]=4;b.resize(2);
        try{f.call(fixed);return false;}catch(e){if(!(e instanceof TypeError))return false;}
        var small=f.call(a);b.resize(6);var four=f.call(fixed),six=f.call(a);
        if(small.length!==2||small[0]!==2||small[1]!==1)return false;
        if(four.length!==4||four[0]!==0||four[1]!==0||four[2]!==2||four[3]!==1)return false;
        if(six.length!==6||six[0]!==0||six[3]!==0||six[4]!==2||six[5]!==1)return false;
        b.resize(0);if(four.length!==4||six.length!==6||small.length!==2)return false;
        var pb=new ArrayBuffer(8,{maxByteLength:8}),raw=new Uint8Array(pb),p=new Uint16Array(pb,2);
        for(var i=0;i<8;i++)raw[i]=i+1;pb.resize(5);var one=p.toReversed();
        if(one.length!==1||one[0]!==1027||one.byteOffset!==0||one.buffer.resizable)return false;
        pb.resize(8);var three=p.toReversed();
        return three.length===3&&three[0]===0&&three[1]===5&&three[2]===1027&&raw[2]===3&&raw[3]===4&&raw[4]===5;
    })()"#,
    );
}

#[test]
fn shared_metadata_saved_identity_and_nonconstructibility_are_exact() {
    authored(
        r#"(function(){
        var owner=Object.getPrototypeOf(Uint8Array.prototype),f=owner.toReversed;
        var kinds=[Int8Array,Uint8Array,Uint8ClampedArray,Int16Array,Uint16Array,Int32Array,Uint32Array,Float16Array,Float32Array,Float64Array];
        var guard=new Uint8Array([1,2]),g=f.call(guard);if(g[0]!==2||guard[0]!==1)return false;
        for(var i=0;i<kinds.length;i++)if(kinds[i].prototype.toReversed!==f)return false;
        var d=Object.getOwnPropertyDescriptor(owner,'toReversed'),n=Object.getOwnPropertyDescriptor(f,'name'),l=Object.getOwnPropertyDescriptor(f,'length');
        if(d.value!==f||!d.writable||d.enumerable||!d.configurable||n.value!=='toReversed'||n.writable||n.enumerable||!n.configurable||l.value!==0||l.writable||l.enumerable||!l.configurable)return false;
        if(Object.hasOwn(f,'prototype')||Object.getPrototypeOf(f)!==Function.prototype)return false;
        function C(){this.proof=7;}if(Reflect.construct(C,[]).proof!==7)return false;
        try{Reflect.construct(f,[]);return false;}catch(e){if(!(e instanceof TypeError))return false;}
        delete owner.toReversed;var r=f.call(guard);
        return owner.toReversed===undefined&&r!==guard&&r[0]===2&&r[1]===1&&guard[0]===1;
    })()"#,
    );
}

#[test]
fn preallocation_validation_cuts_create_no_result_or_backing() {
    for allowance in [0, 7, 8, 20, 21, 28, 29] {
        let (mut runtime, mut doc, _source, record) = fixture(Kind::Uint8, 3, false, 0);
        let before = (runtime.objects.len(), runtime.typed_arrays.records.len());
        runtime.steps = allowance;
        runtime.allocated = 0;
        assert!(
            runtime
                .typed_array_to_reversed(record, &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, 0);
        assert_eq!(
            (runtime.objects.len(), runtime.typed_arrays.records.len()),
            before
        );
        assert_eq!(bytes(&mut runtime, record), vec![1, 2, 3]);
        clean(&runtime);
    }
}

#[test]
fn independent_allocation_control_and_each_copy_endpoint_keep_only_complete_private_scalars() {
    for kind in Kind::ALL {
        let (allocation, heap, lookup) = allocation_contract(kind, 3, false, 0);
        let base = 46 + allocation + lookup;
        let (read, _write, element) = fees(kind);
        let mut cuts = vec![base - 1, base];
        for done in 0..3 {
            for local in [8, 8 + read, element] {
                cuts.extend([
                    base + done * element + local - 1,
                    base + done * element + local,
                ]);
            }
        }
        cuts.sort_unstable();
        cuts.dedup();
        for allowance in cuts {
            let (mut runtime, mut doc, _source, record) = fixture(kind, 3, false, 0);
            let old_records = runtime.typed_arrays.records.len();
            runtime.steps = allowance;
            runtime.allocated = MAX_HEAP - heap;
            let result = runtime.typed_array_to_reversed(record, &mut doc);
            if allowance == base + 3 * element {
                assert!(result.is_ok());
            } else {
                assert!(result.unwrap_err().is_resource_limit());
            }
            assert_eq!(runtime.typed_arrays.records.len(), old_records + 1);
            let target = *runtime.typed_arrays.records.last().unwrap();
            let stores = allowance.saturating_sub(base) / element;
            assert_eq!(bytes(&mut runtime, target), prefix(kind, 3, stores));
            assert_eq!(bytes(&mut runtime, record), literals(kind, &[1, 2, 3]));
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            clean(&runtime);
        }
    }
}

#[test]
fn inherited_allocation_work_boundaries_leave_source_exact_without_realm_rollback_claim() {
    for kind in Kind::ALL {
        let count = 3 * kind.width();
        let z = 1 + count.div_ceil(8);
        // Stable constructor sequence: native prototype key, typed shell,
        // None-record, buffer shell, zero block, buffer publication, backing.
        let mut cuts = vec![60, 61, 82, 85, 86, 87, 94, 95, 100, 101, 122, 123];
        cuts.extend([
            123 + z - 1,
            123 + z,
            124 + z,
            127 + z,
            128 + z,
            135 + z,
            136 + z,
        ]);
        cuts.sort_unstable();
        cuts.dedup();
        for allowance in cuts {
            let (mut runtime, mut doc, _source, record) = fixture(kind, 3, false, 0);
            let objects = runtime.objects.len();
            let records = runtime.typed_arrays.records.len();
            runtime.steps = allowance;
            runtime.allocated = 0;
            assert!(
                runtime
                    .typed_array_to_reversed(record, &mut doc)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(runtime.steps, 0);
            assert_eq!(
                runtime.objects.len(),
                objects + usize::from(allowance >= 86) + usize::from(allowance >= 123)
            );
            assert_eq!(
                runtime.typed_arrays.records.len(),
                records + usize::from(allowance >= 95)
            );
            if allowance >= 95 {
                let target = *runtime.typed_arrays.records.last().unwrap();
                assert_eq!(target.backing.is_some(), allowance >= 136 + z);
                if target.backing.is_some() {
                    assert_eq!(bytes(&mut runtime, target), vec![0; count]);
                }
            }
            assert_eq!(bytes(&mut runtime, record), literals(kind, &[1, 2, 3]));
            clean(&runtime);
        }
    }
}

#[test]
fn exact_heap_and_one_short_preserve_source_and_only_admitted_private_shells() {
    let fields = 72 + std::mem::size_of::<Option<AbortSlot>>() + std::mem::size_of::<Option<f64>>();
    for kind in Kind::ALL {
        for length in [0, 3] {
            let (_, heap, _) = allocation_contract(kind, length, false, 0);
            let mut rooms = vec![
                0,
                117,
                118,
                118 + fields - 1,
                118 + fields,
                118 + 2 * fields - 1,
                heap - 1,
                heap,
            ];
            rooms.sort_unstable();
            rooms.dedup();
            for room in rooms {
                let (mut runtime, mut doc, _source, record) = fixture(kind, length, false, 0);
                let source_bytes = bytes(&mut runtime, record);
                let objects = runtime.objects.len();
                let records = runtime.typed_arrays.records.len();
                runtime.steps = MAX_STEPS;
                runtime.allocated = MAX_HEAP - room;
                let result = runtime.typed_array_to_reversed(record, &mut doc);
                if room == heap {
                    let target = result_record(&mut runtime, &result.unwrap());
                    assert_eq!(bytes(&mut runtime, target), prefix(kind, length, length));
                    assert_eq!(runtime.allocated, MAX_HEAP);
                } else {
                    assert!(result.unwrap_err().is_resource_limit());
                    assert!(runtime.allocated > MAX_HEAP);
                    assert_eq!(
                        runtime.objects.len(),
                        objects
                            + usize::from(room >= 118 + fields)
                            + usize::from(room >= 118 + 2 * fields)
                    );
                    assert_eq!(
                        runtime.typed_arrays.records.len(),
                        records + usize::from(room >= 118 + fields)
                    );
                    if runtime.typed_arrays.records.len() > records {
                        assert!(
                            runtime
                                .typed_arrays
                                .records
                                .last()
                                .unwrap()
                                .backing
                                .is_none()
                        );
                    }
                }
                assert_eq!(bytes(&mut runtime, record), source_bytes);
                clean(&runtime);
            }
        }
    }
}

#[test]
fn genuine_record_growth_and_constructor_expandos_retain_inherited_charges() {
    for (grow, expandos) in [(true, 0), (false, 30), (true, 30)] {
        for kind in [Kind::Uint8, Kind::Float64] {
            let (allocation, heap, lookup) = allocation_contract(kind, 3, grow, expandos);
            let work = 46 + allocation + lookup + 3 * fees(kind).2;
            for (allowance, room) in [(work - 1, heap), (work, heap), (work, heap - 1)] {
                let (mut runtime, mut doc, _source, record) = fixture(kind, 3, grow, expandos);
                let old_capacity = runtime.typed_arrays.records.capacity();
                let old_records = runtime.typed_arrays.records.len();
                runtime.steps = allowance;
                runtime.allocated = MAX_HEAP - room;
                let result = runtime.typed_array_to_reversed(record, &mut doc);
                if allowance == work && room == heap {
                    assert!(result.is_ok());
                } else {
                    assert!(result.unwrap_err().is_resource_limit());
                }
                assert_eq!(runtime.typed_arrays.records.len(), old_records + 1);
                if grow {
                    assert_eq!(runtime.typed_arrays.records.capacity(), old_capacity * 2);
                }
                let target = *runtime.typed_arrays.records.last().unwrap();
                if room == heap {
                    assert_eq!(
                        bytes(&mut runtime, target),
                        prefix(kind, 3, if allowance == work { 3 } else { 2 })
                    );
                    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
                } else {
                    assert!(target.backing.is_none());
                    assert!(runtime.allocated > MAX_HEAP);
                }
                assert_eq!(bytes(&mut runtime, record), literals(kind, &[1, 2, 3]));
                clean(&runtime);
            }
        }
    }
}

#[test]
fn complete_saved_calls_include_name_allocation_and_keep_cleanup_on_terminal_failures() {
    for kind in Kind::ALL {
        for length in [0, 3] {
            let (allocation, allocation_heap, lookup) = allocation_contract(kind, length, false, 0);
            let body = 46 + allocation + lookup + length * fees(kind).2;
            let (mut measured, mut doc, source, source_record) = fixture(kind, length, false, 0);
            let function = saved(&measured);
            let before = (measured.steps, measured.allocated);
            let result = measured.call(function, vec![], source, &mut doc).unwrap();
            let work = before.0 - measured.steps;
            let heap = measured.allocated - before.1;
            assert!(work > body);
            assert_eq!(heap, allocation_heap + 53);
            let target = result_record(&mut measured, &result);
            assert_eq!(bytes(&mut measured, target), prefix(kind, length, length));
            assert_eq!(
                bytes(&mut measured, source_record),
                literals(kind, &(1..=length).collect::<Vec<_>>())
            );
            clean(&measured);
            for (allowance, room, success) in [
                (work, heap, true),
                (work - 1, heap, false),
                (0, heap, false),
                (work, heap - 1, false),
                (work, 52, false),
                (work, 0, false),
            ] {
                let (mut runtime, mut doc, source, record) = fixture(kind, length, false, 0);
                let function = saved(&runtime);
                let bag = runtime.property_object(&function).unwrap();
                let metadata = format!("{:?}", runtime.objects[bag].values);
                let old_records = runtime.typed_arrays.records.len();
                let source_bytes = bytes(&mut runtime, record);
                runtime.steps = allowance;
                runtime.allocated = MAX_HEAP - room;
                let result = runtime.call(function, vec![], source, &mut doc);
                if success {
                    let target = result_record(&mut runtime, &result.unwrap());
                    assert_eq!(bytes(&mut runtime, target), prefix(kind, length, length));
                    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
                } else {
                    assert!(result.unwrap_err().is_resource_limit());
                    if allowance == work - 1 && room == heap {
                        let target = *runtime.typed_arrays.records.last().unwrap();
                        assert_eq!(
                            bytes(&mut runtime, target),
                            prefix(kind, length, length.saturating_sub(1))
                        );
                    }
                    if room < 53 || allowance == 0 {
                        assert_eq!(runtime.typed_arrays.records.len(), old_records);
                    }
                }
                assert_eq!(bytes(&mut runtime, record), source_bytes);
                assert_eq!(format!("{:?}", runtime.objects[bag].values), metadata);
                clean(&runtime);
            }
        }
    }
}
