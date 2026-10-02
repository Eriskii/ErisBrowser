use super::*;

const LOCAL: &str = include_str!("../../../tests/conformance/data-view-local.js");
const LITERALS: &str = include_str!("../../../tests/conformance/data-view-codec-vectors.tsv");

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

fn helpers(runtime: &mut Runtime, doc: &mut Document) {
    for source in [
        include_str!("../../../tests/upstream/test262/harness/sta.js"),
        include_str!("../../../tests/upstream/test262/harness/assert.js"),
        include_str!("../../../tests/upstream/test262/harness/propertyHelper.js"),
    ] {
        runtime.execute(source, doc).unwrap();
    }
}

fn check(source: &str) {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        helpers(&mut runtime, &mut doc);
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        assert!(result.is_ok(), "strict={strict}: {result:?}");
        clean(&runtime);
    }
}

fn make_buffer(
    runtime: &mut Runtime,
    doc: &mut Document,
    bytes: &[u8],
    maximum: Option<usize>,
) -> (Value, BufferId) {
    let mut arguments = vec![Value::Number(bytes.len() as f64)];
    if let Some(maximum) = maximum {
        arguments.push(
            runtime
                .object_ordered([("maxByteLength".into(), Value::Number(maximum as f64))])
                .unwrap(),
        );
    }
    let value = runtime
        .array_buffer_constructor(
            &arguments,
            Runtime::native("ArrayBuffer", Value::Window),
            doc,
        )
        .unwrap();
    let id = runtime.buffer_for_view(&value).unwrap();
    for (index, chunk) in bytes.chunks(8).enumerate() {
        runtime.buffer_view_write(id, index * 8, chunk).unwrap();
    }
    (value, id)
}

fn make_view(
    runtime: &mut Runtime,
    doc: &mut Document,
    buffer: Value,
    offset: usize,
    length: Option<usize>,
) -> Value {
    let mut arguments = vec![buffer, Value::Number(offset as f64)];
    if let Some(length) = length {
        arguments.push(Value::Number(length as f64));
    }
    runtime
        .data_view_constructor(&arguments, Runtime::native("DataView", Value::Window), doc)
        .unwrap()
}

fn bytes(runtime: &mut Runtime, buffer: BufferId, length: usize) -> Vec<u8> {
    // Private observation after a measured refusal; this is not an author
    // continuation and never masks the recorded result or terminal cleanup.
    runtime.steps = MAX_STEPS;
    let mut result = Vec::new();
    for offset in (0..length).step_by(8) {
        let size = (length - offset).min(8);
        result.extend_from_slice(&runtime.buffer_view_read(buffer, offset, size).unwrap()[..size]);
    }
    result
}

fn literal_bytes(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0);
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

fn assert_number(number: f64, expected: &str) {
    if expected == "NaN" {
        assert!(number.is_nan());
    } else {
        assert_eq!(number.to_bits(), u64::from_str_radix(expected, 16).unwrap());
    }
}

#[test]
fn data_view_frozen_local_success_sources_both_modes() {
    let mut total = 0;
    let mut successes = 0;
    let mut prerequisites = 0;
    let mut resources = 0;
    for part in LOCAL.split("// CASE: ").skip(1) {
        let (name, source) = part.split_once('\n').unwrap();
        total += 1;
        if name.starts_with("prerequisite-") {
            prerequisites += 1;
            continue;
        }
        if matches!(
            name,
            "recursive-index-conversion-terminal-resource"
                | "repeated-zero-byte-view-metadata-terminal-resource"
        ) {
            resources += 1;
            continue;
        }
        check(source);
        successes += 1;
    }
    // Explicit unsupported prerequisites/policy rows remain in the external
    // runner's inventory; this group does not reinterpret their expectations.
    assert_eq!((total, successes, prerequisites, resources), (69, 62, 5, 2));
}

#[test]
fn data_view_frozen_terminal_policy_sources_both_modes() {
    let mut count = 0;
    for part in LOCAL.split("// CASE: ").skip(1) {
        let (name, source) = part.split_once('\n').unwrap();
        if !matches!(
            name,
            "recursive-index-conversion-terminal-resource"
                | "repeated-zero-byte-view-metadata-terminal-resource"
        ) {
            continue;
        }
        for strict in [false, true] {
            let (mut runtime, mut doc) = fresh();
            helpers(&mut runtime, &mut doc);
            let result = if strict {
                runtime.execute_strict(source, &mut doc)
            } else {
                runtime.execute(source, &mut doc)
            };
            assert!(
                result.unwrap_err().is_resource_limit(),
                "{name}, strict={strict}"
            );
            clean(&runtime);
            count += 1;
        }
    }
    assert_eq!(count, 4);
}

#[test]
fn data_view_independent_nine_codec_literals_both_endians() {
    let mut total = 0;
    let mut decode_only = 0;
    let mut seen = [false; 9];
    for line in LITERALS
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
    {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 5);
        let codec = Codec::named(fields[1]).unwrap();
        seen[Codec::NAMES
            .iter()
            .position(|name| *name == fields[1])
            .unwrap()] = true;
        let width = codec.width();
        for little in [false, true] {
            let supplied = if fields[3] == "NaN" {
                None
            } else {
                Some(literal_bytes(fields[3]))
            };
            let mut input = [0; 8];
            if fields[2] != "-" {
                let value = f64::from_bits(u64::from_str_radix(fields[2], 16).unwrap());
                input = codec.encode(value, little);
                if let Some(mut expected) = supplied.clone() {
                    assert_eq!(expected.len(), width);
                    if little {
                        expected.reverse();
                    }
                    assert_eq!(&input[..width], expected, "{} little={little}", fields[0]);
                } else {
                    assert!(codec.decode(input, little).is_nan());
                    assert_eq!(input, codec.encode(value, little));
                }
            } else {
                let mut supplied = supplied.unwrap();
                assert_eq!(supplied.len(), width);
                if little {
                    supplied.reverse();
                }
                input[..width].copy_from_slice(&supplied);
            }
            assert_number(codec.decode(input, little), fields[4]);
        }
        total += 1;
        decode_only += usize::from(fields[2] == "-");
    }
    assert_eq!((total, decode_only), (119, 19));
    assert!(seen.into_iter().all(|value| value));
}

#[test]
fn data_view_literal_native_dispatch_preserves_unaligned_sentinels() {
    let (mut runtime, mut doc) = fresh();
    let (buffer, id) = make_buffer(&mut runtime, &mut doc, &[0xa5; 12], None);
    let view = make_view(&mut runtime, &mut doc, buffer, 1, Some(10));
    for line in LITERALS
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
    {
        let fields: Vec<_> = line.split('\t').collect();
        let codec = Codec::named(fields[1]).unwrap();
        let width = codec.width();
        for little in [false, true] {
            runtime.steps = MAX_STEPS;
            for at in 0..12 {
                runtime.buffer_view_write(id, at, &[0xa5]).unwrap();
            }
            if fields[2] != "-" {
                let value = f64::from_bits(u64::from_str_radix(fields[2], 16).unwrap());
                let result = runtime
                    .data_view_native(
                        &format!("set{}", fields[1]),
                        view.clone(),
                        &[
                            Value::Number(1.0),
                            Value::Number(value),
                            Value::Bool(little),
                        ],
                        &mut doc,
                    )
                    .unwrap();
                assert_eq!(result, Value::Undefined);
            } else {
                let mut literal = literal_bytes(fields[3]);
                if little {
                    literal.reverse();
                }
                runtime.buffer_view_write(id, 2, &literal).unwrap();
            }
            let Value::Number(number) = runtime
                .data_view_native(
                    &format!("get{}", fields[1]),
                    view.clone(),
                    &[Value::Number(1.0), Value::Bool(little)],
                    &mut doc,
                )
                .unwrap()
            else {
                panic!()
            };
            assert_number(number, fields[4]);
            let actual = bytes(&mut runtime, id, 12);
            assert_eq!(&actual[..2], &[0xa5; 2]);
            assert!(actual[2 + width..].iter().all(|byte| *byte == 0xa5));
            if fields[3] != "NaN" {
                let mut expected = literal_bytes(fields[3]);
                if little {
                    expected.reverse();
                }
                assert_eq!(&actual[2..2 + width], expected);
            }
        }
    }
    clean(&runtime);
}

// Mathematical value formula, independent of the production bit-field decode.
fn half_value(bits: u16) -> f64 {
    let exponent = (bits >> 10) & 31;
    let fraction = bits & 1023;
    let magnitude = if exponent == 0 {
        f64::from(fraction) * 2.0f64.powi(-24)
    } else {
        f64::from(1024 + fraction) * 2.0f64.powi(i32::from(exponent) - 25)
    };
    if bits & 0x8000 != 0 {
        -magnitude
    } else {
        magnitude
    }
}

#[test]
fn data_view_all_half_encodings_against_exact_value_formula() {
    for bits in 0..=u16::MAX {
        let decoded = decode_f16(bits);
        if bits & 0x7c00 == 0x7c00 {
            if bits & 1023 == 0 {
                assert_eq!(
                    decoded,
                    if bits & 0x8000 == 0 {
                        f64::INFINITY
                    } else {
                        f64::NEG_INFINITY
                    }
                );
                assert_eq!(encode_f16(decoded), bits);
            } else {
                assert!(decoded.is_nan());
                assert_eq!(encode_f16(decoded), 0x7e00);
            }
        } else {
            let expected = half_value(bits);
            assert_eq!(decoded.to_bits(), expected.to_bits(), "{bits:04x}");
            assert_eq!(encode_f16(expected), bits, "{bits:04x}");
        }
    }
}

#[test]
fn data_view_half_all_finite_midpoints_and_binary64_neighbours() {
    for lower in 0..0x7bffu16 {
        let a = half_value(lower);
        let b = half_value(lower + 1);
        let midpoint = a + (b - a) / 2.0;
        let tie = if lower & 1 == 0 { lower } else { lower + 1 };
        for (value, expected) in [
            (f64::from_bits(midpoint.to_bits() - 1), lower),
            (midpoint, tie),
            (f64::from_bits(midpoint.to_bits() + 1), lower + 1),
        ] {
            assert_eq!(encode_f16(value), expected, "lower={lower:04x}");
            assert_eq!(encode_f16(-value), expected | 0x8000, "lower={lower:04x}");
        }
    }
    for (value, expected) in [
        (f64::from_bits(65520.0f64.to_bits() - 1), 0x7bff),
        (65520.0, 0x7c00),
        (f64::from_bits(65520.0f64.to_bits() + 1), 0x7c00),
        (f64::MAX, 0x7c00),
    ] {
        assert_eq!(encode_f16(value), expected);
        assert_eq!(encode_f16(-value), expected | 0x8000);
    }
}

#[test]
fn data_view_explicit_nan_policy_for_all_float_codecs() {
    for (codec, expected) in [
        (Codec::Float16, "7e00"),
        (Codec::Float32, "7fc00000"),
        (Codec::Float64, "7ff8000000000000"),
    ] {
        for bits in [
            0x7ff0_0000_0000_0001,
            0x7ff8_0000_0000_0000,
            0x7fff_ffff_ffff_ffff,
            0xfff8_0000_0000_0001,
        ] {
            assert_eq!(
                &codec.encode(f64::from_bits(bits), false)[..codec.width()],
                literal_bytes(expected)
            );
        }
    }
}

#[test]
fn data_view_backing_helpers_reject_invalid_span_before_any_write() {
    let (mut runtime, mut doc) = fresh();
    let (_, buffer) = make_buffer(&mut runtime, &mut doc, &[7; 8], None);
    for (offset, data) in [
        (0, vec![]),
        (0, vec![1; 9]),
        (usize::MAX, vec![1]),
        (7, vec![1; 2]),
    ] {
        assert!(runtime.buffer_view_write(buffer, offset, &data).is_err());
        assert_eq!(bytes(&mut runtime, buffer, 8), [7; 8]);
    }
    for (offset, length) in [(0, 0), (0, 9), (usize::MAX, 1), (7, 2)] {
        assert!(runtime.buffer_view_read(buffer, offset, length).is_err());
    }
}

#[test]
fn data_view_backing_write_exact_and_one_short_are_atomic() {
    for length in 1..=8 {
        let (mut runtime, mut doc) = fresh();
        let (_, buffer) = make_buffer(&mut runtime, &mut doc, &[9; 8], None);
        runtime.steps = length;
        assert!(
            runtime
                .buffer_view_write(buffer, 0, &vec![3; length])
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(bytes(&mut runtime, buffer, 8), [9; 8]);
        runtime.steps = length + 1;
        runtime
            .buffer_view_write(buffer, 0, &vec![3; length])
            .unwrap();
        assert_eq!(runtime.steps, 0);
        let actual = bytes(&mut runtime, buffer, 8);
        assert!(actual[..length].iter().all(|byte| *byte == 3));
        assert!(actual[length..].iter().all(|byte| *byte == 9));
    }
}

#[test]
fn data_view_zero_byte_record_reserve_exact_and_one_short() {
    let (mut measure, _) = fresh();
    let before = measure.allocated;
    let steps = measure.steps;
    measure.data_view_reserve_record().unwrap();
    let storage = measure.allocated - before;
    let work = steps - measure.steps;
    assert!(storage >= 4 * std::mem::size_of::<Record>());
    for (remaining, allowed) in [(storage - 1, false), (storage, true)] {
        let (mut runtime, _) = fresh();
        runtime.allocated = MAX_HEAP - remaining;
        let result = runtime.data_view_reserve_record();
        assert_eq!(result.is_ok(), allowed);
        assert!(runtime.data_views.records.is_empty());
        if allowed {
            assert!(runtime.data_views.records.capacity() >= 4);
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            assert_eq!(runtime.data_views.records.capacity(), 0);
        }
    }
    let (mut runtime, _) = fresh();
    let before = runtime.allocated;
    runtime.steps = work - 1;
    assert!(
        runtime
            .data_view_reserve_record()
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.allocated, before);
    assert_eq!(runtime.data_views.records.capacity(), 0);
}

#[test]
fn data_view_constructor_final_record_heap_cut_leaves_no_brand() {
    fn setup() -> (Runtime, Document, Value) {
        let (mut runtime, mut doc) = fresh();
        let (buffer, _) = make_buffer(&mut runtime, &mut doc, &[], None);
        (runtime, doc, buffer)
    }
    let (mut measure, mut doc, buffer) = setup();
    let before = measure.allocated;
    make_view(&mut measure, &mut doc, buffer, 0, None);
    let storage = measure.allocated - before;
    let (mut runtime, mut doc, buffer) = setup();
    let objects = runtime.objects.len();
    runtime.allocated = MAX_HEAP - (storage - 1);
    assert!(
        runtime
            .data_view_constructor(
                &[buffer, Value::Number(0.0)],
                Runtime::native("DataView", Value::Window),
                &mut doc
            )
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.objects.len(), objects + 1);
    assert!(runtime.data_views.records.is_empty());
    assert_eq!(runtime.data_views.records.capacity(), 0);
    clean(&runtime);
}

#[test]
fn data_view_nonempty_record_growth_prepays_move_and_full_block() {
    let (mut runtime, mut doc) = fresh();
    let (buffer, _) = make_buffer(&mut runtime, &mut doc, &[], None);
    make_view(&mut runtime, &mut doc, buffer.clone(), 0, None);
    while runtime.data_views.records.len() < runtime.data_views.records.capacity() {
        make_view(&mut runtime, &mut doc, buffer.clone(), 0, None);
    }
    let length = runtime.data_views.records.len();
    let capacity = runtime.data_views.records.capacity();
    let work = 1 + (length * std::mem::size_of::<Record>()).div_ceil(8) * 2;
    let storage = capacity * 2 * std::mem::size_of::<Record>();
    let before = runtime.allocated;
    runtime.steps = work - 1;
    assert!(
        runtime
            .data_view_reserve_record()
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.allocated, before);
    assert_eq!(runtime.data_views.records.len(), length);
    assert_eq!(runtime.data_views.records.capacity(), capacity);
    runtime.steps = work;
    runtime.allocated = MAX_HEAP - (storage - 1);
    assert!(
        runtime
            .data_view_reserve_record()
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.data_views.records.capacity(), capacity);
    runtime.steps = work;
    runtime.allocated = MAX_HEAP - storage;
    runtime.data_view_reserve_record().unwrap();
    assert_eq!(runtime.steps, 0);
    assert_eq!(runtime.allocated, MAX_HEAP);
    assert!(runtime.data_views.records.capacity() >= 2 * capacity);
}

#[test]
fn data_view_byte_access_never_allocates_or_scans_other_bytes() {
    let (mut runtime, mut doc) = fresh();
    let (buffer, id) = make_buffer(&mut runtime, &mut doc, &[0; 4096], None);
    let view = make_view(&mut runtime, &mut doc, buffer, 4090, Some(6));
    let before = runtime.allocated;
    runtime.steps = 100;
    runtime
        .data_view_native(
            "setUint32",
            view.clone(),
            &[Value::Number(1.0), Value::Number(0x12345678 as f64)],
            &mut doc,
        )
        .unwrap();
    assert!(runtime.steps > 0);
    assert_eq!(runtime.allocated, before);
    runtime.steps = 100;
    assert_eq!(
        runtime
            .data_view_native("getUint32", view, &[Value::Number(1.0)], &mut doc)
            .unwrap(),
        Value::Number(0x12345678 as f64)
    );
    assert_eq!(runtime.allocated, before);
    runtime.steps = MAX_STEPS;
    assert_eq!(
        &runtime.buffer_view_read(id, 4090, 6).unwrap()[..6],
        &[0, 0x12, 0x34, 0x56, 0x78, 0]
    );
}

#[test]
fn data_view_isview_empty_fast_path_and_nonempty_search_cut() {
    let (mut runtime, mut doc) = fresh();
    let (buffer, _) = make_buffer(&mut runtime, &mut doc, &[], None);
    runtime.steps = 1;
    assert_eq!(
        runtime
            .array_buffer_native(
                "isView",
                Value::Undefined,
                std::slice::from_ref(&buffer),
                &mut doc
            )
            .unwrap(),
        Value::Bool(false)
    );
    assert_eq!(runtime.steps, 0);
    runtime.steps = MAX_STEPS;
    let mut views = Vec::new();
    for _ in 0..65 {
        views.push(make_view(&mut runtime, &mut doc, buffer.clone(), 0, None));
    }
    let before = runtime.steps;
    assert!(runtime.data_view_is_view(&views[64]).unwrap());
    let work = before - runtime.steps;
    assert!((1..=7).contains(&work));
    runtime.steps = work - 1;
    assert!(
        runtime
            .data_view_is_view(&views[64])
            .unwrap_err()
            .is_resource_limit()
    );
    runtime.steps = work;
    assert!(runtime.data_view_is_view(&views[64]).unwrap());
    assert_eq!(runtime.steps, 0);
}

#[test]
fn data_view_all_setters_measured_final_byte_cut_preserves_entire_span() {
    for suffix in Codec::NAMES {
        fn setup() -> (Runtime, Document, Value, BufferId) {
            let (mut runtime, mut doc) = fresh();
            let (buffer, id) = make_buffer(&mut runtime, &mut doc, &[9; 12], None);
            let view = make_view(&mut runtime, &mut doc, buffer, 1, Some(10));
            (runtime, doc, view, id)
        }
        let method = format!("set{suffix}");
        let args = [Value::Number(1.0), Value::Number(1.5)];
        let (mut measure, mut doc, view, _) = setup();
        let before = measure.steps;
        measure
            .data_view_native(&method, view, &args, &mut doc)
            .unwrap();
        let work = before - measure.steps;
        let (mut runtime, mut doc, view, buffer) = setup();
        runtime.steps = work - 1;
        assert!(
            runtime
                .data_view_native(&method, view, &args, &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(bytes(&mut runtime, buffer, 12), [9; 12]);
        clean(&runtime);
    }
}

#[test]
fn data_view_terminal_write_cut_preserves_prior_author_mutation() {
    fn setup() -> (Runtime, Document, Value, Value, BufferId) {
        let (mut runtime, mut doc) = fresh();
        runtime.execute("var b=new ArrayBuffer(8),v=new DataView(b),effects=0;for(var i=0;i<8;i++)v.setUint8(i,9);var number={valueOf:function(){effects++;v.setUint8(0,77);return 1.5;}};", &mut doc).unwrap();
        let view = runtime.lookup(1, "v").unwrap().1;
        let number = runtime.lookup(1, "number").unwrap().1;
        let value = runtime.lookup(1, "b").unwrap().1;
        let buffer = runtime.buffer_for_view(&value).unwrap();
        (runtime, doc, view, number, buffer)
    }
    let (mut measure, mut doc, view, number, _) = setup();
    let before = measure.steps;
    measure
        .data_view_native("setFloat64", view, &[Value::Number(0.0), number], &mut doc)
        .unwrap();
    let work = before - measure.steps;
    let (mut runtime, mut doc, view, number, buffer) = setup();
    runtime.steps = work - 1;
    assert!(
        runtime
            .data_view_native("setFloat64", view, &[Value::Number(0.0), number], &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    clean(&runtime);
    assert_eq!(bytes(&mut runtime, buffer, 8), [77, 9, 9, 9, 9, 9, 9, 9]);
    assert_eq!(runtime.lookup(1, "effects").unwrap().1, Value::Number(1.0));
}

#[test]
fn data_view_callback_tables_remain_sorted_and_backing_identity_survives() {
    let (mut runtime, mut doc) = fresh();
    helpers(&mut runtime, &mut doc);
    runtime.execute(r#"
        var b=new ArrayBuffer(4,{maxByteLength:8}),v=new DataView(b),old=v.buffer;
        var index={valueOf:function(){for(var i=0;i<80;i++){var x=new ArrayBuffer(1);new DataView(x);}b.resize(2);b.resize(4);return 1;}};
        v.setUint16(index,0x1234);assert.sameValue(v.getUint8(1),0x12);assert.sameValue(v.getUint8(2),0x34);
        assert.sameValue(v.buffer,old);
    "#, &mut doc).unwrap();
    assert_eq!(runtime.data_views.records.len(), 81);
    assert!(
        runtime
            .data_views
            .records
            .windows(2)
            .all(|pair| pair[0].object_id < pair[1].object_id)
    );
    clean(&runtime);
}

#[test]
fn data_view_getters_and_brand_do_not_touch_detached_or_oob_payload() {
    check(
        r#"
        var b=new ArrayBuffer(4,{maxByteLength:8}),v=new DataView(b,2,2);
        b.resize(1);assert.sameValue(v.buffer,b);assert.sameValue(ArrayBuffer.isView(v),true);
        assert.throws(TypeError,function(){return v.byteOffset;});assert.throws(TypeError,function(){return v.byteLength;});
        b.resize(4);assert.sameValue(v.byteOffset,2);assert.sameValue(v.byteLength,2);
        var moved=b.transfer();assert.sameValue(v.buffer,b);assert.sameValue(ArrayBuffer.isView(v),true);
        assert.sameValue(ArrayBuffer.isView(new DataView(moved)),true);
        assert.throws(TypeError,function(){return v.byteLength;});assert.throws(TypeError,function(){v.getUint8(0);});
    "#,
    );
}

#[test]
fn data_view_constructor_precedence_before_prototype_and_final_recheck() {
    check(
        r#"
        var b=new ArrayBuffer(4,{maxByteLength:8}),log='';function C(){}var T=C.bind(null);
        Object.defineProperty(T,'prototype',{get:function(){log+='P';throw 'prototype';}});
        var length={valueOf:function(){log+='L';b.transfer();return 1;}};
        var caught;try{Reflect.construct(DataView,[b,0,length],T);}catch(e){caught=e;}
        assert.sameValue(caught,'prototype');assert.sameValue(log,'LP');
        b=new ArrayBuffer(4,{maxByteLength:8});log='';
        length={valueOf:function(){log+='L';b.resize(8);return 5;}};
        assert.throws(RangeError,function(){Reflect.construct(DataView,[b,0,length],T);});assert.sameValue(log,'L');
    "#,
    );
}

#[test]
fn data_view_oob_typeerror_after_conversions_before_element_range() {
    check(
        r#"
        var b=new ArrayBuffer(4,{maxByteLength:8}),v=new DataView(b,1,3),log='';b.resize(1);
        var index={valueOf:function(){log+='I';return 99;}},value={valueOf:function(){log+='V';return 7;}};
        assert.throws(TypeError,function(){v.setUint32(index,value);});assert.sameValue(log,'IV');
        b.resize(4);log='';assert.throws(RangeError,function(){v.setUint32(index,value);});assert.sameValue(log,'IV');
    "#,
    );
}

#[test]
fn data_view_zero_length_metadata_reuses_backing_without_byte_storage() {
    let (mut runtime, mut doc) = fresh();
    let (buffer, id) = make_buffer(&mut runtime, &mut doc, &[7; 4096], Some(8192));
    let before = runtime.allocated;
    for _ in 0..8 {
        make_view(&mut runtime, &mut doc, buffer.clone(), 4096, Some(0));
    }
    assert!(runtime.allocated - before < 8192);
    assert!(
        runtime
            .data_views
            .records
            .iter()
            .all(|record| record.buffer == id)
    );
    assert_eq!(
        runtime.buffer_view_metadata(id).unwrap().byte_length,
        Some(4096)
    );
}

#[test]
fn data_view_installed_order_descriptors_and_registry_identity() {
    let (runtime, _) = fresh();
    let owner = runtime.prototypes["DataView"];
    let mut expected = vec![
        PropertyKey::from("constructor"),
        PropertyKey::from("buffer"),
        PropertyKey::from("byteLength"),
        PropertyKey::from("byteOffset"),
    ];
    for suffix in Codec::NAMES {
        for prefix in ["get", "set"] {
            expected.push(PropertyKey::from(format!("{prefix}{suffix}")));
        }
    }
    expected.push(runtime.well_known_key("toStringTag"));
    assert_eq!(runtime.objects[owner].order, expected);
    assert_eq!(runtime.objects[owner].values.len(), 23);
    assert!(runtime.objects[owner].order.capacity() >= 23);
    for (key, full, display, length, getter) in [
        ("buffer", "DataView.getBuffer", "get buffer", 0, true),
        ("getFloat16", "DataView.getFloat16", "getFloat16", 1, false),
        ("setFloat64", "DataView.setFloat64", "setFloat64", 2, false),
    ] {
        let id = runtime.native_properties[full];
        let bag = &runtime.objects[id];
        assert_eq!(
            bag.prototype,
            Some(Value::Function(runtime.function_prototype))
        );
        assert_eq!(
            bag.order,
            [PropertyKey::from("name"), PropertyKey::from("length")]
        );
        for (name, expected_value) in [
            ("name", Value::String(display.into())),
            ("length", Value::Number(length as f64)),
        ] {
            let property = &bag.values[&PropertyKey::from(name)];
            assert!(!property.enumerable && property.configurable);
            let PropertyValue::Data { value, writable } = &property.value else {
                panic!()
            };
            assert!(!writable);
            assert_eq!(value, &expected_value);
        }
        let property = &runtime.objects[owner].values[&PropertyKey::from(key)];
        assert!(!property.enumerable && property.configurable);
        let function = if getter {
            let PropertyValue::Accessor { get, set } = &property.value else {
                panic!()
            };
            assert_eq!(set, &Value::Undefined);
            get
        } else {
            let PropertyValue::Data { value, writable } = &property.value else {
                panic!()
            };
            assert!(*writable);
            value
        };
        let Value::Native(native) = function else {
            panic!()
        };
        assert_eq!(native.name, full);
        assert_eq!(runtime.native_properties[&native.name], id);
    }
}

#[test]
fn data_view_intrinsic_order_reserve_exact_and_one_short() {
    fn setup() -> (Runtime, usize, usize) {
        let (mut runtime, _) = fresh();
        let Value::Object(owner) = runtime
            .object_ordered([("kept".into(), Value::Number(7.0))])
            .unwrap()
        else {
            panic!()
        };
        let target = runtime.objects[owner].order.capacity() + 5;
        (runtime, owner, target)
    }
    let (mut measure, owner, target) = setup();
    let before_work = measure.steps;
    let before_heap = measure.allocated;
    measure.data_view_reserve_order(owner, target).unwrap();
    let work = before_work - measure.steps;
    let storage = measure.allocated - before_heap;
    assert_eq!(storage, target * std::mem::size_of::<PropertyKey>());
    for (available_work, available_heap, succeeds) in [
        (work - 1, storage, false),
        (work, storage - 1, false),
        (work, storage, true),
    ] {
        let (mut runtime, owner, target) = setup();
        let capacity = runtime.objects[owner].order.capacity();
        let order = runtime.objects[owner].order.clone();
        runtime.steps = available_work;
        runtime.allocated = MAX_HEAP - available_heap;
        let result = runtime.data_view_reserve_order(owner, target);
        assert_eq!(result.is_ok(), succeeds);
        assert_eq!(runtime.objects[owner].order, order);
        if succeeds {
            assert!(runtime.objects[owner].order.capacity() >= target);
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            assert_eq!(runtime.objects[owner].order.capacity(), capacity);
        }
    }
}

fn installer_setup() -> (Runtime, usize) {
    let (mut runtime, _) = fresh();
    let Value::Object(owner) = runtime.object_ordered([]).unwrap() else {
        panic!()
    };
    runtime.data_view_reserve_order(owner, 3).unwrap();
    (runtime, owner)
}

#[test]
fn data_view_intrinsic_function_admission_exact_and_one_short() {
    let (mut measure, owner) = installer_setup();
    let before_work = measure.steps;
    let before_heap = measure.allocated;
    measure
        .data_view_install_function(owner, "DataView.testBudget", "budget", 3, "budget", false)
        .unwrap();
    let work = before_work - measure.steps;
    let storage = measure.allocated - before_heap;
    println!("DATAVIEW_INSTALL function_work={work} function_heap={storage}");
    for (available_work, available_heap, succeeds) in [
        (work - 1, storage, false),
        (work, storage - 1, false),
        (work, storage, true),
    ] {
        let (mut runtime, owner) = installer_setup();
        let registry_count = runtime.native_properties.len();
        runtime.steps = available_work;
        runtime.allocated = MAX_HEAP - available_heap;
        let result = runtime.data_view_install_function(
            owner,
            "DataView.testBudget",
            "budget",
            3,
            "budget",
            false,
        );
        assert_eq!(result.is_ok(), succeeds);
        if succeeds {
            assert_eq!(runtime.steps, 0);
            assert_eq!(runtime.allocated, MAX_HEAP);
            assert_eq!(runtime.native_properties.len(), registry_count + 1);
            assert_eq!(runtime.objects[owner].order, [PropertyKey::from("budget")]);
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            assert_eq!(runtime.native_properties.len(), registry_count);
            assert!(
                !runtime
                    .native_properties
                    .contains_key("DataView.testBudget")
            );
            assert!(runtime.objects[owner].values.is_empty());
            assert!(runtime.objects[owner].order.is_empty());
        }
    }
}

#[test]
fn data_view_intrinsic_duplicates_never_replace_or_partially_publish() {
    let (mut runtime, owner) = installer_setup();
    runtime
        .data_view_install_function(owner, "DataView.testBudget", "budget", 3, "budget", false)
        .unwrap();
    let id = runtime.native_properties["DataView.testBudget"];
    let registry_count = runtime.native_properties.len();
    let value = runtime.objects[owner].get("budget").unwrap().clone();
    for (full, key) in [
        ("DataView.testBudget", "budget"),
        ("DataView.testNew", "budget"),
        ("DataView.testBudget", "new"),
    ] {
        assert!(
            runtime
                .data_view_install_function(owner, full, "changed", 99, key, true)
                .is_err()
        );
        assert_eq!(runtime.native_properties["DataView.testBudget"], id);
        assert_eq!(runtime.native_properties.len(), registry_count);
        assert!(!runtime.native_properties.contains_key("DataView.testNew"));
        assert_eq!(runtime.objects[owner].order, [PropertyKey::from("budget")]);
        assert_eq!(runtime.objects[owner].get("budget"), Some(&value));
        assert!(!runtime.objects[owner].contains_key("new"));
    }
}

#[test]
fn data_view_intrinsic_tag_admission_and_duplicate_keep_order() {
    let (mut measure, owner) = installer_setup();
    let before_work = measure.steps;
    let before_heap = measure.allocated;
    measure.data_view_install_tag(owner).unwrap();
    let work = before_work - measure.steps;
    let storage = measure.allocated - before_heap;
    for (available_work, available_heap, succeeds) in [
        (work - 1, storage, false),
        (work, storage - 1, false),
        (work, storage, true),
    ] {
        let (mut runtime, owner) = installer_setup();
        runtime.steps = available_work;
        runtime.allocated = MAX_HEAP - available_heap;
        let result = runtime.data_view_install_tag(owner);
        assert_eq!(result.is_ok(), succeeds);
        if succeeds {
            assert_eq!(
                runtime.objects[owner].order,
                [runtime.well_known_key("toStringTag")]
            );
            runtime.steps = MAX_STEPS;
            runtime.allocated = 0;
            assert!(runtime.data_view_install_tag(owner).is_err());
            assert_eq!(runtime.objects[owner].order.len(), 1);
            assert_eq!(
                runtime.objects[owner].get(runtime.well_known_key("toStringTag")),
                Some(&Value::String("DataView".into()))
            );
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            assert!(runtime.objects[owner].order.is_empty());
            assert!(runtime.objects[owner].values.is_empty());
        }
    }
}

#[test]
fn data_view_bootstrap_reports_actual_remaining_budget() {
    let runtime = Runtime::uninitialized().finish_bootstrap().unwrap();
    println!(
        "DATAVIEW_BOOTSTRAP remaining_steps={} allocated={} native_properties={} prototypes={} order_capacity={}",
        runtime.steps,
        runtime.allocated,
        runtime.native_properties.len(),
        runtime.prototypes.len(),
        runtime.objects[runtime.prototypes["DataView"]]
            .order
            .capacity()
    );
    // Bootstrap remains charged. The unchanged large native-operation tests
    // verify usable work; this diagnostic does not reset or enlarge a budget.
    assert!(runtime.steps < MAX_STEPS);
    assert!(runtime.allocated < MAX_HEAP);
}

#[test]
fn data_view_function_metadata_does_not_need_an_object_prototype_anchor() {
    let mut reference_costs = None;
    for anchor in ["normal", "empty", "absent", "misdirected"] {
        let (mut runtime, owner) = installer_setup();
        match anchor {
            "normal" => {}
            "empty" => runtime.prototypes.clear(),
            "absent" => {
                runtime.prototypes.remove("Object");
            }
            "misdirected" => {
                runtime.prototypes.insert("Object", usize::MAX);
            }
            _ => unreachable!(),
        }
        let mut costs = Vec::new();
        let mut ids = Vec::new();
        for (full, display, key, length, getter) in [
            ("DataView.anchorFirst", "first", "first", 1, false),
            ("DataView.anchorSecond", "get second", "second", 0, true),
        ] {
            let before_work = runtime.steps;
            let before_heap = runtime.allocated;
            let expected_id = runtime.objects.len();
            runtime
                .data_view_install_function(owner, full, display, length, key, getter)
                .unwrap();
            costs.push((before_work - runtime.steps, runtime.allocated - before_heap));
            let id = runtime.native_properties[full];
            assert_eq!(id, expected_id);
            assert_eq!(runtime.objects.len(), expected_id + 1);
            ids.push(id);
            let bag = &runtime.objects[id];
            assert_eq!(
                bag.prototype,
                Some(Value::Function(runtime.function_prototype))
            );
            assert_eq!(
                bag.order,
                [PropertyKey::from("name"), PropertyKey::from("length")]
            );
            assert_eq!(bag.values.len(), 2);
            for (name, expected) in [
                ("name", Value::String(display.into())),
                ("length", Value::Number(length as f64)),
            ] {
                let property = &bag.values[&PropertyKey::from(name)];
                assert!(!property.enumerable && property.configurable);
                let PropertyValue::Data { value, writable } = &property.value else {
                    panic!()
                };
                assert!(!writable);
                assert_eq!(value, &expected);
            }
            let property = &runtime.objects[owner].values[&PropertyKey::from(key)];
            assert!(!property.enumerable && property.configurable);
            let function = if getter {
                let PropertyValue::Accessor { get, set } = &property.value else {
                    panic!()
                };
                assert_eq!(set, &Value::Undefined);
                get
            } else {
                let PropertyValue::Data { value, writable } = &property.value else {
                    panic!()
                };
                assert!(*writable);
                value
            };
            let Value::Native(native) = function else {
                panic!()
            };
            assert_eq!(native.name, full);
        }
        // Identical installs must cost the same regardless of an unrelated
        // prototype table. The former empty-table path saved a lookup charge.
        if let Some(reference) = &reference_costs {
            assert_eq!(&costs, reference, "Object anchor: {anchor}");
        } else {
            reference_costs = Some(costs);
        }
        assert_ne!(ids[0], ids[1]);
        runtime.objects[ids[0]].insert_hidden("name".into(), Value::String("changed".into()));
        assert_eq!(
            runtime.objects[ids[1]].get("name"),
            Some(&Value::String("get second".into()))
        );
        assert_eq!(
            runtime.objects[owner].order,
            [PropertyKey::from("first"), PropertyKey::from("second")]
        );
        clean(&runtime);
    }
}
