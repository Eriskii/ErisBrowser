//! Promise.race terminal boundaries using the existing Promise.all abort policy.
//! Existing APIs only; no in-operation budget reset or iteration-count oracle.
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
fn promise_race_fatal_acquisition_stops_before_later_author_callbacks() {
    fatal_stop(
        [
            (
                "fatal_acquisition_resource",
                false,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(){trace+='T';};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};Object.defineProperty(src,Symbol.iterator,{get:function(){trace+='I';while(true){}return function(){trace+='M';return it;};}});queueMicrotask(function(){trace+='Q';});Promise.race.call(C,src);after=true;"###,
            ),
            (
                "fatal_acquisition_unsupported",
                true,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(){trace+='T';};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};Object.defineProperty(src,Symbol.iterator,{get:function(){trace+='I';eval('0');return function(){trace+='M';return it;};}});queueMicrotask(function(){trace+='Q';});Promise.race.call(C,src);after=true;"###,
            ),
        ],
        "I|0|0|0|0|false",
    );
}

#[test]
fn promise_race_fatal_next_stops_before_later_author_callbacks() {
    fatal_stop(
        [
            (
                "fatal_next_resource",
                false,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(){trace+='T';};}};};var it={next:function(){trace+='N';while(true){}return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.race.call(C,src);after=true;"###,
            ),
            (
                "fatal_next_unsupported",
                true,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(){trace+='T';};}};};var it={next:function(){trace+='N';eval('0');return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.race.call(C,src);after=true;"###,
            ),
        ],
        "N|0|0|0|0|false",
    );
}

#[test]
fn promise_race_fatal_resolve_stops_before_later_author_callbacks() {
    fatal_stop(
        [
            (
                "fatal_resolve_resource",
                false,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';while(true){}return {get then(){trace+='G';return function(){trace+='T';};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.race.call(C,src);after=true;"###,
            ),
            (
                "fatal_resolve_unsupported",
                true,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';eval('0');return {get then(){trace+='G';return function(){trace+='T';};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.race.call(C,src);after=true;"###,
            ),
        ],
        "NR|0|0|0|0|false",
    );
}

#[test]
fn promise_race_fatal_then_call_stops_before_later_author_callbacks() {
    fatal_stop(
        [
            (
                "fatal_then_call_resource",
                false,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(){trace+='T';while(true){}};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.race.call(C,src);after=true;"###,
            ),
            (
                "fatal_then_call_unsupported",
                true,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(){trace+='T';eval('0');};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.race.call(C,src);after=true;"###,
            ),
        ],
        "NRGT|0|0|0|0|false",
    );
}

#[test]
fn promise_race_fatal_return_get_stops_before_later_author_callbacks() {
    fatal_stop(
        [
            (
                "fatal_return_get_resource",
                false,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(){trace+='T';throw marker;};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';while(true){}return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.race.call(C,src);after=true;"###,
            ),
            (
                "fatal_return_get_unsupported",
                true,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(){trace+='T';throw marker;};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';eval('0');return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.race.call(C,src);after=true;"###,
            ),
        ],
        "NRGTC|0|0|1|0|false",
    );
}

#[test]
fn promise_race_fatal_return_call_stops_before_later_author_callbacks() {
    fatal_stop(
        [
            (
                "fatal_return_call_resource",
                false,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(){trace+='T';throw marker;};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';while(true){}return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.race.call(C,src);after=true;"###,
            ),
            (
                "fatal_return_call_unsupported",
                true,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(){trace+='T';throw marker;};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';eval('0');return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.race.call(C,src);after=true;"###,
            ),
        ],
        "NRGTCX|0|0|1|1|false",
    );
}
