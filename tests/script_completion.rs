use eris::dom::Document;
use eris::script::{Runtime, Value};

fn scalar_tag(value: Value) -> String {
    match value {
        Value::Undefined => "undefined".into(),
        Value::Null => "null".into(),
        Value::Bool(value) => format!("bool:{value}"),
        Value::Number(value) => format!("number:{:016x}", value.to_bits()),
        Value::String(value) => format!("string:{:?}", value.units()),
        value => panic!("expected a scalar completion, got {value:?}"),
    }
}

fn check_group(group: &str) {
    let mut count = 0;
    for line in include_str!("fixtures/script-completion.tsv").lines() {
        if line.starts_with('#') {
            continue;
        }
        let mut columns = line.splitn(3, '\t');
        let id = columns.next().unwrap();
        let expected = columns.next().unwrap();
        let source = columns.next().unwrap();
        if id.split_once('/').unwrap().0 != group {
            continue;
        }
        count += 1;
        for strict in [false, true] {
            let mut runtime = Runtime::new();
            let mut document = Document::parse("");
            let result = if strict {
                runtime.execute_strict(source, &mut document)
            } else {
                runtime.execute(source, &mut document)
            };
            let value = result.unwrap_or_else(|error| panic!("{id}, strict={strict}: {error}"));
            assert_eq!(
                scalar_tag(value),
                expected,
                "{id}, strict={strict}: {source}"
            );
        }
    }
    assert!(count > 0, "empty completion case group {group}");
}

#[test]
fn lists_and_labels_preserve_empty_and_exact_primitive_values() {
    check_group("list");
}

#[test]
fn if_statements_fill_empty_completions_with_undefined() {
    check_group("if");
}

#[test]
fn loops_preserve_values_on_normal_and_targeted_abrupt_paths() {
    check_group("loop");
}

#[test]
fn switches_preserve_fallthrough_values_and_abrupt_targets() {
    check_group("switch");
}

#[test]
fn try_catch_finally_select_the_correct_completion() {
    check_group("try");
}

#[test]
fn function_returns_are_distinct_from_statement_values() {
    check_group("return");
}

#[test]
fn completions_preserve_reference_identity_and_nan() {
    for strict in [false, true] {
        for initializer in ["{}", "[]", "function(){}", "new Number(7)"] {
            for body in [
                "token; var unused;",
                "token; {}",
                "mark:{token;break mark;}",
                "while(true){token;break;}",
                "for(var i=0;i<2;i++){token;continue;}",
                "for(var k in {a:1}){token;var unused;}",
                "switch(1){case 1:token;break;}",
                "try{token;var unused;}finally{undefined;}",
                "try{throw token;}catch(e){e;var unused;}",
            ] {
                let mut runtime = Runtime::new();
                let mut document = Document::parse("");
                let source = format!("var token={initializer};{body}");
                let result = if strict {
                    runtime.execute_strict(&source, &mut document)
                } else {
                    runtime.execute(&source, &mut document)
                }
                .unwrap();
                let token = runtime.execute("token", &mut document).unwrap();
                assert_eq!(result, token, "strict={strict}: {source}");
            }
        }
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        let source = "try{NaN;var unused;}finally{8;}";
        let result = if strict {
            runtime.execute_strict(source, &mut document)
        } else {
            runtime.execute(source, &mut document)
        }
        .unwrap();
        assert!(matches!(result, Value::Number(value) if value.is_nan()));
    }
}

#[test]
fn completion_values_cannot_override_host_termination() {
    for strict in [false, true] {
        for body in [
            "mark:while(true){7;continue mark;}",
            "mark:for(;;){try{7;continue mark;}finally{8;}}",
            "function recur(){try{return recur();}finally{8;}}recur();",
            "eval('7')",
        ] {
            let mut runtime = Runtime::new();
            let mut document = Document::parse("");
            runtime
                .execute("var caught=false,finalized=false;", &mut document)
                .unwrap();
            let source = format!(
                "exit:{{try{{{body}}}catch(e){{caught=true;}}finally{{finalized=true;break exit;}}}}"
            );
            let error = if strict {
                runtime.execute_strict(&source, &mut document)
            } else {
                runtime.execute(&source, &mut document)
            }
            .unwrap_err();
            if body.starts_with("eval") {
                assert!(error.is_unsupported(), "{error}");
            } else {
                assert!(error.is_resource_limit(), "{error}");
            }
            assert_eq!(
                runtime
                    .execute("caught || finalized", &mut document)
                    .unwrap(),
                Value::Bool(false),
                "strict={strict}: {source}"
            );
        }
    }
}

#[test]
fn recursive_statement_forms_stop_before_native_stack_exhaustion() {
    for form in [
        "block", "if", "while", "do", "for", "for-in", "try", "switch", "label",
    ] {
        let mut body = "return recur();".to_owned();
        for depth in 0..12 {
            body = match form {
                "block" => format!("{{{body}}}"),
                "if" => format!("if(true){{{body}}}"),
                "while" => format!("while(true){{{body}}}"),
                "do" => format!("do{{{body}}}while(true);"),
                "for" => format!("for(;;){{{body}}}"),
                "for-in" => format!("for(var key in {{a:1}}){{{body}}}"),
                "try" => format!("try{{{body}}}finally{{cleaned=true;7;}}"),
                "switch" => format!("switch(0){{case 0:{body}}}"),
                "label" => format!("mark{depth}:{{{body}}}"),
                _ => unreachable!(),
            };
        }
        for strict in [false, true] {
            let source = format!("var cleaned=false;function recur(){{{body}}}recur();");
            Runtime::parse_only(&source).unwrap();
            let mut runtime = Runtime::new();
            let mut document = Document::parse("");
            let error = if strict {
                runtime.execute_strict(&source, &mut document)
            } else {
                runtime.execute(&source, &mut document)
            }
            .unwrap_err();
            assert!(
                error.is_resource_limit(),
                "{form}, strict={strict}: {error}"
            );
            assert_eq!(
                runtime.execute("cleaned", &mut document).unwrap(),
                Value::Bool(false),
                "{form}, strict={strict}"
            );
        }
    }
}
