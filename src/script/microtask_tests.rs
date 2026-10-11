//! Literal host-checkpoint expectations, frozen before implementation.
use super::*;

fn fixture() -> (Runtime, Document, NodeId) {
    let doc = Document::parse("<body><button id=t></button></body>");
    let target = doc.query_selector("#t").unwrap();
    (Runtime::new(), doc, target)
}

fn text(runtime: &mut Runtime, doc: &mut Document, expression: &str, expected: &str) {
    assert_eq!(
        runtime.execute(expression, doc).unwrap(),
        Value::String(expected.into())
    );
}

#[test]
fn microtask_fifo_append_identity_duplicates_and_completion_are_distinct() {
    let (mut runtime, mut doc, _) = fixture();
    assert_eq!(
        runtime.execute(
            "var log='';queueMicrotask(function(){log+='A';queueMicrotask(function(){log+='C';});});queueMicrotask(function(){log+='B';});log+='S';42;",
            &mut doc,
        ).unwrap(),
        Value::Number(42.0)
    );
    text(&mut runtime, &mut doc, "log", "SABC");
    runtime.execute(
        "log='';var x='a';var f=function(){log+=x;};queueMicrotask(f);queueMicrotask(f);f=function(){log+='wrong';};x='b';",
        &mut doc,
    ).unwrap();
    text(&mut runtime, &mut doc, "log", "bb");
    runtime.execute(
        "log='';function ordinary(){queueMicrotask(function(){log+='M';});log+='A';}ordinary();log+='B';",
        &mut doc,
    ).unwrap();
    text(&mut runtime, &mut doc, "log", "ABM");
}

#[test]
fn microtask_callback_this_zero_arguments_and_void_return_are_preserved() {
    let (mut runtime, mut doc, _) = fixture();
    runtime.execute(
        "var log='';queueMicrotask(function(){'use strict';log+=this===undefined?'S':'X';log+=arguments.length;var ignored={};Object.defineProperty(ignored,'then',{get:function(){throw 'then read';}});Object.defineProperty(ignored,'toString',{get:function(){throw 'coercion read';}});return ignored;});queueMicrotask(function(){log+=this===window?'G':'X';});queueMicrotask(function(){log+=this.tag;}.bind({tag:'B'}));var arrow=(function(){return ()=>{log+=this.tag;};}).call({tag:'A'});queueMicrotask(arrow);",
        &mut doc,
    ).unwrap();
    text(&mut runtime, &mut doc, "log", "S0GBA");
    assert!(runtime.console.is_empty());
}

#[test]
fn microtask_host_listener_cleanup_differs_from_script_dispatch() {
    for host in [true, false] {
        let (mut runtime, mut doc, target) = fixture();
        runtime.execute(
            "var log='';var t=document.getElementById('t');t.addEventListener('click',function(){log+='A';queueMicrotask(function(){log+='M';});});t.addEventListener('click',function(){log+='B';});",
            &mut doc,
        ).unwrap();
        if host {
            runtime.dispatch_click(target, &mut doc).unwrap();
        } else {
            runtime
                .execute("t.dispatchEvent(new Event('click'));log+='S';", &mut doc)
                .unwrap();
        }
        text(
            &mut runtime,
            &mut doc,
            "log",
            if host { "AMB" } else { "ABSM" },
        );
    }
}

#[test]
fn microtask_nested_dispatch_during_drain_appends_without_reentry() {
    let (mut runtime, mut doc, _) = fixture();
    runtime.execute(
        "var log='';var t=document.getElementById('t');t.addEventListener('click',function(){log+='L';queueMicrotask(function(){log+='C';});});queueMicrotask(function(){log+='A';t.dispatchEvent(new Event('click'));log+='R';});queueMicrotask(function(){log+='B';});",
        &mut doc,
    ).unwrap();
    text(&mut runtime, &mut doc, "log", "ALRBC");
}

#[test]
fn microtask_handle_event_getter_and_method_share_one_author_entry() {
    for host in [true, false] {
        let (mut runtime, mut doc, target) = fixture();
        runtime.execute(
            "var log='';var t=document.getElementById('t');var listener={get handleEvent(){log+='G';queueMicrotask(function(){log+='M';});return function(){log+='H';};}};t.addEventListener('click',listener);t.addEventListener('click',function(){log+='B';});",
            &mut doc,
        ).unwrap();
        if host {
            runtime.dispatch_click(target, &mut doc).unwrap();
        } else {
            runtime
                .execute("t.dispatchEvent(new Event('click'));log+='S';", &mut doc)
                .unwrap();
        }
        text(
            &mut runtime,
            &mut doc,
            "log",
            if host { "GHMB" } else { "GHBSM" },
        );
    }
}

#[test]
fn microtask_event_phase_passive_and_immediate_flags_keep_cleanup_order() {
    for passive in [true, false] {
        let (mut runtime, mut doc, target) = fixture();
        runtime.execute(
            &format!(
                "var log='';var saved;var t=document.getElementById('t');t.addEventListener('click',function(e){{saved=e;log+='A';queueMicrotask(function(){{log+=(e.currentTarget===t&&e.eventPhase===2?'M':'X');e.preventDefault();}});}},{{passive:{passive}}});t.addEventListener('click',function(e){{log+=e.defaultPrevented?'C':'B';}});"
            ),
            &mut doc,
        ).unwrap();
        runtime.dispatch_click(target, &mut doc).unwrap();
        assert_eq!(runtime.last_default_prevented, !passive);
        text(
            &mut runtime,
            &mut doc,
            "log",
            if passive { "AMB" } else { "AMC" },
        );
        assert_eq!(
            runtime
                .execute("saved.currentTarget===null&&saved.eventPhase===0", &mut doc)
                .unwrap(),
            Value::Bool(true)
        );
    }
    for host in [true, false] {
        let (mut runtime, mut doc, target) = fixture();
        runtime.execute(
            "var log='';var t=document.getElementById('t');t.addEventListener('click',function(e){log+='A';queueMicrotask(function(){log+='M';e.stopImmediatePropagation();});});t.addEventListener('click',function(){log+='B';});",
            &mut doc,
        ).unwrap();
        if host {
            runtime.dispatch_click(target, &mut doc).unwrap();
        } else {
            runtime
                .execute("t.dispatchEvent(new Event('click'));log+='S';", &mut doc)
                .unwrap();
        }
        text(
            &mut runtime,
            &mut doc,
            "log",
            if host { "AM" } else { "ABSM" },
        );
    }
}

#[test]
fn microtask_handler_false_processing_follows_its_checkpoint() {
    let (mut runtime, mut doc, target) = fixture();
    runtime.execute(
        "var log='';var t=document.getElementById('t');t.onclick=function(e){queueMicrotask(function(){log+=e.defaultPrevented?'X':'M';});return false;};t.addEventListener('click',function(e){log+=e.defaultPrevented?'C':'X';});",
        &mut doc,
    ).unwrap();
    runtime.dispatch_click(target, &mut doc).unwrap();
    assert!(runtime.last_default_prevented);
    text(&mut runtime, &mut doc, "log", "MC");
}

#[test]
fn microtask_ordinary_throws_report_and_continue_without_replacing_completion() {
    let (mut runtime, mut doc, target) = fixture();
    runtime.execute(
        "var log='';queueMicrotask(function(){log+='A';throw 'marker';});queueMicrotask(function(){log+='B';});",
        &mut doc,
    ).unwrap();
    text(&mut runtime, &mut doc, "log", "AB");
    assert_eq!(runtime.console.len(), 1);
    assert!(runtime.console[0].contains("marker"));
    let error = runtime
        .execute(
            "log='';queueMicrotask(function(){log+='M';});throw 'outer';",
            &mut doc,
        )
        .unwrap_err();
    assert!(!error.is_resource_limit() && !error.is_unsupported());
    assert!(error.message.contains("outer"));
    text(&mut runtime, &mut doc, "log", "M");
    assert_eq!(runtime.console.len(), 1);
    runtime.execute(
        "log='';var t=document.getElementById('t');t.addEventListener('click',{get handleEvent(){log+='G';queueMicrotask(function(){log+='M';});throw 'getter';}});t.addEventListener('click',function(){log+='B';});",
        &mut doc,
    ).unwrap();
    runtime.dispatch_click(target, &mut doc).unwrap();
    text(&mut runtime, &mut doc, "log", "GMB");
    assert_eq!(runtime.console.len(), 2);
    assert!(runtime.console[1].contains("getter"));
}

#[test]
fn microtask_each_script_and_readiness_listener_has_its_own_cleanup_boundary() {
    let (mut runtime, mut doc, _) = fixture();
    runtime
        .execute(
            "var log='A';queueMicrotask(function(){log+='M';});",
            &mut doc,
        )
        .unwrap();
    runtime.execute(
        "log+='B';document.addEventListener('DOMContentLoaded',function(){log+='D';queueMicrotask(function(){log+='m';});});window.addEventListener('load',function(){log+='L';});",
        &mut doc,
    ).unwrap();
    runtime.dispatch_dom_content_loaded(&mut doc).unwrap();
    text(&mut runtime, &mut doc, "log", "AMBDmL");
    runtime.dispatch_dom_content_loaded(&mut doc).unwrap();
    text(&mut runtime, &mut doc, "log", "AMBDmL");
}

#[test]
fn microtask_fatal_callback_is_not_replayed_by_later_host_budgets() {
    for fatal in ["while(true){}", "eval('0')"] {
        let (mut runtime, mut doc, _) = fixture();
        let error = runtime.execute(
            &format!("var log='';queueMicrotask(function(){{log+='A';{fatal}}});queueMicrotask(function(){{log+='B';}});"),
            &mut doc,
        ).unwrap_err();
        assert!(error.is_resource_limit() || error.is_unsupported());
        for _ in 0..3 {
            text(&mut runtime, &mut doc, "log", "A");
        }
        assert!(
            runtime
                .execute("queueMicrotask(function(){log+='C';})", &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        text(&mut runtime, &mut doc, "log", "A");
        let (mut fresh, mut fresh_doc, _) = fixture();
        fresh
            .execute(
                "var log='';queueMicrotask(function(){log+='fresh';});",
                &mut fresh_doc,
            )
            .unwrap();
        text(&mut fresh, &mut fresh_doc, "log", "fresh");
    }
}

#[test]
fn microtask_fatal_host_listener_checkpoint_still_restores_event_state() {
    let (mut runtime, mut doc, target) = fixture();
    runtime.execute(
        "var log='';var saved;var t=document.getElementById('t');t.addEventListener('click',function(e){saved=e;queueMicrotask(function(){log+='A';while(true){}});});t.addEventListener('click',function(){log+='B';});",
        &mut doc,
    ).unwrap();
    assert!(
        runtime
            .dispatch_click(target, &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    text(&mut runtime, &mut doc, "log", "A");
    assert_eq!(
        runtime
            .execute("saved.currentTarget===null&&saved.eventPhase===0", &mut doc)
            .unwrap(),
        Value::Bool(true)
    );
    assert_eq!(runtime.calls, 0);
    assert_eq!(runtime.eval_depth, 0);
    assert_eq!(runtime.stack_units, 0);
    assert!(runtime.frames.is_empty());
}
