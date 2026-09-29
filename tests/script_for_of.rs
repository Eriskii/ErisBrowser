use eris::dom::Document;
use eris::script::{Runtime, Value};

#[test]
fn for_of_completion_values_survive_empty_bodies_labels_and_finally() {
    let cases = [
        ("7;for(var x of []){9;}", Value::Undefined),
        ("7;for(var x of [2,4]){}", Value::Undefined),
        ("for(var x of [2,4]){x;}", Value::Number(4.0)),
        ("for(var x of [2,4]){x;break;}", Value::Number(2.0)),
        ("for(var x of [2,4]){x;continue;}", Value::Number(4.0)),
        ("for(var x of [2,4]){x;var unused;}", Value::Number(4.0)),
        ("for(var x of [2,4]){if(x===2){9;}}", Value::Undefined),
        (
            "for(var x of [2,4]){try{x;break;}finally{99;}}",
            Value::Number(2.0),
        ),
        (
            "outer:{for(var x of [2,4]){x;break outer;}}",
            Value::Number(2.0),
        ),
        (
            "outer:for(var x of [2,4]){for(var y of [6,8]){x+y;continue outer;}}",
            Value::Number(10.0),
        ),
        ("for(let x of [null,false]){x;}", Value::Bool(false)),
        ("for(const x of [2,null]){x;}", Value::Null),
    ];
    for strict in [false, true] {
        for (source, expected) in &cases {
            let mut runtime = Runtime::new();
            let mut document = Document::parse("");
            let actual = if strict {
                runtime.execute_strict(source, &mut document)
            } else {
                runtime.execute(source, &mut document)
            }
            .unwrap_or_else(|error| panic!("strict={strict}: {source}: {error}"));
            assert_eq!(&actual, expected, "strict={strict}: {source}");
        }
    }
}

#[test]
fn iterator_close_preserves_completion_identity_and_runs_once() {
    for strict in [false, true] {
        for initializer in ["{}", "[]", "function(){}", "new Number(7)"] {
            for body in [
                "for(var x of source){token;break;}",
                "outer:{for(let x of source){token;break outer;}}",
                "for(const x of source){try{token;break;}finally{99;}}",
            ] {
                let source = format!(
                    "var token={initializer},closed=0,source={{}};\
                     source[Symbol.iterator]=function(){{return {{\
                     next:function(){{return {{done:false,value:2}};}},\
                     return:function(){{closed++;return {{value:99,done:true}};}}\
                     }};}};{body}"
                );
                let mut runtime = Runtime::new();
                let mut document = Document::parse("");
                let actual = if strict {
                    runtime.execute_strict(&source, &mut document)
                } else {
                    runtime.execute(&source, &mut document)
                }
                .unwrap_or_else(|error| panic!("strict={strict}: {source}: {error}"));
                assert_eq!(
                    actual,
                    runtime.execute("token", &mut document).unwrap(),
                    "strict={strict}: {source}"
                );
                assert_eq!(
                    runtime.execute("closed", &mut document).unwrap(),
                    Value::Number(1.0),
                    "strict={strict}: {source}"
                );
            }
        }
    }
}
