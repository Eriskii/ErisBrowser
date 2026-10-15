//! Promise.any public contracts frozen before production or execution.
//! Original thirteen literals and separately selected additions remain exact.
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
fn promise_any_metadata_and_invalid_constructor_precedes_iterable() {
    observe(
        "any_metadata_and_invalid_constructor_precedes_iterable",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
if(typeof Promise.any!=='function')throw new Error('Promise.any missing');
function check(v){if(!v)throw new Error('oracle');}
var f=Promise.any,d=Object.getOwnPropertyDescriptor(Promise,'any');check(d.value===f&&d.writable&&!d.enumerable&&d.configurable);
var l=Object.getOwnPropertyDescriptor(f,'length'),n=Object.getOwnPropertyDescriptor(f,'name');check(l.value===1&&!l.writable&&!l.enumerable&&l.configurable&&n.value==='any'&&!n.writable&&!n.enumerable&&n.configurable);
check(!Object.prototype.hasOwnProperty.call(f,'prototype'));var hit=0,items={};Object.defineProperty(items,Symbol.iterator,{get:function(){hit++;}});
var bad=[null,1,{},()=>{}],count=0;for(var i=0;i<bad.length;i++){try{f.call(bad[i],items);}catch(e){check(e instanceof TypeError);count++;}}
try{new f(items);}catch(e){check(e instanceof TypeError);count++;}check(count===5&&hit===0);var result='ok';
"###,
        r###"result"###,
        Value::String("ok".into()),
    );
}

#[test]
fn promise_any_intrinsic_first_fulfillment_and_exact_payload() {
    observe(
        "any_intrinsic_first_fulfillment_and_exact_payload",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
if(typeof Promise.any!=='function')throw new Error('Promise.any missing');
function check(v){if(!v)throw new Error('oracle');}
var a={},winner={},result='pending';Promise.any([Promise.reject(a),Promise.resolve(winner),new Promise(function(){})]).then(function(v){result=v===winner?'ok':'wrong';},function(){result='rejected';});
"###,
        r###"result"###,
        Value::String("ok".into()),
    );
}

#[test]
fn promise_any_intrinsic_ordered_errors_and_empty() {
    observe(
        "any_intrinsic_ordered_errors_and_empty",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
if(typeof Promise.any!=='function')throw new Error('Promise.any missing');
function check(v){if(!v)throw new Error('oracle');}
var a={},b={},c={},rejectFirst,result='pending',parts=[];
var p=new Promise(function(resolve,reject){rejectFirst=reject;});
Promise.any([p,Promise.reject(b),Promise.reject(c)]).then(function(){parts.push('bad');},function(e){check(e instanceof AggregateError&&e.errors.length===3&&e.errors[0]===a&&e.errors[1]===b&&e.errors[2]===c);parts.push('ordered');});
Promise.any([]).then(function(){parts.push('bad');},function(e){check(e instanceof AggregateError&&e.errors.length===0);parts.push('empty');});rejectFirst(a);
Promise.resolve().then(function(){}).then(function(){}).then(function(){result=parts.sort().join(',');});
"###,
        r###"result"###,
        Value::String("empty,ordered".into()),
    );
}

#[test]
fn promise_any_generic_lookup_order_and_callback_return() {
    observe(
        "any_generic_lookup_order_and_callback_return",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
if(typeof Promise.any!=='function')throw new Error('Promise.any missing');
function check(v){if(!v)throw new Error('oracle');}
var t='',token={},ret={},holder={},captured,resolve=function(){},error,N=0;
function C(ex){t+='C';ex(resolve,function(e){t+='J';error=e;return ret;});return holder;}
Object.defineProperty(C,'resolve',{get:function(){t+='R';return function(v){check(this===C&&v===token);t+='Q';return {get then(){t+='T';return function(f,r){t+='H';captured=[f,r];};}};};}});
var it={};Object.defineProperty(it,'next',{get:function(){t+='N';return function(){t+='S';if(N++===0)return {get done(){t+='D';return false;},get value(){t+='V';return token;}};return {get done(){t+='D';return true;}};};}});
var items={};Object.defineProperty(items,Symbol.iterator,{get:function(){t+='I';return function(){t+='i';return it;};}});
check(Promise.any.call(C,items)===holder&&captured[0]===resolve);check(captured[1](token)===ret&&error.errors[0]===token);var result=t;
"###,
        r###"result"###,
        Value::String("CRIiNSDVQTHSDJ".into()),
    );
}

#[test]
fn promise_any_shared_fulfill_and_distinct_reject_gates() {
    observe(
        "any_shared_fulfill_and_distinct_reject_gates",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
if(typeof Promise.any!=='function')throw new Error('Promise.any missing');
function check(v){if(!v)throw new Error('oracle');}
var pairs=[],fcount=0,jcount=0,error,holder={},a={},b={},ret={};function resolve(){fcount++;return ret;}
function C(ex){ex(resolve,function(e){jcount++;error=e;return ret;});return holder;}C.resolve=function(){return {then:function(f,r){pairs.push([f,r]);}};};
check(Promise.any.call(C,[0,1])===holder);check(pairs[0][0]===resolve&&pairs[1][0]===resolve&&pairs[0][1]!==pairs[1][1]);
check(pairs[0][0](1)===ret&&pairs[1][0](2)===ret&&pairs[0][0](3)===ret);
check(pairs[0][1](a)===undefined&&pairs[0][1]({})===undefined&&pairs[1][1](b)===ret&&pairs[1][1]({})===undefined);
check(fcount===3&&jcount===1&&error.errors[0]===a&&error.errors[1]===b);var result='ok';
"###,
        r###"result"###,
        Value::String("ok".into()),
    );
}

#[test]
fn promise_any_reject_callback_metadata_and_post_throw_duplicate() {
    observe(
        "any_reject_callback_metadata_and_post_throw_duplicate",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
if(typeof Promise.any!=='function')throw new Error('Promise.any missing');
function check(v){if(!v)throw new Error('oracle');}
var saved,token={},calls=0;function C(ex){ex(function(){},function(){calls++;throw token;});}C.resolve=function(){return {then:function(f,r){saved=r;}};};Promise.any.call(C,[1]);
var n=Object.getOwnPropertyDescriptor(saved,'name'),l=Object.getOwnPropertyDescriptor(saved,'length');check(n.value===''&&!n.writable&&!n.enumerable&&n.configurable&&l.value===1&&!l.writable&&!l.enumerable&&l.configurable);
check(Object.getPrototypeOf(saved)===Function.prototype&&!Object.prototype.hasOwnProperty.call(saved,'prototype')&&Object.isExtensible(saved));
var type=false;try{new saved();}catch(e){type=e instanceof TypeError;}check(type&&calls===0);
var same=false;try{saved(1);}catch(e){same=e===token;}check(same&&calls===1&&saved(2)===undefined&&calls===1);var result='ok';
"###,
        r###"result"###,
        Value::String("ok".into()),
    );
}

#[test]
fn promise_any_empty_custom_reject_throw_is_rejected_again() {
    observe(
        "any_empty_custom_reject_throw_is_rejected_again",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
if(typeof Promise.any!=='function')throw new Error('Promise.any missing');
function check(v){if(!v)throw new Error('oracle');}
var token={},holder={},calls=0,first,second,closed=0;function C(ex){ex(function(){},function(e){calls++;if(calls===1){first=e;throw token;}second=e;return 9;});return holder;}C.resolve=function(v){return v;};
var it={next:function(){return {done:true};}};Object.defineProperty(it,'return',{get:function(){closed++;return function(){};}});var items={};items[Symbol.iterator]=function(){return it;};
check(Promise.any.call(C,items)===holder&&calls===2&&first instanceof AggregateError&&first.errors.length===0&&second===token&&closed===0);var result='ok';
"###,
        r###"result"###,
        Value::String("ok".into()),
    );
}

#[test]
fn promise_any_ordinary_abort_close_preserves_sentinel_and_reason() {
    observe(
        "any_ordinary_abort_close_preserves_sentinel_and_reason",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
if(typeof Promise.any!=='function')throw new Error('Promise.any missing');
function check(v){if(!v)throw new Error('oracle');}
var token={},other={},saved,seen=[],count=0,closed=0;function C(ex){ex(function(){},function(e){seen.push(e);});}C.resolve=function(v){if(v===1)throw token;return {then:function(f,r){saved=r;}};};
var it={next:function(){return {done:false,value:count++};}};Object.defineProperty(it,'return',{get:function(){closed++;throw other;}});var items={};items[Symbol.iterator]=function(){return it;};
Promise.any.call(C,items);check(count===2&&closed===1&&seen.length===1&&seen[0]===token);check(saved(other)===undefined&&saved(token)===undefined&&seen.length===1);var result='ok';
"###,
        r###"result"###,
        Value::String("ok".into()),
    );
}

#[test]
fn promise_any_iterator_step_abrupt_does_not_close() {
    observe(
        "any_iterator_step_abrupt_does_not_close",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
if(typeof Promise.any!=='function')throw new Error('Promise.any missing');
function check(v){if(!v)throw new Error('oracle');}
var rows=[];
for(var stage=0;stage<3;stage++){
 var token={},seen,closed=0,it={next:function(){if(stage===0)throw token;return {get done(){if(stage===1)throw token;return false;},get value(){throw token;}};}};
 Object.defineProperty(it,'return',{get:function(){closed++;throw new Error('close');}});var items={};items[Symbol.iterator]=function(){return it;};function C(ex){ex(function(){},function(e){seen=e;});}C.resolve=function(){throw new Error('resolve');};
 Promise.any.call(C,items);check(seen===token&&closed===0);rows.push('0');
}
var result=rows.join(',');
"###,
        r###"result"###,
        Value::String("0,0,0".into()),
    );
}

#[test]
fn promise_any_resolve_getter_failure_precedes_iterator() {
    observe(
        "any_resolve_getter_failure_precedes_iterator",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
if(typeof Promise.any!=='function')throw new Error('Promise.any missing');
function check(v){if(!v)throw new Error('oracle');}
var token={},seen,hit=0;function C(ex){ex(function(){},function(e){seen=e;});}Object.defineProperty(C,'resolve',{get:function(){throw token;}});
var items={};Object.defineProperty(items,Symbol.iterator,{get:function(){hit++;throw new Error('iterator');}});Promise.any.call(C,items);check(seen===token&&hit===0);var result='ok';
"###,
        r###"result"###,
        Value::String("ok".into()),
    );
}

#[test]
fn promise_any_intrinsic_error_ignores_global_replacements() {
    observe(
        "any_intrinsic_error_ignores_global_replacements",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
if(typeof Promise.any!=='function')throw new Error('Promise.any missing');
function check(v){if(!v)throw new Error('oracle');}
var A=AggregateError,Arr=Array,O=Object,proto=A.prototype,a={},b={},r0,r1,result='pending',setter=0;
var p0=new Promise(function(f,r){r0=r;}),p1=new Promise(function(f,r){r1=r;});
Promise.any([p0,p1]).then(function(){result='bad';},function(e){var d=O.getOwnPropertyDescriptor(e,'errors');check(O.getPrototypeOf(e)===proto&&Arr.isArray(e.errors)&&O.getPrototypeOf(e.errors)===Arr.prototype&&e.errors[0]===a&&e.errors[1]===b&&d.writable&&!d.enumerable&&d.configurable&&setter===0);result='ok';});
var descriptor=O.create(null);descriptor.set=function(){setter++;};descriptor.configurable=true;O.defineProperty(proto,'errors',descriptor);
AggregateError=function(){throw new Error('global AggregateError');};Array=function(){throw new Error('global Array');};r1(b);r0(a);
"###,
        r###"result"###,
        Value::String("ok".into()),
    );
}

#[test]
fn promise_any_synchronous_rejections_wait_for_exhaustion() {
    observe(
        "any_synchronous_rejections_wait_for_exhaustion",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
if(typeof Promise.any!=='function')throw new Error('Promise.any missing');
function check(v){if(!v)throw new Error('oracle');}
var trace='',error,count=0,holder={};function C(ex){ex(function(){},function(e){trace+='J';error=e;count++;return 7;});return holder;}
C.resolve=function(v){trace+='Q'+v;return {then:function(f,r){check(r(v)===undefined&&r(99)===undefined);trace+='H'+v;}};};
check(Promise.any.call(C,[0,1])===holder&&count===1&&error.errors[0]===0&&error.errors[1]===1);var result=trace;
"###,
        r###"result"###,
        Value::String("Q0H0Q1H1J".into()),
    );
}

#[test]
fn promise_any_synchronous_fulfillment_does_not_stop_iteration() {
    observe(
        "any_synchronous_fulfillment_does_not_stop_iteration",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
if(typeof Promise.any!=='function')throw new Error('Promise.any missing');
function check(v){if(!v)throw new Error('oracle');}
var fcount=0,jcount=0,steps=0,holder={};function C(ex){ex(function(){fcount++;},function(){jcount++;});return holder;}C.resolve=function(v){return {then:function(f,r){f(v);f(v);}};};
var it={next:function(){steps++;return steps<=2?{done:false,value:steps}:{done:true};}},items={};items[Symbol.iterator]=function(){return it;};
check(Promise.any.call(C,items)===holder);var result=fcount+':'+steps+':'+jcount;
"###,
        r###"result"###,
        Value::String("4:3:0".into()),
    );
}

#[test]
fn promise_any_cached_resolve_next_receivers_and_species() {
    observe(
        "any_cached_resolve_next_receivers_and_species",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
if(typeof Promise.any!=='function')throw new Error('Promise.any missing');
function check(v){if(!v)throw new Error('oracle');}
var trace='',bad=false,values,holder={},index=0;
function sharedResolve(v){trace+='F'+v;}
function C(ex){trace+='C';ex(sharedResolve,function(e){'use strict';bad=bad||this!==undefined||arguments.length!==1;trace+='J';values=e.errors;});return holder;}
Object.defineProperty(C,Symbol.species,{get:function(){throw new Error('species');}});
Object.defineProperty(C,'resolve',{configurable:true,get:function(){trace+='R';return function(v){'use strict';bad=bad||this!==C||arguments.length!==1;trace+='Q'+v;Object.defineProperty(C,'resolve',{value:function(){throw new Error('uncached resolve');},configurable:true});var returned={then:function(f,r){'use strict';bad=bad||this!==returned||arguments.length!==2||f!==sharedResolve;trace+='T'+v;check(r(v)===undefined);}};return returned;};}});
var it={};Object.defineProperty(it,'next',{configurable:true,get:function(){trace+='N';return function(){'use strict';bad=bad||this!==it||arguments.length!==0;trace+='S';Object.defineProperty(it,'next',{value:function(){throw new Error('uncached next');},configurable:true});return index<2?{done:false,value:++index}:{done:true,get value(){throw new Error('exhausted value');}};};}});
var items={};Object.defineProperty(items,Symbol.iterator,{get:function(){trace+='I';return function(){'use strict';bad=bad||this!==items||arguments.length!==0;trace+='i';return it;};}});
check(Promise.any.call(C,items)===holder);var result=trace+'|'+values.join(',')+'|'+bad;
"###,
        r###"result"###,
        Value::String("CRIiNSQ1T1SQ2T2SJ|1,2|false".into()),
    );
}

#[test]
fn promise_any_then_abrupt_close_preserves_original_throw() {
    observe(
        "any_then_abrupt_close_preserves_original_throw",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
if(typeof Promise.any!=='function')throw new Error('Promise.any missing');
function check(v){if(!v)throw new Error('oracle');}
var rows=[];
for(var stage=0;stage<2;stage++)for(var closeKind=0;closeKind<4;closeKind++){
 var marker={},other={},seen,gets=0,calls=0,rejects=0;
 function C(ex){ex(function(){throw new Error('fulfill');},function(e){seen=e;rejects++;});}
 C.resolve=function(){return {get then(){if(stage===0)throw marker;return function(){throw marker;};}};};
 var it={next:function(){return {done:false,value:1};}};
 Object.defineProperty(it,'return',{get:function(){gets++;if(closeKind===0)throw other;if(closeKind===1)return 1;return function(){calls++;if(closeKind===2)throw other;return 0;};}});
 var items={};items[Symbol.iterator]=function(){return it;};
 Promise.any.call(C,items);check(seen===marker&&rejects===1);rows.push(''+stage+closeKind+':'+gets+':'+calls);
}
var result=rows.join(',');
"###,
        r###"result"###,
        Value::String("00:1:0,01:1:0,02:1:1,03:1:1,10:1:0,11:1:0,12:1:1,13:1:1".into()),
    );
}

#[test]
fn promise_any_empty_and_nonempty_checkpoint_order_and_brand() {
    observe(
        "any_empty_and_nonempty_checkpoint_order_and_brand",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
if(typeof Promise.any!=='function')throw new Error('Promise.any missing');
function check(v){if(!v)throw new Error('oracle');}
var trace='',A=AggregateError,O=Object;
Promise.any([]).then(function(){throw new Error('empty fulfill');},function(e){var d=O.getOwnPropertyDescriptor(e,'errors');check(O.getPrototypeOf(e)===A.prototype&&O.prototype.toString.call(e)==='[object Error]'&&Array.isArray(e.errors)&&e.errors.length===0&&d.value===e.errors&&d.writable&&!d.enumerable&&d.configurable);trace+='E';});
Promise.any([Promise.reject('no'),Promise.resolve('x')]).then(function(v){check(v==='x');trace+='F'+v;},function(){throw new Error('nonempty reject');});
queueMicrotask(function(){trace+='Q';});trace+='S';
"###,
        r###"trace"###,
        Value::String("SEQFx".into()),
    );
}

#[test]
fn promise_any_exhaustion_custom_reject_second_throw_escapes_without_close() {
    observe(
        "any_exhaustion_custom_reject_second_throw_escapes_without_close",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
if(typeof Promise.any!=='function')throw new Error('Promise.any missing');
function check(v){if(!v)throw new Error('oracle');}
var rows=[];
for(var length=0;length<2;length++){
 var first,second,caught,one={},two={},payload={},calls=0,steps=0,closed=0,holder={};
 function C(ex){ex(function(){throw new Error('fulfill');},function(e){calls++;if(calls===1){first=e;throw one;}second=e;throw two;});return holder;}
 C.resolve=function(v){return {then:function(f,r){check(r(v)===undefined);}};};
 var it={next:function(){steps++;return length===1&&steps===1?{done:false,value:payload}:{done:true};}};
 Object.defineProperty(it,'return',{get:function(){closed++;throw new Error('close after done');}});var items={};items[Symbol.iterator]=function(){return it;};
 try{Promise.any.call(C,items);}catch(e){caught=e;}
 check(calls===2&&first instanceof AggregateError&&first.errors.length===length&&(length===0||first.errors[0]===payload)&&second===one&&caught===two&&closed===0);
 rows.push(length+':'+calls+':'+steps+':'+closed);
}
var result=rows.join(',');
"###,
        r###"result"###,
        Value::String("0:2:1:0,1:2:2:0".into()),
    );
}
