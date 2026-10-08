//! Source-only handoff: register as typed_array::set_key::tests.
use super::*;

fn fresh() -> (Runtime, Document, Value) {
    let mut runtime = Runtime::try_new().unwrap();
    let mut doc = Document::parse("<p>kept</p>");
    runtime
        .execute("var tokenTarget=new Uint8Array([7,9,11,13]);", &mut doc)
        .unwrap();
    let target = runtime.environments[0].bindings["tokenTarget"]
        .value
        .clone();
    reset(&mut runtime);
    (runtime, doc, target)
}

fn reset(runtime: &mut Runtime) {
    runtime.steps = MAX_STEPS;
    runtime.allocated = 0;
}
fn spent(runtime: &Runtime) -> (usize, usize) {
    (MAX_STEPS - runtime.steps, runtime.allocated)
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
fn authored(source: &str) {
    for strict in [false, true] {
        let (mut runtime, mut doc, _) = fresh();
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
fn classified_key_67_literals_preserve_immutable_identity() {
    for (name, units, expected, _) in LITERALS {
        let (mut runtime, _, target) = fresh();
        let text = JsString::from(*units);
        let property = PropertyKey::String(text.clone());
        let mut key = SetKey::from_property(&property).unwrap();
        assert!(std::ptr::eq(key.text().units(), text.units()));
        assert!(std::ptr::eq(key.property(), &property));
        assert!(key.index.is_none());
        runtime
            .typed_array_own_property_for_set(&target, &mut key, false)
            .unwrap();
        match (key.index.unwrap(), expected) {
            (Index::Ordinary, None) => {}
            (Index::Numeric(value), Some(bits)) => assert_eq!(value.to_bits(), *bits, "{name}"),
            pair => panic!("{name}: {pair:?}"),
        }
        let before = key.index.unwrap();
        runtime
            .typed_array_own_property_for_set(&target, &mut key, false)
            .unwrap();
        match (before, key.index.unwrap()) {
            (Index::Ordinary, Index::Ordinary) => {}
            (Index::Numeric(a), Index::Numeric(b)) => assert_eq!(a.to_bits(), b.to_bits()),
            _ => panic!("classification changed"),
        }
        assert_eq!(key.text().units(), *units);
        assert!(SetKey::new(&text).index.is_none());
        let equal = JsString::from(*units);
        assert_eq!(equal, text);
        assert!(SetKey::new(&equal).index.is_none());
    }
}

#[test]
fn classified_key_first_reuse_and_no_record_fee_contracts() {
    for text in ["1.0", "entries", "abcdefghijklmnopqrstuvwxyz"] {
        let (mut runtime, mut doc, target) = fresh();
        let plain = runtime.object_ordered([]).unwrap();
        let text = JsString::from(text);
        reset(&mut runtime);
        runtime.typed_array_record(&target).unwrap().unwrap();
        let record_work = spent(&runtime).0;
        reset(&mut runtime);
        assert!(matches!(
            runtime.typed_array_index(&text).unwrap(),
            Index::Ordinary
        ));
        let (class_work, class_heap) = spent(&runtime);
        let mut key = SetKey::new(&text);
        reset(&mut runtime);
        assert!(matches!(
            runtime
                .typed_array_set_for_set(&target, &mut key, &target, Value::Number(1.0), &mut doc)
                .unwrap(),
            Exotic::Ordinary
        ));
        assert_eq!(spent(&runtime), (record_work + class_work + 8, class_heap));
        reset(&mut runtime);
        assert!(matches!(
            runtime
                .typed_array_own_property_for_set(&target, &mut key, false)
                .unwrap(),
            Exotic::Ordinary
        ));
        assert_eq!(spent(&runtime), (record_work + 4, 0));
        reset(&mut runtime);
        runtime.typed_array_record(&plain).unwrap();
        let no_record = spent(&runtime);
        reset(&mut runtime);
        assert!(matches!(
            runtime
                .typed_array_own_property_for_set(&plain, &mut key, false)
                .unwrap(),
            Exotic::Ordinary
        ));
        assert_eq!(spent(&runtime), no_record);
        let mut again = SetKey::new(&text);
        reset(&mut runtime);
        runtime
            .typed_array_own_property_for_set(&target, &mut again, false)
            .unwrap();
        assert_eq!(spent(&runtime), (record_work + class_work + 8, class_heap));
        // The standalone own path still classifies and does not manufacture a token.
        reset(&mut runtime);
        assert!(matches!(
            runtime
                .typed_array_own_property(&target, &text, false)
                .unwrap(),
            Exotic::Ordinary
        ));
        assert_eq!(spent(&runtime), (record_work + class_work, class_heap));
    }
}

#[test]
fn classified_key_completed_numeric_set_has_no_new_debit() {
    for (text, distinct) in [("0", false), ("-0", false), ("NaN", false), ("99", true)] {
        let (mut runtime, mut doc, target) = fresh();
        let receiver = if distinct {
            runtime.object_ordered([]).unwrap()
        } else {
            target.clone()
        };
        let text = JsString::from(text);
        reset(&mut runtime);
        let record = runtime.typed_array_record(&target).unwrap().unwrap();
        let Index::Numeric(number) = runtime.typed_array_index(&text).unwrap() else {
            panic!("numeric")
        };
        assert!(matches!(
            runtime
                .typed_array_set_index(
                    &target,
                    &receiver,
                    Value::Number(7.0),
                    record,
                    number,
                    &mut doc
                )
                .unwrap(),
            Exotic::Handled(true)
        ));
        let original = spent(&runtime);
        let mut key = SetKey::new(&text);
        reset(&mut runtime);
        assert!(matches!(
            runtime
                .typed_array_set_for_set(&target, &mut key, &receiver, Value::Number(7.0), &mut doc)
                .unwrap(),
            Exotic::Handled(true)
        ));
        assert_eq!(spent(&runtime), original);
        assert!(key.index.is_none());
    }
    println!(
        "SET_KEY_STORAGE bytes={}",
        std::mem::size_of::<SetKey<'_>>()
    );
}

#[test]
fn classified_key_creation_reuse_and_conversion_refusals_preserve_effects() {
    let (mut runtime, mut doc, target) = fresh();
    let text = JsString::from("1.0");
    reset(&mut runtime);
    runtime.typed_array_record(&target).unwrap().unwrap();
    let record_work = spent(&runtime).0;
    runtime.typed_array_index(&text).unwrap();
    let prefix = spent(&runtime).0;
    let mut key = SetKey::new(&text);
    reset(&mut runtime);
    runtime.steps = prefix + 7;
    let error = runtime
        .typed_array_set_for_set(&target, &mut key, &target, Value::Number(37.0), &mut doc)
        .err()
        .expect("creation refusal");
    assert_eq!(error.kind, ErrorKind::Resource);
    assert!(key.index.is_none());
    assert!(runtime.own_property(&target, &text).is_none());
    reset(&mut runtime);
    runtime.steps = prefix + 8;
    runtime
        .typed_array_set_for_set(&target, &mut key, &target, Value::Number(37.0), &mut doc)
        .unwrap();
    assert_eq!(runtime.steps, 0);
    assert!(matches!(key.index, Some(Index::Ordinary)));
    reset(&mut runtime);
    runtime.steps = record_work + 3;
    assert_eq!(
        runtime
            .typed_array_own_property_for_set(&target, &mut key, false)
            .err()
            .expect("resource refusal")
            .kind,
        ErrorKind::Resource
    );
    assert!(runtime.own_property(&target, &text).is_none());

    // Measure only the unchanged predecessor stages through conversion, then
    // refuse the next live-index admission. The authored effect is retained.
    fn conversion_setup() -> (Runtime, Document, Value, Value) {
        let (mut runtime, mut doc, target) = fresh();
        let value = runtime
            .execute(
                "var tokenEffect=0;({valueOf:function(){tokenEffect++;return 41;}})",
                &mut doc,
            )
            .unwrap();
        reset(&mut runtime);
        (runtime, doc, target, value)
    }
    let (mut reference, mut doc, target, value) = conversion_setup();
    reference.typed_array_record(&target).unwrap().unwrap();
    reference.typed_array_index(&JsString::from("0")).unwrap();
    assert_eq!(reference.splice_number(value, &mut doc).unwrap(), 41.0);
    let through_conversion = spent(&reference).0;
    let (mut runtime, mut doc, target, value) = conversion_setup();
    runtime.steps = through_conversion;
    let numeric = JsString::from("0");
    let mut key = SetKey::new(&numeric);
    assert_eq!(
        runtime
            .typed_array_set_for_set(&target, &mut key, &target, value, &mut doc)
            .err()
            .expect("resource refusal")
            .kind,
        ErrorKind::Resource
    );
    assert_eq!(
        runtime.environments[0].bindings["tokenEffect"].value,
        Value::Number(1.0)
    );
    assert!(key.index.is_none());
    runtime.steps = MAX_STEPS;
    assert_eq!(
        runtime.get_key(target, &numeric, &mut doc).unwrap(),
        Value::Number(7.0)
    );
    clean(&runtime);
}

#[test]
fn classified_key_reflect_expandos_and_key_reentrancy() {
    for index in [0, 1, 2] {
        authored(AUTHORED[index].1);
    }
}
#[test]
fn classified_key_distinct_receiver_refusals_and_definition() {
    for index in [3, 4, 7] {
        authored(AUTHORED[index].1);
    }
}
#[test]
fn classified_key_fresh_resize_detach_and_record_reallocation() {
    for index in [5, 6] {
        authored(AUTHORED[index].1);
    }
}
#[test]
fn classified_key_assignment_and_prototype_mutation() {
    for index in [8, 9] {
        authored(AUTHORED[index].1);
    }
    for strict in [false, true] {
        let (mut runtime, mut doc, target) = fresh();
        let key = JsString::from("1.0");
        assert!(
            runtime
                .define_property_key(
                    &target,
                    &PropertyKey::String(key.clone()),
                    PropertyDescriptor::data_property(Value::Number(23.0), false, true, true),
                    &mut doc
                )
                .unwrap()
        );
        let result =
            runtime.set_key_strict(target.clone(), &key, Value::Number(37.0), strict, &mut doc);
        if strict {
            assert_eq!(
                result.expect_err("strict assignment refusal").kind,
                ErrorKind::Runtime("TypeError")
            );
        } else {
            result.unwrap();
        }
        assert_eq!(
            runtime.get_key(target, &key, &mut doc).unwrap(),
            Value::Number(23.0)
        );
    }
}
#[test]
fn classified_key_splice_ingress_and_fresh_numeric_state() {
    for index in [10, 11] {
        authored(AUTHORED[index].1);
    }
}
#[test]
fn classified_key_symbol_and_plain_object_boundaries() {
    authored(AUTHORED[12].1);
}

// Frozen classifier literal table copied byte-for-byte; no result-derived oracle.
const LITERALS: &[(&str, &[u16], Option<u64>, bool)] = &[
    ("numeric_0", &[48], Some(0x0000000000000000), true),
    ("numeric_1", &[49], Some(0x3ff0000000000000), true),
    ("numeric_2", &[51], Some(0x4008000000000000), true),
    ("numeric_3", &[52], Some(0x4010000000000000), false),
    ("numeric_4", &[45, 48], Some(0x8000000000000000), false),
    ("numeric_5", &[45, 49], Some(0xbff0000000000000), false),
    ("numeric_6", &[48, 46, 53], Some(0x3fe0000000000000), false),
    (
        "numeric_7",
        &[49, 46, 50, 53],
        Some(0x3ff4000000000000),
        false,
    ),
    ("numeric_8", &[78, 97, 78], Some(0x7ff8000000000000), false),
    (
        "numeric_9",
        &[73, 110, 102, 105, 110, 105, 116, 121],
        Some(0x7ff0000000000000),
        false,
    ),
    (
        "numeric_10",
        &[45, 73, 110, 102, 105, 110, 105, 116, 121],
        Some(0xfff0000000000000),
        false,
    ),
    (
        "numeric_11",
        &[52, 50, 57, 52, 57, 54, 55, 50, 57, 53],
        Some(0x41efffffffe00000),
        false,
    ),
    (
        "numeric_12",
        &[52, 50, 57, 52, 57, 54, 55, 50, 57, 54],
        Some(0x41f0000000000000),
        false,
    ),
    (
        "numeric_13",
        &[
            57, 48, 48, 55, 49, 57, 57, 50, 53, 52, 55, 52, 48, 57, 57, 49,
        ],
        Some(0x433fffffffffffff),
        false,
    ),
    (
        "numeric_14",
        &[
            57, 48, 48, 55, 49, 57, 57, 50, 53, 52, 55, 52, 48, 57, 57, 50,
        ],
        Some(0x4340000000000000),
        false,
    ),
    (
        "numeric_15",
        &[
            57, 48, 48, 55, 49, 57, 57, 50, 53, 52, 55, 52, 48, 57, 57, 52,
        ],
        Some(0x4340000000000001),
        false,
    ),
    (
        "numeric_16",
        &[
            49, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 49, 48, 48,
        ],
        Some(0x43abc16d674ec801),
        false,
    ),
    (
        "numeric_17",
        &[
            49, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 46, 50,
        ],
        Some(0x430c6bf526340002),
        false,
    ),
    (
        "numeric_18",
        &[
            45, 49, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 46, 50,
        ],
        Some(0xc30c6bf526340002),
        false,
    ),
    (
        "numeric_19",
        &[49, 101, 45, 55],
        Some(0x3e7ad7f29abcaf48),
        false,
    ),
    (
        "numeric_20",
        &[48, 46, 48, 48, 48, 48, 48, 49],
        Some(0x3eb0c6f7a0b5ed8d),
        false,
    ),
    (
        "numeric_21",
        &[
            45, 48, 46, 48, 48, 48, 48, 48, 49, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 50,
        ],
        Some(0xbeb0c6f7a0b5ed8e),
        false,
    ),
    (
        "numeric_22",
        &[49, 101, 43, 50, 49],
        Some(0x444b1ae4d6e2ef50),
        false,
    ),
    (
        "numeric_23",
        &[53, 101, 45, 51, 50, 52],
        Some(0x0000000000000001),
        false,
    ),
    (
        "numeric_24",
        &[
            49, 46, 55, 57, 55, 54, 57, 51, 49, 51, 52, 56, 54, 50, 51, 49, 53, 55, 101, 43, 51,
            48, 56,
        ],
        Some(0x7fefffffffffffff),
        false,
    ),
    ("ordinary_25", &[], None, false),
    ("ordinary_26", &[43, 48], None, false),
    ("ordinary_27", &[48, 48], None, false),
    ("ordinary_28", &[48, 49], None, false),
    ("ordinary_29", &[45, 48, 48], None, false),
    ("ordinary_30", &[43, 49], None, false),
    ("ordinary_31", &[49, 46, 48], None, false),
    ("ordinary_32", &[49, 46], None, false),
    ("ordinary_33", &[49, 101, 48], None, false),
    ("ordinary_34", &[49, 69, 43, 50, 49], None, false),
    ("ordinary_35", &[49, 101, 50, 49], None, false),
    ("ordinary_36", &[49, 101, 43, 48, 50, 49], None, false),
    ("ordinary_37", &[49, 101, 45, 54], None, false),
    (
        "ordinary_38",
        &[48, 46, 48, 48, 48, 48, 48, 48, 49],
        None,
        false,
    ),
    (
        "ordinary_39",
        &[
            57, 48, 48, 55, 49, 57, 57, 50, 53, 52, 55, 52, 48, 57, 57, 51,
        ],
        None,
        false,
    ),
    (
        "ordinary_40",
        &[
            49, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 49, 50, 56,
        ],
        None,
        false,
    ),
    (
        "ordinary_41",
        &[
            49, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 46, 51,
        ],
        None,
        false,
    ),
    (
        "ordinary_42",
        &[
            45, 49, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 46, 51,
        ],
        None,
        false,
    ),
    (
        "ordinary_43",
        &[43, 73, 110, 102, 105, 110, 105, 116, 121],
        None,
        false,
    ),
    ("ordinary_44", &[110, 97, 110], None, false),
    ("ordinary_45", &[43, 78, 97, 78], None, false),
    ("ordinary_46", &[45, 78, 97, 78], None, false),
    ("ordinary_47", &[32, 49], None, false),
    ("ordinary_48", &[49, 32], None, false),
    ("ordinary_49", &[9, 49], None, false),
    ("ordinary_50", &[160, 49], None, false),
    ("ordinary_51", &[48, 120, 49], None, false),
    ("ordinary_52", &[48, 88, 49], None, false),
    ("ordinary_53", &[48, 98, 49], None, false),
    ("ordinary_54", &[48, 111, 49], None, false),
    ("ordinary_55", &[46, 53], None, false),
    ("ordinary_56", &[49, 95, 48], None, false),
    ("ordinary_57", &[49, 101, 51, 48, 57], None, false),
    ("ordinary_58", &[49, 101, 45, 52, 48, 48], None, false),
    ("ordinary_59", &[116, 114, 117, 101], None, false),
    (
        "ordinary_60",
        &[117, 110, 100, 101, 102, 105, 110, 101, 100],
        None,
        false,
    ),
    (
        "ordinary_61",
        &[
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48,
        ],
        None,
        false,
    ),
    (
        "ordinary_62",
        &[
            45, 48, 46, 48, 48, 48, 48, 48, 49, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 50, 48,
        ],
        None,
        false,
    ),
    ("ordinary_63", &[55296], None, false),
    ("ordinary_64", &[49, 57343], None, false),
    ("ordinary_65", &[65297], None, false),
    ("ordinary_66", &[49, 0], None, false),
];

const AUTHORED: &[(&str, &str)] = &[
    (
        "reflect_noncanonical_create_update_readonly",
        r####"(function(){

var t=new Uint8Array([9]), calls=0;
if(!Reflect.set(t,'1.0',17)||!Reflect.set(t,'1.0',23))return false;
Object.defineProperty(t,'1.0',{writable:false});
var value={valueOf:function(){calls++;return 41;}};
return Reflect.set(t,'1.0',value)===false&&calls===0&&t['1.0']===23&&t[0]===9&&Object.keys(t).join(',')==='0,1.0';
})()"####,
    ),
    (
        "key_conversion_and_reentrant_separate_operation",
        r####"(function(){

var t=new Uint8Array([9]), calls=0, key={};
key[Symbol.toPrimitive]=function(hint){if(hint!=='string')throw new Error('hint');calls++;Reflect.set(t,'+1',11);return '1.0';};
if(!Reflect.set(t,key,17)||!Reflect.set(t,key,19))return false;
var same=('1'+'.0');if(!Reflect.set(t,same,23))return false;
return calls===2&&t['+1']===11&&t['1.0']===23&&t[0]===9;
})()"####,
    ),
    (
        "exact_utf16_and_numeric_special_keys",
        r####"(function(){

var t=new Uint8Array([9]), calls=0, v={valueOf:function(){calls++;return 7;}};
t['\ud800']=11;t['\ufffd']=13;
if(!Reflect.set(t,'-0',v)||!Reflect.set(t,'NaN',v)||!Reflect.set(t,'Infinity',v))return false;
return calls===3&&t[0]===9&&!Object.hasOwn(t,'-0')&&!Object.hasOwn(t,'NaN')&&!Object.hasOwn(t,'Infinity')&&t['\ud800']===11&&t['\ufffd']===13;
})()"####,
    ),
    (
        "distinct_target_invalid_skips_receiver_and_value",
        r####"(function(){

var target=new Uint8Array([9]), receiver=new Uint8Array([3,4]), calls=0;
var v={valueOf:function(){calls++;throw new Error('value conversion');}};
var ok=Reflect.set(target,'1',v,receiver);
return ok===true&&calls===0&&target[0]===9&&receiver[0]===3&&receiver[1]===4;
})()"####,
    ),
    (
        "receiver_accessor_or_initial_invalid_rejects_without_hooks",
        r####"(function(){

var target=new Uint8Array([9]), gets=0,sets=0,converts=0,receiver={};
Object.defineProperty(receiver,'0',{get:function(){gets++;return 3;},set:function(){sets++;},configurable:true});
var v={valueOf:function(){converts++;return 7;}};
if(Reflect.set(target,'0',v,receiver)!==false)return false;
if(Reflect.set(target,'0',v,new Uint8Array(0))!==false)return false;
return gets===0&&sets===0&&converts===0&&target[0]===9;
})()"####,
    ),
    (
        "fresh_receiver_shrink_then_grow",
        r####"(function(){

var target=new Uint8Array([9]),b=new ArrayBuffer(1,{maxByteLength:4}),r=new Uint8Array(b),calls=0;
r[0]=3;var v={valueOf:function(){calls++;b.resize(0);return 7;}};
if(Reflect.set(target,'0',v,r)!==true||calls!==1||r.length!==0)return false;
b.resize(1);v={valueOf:function(){calls++;b.resize(4);for(var i=0;i<12;i++)new Uint16Array(0);return 11;}};
return Reflect.set(target,'0',v,r)===true&&calls===2&&r.length===4&&r[0]===11&&target[0]===9;
})()"####,
    ),
    (
        "fresh_fixed_receiver_detach_retains_effect",
        r####"(function(){

var target=new Uint8Array([9]),b=new ArrayBuffer(1,{maxByteLength:2}),r=new Uint8Array(b,0,1),calls=0,copy;
r[0]=3;var v={valueOf:function(){calls++;copy=b.transfer();return 7;}};
return Reflect.set(target,'0',v,r)===true&&calls===1&&b.detached&&r.length===0&&new Uint8Array(copy)[0]===3&&target[0]===9;
})()"####,
    ),
    (
        "ordinary_target_typed_receiver_uses_receiver_classification",
        r####"(function(){

var target={'0':5},r=new Uint16Array([3]),calls=0;
var v={valueOf:function(){calls++;return 513;}};
return Reflect.set(target,'0',v,r)===true&&calls===1&&r[0]===513&&target[0]===5;
})()"####,
    ),
    (
        "assignment_typed_prototype_terminal_numeric",
        r####"(function(){

var p=new Uint8Array([9]),o=Object.create(p),calls=0,v={valueOf:function(){calls++;return 7;}};
o[0]=7;o[1]=v;o['1.0']=17;
return Object.hasOwn(o,'0')&&o[0]===7&&!Object.hasOwn(o,'1')&&calls===0&&p[0]===9&&o['1.0']===17;
})()"####,
    ),
    (
        "prototype_changes_and_nested_set_do_not_reuse_state",
        r####"(function(){

var p=new Uint8Array([9]),o={},replacement={},trace='',calls=0,key={};
Object.defineProperty(replacement,'1.0',{value:99,writable:false});
Object.defineProperty(p,'1.0',{set:function(v){trace+='S';Object.setPrototypeOf(this,replacement);Reflect.set(this,'+1',11);trace+='N';},configurable:true});
key[Symbol.toPrimitive]=function(){trace+='K';calls++;if(calls===1)Object.setPrototypeOf(o,p);return '1.0';};
var first=Reflect.set(o,key,17),second=Reflect.set(o,key,19);
return first===true&&second===false&&trace==='KSNK'&&calls===2&&o['+1']===11&&o['1.0']===99&&!Object.hasOwn(o,'1.0')&&p[0]===9;
})()"####,
    ),
    (
        "array_splice_typed_prototype_distinct_receiver",
        r####"(function(){
if(typeof Array.prototype.splice!=='function')throw new Error('splice prerequisite');
var control=[1], removed=Array.prototype.splice.call(control,0,0,3);
if(removed.length!==0||control.length!==2||control[0]!==3||control[1]!==1)return false;
var p=new Uint8Array([9]),o=Object.create(p);Object.defineProperty(o,'length',{value:1,writable:true});
var result=Array.prototype.splice.call(o,0,0,7);
return result.length===0&&o.length===2&&Object.hasOwn(o,'0')&&o[0]===7&&!Object.hasOwn(o,'1')&&p[0]===9;
})()"####,
    ),
    (
        "array_splice_detach_still_converts_each_same_receiver_set",
        r####"(function(){
if(typeof Array.prototype.splice!=='function')throw new Error('splice prerequisite');
var control=[1,2], removed=Array.prototype.splice.call(control,0,2,3,4);
if(removed.length!==2||removed[0]!==1||removed[1]!==2||control[0]!==3||control[1]!==4)return false;
var b=new ArrayBuffer(2),t=new Uint8Array(b),calls=0,copy;t[0]=1;t[1]=2;
var value={valueOf:function(){calls++;if(calls===1)copy=b.transfer();return 9;}};
var threw=false;try{Array.prototype.splice.call(t,0,2,value,value);}catch(e){threw=e instanceof TypeError;}
var bytes=new Uint8Array(copy);
return threw&&calls===2&&b.detached&&t.length===0&&bytes[0]===1&&bytes[1]===2;
})()"####,
    ),
    (
        "symbol_and_plain_object_boundaries",
        r####"(function(){

var t=new Uint8Array([9]),s=Symbol('0'),plain={},calls=0,v={valueOf:function(){calls++;return 7;}};
if(!Reflect.set(t,s,v)||!Reflect.set(plain,'1.0',v))return false;
return t[s]===v&&plain['1.0']===v&&calls===0&&t[0]===9&&Object.getOwnPropertySymbols(t)[0]===s;
})()"####,
    ),
];
