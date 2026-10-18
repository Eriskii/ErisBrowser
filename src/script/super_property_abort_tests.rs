//! Independent public SuperProperty literals, frozen before implementation.
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
    &runtime.environments[0]
        .bindings
        .get(name)
        .unwrap_or_else(|| panic!("retained global {name} was never initialized"))
        .value
}

fn fatal(case_id: &str, source: &str, expected_trace: &str, unsupported: bool) {
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        let error = execute_mode(&mut runtime, &mut document, strict, source)
            .expect_err("fatal source completed");
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
        if !unsupported {
            assert_eq!(error.message, "script instruction limit exceeded");
            assert_eq!(runtime.steps, 0);
        }
        assert!(runtime.microtasks_suspended(), "{case_id} strict={strict}");
        assert!(runtime.console.is_empty());
        assert_eq!(
            (
                runtime.frames.len(),
                runtime.calls,
                runtime.eval_depth,
                runtime.stack_units
            ),
            (0, 0, 0, 0)
        );
        assert_eq!(runtime.microtasks.author_depth, 0);
        assert!(!runtime.microtasks.performing_checkpoint);
        // Borrowed reads only: no second execute, checkpoint or allowance reset.
        assert_eq!(
            retained(&runtime, "trace"),
            &Value::String(expected_trace.into()),
            "{case_id} strict={strict}"
        );
        for name in ["caught", "finalized", "after", "queuedRan"] {
            assert_eq!(
                retained(&runtime, name),
                &Value::Number(0.0),
                "{case_id} strict={strict} {name}"
            );
        }
        assert_eq!(runtime.microtasks.queue.len(), 1);
        let Some(promises::QueueJob::Callback(callback)) = runtime.microtasks.queue.front() else {
            panic!("{case_id} strict={strict}: saved queued callback missing");
        };
        assert_eq!(callback, retained(&runtime, "queued"));
    }
}

#[test]
fn super_property_terminal_computed_key_expression() {
    fatal(
        r####"terminal_computed_key_expression"####,
        r####"var trace='',caught=0,finalized=0,after=0,queuedRan=0;
var queued=function(){queuedRan++;};queueMicrotask(queued);
function spin(){while(true){}}
class A { get x(){trace+='G';return 1;} }class C extends A { run(){return super[(trace+='K',spin())];} }
try{new C().run();after++;}catch(e){caught++;}finally{finalized++;}after++;
"####,
        r####"K"####,
        false,
    );
}

#[test]
fn super_property_terminal_key_conversion() {
    fatal(
        r####"terminal_key_conversion"####,
        r####"var trace='',caught=0,finalized=0,after=0,queuedRan=0;
var queued=function(){queuedRan++;};queueMicrotask(queued);
function spin(){while(true){}}
var key={};key[Symbol.toPrimitive]=function(){trace+='C';spin();return 'x';};
class A { get x(){trace+='G';return 1;} }class C extends A { run(){return super[(trace+='K',key)];} }
try{new C().run();after++;}catch(e){caught++;}finally{finalized++;}after++;
"####,
        r####"KC"####,
        false,
    );
}

#[test]
fn super_property_terminal_getter() {
    fatal(
        r####"terminal_getter"####,
        r####"var trace='',caught=0,finalized=0,after=0,queuedRan=0;
var queued=function(){queuedRan++;};queueMicrotask(queued);
function spin(){while(true){}}
var key={};key[Symbol.toPrimitive]=function(){trace+='C';return 'x';};
class A { get x(){trace+='G';spin();return 1;} }class C extends A { run(){return super[(trace+='K',key)];} }
try{new C().run();after++;}catch(e){caught++;}finally{finalized++;}after++;
"####,
        r####"KCG"####,
        false,
    );
}

#[test]
fn super_property_terminal_assignment_rhs_before_conversion() {
    fatal(
        r####"terminal_assignment_rhs_before_conversion"####,
        r####"var trace='',caught=0,finalized=0,after=0,queuedRan=0;
var queued=function(){queuedRan++;};queueMicrotask(queued);
function spin(){while(true){}}
var key={};key[Symbol.toPrimitive]=function(){trace+='C';return 'x';};
class A { set x(v){trace+='S';} }class C extends A { run(){super[(trace+='K',key)]=(trace+='R',spin());} }
try{new C().run();after++;}catch(e){caught++;}finally{finalized++;}after++;
"####,
        r####"KR"####,
        false,
    );
}

#[test]
fn super_property_terminal_setter() {
    fatal(
        r####"terminal_setter"####,
        r####"var trace='',caught=0,finalized=0,after=0,queuedRan=0;
var queued=function(){queuedRan++;};queueMicrotask(queued);
function spin(){while(true){}}
var key={};key[Symbol.toPrimitive]=function(){trace+='C';return 'x';};
class A { set x(v){trace+='S';spin();} }class C extends A { run(){super[(trace+='K',key)]=(trace+='R',1);trace+='B';} }
try{new C().run();after++;}catch(e){caught++;}finally{finalized++;}after++;
"####,
        r####"KRCS"####,
        false,
    );
}

#[test]
fn super_property_terminal_for_of_setter_does_not_close() {
    fatal(
        r####"terminal_for_of_setter_does_not_close"####,
        r####"var trace='',caught=0,finalized=0,after=0,queuedRan=0;
var queued=function(){queuedRan++;};queueMicrotask(queued);
function spin(){while(true){}}
var source={},iterator={next:function(){trace+='N';return {value:1,done:false};},return:function(){trace+='R';return {};}};source[Symbol.iterator]=function(){trace+='I';return iterator;};
class A { set x(v){trace+='S';spin();} }class C extends A { run(){for(super.x of source){trace+='B';}} }
try{new C().run();after++;}catch(e){caught++;}finally{finalized++;}after++;
"####,
        r####"INS"####,
        false,
    );
}

#[test]
fn super_property_unsupported_key_expression() {
    fatal(
        r####"unsupported_key_expression"####,
        r####"var trace='',caught=0,finalized=0,after=0,queuedRan=0;
var queued=function(){queuedRan++;};queueMicrotask(queued);
function spin(){while(true){}}
class A { get x(){trace+='G';return 1;} }class C extends A { run(){return super[(trace+='K',eval('1'))];} }

try{new C().run();after++;}catch(e){caught++;}finally{finalized++;}after++;
"####,
        r####"K"####,
        true,
    );
}

#[test]
fn super_property_unsupported_key_conversion() {
    fatal(
        r####"unsupported_key_conversion"####,
        r####"var trace='',caught=0,finalized=0,after=0,queuedRan=0;
var queued=function(){queuedRan++;};queueMicrotask(queued);
function spin(){while(true){}}
var key={};key[Symbol.toPrimitive]=function(){trace+='C';return eval('1');};
class A { get x(){trace+='G';return 1;} }class C extends A { run(){return super[(trace+='K',key)];} }

try{new C().run();after++;}catch(e){caught++;}finally{finalized++;}after++;
"####,
        r####"KC"####,
        true,
    );
}

#[test]
fn super_property_unsupported_getter() {
    fatal(
        r####"unsupported_getter"####,
        r####"var trace='',caught=0,finalized=0,after=0,queuedRan=0;
var queued=function(){queuedRan++;};queueMicrotask(queued);
function spin(){while(true){}}
class A { get x(){trace+='G';return eval('1');} }class C extends A { run(){return super[(trace+='K','x')];} }

try{new C().run();after++;}catch(e){caught++;}finally{finalized++;}after++;
"####,
        r####"KG"####,
        true,
    );
}

#[test]
fn super_property_unsupported_assignment_rhs() {
    fatal(
        r####"unsupported_assignment_rhs"####,
        r####"var trace='',caught=0,finalized=0,after=0,queuedRan=0;
var queued=function(){queuedRan++;};queueMicrotask(queued);
function spin(){while(true){}}
var key={};key[Symbol.toPrimitive]=function(){trace+='C';return 'x';};
class A { set x(v){trace+='S';} }class C extends A { run(){return super[(trace+='K',key)]=(trace+='R',eval('1'));} }

try{new C().run();after++;}catch(e){caught++;}finally{finalized++;}after++;
"####,
        r####"KR"####,
        true,
    );
}

#[test]
fn super_property_unsupported_setter() {
    fatal(
        r####"unsupported_setter"####,
        r####"var trace='',caught=0,finalized=0,after=0,queuedRan=0;
var queued=function(){queuedRan++;};queueMicrotask(queued);
function spin(){while(true){}}
var key={};key[Symbol.toPrimitive]=function(){trace+='C';return 'x';};
class A { set x(v){trace+='S';eval('1');} }class C extends A { run(){return super[(trace+='K',key)]=(trace+='R',1);} }

try{new C().run();after++;}catch(e){caught++;}finally{finalized++;}after++;
"####,
        r####"KRCS"####,
        true,
    );
}
