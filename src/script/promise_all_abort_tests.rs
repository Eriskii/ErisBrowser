//! Additive Promise.all abort controls; the earlier24-case module is unchanged.
//! Existing Runtime APIs only; no private fees or fresh budget inside an operation.
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

#[test]
fn promise_all_ordinary_abort_preserves_sentinel_for_saved_fulfillment_callbacks() {
    for (case_id, source, expected) in [
        (
            "ordinary_abort_next",
            r###"var stage='next',saved=[],aggregate=0,rejects=0,seen,closes=0,step=0,marker={},returned={};function C(executor){executor(function(v){aggregate++;return returned;},function(e){rejects++;seen=e;});}C.resolve=function(x){if(stage==='resolve'&&x===2)throw marker;return {then:function(f){saved.push(f);if(stage==='then'&&x===2)throw marker;}};};var it={next:function(){step++;if(stage==='next'&&step===2)throw marker;return step<=2?{done:false,value:step}:{done:true};},return:function(){closes++;return {};}};var src={};src[Symbol.iterator]=function(){return it;};var result=Promise.all.call(C,src);var undefineds=true;for(var i=0;i<saved.length;i++){undefineds=undefineds&&saved[i](i+7)===undefined;}var duplicate=saved[0](99);"###,
            "0|1|true|0|1|true|true|true",
        ),
        (
            "ordinary_abort_resolve",
            r###"var stage='resolve',saved=[],aggregate=0,rejects=0,seen,closes=0,step=0,marker={},returned={};function C(executor){executor(function(v){aggregate++;return returned;},function(e){rejects++;seen=e;});}C.resolve=function(x){if(stage==='resolve'&&x===2)throw marker;return {then:function(f){saved.push(f);if(stage==='then'&&x===2)throw marker;}};};var it={next:function(){step++;if(stage==='next'&&step===2)throw marker;return step<=2?{done:false,value:step}:{done:true};},return:function(){closes++;return {};}};var src={};src[Symbol.iterator]=function(){return it;};var result=Promise.all.call(C,src);var undefineds=true;for(var i=0;i<saved.length;i++){undefineds=undefineds&&saved[i](i+7)===undefined;}var duplicate=saved[0](99);"###,
            "0|1|true|1|1|true|true|true",
        ),
        (
            "ordinary_abort_then",
            r###"var stage='then',saved=[],aggregate=0,rejects=0,seen,closes=0,step=0,marker={},returned={};function C(executor){executor(function(v){aggregate++;return returned;},function(e){rejects++;seen=e;});}C.resolve=function(x){if(stage==='resolve'&&x===2)throw marker;return {then:function(f){saved.push(f);if(stage==='then'&&x===2)throw marker;}};};var it={next:function(){step++;if(stage==='next'&&step===2)throw marker;return step<=2?{done:false,value:step}:{done:true};},return:function(){closes++;return {};}};var src={};src[Symbol.iterator]=function(){return it;};var result=Promise.all.call(C,src);var undefineds=true;for(var i=0;i<saved.length;i++){undefineds=undefineds&&saved[i](i+7)===undefined;}var duplicate=saved[0](99);"###,
            "0|1|true|1|2|true|true|true",
        ),
    ] {
        for strict in [false, true] {
            let mut runtime = Runtime::try_new().unwrap();
            let mut document = Document::parse("");
            execute_mode(&mut runtime, &mut document, strict, source)
                .unwrap_or_else(|error| panic!("{case_id} strict={strict}: {error:?}"));
            let value = execute_mode(
                &mut runtime,
                &mut document,
                strict,
                r###"[aggregate,rejects,seen===marker,closes,saved.length,undefineds,duplicate===undefined,result instanceof C].join('|')"###,
            ).unwrap();
            assert_eq!(
                value,
                Value::String(expected.into()),
                "{case_id} strict={strict}"
            );
            assert!(!runtime.microtasks_suspended());
            assert!(runtime.console.is_empty());
        }
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
fn promise_all_fatal_next_stops_before_later_author_callbacks() {
    fatal_stop(
        [
            (
                "fatal_next_resource",
                false,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(){trace+='T';};}};};var it={next:function(){trace+='N';while(true){}return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.all.call(C,src);after=true;"###,
            ),
            (
                "fatal_next_unsupported",
                true,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(){trace+='T';};}};};var it={next:function(){trace+='N';eval('0');return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.all.call(C,src);after=true;"###,
            ),
        ],
        "N|0|0|0|0|false",
    );
}

#[test]
fn promise_all_fatal_resolve_stops_before_later_author_callbacks() {
    fatal_stop(
        [
            (
                "fatal_resolve_resource",
                false,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';while(true){}return {get then(){trace+='G';return function(){trace+='T';};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.all.call(C,src);after=true;"###,
            ),
            (
                "fatal_resolve_unsupported",
                true,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';eval('0');return {get then(){trace+='G';return function(){trace+='T';};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.all.call(C,src);after=true;"###,
            ),
        ],
        "NR|0|0|0|0|false",
    );
}

#[test]
fn promise_all_fatal_then_get_stops_before_later_author_callbacks() {
    fatal_stop(
        [
            (
                "fatal_then_get_resource",
                false,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';while(true){}return function(){trace+='T';};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.all.call(C,src);after=true;"###,
            ),
            (
                "fatal_then_get_unsupported",
                true,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';eval('0');return function(){trace+='T';};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.all.call(C,src);after=true;"###,
            ),
        ],
        "NRG|0|0|0|0|false",
    );
}

#[test]
fn promise_all_fatal_then_call_stops_before_later_author_callbacks() {
    fatal_stop(
        [
            (
                "fatal_then_call_resource",
                false,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(){trace+='T';while(true){}};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.all.call(C,src);after=true;"###,
            ),
            (
                "fatal_then_call_unsupported",
                true,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(){trace+='T';eval('0');};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.all.call(C,src);after=true;"###,
            ),
        ],
        "NRGT|0|0|0|0|false",
    );
}

#[test]
fn promise_all_fatal_return_get_stops_before_later_author_callbacks() {
    fatal_stop(
        [
            (
                "fatal_return_get_resource",
                false,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(){trace+='T';throw marker;};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';while(true){}return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.all.call(C,src);after=true;"###,
            ),
            (
                "fatal_return_get_unsupported",
                true,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(){trace+='T';throw marker;};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';eval('0');return function(){returnCalls++;trace+='X';return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.all.call(C,src);after=true;"###,
            ),
        ],
        "NRGTC|0|0|1|0|false",
    );
}

#[test]
fn promise_all_fatal_return_call_stops_before_later_author_callbacks() {
    fatal_stop(
        [
            (
                "fatal_return_call_resource",
                false,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(){trace+='T';throw marker;};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';while(true){}return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.all.call(C,src);after=true;"###,
            ),
            (
                "fatal_return_call_unsupported",
                true,
                r###"var trace='',after=false,resolves=0,rejects=0,returnGets=0,returnCalls=0,marker={};function C(executor){executor(function(){resolves++;trace+='A';},function(){rejects++;trace+='J';});}C.resolve=function(x){trace+='R';return {get then(){trace+='G';return function(){trace+='T';throw marker;};}};};var it={next:function(){trace+='N';return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='C';return function(){returnCalls++;trace+='X';eval('0');return {};};}});var src={};src[Symbol.iterator]=function(){return it;};queueMicrotask(function(){trace+='Q';});Promise.all.call(C,src);after=true;"###,
            ),
        ],
        "NRGTCX|0|0|1|1|false",
    );
}
