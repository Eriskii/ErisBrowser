use super::*;

fn fresh() -> (Runtime, Document) {
    (Runtime::try_new().unwrap(), Document::parse("<p>kept</p>"))
}

fn names(values: &[&str]) -> Vec<JsString> {
    values.iter().map(|value| (*value).into()).collect()
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

fn eight(runtime: &mut Runtime) -> Value {
    runtime
        .object_ordered(
            ["z", "a", "m", "b", "y", "c", "x", "d"]
                .into_iter()
                .map(|name| (name.into(), Value::Bool(true))),
        )
        .unwrap()
}

#[test]
fn ordinary_snapshot_preserves_literal_creation_order_symbols_and_utf16_handles() {
    let (mut runtime, mut doc) = fresh();
    let value = runtime
        .execute(
            r#"var reads=0,a={};
        Object.defineProperty(a,'z',{get:function(){reads++;throw 1;}});
        a[Symbol('one')]=1;a['']=2;a['\uD800x']=3;a.a=4;a['é']=5;
        a[Symbol('two')]=6;a['\0x']=7;a['+1']=8;a['-0']=9;a;"#,
            &mut doc,
        )
        .unwrap();
    // Fixed before invoking own_keys; neither lexical sorting nor descriptor
    // enumerability may alter this complete string-key snapshot.
    let expected: Vec<JsString> = vec![
        "z".into(),
        "".into(),
        vec![0xd800, 120].into(),
        "a".into(),
        vec![0xe9].into(),
        vec![0, 120].into(),
        "+1".into(),
        "-0".into(),
    ];
    let id = runtime.property_object(&value).unwrap();
    assert_eq!(runtime.objects[id].order.len(), 10);
    let pointers: Vec<_> = runtime.objects[id]
        .order
        .iter()
        .filter_map(|key| key.as_string().map(|text| text.units().as_ptr()))
        .collect();
    let before = (runtime.steps, runtime.allocated);
    let result = runtime.own_keys(&value).unwrap();
    assert_eq!(result, expected);
    assert_eq!(
        (before.0 - runtime.steps, runtime.allocated - before.1),
        (132, 8 * std::mem::size_of::<JsString>())
    );
    assert_eq!(
        result
            .iter()
            .map(|key| key.units().as_ptr())
            .collect::<Vec<_>>(),
        pointers
    );
    assert_eq!(runtime.lookup(1, "reads").unwrap().1, Value::Number(0.0));
    clean(&runtime);
}

#[test]
fn ordinary_snapshot_long_names_read_only_first_unit_and_share_payloads() {
    let mut costs = Vec::new();
    for length in [3, 512, 4096] {
        let (mut runtime, _) = fresh();
        let keys: Vec<JsString> = (0..8)
            .map(|index| {
                let mut units = vec![0xd800; length];
                units[length - 1] = 48 + index;
                units.into()
            })
            .collect();
        let value = runtime
            .object_ordered(keys.iter().cloned().map(|key| (key, Value::Bool(true))))
            .unwrap();
        let before = (runtime.steps, runtime.allocated);
        let result = runtime.own_keys(&value).unwrap();
        costs.push((before.0 - runtime.steps, runtime.allocated - before.1));
        assert_eq!(result, keys);
        for (actual, original) in result.iter().zip(&keys) {
            assert_eq!(actual.units().as_ptr(), original.units().as_ptr());
        }
    }
    assert_eq!(costs, vec![(112, 8 * std::mem::size_of::<JsString>()); 3]);
}

#[test]
fn ordinary_snapshot_small_orders_keep_generic_builder_and_symbols_stay_absent() {
    for (source, expected, work, heap) in [
        ("({})", Vec::new(), 5, 0),
        (
            "({z:1,a:2})",
            names(&["z", "a"]),
            38,
            2 * (std::mem::size_of::<OwnKey>() + std::mem::size_of::<JsString>()),
        ),
        (
            "var a={};for(var i=0;i<8;i++)a[Symbol('s')]=i;a;",
            Vec::new(),
            96,
            0,
        ),
    ] {
        let (mut runtime, mut doc) = fresh();
        let value = runtime.execute(source, &mut doc).unwrap();
        let before = (runtime.steps, runtime.allocated);
        assert_eq!(runtime.own_keys(&value).unwrap(), expected);
        assert_eq!(
            (before.0 - runtime.steps, runtime.allocated - before.1),
            (work, heap)
        );
        clean(&runtime);
    }
}

#[test]
fn ordinary_snapshot_digit_leading_names_use_original_numeric_ordering() {
    let (mut runtime, mut doc) = fresh();
    let value = runtime
        .execute(
            "({tail:0,'01':1,'4294967295':2,'10':3,'2':4,'0':5,alpha:6,beta:7})",
            &mut doc,
        )
        .unwrap();
    let expected = names(&["0", "2", "10", "tail", "01", "4294967295", "alpha", "beta"]);
    let id = runtime.property_object(&value).unwrap();
    let before = (runtime.steps, runtime.allocated);
    assert!(runtime.own_key_stored_strings(id).unwrap().is_none());
    assert_eq!(
        (before.0 - runtime.steps, runtime.allocated - before.1),
        (16, 0)
    );
    assert_eq!(runtime.own_keys(&value).unwrap(), expected);
    // Both are ordinary property names, yet the conservative first-unit proof
    // must reject them before the unchanged full numeric classifier decides.
    for leading in ["01", "4294967295"] {
        let value = runtime
            .object_ordered(
                std::iter::once((leading.into(), Value::Bool(true))).chain(
                    ["z", "a", "b", "c", "d", "e", "f"]
                        .into_iter()
                        .map(|name| (name.into(), Value::Bool(true))),
                ),
            )
            .unwrap();
        let id = runtime.property_object(&value).unwrap();
        let before = (runtime.steps, runtime.allocated);
        assert!(runtime.own_key_stored_strings(id).unwrap().is_none());
        assert_eq!(
            (before.0 - runtime.steps, runtime.allocated - before.1),
            (10, 0)
        );
        assert_eq!(
            runtime.own_keys(&value).unwrap()[0],
            JsString::from(leading)
        );
    }
    clean(&runtime);
}

#[test]
fn ordinary_snapshot_does_not_take_virtual_array_or_boxed_string_keys() {
    for (source, expected) in [
        (
            "var a=[0,1,2];delete a[1];a;",
            names(&["0", "2", "length", "z", "a", "m", "b", "y", "c", "x", "d"]),
        ),
        (
            "var a=Object('\\uD800x');a;",
            names(&["0", "1", "length", "z", "a", "m", "b", "y", "c", "x", "d"]),
        ),
    ] {
        let (mut runtime, mut doc) = fresh();
        runtime.execute(source, &mut doc).unwrap();
        let value = runtime
            .execute(
                "a.z=0;a.a=1;a.m=2;a.b=3;a.y=4;a.c=5;a.x=6;a.d=7;a;",
                &mut doc,
            )
            .unwrap();
        let id = runtime.property_object(&value).unwrap();
        assert!(runtime.objects[id].order.len() >= 8);
        assert_eq!(runtime.own_keys(&value).unwrap(), expected);
        clean(&runtime);
    }
}

#[test]
fn ordinary_snapshot_reproves_after_mutation_and_for_in_keeps_live_descriptors() {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = r#"(function(){
            var a={z:0,a:1,m:2,b:3,y:4,c:5,x:6,d:7};
            var first=Object.getOwnPropertyNames(a).join(',');a['0']=8;
            var second=Object.getOwnPropertyNames(a).join(',');delete a['0'];delete a.a;a.a=9;
            var third=Object.getOwnPropertyNames(a).join(',');
            if(first!=='z,a,m,b,y,c,x,d'||second!=='0,z,a,m,b,y,c,x,d'||third!=='z,m,b,y,c,x,d,a')return false;
            var p={hidden:1,later:2},o={first:1,a:2,b:3,c:4,d:5,e:6,f:7,g:8};
            Object.setPrototypeOf(o,p);Object.defineProperty(o,'hidden',{value:0});
            var seen='';for(var k in o){seen+=k+',';if(k==='first'){
                delete o.b;Object.defineProperty(o,'c',{enumerable:false});p.later=3;
            }}
            return seen==='first,a,d,e,f,g,later,'&&o.hidden===0;
        })()"#;
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
fn ordinary_snapshot_exact_and_one_short_admission_preserves_all_input_state() {
    let expected = names(&["z", "a", "m", "b", "y", "c", "x", "d"]);
    let heap = 8 * std::mem::size_of::<JsString>();
    // Literal schedule: route4 + setup4 + 8*probe6 + (8+8*4+8*2).
    for steps in [0, 3, 4, 7, 8, 13, 55, 56, 111, 112] {
        let (mut runtime, _) = fresh();
        let value = eight(&mut runtime);
        let id = runtime.property_object(&value).unwrap();
        let original = format!("{:?}", runtime.objects[id].values);
        let order = runtime.objects[id].order.clone();
        let allocated = runtime.allocated;
        runtime.steps = steps;
        let result = runtime.own_keys(&value);
        if steps == 112 {
            assert_eq!(result.unwrap(), expected);
            assert_eq!((runtime.steps, runtime.allocated - allocated), (0, heap));
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            assert_eq!(runtime.allocated, allocated);
        }
        assert_eq!(runtime.objects[id].order, order);
        assert_eq!(format!("{:?}", runtime.objects[id].values), original);
        clean(&runtime);
    }
    for room in [0, heap - 1, heap] {
        let (mut runtime, _) = fresh();
        let value = eight(&mut runtime);
        let id = runtime.property_object(&value).unwrap();
        let order = runtime.objects[id].order.clone();
        runtime.steps = 112;
        runtime.allocated = MAX_HEAP - room;
        let result = runtime.own_keys(&value);
        if room == heap {
            assert_eq!(result.unwrap(), expected);
            assert_eq!(runtime.allocated, MAX_HEAP);
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.objects[id].order, order);
        clean(&runtime);
    }
}
