//! Promise.try public contracts, frozen independently before production.
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

fn fatal_stop(cases: &[(&str, bool, &str, &str)], expression: &str) {
    for &(case_id, unsupported, source, expected) in cases {
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
    // Ordinary queued work still drains in a fresh Runtime. Suspended runtimes
    // above are never manually reset or used for this preservation control.
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        execute_mode(
            &mut runtime,
            &mut document,
            strict,
            r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
var trace='S';queueMicrotask(function(){trace+='Q';});"###,
        )
        .unwrap();
        let observed = execute_mode(&mut runtime, &mut document, strict, r###"trace"###).unwrap();
        assert_eq!(observed, Value::String("SQ".into()));
        assert!(!runtime.microtasks_suspended());
    }
}

#[test]
fn promise_try_callback_terminal_direct_and_queued() {
    fatal_stop(
        &[
            (
                "callback_direct_resource",
                false,
                r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
var P=Promise,f=P.try,token={},holder={},trace='',constructors=0,captures=0,resolves=0,rejects=0,gets=0,after=false;
function resolve(){resolves++;trace+='R';}
function reject(){rejects++;trace+='J';}
function C(ex){constructors++;trace+='C';ex(resolve,reject);captures++;return holder;}
function callback(){trace+='F';while(true){}}
queueMicrotask(function(){trace+='Q';console.log('late');});
try{f.call(C,callback);after=true;}catch(e){trace+='X';}finally{trace+='Z';}trace+='A';
"###,
                "F|0|0|0|0|0|false",
            ),
            (
                "callback_direct_unsupported",
                true,
                r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
var P=Promise,f=P.try,token={},holder={},trace='',constructors=0,captures=0,resolves=0,rejects=0,gets=0,after=false;
function resolve(){resolves++;trace+='R';}
function reject(){rejects++;trace+='J';}
function C(ex){constructors++;trace+='C';ex(resolve,reject);captures++;return holder;}
function callback(){trace+='F';eval('0');}
queueMicrotask(function(){trace+='Q';console.log('late');});
try{f.call(C,callback);after=true;}catch(e){trace+='X';}finally{trace+='Z';}trace+='A';
"###,
                "F|0|0|0|0|0|false",
            ),
            (
                "callback_queued_resource",
                false,
                r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
var P=Promise,f=P.try,token={},holder={},trace='',constructors=0,captures=0,resolves=0,rejects=0,gets=0,after=false;
function resolve(){resolves++;trace+='R';}
function reject(){rejects++;trace+='J';}
function C(ex){constructors++;trace+='C';ex(resolve,reject);captures++;return holder;}
function callback(){trace+='F';while(true){}}
queueMicrotask(function(){try{f.call(C,callback);after=true;}catch(e){trace+='X';}finally{trace+='Z';}trace+='A';});
queueMicrotask(function(){trace+='Q';console.log('late');});trace+='S';
"###,
                "SF|0|0|0|0|0|false",
            ),
            (
                "callback_queued_unsupported",
                true,
                r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
var P=Promise,f=P.try,token={},holder={},trace='',constructors=0,captures=0,resolves=0,rejects=0,gets=0,after=false;
function resolve(){resolves++;trace+='R';}
function reject(){rejects++;trace+='J';}
function C(ex){constructors++;trace+='C';ex(resolve,reject);captures++;return holder;}
function callback(){trace+='F';eval('0');}
queueMicrotask(function(){try{f.call(C,callback);after=true;}catch(e){trace+='X';}finally{trace+='Z';}trace+='A';});
queueMicrotask(function(){trace+='Q';console.log('late');});trace+='S';
"###,
                "SF|0|0|0|0|0|false",
            ),
        ],
        r###"[trace,constructors,captures,resolves,rejects,gets,after].join('|')"###,
    );
}

#[test]
fn promise_try_constructor_and_genuine_constructor_getter_terminal() {
    fatal_stop(
        &[
            (
                "constructor_resource",
                false,
                r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
var P=Promise,f=P.try,token={},holder={},trace='',constructors=0,captures=0,resolves=0,rejects=0,gets=0,after=false;
function resolve(){resolves++;trace+='R';}
function reject(){rejects++;trace+='J';}
function C(ex){constructors++;trace+='C';while(true){}}
function callback(){trace+='F';return token;}
queueMicrotask(function(){trace+='Q';console.log('late');});
try{f.call(C,callback);after=true;}catch(e){trace+='X';}finally{trace+='Z';}trace+='A';
"###,
                "FC|1|0|0|0|0|false",
            ),
            (
                "constructor_unsupported",
                true,
                r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
var P=Promise,f=P.try,token={},holder={},trace='',constructors=0,captures=0,resolves=0,rejects=0,gets=0,after=false;
function resolve(){resolves++;trace+='R';}
function reject(){rejects++;trace+='J';}
function C(ex){constructors++;trace+='C';eval('0');}
function callback(){trace+='F';return token;}
queueMicrotask(function(){trace+='Q';console.log('late');});
try{f.call(C,callback);after=true;}catch(e){trace+='X';}finally{trace+='Z';}trace+='A';
"###,
                "FC|1|0|0|0|0|false",
            ),
            (
                "constructor_getter_resource",
                false,
                r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
var P=Promise,f=P.try,token={},holder={},trace='',constructors=0,captures=0,resolves=0,rejects=0,gets=0,after=false;
function resolve(){resolves++;trace+='R';}
function reject(){rejects++;trace+='J';}
function C(ex){constructors++;trace+='C';ex(resolve,reject);captures++;return holder;}
var p=P.resolve(token);Object.defineProperty(p,'constructor',{get:function(){gets++;trace+='G';while(true){}}});
function callback(){trace+='F';return p;}
queueMicrotask(function(){trace+='Q';console.log('late');});
try{f.call(C,callback);after=true;}catch(e){trace+='X';}finally{trace+='Z';}trace+='A';
"###,
                "FG|0|0|0|0|1|false",
            ),
            (
                "constructor_getter_unsupported",
                true,
                r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
var P=Promise,f=P.try,token={},holder={},trace='',constructors=0,captures=0,resolves=0,rejects=0,gets=0,after=false;
function resolve(){resolves++;trace+='R';}
function reject(){rejects++;trace+='J';}
function C(ex){constructors++;trace+='C';ex(resolve,reject);captures++;return holder;}
var p=P.resolve(token);Object.defineProperty(p,'constructor',{get:function(){gets++;trace+='G';eval('0');}});
function callback(){trace+='F';return p;}
queueMicrotask(function(){trace+='Q';console.log('late');});
try{f.call(C,callback);after=true;}catch(e){trace+='X';}finally{trace+='Z';}trace+='A';
"###,
                "FG|0|0|0|0|1|false",
            ),
        ],
        r###"[trace,constructors,captures,resolves,rejects,gets,after].join('|')"###,
    );
}

#[test]
fn promise_try_resolver_and_rejector_terminal() {
    fatal_stop(
        &[
            (
                "resolve_resource",
                false,
                r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
var P=Promise,f=P.try,token={},holder={},trace='',constructors=0,captures=0,resolves=0,rejects=0,gets=0,after=false;
function resolve(){resolves++;trace+='R';while(true){}}
function reject(){rejects++;trace+='J';}
function C(ex){constructors++;trace+='C';ex(resolve,reject);captures++;return holder;}
function callback(){trace+='F';return token;}
queueMicrotask(function(){trace+='Q';console.log('late');});
try{f.call(C,callback);after=true;}catch(e){trace+='X';}finally{trace+='Z';}trace+='A';
"###,
                "FCR|1|1|1|0|0|false",
            ),
            (
                "resolve_unsupported",
                true,
                r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
var P=Promise,f=P.try,token={},holder={},trace='',constructors=0,captures=0,resolves=0,rejects=0,gets=0,after=false;
function resolve(){resolves++;trace+='R';eval('0');}
function reject(){rejects++;trace+='J';}
function C(ex){constructors++;trace+='C';ex(resolve,reject);captures++;return holder;}
function callback(){trace+='F';return token;}
queueMicrotask(function(){trace+='Q';console.log('late');});
try{f.call(C,callback);after=true;}catch(e){trace+='X';}finally{trace+='Z';}trace+='A';
"###,
                "FCR|1|1|1|0|0|false",
            ),
            (
                "reject_resource",
                false,
                r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
var P=Promise,f=P.try,token={},holder={},trace='',constructors=0,captures=0,resolves=0,rejects=0,gets=0,after=false;
function resolve(){resolves++;trace+='R';}
function reject(){rejects++;trace+='J';while(true){}}
function C(ex){constructors++;trace+='C';ex(resolve,reject);captures++;return holder;}
function callback(){trace+='F';throw token;}
queueMicrotask(function(){trace+='Q';console.log('late');});
try{f.call(C,callback);after=true;}catch(e){trace+='X';}finally{trace+='Z';}trace+='A';
"###,
                "FCJ|1|1|0|1|0|false",
            ),
            (
                "reject_unsupported",
                true,
                r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
var P=Promise,f=P.try,token={},holder={},trace='',constructors=0,captures=0,resolves=0,rejects=0,gets=0,after=false;
function resolve(){resolves++;trace+='R';}
function reject(){rejects++;trace+='J';eval('0');}
function C(ex){constructors++;trace+='C';ex(resolve,reject);captures++;return holder;}
function callback(){trace+='F';throw token;}
queueMicrotask(function(){trace+='Q';console.log('late');});
try{f.call(C,callback);after=true;}catch(e){trace+='X';}finally{trace+='Z';}trace+='A';
"###,
                "FCJ|1|1|0|1|0|false",
            ),
        ],
        r###"[trace,constructors,captures,resolves,rejects,gets,after].join('|')"###,
    );
}
