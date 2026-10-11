//! Existing-API semantic oracles, frozen before the diagnostic-path repair.
use super::*;

fn fresh() -> (Runtime, Document) {
    (
        Runtime::new(),
        Document::parse("<button id=b>button</button>"),
    )
}

fn idle(runtime: &Runtime) {
    assert_eq!(
        (runtime.calls, runtime.eval_depth, runtime.stack_units),
        (0, 0, 0)
    );
    assert!(runtime.frames.is_empty());
    assert!(!runtime.microtasks_suspended());
}

#[test]
fn throw_completion_constructor_accessor_preserves_operand_and_exact_value() {
    let (mut runtime, mut document) = fresh();
    let result = runtime.execute(r#"
        var operandReads=0,metadataReads=0,token={},replacement={};
        Object.defineProperty(token,'constructor',{get:function(){metadataReads++;throw replacement;}});
        var holder={get value(){operandReads++;return token;}},exact=false;
        try{throw holder.value;}catch(e){exact=e===token;}
        exact+'|'+operandReads+'|'+metadataReads;
    "#, &mut document).unwrap();
    assert_eq!(result, Value::String("true|1|0".into()));
    assert!(runtime.console.is_empty());
    idle(&runtime);
}

#[test]
fn throw_completion_name_accessor_and_setter_only_shadow_stop_diagnostics() {
    for setter_only in [false, true] {
        let (mut runtime, mut document) = fresh();
        let descriptor = if setter_only {
            "{set:function(v){metadataReads++;}}"
        } else {
            "{get:function(){metadataReads++;throw replacement;}}"
        };
        let source = format!(
            r#"
            var metadataReads=0,token={{}},replacement={{}};
            function C(){{}}
            Object.defineProperty(C,'name',{descriptor});
            token.constructor=C;
            var exact=false;
            try{{throw token;}}catch(e){{exact=e===token;}}
            exact+'|'+metadataReads;
        "#
        );
        assert_eq!(
            runtime.execute(&source, &mut document).unwrap(),
            Value::String("true|0".into())
        );
        let error = runtime.execute("throw token;", &mut document).unwrap_err();
        assert_eq!(error.name(), "ThrownValue");
        assert_eq!(error.intrinsic_error_name(), None);
        idle(&runtime);
    }
}

#[test]
fn throw_completion_top_level_poison_does_not_replace_throw_with_resource_failure() {
    let (mut runtime, mut document) = fresh();
    let token = runtime
        .execute(
            r#"
        var reads=0,token={};
        Object.defineProperty(token,'constructor',{get:function(){reads++;while(true){}}});
        token;
    "#,
            &mut document,
        )
        .unwrap();
    let error = runtime.execute("throw token;", &mut document).unwrap_err();
    assert!(!error.is_resource_limit());
    assert!(!error.is_unsupported());
    let ErrorKind::Thrown(value) = &error.kind else {
        panic!("original author throw")
    };
    assert_eq!(value, &token);
    assert_eq!(error.name(), "ThrownValue");
    assert_eq!(error.intrinsic_error_name(), None);
    assert_eq!(
        runtime.execute("reads", &mut document).unwrap(),
        Value::Number(0.0)
    );
    idle(&runtime);
}

#[test]
fn throw_completion_payload_matrix_preserves_nan_negative_zero_symbol_and_handles() {
    for expression in [
        "undefined",
        "null",
        "true",
        "NaN",
        "-0",
        "Symbol('token')",
        "'\\ud800'",
        "({})",
        "(function(){})",
    ] {
        let (mut runtime, mut document) = fresh();
        let expected = runtime
            .execute(&format!("var token={expression};token;"), &mut document)
            .unwrap();
        let error = runtime.execute("throw token;", &mut document).unwrap_err();
        let ErrorKind::Thrown(actual) = &error.kind else {
            panic!("author value")
        };
        match (&expected, actual) {
            (Value::Number(expected), Value::Number(actual)) if expected.is_nan() => {
                assert!(actual.is_nan())
            }
            (Value::Number(expected), Value::Number(actual)) => {
                assert_eq!(expected.to_bits(), actual.to_bits())
            }
            _ => assert_eq!(actual, &expected),
        }
        assert!(runtime.console.is_empty());
        idle(&runtime);
    }
}

#[test]
fn throw_completion_microtask_reports_original_value_and_continues_without_metadata_calls() {
    let (mut runtime, mut document) = fresh();
    let completion = runtime
        .execute(
            r#"
        var log='',token={};
        Object.defineProperty(token,'constructor',{get:function(){log+='X';while(true){}}});
        queueMicrotask(function(){log+='A';throw token;});
        queueMicrotask(function(){log+='B';});
        log+='S';log;
    "#,
            &mut document,
        )
        .unwrap();
    assert_eq!(completion, Value::String("S".into()));
    assert_eq!(
        runtime.execute("log", &mut document).unwrap(),
        Value::String("SAB".into())
    );
    assert_eq!(
        runtime.console,
        ["Uncaught microtask exception: uncaught exception: [object Object]"]
    );
    idle(&runtime);
}

#[test]
fn throw_completion_host_listener_reporting_keeps_next_listener_and_event_cleanup() {
    let (mut runtime, mut document) = fresh();
    runtime
        .execute(
            r#"
        var log='',token={},retained,b=document.getElementById('b');
        Object.defineProperty(token,'constructor',{get:function(){log+='X';while(true){}}});
        b.addEventListener('click',function(e){retained=e;log+='A';throw token;});
        b.addEventListener('click',function(){log+='B';});
    "#,
            &mut document,
        )
        .unwrap();
    runtime
        .dispatch_click(document.query_selector("#b").unwrap(), &mut document)
        .unwrap();
    assert_eq!(
        runtime
            .execute(
                "log+'|'+(retained.currentTarget===null)+'|'+retained.eventPhase",
                &mut document
            )
            .unwrap(),
        Value::String("AB|true|0".into())
    );
    assert_eq!(
        runtime.console,
        ["Uncaught event listener exception: uncaught exception: [object Object]"]
    );
    idle(&runtime);
}

#[test]
fn throw_completion_data_diagnostics_and_independent_canonical_identity_are_explicit() {
    for (setup, display, intrinsic) in [
        ("var token=new TypeError();", "TypeError", Some("TypeError")),
        (
            "function Test262Error(){}var token=new Test262Error();",
            "Test262Error",
            None,
        ),
        (
            "function Fake(){}Object.defineProperty(Fake,'name',{value:'TypeError'});var token=new Fake();",
            "TypeError",
            None,
        ),
        (
            "var token=Object.create(TypeError.prototype);",
            "TypeError",
            None,
        ),
        (
            "var token=new TypeError();token.constructor=RangeError;",
            "RangeError",
            None,
        ),
        (
            "function StoredLabel(){}Math.constructor=StoredLabel;var token=Math;",
            "StoredLabel",
            None,
        ),
        (
            "function StoredLabel(){}JSON.constructor=StoredLabel;var token=JSON;",
            "StoredLabel",
            None,
        ),
        (
            "Object.defineProperty(Math,'constructor',{get:function(){reads++;throw 7;}});var token=Math;",
            "ThrownValue",
            None,
        ),
        (
            "Object.defineProperty(JSON,'constructor',{get:function(){reads++;throw 7;}});var token=JSON;",
            "ThrownValue",
            None,
        ),
        (
            "var parent={};Object.defineProperty(parent,'constructor',{get:function(){reads++;throw 7;}});var token=Object.create(parent);",
            "ThrownValue",
            None,
        ),
        (
            "function C(){}var parent={constructor:C};var token=Object.create(parent);Object.defineProperty(token,'constructor',{set:function(v){reads++;}});",
            "ThrownValue",
            None,
        ),
        ("var token=document;", "ThrownValue", None),
        (
            "var token=document.getElementById('b').style;",
            "ThrownValue",
            None,
        ),
        (
            "Object.defineProperty(TypeError,'name',{value:'Renamed'});var token=new TypeError();",
            "Renamed",
            Some("TypeError"),
        ),
        (
            "Object.defineProperty(TypeError,'name',{value:19});var token=new TypeError();",
            "ThrownValue",
            Some("TypeError"),
        ),
        (
            "Object.defineProperty(TypeError,'name',{get:function(){reads++;throw 7;}});var token=new TypeError();",
            "ThrownValue",
            Some("TypeError"),
        ),
        (
            "var token=new TypeError();Object.defineProperty(token,'constructor',{get:function(){reads++;return TypeError;}});",
            "ThrownValue",
            None,
        ),
        (
            "var token=new TypeError();Object.defineProperty(token,'constructor',{set:function(v){reads++;}});",
            "ThrownValue",
            None,
        ),
    ] {
        let (mut runtime, mut document) = fresh();
        let token = runtime
            .execute(&format!("var reads=0;{setup}token;"), &mut document)
            .unwrap();
        let error = runtime.execute("throw token;", &mut document).unwrap_err();
        let ErrorKind::Thrown(value) = &error.kind else {
            panic!("author error value")
        };
        assert_eq!(value, &token, "{setup}");
        assert_eq!(error.name(), display, "{setup}");
        assert_eq!(error.intrinsic_error_name(), intrinsic, "{setup}");
        assert_eq!(
            runtime.execute("reads", &mut document).unwrap(),
            Value::Number(0.0)
        );
        idle(&runtime);
    }
}
