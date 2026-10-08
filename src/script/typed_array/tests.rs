use super::*;

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

fn construct(
    runtime: &mut Runtime,
    doc: &mut Document,
    kind: Kind,
    arguments: &[Value],
) -> Result<Value> {
    let target = runtime.typed_arrays.constructors[kind.index()]
        .clone()
        .unwrap();
    runtime.typed_array_constructor(kind, arguments, target, doc)
}

fn record(runtime: &mut Runtime, value: &Value) -> Record {
    runtime.typed_array_record(value).unwrap().unwrap()
}

fn observed_bytes(runtime: &mut Runtime, backing: Backing, length: usize) -> Vec<u8> {
    // Private observation after preserving the measured result. This does not
    // resume author execution or refund any cumulative allocation.
    runtime.steps = MAX_STEPS;
    let mut result = Vec::new();
    for offset in (0..length).step_by(8) {
        let size = (length - offset).min(8);
        result.extend_from_slice(
            &runtime
                .buffer_view_read(backing.buffer, backing.offset + offset, size)
                .unwrap()[..size],
        );
    }
    result
}

fn one(kind: Kind, initial: u8) -> (Runtime, Document, Record) {
    let (mut runtime, mut doc) = fresh();
    let value = construct(&mut runtime, &mut doc, kind, &[Value::Number(1.0)]).unwrap();
    let record = record(&mut runtime, &value);
    let backing = record.backing.unwrap();
    runtime
        .buffer_view_write(backing.buffer, 0, &vec![initial; kind.width()])
        .unwrap();
    (runtime, doc, record)
}

#[test]
fn typed_array_bootstrap_reports_types_and_reconciles_installation_ledger() {
    use std::mem::size_of;
    let runtime = Runtime::uninitialized().finish_bootstrap().unwrap();
    let s = size_of::<ScriptObject>();
    let a = size_of::<Option<AbortSlot>>();
    let d = size_of::<Option<f64>>();
    let k = size_of::<PropertyKey>();
    let n = size_of::<Native>();
    let g = size_of::<(String, Binding)>();
    let t = size_of::<State>();
    let bpp = 16 * (k + size_of::<Property>()) + 32 * size_of::<usize>() + 64;
    let bsb = 16 * (size_of::<String>() + size_of::<Binding>()) + 32 * size_of::<usize>() + 64;
    // Independent held installation expression. The predecessor's measured
    // allocation is retained; old bootstrap snapshots are not rewritten here.
    let added = t
        + 31 * s
        + 31 * (72 + a + d)
        + 92 * k
        + 31 * bpp
        + 3016
        + 20 * (n + 32)
        + 300
        + 30 * g
        + bsb
        + 10 * BINDING_BYTES
        + 114;
    println!(
        "TYPED_ARRAY_TYPES S={s} A={a} D={d} K={k} N={n} G={g} T={t} Binding={} BINDING_BYTES={} Bpp={bpp} Bsb={bsb} added={added}",
        size_of::<Binding>(),
        BINDING_BYTES
    );
    println!(
        "TYPED_ARRAY_BOOTSTRAP remaining={} allocated={} objects={} capacity={} native={} prototypes={} globals={}",
        runtime.steps,
        runtime.allocated,
        runtime.objects.len(),
        runtime.objects.capacity(),
        runtime.native_properties.len(),
        runtime.prototypes.len(),
        runtime.environments[0].bindings.len()
    );
    // Later ArrayBuffer initialization saves3489work/5711bytes; the Reflect
    // extension consumes3278work and27648bytes (including9 arena slots).
    // DataView direct metadata then saves4675work/197266bytes; view metadata
    // adds1970work/11514bytes including two arena slots. Independent
    // endpoint tests and the raw diagnostic check the snapshot expression.
    // The paired for-in reader enlarges eight initial frames by eight bytes.
    // Fill adds218work/2095bytes and one reserved bag.
    // Reverse forecasts227work/2110bytes and one further bag from its ledger;
    // the separately printed raw diagnostic must confirm these totals.
    // ToReversed forecasts236work/2125bytes and one further reserved bag.
    // Four search methods add908work/8440bytes and four reserved bags; the
    // independent metadata endpoints and raw diagnostic verify that delta.
    assert_eq!(
        runtime.steps,
        5941 - 5656 + 3489 - 3278 + 4675 - 1970 - 908 - 218 - 227 - 236
    );
    assert_eq!(
        runtime.allocated,
        1_782_581 + added - 5711 + 27648 - 197266 + 11514 + 64 + 8440 + 2095 + 2110 + 2125
    );
    assert_eq!(
        (runtime.objects.len(), runtime.objects.capacity()),
        (739, 739)
    );
    assert_eq!(
        (runtime.native_properties.len(), runtime.prototypes.len()),
        (299, 25)
    );
    assert_eq!(runtime.environments[0].bindings.len(), 210);
    assert!(runtime.typed_arrays.records.is_empty());
    assert!(runtime.allocated < MAX_HEAP);
    clean(&runtime);
}

#[test]
fn typed_array_uint8_clamp_literal_ties_are_visible_in_raw_bytes() {
    let values = [
        f64::NAN,
        f64::NEG_INFINITY,
        -1.0,
        -0.0,
        0.0,
        f64::from_bits(0x3fdf_ffff_ffff_ffff),
        0.5,
        f64::from_bits(0x3fe0_0000_0000_0001),
        1.5,
        2.5,
        3.5,
        127.5,
        128.5,
        253.5,
        254.5,
        254.500_001,
        255.0,
        256.0,
        f64::INFINITY,
    ];
    let expected: [u8; 19] = [
        0, 0, 0, 0, 0, 0, 0, 1, 2, 2, 4, 128, 128, 254, 254, 255, 255, 255, 255,
    ];
    let (mut runtime, mut doc) = fresh();
    let value = construct(
        &mut runtime,
        &mut doc,
        Kind::Uint8Clamped,
        &[Value::Number(values.len() as f64)],
    )
    .unwrap();
    let record = record(&mut runtime, &value);
    for (index, number) in values.into_iter().enumerate() {
        runtime
            .typed_array_write_number(record, index, number)
            .unwrap();
        assert_eq!(
            runtime.typed_array_read_number(record, index).unwrap(),
            f64::from(expected[index])
        );
    }
    assert_eq!(
        observed_bytes(&mut runtime, record.backing.unwrap(), expected.len()),
        expected
    );
    clean(&runtime);
}

#[test]
fn typed_array_all_ten_scalar_read_write_endpoints_keep_unpaid_bytes() {
    let rows: &[(Kind, &[u8])] = &[
        (Kind::Int8, &[42]),
        (Kind::Uint8, &[42]),
        (Kind::Uint8Clamped, &[42]),
        (Kind::Int16, &[42, 0]),
        (Kind::Uint16, &[42, 0]),
        (Kind::Int32, &[42, 0, 0, 0]),
        (Kind::Uint32, &[42, 0, 0, 0]),
        (Kind::Float16, &[0x40, 0x51]),
        (Kind::Float32, &[0, 0, 0x28, 0x42]),
        (Kind::Float64, &[0, 0, 0, 0, 0, 0, 0x45, 0x40]),
    ];
    assert_eq!(rows.len(), 10);
    for &(kind, expected) in rows {
        let (mut measure, _, record) = one(kind, 0xa5);
        let before = (measure.steps, measure.allocated);
        measure.typed_array_write_number(record, 0, 42.0).unwrap();
        let write_work = before.0 - measure.steps;
        assert_eq!(measure.allocated, before.1);
        assert_eq!(
            observed_bytes(&mut measure, record.backing.unwrap(), kind.width()),
            expected
        );
        let before_read = measure.steps;
        assert_eq!(measure.typed_array_read_number(record, 0).unwrap(), 42.0);
        let read_work = before_read - measure.steps;
        assert!(write_work > 0 && read_work > 0);
        for succeeds in [false, true] {
            let (mut runtime, _, record) = one(kind, 0xa5);
            runtime.steps = write_work - usize::from(!succeeds);
            runtime.allocated = MAX_HEAP;
            let result = runtime.typed_array_write_number(record, 0, 42.0);
            assert_eq!(result.is_ok(), succeeds, "{} write", kind.name());
            if !succeeds {
                assert!(result.unwrap_err().is_resource_limit());
            }
            assert_eq!(runtime.steps, 0);
            assert_eq!(runtime.allocated, MAX_HEAP);
            let wanted = if succeeds {
                expected.to_vec()
            } else {
                vec![0xa5; kind.width()]
            };
            assert_eq!(
                observed_bytes(&mut runtime, record.backing.unwrap(), kind.width()),
                wanted
            );
            clean(&runtime);

            let (mut runtime, _, record) = one(kind, 0);
            let backing = record.backing.unwrap();
            runtime
                .buffer_view_write(backing.buffer, 0, expected)
                .unwrap();
            runtime.steps = read_work - usize::from(!succeeds);
            runtime.allocated = MAX_HEAP;
            let result = runtime.typed_array_read_number(record, 0);
            if succeeds {
                assert_eq!(result.unwrap().to_bits(), 42.0f64.to_bits());
            } else {
                assert!(result.unwrap_err().is_resource_limit());
            }
            assert_eq!(runtime.steps, 0);
            assert_eq!(runtime.allocated, MAX_HEAP);
            assert_eq!(
                observed_bytes(&mut runtime, backing, kind.width()),
                expected
            );
            clean(&runtime);
        }
    }
}

#[test]
fn typed_array_same_kind_clone_preserves_nan_payloads_and_negative_zero() {
    let rows: &[(Kind, &[u8])] = &[
        (Kind::Float16, &[0x01, 0x7c, 0, 0x80]),
        (Kind::Float32, &[0x01, 0, 0xc0, 0x7f, 0, 0, 0, 0x80]),
        (
            Kind::Float64,
            &[0x01, 0, 0, 0, 0, 0, 0xf8, 0x7f, 0, 0, 0, 0, 0, 0, 0, 0x80],
        ),
    ];
    for &(kind, payload) in rows {
        let (mut runtime, mut doc) = fresh();
        let width = kind.width();
        let buffer = runtime
            .array_buffer_constructor(
                &[Value::Number((4 * width) as f64)],
                Runtime::native("ArrayBuffer", Value::Window),
                &mut doc,
            )
            .unwrap();
        let id = runtime.buffer_for_view(&buffer).unwrap();
        for offset in (0..4 * width).step_by(width) {
            runtime
                .buffer_view_write(id, offset, &vec![0xa5; width])
                .unwrap();
        }
        for (index, chunk) in payload.chunks(width).enumerate() {
            runtime
                .buffer_view_write(id, width + index * width, chunk)
                .unwrap();
        }
        let source = construct(
            &mut runtime,
            &mut doc,
            kind,
            &[buffer, Value::Number(width as f64), Value::Number(2.0)],
        )
        .unwrap();
        let source_record = record(&mut runtime, &source);
        assert!(
            runtime
                .typed_array_read_number(source_record, 0)
                .unwrap()
                .is_nan()
        );
        assert_eq!(
            runtime
                .typed_array_read_number(source_record, 1)
                .unwrap()
                .to_bits(),
            1u64 << 63
        );
        let copied =
            construct(&mut runtime, &mut doc, kind, std::slice::from_ref(&source)).unwrap();
        assert_ne!(copied, source);
        let copied_record = record(&mut runtime, &copied);
        let backing = copied_record.backing.unwrap();
        assert_ne!(backing.buffer, id);
        assert_eq!(backing.offset, 0);
        assert_eq!(runtime.typed_array_length(copied_record).unwrap(), 2);
        assert_eq!(
            observed_bytes(&mut runtime, backing, payload.len()),
            payload
        );
        runtime
            .buffer_view_write(backing.buffer, 0, &vec![0; width])
            .unwrap();
        assert_eq!(
            observed_bytes(&mut runtime, source_record.backing.unwrap(), payload.len()),
            payload
        );
        assert_eq!(
            &runtime.buffer_view_read(id, 0, width).unwrap()[..width],
            &vec![0xa5; width]
        );
        assert_eq!(
            &runtime.buffer_view_read(id, 3 * width, width).unwrap()[..width],
            &vec![0xa5; width]
        );
        clean(&runtime);
    }
}

#[test]
fn typed_array_reentrant_construction_keeps_sorted_brands_and_failed_shell() {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = r#"
            var log='', nested;
            var source={get length(){log+='L';nested=new Uint16Array([7,9]);return 2;},
                get 0(){log+='A';return 11;},get 1(){log+='B';return 13;}};
            var outer=new Uint8Array(source), failed=false;
            try {new Float32Array({get length(){log+='F';new Int8Array(1);throw 73;}});}
            catch(e){if(e!==73)throw e;failed=true;}
            if(!failed||log!=='LABF'||outer.length!==2||outer[0]!==11||outer[1]!==13||
                nested[0]!==7||nested[1]!==9)throw new Error('recursive construction');
            true;
        "#;
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        assert_eq!(result.unwrap(), Value::Bool(true));
        clean(&runtime);
        assert_eq!(runtime.typed_arrays.records.len(), 4);
        assert!(
            runtime
                .typed_arrays
                .records
                .windows(2)
                .all(|pair| pair[0].object_id < pair[1].object_id)
        );
        let records = runtime.typed_arrays.records.clone();
        assert_eq!(
            records.iter().map(|record| record.kind).collect::<Vec<_>>(),
            [Kind::Uint8, Kind::Uint16, Kind::Float32, Kind::Int8]
        );
        assert!(records[0].backing.is_some() && records[1].backing.is_some());
        assert!(records[2].backing.is_none());
        assert!(records[3].backing.is_some());
        for expected in records {
            let actual = record(&mut runtime, &Value::Object(expected.object_id));
            assert_eq!(
                (actual.object_id, actual.kind),
                (expected.object_id, expected.kind)
            );
        }
    }
}

#[test]
fn typed_array_primitive_constructor_measured_work_and_heap_boundaries() {
    for kind in Kind::ALL {
        let (mut measure, mut doc) = fresh();
        let before = (measure.steps, measure.allocated);
        let value = construct(&mut measure, &mut doc, kind, &[Value::Number(3.0)]).unwrap();
        let work = before.0 - measure.steps;
        let heap = measure.allocated - before.1;
        let measured_record = record(&mut measure, &value);
        assert_eq!(
            observed_bytes(
                &mut measure,
                measured_record.backing.unwrap(),
                3 * kind.width()
            ),
            vec![0; 3 * kind.width()]
        );
        assert!(work > 0 && heap > 0);
        for (work_limit, heap_limit, succeeds) in [
            (work, heap, true),
            (work - 1, heap, false),
            (work, heap - 1, false),
        ] {
            let (mut runtime, mut doc) = fresh();
            runtime.steps = work_limit;
            runtime.allocated = MAX_HEAP - heap_limit;
            let result = construct(&mut runtime, &mut doc, kind, &[Value::Number(3.0)]);
            assert_eq!(result.is_ok(), succeeds, "{}", kind.name());
            clean(&runtime);
            if succeeds {
                assert_eq!(runtime.steps, 0);
                assert_eq!(runtime.allocated, MAX_HEAP);
                runtime.steps = MAX_STEPS;
                let actual = record(&mut runtime, &result.unwrap());
                assert_eq!(runtime.typed_array_length(actual).unwrap(), 3);
                assert_eq!(
                    observed_bytes(&mut runtime, actual.backing.unwrap(), 3 * kind.width()),
                    vec![0; 3 * kind.width()]
                );
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert!(
                    runtime
                        .typed_arrays
                        .records
                        .iter()
                        .all(|record| record.backing.is_none())
                );
            }
        }
    }
}

#[test]
fn typed_array_iterable_exhaustion_precedes_coercion_and_array_like_interleaves() {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = r#"
            var log='', step=0, closed=0;
            var a={valueOf:function(){log+='A';return 5;}},b={valueOf:function(){log+='B';return 9;}};
            var source={};source[Symbol.iterator]=function(){log+='I';return {
                next:function(){log+='N';step++;return step===1?{value:a,done:false}:
                    step===2?{value:b,done:false}:{done:true};},
                return:function(){closed++;return {};}};};
            var result=new Uint8Array(source);
            if(log!=='INNNAB'||closed!==0||result[0]!==5||result[1]!==9)throw new Error('iterator order');
            log='';
            var list={get length(){log+='L';return 2;},get 0(){log+='0';return a;},get 1(){log+='1';return b;}};
            var other=new Uint8Array(list);
            if(log!=='L0A1B'||other[0]!==5||other[1]!==9)throw new Error('array-like order');
            true;
        "#;
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        assert_eq!(result.unwrap(), Value::Bool(true));
        clean(&runtime);
    }
}

#[test]
fn typed_array_array_like_final_write_cut_retains_callback_and_zero_destination() {
    fn setup() -> (Runtime, Document, Value, BufferId) {
        let (mut runtime, mut doc) = fresh();
        runtime
            .execute(
                r#"
            var side=new ArrayBuffer(1), view=new DataView(side), effects=0;
            view.setUint8(0,9);
            var source={length:1,0:{valueOf:function(){effects++;view.setUint8(0,77);return 42;}}};
        "#,
                &mut doc,
            )
            .unwrap();
        let source = runtime.lookup(1, "source").unwrap().1;
        let side = runtime.lookup(1, "side").unwrap().1;
        let buffer = runtime.buffer_for_view(&side).unwrap();
        (runtime, doc, source, buffer)
    }
    let (mut measure, mut doc, source, _) = setup();
    let before = measure.steps;
    construct(&mut measure, &mut doc, Kind::Uint8, &[source]).unwrap();
    let work = before - measure.steps;
    for succeeds in [false, true] {
        let (mut runtime, mut doc, source, side) = setup();
        runtime.steps = work - usize::from(!succeeds);
        let result = construct(&mut runtime, &mut doc, Kind::Uint8, &[source]);
        assert_eq!(result.is_ok(), succeeds);
        if !succeeds {
            assert!(result.unwrap_err().is_resource_limit());
        }
        assert_eq!(runtime.steps, 0);
        clean(&runtime);
        assert_eq!(runtime.typed_arrays.records.len(), 1);
        let target = runtime.typed_arrays.records[0];
        assert_eq!(
            observed_bytes(&mut runtime, target.backing.unwrap(), 1),
            [if succeeds { 42 } else { 0 }]
        );
        assert_eq!(runtime.buffer_view_read(side, 0, 1).unwrap()[0], 77);
        assert_eq!(runtime.lookup(1, "effects").unwrap().1, Value::Number(1.0));
    }
}

#[test]
fn typed_array_buffer_final_publication_cut_retains_conversion_side_effects() {
    fn setup() -> (Runtime, Document, Vec<Value>, BufferId) {
        let (mut runtime, mut doc) = fresh();
        runtime
            .execute(
                r#"
            var buffer=new ArrayBuffer(4), view=new DataView(buffer), effects=0;
            var length={valueOf:function(){effects++;view.setUint8(0,77);return 2;}};
        "#,
                &mut doc,
            )
            .unwrap();
        let buffer = runtime.lookup(1, "buffer").unwrap().1;
        let length = runtime.lookup(1, "length").unwrap().1;
        let id = runtime.buffer_for_view(&buffer).unwrap();
        (runtime, doc, vec![buffer, Value::Number(0.0), length], id)
    }
    let (mut measure, mut doc, arguments, _) = setup();
    let before = measure.steps;
    construct(&mut measure, &mut doc, Kind::Uint16, &arguments).unwrap();
    let work = before - measure.steps;
    for succeeds in [false, true] {
        let (mut runtime, mut doc, arguments, buffer) = setup();
        runtime.steps = work - usize::from(!succeeds);
        let result = construct(&mut runtime, &mut doc, Kind::Uint16, &arguments);
        assert_eq!(result.is_ok(), succeeds);
        if !succeeds {
            assert!(result.unwrap_err().is_resource_limit());
        }
        assert_eq!(runtime.steps, 0);
        clean(&runtime);
        assert_eq!(runtime.typed_arrays.records.len(), 1);
        assert_eq!(runtime.typed_arrays.records[0].backing.is_some(), succeeds);
        runtime.steps = MAX_STEPS;
        assert_eq!(
            &runtime.buffer_view_read(buffer, 0, 4).unwrap()[..4],
            &[77, 0, 0, 0]
        );
        assert_eq!(runtime.lookup(1, "effects").unwrap().1, Value::Number(1.0));
    }
}
