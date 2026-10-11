//! Independent WebIDL operation-binding literals; no queue-private API.
use super::*;

fn fresh() -> (Runtime, Document) {
    (Runtime::try_new().unwrap(), Document::parse(""))
}

fn evaluate(runtime: &mut Runtime, document: &mut Document, source: &str) -> Value {
    let result = runtime.execute(source, document).unwrap();
    assert!(runtime.frames.is_empty());
    assert_eq!(
        (runtime.calls, runtime.eval_depth, runtime.stack_units),
        (0, 0, 0)
    );
    result
}

#[test]
fn microtask_binding_identity_descriptors_and_nonconstructability() {
    let (mut runtime, mut document) = fresh();
    assert_eq!(
        evaluate(
            &mut runtime,
            &mut document,
            r#"
        var q = queueMicrotask;
        function require(ok) { if (!ok) throw 'binding mismatch'; }
        function typeError(f) {
            try { f(); } catch (e) { require(e instanceof TypeError); return; }
            throw 'missing TypeError';
        }
        require(q === window.queueMicrotask && q === globalThis.queueMicrotask);
        var global = Object.getOwnPropertyDescriptor(window, 'queueMicrotask');
        require(global.value === q && global.writable && global.enumerable && global.configurable);
        require(Object.getPrototypeOf(q) === Function.prototype);
        var name = Object.getOwnPropertyDescriptor(q, 'name');
        var length = Object.getOwnPropertyDescriptor(q, 'length');
        require(name.value === 'queueMicrotask' && !name.writable && !name.enumerable && name.configurable);
        require(length.value === 1 && !length.writable && !length.enumerable && length.configurable);
        require(!Object.prototype.hasOwnProperty.call(q, 'prototype'));
        typeError(function () { new q(function () {}); });
        typeError(function () { Reflect.construct(q, [function () {}]); });
        typeError(function () { Reflect.construct(function () {}, [], q); });
        q.prototype = {};
        typeError(function () { new q(function () {}); });
        'binding-ok';
    "#
        ),
        Value::String("binding-ok".into())
    );
}

#[test]
fn microtask_binding_detached_null_undefined_and_authentic_global_receivers() {
    let (mut runtime, mut document) = fresh();
    assert_eq!(
        evaluate(
            &mut runtime,
            &mut document,
            r#"
        var q = queueMicrotask, log = '';
        q(function () { log += 'A'; });
        (function () { 'use strict'; q(function () { log += 'B'; }); })();
        q.call(null, function () { log += 'C'; });
        q.call(undefined, function () { log += 'D'; });
        q.call(window, function () { log += 'E'; });
        globalThis = {};
        q.call(null, function () { log += 'F'; });
        log += 'S';
        log;
    "#
        ),
        Value::String("S".into())
    );
    assert_eq!(
        evaluate(&mut runtime, &mut document, "log;"),
        Value::String("SABCDEF".into())
    );
}

#[test]
fn microtask_binding_rejects_missing_noncallable_and_wrong_brand_synchronously() {
    let (mut runtime, mut document) = fresh();
    assert_eq!(
        evaluate(
            &mut runtime,
            &mut document,
            r#"
        var q = queueMicrotask, failures = 0, args = '', calls = '';
        function callback() { calls += 'K'; }
        function rejects(f) {
            try { f(); } catch (e) {
                if (!(e instanceof TypeError)) throw e;
                failures++; return;
            }
            throw 'missing TypeError';
        }
        var listener = {};
        Object.defineProperty(listener, 'handleEvent', {get: function () { throw 'not callable conversion'; }});
        rejects(function () { q(); });
        rejects(function () { q(undefined); });
        rejects(function () { q(null); });
        rejects(function () { q(1); });
        rejects(function () { q(listener); });
        rejects(function () { q.call({}, callback); });
        rejects(function () { q.call(Object.create(window), callback); });
        rejects(function () { ({q:q}).q(callback); });
        rejects(function () { q.call(1, callback); });
        rejects(function () { q.call(Symbol('receiver'), callback); });
        rejects(function () { q.call({}, (args += 'a', callback), (args += 'b', 0)); });
        var extra = {toString:function () { throw 'unused extra'; }};
        var result = q((args += 'c', callback), (args += 'd', extra));
        if (result !== undefined) throw 'return mismatch';
        failures + '|' + args + '|' + calls;
    "#
        ),
        Value::String("11|abcd|".into())
    );
    assert_eq!(
        evaluate(
            &mut runtime,
            &mut document,
            "failures + '|' + args + '|' + calls;"
        ),
        Value::String("11|abcd|K".into())
    );
}

#[test]
fn microtask_binding_global_replacement_deletion_and_saved_intrinsic() {
    let (mut runtime, mut document) = fresh();
    assert_eq!(
        evaluate(
            &mut runtime,
            &mut document,
            r#"
        var saved = queueMicrotask, log = '';
        window.queueMicrotask = 7;
        if (queueMicrotask !== 7 || globalThis.queueMicrotask !== 7) throw 'replacement mismatch';
        Object.defineProperty(globalThis, 'queueMicrotask', {
            value: 9, writable: true, enumerable: true, configurable: true
        });
        if (queueMicrotask !== 9 || window.queueMicrotask !== 9) throw 'definition mismatch';
        if (!delete window.queueMicrotask || typeof queueMicrotask !== 'undefined') throw 'deletion mismatch';
        if (Object.prototype.hasOwnProperty.call(globalThis, 'queueMicrotask')) throw 'remaining property';
        saved(function () { log += 'saved'; });
        'global-ok';
    "#
        ),
        Value::String("global-ok".into())
    );
    assert_eq!(
        evaluate(&mut runtime, &mut document, "log;"),
        Value::String("saved".into())
    );
}
