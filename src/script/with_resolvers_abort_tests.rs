//! withResolvers host-terminal contracts; no manual budget or queue reset.
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

fn fatal_stop(cases: &[(&str, bool, &str)], expression: &str, expected: &str) {
    for &(case_id, unsupported, source) in cases {
        for strict in [false, true] {
            let mut runtime = Runtime::try_new().unwrap();
            let mut document = Document::parse("");
            let error = execute_mode(&mut runtime, &mut document, strict, source).unwrap_err();
            assert_eq!(
                error.is_unsupported(),
                unsupported,
                "{case_id} strict={strict}: {error:?}"
            );
            assert_eq!(
                error.is_resource_limit(),
                !unsupported,
                "{case_id} strict={strict}: {error:?}"
            );
            assert!(runtime.microtasks_suspended(), "{case_id} strict={strict}");
            for _ in 0..2 {
                let observed =
                    execute_mode(&mut runtime, &mut document, strict, expression).unwrap();
                assert_eq!(
                    observed,
                    Value::String(expected.into()),
                    "{case_id} strict={strict}"
                );
                assert!(runtime.microtasks_suspended());
                assert!(runtime.console.is_empty());
            }
        }
    }
    // Fresh runtime proves ordinary queued work still drains; the suspended
    // runtimes above are never reset or reused for this preservation control.
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        execute_mode(
            &mut runtime,
            &mut document,
            strict,
            r###"var trace='S';queueMicrotask(function(){trace+='Q';});"###,
        )
        .unwrap();
        let value = execute_mode(&mut runtime, &mut document, strict, r###"trace"###).unwrap();
        assert_eq!(value, Value::String("SQ".into()));
        assert!(!runtime.microtasks_suspended());
    }
}

#[test]
fn promise_with_resolvers_before_capture_terminal() {
    fatal_stop(
        &[
            (
                "before_capture_resource",
                false,
                r###"if(typeof Promise.withResolvers!=='function')throw new Error('Promise.withResolvers missing');
var trace='',captures=0,fulfills=0,rejects=0,after=false;function resolve(){fulfills++;}function reject(){rejects++;}
function C(ex){trace+='C';while(true){}ex(resolve,reject);captures++;trace+='E';}
queueMicrotask(function(){trace+='Q';});try{Promise.withResolvers.call(C);after=true;}catch(e){trace+='X';}finally{trace+='F';}trace+='Z';
"###,
            ),
            (
                "before_capture_unsupported",
                true,
                r###"if(typeof Promise.withResolvers!=='function')throw new Error('Promise.withResolvers missing');
var trace='',captures=0,fulfills=0,rejects=0,after=false;function resolve(){fulfills++;}function reject(){rejects++;}
function C(ex){trace+='C';eval('0');ex(resolve,reject);captures++;trace+='E';}
queueMicrotask(function(){trace+='Q';});try{Promise.withResolvers.call(C);after=true;}catch(e){trace+='X';}finally{trace+='F';}trace+='Z';
"###,
            ),
        ],
        r###"[trace,captures,fulfills,rejects,after].join('|')"###,
        "C|0|0|0|false",
    );
}

#[test]
fn promise_with_resolvers_after_capture_terminal() {
    fatal_stop(
        &[
            (
                "after_capture_resource",
                false,
                r###"if(typeof Promise.withResolvers!=='function')throw new Error('Promise.withResolvers missing');
var trace='',captures=0,fulfills=0,rejects=0,after=false;function resolve(){fulfills++;}function reject(){rejects++;}
function C(ex){trace+='C';ex(resolve,reject);captures++;trace+='E';while(true){}}
queueMicrotask(function(){trace+='Q';});try{Promise.withResolvers.call(C);after=true;}catch(e){trace+='X';}finally{trace+='F';}trace+='Z';
"###,
            ),
            (
                "after_capture_unsupported",
                true,
                r###"if(typeof Promise.withResolvers!=='function')throw new Error('Promise.withResolvers missing');
var trace='',captures=0,fulfills=0,rejects=0,after=false;function resolve(){fulfills++;}function reject(){rejects++;}
function C(ex){trace+='C';ex(resolve,reject);captures++;trace+='E';eval('0');}
queueMicrotask(function(){trace+='Q';});try{Promise.withResolvers.call(C);after=true;}catch(e){trace+='X';}finally{trace+='F';}trace+='Z';
"###,
            ),
        ],
        r###"[trace,captures,fulfills,rejects,after].join('|')"###,
        "CE|1|0|0|false",
    );
}

#[test]
fn promise_with_resolvers_queued_before_capture_terminal() {
    fatal_stop(
        &[
            (
                "queued_before_capture_resource",
                false,
                r###"if(typeof Promise.withResolvers!=='function')throw new Error('Promise.withResolvers missing');
var trace='',captures=0,fulfills=0,rejects=0,after=false;function resolve(){fulfills++;}function reject(){rejects++;}
function C(ex){trace+='C';while(true){}ex(resolve,reject);captures++;trace+='E';}
queueMicrotask(function(){try{Promise.withResolvers.call(C);after=true;}catch(e){trace+='X';}finally{trace+='F';}trace+='Z';});queueMicrotask(function(){trace+='Q';});trace+='S';
"###,
            ),
            (
                "queued_before_capture_unsupported",
                true,
                r###"if(typeof Promise.withResolvers!=='function')throw new Error('Promise.withResolvers missing');
var trace='',captures=0,fulfills=0,rejects=0,after=false;function resolve(){fulfills++;}function reject(){rejects++;}
function C(ex){trace+='C';eval('0');ex(resolve,reject);captures++;trace+='E';}
queueMicrotask(function(){try{Promise.withResolvers.call(C);after=true;}catch(e){trace+='X';}finally{trace+='F';}trace+='Z';});queueMicrotask(function(){trace+='Q';});trace+='S';
"###,
            ),
        ],
        r###"[trace,captures,fulfills,rejects,after].join('|')"###,
        "SC|0|0|0|false",
    );
}
