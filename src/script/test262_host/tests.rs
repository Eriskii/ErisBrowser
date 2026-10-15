//! Runtime-only completion authority controls. No transport or upstream run claim.
use super::*;

const DONE: &str = include_str!(
    "../../../tests/upstream/test262-promise-methods-async/harness/doneprintHandle.js"
);

fn prepared(policy: Test262Policy, arm: bool) -> (Runtime, Document) {
    let mut runtime = Runtime::new();
    let mut document = Document::parse("");
    runtime.enable_test262_host(policy).unwrap();
    if policy == Test262Policy::AsyncDone {
        runtime.execute(DONE, &mut document).unwrap();
        runtime.bind_test262_done_from_harness().unwrap();
    }
    if arm {
        runtime.arm_test262_body().unwrap();
    }
    (runtime, document)
}

fn counts(observation: Test262Observation) -> (u8, u8, u8) {
    (
        observation.done_calls,
        observation.success_prints,
        observation.failure_prints,
    )
}

fn assert_clean(observation: Test262Observation, expected: (u8, u8, u8)) {
    assert_eq!(counts(observation), expected);
    assert_eq!(observation.flags, 0);
    assert!(observation.quiescent && observation.audited);
    assert_eq!(observation.unhandled, 0);
}

fn global<'a>(runtime: &'a Runtime, name: &str) -> &'a Value {
    &runtime.environments[0].bindings[name].value
}

#[test]
fn host_falsy_completion_delegates_the_unchanged_helper_in_both_modes() {
    for strict in [false, true] {
        for source in [
            "$DONE();",
            "$DONE(undefined);",
            "$DONE(null);",
            "$DONE(false);",
            "$DONE(0);",
            "$DONE(-0);",
            "$DONE(NaN);",
            "$DONE('');",
        ] {
            let (mut runtime, mut document) = prepared(Test262Policy::AsyncDone, true);
            let body = if strict {
                runtime.execute_strict(source, &mut document)
            } else {
                runtime.execute(source, &mut document)
            };
            assert_eq!(body.as_ref().unwrap(), &Value::Undefined);
            let observation = runtime.finish_test262_body(&body).unwrap();
            assert_eq!(observation.policy, Test262Policy::AsyncDone);
            assert_clean(observation, (1, 1, 0));
            assert!(runtime.console.is_empty());
        }
    }
}

#[test]
fn host_hooks_keep_public_descriptors_and_saved_identity_after_mutation() {
    let (mut runtime, mut document) = prepared(Test262Policy::AsyncDone, true);
    let body = runtime.execute(
        r#"
var saved=$DONE;var out='';
var p=Object.getOwnPropertyDescriptor(globalThis,'print');
out+=p.writable&&!p.enumerable&&p.configurable?'P':'X';
for(var f of [print,saved]){
 var n=Object.getOwnPropertyDescriptor(f,'name');
 var l=Object.getOwnPropertyDescriptor(f,'length');
 out+=!n.writable&&!n.enumerable&&n.configurable?'N':'X';
 out+=l.value===1&&!l.writable&&!l.enumerable&&l.configurable?'L':'X';
 out+=Object.getPrototypeOf(f)===Function.prototype&&!Object.hasOwn(f,'prototype')?'F':'X';
 try{new f();out+='X';}catch(e){out+=e instanceof TypeError?'T':'X';}
}
out+=print.name==='print'&&saved.name==='$DONE'?'I':'X';
Object.defineProperty(saved,'name',{value:'forged'});
Object.defineProperty(saved,'length',{value:99});
$DONE=function(){throw 'replacement';};
Promise.resolve().then(function(){saved();});out;
"#,
        &mut document,
    );
    assert_eq!(body.as_ref().unwrap(), &Value::String("PNLFTNLFTI".into()));
    let observation = runtime.finish_test262_body(&body).unwrap();
    assert_clean(observation, (1, 1, 0));
    assert!(runtime.console.is_empty());
}

#[test]
fn host_delegate_preserves_receiver_arguments_return_and_exact_abrupt_value() {
    // Trusted Rust call-routing fixture only: the adapter may bind only after
    // authenticating the pinned helper. This custom original isolates forwarding.
    let mut runtime = Runtime::new();
    let mut document = Document::parse("");
    runtime
        .enable_test262_host(Test262Policy::AsyncDone)
        .unwrap();
    runtime.execute("var seen='';function $DONE(a,b){'use strict';seen=this.tag+arguments.length+String(a)+b;print('Test262:AsyncTestComplete');return b;}", &mut document).unwrap();
    runtime.bind_test262_done_from_harness().unwrap();
    runtime.arm_test262_body().unwrap();
    let body = runtime.execute("$DONE.call({tag:'R'},undefined,'Q');", &mut document);
    assert_eq!(body.as_ref().unwrap(), &Value::String("Q".into()));
    assert_eq!(
        global(&runtime, "seen"),
        &Value::String("R2undefinedQ".into())
    );
    assert_clean(runtime.finish_test262_body(&body).unwrap(), (1, 1, 0));

    let (mut runtime, mut document) = prepared(Test262Policy::AsyncDone, true);
    let body = runtime.execute("var token={};var hits=0;var same=false;try{$DONE({get name(){hits++;throw token;}});}catch(e){same=e===token;}", &mut document);
    assert!(body.is_ok());
    assert_eq!(global(&runtime, "hits"), &Value::Number(1.0));
    assert_eq!(global(&runtime, "same"), &Value::Bool(true));
    let observation = runtime.finish_test262_body(&body).unwrap();
    assert_eq!(counts(observation), (1, 0, 0));
    assert_eq!(observation.flags, 0x41);
    assert!(observation.audited && observation.quiescent);
    assert_eq!(
        runtime.test262_session.as_ref().unwrap().active_done_depth,
        0
    );
}

#[test]
fn host_reentrant_helper_scope_restores_depth_and_cannot_erase_truthy_failure() {
    for (source, expected, flags, trace) in [
        (
            "var log='';$DONE({get name(){log+='N';$DONE();return 'E';},get message(){log+='M';return 'marker';}});",
            (2, 1, 1),
            1,
            "NM",
        ),
        (
            "var log='';$DONE({get name(){log+='N';print('Test262:AsyncTestComplete');return 'E';},get message(){log+='M';return 'marker';}});",
            (1, 1, 1),
            1,
            "NM",
        ),
        ("var log='';$DONE();$DONE();$DONE();", (2, 2, 0), 0, ""),
    ] {
        let (mut runtime, mut document) = prepared(Test262Policy::AsyncDone, true);
        let body = runtime.execute(source, &mut document);
        assert!(body.is_ok());
        assert_eq!(global(&runtime, "log"), &Value::String(trace.into()));
        let observation = runtime.finish_test262_body(&body).unwrap();
        assert_eq!(counts(observation), expected);
        assert_eq!(observation.flags, flags);
        assert!(observation.quiescent && observation.audited);
        assert_eq!(
            runtime.test262_session.as_ref().unwrap().active_done_depth,
            0
        );
    }
}

#[test]
fn host_printed_text_and_replaced_public_functions_have_no_completion_authority() {
    for (source, expected, flags, console) in [
        ("print('Test262:AsyncTestComplete');", (0, 0, 0), 0x10, 0),
        ("console.log('Test262:AsyncTestComplete');", (0, 0, 0), 0, 1),
        ("$DONE=function(){};$DONE();", (0, 0, 0), 0, 0),
        ("print=function(){};$DONE();", (1, 0, 0), 0, 0),
        (
            "__consolePrintHandle__=function(){print({get toString(){throw 'coercion';}});};$DONE();",
            (1, 0, 0),
            0x20,
            0,
        ),
        (
            "__consolePrintHandle__=function(){print('Test262:AsyncTestComplete!');};$DONE();",
            (1, 0, 0),
            0x20,
            0,
        ),
    ] {
        let (mut runtime, mut document) = prepared(Test262Policy::AsyncDone, true);
        let body = runtime.execute(source, &mut document);
        assert!(body.is_ok());
        let observation = runtime.finish_test262_body(&body).unwrap();
        assert_eq!(counts(observation), expected);
        assert_eq!(observation.flags, flags);
        assert!(observation.quiescent && observation.audited);
        assert_eq!(runtime.console.len(), console);
    }
}

#[test]
fn host_private_native_identity_requires_both_stored_receiver_and_metadata_id() {
    for field in ["name", "receiver", "properties"] {
        let (mut runtime, _) = prepared(Test262Policy::AsyncDone, true);
        let Value::Native(real) = global(&runtime, "$DONE") else {
            panic!("DONE hook");
        };
        let mut forged = (**real).clone();
        match field {
            "name" => forged.name = "Test262.print".into(),
            "receiver" => forged.receiver = Value::Undefined,
            "properties" => forged.properties = None,
            _ => unreachable!(),
        }
        // Impossible from author JavaScript: deliberately corrupt one private
        // field in a Rust fixture to test the authentication conjunction.
        assert!(
            runtime
                .test262_done_begin(&forged, &[Value::Undefined])
                .is_err()
        );
        let state = runtime.test262_session.as_ref().unwrap();
        assert_eq!((state.done_calls, state.active_done_depth), (0, 0));
        assert_ne!(state.flags & 0x80, 0);
    }
}

#[test]
fn host_phases_are_one_shot_and_harness_faults_are_visible_before_next_execute() {
    let (mut runtime, mut document) = prepared(Test262Policy::AsyncDone, false);
    let body = runtime.execute("$DONE();", &mut document);
    assert!(body.is_ok());
    let observation = runtime.test262_host_observation().unwrap();
    assert_eq!(counts(observation), (1, 1, 0));
    assert_eq!(observation.flags, 2);
    assert!(!observation.audited);
    assert!(runtime.arm_test262_body().is_err());

    for action in ["enable", "bind", "arm", "finish"] {
        let (mut runtime, mut document) = prepared(Test262Policy::AsyncDone, true);
        let before = global(&runtime, "$DONE").clone();
        let print = global(&runtime, "print").clone();
        let body = runtime.execute("$DONE();", &mut document);
        assert!(body.is_ok());
        let result = match action {
            "enable" => runtime.enable_test262_host(Test262Policy::CheckedSync),
            "bind" => runtime.bind_test262_done_from_harness(),
            "arm" => runtime.arm_test262_body(),
            "finish" => {
                assert_clean(runtime.finish_test262_body(&body).unwrap(), (1, 1, 0));
                runtime.finish_test262_body(&body).map(|_| ())
            }
            _ => unreachable!(),
        };
        assert!(result.is_err());
        assert_eq!(global(&runtime, "$DONE"), &before);
        assert_eq!(global(&runtime, "print"), &print);
        let observation = runtime.test262_host_observation().unwrap();
        assert_eq!(counts(observation), (1, 1, 0));
        assert_ne!(observation.flags & 0x80, 0);
    }

    let (mut runtime, mut document) = prepared(Test262Policy::CheckedSync, false);
    assert!(
        runtime
            .execute("queueMicrotask(function(){throw 7;});", &mut document)
            .is_ok()
    );
    assert_eq!(runtime.test262_host_observation().unwrap().flags, 4);
    assert!(runtime.arm_test262_body().is_err());
}

#[test]
fn host_final_rejection_policy_uses_genuine_quiescent_records_not_console_text() {
    for (source, unhandled) in [
        ("Promise.reject('x').catch(function(){}).then($DONE);", 0),
        (
            "var p=Promise.reject('x');queueMicrotask(function(){p.catch(function(){}).then($DONE);});",
            0,
        ),
        ("$DONE();Promise.resolve().then(function(){throw 'x';});", 1),
        (
            "var p=Promise.reject('x');p.then();p.catch(function(){});$DONE();",
            1,
        ),
        ("new Promise(function(){});$DONE();", 0),
    ] {
        let (mut runtime, mut document) = prepared(Test262Policy::AsyncDone, true);
        let body = runtime.execute(source, &mut document);
        assert!(body.is_ok());
        let observation = runtime.finish_test262_body(&body).unwrap();
        assert_eq!(counts(observation), (1, 1, 0));
        assert_eq!(observation.flags, 0);
        assert!(observation.quiescent && observation.audited);
        assert_eq!(observation.unhandled, unhandled);
        assert!(runtime.console.is_empty());
    }
    let (mut runtime, mut document) = prepared(Test262Policy::CheckedSync, true);
    let body = runtime.execute(
        "Promise.resolve(1).then(function(v){if(v!==1)throw 'bad';});",
        &mut document,
    );
    assert!(body.is_ok());
    assert_clean(runtime.finish_test262_body(&body).unwrap(), (0, 0, 0));
}

#[test]
fn host_later_abrupt_or_fatal_work_cannot_be_hidden_by_provisional_success() {
    let (mut runtime, mut document) = prepared(Test262Policy::AsyncDone, true);
    let body = runtime.execute("$DONE();throw 7;", &mut document);
    assert!(matches!(
        body.as_ref().unwrap_err().kind,
        ErrorKind::Thrown(Value::Number(7.0))
    ));
    let observation = runtime.finish_test262_body(&body).unwrap();
    assert_eq!(counts(observation), (1, 1, 0));
    assert_eq!(observation.flags, 0x100);
    assert!(observation.quiescent && observation.audited);

    let (mut runtime, mut document) = prepared(Test262Policy::AsyncDone, true);
    let body = runtime.execute("var reached=0;$DONE();queueMicrotask(function(){while(true){}});queueMicrotask(function(){reached=1;});", &mut document);
    assert!(body.as_ref().unwrap_err().is_resource_limit());
    assert_eq!(global(&runtime, "reached"), &Value::Number(0.0));
    let steps = runtime.steps;
    let queued = runtime.microtasks.queue.len();
    assert!(queued > 0);
    assert!(
        runtime
            .finish_test262_body(&body)
            .unwrap_err()
            .is_resource_limit()
    );
    let observation = runtime.test262_host_observation().unwrap();
    assert_eq!(counts(observation), (1, 1, 0));
    assert_eq!(observation.flags & 0x108, 0x108);
    assert!(!observation.audited && !observation.quiescent);
    assert_eq!(observation.unhandled, 0);
    assert_eq!(
        (runtime.steps, runtime.microtasks.queue.len()),
        (steps, queued)
    );
    assert_eq!(global(&runtime, "reached"), &Value::Number(0.0));

    // Private error-observer fixture: record fatal evidence at exhausted work,
    // without constructing a message or traversing state in the latch itself.
    for unsupported in [false, true] {
        let (mut runtime, _) = prepared(Test262Policy::AsyncDone, true);
        let error = if unsupported {
            ScriptError::unsupported("fixture")
        } else {
            ScriptError::resource("fixture")
        };
        runtime.steps = 0;
        let allocated = runtime.allocated;
        let result: Result<()> = runtime.observe_microtask_failure(Err(error));
        assert!(result.is_err());
        assert_eq!((runtime.steps, runtime.allocated), (0, allocated));
        let observation = runtime.test262_host_observation().unwrap();
        assert_eq!(observation.flags, 8);
        assert_eq!(counts(observation), (0, 0, 0));
        assert!(!observation.quiescent && !observation.audited);
    }
}
