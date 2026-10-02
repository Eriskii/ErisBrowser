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

fn make(
    runtime: &mut Runtime,
    doc: &mut Document,
    bytes: &[u8],
    maximum: Option<u64>,
) -> (Value, usize) {
    let ctor = runtime.array_buffers.intrinsic.as_ref().unwrap().clone();
    let (value, index) = runtime
        .buffer_allocate(bytes.len() as u64, maximum, ctor, doc)
        .unwrap();
    runtime.array_buffers.records[index]
        .bytes
        .as_mut()
        .unwrap()
        .copy_from_slice(bytes);
    (value, index)
}

fn bytes(runtime: &Runtime, index: usize) -> &[u8] {
    runtime.array_buffers.records[index].bytes.as_ref().unwrap()
}

fn bind(runtime: &mut Runtime, name: &str, value: Value) {
    runtime.define(1, name, value, true).unwrap();
}

#[test]
fn array_buffer_metadata_brands_aliases_and_isview() {
    check(
        r#"
        var A=ArrayBuffer,p=A.prototype,b=new A(3),bl=Object.getOwnPropertyDescriptor(p,'byteLength').get;
        verifyProperty(A,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});
        verifyProperty(A,'prototype',{value:p,writable:false,enumerable:false,configurable:false},{restore:true});
        verifyProperty(p,'constructor',{value:A,writable:true,enumerable:false,configurable:true},{restore:true});
        var names=['byteLength','maxByteLength','resizable','detached'];
        for(var i=0;i<names.length;i++){
          var d=Object.getOwnPropertyDescriptor(p,names[i]),g=d.get;
          assert.sameValue(d.set,undefined);assert.sameValue(d.enumerable,false);assert.sameValue(d.configurable,true);
          verifyProperty(g,'name',{value:'get '+names[i],writable:false,enumerable:false,configurable:true},{restore:true});
          assert.sameValue(g.length,0);assert.sameValue(g.hasOwnProperty('prototype'),false);
          assert.throws(TypeError,function(){new g();});assert.throws(TypeError,function(){g.call(p);});
          assert.throws(TypeError,function(){g.call(Object.create(b));});
        }
        var ms=['resize','slice','transfer','transferToFixedLength'],lengths=[1,2,0,0];
        for(var j=0;j<ms.length;j++){var f=p[ms[j]];verifyProperty(p,ms[j],{value:f,writable:true,enumerable:false,configurable:true},{restore:true});
          assert.sameValue(f.name,ms[j]);assert.sameValue(f.length,lengths[j]);assert.throws(TypeError,function(){new f();});}
        var sg=Object.getOwnPropertyDescriptor(A,Symbol.species).get;
        assert.sameValue(sg.name,'get [Symbol.species]');assert.sameValue(sg.length,0);assert.sameValue(sg.call(null),null);
        var s=Symbol('x');assert.sameValue(sg.call(s),s);
        verifyProperty(p,Symbol.toStringTag,{value:'ArrayBuffer',writable:false,enumerable:false,configurable:true},{restore:true});
        assert.sameValue(Object.prototype.toString.call(p),'[object ArrayBuffer]');
        var n=0,o={get buffer(){n++;throw 'get';},get [Symbol.toStringTag](){n++;throw 'tag';}};
        assert.sameValue(A.isView(o),false);assert.sameValue(A.isView(b),false);assert.sameValue(A.isView(),false);assert.sameValue(n,0);
        var cut=p.slice;p.slice=undefined;assert.sameValue(bl.call(cut.bind(b,1)()),2);
    "#,
    );
}

#[test]
fn array_buffer_constructor_conversion_options_and_prototype_order() {
    check(
        r#"
        var A=ArrayBuffer,bl=Object.getOwnPropertyDescriptor(A.prototype,'byteLength').get;
        var log='',N=(function(){}).bind(null),proto={},o={};
        Object.defineProperty(N,'prototype',{get:function(){log+='P';return proto;}});
        Object.defineProperty(o,'maxByteLength',{get:function(){log+='M';assert.sameValue(this,o);return {valueOf:function(){log+='V';return 7;}};}});
        var b=Reflect.construct(A,[{valueOf:function(){log+='L';return 2.9;}},o],N);
        assert.sameValue(log,'LMVP');assert.sameValue(Object.getPrototypeOf(b),proto);assert.sameValue(bl.call(b),2);
        log='';assert.throws(TypeError,function(){A({valueOf:function(){log+='x';}},o);});assert.sameValue(log,'');
        assert.throws(RangeError,function(){Reflect.construct(A,[-1,o],N);});assert.sameValue(log,'');
        assert.throws(RangeError,function(){Reflect.construct(A,[8,o],N);});assert.sameValue(log,'MV');
        var token={},seen;try{new A({valueOf:function(){throw token;}},o);}catch(e){seen=e;}assert.sameValue(seen,token);
        var count=0;Object.defineProperty(Number.prototype,'maxByteLength',{get:function(){count++;throw 'boxed';},configurable:true});
        try{var fixed=new A(2,7);assert.sameValue(fixed.resizable,false);assert.sameValue(count,0);}finally{delete Number.prototype.maxByteLength;}
        var inputs=[undefined,NaN,null,-0,-0.9,2.9,'3'],expect=[0,0,0,0,0,2,3];
        for(var i=0;i<inputs.length;i++)assert.sameValue(new A(inputs[i]).byteLength,expect[i]);
        assert.throws(TypeError,function(){new A(Symbol());});assert.throws(RangeError,function(){new A(Infinity);});
        assert.throws(RangeError,function(){new A(9007199254740992);});
    "#,
    );
}

#[test]
fn array_buffer_host_maximum_policy_is_late_and_does_not_allocate_maximum() {
    check(
        r#"
        var A=ArrayBuffer,log='',N=(function(){}).bind(null);
        Object.defineProperty(N,'prototype',{get:function(){log+='P';return {};}});
        assert.throws(RangeError,function(){Reflect.construct(A,[0,{maxByteLength:8388609}],N);});
        assert.sameValue(log,'P');log='';
        assert.throws(RangeError,function(){Reflect.construct(A,[2,{maxByteLength:1}],N);});assert.sameValue(log,'');
    "#,
    );
    let (mut runtime, mut doc) = fresh();
    let ctor = runtime.array_buffers.intrinsic.as_ref().unwrap().clone();
    let before = runtime.allocated;
    let (_, index) = runtime
        .buffer_allocate(0, Some(MAX_HEAP as u64), ctor, &mut doc)
        .unwrap();
    assert!(runtime.allocated - before < 4096);
    assert!(bytes(&runtime, index).is_empty());
    assert_eq!(
        runtime.array_buffers.records[index].max_byte_length,
        Some(MAX_HEAP as u64)
    );
}

#[test]
fn array_buffer_ordinary_properties_and_integrity_do_not_change_backing_slots() {
    check(
        r#"
        var b=new ArrayBuffer(3,{maxByteLength:6}),p=ArrayBuffer.prototype,bl=Object.getOwnPropertyDescriptor(p,'byteLength').get;
        assert.sameValue(Object.keys(b).length,0);assert.sameValue(JSON.stringify(b),'{}');
        b[1]=7;b[0]=8;b.note=9;Object.defineProperty(b,'byteLength',{value:99,configurable:true});
        assert.sameValue(Object.keys(b).join(','),'0,1,note');assert.sameValue(bl.call(b),3);assert.sameValue(b.byteLength,99);
        Object.freeze(b);assert.sameValue(Object.isFrozen(b),true);b.resize(5);assert.sameValue(bl.call(b),5);
        var t=b.transferToFixedLength(7);assert.sameValue(t.byteLength,7);assert.sameValue(b.detached,true);
        assert.sameValue(b.note,9);assert.sameValue(b[0],8);assert.sameValue(b.byteLength,99);assert.sameValue(t.hasOwnProperty('note'),false);
        assert.sameValue(b.resizable,true);assert.sameValue(b.maxByteLength,0);
    "#,
    );
}

#[test]
fn array_buffer_resize_reentry_and_detached_conversion_order() {
    check(
        r#"
        var p=ArrayBuffer.prototype,r=p.resize,b=new ArrayBuffer(4,{maxByteLength:8});
        r.call(b,{valueOf:function(){b.resize(1);return 6;}});assert.sameValue(b.byteLength,6);
        var moved;assert.throws(TypeError,function(){b.resize({valueOf:function(){moved=b.transfer();return 2;}});});
        assert.sameValue(moved.byteLength,6);assert.sameValue(b.detached,true);var n=0;
        assert.throws(TypeError,function(){r.call(b,{valueOf:function(){n++;return 0;}});});assert.sameValue(n,1);
        assert.throws(RangeError,function(){r.call(b,-1);});
        var fixed=new ArrayBuffer(1);fixed.transfer();assert.throws(TypeError,function(){r.call(fixed,{valueOf:function(){n++;return 0;}});});assert.sameValue(n,1);
        var x=new ArrayBuffer(3,{maxByteLength:4}),token={},seen;
        try{x.resize({valueOf:function(){x.resize(1);throw token;}});}catch(e){seen=e;}
        assert.sameValue(seen,token);assert.sameValue(x.byteLength,1);
    "#,
    );
}

#[test]
fn array_buffer_slice_species_order_result_checks_and_saved_intrinsic() {
    check(
        r#"
        var A=ArrayBuffer,s=A.prototype.slice,b=new A(6),h={},log='';
        Object.defineProperty(b,'constructor',{get:function(){log+='C';return h;},configurable:true});
        Object.defineProperty(h,Symbol.species,{get:function(){log+='S';return C;}});
        function C(n){log+='N';assert.sameValue(n,3);assert.sameValue(arguments.length,1);assert.sameValue(new.target,C);return new A(n,{maxByteLength:7});}
        var out=s.call(b,{valueOf:function(){log+='A';return 1;}},{valueOf:function(){log+='E';return 4;}});
        assert.sameValue(log,'AECSN');assert.sameValue(out.byteLength,3);assert.sameValue(out.resizable,true);
        var source=new A(3),holder={};source.constructor=holder;
        holder[Symbol.species]=function(){return source;};assert.throws(TypeError,function(){s.call(source,0,0);});
        var bad={get byteLength(){throw 'duck brand';}};holder[Symbol.species]=function(){return bad;};assert.throws(TypeError,function(){s.call(source,0,1);});
        holder[Symbol.species]=function(){return new A(0);};assert.throws(TypeError,function(){s.call(source,0,1);});
        source.constructor=undefined;ArrayBuffer=function(){throw 'global';};
        try{assert.sameValue(Object.getPrototypeOf(s.call(source,1)),A.prototype);}finally{ArrayBuffer=A;}
    "#,
    );
}

#[test]
fn array_buffer_slice_detachment_waits_for_species_and_keeps_prior_effects() {
    check(
        r#"
        var A=ArrayBuffer,b=new A(4),h={},log='',out;b.constructor=h;
        h[Symbol.species]=function(n){log+='S';out=new A(n);return out;};
        assert.throws(TypeError,function(){b.slice({valueOf:function(){log+='A';b.transfer();return 1;}},{valueOf:function(){log+='E';return 3;}});});
        assert.sameValue(log,'AES');assert.sameValue(out.byteLength,2);assert.sameValue(b.detached,true);
        var source=new A(4),token={};source.constructor={};source.constructor[Symbol.species]=function(){source.transfer();throw token;};
        var seen;try{source.slice();}catch(e){seen=e;}assert.sameValue(seen,token);assert.sameValue(source.detached,true);
    "#,
    );
}

#[test]
fn array_buffer_transfer_conversion_reentry_limits_and_intrinsic_identity() {
    check(
        r#"
        var A=ArrayBuffer,t=A.prototype.transfer,f=A.prototype.transferToFixedLength;
        var b=new A(4,{maxByteLength:6}),n=0;Object.defineProperty(b,'constructor',{get:function(){throw 'species';}});
        var out=t.call(b,{valueOf:function(){n++;b.resize(2);return 5;}});
        assert.sameValue(n,1);assert.sameValue(out.byteLength,5);assert.sameValue(out.maxByteLength,6);assert.sameValue(b.resizable,true);
        assert.sameValue(b.detached,true);assert.sameValue(b.byteLength,0);assert.sameValue(b.maxByteLength,0);
        assert.throws(RangeError,function(){t.call(out,7);});assert.sameValue(out.byteLength,5);assert.sameValue(out.detached,false);
        var fixed=f.call(out,7);assert.sameValue(fixed.byteLength,7);assert.sameValue(fixed.resizable,false);
        assert.sameValue(fixed.maxByteLength,7);assert.sameValue(Object.getPrototypeOf(fixed),A.prototype);
        var inner,x=new A(3);assert.throws(TypeError,function(){t.call(x,{valueOf:function(){inner=t.call(x,1);return 2;}});});
        assert.sameValue(inner.byteLength,1);assert.sameValue(x.detached,true);
        assert.throws(RangeError,function(){t.call(x,-1);});var hits=0;
        assert.throws(TypeError,function(){t.call(x,{valueOf:function(){hits++;return 1;}});});assert.sameValue(hits,1);
    "#,
    );
}

// Byte expectations were frozen independently before implementation:
// /tmp/eris-array-buffer-backing-oracle.json, SHA-256
// fbdf470c1973384361525b0adb94c8dca1d9c442e84d9f64c73d37aeac71d230.
// No /tmp data is included or required to compile/run these tests.
#[test]
fn array_buffer_bytes_zero_initialization_and_shrink_regrow() {
    let (mut runtime, mut doc) = fresh();
    let ctor = runtime.array_buffers.intrinsic.as_ref().unwrap().clone();
    let (_, zero) = runtime.buffer_allocate(4, None, ctor, &mut doc).unwrap();
    assert_eq!(bytes(&runtime, zero), [0, 0, 0, 0]);
    let (empty, empty_index) = make(&mut runtime, &mut doc, &[], None);
    let moved = runtime
        .buffer_transfer(empty_index, Value::Undefined, true, &mut doc)
        .unwrap();
    let moved_index = runtime.buffer_record(&moved).unwrap();
    assert_ne!(moved, empty);
    assert!(runtime.array_buffers.records[empty_index].bytes.is_none());
    assert!(bytes(&runtime, moved_index).is_empty());
    assert_eq!(
        runtime.array_buffers.records[moved_index]
            .bytes
            .as_ref()
            .unwrap()
            .capacity(),
        0
    );
    let (_, index) = make(&mut runtime, &mut doc, &[1, 2, 3, 4], Some(8));
    runtime
        .buffer_resize(index, Value::Number(2.0), &mut doc)
        .unwrap();
    runtime
        .buffer_resize(index, Value::Number(5.0), &mut doc)
        .unwrap();
    assert_eq!(bytes(&runtime, index), [1, 2, 0, 0, 0]);
    assert_eq!(
        runtime.array_buffers.records[index].max_byte_length,
        Some(8)
    );
}

#[test]
fn array_buffer_bytes_slice_shrink_and_grow_preserve_existing_target_suffix() {
    for grow in [false, true] {
        let (mut runtime, mut doc) = fresh();
        // Target precedes source here, exercising both split_at_mut directions
        // across this group and fresh-target slice elsewhere.
        let (target, target_index) = make(&mut runtime, &mut doc, &[9, 9, 9, 9, 9], None);
        let initial: &[u8] = if grow {
            &[1, 2, 3, 4]
        } else {
            &[1, 2, 3, 4, 5, 6]
        };
        let (source, source_index) = make(&mut runtime, &mut doc, initial, Some(8));
        bind(&mut runtime, "source", source.clone());
        bind(&mut runtime, "target", target.clone());
        runtime.execute(if grow {
            "source.constructor={};source.constructor[Symbol.species]=function(n){if(n!==3)throw 'length';source.resize(6);return target;};"
        } else {
            "source.constructor={};source.constructor[Symbol.species]=function(n){if(n!==3)throw 'length';source.resize(2);return target;};"
        }, &mut doc).unwrap();
        let args = if grow {
            vec![Value::Number(1.0)]
        } else {
            vec![Value::Number(1.0), Value::Number(4.0)]
        };
        let result = runtime
            .buffer_slice(source, source_index, &args, &mut doc)
            .unwrap();
        assert_eq!(result, target);
        assert_eq!(
            bytes(&runtime, target_index),
            if grow {
                &[2, 3, 4, 9, 9]
            } else {
                &[2, 9, 9, 9, 9]
            }
        );
        assert_eq!(
            bytes(&runtime, source_index),
            if grow {
                &[1, 2, 3, 4, 0, 0][..]
            } else {
                &[1, 2][..]
            }
        );
        clean(&runtime);
    }
}

#[test]
fn array_buffer_bytes_slice_zero_copy_and_reentrant_side_table_growth() {
    let (mut runtime, mut doc) = fresh();
    let (source, index) = make(&mut runtime, &mut doc, &[1, 2, 3, 4], Some(8));
    let (target, target_index) = make(&mut runtime, &mut doc, &[9, 9, 9, 9, 9], None);
    bind(&mut runtime, "source", source.clone());
    bind(&mut runtime, "target", target.clone());
    runtime.execute("source.constructor={};source.constructor[Symbol.species]=function(n){source.resize(0);for(var i=0;i<17;i++)new ArrayBuffer(0);return target;};", &mut doc).unwrap();
    let old_capacity = runtime.array_buffers.records.capacity();
    let result = runtime
        .buffer_slice(source, index, &[Value::Number(2.0)], &mut doc)
        .unwrap();
    assert_eq!(result, target);
    assert!(runtime.array_buffers.records.capacity() > old_capacity);
    assert_eq!(bytes(&runtime, target_index), [9, 9, 9, 9, 9]);
    assert_eq!(bytes(&runtime, index), []);
    clean(&runtime);
}

#[test]
fn array_buffer_bytes_transfer_copies_current_prefix_and_zeroes_extension() {
    let (mut runtime, mut doc) = fresh();
    let (source, index) = make(&mut runtime, &mut doc, &[1, 2, 3, 4], Some(8));
    bind(&mut runtime, "source", source);
    let argument = runtime
        .execute(
            "({valueOf:function(){source.resize(2);return 6;}})",
            &mut doc,
        )
        .unwrap();
    let result = runtime
        .buffer_transfer(index, argument, true, &mut doc)
        .unwrap();
    let target = runtime.buffer_record(&result).unwrap();
    assert_eq!(bytes(&runtime, target), [1, 2, 0, 0, 0, 0]);
    assert_eq!(
        runtime.array_buffers.records[target].max_byte_length,
        Some(8)
    );
    assert!(runtime.array_buffers.records[index].bytes.is_none());
    assert_eq!(
        runtime.array_buffers.records[index].max_byte_length,
        Some(8)
    );
    let (_, fixed_source) = make(&mut runtime, &mut doc, &[1, 2], Some(4));
    let result = runtime
        .buffer_transfer(fixed_source, Value::Number(7.0), false, &mut doc)
        .unwrap();
    let fixed_target = runtime.buffer_record(&result).unwrap();
    assert_eq!(bytes(&runtime, fixed_target), [1, 2, 0, 0, 0, 0, 0]);
    assert_eq!(
        runtime.array_buffers.records[fixed_target].max_byte_length,
        None
    );
    assert!(runtime.array_buffers.records[fixed_source].bytes.is_none());
}

#[test]
fn array_buffer_zero_fill_and_record_growth_refuse_before_publication() {
    let (mut runtime, _) = fresh();
    let allocation = runtime.allocated;
    runtime.steps = byte_work(33) - 1;
    assert!(
        runtime
            .buffer_empty_block(33)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.allocated, allocation);
    runtime.steps = MAX_STEPS;
    runtime.allocated = MAX_HEAP - 32;
    assert!(
        runtime
            .buffer_empty_block(33)
            .unwrap_err()
            .is_resource_limit()
    );
    assert!(runtime.array_buffers.records.is_empty());
    let (mut runtime, mut doc) = fresh();
    let ctor = runtime.array_buffers.intrinsic.as_ref().unwrap().clone();
    let before = (runtime.steps, runtime.allocated);
    runtime.buffer_allocate(0, None, ctor, &mut doc).unwrap();
    let cost = before.0 - runtime.steps;
    let allocation_cost = runtime.allocated - before.1;
    let (mut replay, mut doc) = fresh();
    let ctor = replay.array_buffers.intrinsic.as_ref().unwrap().clone();
    replay.steps = cost - 1;
    assert!(
        replay
            .buffer_allocate(0, None, ctor, &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert!(replay.array_buffers.records.is_empty());
    assert_eq!(replay.array_buffers.records.capacity(), 0);
    let (mut replay, mut doc) = fresh();
    let ctor = replay.array_buffers.intrinsic.as_ref().unwrap().clone();
    replay.allocated = MAX_HEAP - allocation_cost + 1;
    assert!(
        replay
            .buffer_allocate(0, None, ctor, &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert!(replay.array_buffers.records.is_empty());
    assert_eq!(replay.array_buffers.records.capacity(), 0);
}

#[test]
fn array_buffer_resize_work_and_full_growth_heap_are_transactional() {
    let (mut measure, mut doc) = fresh();
    let (_, index) = make(&mut measure, &mut doc, &[1, 2, 3, 4, 5, 6, 7, 8], Some(16));
    measure
        .buffer_resize(index, Value::Number(2.0), &mut doc)
        .unwrap();
    let start = (measure.steps, measure.allocated);
    measure
        .buffer_resize(index, Value::Number(5.0), &mut doc)
        .unwrap();
    let work = start.0 - measure.steps;
    assert_eq!(measure.allocated, start.1);
    let (mut runtime, mut doc) = fresh();
    let (_, index) = make(&mut runtime, &mut doc, &[1, 2, 3, 4, 5, 6, 7, 8], Some(16));
    runtime
        .buffer_resize(index, Value::Number(2.0), &mut doc)
        .unwrap();
    runtime.steps = work - 1;
    assert!(
        runtime
            .buffer_resize(index, Value::Number(5.0), &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(bytes(&runtime, index), [1, 2]);
    runtime.steps = MAX_STEPS;
    let length = runtime.array_buffers.records[index]
        .bytes
        .as_ref()
        .unwrap()
        .capacity()
        + 1;
    assert!(length <= 16);
    runtime.allocated = MAX_HEAP - (length - 1);
    assert!(
        runtime
            .buffer_resize(index, Value::Number(length as f64), &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(bytes(&runtime, index), [1, 2]);
}

fn transfer_setup() -> (Runtime, Document, usize, Value) {
    let (mut runtime, mut doc) = fresh();
    let (source, index) = make(&mut runtime, &mut doc, &[1, 2, 3, 4], Some(8));
    bind(&mut runtime, "source", source);
    let argument = runtime
        .execute(
            "var effects=0;({valueOf:function(){effects++;source.resize(2);return 6;}})",
            &mut doc,
        )
        .unwrap();
    (runtime, doc, index, argument)
}

#[test]
fn array_buffer_transfer_copy_cut_retains_post_callback_source_and_zero_destination() {
    let (mut measure, mut doc, index, argument) = transfer_setup();
    let start = measure.steps;
    measure
        .buffer_transfer(index, argument, true, &mut doc)
        .unwrap();
    let work = start - measure.steps;
    let (mut runtime, mut doc, index, argument) = transfer_setup();
    runtime.steps = work - 1;
    assert!(
        runtime
            .buffer_transfer(index, argument, true, &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.lookup(1, "effects").unwrap().1, Value::Number(1.0));
    assert_eq!(bytes(&runtime, index), [1, 2]);
    assert_eq!(
        runtime.array_buffers.records[index].max_byte_length,
        Some(8)
    );
    assert_eq!(runtime.array_buffers.records.len(), 2);
    assert_eq!(bytes(&runtime, 1), [0, 0, 0, 0, 0, 0]);
    assert_ne!(
        runtime.array_buffers.records[index].object_id,
        runtime.array_buffers.records[1].object_id
    );
    clean(&runtime);
}

#[test]
fn array_buffer_transfer_heap_cut_never_detaches_source() {
    let (mut measure, mut doc, index, argument) = transfer_setup();
    let start = measure.allocated;
    measure
        .buffer_transfer(index, argument, true, &mut doc)
        .unwrap();
    let cost = measure.allocated - start;
    let (mut runtime, mut doc, index, argument) = transfer_setup();
    runtime.allocated = MAX_HEAP - cost + 1;
    assert!(
        runtime
            .buffer_transfer(index, argument, true, &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.lookup(1, "effects").unwrap().1, Value::Number(1.0));
    assert_eq!(bytes(&runtime, index), [1, 2]);
    clean(&runtime);
}

fn slice_setup() -> (Runtime, Document, Value, usize, usize) {
    let (mut runtime, mut doc) = fresh();
    let (source, index) = make(&mut runtime, &mut doc, &[1, 2, 3, 4], None);
    let (target, target_index) = make(&mut runtime, &mut doc, &[9, 9, 9, 9, 9], None);
    bind(&mut runtime, "source", source.clone());
    bind(&mut runtime, "target", target);
    runtime.execute("var effects=0;source.constructor={};source.constructor[Symbol.species]=function(){effects++;return target;};", &mut doc).unwrap();
    (runtime, doc, source, index, target_index)
}

#[test]
fn array_buffer_slice_copy_cut_does_not_modify_preexposed_species_output() {
    let (mut measure, mut doc, source, index, _) = slice_setup();
    let start = measure.steps;
    measure
        .buffer_slice(source, index, &[Value::Number(1.0)], &mut doc)
        .unwrap();
    let work = start - measure.steps;
    let (mut runtime, mut doc, source, index, target) = slice_setup();
    runtime.steps = work - 1;
    assert!(
        runtime
            .buffer_slice(source, index, &[Value::Number(1.0)], &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.lookup(1, "effects").unwrap().1, Value::Number(1.0));
    assert_eq!(bytes(&runtime, target), [9, 9, 9, 9, 9]);
    assert_eq!(bytes(&runtime, index), [1, 2, 3, 4]);
    clean(&runtime);
}

#[test]
fn array_buffer_record_search_growth_and_cheap_getters_do_not_scan_bytes() {
    let (mut runtime, mut doc) = fresh();
    let mut values = Vec::new();
    for _ in 0..64 {
        values.push(make(&mut runtime, &mut doc, &[], None).0);
    }
    values.push(make(&mut runtime, &mut doc, &[7; 4096], None).0);
    let start = runtime.steps;
    let index = runtime.buffer_record(&values[64]).unwrap();
    assert_eq!(index, 64);
    assert!(start - runtime.steps <= 9);
    let capacity = runtime.array_buffers.records.capacity();
    assert!((65..132).contains(&capacity));
    let allocation = runtime.allocated;
    runtime.steps = 9;
    let output = runtime
        .array_buffer_native("getByteLength", values[64].clone(), &[], &mut doc)
        .unwrap();
    assert_eq!(output, Value::Number(4096.0));
    assert_eq!(runtime.allocated, allocation);
    runtime.steps = 1;
    assert_eq!(
        runtime
            .array_buffer_native("isView", Value::Undefined, &values, &mut doc)
            .unwrap(),
        Value::Bool(false)
    );
    assert_eq!(runtime.steps, 0);
    assert_eq!(runtime.allocated, allocation);
}

#[test]
fn array_buffer_ignored_extra_arguments_and_detach_never_refund_storage() {
    let (mut runtime, mut doc) = fresh();
    let (source, index) = make(&mut runtime, &mut doc, &[1, 2, 3, 4], Some(8));
    let huge = Value::String(JsString::from("x".repeat(MAX_STRING)));
    let start = runtime.allocated;
    let _ = runtime
        .array_buffer_native(
            "getByteLength",
            source.clone(),
            std::slice::from_ref(&huge),
            &mut doc,
        )
        .unwrap();
    assert_eq!(runtime.allocated, start);
    runtime
        .buffer_resize(index, Value::Number(0.0), &mut doc)
        .unwrap();
    assert_eq!(runtime.allocated, start);
    runtime
        .array_buffer_native("transfer", source, &[Value::Undefined, huge], &mut doc)
        .unwrap();
    assert!(runtime.allocated > start);
    assert!(runtime.array_buffers.records[index].bytes.is_none());
}

#[test]
fn array_buffer_repeated_creation_shares_cumulative_heap_without_refunds() {
    let (mut runtime, mut doc) = fresh();
    let ctor = runtime.array_buffers.intrinsic.as_ref().unwrap().clone();
    runtime.allocated = MAX_HEAP - 12_000;
    let mut succeeded = 0;
    loop {
        let before = runtime.allocated;
        match runtime.buffer_allocate(1, None, ctor.clone(), &mut doc) {
            Ok((_, index)) => {
                assert!(runtime.allocated > before);
                runtime.array_buffers.records[index].bytes = None;
                assert!(runtime.allocated > before);
                succeeded += 1;
            }
            Err(error) => {
                assert!(error.is_resource_limit(), "{error:?}");
                break;
            }
        }
    }
    assert!(succeeded >= 2);
    assert_eq!(runtime.array_buffers.records.len(), succeeded);
    assert!(runtime.allocated > MAX_HEAP);
    assert!(runtime.steps > 0);
}

#[test]
fn array_buffer_terminal_recursion_and_allocation_skip_author_cleanup() {
    for body in [
        "var n={valueOf:function(){effects++;return new ArrayBuffer(n);}};new ArrayBuffer(n);",
        "var b=new ArrayBuffer(1),h={};h[Symbol.species]=function(){effects++;return b.slice();};b.constructor=h;b.slice();",
        "while(true){effects++;new ArrayBuffer(1);}",
    ] {
        let (mut runtime, mut doc) = fresh();
        let result = runtime.execute(&format!("var effects=0,caught=0,finalized=0;try{{{body}}}catch(e){{caught++;}}finally{{finalized++;}}"), &mut doc);
        assert!(result.unwrap_err().is_resource_limit());
        assert!(runtime.lookup(1, "effects").unwrap().1.number() > 0.0);
        assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Number(0.0));
        assert_eq!(
            runtime.lookup(1, "finalized").unwrap().1,
            Value::Number(0.0)
        );
        clean(&runtime);
    }
}

#[test]
fn array_buffer_reached_host_and_cycle_boundaries_preserve_order() {
    let (mut runtime, mut doc) = fresh();
    let options = runtime.object_ordered([]).unwrap();
    let Value::Object(id) = options.clone() else {
        unreachable!()
    };
    runtime.objects[id].prototype = Some(Value::Window);
    runtime.objects[id].insert("maxByteLength".into(), Value::Number(4.0));
    let ctor = runtime.array_buffers.intrinsic.as_ref().unwrap().clone();
    runtime
        .array_buffer_constructor(
            &[Value::Number(1.0), options.clone()],
            ctor.clone(),
            &mut doc,
        )
        .unwrap();
    runtime.objects[id].remove(&"maxByteLength".into());
    assert!(
        runtime
            .array_buffer_constructor(
                &[Value::Number(1.0), options.clone()],
                ctor.clone(),
                &mut doc
            )
            .unwrap_err()
            .is_unsupported()
    );
    runtime.objects[id].prototype = Some(options.clone());
    assert!(
        runtime
            .array_buffer_constructor(&[Value::Number(1.0), options], ctor, &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    clean(&runtime);
}

#[test]
fn array_buffer_absolute_capacity_rejects_before_fictional_byte_work_or_storage() {
    let (mut runtime, mut doc) = fresh();
    let before = (runtime.steps, runtime.allocated);
    let error = runtime.buffer_empty_block(MAX_HEAP as u64 + 1).unwrap_err();
    assert_eq!(error.kind, ErrorKind::Runtime("RangeError"));
    assert_eq!((runtime.steps, runtime.allocated), before);
    let ctor = runtime.array_buffers.intrinsic.as_ref().unwrap().clone();
    let objects = runtime.objects.len();
    let error = runtime
        .buffer_allocate(MAX_HEAP as u64 + 1, None, ctor, &mut doc)
        .unwrap_err();
    assert_eq!(error.kind, ErrorKind::Runtime("RangeError"));
    assert_eq!(runtime.objects.len(), objects + 1);
    assert!(runtime.array_buffers.records.is_empty());
    assert!(before.0 - runtime.steps < 1000);
    assert!(runtime.allocated > before.1 && runtime.allocated - before.1 < 1024);
    check(
        r#"
        var token={},seen,N=(function(){}).bind(null),hits=0;
        Object.defineProperty(N,'prototype',{get:function(){hits++;throw token;}});
        try{Reflect.construct(ArrayBuffer,[9007199254740991],N);}catch(e){seen=e;}
        assert.sameValue(seen,token);assert.sameValue(hits,1);
    "#,
    );
}

#[test]
fn array_buffer_impossible_fixed_transfer_preserves_callback_effects_and_source() {
    let (mut runtime, mut doc) = fresh();
    let (source, index) = make(&mut runtime, &mut doc, &[1, 2, 3, 4], Some(8));
    bind(&mut runtime, "source", source);
    let argument = runtime
        .execute(
            "var effects=0;({valueOf:function(){effects++;source.resize(2);return 8388609;}})",
            &mut doc,
        )
        .unwrap();
    let error = runtime
        .buffer_transfer(index, argument, false, &mut doc)
        .unwrap_err();
    assert_eq!(error.kind, ErrorKind::Runtime("RangeError"));
    assert_eq!(bytes(&runtime, index), [1, 2]);
    assert_eq!(runtime.lookup(1, "effects").unwrap().1, Value::Number(1.0));
    assert_eq!(runtime.array_buffers.records.len(), 1);
    clean(&runtime);
}
