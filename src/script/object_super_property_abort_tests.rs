//! Frozen independent object-method SuperProperty literals.
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

fn fatal(
    case_id: &str,
    source: &str,
    expected_trace: &str,
    unsupported: bool,
    retained_strings: &[(&str, &str)],
) {
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
        for &(name, value) in retained_strings {
            assert_eq!(
                retained(&runtime, name),
                &Value::String(value.into()),
                "{case_id} strict={strict} {name}"
            );
        }
    }
}

#[test]
fn object_super_property_terminal_computed_definition_key() {
    fatal(
        r####"terminal_computed_definition_key"####,
        r####"var trace='',caught=0,finalized=0,after=0,queuedRan=0;
var queued=function(){queuedRan++;};queueMicrotask(queued);
function spin(){while(true){}}
var published='old';
try{published={m(){return super.x;},[(trace+='D',spin())](){return super.x;}};after++;}catch(e){caught++;}finally{finalized++;}after++;
"####,
        r####"D"####,
        false,
        &[(r####"published"####, r####"old"####)],
    );
}

#[test]
fn object_super_property_terminal_super_key_coercion() {
    fatal(
        r####"terminal_super_key_coercion"####,
        r####"var trace='',caught=0,finalized=0,after=0,queuedRan=0;
var queued=function(){queuedRan++;};queueMicrotask(queued);
function spin(){while(true){}}
var key={};key[Symbol.toPrimitive]=function(){trace+='C';spin();return 'x';};
var home={__proto__:{get x(){trace+='G';return 1;}},m(){return super[(trace+='K',key)];}};
try{home.m();after++;}catch(e){caught++;}finally{finalized++;}after++;
"####,
        r####"KC"####,
        false,
        &[],
    );
}

#[test]
fn object_super_property_terminal_parameter_default_getter() {
    fatal(
        r####"terminal_parameter_default_getter"####,
        r####"var trace='',caught=0,finalized=0,after=0,queuedRan=0;
var queued=function(){queuedRan++;};queueMicrotask(queued);
function spin(){while(true){}}
var home={__proto__:{get x(){trace+='G';spin();return 1;}},m(v=super.x){trace+='B';return v;}};
try{home.m();after++;}catch(e){caught++;}finally{finalized++;}after++;
"####,
        r####"G"####,
        false,
        &[],
    );
}

#[test]
fn object_super_property_terminal_escaped_arrow_setter() {
    fatal(
        r####"terminal_escaped_arrow_setter"####,
        r####"var trace='',caught=0,finalized=0,after=0,queuedRan=0;
var queued=function(){queuedRan++;};queueMicrotask(queued);
function spin(){while(true){}}
var key={};key[Symbol.toPrimitive]=function(){trace+='C';return 'x';};
var home={__proto__:{set x(v){trace+='S';spin();}},make(){trace+='A';return v=>super[(trace+='K',key)]=v;}};
var arrow=home.make();try{arrow(3);after++;}catch(e){caught++;}finally{finalized++;}after++;
"####,
        r####"AKCS"####,
        false,
        &[],
    );
}

#[test]
fn object_super_property_unsupported_super_getter() {
    fatal(
        r####"unsupported_super_getter"####,
        r####"var trace='',caught=0,finalized=0,after=0,queuedRan=0;
var queued=function(){queuedRan++;};queueMicrotask(queued);
function spin(){while(true){}}
var home={__proto__:{get x(){trace+='G';eval('1');return 1;}},m(){return super.x;}};
try{home.m();after++;}catch(e){caught++;}finally{finalized++;}after++;
"####,
        r####"G"####,
        true,
        &[],
    );
}

#[test]
fn object_super_property_unsupported_assignment_rhs() {
    fatal(
        r####"unsupported_assignment_rhs"####,
        r####"var trace='',caught=0,finalized=0,after=0,queuedRan=0;
var queued=function(){queuedRan++;};queueMicrotask(queued);
function spin(){while(true){}}
var key={};key[Symbol.toPrimitive]=function(){trace+='C';return 'x';};
var home={__proto__:{set x(v){trace+='S';}},m(){super[(trace+='K',key)]=(trace+='R',eval('1'));trace+='B';}};
try{home.m();after++;}catch(e){caught++;}finally{finalized++;}after++;
"####,
        r####"KR"####,
        true,
        &[],
    );
}
