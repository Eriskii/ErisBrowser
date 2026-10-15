//! Call/new argument +In literals held before the grammar correction.
//! Parent-compatible Runtime and active/legacy parser interfaces only.
use super::*;

fn execute_mode(
    runtime: &mut Runtime,
    document: &mut Document,
    strict: bool,
    source: &str,
) -> Result<Value> {
    if strict {
        runtime.execute_strict(source, document)
    } else {
        runtime.execute(source, document)
    }
}

fn clean(runtime: &Runtime, case_id: &str, strict: bool) {
    assert_eq!(
        (
            runtime.frames.len(),
            runtime.eval_depth,
            runtime.stack_units,
            runtime.calls
        ),
        (0, 0, 0, 0),
        "{case_id} strict={strict}"
    );
    assert!(!runtime.microtasks_suspended(), "{case_id} strict={strict}");
    assert!(runtime.console.is_empty(), "{case_id} strict={strict}");
}

fn succeeds(case_id: &str, source: &str, expected: &str) {
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        let observed = execute_mode(&mut runtime, &mut document, strict, source)
            .unwrap_or_else(|error| panic!("{case_id} strict={strict} runtime: {error:?}"));
        assert_eq!(
            observed,
            Value::String(expected.into()),
            "{case_id} strict={strict}"
        );
        clean(&runtime, case_id, strict);

        let active = parser::Parser::program_context(source, false, strict)
            .map(|unit| code::canonical(&unit))
            .unwrap_or_else(|error| panic!("{case_id} strict={strict} active: {error:?}"));
        let legacy = parser_legacy::Parser::program_context(source, false, strict)
            .and_then(code::compile)
            .map(|unit| code::canonical(&unit))
            .unwrap_or_else(|error| panic!("{case_id} strict={strict} legacy: {error:?}"));
        assert_eq!(active, legacy, "{case_id} strict={strict}");
    }
}

fn syntax_error(case_id: &str, source: &str) {
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        let runtime_error = execute_mode(&mut runtime, &mut document, strict, source).unwrap_err();
        let active_error = parser::Parser::program_context(source, false, strict).unwrap_err();
        let legacy_error =
            parser_legacy::Parser::program_context(source, false, strict).unwrap_err();
        for (path, error) in [
            ("runtime", runtime_error),
            ("active", active_error),
            ("legacy", legacy_error),
        ] {
            assert_eq!(
                error.name(),
                "SyntaxError",
                "{case_id} strict={strict} {path}"
            );
            assert!(
                error.is_parse_error() && !error.is_unsupported(),
                "{case_id} strict={strict} {path}: {error:?}"
            );
        }
        clean(&runtime, case_id, strict);
    }
}

#[test]
fn call_argument_in_call_direct() {
    succeeds(
        "call-direct",
        r###"var o={x:1}, out='';function f(v){out+=v?'T':'F';} for(f('x' in o);false;){} out;"###,
        "T",
    );
}

#[test]
fn call_argument_in_new_direct() {
    succeeds(
        "new-direct",
        r###"var o={x:1}, out='';function C(v){out+=v?'T':'F';} for(new C('x' in o);false;){} out;"###,
        "T",
    );
}

#[test]
fn call_argument_in_call_nested_multiple() {
    succeeds(
        "call-nested-multiple",
        r###"var o={x:1}, out='';function f(a,b){out+=a+','+b+';';return false;} for(f('x' in o,f('y' in o,'x' in o));false;){} out;"###,
        "false,true;true,false;",
    );
}

#[test]
fn call_argument_in_new_nested_multiple() {
    succeeds(
        "new-nested-multiple",
        r###"var o={x:1}, out='';function C(a,b){out+=a+','+b+';';this.flag=a;} for(new C('x' in o,new C('y' in o,'x' in o).flag);false;){} out;"###,
        "false,true;true,false;",
    );
}

#[test]
fn call_argument_in_call_mixed_spread() {
    succeeds(
        "call-mixed-spread",
        r###"var o={x:1}, out='';function f(a,b,c){out=a+','+b+','+c;} for(f('x' in o,...'missing' in o?[]:[7],'missing' in o);false;){} out;"###,
        "true,7,false",
    );
}

#[test]
fn call_argument_in_new_spread_trailing() {
    succeeds(
        "new-spread-trailing",
        r###"var o={x:1}, out='';function C(a,b){out=a+','+b;} for(new C(...'x' in o?[3,4]:[],);false;){} out;"###,
        "3,4",
    );
}

#[test]
fn call_argument_in_declaration_var() {
    succeeds(
        "declaration-var",
        r###"var o={x:1}, out='';function f(v){out+=v?'T':'F';return 0;} for(var saved=f('x' in o);false;){} out;"###,
        "T",
    );
}

#[test]
fn call_argument_in_declaration_let() {
    succeeds(
        "declaration-let",
        r###"var o={x:1}, out='';function f(v){out+=v?'T':'F';return 0;} for(let saved=f('x' in o);false;){} out;"###,
        "T",
    );
}

#[test]
fn call_argument_in_parenthesized_call_new() {
    succeeds(
        "parenthesized-call-new",
        r###"var o={x:1}, out='';function f(v){out+=v?'T':'F';} function C(v){out+=v?'T':'F';} for(f(('x' in o));false;){} for(new C(('x' in o));false;){} out;"###,
        "TT",
    );
}

#[test]
fn call_argument_in_parenthesized_spread() {
    succeeds(
        "parenthesized-spread",
        r###"var o={x:1}, out='';function f(a,b){out+=a+','+b+';';} function C(a,b){out+=a+','+b+';';} for(f(...(('x' in o)?[1,2]:[]));false;){} for(new C(...(('x' in o)?[3,4]:[]),);false;){} out;"###,
        "1,2;3,4;",
    );
}

#[test]
fn call_argument_in_outer_for_in_after_call_new() {
    succeeds(
        "outer-for-in-after-call-new",
        r###"var holder={}, out=''; function f(v){out+='A';return holder;} function C(v){out+='B';return holder;} for(f(0).slot in {a:1}){out+=holder.slot;} for(new C(0).slot in {a:1}){out+=holder.slot;} out;"###,
        "AaBa",
    );
}

#[test]
fn call_argument_in_inner_and_outer_in() {
    succeeds(
        "inner-and-outer-in",
        r###"var o={x:1}, out='';var holder={}; function f(v){out+=v?'T':'F';return holder;} function C(v){out+=v?'T':'F';return holder;} for(f('x' in o).slot in {a:1}){out+=holder.slot;} for(new C('x' in o).slot in {a:1}){out+=holder.slot;} out;"###,
        "TaTa",
    );
}

#[test]
fn call_argument_in_no_leak_call_classic_for() {
    syntax_error(
        "no-leak-call-classic-for",
        r###"function f(x){return 'x';} for(f(0) in {x:1};false;){}"###,
    );
}

#[test]
fn call_argument_in_no_leak_new_classic_for() {
    syntax_error(
        "no-leak-new-classic-for",
        r###"function C(x){} for(new C(0) in {x:1};false;){}"###,
    );
}

#[test]
fn call_argument_in_parenthesized_outer_in() {
    succeeds(
        "parenthesized-outer-in",
        r###"var out=''; function f(x){out+='F';return 'x';} for((f(0) in {x:1});false;){} out;"###,
        "F",
    );
}
