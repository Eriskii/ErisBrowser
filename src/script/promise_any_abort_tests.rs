//! Promise.any host-terminal cases, distinct from catchable rejection errors.
//! Existing allSettled fatal helper is unchanged; no in-operation resets.
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

fn fatal_stop(cases: [(&str, bool, &str); 2], expected: &str) {
    for (case_id, unsupported, source) in cases {
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
                let value = execute_mode(
                    &mut runtime,
                    &mut document,
                    strict,
                    r###"[trace,resolves,rejects,returnGets,returnCalls,after].join('|')"###,
                )
                .unwrap();
                assert_eq!(
                    value,
                    Value::String(expected.into()),
                    "{case_id} strict={strict}"
                );
                assert!(runtime.microtasks_suspended());
                assert!(runtime.console.is_empty());
            }
        }
    }
}

#[test]
fn promise_any_resolve_terminal_stops_without_later_author_work() {
    fatal_stop(
        [
            (
                "any_resolve_resource",
                false,
                r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');if(typeof Promise.any!=='function')throw new Error('Promise.any missing');var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0;queueMicrotask(function(){trace+='Q';});var saved,step=0;function C(ex){ex(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';while(true){}return {get then(){trace+='G';return function(f,r){trace+='T';saved=r;};}};};var it={next:function(){trace+='N';return step++===0?{done:false,value:1}:{done:true};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var items={};items[Symbol.iterator]=function(){return it;};Promise.any.call(C,items);after=true;"###,
            ),
            (
                "any_resolve_unsupported",
                true,
                r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');if(typeof Promise.any!=='function')throw new Error('Promise.any missing');var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0;queueMicrotask(function(){trace+='Q';});var saved,step=0;function C(ex){ex(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';eval('0');return {get then(){trace+='G';return function(f,r){trace+='T';saved=r;};}};};var it={next:function(){trace+='N';return step++===0?{done:false,value:1}:{done:true};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var items={};items[Symbol.iterator]=function(){return it;};Promise.any.call(C,items);after=true;"###,
            ),
        ],
        "NR|0|0|0|0|false",
    );
}

#[test]
fn promise_any_then_get_terminal_stops_without_later_author_work() {
    fatal_stop(
        [
            (
                "any_then_get_resource",
                false,
                r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');if(typeof Promise.any!=='function')throw new Error('Promise.any missing');var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0;queueMicrotask(function(){trace+='Q';});var saved,step=0;function C(ex){ex(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';while(true){}return function(f,r){trace+='T';saved=r;};}};};var it={next:function(){trace+='N';return step++===0?{done:false,value:1}:{done:true};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var items={};items[Symbol.iterator]=function(){return it;};Promise.any.call(C,items);after=true;"###,
            ),
            (
                "any_then_get_unsupported",
                true,
                r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');if(typeof Promise.any!=='function')throw new Error('Promise.any missing');var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0;queueMicrotask(function(){trace+='Q';});var saved,step=0;function C(ex){ex(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';eval('0');return function(f,r){trace+='T';saved=r;};}};};var it={next:function(){trace+='N';return step++===0?{done:false,value:1}:{done:true};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var items={};items[Symbol.iterator]=function(){return it;};Promise.any.call(C,items);after=true;"###,
            ),
        ],
        "NRG|0|0|0|0|false",
    );
}

#[test]
fn promise_any_exhaustion_reject_terminal_stops_without_later_author_work() {
    fatal_stop(
        [
            (
                "any_exhaustion_reject_resource",
                false,
                r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');if(typeof Promise.any!=='function')throw new Error('Promise.any missing');var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0;queueMicrotask(function(){trace+='Q';});var saved,step=0;function C(ex){ex(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';while(true){}});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(f,r){trace+='T';r(x);trace+='F';};}};};var it={next:function(){trace+='N';return step++===0?{done:false,value:1}:{done:true};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var items={};items[Symbol.iterator]=function(){return it;};Promise.any.call(C,items);after=true;"###,
            ),
            (
                "any_exhaustion_reject_unsupported",
                true,
                r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');if(typeof Promise.any!=='function')throw new Error('Promise.any missing');var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0;queueMicrotask(function(){trace+='Q';});var saved,step=0;function C(ex){ex(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';eval('0');});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(f,r){trace+='T';r(x);trace+='F';};}};};var it={next:function(){trace+='N';return step++===0?{done:false,value:1}:{done:true};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var items={};items[Symbol.iterator]=function(){return it;};Promise.any.call(C,items);after=true;"###,
            ),
        ],
        "NRGTFNJ|0|1|0|0|false",
    );
}

#[test]
fn promise_any_saved_reject_terminal_stops_without_later_author_work() {
    fatal_stop(
        [
            (
                "any_saved_reject_resource",
                false,
                r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');if(typeof Promise.any!=='function')throw new Error('Promise.any missing');var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0;queueMicrotask(function(){trace+='Q';});var saved,step=0;function C(ex){ex(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';while(true){}});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(f,r){trace+='T';saved=r;};}};};var it={next:function(){trace+='N';return step++===0?{done:false,value:1}:{done:true};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var items={};items[Symbol.iterator]=function(){return it;};Promise.any.call(C,items);trace+='B';saved(1);after=true;"###,
            ),
            (
                "any_saved_reject_unsupported",
                true,
                r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');if(typeof Promise.any!=='function')throw new Error('Promise.any missing');var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0;queueMicrotask(function(){trace+='Q';});var saved,step=0;function C(ex){ex(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';eval('0');});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(f,r){trace+='T';saved=r;};}};};var it={next:function(){trace+='N';return step++===0?{done:false,value:1}:{done:true};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var items={};items[Symbol.iterator]=function(){return it;};Promise.any.call(C,items);trace+='B';saved(1);after=true;"###,
            ),
        ],
        "NRGTNBJ|0|1|0|0|false",
    );
}

#[test]
fn promise_any_then_call_terminal_stops_without_later_author_work() {
    fatal_stop(
        [
            (
                "any_then_call_resource",
                false,
                r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');if(typeof Promise.any!=='function')throw new Error('Promise.any missing');var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0;queueMicrotask(function(){trace+='Q';});var saved,step=0;function C(ex){ex(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(f,r){trace+='T';while(true){}saved=r;};}};};var it={next:function(){trace+='N';return step++===0?{done:false,value:1}:{done:true};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var items={};items[Symbol.iterator]=function(){return it;};Promise.any.call(C,items);after=true;"###,
            ),
            (
                "any_then_call_unsupported",
                true,
                r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');if(typeof Promise.any!=='function')throw new Error('Promise.any missing');var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0;queueMicrotask(function(){trace+='Q';});var saved,step=0;function C(ex){ex(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(f,r){trace+='T';eval('0');saved=r;};}};};var it={next:function(){trace+='N';return step++===0?{done:false,value:1}:{done:true};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var items={};items[Symbol.iterator]=function(){return it;};Promise.any.call(C,items);after=true;"###,
            ),
        ],
        "NRGT|0|0|0|0|false",
    );
}
