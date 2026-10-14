//! Promise.prototype.finally literals frozen before implementation or execution.
use super::*;

fn observe(script: &str, expression: &str, expected: Value, entry: Option<Value>) {
    let mut runtime = Runtime::try_new().unwrap();
    let mut document = Document::parse("");
    let completion = runtime.execute(script, &mut document).unwrap();
    if let Some(expected) = entry {
        assert_eq!(completion, expected);
    }
    assert_eq!(
        runtime.execute(expression, &mut document).unwrap(),
        expected
    );
    assert!(runtime.console.is_empty(), "{:?}", runtime.console);
}

#[test]
fn promise_finally_object_requirement_precedes_observable_metadata() {
    observe(
        r###"var hits=0,bad=0;var saved=Promise.prototype.finally;Object.defineProperty(Object.prototype,'constructor',{configurable:true,get:function(){hits++;throw 9;}});var values=[undefined,null,7,true,'x',Symbol('x')];for(var i=0;i<values.length;i++){try{saved.call(values[i],17);}catch(e){if(e instanceof TypeError)bad++;}}delete Object.prototype.constructor;"###,
        r###"typeof saved+'|'+bad+'|'+hits"###,
        Value::String("function|6|0".into()),
        None,
    );
}

#[test]
fn promise_finally_generic_species_then_order_and_noncallable_identity() {
    observe(
        r###"var log='',raw={},marker={},receiver={},bag={};Object.defineProperty(bag,Symbol.species,{get:function(){log+='S';return Promise;}});Object.defineProperty(receiver,'constructor',{get:function(){log+='C';return bag;}});Object.defineProperty(receiver,'then',{get:function(){log+='T';return function(a,b){'use strict';log+=this===receiver&&arguments.length===2&&a===raw&&b===raw?'I':'!';return marker;};}});var same=Promise.prototype.finally.call(receiver,raw,'ignored')===marker;"###,
        r###"log+'|'+same"###,
        Value::String("CSTI|true".into()),
        None,
    );
}

#[test]
fn promise_finally_species_abrupt_precedes_then_and_then_abrupt_is_exact() {
    observe(
        r###"var log='',exact=0,bad=0,poison=0,token={};Object.defineProperty(token,'constructor',{get:function(){poison++;throw 91;}});var saved=Promise.prototype.finally;var a={};Object.defineProperty(a,'constructor',{get:function(){log+='C';throw token;}});Object.defineProperty(a,'then',{get:function(){log+='!';throw 1;}});try{saved.call(a,0);}catch(e){if(e===token)exact++;}var bag={};Object.defineProperty(bag,Symbol.species,{get:function(){log+='S';throw token;}});try{saved.call({constructor:bag,then:function(){log+='!';}},0);}catch(e){if(e===token)exact++;}try{saved.call({constructor:7,then:function(){log+='!';}},0);}catch(e){if(e instanceof TypeError)bad++;}var wrong={};wrong[Symbol.species]={};try{saved.call({constructor:wrong,then:function(){log+='!';}},0);}catch(e){if(e instanceof TypeError)bad++;}var b={constructor:undefined};Object.defineProperty(b,'then',{get:function(){log+='G';throw token;}});try{saved.call(b,0);}catch(e){if(e===token)exact++;}try{saved.call({constructor:undefined,then:function(){log+='I';throw token;}},0);}catch(e){if(e===token)exact++;}"###,
        r###"log+'|'+exact+'|'+bad+'|'+poison"###,
        Value::String("CSGI|4|2|0".into()),
        None,
    );
}

#[test]
fn promise_finally_reusable_wrappers_thunks_capture_constructor_and_exact_values() {
    observe(
        r###"var log='',pairs=[],thunks=[],seen=[],marker={},cleanup={},value={},reason={},strictOK=true;function C(executor){log+='N';executor(function(v){log+='R';seen.push(v);},function(){});Object.defineProperty(this,'then',{get:function(){log+='T';return function(thunk){'use strict';log+='I';thunks.push(thunk);return marker;};}});}Object.defineProperty(C,'resolve',{get:function(){log+='!';throw 1;}});var bag={};Object.defineProperty(bag,Symbol.species,{configurable:true,get:function(){log+='S';return C;}});var receiver={constructor:bag};Object.defineProperty(receiver,'then',{get:function(){log+='G';return function(a,b){log+='O';pairs.push(a);pairs.push(b);return marker;};}});var outer=Promise.prototype.finally.call(receiver,function(){'use strict';strictOK=strictOK&&this===undefined&&arguments.length===0;log+='F';return cleanup;});Object.defineProperty(bag,Symbol.species,{value:function(){throw 8;},configurable:true});var first=pairs[0].call({wrong:true},value,99)===marker;var second=pairs[1].call(null,reason,99)===marker;var v=thunks[0].call({wrong:true},99)===value;var r=false;try{thunks[1].call({wrong:true},99);}catch(e){r=e===reason;}var repeat=pairs[0](value)===marker;var captured=seen.length===3&&seen[0]===cleanup&&seen[1]===cleanup&&seen[2]===cleanup;"###,
        r###"log+'|'+(outer===marker&&first&&second&&v&&r&&repeat&&captured&&strictOK)"###,
        Value::String("SGOFNRTIFNRTIFNRTI|true".into()),
        None,
    );
}

#[test]
fn promise_finally_public_and_generated_function_metadata_identity_and_nonconstruction() {
    observe(
        r###"var saved=Promise.prototype.finally;var d=Object.getOwnPropertyDescriptor(Promise.prototype,'finally'),n=Object.getOwnPropertyDescriptor(saved,'name'),l=Object.getOwnPropertyDescriptor(saved,'length');var wrappers=[],thunks=[];function C(executor){executor(function(){},function(){});this.then=function(thunk){thunks.push(thunk);return 3;};}var bag={};bag[Symbol.species]=C;var receiver={constructor:bag,then:function(a,b){wrappers.push(a);wrappers.push(b);return 4;}};function cleanup(){}saved.call(receiver,cleanup);saved.call(receiver,cleanup);wrappers[0](7);wrappers[1](9);var ok=saved.name==='finally'&&saved.length===1&&d.value===saved&&d.writable&&!d.enumerable&&d.configurable&&!n.writable&&!n.enumerable&&n.configurable&&!l.writable&&!l.enumerable&&l.configurable&&Object.getPrototypeOf(saved)===Function.prototype&&!Object.prototype.hasOwnProperty.call(saved,'prototype')&&wrappers[0]!==wrappers[1]&&wrappers[0]!==wrappers[2]&&wrappers[1]!==wrappers[3]&&thunks[0]!==thunks[1];var all=[saved,wrappers[0],wrappers[1],thunks[0],thunks[1]],bad=0;for(var i=0;i<all.length;i++){var f=all[i];if(i>0){ok=ok&&f.name===''&&f.length===(i<3?1:0)&&Object.getPrototypeOf(f)===Function.prototype&&!Object.prototype.hasOwnProperty.call(f,'prototype');}try{new f();}catch(e){if(e instanceof TypeError)bad++;}}delete Promise.prototype.finally;ok=ok&&!Object.prototype.hasOwnProperty.call(Promise.prototype,'finally')&&saved.call({then:function(a,b){return a===17&&b===17;}},17);"###,
        r###"ok&&bad===5"###,
        Value::Bool(true),
        None,
    );
}

#[test]
fn promise_finally_genuine_fulfillment_waits_and_preserves_original_value() {
    observe(
        r###"var log='';Promise.resolve(7).finally(function(){'use strict';log+=this===undefined&&arguments.length===0?'F':'!';return 99;}).then(function(v){log+='V'+v;});queueMicrotask(function(){log+='Q';});log+='S';log;"###,
        r###"log"###,
        Value::String("SFQV7".into()),
        Some(Value::String("S".into())),
    );
}

#[test]
fn promise_finally_genuine_rejection_preserves_reason_and_cleanup_throw_replaces_it() {
    observe(
        r###"var log='',original={},replacement={},poison=0;Object.defineProperty(original,'constructor',{get:function(){poison++;throw 91;}});Object.defineProperty(replacement,'constructor',{get:function(){poison++;throw 92;}});Promise.reject(original).finally(function(){log+='F';return 99;}).then(function(){log+='!';},function(e){log+=e===original?'R':'!';});Promise.reject(original).finally(function(){log+='T';throw replacement;}).then(function(){log+='!';},function(e){log+=e===replacement?'E':'!';});queueMicrotask(function(){log+='Q';});log+='S';"###,
        r###"log+'|'+poison"###,
        Value::String("SFTQER|0".into()),
        None,
    );
}

#[test]
fn promise_finally_thenable_cleanup_waits_with_shared_fifo_phase() {
    observe(
        r###"var log='',cleanup={};Object.defineProperty(cleanup,'then',{get:function(){log+='G';return function(resolve){log+='T';queueMicrotask(function(){log+='M';resolve(99);});};}});Promise.resolve(7).finally(function(){log+='F';return cleanup;}).then(function(v){log+='V'+v;});queueMicrotask(function(){log+='Q';});log+='S';log;"###,
        r###"log"###,
        Value::String("SFGQTMV7".into()),
        Some(Value::String("S".into())),
    );
}

#[test]
fn promise_finally_species_is_reobserved_by_then_but_finalizer_retains_first_constructor() {
    observe(
        r###"var log='',reads=0,P=Promise;function A(executor){log+='A';return new P(executor);}function B(executor){log+='B';return new P(executor);}var bag={};Object.defineProperty(bag,Symbol.species,{get:function(){reads++;log+=reads;return reads===1?A:B;}});var p=P.resolve(7);Object.defineProperty(p,'constructor',{get:function(){log+='C';return bag;}});p.finally(function(){log+='F';return 1;}).then(function(v){log+='V'+v;});queueMicrotask(function(){log+='Q';});log+='S';"###,
        r###"log"###,
        Value::String("C1C2BSFAQV7".into()),
        None,
    );
}

#[test]
fn promise_finally_intrinsic_resolve_ignores_replaced_global_and_resolve_property() {
    observe(
        r###"var log='',P=Promise,p=P.resolve(7);p.constructor=undefined;P.resolve=function(){log+='!';throw 1;};globalThis.Promise=function(){log+='!';throw 2;};p.finally(function(){log+='F';return 1;}).then(function(v){log+='V'+v;});queueMicrotask(function(){log+='Q';});log+='S';"###,
        r###"log"###,
        Value::String("SFQV7".into()),
        None,
    );
}

#[test]
fn promise_finally_noncallable_finalizer_passes_through_without_own_calls() {
    observe(
        r###"var log='',poison=0,raw={};Object.defineProperty(raw,'then',{get:function(){poison++;throw 1;}});Promise.resolve(7).finally(raw).then(function(v){log+='V'+v;});queueMicrotask(function(){log+='Q';});log+='S';"###,
        r###"log+'|'+poison"###,
        Value::String("SQV7|0".into()),
        None,
    );
}

#[test]
fn promise_finally_cleanup_abrupt_values_replace_both_fulfillment_and_rejection() {
    observe(
        r###"var token={},original={},poison=0,ok=false;Object.defineProperty(token,'constructor',{get:function(){poison++;throw 91;}});var p=Promise.resolve(7);p.finally(function(){throw token;}).then(function(){ok=false;},function(e){ok=e===token;});"###,
        r###"ok&&poison===0"###,
        Value::Bool(true),
        None,
    );
    observe(
        r###"var token={},original={},poison=0,ok=false;Object.defineProperty(token,'constructor',{get:function(){poison++;throw 91;}});var replacement={};Object.defineProperty(replacement,'then',{get:function(){throw token;}});var p=Promise.resolve(7);p.finally(function(){return replacement;}).then(function(){ok=false;},function(e){ok=e===token;});"###,
        r###"ok&&poison===0"###,
        Value::Bool(true),
        None,
    );
    observe(
        r###"var token={},original={},poison=0,ok=false;Object.defineProperty(token,'constructor',{get:function(){poison++;throw 91;}});var p=Promise.reject(original);p.finally(function(){return new Promise(function(resolve,reject){queueMicrotask(function(){reject(token);});});}).then(function(){ok=false;},function(e){ok=e===token;});"###,
        r###"ok&&poison===0"###,
        Value::Bool(true),
        None,
    );
    observe(
        r###"var token={},original={},poison=0,ok=false;Object.defineProperty(token,'constructor',{get:function(){poison++;throw 91;}});var replacement=Promise.resolve(9);Object.defineProperty(replacement,'constructor',{get:function(){throw token;}});var p=Promise.resolve(7);p.finally(function(){return replacement;}).then(function(){ok=false;},function(e){ok=e===token;});"###,
        r###"ok&&poison===0"###,
        Value::Bool(true),
        None,
    );
}

#[test]
fn promise_finally_original_outcomes_use_same_value_after_successful_cleanup() {
    observe(
        r###"var original=undefined,ok=false;Promise.resolve(original).finally(function(){return 99;}).then(function(v){ok=Object.is(v,original);},function(){ok=false;});"###,
        r###"ok"###,
        Value::Bool(true),
        None,
    );
    observe(
        r###"var original=undefined,ok=false;Promise.reject(original).finally(function(){return 99;}).then(function(){ok=false;},function(e){ok=Object.is(e,original);});"###,
        r###"ok"###,
        Value::Bool(true),
        None,
    );
    observe(
        r###"var original=null,ok=false;Promise.resolve(original).finally(function(){return 99;}).then(function(v){ok=Object.is(v,original);},function(){ok=false;});"###,
        r###"ok"###,
        Value::Bool(true),
        None,
    );
    observe(
        r###"var original=null,ok=false;Promise.reject(original).finally(function(){return 99;}).then(function(){ok=false;},function(e){ok=Object.is(e,original);});"###,
        r###"ok"###,
        Value::Bool(true),
        None,
    );
    observe(
        r###"var original=NaN,ok=false;Promise.resolve(original).finally(function(){return 99;}).then(function(v){ok=Object.is(v,original);},function(){ok=false;});"###,
        r###"ok"###,
        Value::Bool(true),
        None,
    );
    observe(
        r###"var original=NaN,ok=false;Promise.reject(original).finally(function(){return 99;}).then(function(){ok=false;},function(e){ok=Object.is(e,original);});"###,
        r###"ok"###,
        Value::Bool(true),
        None,
    );
    observe(
        r###"var original=-0,ok=false;Promise.resolve(original).finally(function(){return 99;}).then(function(v){ok=Object.is(v,original);},function(){ok=false;});"###,
        r###"ok"###,
        Value::Bool(true),
        None,
    );
    observe(
        r###"var original=-0,ok=false;Promise.reject(original).finally(function(){return 99;}).then(function(){ok=false;},function(e){ok=Object.is(e,original);});"###,
        r###"ok"###,
        Value::Bool(true),
        None,
    );
    observe(
        r###"var original=Symbol('outcome'),ok=false;Promise.resolve(original).finally(function(){return 99;}).then(function(v){ok=Object.is(v,original);},function(){ok=false;});"###,
        r###"ok"###,
        Value::Bool(true),
        None,
    );
    observe(
        r###"var original=Symbol('outcome'),ok=false;Promise.reject(original).finally(function(){return 99;}).then(function(){ok=false;},function(e){ok=Object.is(e,original);});"###,
        r###"ok"###,
        Value::Bool(true),
        None,
    );
    observe(
        r###"var original={},ok=false;Promise.resolve(original).finally(function(){return 99;}).then(function(v){ok=Object.is(v,original);},function(){ok=false;});"###,
        r###"ok"###,
        Value::Bool(true),
        None,
    );
    observe(
        r###"var original={},ok=false;Promise.reject(original).finally(function(){return 99;}).then(function(){ok=false;},function(e){ok=Object.is(e,original);});"###,
        r###"ok"###,
        Value::Bool(true),
        None,
    );
}

#[test]
fn promise_finally_generic_noncallable_and_omitted_values_are_forwarded_without_conversion() {
    observe(
        r###"var saved=Promise.prototype.finally,poison=0,seen=0,same=0,expected,raw={};Object.defineProperty(raw,'then',{get:function(){poison++;throw 91;}});var symbol=Symbol('raw'),values=[undefined,null,7,raw,symbol];var receiver={constructor:undefined,then:function(a,b){'use strict';if(this===receiver&&arguments.length===2&&a===expected&&b===expected)seen++;return a;}};for(var i=0;i<values.length;i++){expected=values[i];if(saved.call(receiver,expected,'ignored')===expected)same++;}expected=undefined;if(saved.call(receiver)===undefined)same++;"###,
        r###"seen+'|'+same+'|'+poison"###,
        Value::String("6|6|0".into()),
        None,
    );
}

#[test]
fn promise_finally_fatal_finalizer_keeps_fifo_suspended_and_fresh_realm_independent() {
    let mut runtime = Runtime::try_new().unwrap();
    let mut document = Document::parse("");
    let error = runtime.execute(
        r###"var log='';Promise.resolve(1).finally(function(){log+='F';while(true){};});queueMicrotask(function(){log+='!';});"###,
        &mut document,
    ).unwrap_err();
    assert!(error.is_resource_limit());
    assert!(runtime.microtasks_suspended());
    assert_eq!(
        runtime.execute("log", &mut document).unwrap(),
        Value::String("F".into())
    );
    assert_eq!(
        runtime.execute("log+='S';log;", &mut document).unwrap(),
        Value::String("FS".into())
    );
    assert!(runtime.microtasks_suspended());
    assert!(runtime.console.is_empty());
    observe(
        r###"var log='';Promise.resolve(1).finally(function(){log+='F';}).then(function(){log+='R';});"###,
        "log",
        Value::String("FR".into()),
        None,
    );
}
