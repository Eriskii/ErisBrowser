//! AggregateError host-terminal controls with frozen source literals.
//! No manual allowance, queue or suspension resets.
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
                    r###"[trace,returnGets,returnCalls,caught,finalized,after].join('|')"###,
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
fn aggregate_error_terminal_prototype_stops_author_code_and_jobs() {
    fatal_stop(
        [
            (
                "aggregate_terminal_prototype_work",
                false,
                r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
var trace='',returnGets=0,returnCalls=0,caught=0,finalized=0,after=false,marker={},proto={},step=0;
var N=(function(){}).bind(null);Object.defineProperty(N,'prototype',{get:function(){trace+='P';while(true){}return proto;}});
var msg={toString:function(){trace+='M';return 'm';}};
var options={get cause(){trace+='C';return marker;}};
var it={};Object.defineProperty(it,'next',{get:function(){trace+='N';return function(){trace+='S';if(step++===0)return {get done(){trace+='D';return false;},get value(){trace+='V';return marker;}};return {done:true};};}});
Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='R';return function(){returnCalls++;trace+='r';return {};};}});
var items={};Object.defineProperty(items,Symbol.iterator,{get:function(){trace+='I';return function(){trace+='i';return it;};}});
queueMicrotask(function(){trace+='Q';});
try{Reflect.construct(AggregateError,[items,msg,options],N);after=true;}catch(e){caught++;trace+='K';}finally{finalized++;trace+='F';}
after=true;
"###,
            ),
            (
                "aggregate_terminal_prototype_unsupported",
                true,
                r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
var trace='',returnGets=0,returnCalls=0,caught=0,finalized=0,after=false,marker={},proto={},step=0;
var N=(function(){}).bind(null);Object.defineProperty(N,'prototype',{get:function(){trace+='P';eval('0');return proto;}});
var msg={toString:function(){trace+='M';return 'm';}};
var options={get cause(){trace+='C';return marker;}};
var it={};Object.defineProperty(it,'next',{get:function(){trace+='N';return function(){trace+='S';if(step++===0)return {get done(){trace+='D';return false;},get value(){trace+='V';return marker;}};return {done:true};};}});
Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='R';return function(){returnCalls++;trace+='r';return {};};}});
var items={};Object.defineProperty(items,Symbol.iterator,{get:function(){trace+='I';return function(){trace+='i';return it;};}});
queueMicrotask(function(){trace+='Q';});
try{Reflect.construct(AggregateError,[items,msg,options],N);after=true;}catch(e){caught++;trace+='K';}finally{finalized++;trace+='F';}
after=true;
"###,
            ),
        ],
        "P|0|0|0|0|false",
    );
}

#[test]
fn aggregate_error_terminal_message_stops_author_code_and_jobs() {
    fatal_stop(
        [
            (
                "aggregate_terminal_message_work",
                false,
                r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
var trace='',returnGets=0,returnCalls=0,caught=0,finalized=0,after=false,marker={},proto={},step=0;
var N=(function(){}).bind(null);Object.defineProperty(N,'prototype',{get:function(){trace+='P';return proto;}});
var msg={toString:function(){trace+='M';while(true){}return 'm';}};
var options={get cause(){trace+='C';return marker;}};
var it={};Object.defineProperty(it,'next',{get:function(){trace+='N';return function(){trace+='S';if(step++===0)return {get done(){trace+='D';return false;},get value(){trace+='V';return marker;}};return {done:true};};}});
Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='R';return function(){returnCalls++;trace+='r';return {};};}});
var items={};Object.defineProperty(items,Symbol.iterator,{get:function(){trace+='I';return function(){trace+='i';return it;};}});
queueMicrotask(function(){trace+='Q';});
try{Reflect.construct(AggregateError,[items,msg,options],N);after=true;}catch(e){caught++;trace+='K';}finally{finalized++;trace+='F';}
after=true;
"###,
            ),
            (
                "aggregate_terminal_message_unsupported",
                true,
                r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
var trace='',returnGets=0,returnCalls=0,caught=0,finalized=0,after=false,marker={},proto={},step=0;
var N=(function(){}).bind(null);Object.defineProperty(N,'prototype',{get:function(){trace+='P';return proto;}});
var msg={toString:function(){trace+='M';eval('0');return 'm';}};
var options={get cause(){trace+='C';return marker;}};
var it={};Object.defineProperty(it,'next',{get:function(){trace+='N';return function(){trace+='S';if(step++===0)return {get done(){trace+='D';return false;},get value(){trace+='V';return marker;}};return {done:true};};}});
Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='R';return function(){returnCalls++;trace+='r';return {};};}});
var items={};Object.defineProperty(items,Symbol.iterator,{get:function(){trace+='I';return function(){trace+='i';return it;};}});
queueMicrotask(function(){trace+='Q';});
try{Reflect.construct(AggregateError,[items,msg,options],N);after=true;}catch(e){caught++;trace+='K';}finally{finalized++;trace+='F';}
after=true;
"###,
            ),
        ],
        "PM|0|0|0|0|false",
    );
}

#[test]
fn aggregate_error_terminal_cause_stops_author_code_and_jobs() {
    fatal_stop(
        [
            (
                "aggregate_terminal_cause_work",
                false,
                r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
var trace='',returnGets=0,returnCalls=0,caught=0,finalized=0,after=false,marker={},proto={},step=0;
var N=(function(){}).bind(null);Object.defineProperty(N,'prototype',{get:function(){trace+='P';return proto;}});
var msg={toString:function(){trace+='M';return 'm';}};
var options={get cause(){trace+='C';while(true){}return marker;}};
var it={};Object.defineProperty(it,'next',{get:function(){trace+='N';return function(){trace+='S';if(step++===0)return {get done(){trace+='D';return false;},get value(){trace+='V';return marker;}};return {done:true};};}});
Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='R';return function(){returnCalls++;trace+='r';return {};};}});
var items={};Object.defineProperty(items,Symbol.iterator,{get:function(){trace+='I';return function(){trace+='i';return it;};}});
queueMicrotask(function(){trace+='Q';});
try{Reflect.construct(AggregateError,[items,msg,options],N);after=true;}catch(e){caught++;trace+='K';}finally{finalized++;trace+='F';}
after=true;
"###,
            ),
            (
                "aggregate_terminal_cause_unsupported",
                true,
                r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
var trace='',returnGets=0,returnCalls=0,caught=0,finalized=0,after=false,marker={},proto={},step=0;
var N=(function(){}).bind(null);Object.defineProperty(N,'prototype',{get:function(){trace+='P';return proto;}});
var msg={toString:function(){trace+='M';return 'm';}};
var options={get cause(){trace+='C';eval('0');return marker;}};
var it={};Object.defineProperty(it,'next',{get:function(){trace+='N';return function(){trace+='S';if(step++===0)return {get done(){trace+='D';return false;},get value(){trace+='V';return marker;}};return {done:true};};}});
Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='R';return function(){returnCalls++;trace+='r';return {};};}});
var items={};Object.defineProperty(items,Symbol.iterator,{get:function(){trace+='I';return function(){trace+='i';return it;};}});
queueMicrotask(function(){trace+='Q';});
try{Reflect.construct(AggregateError,[items,msg,options],N);after=true;}catch(e){caught++;trace+='K';}finally{finalized++;trace+='F';}
after=true;
"###,
            ),
        ],
        "PMC|0|0|0|0|false",
    );
}

#[test]
fn aggregate_error_terminal_iterator_acquisition_stops_author_code_and_jobs() {
    fatal_stop(
        [
            (
                "aggregate_terminal_iterator_acquisition_work",
                false,
                r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
var trace='',returnGets=0,returnCalls=0,caught=0,finalized=0,after=false,marker={},proto={},step=0;
var N=(function(){}).bind(null);Object.defineProperty(N,'prototype',{get:function(){trace+='P';return proto;}});
var msg={toString:function(){trace+='M';return 'm';}};
var options={get cause(){trace+='C';return marker;}};
var it={};Object.defineProperty(it,'next',{get:function(){trace+='N';return function(){trace+='S';if(step++===0)return {get done(){trace+='D';return false;},get value(){trace+='V';return marker;}};return {done:true};};}});
Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='R';return function(){returnCalls++;trace+='r';return {};};}});
var items={};Object.defineProperty(items,Symbol.iterator,{get:function(){trace+='I';while(true){}return function(){trace+='i';return it;};}});
queueMicrotask(function(){trace+='Q';});
try{Reflect.construct(AggregateError,[items,msg,options],N);after=true;}catch(e){caught++;trace+='K';}finally{finalized++;trace+='F';}
after=true;
"###,
            ),
            (
                "aggregate_terminal_iterator_acquisition_unsupported",
                true,
                r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
var trace='',returnGets=0,returnCalls=0,caught=0,finalized=0,after=false,marker={},proto={},step=0;
var N=(function(){}).bind(null);Object.defineProperty(N,'prototype',{get:function(){trace+='P';return proto;}});
var msg={toString:function(){trace+='M';return 'm';}};
var options={get cause(){trace+='C';return marker;}};
var it={};Object.defineProperty(it,'next',{get:function(){trace+='N';return function(){trace+='S';if(step++===0)return {get done(){trace+='D';return false;},get value(){trace+='V';return marker;}};return {done:true};};}});
Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='R';return function(){returnCalls++;trace+='r';return {};};}});
var items={};Object.defineProperty(items,Symbol.iterator,{get:function(){trace+='I';eval('0');return function(){trace+='i';return it;};}});
queueMicrotask(function(){trace+='Q';});
try{Reflect.construct(AggregateError,[items,msg,options],N);after=true;}catch(e){caught++;trace+='K';}finally{finalized++;trace+='F';}
after=true;
"###,
            ),
        ],
        "PMCI|0|0|0|0|false",
    );
}

#[test]
fn aggregate_error_terminal_iterator_value_stops_author_code_and_jobs() {
    fatal_stop(
        [
            (
                "aggregate_terminal_iterator_value_work",
                false,
                r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
var trace='',returnGets=0,returnCalls=0,caught=0,finalized=0,after=false,marker={},proto={},step=0;
var N=(function(){}).bind(null);Object.defineProperty(N,'prototype',{get:function(){trace+='P';return proto;}});
var msg={toString:function(){trace+='M';return 'm';}};
var options={get cause(){trace+='C';return marker;}};
var it={};Object.defineProperty(it,'next',{get:function(){trace+='N';return function(){trace+='S';if(step++===0)return {get done(){trace+='D';return false;},get value(){trace+='V';while(true){}return marker;}};return {done:true};};}});
Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='R';return function(){returnCalls++;trace+='r';return {};};}});
var items={};Object.defineProperty(items,Symbol.iterator,{get:function(){trace+='I';return function(){trace+='i';return it;};}});
queueMicrotask(function(){trace+='Q';});
try{Reflect.construct(AggregateError,[items,msg,options],N);after=true;}catch(e){caught++;trace+='K';}finally{finalized++;trace+='F';}
after=true;
"###,
            ),
            (
                "aggregate_terminal_iterator_value_unsupported",
                true,
                r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
var trace='',returnGets=0,returnCalls=0,caught=0,finalized=0,after=false,marker={},proto={},step=0;
var N=(function(){}).bind(null);Object.defineProperty(N,'prototype',{get:function(){trace+='P';return proto;}});
var msg={toString:function(){trace+='M';return 'm';}};
var options={get cause(){trace+='C';return marker;}};
var it={};Object.defineProperty(it,'next',{get:function(){trace+='N';return function(){trace+='S';if(step++===0)return {get done(){trace+='D';return false;},get value(){trace+='V';eval('0');return marker;}};return {done:true};};}});
Object.defineProperty(it,'return',{get:function(){returnGets++;trace+='R';return function(){returnCalls++;trace+='r';return {};};}});
var items={};Object.defineProperty(items,Symbol.iterator,{get:function(){trace+='I';return function(){trace+='i';return it;};}});
queueMicrotask(function(){trace+='Q';});
try{Reflect.construct(AggregateError,[items,msg,options],N);after=true;}catch(e){caught++;trace+='K';}finally{finalized++;trace+='F';}
after=true;
"###,
            ),
        ],
        "PMCIiNSDV|0|0|0|0|false",
    );
}
