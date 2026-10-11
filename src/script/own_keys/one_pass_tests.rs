use super::*;

// Independent proposal tests. The one-pass production draft was not read.
fn fresh() -> (Runtime, Document) {
    (Runtime::try_new().unwrap(), Document::parse("<p>kept</p>"))
}

fn names(keys: &[&str]) -> Vec<JsString> {
    keys.iter().map(|key| (*key).into()).collect()
}

fn ordinary(runtime: &mut Runtime, keys: &[&str]) -> Value {
    runtime
        .object_ordered(keys.iter().map(|key| ((*key).into(), Value::Bool(true))))
        .unwrap()
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

struct Kept {
    order: Vec<PropertyKey>,
    values: String,
    pointers: Vec<*const u16>,
    objects: usize,
}

fn keep(runtime: &Runtime, id: usize) -> Kept {
    Kept {
        order: runtime.objects[id].order.clone(),
        values: format!("{:?}", runtime.objects[id].values),
        pointers: runtime.objects[id]
            .order
            .iter()
            .filter_map(|k| k.as_string().map(|s| s.units().as_ptr()))
            .collect(),
        objects: runtime.objects.len(),
    }
}

fn unchanged(runtime: &Runtime, id: usize, kept: &Kept) {
    assert_eq!(runtime.objects.len(), kept.objects);
    assert_eq!(runtime.objects[id].order, kept.order);
    assert_eq!(format!("{:?}", runtime.objects[id].values), kept.values);
    assert_eq!(
        runtime.objects[id]
            .order
            .iter()
            .filter_map(|k| k.as_string().map(|s| s.units().as_ptr()))
            .collect::<Vec<_>>(),
        kept.pointers
    );
    clean(runtime);
}

const EIGHT: [&str; 8] = ["z", "a", "m", "b", "y", "c", "x", "d"];

#[test]
fn complete_literal_shapes_pay_reacquisition_and_only_reached_capacity() {
    let rows = [
        ("({z:0,a:1,m:2,b:3,y:4,c:5,x:6,d:7})", 96, 8),
        (
            "var a={z:0};a[Symbol('one')]=0;a.a=1;a.m=2;a.b=3;a.y=4;a.c=5;a.x=6;a.d=7;a[Symbol('two')]=0;a;",
            108,
            10,
        ),
        (
            "var a={};a[Symbol('one')]=0;a[Symbol('two')]=0;a.z=0;a.a=1;a.m=2;a.b=3;a.y=4;a.c=5;a.x=6;a.d=7;a;",
            108,
            8,
        ),
    ];
    for (source, work, slots) in rows {
        let (mut runtime, mut doc) = fresh();
        let value = runtime.execute(source, &mut doc).unwrap();
        let id = runtime.property_object(&value).unwrap();
        let kept = keep(&runtime, id);
        let start = (runtime.steps, runtime.allocated);
        let keys = runtime.own_keys(&value).unwrap();
        assert_eq!(keys, names(&EIGHT));
        assert_eq!(
            (start.0 - runtime.steps, runtime.allocated - start.1),
            (work, slots * std::mem::size_of::<JsString>())
        );
        // try_reserve_exact promises at least the requested capacity; charges
        // and absence of later growth are checked without assuming allocator rounding.
        assert!(keys.capacity() >= slots);
        assert_eq!(
            keys.iter().map(|s| s.units().as_ptr()).collect::<Vec<_>>(),
            kept.pointers
        );
        unchanged(&runtime, id, &kept);
    }
}

#[test]
fn retained_utf16_payload_lengths_do_not_change_work_or_copy_text() {
    for length in [2, 512, 4096] {
        let (mut runtime, _) = fresh();
        let keys: Vec<JsString> = (0..8)
            .map(|i| {
                let mut units = vec![0xd800; length];
                units[length - 1] = 48 + i;
                units.into()
            })
            .collect();
        let value = runtime
            .object_ordered(keys.iter().cloned().map(|k| (k, Value::Bool(true))))
            .unwrap();
        let id = runtime.property_object(&value).unwrap();
        let kept = keep(&runtime, id);
        let start = (runtime.steps, runtime.allocated);
        let out = runtime.own_keys(&value).unwrap();
        assert_eq!(out, keys);
        assert_eq!(
            (start.0 - runtime.steps, runtime.allocated - start.1),
            (96, 8 * std::mem::size_of::<JsString>())
        );
        assert_eq!(
            out.iter().map(|s| s.units().as_ptr()).collect::<Vec<_>>(),
            kept.pointers
        );
        unchanged(&runtime, id, &kept);
    }
    let (mut runtime, _) = fresh();
    let keys: Vec<JsString> = vec![
        "".into(),
        vec![0, 120].into(),
        vec![0xd800].into(),
        vec![0xdc00].into(),
        vec![0xffff].into(),
        "é".into(),
        "+1".into(),
        "-0".into(),
    ];
    let value = runtime
        .object_ordered(keys.iter().cloned().map(|k| (k, Value::Bool(true))))
        .unwrap();
    let start = (runtime.steps, runtime.allocated);
    let out = runtime.own_keys(&value).unwrap();
    assert_eq!(out, keys);
    assert_eq!(
        (start.0 - runtime.steps, runtime.allocated - start.1),
        (96, 8 * std::mem::size_of::<JsString>())
    );
}

#[test]
fn first_digit_rejects_before_any_reservation_including_leading_symbols() {
    for digit in [
        "0",
        "01",
        "10",
        "2",
        "4294967295",
        "1234567890123456789012345678901234567890",
    ] {
        for symbols in [0, 2] {
            let (mut runtime, mut doc) = fresh();
            let value = runtime
                .execute(
                    if symbols == 0 {
                        "({})"
                    } else {
                        "var a={};a[Symbol('s')]=1;a[Symbol('t')]=2;a;"
                    },
                    &mut doc,
                )
                .unwrap();
            let id = runtime.property_object(&value).unwrap();
            runtime.objects[id].insert(digit.into(), Value::Bool(true));
            for key in EIGHT {
                runtime.objects[id].insert(key.into(), Value::Bool(true));
            }
            let kept = keep(&runtime, id);
            let work = 10 + 6 * symbols;
            for allowance in [work - 1, work] {
                runtime.steps = allowance;
                runtime.allocated = MAX_HEAP;
                let out = runtime.own_key_stored_strings(id);
                if allowance == work {
                    assert!(out.unwrap().is_none());
                } else {
                    assert!(out.unwrap_err().is_resource_limit());
                }
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
                unchanged(&runtime, id, &kept);
            }
        }
    }
}

#[test]
fn late_rejection_pays_retained_handles_and_unchanged_generic_ordering() {
    for (keys, helper, whole) in [
        (["01", "z", "a", "m", "b", "y", "c", "x"], 10, 156),
        (["z", "01", "a", "m", "b", "y", "c", "x"], 28, 174),
        (["z", "a", "m", "b", "y", "c", "x", "01"], 88, 234),
    ] {
        let (mut runtime, _) = fresh();
        let value = ordinary(&mut runtime, &keys);
        let id = runtime.property_object(&value).unwrap();
        let kept = keep(&runtime, id);
        let start = (runtime.steps, runtime.allocated);
        assert!(runtime.own_key_stored_strings(id).unwrap().is_none());
        let temporary = if helper == 10 {
            0
        } else {
            8 * std::mem::size_of::<JsString>()
        };
        assert_eq!(
            (start.0 - runtime.steps, runtime.allocated - start.1),
            (helper, temporary)
        );
        let start = (runtime.steps, runtime.allocated);
        let out = runtime.own_keys(&value).unwrap();
        assert_eq!(out, names(&keys));
        assert_eq!(
            (start.0 - runtime.steps, runtime.allocated - start.1),
            (
                whole,
                temporary + 8 * (std::mem::size_of::<OwnKey>() + std::mem::size_of::<JsString>())
            )
        );
        assert_eq!(
            out.iter().map(|s| s.units().as_ptr()).collect::<Vec<_>>(),
            kept.pointers
        );
        unchanged(&runtime, id, &kept);
    }
    let (mut runtime, _) = fresh();
    let value = ordinary(
        &mut runtime,
        &["tail", "01", "4294967295", "10", "2", "0", "alpha", "beta"],
    );
    let id = runtime.property_object(&value).unwrap();
    let kept = keep(&runtime, id);
    let expected = names(&["0", "2", "10", "tail", "01", "4294967295", "alpha", "beta"]);
    let out = runtime.own_keys(&value).unwrap();
    assert_eq!(out, expected);
    for key in out {
        let original = kept
            .order
            .iter()
            .find_map(|k| k.as_string().filter(|s| *s == &key))
            .unwrap();
        assert_eq!(key.units().as_ptr(), original.units().as_ptr());
    }
    unchanged(&runtime, id, &kept);
}

#[test]
fn every_success_path_work_cut_retires_scratch_without_publishing_state() {
    let heap = 8 * std::mem::size_of::<JsString>();
    // route4 + helper4 + probe6 + reserve8 reaches allocation at22.
    // First reacquire/clone4 ends26; seven further6+4 stages end96.
    for allowance in 0..=96 {
        let (mut runtime, _) = fresh();
        let value = ordinary(&mut runtime, &EIGHT);
        let id = runtime.property_object(&value).unwrap();
        let kept = keep(&runtime, id);
        runtime.steps = allowance;
        runtime.allocated = 0;
        let out = runtime.own_keys(&value);
        if allowance == 96 {
            assert_eq!(out.unwrap(), names(&EIGHT));
        } else {
            assert!(out.unwrap_err().is_resource_limit());
        }
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, if allowance >= 22 { heap } else { 0 });
        unchanged(&runtime, id, &kept);
    }
}

#[test]
fn early_heap_refusal_precedes_remaining_probes_and_keeps_actual_charge() {
    for (shape, work, allocation_at, slots) in [(0, 96, 22, 8), (1, 108, 34, 8), (2, 108, 22, 10)] {
        let heap = slots * std::mem::size_of::<JsString>();
        for room in [0, heap - 1, heap] {
            let (mut runtime, mut doc) = fresh();
            let value = if shape == 1 {
                runtime.execute("var a={};a[Symbol('s')]=0;a[Symbol('t')]=0;a.z=0;a.a=1;a.m=2;a.b=3;a.y=4;a.c=5;a.x=6;a.d=7;a;",&mut doc).unwrap()
            } else if shape == 2 {
                runtime.execute("var a={z:0,a:1,m:2,b:3,y:4,c:5,x:6,d:7};a[Symbol('s')]=0;a[Symbol('t')]=0;a;",&mut doc).unwrap()
            } else {
                ordinary(&mut runtime, &EIGHT)
            };
            let id = runtime.property_object(&value).unwrap();
            let kept = keep(&runtime, id);
            runtime.steps = work;
            runtime.allocated = MAX_HEAP - room;
            let out = runtime.own_keys(&value);
            if room == heap {
                assert_eq!(out.unwrap(), names(&EIGHT));
                assert_eq!(runtime.steps, 0);
            } else {
                assert!(out.unwrap_err().is_resource_limit());
                assert_eq!(runtime.steps, work - allocation_at);
            }
            assert_eq!(runtime.allocated, MAX_HEAP - room + heap);
            unchanged(&runtime, id, &kept);
        }
    }
}

#[test]
fn all_symbols_and_empty_direct_helper_require_no_output_heap() {
    for allowance in 0..=56 {
        let (mut runtime, mut doc) = fresh();
        let value = runtime
            .execute("var a={};for(var i=0;i<8;i++)a[Symbol('s')]=i;a;", &mut doc)
            .unwrap();
        let id = runtime.property_object(&value).unwrap();
        let kept = keep(&runtime, id);
        runtime.steps = allowance;
        runtime.allocated = MAX_HEAP;
        let out = runtime.own_keys(&value);
        if allowance == 56 {
            let keys = out.unwrap();
            assert!(keys.is_empty());
            assert_eq!(keys.capacity(), 0);
        } else {
            assert!(out.unwrap_err().is_resource_limit());
        }
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        unchanged(&runtime, id, &kept);
    }
    for allowance in [3, 4] {
        let (mut runtime, _) = fresh();
        let value = ordinary(&mut runtime, &[]);
        let id = runtime.property_object(&value).unwrap();
        runtime.steps = allowance;
        runtime.allocated = MAX_HEAP;
        let out = runtime.own_key_stored_strings(id);
        if allowance == 4 {
            assert!(out.unwrap().unwrap().is_empty());
        } else {
            assert!(out.unwrap_err().is_resource_limit());
        }
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    }
}

#[test]
fn late_generic_fallback_has_two_distinct_allocation_and_sort_admissions() {
    let keys = ["z", "a", "m", "b", "y", "c", "x", "01"];
    let temporary = 8 * std::mem::size_of::<JsString>();
    let generic = 8 * (std::mem::size_of::<OwnKey>() + std::mem::size_of::<JsString>());
    // caller4 + helper88 + count9 + retained112 + sorted-window21 =234.
    // Temporary storage at22; unchanged generic allocation at213.
    for allowance in [
        0, 21, 22, 25, 26, 81, 82, 87, 88, 91, 92, 100, 101, 212, 213, 215, 216, 233, 234,
    ] {
        let (mut runtime, _) = fresh();
        let value = ordinary(&mut runtime, &keys);
        let id = runtime.property_object(&value).unwrap();
        let kept = keep(&runtime, id);
        runtime.steps = allowance;
        runtime.allocated = 0;
        let out = runtime.own_keys(&value);
        if allowance == 234 {
            assert_eq!(out.unwrap(), names(&keys));
        } else {
            assert!(out.unwrap_err().is_resource_limit());
        }
        assert_eq!(runtime.steps, 0);
        assert_eq!(
            runtime.allocated,
            if allowance >= 213 {
                temporary + generic
            } else if allowance >= 22 {
                temporary
            } else {
                0
            }
        );
        unchanged(&runtime, id, &kept);
    }
    for room in [
        0,
        temporary - 1,
        temporary,
        temporary + generic - 1,
        temporary + generic,
    ] {
        let (mut runtime, _) = fresh();
        let value = ordinary(&mut runtime, &keys);
        let id = runtime.property_object(&value).unwrap();
        let kept = keep(&runtime, id);
        runtime.steps = 234;
        runtime.allocated = MAX_HEAP - room;
        let out = runtime.own_keys(&value);
        let charged = if room < temporary {
            temporary
        } else {
            temporary + generic
        };
        if room == temporary + generic {
            assert_eq!(out.unwrap(), names(&keys));
            assert_eq!(runtime.steps, 0);
        } else {
            assert!(out.unwrap_err().is_resource_limit());
            assert_eq!(runtime.steps, if room < temporary { 212 } else { 21 });
        }
        assert_eq!(runtime.allocated, MAX_HEAP - room + charged);
        unchanged(&runtime, id, &kept);
    }
}

#[test]
fn caller_virtual_dense_and_small_order_domains_remain_literal() {
    for (source, expected) in [
        ("({})", names(&[])),
        ("({z:1,a:2})", names(&["z", "a"])),
        (
            "var a=[0,1,2];delete a[1];a.z=0;a.a=1;a.m=2;a.b=3;a.y=4;a.c=5;a.x=6;a.d=7;a;",
            names(&["0", "2", "length", "z", "a", "m", "b", "y", "c", "x", "d"]),
        ),
        (
            "var a=Object('xy');a.z=0;a.a=1;a.m=2;a.b=3;a.y=4;a.c=5;a.x=6;a.d=7;a;",
            names(&["0", "1", "length", "z", "a", "m", "b", "y", "c", "x", "d"]),
        ),
        (
            "var a=new Uint8Array([1,2]);a.z=0;a.a=1;a.m=2;a.b=3;a.y=4;a.c=5;a.x=6;a.d=7;a;",
            names(&["0", "1", "z", "a", "m", "b", "y", "c", "x", "d"]),
        ),
    ] {
        let (mut runtime, mut doc) = fresh();
        let value = runtime.execute(source, &mut doc).unwrap();
        assert_eq!(runtime.own_keys(&value).unwrap(), expected);
        clean(&runtime);
    }
    for with_record in [false, true] {
        let (mut runtime, mut doc) = fresh();
        if with_record {
            runtime.execute("new Uint8Array(0);", &mut doc).unwrap();
        }
        let value = ordinary(&mut runtime, &EIGHT);
        let start = (runtime.steps, runtime.allocated);
        let (object, keys) = runtime.for_in_snapshot(value.clone()).unwrap();
        assert_eq!(object.value(), &value);
        assert_eq!(keys, names(&EIGHT));
        // No records: selector4. One genuine negative lookup:4, selector4, proof8.
        assert_eq!(
            (start.0 - runtime.steps, runtime.allocated - start.1),
            (
                if with_record { 112 } else { 100 },
                8 * std::mem::size_of::<JsString>()
            )
        );
        clean(&runtime);
    }
}

#[test]
fn full_for_in_keeps_descriptors_prototypes_and_typed_bounds_live() {
    let source = r#"(function(){
        var guard=new Uint8Array(0),old={old:1},next={n:1,shadow:2};
        var o={z:0,a:1,m:2,b:3,y:4,c:5,x:6,d:7};
        Object.defineProperty(o,'shadow',{value:9,enumerable:false,configurable:true});
        Object.setPrototypeOf(o,old);var seen='';
        for(var k in o){seen+=k+',';if(k==='z'){
            delete o.a;Object.defineProperty(o,'m',{enumerable:false});delete o.b;o.b=33;
            o.added=8;Object.setPrototypeOf(o,next);for(var i=0;i<12;i++)new Uint8Array(0);
        }}
        if(seen!=='z,b,y,c,x,d,n,'||o.shadow!==9||o.b!==33)return false;
        var buffer=new ArrayBuffer(2,{maxByteLength:4}),t=new Uint8Array(buffer);
        t.z=0;t.a=1;t.m=2;t.b=3;t.y=4;t.c=5;t.x=6;t.d=7;
        var keys='';for(var q in t){keys+=q+',';if(q==='0')buffer.resize(1);}
        return keys==='0,z,a,m,b,y,c,x,d,'&&t.length===1;
    })()"#;
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let out = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        assert_eq!(out.unwrap(), Value::Bool(true), "strict={strict}");
        clean(&runtime);
    }
}
