//! Natural class work exhaustion; retained-state inspection never re-enters JS.
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

fn retained<'a>(runtime: &'a Runtime, name: &str) -> &'a Value {
    &runtime.environments[0].bindings[name].value
}

fn terminal(cases: &[(&str, &str, &str, f64)]) {
    for &(case_id, source, expected_trace, expected_constructors) in cases {
        for strict in [false, true] {
            let mut runtime = Runtime::try_new().unwrap();
            let mut document = Document::parse("");
            let error = execute_mode(&mut runtime, &mut document, strict, source)
                .expect_err("infinite author loop unexpectedly completed");
            // Parent syntax refusal must fail here, before any retained-state
            // assertions. The loop must reach the natural work-limit path.
            assert!(
                error.is_resource_limit(),
                "{case_id} strict={strict}: {error:?}"
            );
            assert!(
                !error.is_unsupported(),
                "{case_id} strict={strict}: {error:?}"
            );
            assert_eq!(
                error.message, "script instruction limit exceeded",
                "{case_id} strict={strict}"
            );
            assert_eq!(runtime.steps, 0, "{case_id} strict={strict}");
            assert!(runtime.microtasks_suspended(), "{case_id} strict={strict}");
            assert!(runtime.console.is_empty(), "{case_id} strict={strict}");
            assert_eq!(
                (
                    runtime.frames.len(),
                    runtime.calls,
                    runtime.eval_depth,
                    runtime.stack_units
                ),
                (0, 0, 0, 0),
                "{case_id} strict={strict}"
            );
            assert_eq!(
                runtime.microtasks.author_depth, 0,
                "{case_id} strict={strict}"
            );
            assert!(
                !runtime.microtasks.performing_checkpoint,
                "{case_id} strict={strict}"
            );
            // Only borrowed reads follow the terminal public entry. No script
            // readback, checkpoint, callback call, counter or allowance reset.
            assert_eq!(
                retained(&runtime, "trace"),
                &Value::String(expected_trace.into()),
                "{case_id} strict={strict}"
            );
            assert_eq!(
                retained(&runtime, "entered"),
                &Value::Number(1.0),
                "{case_id} strict={strict}"
            );
            assert_eq!(
                retained(&runtime, "constructors"),
                &Value::Number(expected_constructors),
                "{case_id} strict={strict}"
            );
            for name in ["base", "body", "caught", "finalized", "queuedRan"] {
                assert_eq!(
                    retained(&runtime, name),
                    &Value::Number(0.0),
                    "{case_id} strict={strict} {name}"
                );
            }
            assert_eq!(
                retained(&runtime, "after"),
                &Value::Bool(false),
                "{case_id} strict={strict}"
            );
            assert_eq!(
                runtime.microtasks.queue.len(),
                1,
                "{case_id} strict={strict}"
            );
            let Some(promises::QueueJob::Callback(callback)) = runtime.microtasks.queue.front()
            else {
                panic!("{case_id} strict={strict}: saved sentinel missing");
            };
            assert_eq!(
                callback,
                retained(&runtime, "queued"),
                "{case_id} strict={strict}"
            );
        }
    }
}

#[test]
fn class_public_terminal_heritage() {
    terminal(&[(
        "heritage",
        r###"var trace='',entered=0,constructors=0,base=0,body=0,caught=0,finalized=0,queuedRan=0,after=false;
var queued=()=>{queuedRan++;trace+='Q';console.log('late');};
queueMicrotask(queued);
function exhaust(mark){trace+=mark;entered++;while(true){}}
try{
 class C extends exhaust('H') {constructor(){constructors++;super();body++;}static [(trace+='L','later')](){}}
 after=true;
}catch(e){caught++;trace+='X';}finally{finalized++;trace+='F';}
after=true;trace+='Z';
"###,
        "H",
        0.0,
    )]);
}

#[test]
fn class_public_terminal_computed_keys() {
    terminal(&[
        (
            "computed_instance",
            r###"var trace='',entered=0,constructors=0,base=0,body=0,caught=0,finalized=0,queuedRan=0,after=false;
var queued=()=>{queuedRan++;trace+='Q';console.log('late');};
queueMicrotask(queued);
function exhaust(mark){trace+=mark;entered++;while(true){}}
try{
 class C {[(trace+='P','first')](){}[exhaust('K')](){}[(trace+='L','later')](){}constructor(){constructors++;body++;}}
 after=true;
}catch(e){caught++;trace+='X';}finally{finalized++;trace+='F';}
after=true;trace+='Z';
"###,
            "PK",
            0.0,
        ),
        (
            "computed_static",
            r###"var trace='',entered=0,constructors=0,base=0,body=0,caught=0,finalized=0,queuedRan=0,after=false;
var queued=()=>{queuedRan++;trace+='Q';console.log('late');};
queueMicrotask(queued);
function exhaust(mark){trace+=mark;entered++;while(true){}}
try{
 class C {static [(trace+='P','first')](){}static [exhaust('S')](){}static [(trace+='L','later')](){}constructor(){constructors++;body++;}}
 after=true;
}catch(e){caught++;trace+='X';}finally{finalized++;trace+='F';}
after=true;trace+='Z';
"###,
            "PS",
            0.0,
        ),
    ]);
}

#[test]
fn class_public_terminal_parameter_default() {
    terminal(&[(
        "derived_parameter_default",
        r###"var trace='',entered=0,constructors=0,base=0,body=0,caught=0,finalized=0,queuedRan=0,after=false;
var queued=()=>{queuedRan++;trace+='Q';console.log('late');};
queueMicrotask(queued);
function exhaust(mark){trace+=mark;entered++;while(true){}}
function B(){base++;trace+='B';}
class D extends B {constructor(x=exhaust('P')){constructors++;trace+='C';super();body++;trace+='D';}}
try{new D();after=true;}catch(e){caught++;trace+='X';}finally{finalized++;trace+='F';}
after=true;trace+='Z';
"###,
        "P",
        0.0,
    )]);
}

#[test]
fn class_public_terminal_super_argument() {
    terminal(&[(
        "explicit_super_argument",
        r###"var trace='',entered=0,constructors=0,base=0,body=0,caught=0,finalized=0,queuedRan=0,after=false;
var queued=()=>{queuedRan++;trace+='Q';console.log('late');};
queueMicrotask(queued);
function exhaust(mark){trace+=mark;entered++;while(true){}}
function B(){base++;trace+='B';}
class D extends B {constructor(){constructors++;trace+='C';super(exhaust('A'));body++;trace+='D';}}
try{new D();after=true;}catch(e){caught++;trace+='X';}finally{finalized++;trace+='F';}
after=true;trace+='Z';
"###,
        "CA",
        1.0,
    )]);
}

#[test]
fn class_public_terminal_native_promise_executor() {
    terminal(&[
        (
            "promise_default_derived_executor",
            r###"var trace='',entered=0,constructors=0,base=0,body=0,caught=0,finalized=0,queuedRan=0,after=false;
var queued=()=>{queuedRan++;trace+='Q';console.log('late');};
queueMicrotask(queued);
function exhaust(mark){trace+=mark;entered++;while(true){}}
function executor(resolve,reject){
 if(typeof resolve!=='function'||typeof reject!=='function')throw new Error('executor callbacks');
 try{exhaust('E');body++;trace+='e';}finally{finalized++;trace+='f';}
}
class P extends Promise {}
try{new P(executor);after=true;}catch(e){caught++;trace+='X';}finally{finalized++;trace+='F';}
after=true;trace+='Z';
"###,
            "E",
            0.0,
        ),
        (
            "promise_explicit_derived_executor",
            r###"var trace='',entered=0,constructors=0,base=0,body=0,caught=0,finalized=0,queuedRan=0,after=false;
var queued=()=>{queuedRan++;trace+='Q';console.log('late');};
queueMicrotask(queued);
function exhaust(mark){trace+=mark;entered++;while(true){}}
function executor(resolve,reject){
 if(typeof resolve!=='function'||typeof reject!=='function')throw new Error('executor callbacks');
 try{exhaust('E');body++;trace+='e';}finally{finalized++;trace+='f';}
}
class P extends Promise {constructor(executor){constructors++;trace+='C';super(executor);body++;trace+='D';}}
try{new P(executor);after=true;}catch(e){caught++;trace+='X';}finally{finalized++;trace+='F';}
after=true;trace+='Z';
"###,
            "CE",
            1.0,
        ),
    ]);
}
