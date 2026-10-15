//! Additive fatal/explicit-guard controls; public hold1 stays byte-exact.
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

fn clean_at(runtime: &Runtime, calls: usize, stack_units: usize) {
    assert_eq!(
        (
            runtime.frames.len(),
            runtime.eval_depth,
            runtime.calls,
            runtime.stack_units
        ),
        (0, 0, calls, stack_units)
    );
}

const FATAL_SETUP_PREFIX: &str = r###"
    var trace='',target=0,later=0,caught=0,finalized=0,returnGets=0,returnCalls=0,after=false;
    function sink(){target++;}function late(){later++;return 2;}
    var it={next:function(){trace+='N';
"###;
const FATAL_SETUP_SUFFIX: &str = r###"
    return {done:false,value:1};}};
    Object.defineProperty(it,'return',{get:function(){returnGets++;return function(){returnCalls++;return {};};}});
    var src={};src[Symbol.iterator]=function(){return it;};
"###;
const CALL: &str = r###"
    queueMicrotask(function(){trace+='Q';});
    try{sink(...src,late());}catch(e){caught++;}finally{finalized++;}
    after=true;
"###;
const CONSTRUCT: &str = r###"
    queueMicrotask(function(){trace+='Q';});
    try{new sink(...src,late());}catch(e){caught++;}finally{finalized++;}
    after=true;
"###;
const FATAL_OBSERVE: &str =
    "[trace,target,later,caught,finalized,returnGets,returnCalls,after].join('|')";

fn fatal_next(unsupported: bool, next_body: &str) {
    for strict in [false, true] {
        for (kind, invocation) in [("call", CALL), ("new", CONSTRUCT)] {
            let mut runtime = Runtime::try_new().unwrap();
            let mut document = Document::parse("");
            let setup = [FATAL_SETUP_PREFIX, next_body, FATAL_SETUP_SUFFIX].concat();
            execute_mode(&mut runtime, &mut document, strict, &setup).unwrap();
            clean_at(&runtime, 0, 0);
            let initial = execute_mode(&mut runtime, &mut document, strict, FATAL_OBSERVE).unwrap();
            assert_eq!(initial, Value::String("|0|0|0|0|0|0|false".into()));
            let error = execute_mode(&mut runtime, &mut document, strict, invocation).unwrap_err();
            assert_eq!(
                error.is_unsupported(),
                unsupported,
                "{kind} strict={strict}: {error:?}"
            );
            assert_eq!(
                error.is_resource_limit(),
                !unsupported,
                "{kind} strict={strict}: {error:?}"
            );
            clean_at(&runtime, 0, 0);
            assert!(runtime.microtasks_suspended());
            // Readback is a later synchronous entry, never a restart of queued work.
            for _ in 0..2 {
                let observed =
                    execute_mode(&mut runtime, &mut document, strict, FATAL_OBSERVE).unwrap();
                assert_eq!(
                    observed,
                    Value::String("N|0|0|0|0|0|0|false".into()),
                    "{kind} strict={strict}"
                );
                clean_at(&runtime, 0, 0);
                assert!(runtime.microtasks_suspended());
                assert!(runtime.console.is_empty());
            }
            // Suspension is owned by the failed Runtime, not a global switch.
            let mut fresh = Runtime::try_new().unwrap();
            execute_mode(
                &mut fresh,
                &mut document,
                strict,
                "var fresh='';queueMicrotask(function(){fresh+='Q';});fresh+='S';",
            )
            .unwrap();
            assert_eq!(
                execute_mode(&mut fresh, &mut document, strict, "fresh").unwrap(),
                Value::String("SQ".into())
            );
            clean_at(&fresh, 0, 0);
            assert!(!fresh.microtasks_suspended());
        }
    }
}

#[test]
fn call_spread_resource_failure_in_current_next_stops_all_later_author_work() {
    // No growing result list: the current callback spends the work allowance.
    // The queued sentinel is behind the still-running author entry and is unstarted.
    fatal_next(false, "while(true){}");
}

#[test]
fn call_spread_unsupported_current_next_stops_all_later_author_work() {
    fatal_next(true, "eval('0');");
}

#[test]
fn call_spread_existing_call_and_stack_guards_restore_incoming_ownership() {
    const SETUP: &str = r###"
        var method=0,next=0,target=0,later=0,caught=0,finalized=0,after=false;
        function sink(){target++;}function late(){later++;return 1;}
        var src={};src[Symbol.iterator]=function(){method++;return {next:function(){next++;return {done:true};}};};
    "###;
    const OBSERVE: &str = "[method,next,target,later,caught,finalized,after].join('|')";
    for strict in [false, true] {
        for (guard, calls, stack_units) in [("call", MAX_CALLS, 0), ("stack", 0, MAX_STACK_UNITS)] {
            for (kind, source) in [
                (
                    "call",
                    "try{sink(...src,late());}catch(e){caught++;}finally{finalized++;}after=true;",
                ),
                (
                    "new",
                    "try{new sink(...src,late());}catch(e){caught++;}finally{finalized++;}after=true;",
                ),
            ] {
                let mut runtime = Runtime::try_new().unwrap();
                let mut document = Document::parse("");
                execute_mode(&mut runtime, &mut document, strict, SETUP).unwrap();
                assert_eq!(
                    execute_mode(&mut runtime, &mut document, strict, OBSERVE).unwrap(),
                    Value::String("0|0|0|0|0|0|false".into())
                );
                // Existing guard fields model a caller whose ownership is external.
                runtime.calls = calls;
                runtime.stack_units = stack_units;
                let error = execute_mode(&mut runtime, &mut document, strict, source).unwrap_err();
                assert!(
                    error.is_resource_limit(),
                    "{guard}/{kind} strict={strict}: {error:?}"
                );
                clean_at(&runtime, calls, stack_units);
                assert!(runtime.microtasks_suspended());
                // Release only the fixture's incoming guard before independent readback.
                runtime.calls = 0;
                runtime.stack_units = 0;
                assert_eq!(
                    execute_mode(&mut runtime, &mut document, strict, OBSERVE).unwrap(),
                    Value::String("0|0|0|0|0|0|false".into()),
                    "{guard}/{kind} strict={strict}"
                );
                clean_at(&runtime, 0, 0);
                assert!(runtime.microtasks_suspended());
                assert!(runtime.console.is_empty());
            }
        }
    }
}
