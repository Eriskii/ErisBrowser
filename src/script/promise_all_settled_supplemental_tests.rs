//! Separately selected supplement; original fourteen semantic cases stay exact.
//! Descriptor checks, independent callback metadata, and both snapshot prototypes.
use super::*;

fn observe(case_id: &str, source: &str, expression: &str, expected: Value) {
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
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
fn promise_all_settled_supplemental_metadata_status_and_intrinsic_prototypes() {
    observe(
        "supplemental_metadata_status_and_intrinsic_prototypes",
        r###"var O = Object, op = O.prototype, fp = Function.prototype;
function metadata(f, name) {
  var n = O.getOwnPropertyDescriptor(f, 'name');
  var l = O.getOwnPropertyDescriptor(f, 'length');
  return f.name === name && f.length === 1 && n.value === name && l.value === 1 &&
    !n.writable && !n.enumerable && n.configurable &&
    !l.writable && !l.enumerable && l.configurable &&
    O.getPrototypeOf(f) === fp && !O.prototype.hasOwnProperty.call(f, 'prototype');
}
function notConstructor(f) {
  var correct = false;
  try { new f(1); } catch (e) { correct = e instanceof TypeError; }
  return correct;
}
var methodMeta = metadata(Promise.allSettled, 'allSettled') && notConstructor(Promise.allSettled);
var pairs = [], result, calls = 0, token = {}, reason = {}, setters = 0;
function C(executor) {
  executor(function (a) { result = a; calls++; }, function () { throw 'unexpected reject'; });
}
C.resolve = function () { return {then:function (f,r) { pairs.push([f,r]); }}; };
Promise.allSettled.call(C, [0,1]);
var f = pairs[0][0], r = pairs[0][1], f1 = pairs[1][0], r1 = pairs[1][1];
var callbackMeta = metadata(f, '') && metadata(r, '') && metadata(f1, '') && metadata(r1, '') &&
  notConstructor(f) && notConstructor(r) && notConstructor(f1) && notConstructor(r1) && calls === 0;
var distinct = f !== r && f !== f1 && f !== r1 && r !== f1 && r !== r1 && f1 !== r1;
O.defineProperty(f, 'name', {value:'left'});
O.defineProperty(f, 'length', {value:7});
O.defineProperty(r, 'name', {value:'right'});
var independent = f.name === 'left' && f.length === 7 && r.name === 'right' && r.length === 1 &&
  f1.name === '' && f1.length === 1 && r1.name === '' && r1.length === 1;
O.defineProperty(op, 'status', {set:function () { setters++; }, configurable:true});
Object = function () { throw 'replaced global Object'; };
f(token); r1(reason);
Object = O;
delete op.status;
var d = O.getOwnPropertyDescriptor(result[0], 'status');
var e = O.getOwnPropertyDescriptor(result[1], 'status');
var statuses = d.value === 'fulfilled' && d.writable && d.enumerable && d.configurable &&
  e.value === 'rejected' && e.writable && e.enumerable && e.configurable;
var prototypes = O.getPrototypeOf(result[0]) === op && O.getPrototypeOf(result[1]) === op;
var payloads = result[0].value === token && result[1].reason === reason;
var out = [methodMeta, callbackMeta, distinct, independent, statuses, prototypes, payloads,
  calls === 1 && setters === 0].join('|');
"###,
        r###"out"###,
        Value::String("true|true|true|true|true|true|true|true".into()),
    );
}
