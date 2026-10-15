//! Additive allSettled host-terminal controls, frozen before their execution.
//! Existing race abort helper is exact; no in-operation allowance resets.
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
fn promise_all_settled_fatal_resolve_stops_before_later_author_callbacks() {
    fatal_stop(
        [
            (
                "fatal_resolve_resource",
                false,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';while(true){}return {get then(){trace+='G';return function(){trace+='T';};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.allSettled.call(C,src);after=true;"###,
            ),
            (
                "fatal_resolve_unsupported",
                true,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';eval('0');return {get then(){trace+='G';return function(){trace+='T';};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.allSettled.call(C,src);after=true;"###,
            ),
        ],
        "NR|0|0|0|0|false",
    );
}

#[test]
fn promise_all_settled_fatal_then_get_stops_before_later_author_callbacks() {
    fatal_stop(
        [
            (
                "fatal_then_get_resource",
                false,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';while(true){}return function(){trace+='T';};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.allSettled.call(C,src);after=true;"###,
            ),
            (
                "fatal_then_get_unsupported",
                true,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';eval('0');return function(){trace+='T';};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.allSettled.call(C,src);after=true;"###,
            ),
        ],
        "NRG|0|0|0|0|false",
    );
}

#[test]
fn promise_all_settled_fatal_then_call_stops_before_later_author_callbacks() {
    fatal_stop(
        [
            (
                "fatal_then_call_resource",
                false,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(){trace+='T';while(true){}};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.allSettled.call(C,src);after=true;"###,
            ),
            (
                "fatal_then_call_unsupported",
                true,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(){trace+='T';eval('0');};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.allSettled.call(C,src);after=true;"###,
            ),
        ],
        "NRGT|0|0|0|0|false",
    );
}

#[test]
fn promise_all_settled_fatal_aggregate_resolve_at_normal_exhaustion_stops_without_reject_or_jobs() {
    fatal_stop(
        [
            (
                "fatal_aggregate_exhaustion_resource",
                false,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={},savedF,savedR,step=0;function C(executor){executor(function(){resolves++;trace+='A';while(true){}},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(f,r){trace+='T';f(x);trace+='F';};}};};var it={next:function(){trace+='N';return step++===0?{done:false,value:1}:{done:true};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.allSettled.call(C,src);after=true;"###,
            ),
            (
                "fatal_aggregate_exhaustion_unsupported",
                true,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={},savedF,savedR,step=0;function C(executor){executor(function(){resolves++;trace+='A';eval('0');},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(f,r){trace+='T';f(x);trace+='F';};}};};var it={next:function(){trace+='N';return step++===0?{done:false,value:1}:{done:true};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.allSettled.call(C,src);after=true;"###,
            ),
        ],
        "NRGTFNA|1|0|0|0|false",
    );
}

#[test]
fn promise_all_settled_fatal_saved_final_callback_stops_without_reject_or_jobs() {
    fatal_stop(
        [
            (
                "fatal_saved_fulfill_resource",
                false,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={},savedF,savedR,step=0;function C(executor){executor(function(){resolves++;trace+='A';while(true){}},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(f,r){trace+='T';savedF=f;savedR=r;};}};};var it={next:function(){trace+='N';return step++===0?{done:false,value:1}:{done:true};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.allSettled.call(C,src);trace+='B';savedF(marker);after=true;"###,
            ),
            (
                "fatal_saved_fulfill_unsupported",
                true,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={},savedF,savedR,step=0;function C(executor){executor(function(){resolves++;trace+='A';eval('0');},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(f,r){trace+='T';savedF=f;savedR=r;};}};};var it={next:function(){trace+='N';return step++===0?{done:false,value:1}:{done:true};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.allSettled.call(C,src);trace+='B';savedF(marker);after=true;"###,
            ),
        ],
        "NRGTNBA|1|0|0|0|false",
    );
    fatal_stop(
        [
            (
                "fatal_saved_reject_resource",
                false,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={},savedF,savedR,step=0;function C(executor){executor(function(){resolves++;trace+='A';while(true){}},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(f,r){trace+='T';savedF=f;savedR=r;};}};};var it={next:function(){trace+='N';return step++===0?{done:false,value:1}:{done:true};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.allSettled.call(C,src);trace+='B';savedR(marker);after=true;"###,
            ),
            (
                "fatal_saved_reject_unsupported",
                true,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={},savedF,savedR,step=0;function C(executor){executor(function(){resolves++;trace+='A';eval('0');},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(f,r){trace+='T';savedF=f;savedR=r;};}};};var it={next:function(){trace+='N';return step++===0?{done:false,value:1}:{done:true};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.allSettled.call(C,src);trace+='B';savedR(marker);after=true;"###,
            ),
        ],
        "NRGTNBA|1|0|0|0|false",
    );
}
