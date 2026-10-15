//! Fourteen exact allSettled literals held before implementation.
//! A fresh Runtime is used for each mode. No test counter reset or private API.
use super::*;

fn observe(case_id: &str, source: &str, expression: &str, expected: Value) {
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        if case_id == "invalid_constructor_before_inputs" {
            let present = if strict {
                runtime.execute_strict("typeof Promise.allSettled === 'function'", &mut document)
            } else {
                runtime.execute("typeof Promise.allSettled === 'function'", &mut document)
            }
            .unwrap_or_else(|error| panic!("{case_id} strict={strict} presence: {error:?}"));
            assert_eq!(
                present,
                Value::Bool(true),
                "{case_id} strict={strict} method presence"
            );
        }
        let result = if strict {
            runtime.execute_strict(source, &mut document)
        } else {
            runtime.execute(source, &mut document)
        };
        result.unwrap_or_else(|error| panic!("{case_id} strict={strict} source: {error:?}"));
        let observed = if strict {
            runtime.execute_strict(expression, &mut document)
        } else {
            runtime.execute(expression, &mut document)
        }
        .unwrap_or_else(|error| panic!("{case_id} strict={strict} observation: {error:?}"));
        assert_eq!(observed, expected, "{case_id} strict={strict}");
    }
}

#[test]
fn promise_all_settled_metadata() {
    observe(
        "metadata",
        r###"var d = Object.getOwnPropertyDescriptor(Promise, 'allSettled');
var f = d.value, error = '';
try { new f([]); } catch (e) { error = e.name; }
var out = f.name + '|' + f.length + '|' + d.writable + '|' + d.enumerable + '|' + d.configurable + '|' + Object.prototype.hasOwnProperty.call(f, 'prototype') + '|' + error;
"###,
        r###"out"###,
        Value::String("allSettled|1|true|false|true|false|TypeError".into()),
    );
}

#[test]
fn promise_all_settled_mixed_order_identity() {
    observe(
        "mixed_order_identity",
        r###"var token = {}, out = '';
Promise.allSettled([1, Promise.reject(token), Promise.resolve(3)]).then(function (a) {
  out = a[0].status + ':' + a[0].value + '|' + a[1].status + ':' + (a[1].reason === token) + '|' + a[2].status + ':' + a[2].value;
});
"###,
        r###"out"###,
        Value::String("fulfilled:1|rejected:true|fulfilled:3".into()),
    );
}

#[test]
fn promise_all_settled_empty_job_order() {
    observe(
        "empty_job_order",
        r###"var out = 'S';
Promise.allSettled([]).then(function () { out += 'E'; });
queueMicrotask(function () { out += 'Q'; });
out += 'B';
"###,
        r###"out"###,
        Value::String("SBEQ".into()),
    );
}

#[test]
fn promise_all_settled_nonempty_job_order() {
    observe(
        "nonempty_job_order",
        r###"var out = 'S';
Promise.allSettled([1]).then(function () { out += 'A'; });
queueMicrotask(function () { out += 'Q'; });
out += 'B';
"###,
        r###"out"###,
        Value::String("SBQA".into()),
    );
}

#[test]
fn promise_all_settled_generic_empty_order_no_species_or_close() {
    observe(
        "generic_empty_order_no_species_or_close",
        r###"var out = '';
function C(executor) {
  out += 'C';
  executor(function (a) { 'use strict'; out += 'R' + a.length + (this === undefined ? 'U' : 'X'); }, function () { out += 'X'; });
}
Object.defineProperty(C, 'resolve', {get: function () { out += 'G'; return function () { out += 'W'; }; }});
Object.defineProperty(C, Symbol.species, {get: function () { throw 'species'; }});
var iterator = {};
Object.defineProperty(iterator, 'next', {get: function () { out += 'N'; return function () { out += 'D'; return {done:true}; }; }});
Object.defineProperty(iterator, 'return', {get: function () { throw 'close'; }});
var items = {};
Object.defineProperty(items, Symbol.iterator, {get: function () { out += 'I'; return function () { out += 'J'; return iterator; }; }});
var result = Promise.allSettled.call(C, items);
out += '|' + (result instanceof C);
"###,
        r###"out"###,
        Value::String("CGIJNDR0U|true".into()),
    );
}

#[test]
fn promise_all_settled_paired_gate_synchronous_sentinel() {
    observe(
        "paired_gate_synchronous_sentinel",
        r###"var token = {}, result, calls = 0, pairs = [], out;
function C(executor) { executor(function (a) { result = a; calls++; }, function () { throw 'reject'; }); }
C.resolve = function (v) { return {then:function (f,r) {
  pairs.push([f,r]);
  if (calls !== 0) throw 'premature';
  if (v === 0) { f('A'); r('ignored'); f('ignored'); }
  else { r(token); f('ignored'); r('ignored'); }
  if (calls !== 0) throw 'premature';
}}; };
Promise.allSettled.call(C, [0,1]);
var dup = pairs[0][1]('late');
out = calls + '|' + result[0].status + ':' + result[0].value + '|' + result[1].status + ':' + (result[1].reason === token) + '|' + (dup === undefined) + '|' + (pairs[0][0] !== pairs[0][1]) + '|' + (pairs[0][0] !== pairs[1][0]);
"###,
        r###"out"###,
        Value::String("1|fulfilled:A|rejected:true|true|true|true".into()),
    );
}

#[test]
fn promise_all_settled_saved_callbacks_return_and_order() {
    observe(
        "saved_callbacks_return_and_order",
        r###"var pairs = [], marker = {}, token = {}, result, calls = 0;
function C(executor) { executor(function (a) { 'use strict'; if (this !== undefined) throw 'this'; result = a; calls++; return 'RET'; }, function () { throw 'reject'; }); return marker; }
C.resolve = function () { return {then:function (f,r) { pairs.push([f,r]); }}; };
var p = Promise.allSettled.call(C, [0,1]);
var first = pairs[1][1].call({wrong:true}, token);
var last = pairs[0][0].call(null, 7);
var duplicate = pairs[0][1]('ignored');
var out = (p === marker) + '|' + (first === undefined) + '|' + last + '|' + (duplicate === undefined) + '|' + calls + '|' + result[0].status + ':' + result[0].value + '|' + result[1].status + ':' + (result[1].reason === token);
"###,
        r###"out"###,
        Value::String("true|true|RET|true|1|fulfilled:7|rejected:true".into()),
    );
}

#[test]
fn promise_all_settled_last_resolve_throw_consumes_pair() {
    observe(
        "last_resolve_throw_consumes_pair",
        r###"var f, r, token = {}, seen, calls = 0;
function C(executor) { executor(function () { calls++; throw token; }, function () { throw 'reject'; }); }
C.resolve = function () { return {then:function (a,b) { f = a; r = b; }}; };
Promise.allSettled.call(C, [1]);
try { f(7); } catch (e) { seen = e; }
var duplicate = r('ignored');
var out = (seen === token) + '|' + calls + '|' + (duplicate === undefined);
"###,
        r###"out"###,
        Value::String("true|1|true".into()),
    );
}

#[test]
fn promise_all_settled_ordinary_resolve_abort_retains_sentinel() {
    observe(
        "ordinary_resolve_abort_retains_sentinel",
        r###"var f, r, token = {}, other = {}, rejected, calls = 0, closed = 0, index = 0;
function C(executor) { executor(function () { calls++; }, function (x) { rejected = x; }); }
C.resolve = function (v) { if (v === 2) throw token; return {then:function (a,b) { f = a; r = b; }}; };
var iterator = {next:function () { index++; return index <= 2 ? {done:false,value:index} : {done:true}; }, return:function () { closed++; throw other; }};
var items = {}; items[Symbol.iterator] = function () { return iterator; };
Promise.allSettled.call(C, items);
var first = r('reason'), duplicate = f('ignored');
var out = (rejected === token) + '|' + closed + '|' + calls + '|' + (first === undefined) + '|' + (duplicate === undefined);
"###,
        r###"out"###,
        Value::String("true|1|0|true|true".into()),
    );
}

#[test]
fn promise_all_settled_ordinary_next_abort_does_not_close() {
    observe(
        "ordinary_next_abort_does_not_close",
        r###"var f, r, token = {}, rejected, calls = 0, closed = 0, index = 0;
function C(executor) { executor(function () { calls++; }, function (x) { rejected = x; }); }
C.resolve = function () { return {then:function (a,b) { f = a; r = b; }}; };
var iterator = {next:function () { if (index++) throw token; return {done:false,value:1}; }};
Object.defineProperty(iterator, 'return', {get:function () { closed++; throw 'close'; }});
var items = {}; items[Symbol.iterator] = function () { return iterator; };
Promise.allSettled.call(C, items);
f(1); var duplicate = r('ignored');
var out = (rejected === token) + '|' + closed + '|' + calls + '|' + (duplicate === undefined);
"###,
        r###"out"###,
        Value::String("true|0|0|true".into()),
    );
}

#[test]
fn promise_all_settled_invalid_constructor_before_inputs() {
    observe(
        "invalid_constructor_before_inputs",
        r###"var touched = 0, target = {}, items = {}, error = '';
Object.defineProperty(target, 'resolve', {get:function () { touched++; throw 'resolve'; }});
Object.defineProperty(items, Symbol.iterator, {get:function () { touched++; throw 'iterator'; }});
try { Promise.allSettled.call(target, items); } catch (e) { error = e.name; }
var out = error + '|' + touched;
"###,
        r###"out"###,
        Value::String("TypeError|0".into()),
    );
}

#[test]
fn promise_all_settled_noncallable_resolve_rejects_before_iterator() {
    observe(
        "noncallable_resolve_rejects_before_iterator",
        r###"var out = '', reason;
function C(executor) { out += 'C'; executor(function () { out += 'R'; }, function (x) { reason = x; out += 'X'; }); }
Object.defineProperty(C, 'resolve', {get:function () { out += 'G'; return 1; }});
var items = {}; Object.defineProperty(items, Symbol.iterator, {get:function () { out += 'I'; throw 'iterator'; }});
var p = Promise.allSettled.call(C, items);
out += '|' + reason.name + '|' + (p instanceof C);
"###,
        r###"out"###,
        Value::String("CGX|TypeError|true".into()),
    );
}

#[test]
fn promise_all_settled_snapshot_shape_intrinsics_and_undefined() {
    observe(
        "snapshot_shape_intrinsics_and_undefined",
        r###"var O = Object, A = Array, op = Object.prototype, ap = Array.prototype, pairs = [], result, setterCalls = 0;
function C(executor) { executor(function (a) { result = a; }, function () { throw 'reject'; }); }
C.resolve = function () { return {then:function (f,r) { pairs.push([f,r]); }}; };
Promise.allSettled.call(C, [0,1]);
var setterDescriptor = O.create(null);
setterDescriptor.set = function () { setterCalls++; };
setterDescriptor.configurable = true;
O.defineProperty(op, 'status', setterDescriptor);
O.defineProperty(op, 'value', setterDescriptor);
O.defineProperty(op, 'reason', setterDescriptor);
Object = function () { throw 'global Object'; }; Array = function () { throw 'global Array'; };
pairs[0][0](); pairs[1][1]();
Object = O; Array = A;
delete op.status; delete op.value; delete op.reason;
var d = O.getOwnPropertyDescriptor(result[0], 'value'), e = O.getOwnPropertyDescriptor(result[1], 'reason');
var out = setterCalls + '|' + (O.getPrototypeOf(result) === ap) + '|' + (O.getPrototypeOf(result[0]) === op) + '|' + (result[0] !== result[1]) + '|' + O.keys(result[0]).join(',') + '|' + O.keys(result[1]).join(',') + '|' + (d.value === undefined && e.value === undefined) + '|' + (d.writable && d.enumerable && d.configurable && e.writable && e.enumerable && e.configurable);
"###,
        r###"out"###,
        Value::String("0|true|true|true|status,value|status,reason|true|true".into()),
    );
}

#[test]
fn promise_all_settled_snapshot_payload_is_not_assimilated() {
    observe(
        "snapshot_payload_is_not_assimilated",
        r###"var token = {}, reads = 0, result, pairs = [];
Object.defineProperty(token, 'then', {get:function () { reads++; throw 'payload then'; }});
function C(executor) { executor(function (a) { result = a; }, function () { throw 'reject'; }); }
C.resolve = function () { return {then:function (f,r) { pairs.push([f,r]); }}; };
Promise.allSettled.call(C, [0,1]);
pairs[0][0](token); pairs[1][1](token);
var out = reads + '|' + (result[0].value === token) + '|' + (result[1].reason === token);
"###,
        r###"out"###,
        Value::String("0|true|true".into()),
    );
}
