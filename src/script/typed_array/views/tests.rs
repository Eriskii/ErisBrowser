use super::*;

#[test]
fn views_preserve_published_non_numeric_descriptor_upstream_case() {
    // This exact case passed before the shared prototype gained three keys.
    // Its helper enumerates prototypes, so aggregate conformance totals can
    // hide a resource regression when unrelated cases begin passing.
    const HARNESS: [&str; 4] = [
        include_str!("../../../../tests/upstream/test262-typedarray-foundation/harness/assert.js"),
        include_str!("../../../../tests/upstream/test262-typedarray-foundation/harness/sta.js"),
        include_str!(
            "../../../../tests/upstream/test262-typedarray-foundation/harness/testTypedArray.js"
        ),
        include_str!(
            "../../../../tests/upstream/test262-typedarray-foundation/harness/propertyHelper.js"
        ),
    ];
    const SOURCE: &str = include_str!(
        "../../../../tests/upstream/test262-typedarray-foundation/test/built-ins/TypedArrayConstructors/internals/DefineOwnProperty/key-is-not-numeric-index.js"
    );
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        for source in HARNESS {
            runtime.execute(source, &mut doc).unwrap();
        }
        let result = if strict {
            runtime.execute_strict(SOURCE, &mut doc)
        } else {
            runtime.execute(SOURCE, &mut doc)
        };
        println!(
            "TYPED_VIEWS_PRIOR_PASS strict={strict} remaining={} records={} allocated={}",
            runtime.steps,
            runtime.typed_arrays.records.len(),
            runtime.allocated
        );
        assert!(result.is_ok(), "strict={strict}: {result:?}");
        clean(&runtime);
    }
}

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

fn construct(runtime: &mut Runtime, doc: &mut Document, kind: Kind, length: usize) -> Value {
    let target = runtime.typed_arrays.constructors[kind.index()]
        .clone()
        .unwrap();
    runtime
        .typed_array_constructor(kind, &[Value::Number(length as f64)], target, doc)
        .unwrap()
}

fn record(runtime: &mut Runtime, value: &Value) -> Record {
    runtime.typed_array_record(value).unwrap().unwrap()
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
fn subarray_all_10_kinds_share_backing_without_allocating_a_buffer() {
    for kind in Kind::ALL {
        let (mut runtime, mut doc) = fresh();
        let source = construct(&mut runtime, &mut doc, kind, 4);
        let before = record(&mut runtime, &source);
        for (index, value) in [1.0, 2.0, 3.0, 4.0].into_iter().enumerate() {
            runtime
                .typed_array_write_number(before, index, value)
                .unwrap();
        }
        let function = saved(&runtime, "subarray");
        let objects = runtime.objects.len();
        let result = runtime
            .call(
                function,
                vec![Value::Number(1.0), Value::Number(3.0)],
                source,
                &mut doc,
            )
            .unwrap();
        assert_eq!(
            runtime.objects.len(),
            objects + 1,
            "only the view shell is new"
        );
        let after = record(&mut runtime, &result);
        assert_eq!(after.kind, kind);
        assert_eq!(
            after.backing.unwrap().buffer,
            before.backing.unwrap().buffer
        );
        assert_eq!(after.backing.unwrap().offset, kind.width());
        assert_eq!(runtime.typed_array_length(after).unwrap(), 2);
        assert_eq!(
            runtime.objects[after.object_id].prototype,
            Some(Value::Object(
                runtime.typed_arrays.prototypes[kind.index()].unwrap()
            ))
        );
        assert_eq!(runtime.typed_array_read_number(after, 0).unwrap(), 2.0);
        assert_eq!(runtime.typed_array_read_number(after, 1).unwrap(), 3.0);
        runtime.typed_array_write_number(after, 0, 9.0).unwrap();
        runtime.typed_array_write_number(before, 2, 11.0).unwrap();
        assert_eq!(runtime.typed_array_read_number(before, 1).unwrap(), 9.0);
        assert_eq!(runtime.typed_array_read_number(after, 1).unwrap(), 11.0);
        assert_eq!(runtime.typed_array_read_number(before, 0).unwrap(), 1.0);
        assert_eq!(runtime.typed_array_read_number(before, 3).unwrap(), 4.0);
        clean(&runtime);
    }
}

#[test]
fn subarray_captured_zero_length_stored_offset_and_species_argument_arity() {
    authored(
        r#"(function(){
        var buffer=new ArrayBuffer(6,{maxByteLength:10});
        var tracking=new Uint8Array(buffer,2), fixed=new Uint8Array(buffer,2,4), trace='';
        var owner={};owner[Symbol.species]=function(b,o,n){
            if(b!==buffer)throw new Error('buffer identity');
            trace+=arguments.length+':'+o+':'+n+';';return new Uint16Array(0);
        };
        tracking.constructor=owner;fixed.constructor=owner;
        tracking.subarray(1);tracking.subarray(1,undefined);fixed.subarray(1);
        tracking.subarray(1,{valueOf:function(){trace+='E;';return undefined;}});
        if(trace!=='2:3:undefined;2:3:undefined;3:3:3;E;3:3:0;')return false;
        delete tracking.constructor;delete fixed.constructor;
        buffer.resize(0);
        var revive={valueOf:function(){buffer.resize(8);return 1;}};
        var first=tracking.subarray(revive);
        buffer.resize(0);var second=fixed.subarray(revive);
        if(first.byteOffset!==2||first.length!==6||second.byteOffset!==2||second.length!==0)return false;
        fixed.constructor=owner;trace='';buffer.transfer();
        var third=fixed.subarray(1,2);
        return trace==='3:2:0;'&&third.length===0&&third instanceof Uint16Array;
    })()"#,
    );
}

#[test]
fn subarray_callbacks_reallocate_records_and_fresh_result_validation_is_terminal() {
    authored(
        r#"(function(){
        var a=new Uint8Array([3,5,7]), b=new ArrayBuffer(4,{maxByteLength:8});
        var chosen=new Float32Array(b,0,1), trace='', owner={};
        Object.defineProperty(a,'constructor',{get:function(){trace+='C';return owner;}});
        Object.defineProperty(owner,Symbol.species,{configurable:true,get:function(){trace+='S';return function(){
            trace+='N';for(var i=0;i<12;i++)new Uint8Array(0);return chosen;
        };}});
        var begin={valueOf:function(){trace+='B';return 1;}};
        var end={valueOf:function(){trace+='E';return 3;}};
        if(a.subarray(begin,end)!==chosen||trace!=='BECSN'||chosen.length!==1)return false;
        Object.defineProperty(owner,Symbol.species,{value:function(){b.resize(0);return chosen;},writable:true});
        var rejected=false;try{a.subarray(0);}catch(e){if(!(e instanceof TypeError))throw e;rejected=true;}
        if(!rejected||chosen.length!==0)return false;
        owner[Symbol.species]=function(){b.resize(4);return chosen;};
        if(a.subarray(0)!==chosen)return false;
        owner[Symbol.species]=function(){b.transfer();return chosen;};rejected=false;
        try{a.subarray(0);}catch(e){if(!(e instanceof TypeError))throw e;rejected=true;}
        var reason={};owner[Symbol.species]=function(){throw reason;};var identity=false;
        try{a.subarray(0);}catch(e){identity=e===reason;}
        return rejected&&b.detached&&identity;
    })()"#,
    );
}

#[test]
fn join_captured_length_fresh_indices_and_poisoned_observable_length() {
    authored(
        r#"(function(){
        var b=new ArrayBuffer(4,{maxByteLength:8}),a=new Uint8Array(b),calls=0;
        a[0]=7;a[1]=8;a[2]=9;a[3]=10;
        Object.defineProperty(a,'length',{get:function(){throw new Error('observable length');}});
        var p=Object.getPrototypeOf(Uint8Array.prototype);
        Object.defineProperty(p,'3',{get:function(){throw new Error('prototype numeric key');},configurable:true});
        if(a.join({toString:function(){calls++;b.resize(2);return '|';}})!=='7|8||')return false;
        b.resize(4);var fixed=new Uint8Array(b,0,4);
        if(fixed.join({toString:function(){calls++;b.resize(2);return '|';}})!=='|||')return false;
        a[0]=11;a[1]=12;
        if(a.join({toString:function(){calls++;b.resize(4);a[0]=21;a[2]=99;return ':';}})!=='21:12')return false;
        b.resize(3);
        return a.join({toString:function(){calls++;b.transfer();return '-';}})==='--'&&calls===4&&b.detached;
    })()"#,
    );
}

#[test]
fn join_initial_refusal_precedes_separator_but_empty_valid_view_coerces_once() {
    authored(
        r#"(function(){
        var b=new ArrayBuffer(2,{maxByteLength:4}),a=new Uint8Array(b,1,1),calls=0;
        var sep={toString:function(){calls++;return '|';}};
        if(a.join(sep)!=='0'||calls!==1)return false;
        b.resize(0);var rejected=false;
        try{a.join(sep);}catch(e){if(!(e instanceof TypeError))throw e;rejected=true;}
        if(!rejected||calls!==1)return false;
        var empty=new Uint8Array(0);
        if(empty.join(sep)!==''||calls!==2)return false;
        var reason={},identity=false;
        try{empty.join({toString:function(){throw reason;}});}catch(e){identity=e===reason;}
        rejected=false;try{empty.join(Symbol('s'));}catch(e){if(!(e instanceof TypeError))throw e;rejected=true;}
        return identity&&rejected;
    })()"#,
    );
}

#[test]
fn join_exact_number_text_and_unpaired_utf16_units() {
    let (mut runtime, mut doc) = fresh();
    let source = construct(&mut runtime, &mut doc, Kind::Float64, 6);
    let source_record = record(&mut runtime, &source);
    for (index, number) in [
        f64::NAN,
        -0.0,
        f64::INFINITY,
        f64::NEG_INFINITY,
        1e21,
        f64::from_bits(0x430c_6bf5_2634_0002),
    ]
    .into_iter()
    .enumerate()
    {
        runtime
            .typed_array_write_number(source_record, index, number)
            .unwrap();
    }
    let function = saved(&runtime, "join");
    let result = runtime
        .call(
            function.clone(),
            vec![Value::String(JsString::from("|"))],
            source,
            &mut doc,
        )
        .unwrap();
    assert_eq!(
        result,
        Value::String(JsString::from(
            "NaN|0|Infinity|-Infinity|1e+21|1000000000000000.2"
        ))
    );
    let source = construct(&mut runtime, &mut doc, Kind::Uint8, 3);
    let source_record = record(&mut runtime, &source);
    for index in 0..3 {
        runtime
            .typed_array_write_number(source_record, index, (index + 1) as f64)
            .unwrap();
    }
    let separator: JsString = vec![0xd800, 88, 0xdfff].into();
    let result = runtime
        .call(function, vec![Value::String(separator)], source, &mut doc)
        .unwrap();
    let Value::String(result) = result else {
        panic!("joined string")
    };
    assert_eq!(
        result.units(),
        &[49, 0xd800, 88, 0xdfff, 50, 0xd800, 88, 0xdfff, 51]
    );
    clean(&runtime);
}

#[test]
fn views_saved_native_preflight_exact_and_one_short_preserves_metadata() {
    for name in ["subarray", "join"] {
        let (mut runtime, mut doc) = fresh();
        let function = saved(&runtime, name);
        let Value::Native(native) = &function else {
            panic!("installed Native")
        };
        let identity = native.clone();
        let work = 4 + native.name.len();
        let heap = 32 + native.name.len();
        let bag = runtime.property_object(&function).unwrap();
        let metadata = format!("{:?}", runtime.objects[bag].values);
        let original = (runtime.objects.len(), runtime.typed_arrays.records.len());
        for (steps, room, charged) in [
            (work, heap, 0),
            (1 + work, heap, heap),
            (1 + work, heap - 1, heap),
        ] {
            runtime.steps = steps;
            runtime.allocated = MAX_HEAP - room;
            let before = runtime.allocated;
            let error = runtime
                .call(function.clone(), Vec::new(), Value::Null, &mut doc)
                .unwrap_err();
            assert!(error.is_resource_limit());
            assert_eq!(runtime.steps, 0);
            assert_eq!(runtime.allocated, before + charged);
            assert_eq!(
                (runtime.objects.len(), runtime.typed_arrays.records.len()),
                original
            );
            assert_eq!(format!("{:?}", runtime.objects[bag].values), metadata);
            let Value::Native(current) = &function else {
                unreachable!()
            };
            assert!(Rc::ptr_eq(current, &identity));
            assert_eq!(current.receiver, Value::Undefined);
            clean(&runtime);
        }
    }
}

fn endpoint_fixture(name: &str) -> (Runtime, Document, Value, Value, Vec<Value>) {
    let (mut runtime, mut doc) = fresh();
    let source = construct(&mut runtime, &mut doc, Kind::Uint8, 3);
    let r = record(&mut runtime, &source);
    for index in 0..3 {
        runtime
            .typed_array_write_number(r, index, (index + 1) as f64)
            .unwrap();
    }
    // Exactly fill the first record allocation. Subarray's new view must grow
    // the real registry, while join never appends a record.
    for _ in 0..3 {
        construct(&mut runtime, &mut doc, Kind::Uint8, 0);
    }
    assert_eq!(
        (
            runtime.typed_arrays.records.len(),
            runtime.typed_arrays.records.capacity()
        ),
        (4, 4)
    );
    let arguments = if name == "subarray" {
        vec![Value::Number(1.0)]
    } else {
        vec![Value::String(JsString::from("|"))]
    };
    let function = saved(&runtime, name);
    (runtime, doc, function, source, arguments)
}

#[test]
fn views_full_saved_call_work_and_heap_endpoints_include_record_growth() {
    for name in ["subarray", "join"] {
        let (mut measured, mut doc, function, source, arguments) = endpoint_fixture(name);
        let start = (measured.steps, measured.allocated);
        measured
            .call(function, arguments, source, &mut doc)
            .unwrap();
        let (work, heap) = (start.0 - measured.steps, measured.allocated - start.1);
        assert!(work > 0 && heap > 0);
        if name == "subarray" {
            assert_eq!(
                (
                    measured.typed_arrays.records.len(),
                    measured.typed_arrays.records.capacity()
                ),
                (5, 8)
            );
        }
        for (steps, room, success) in [
            (work, heap, true),
            (work - 1, heap, false),
            (work, heap - 1, false),
        ] {
            let (mut runtime, mut doc, function, source, arguments) = endpoint_fixture(name);
            let bag = runtime.property_object(&function).unwrap();
            let metadata = format!("{:?}", runtime.objects[bag].values);
            runtime.steps = steps;
            runtime.allocated = MAX_HEAP - room;
            let result = runtime.call(function, arguments, source, &mut doc);
            if success {
                let result = result.unwrap();
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
                if name == "join" {
                    assert_eq!(result, Value::String(JsString::from("1|2|3")));
                } else {
                    assert!(matches!(result, Value::Object(_)));
                }
            } else {
                assert!(result.unwrap_err().is_resource_limit());
            }
            assert_eq!(format!("{:?}", runtime.objects[bag].values), metadata);
            clean(&runtime);
        }
    }
}

fn callback_fixture(name: &str) -> (Runtime, Document, Value, Value, Vec<Value>) {
    let (mut runtime, mut doc) = fresh();
    runtime.execute(if name=="subarray" {
        "var effects=0;var source=new Uint8Array([1,2]);var chosen=new Float64Array([2.5]);source.constructor={};source.constructor[Symbol.species]=function(){effects++;return chosen;};"
    } else {
        "var effects=0;var source=new Uint8Array([1,2]);var separator={toString:function(){effects++;source[0]=9;return '|';}};"
    },&mut doc).unwrap();
    let function = saved(&runtime, name);
    let source = runtime.lookup(1, "source").unwrap().1;
    let arguments = if name == "subarray" {
        vec![Value::Number(0.0)]
    } else {
        vec![runtime.lookup(1, "separator").unwrap().1]
    };
    (runtime, doc, function, source, arguments)
}

#[test]
fn views_late_native_refusal_retains_callbacks_and_cleans_machine_guards() {
    for name in ["subarray", "join"] {
        let (mut measured, mut doc, function, source, arguments) = callback_fixture(name);
        let start = (measured.steps, measured.allocated);
        let result = measured
            .call(function, arguments, source, &mut doc)
            .unwrap();
        if name == "subarray" {
            assert_eq!(result, measured.lookup(1, "chosen").unwrap().1);
        } else {
            assert_eq!(result, Value::String(JsString::from("9|2")));
        }
        let (work, heap) = (start.0 - measured.steps, measured.allocated - start.1);
        // Subarray's last step is fresh result validation; join's last heap
        // charge is final String storage. Neither is whole-evaluation rollback.
        let (mut runtime, mut doc, function, source, arguments) = callback_fixture(name);
        let source_record = record(&mut runtime, &source);
        runtime.steps = if name == "subarray" { work - 1 } else { work };
        runtime.allocated = MAX_HEAP - if name == "join" { heap - 1 } else { heap };
        let error = runtime
            .call(function, arguments, source, &mut doc)
            .unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(runtime.lookup(1, "effects").unwrap().1, Value::Number(1.0));
        clean(&runtime);
        if name == "join" {
            assert_eq!(runtime.allocated, MAX_HEAP + 1);
            // Private observation after preserving the actual terminal result.
            runtime.steps = MAX_STEPS;
            assert_eq!(
                runtime.typed_array_read_number(source_record, 0).unwrap(),
                9.0
            );
        }
    }
}

#[test]
fn join_maximum_utf16_result_and_one_unit_over_refusal() {
    for separator_length in [MAX_STRING - 2, MAX_STRING - 1] {
        let (mut runtime, mut doc) = fresh();
        let source = construct(&mut runtime, &mut doc, Kind::Uint8, 2);
        let source_record = record(&mut runtime, &source);
        runtime
            .typed_array_write_number(source_record, 0, 1.0)
            .unwrap();
        runtime
            .typed_array_write_number(source_record, 1, 2.0)
            .unwrap();
        let function = saved(&runtime, "join");
        // The separator is an already-existing caller value. The native still
        // pays for every output growth and the final immutable String copy.
        let separator: JsString = vec![0xd800; separator_length].into();
        runtime.steps = MAX_STEPS;
        let result = runtime.call(function, vec![Value::String(separator)], source, &mut doc);
        if separator_length == MAX_STRING - 2 {
            let Value::String(text) = result.unwrap() else {
                panic!("joined String")
            };
            assert_eq!(text.len(), MAX_STRING);
            assert_eq!(text.units()[0], 49);
            assert_eq!(text.units()[MAX_STRING - 1], 50);
            assert!(
                text.units()[1..MAX_STRING - 1]
                    .iter()
                    .all(|unit| *unit == 0xd800)
            );
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        clean(&runtime);
    }
}

#[test]
fn join_empty_fields_after_detach_still_exhaust_captured_loop_work() {
    let (mut runtime, mut doc) = fresh();
    runtime.execute("var effects=0;var source=new Uint8Array(10000);var buffer=source.buffer;var separator={toString:function(){effects++;buffer.transfer();return '';}};",&mut doc).unwrap();
    let source = runtime.lookup(1, "source").unwrap().1;
    let source_record = record(&mut runtime, &source);
    let separator = runtime.lookup(1, "separator").unwrap().1;
    let function = saved(&runtime, "join");
    runtime.steps = MAX_STEPS;
    let error = runtime
        .call(function, vec![separator], source, &mut doc)
        .unwrap_err();
    assert!(error.is_resource_limit());
    assert_eq!(runtime.steps, 0);
    assert_eq!(runtime.lookup(1, "effects").unwrap().1, Value::Number(1.0));
    clean(&runtime);
    runtime.steps = MAX_STEPS;
    assert!(runtime.typed_array_live(source_record).unwrap().is_none());
}
