//! Promise.prototype.catch literals frozen before implementation or execution.
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
fn promise_catch_generic_getter_receiver_identity_and_raw_return() {
    observe(
        r###"var log='',touched=0;var reject={};var result={};Object.defineProperty(reject,'then',{get:function(){touched++;throw 1;}});Object.defineProperty(result,'then',{get:function(){touched++;throw 2;}});var receiver={};Object.defineProperty(receiver,'constructor',{get:function(){touched++;throw 3;}});Object.defineProperty(receiver,'then',{get:function(){log+=this===receiver?'G':'!';return function(a,b){'use strict';log+=this===receiver&&arguments.length===2&&a===undefined&&b===reject?'T':'!';return result;};}});var same=Promise.prototype.catch.call(receiver,reject)===result;log+='S';"###,
        r###"log+'|'+same+'|'+touched"###,
        Value::String("GTS|true|0".into()),
        None,
    );
}

#[test]
fn promise_catch_primitives_use_getv_and_preserve_receiver() {
    observe(
        r###"var log='',calls=0;var symbol=Symbol('r'),expected;Object.defineProperty(Object.prototype,'then',{configurable:true,get:function(){'use strict';log+='G'+typeof this;return function(a,b){'use strict';log+='T'+typeof this;if(this===expected&&arguments.length===2&&a===undefined&&b===17)calls++;return this;};}});var values=[7,true,'s',symbol];var same=0;for(var i=0;i<values.length;i++){expected=values[i];if(Promise.prototype.catch.call(expected,17)===expected)same++;}var bad=0;try{Promise.prototype.catch.call(null,17);}catch(e){if(e instanceof TypeError)bad++;}try{Promise.prototype.catch.call(undefined,17);}catch(e){if(e instanceof TypeError)bad++;}delete Object.prototype.then;"###,
        r###"log+'|'+calls+'|'+same+'|'+bad"###,
        Value::String("GnumberTnumberGbooleanTbooleanGstringTstringGsymbolTsymbol|4|4|2".into()),
        None,
    );
}

#[test]
fn promise_catch_forwards_unfiltered_arguments_and_ignores_extras() {
    observe(
        r###"var calls=0,poison=0;var object={};Object.defineProperty(object,'constructor',{get:function(){poison++;throw 9;}});var fn=function(){poison++;};var values=[undefined,null,7,object,Symbol('x'),fn];var expected;var receiver={then:function(a,b){'use strict';if(this===receiver&&arguments.length===2&&a===undefined&&b===expected)calls++;return b;}};var same=0;for(var i=0;i<values.length;i++){expected=values[i];if(Promise.prototype.catch.call(receiver,expected,'ignored')===expected)same++;}expected=undefined;if(Promise.prototype.catch.call(receiver)===undefined)same++;"###,
        r###"calls+'|'+same+'|'+poison"###,
        Value::String("7|7|0".into()),
        None,
    );
    observe(
        r###"var receiver={then:function(a,b){'use strict';return this===receiver&&arguments.length===2&&a===undefined&&b===3;}};var saved=Promise.prototype.catch;var bound=saved.bind(receiver,3);var answer=bound.call({wrong:true});"###,
        r###"answer"###,
        Value::Bool(true),
        None,
    );
}

#[test]
fn promise_catch_get_and_call_abrupt_completions_are_exact() {
    observe(
        r###"var log='',poison=0,token={};Object.defineProperty(token,'constructor',{get:function(){poison++;throw 91;}});var fromGet={},fromCall={then:function(){log+='C';throw token;}};Object.defineProperty(fromGet,'then',{get:function(){log+='G';throw token;}});var exact=0;try{Promise.prototype.catch.call(fromGet,0);}catch(e){if(e===token)exact++;}try{Promise.prototype.catch.call(fromCall,0);}catch(e){if(e===token)exact++;}var type=0;try{Promise.prototype.catch.call({then:7},0);}catch(e){if(e instanceof TypeError)type++;}try{Promise.prototype.catch.call({},0);}catch(e){if(e instanceof TypeError)type++;}"###,
        r###"log+'|'+exact+'|'+type+'|'+poison"###,
        Value::String("GC|2|2|0".into()),
        None,
    );
    observe(
        r###"var symbol=Symbol('reason'),values=[undefined,null,NaN,-0,symbol],exact=0,reason;var receiver={then:function(){throw reason;}};for(var i=0;i<values.length;i++){reason=values[i];try{Promise.prototype.catch.call(receiver,0);}catch(e){if(Object.is(e,reason))exact++;}}"###,
        r###"exact"###,
        Value::Number(5.0),
        None,
    );
}

#[test]
fn promise_catch_genuine_reactions_share_fifo_and_propagate() {
    observe(
        r###"var log='',token={};queueMicrotask(function(){log+='Q';});var p=Promise.reject(token);var child=p.catch(function(e){'use strict';log+=this===undefined&&arguments.length===1&&e===token?'R5':'!';return 6;});var distinct=child!==p;child.then(function(v){log+='D'+v;});log+='S';log;"###,
        r###"log+'|'+distinct"###,
        Value::String("SQR5D6|true".into()),
        Some(Value::String("S".into())),
    );
    observe(
        r###"var token={},a=false,b=false;Promise.resolve(7).catch(function(){throw 1;}).then(function(v){a=v===7;});Promise.reject(token).catch({bad:true}).catch(function(e){b=e===token;});"###,
        r###"a&&b"###,
        Value::Bool(true),
        None,
    );
}

#[test]
fn promise_catch_uses_observed_then_once_without_private_brand_or_species() {
    observe(
        r###"var log='',token={},p=Promise.resolve(1);Object.defineProperty(p,'constructor',{get:function(){log+='X';throw 1;}});Object.defineProperty(p,'then',{configurable:true,get:function(){log+='G';Object.defineProperty(p,'then',{value:function(){log+='U';return token;},configurable:true});queueMicrotask(function(){log+='M';});return function(a,b){'use strict';log+=this===p&&a===undefined&&b===token?'T':'!';return token;};}});var same=Promise.prototype.catch.call(p,token)===token;var again=Promise.prototype.catch.call(p,token)===token;log+='S';"###,
        r###"log+'|'+same+'|'+again"###,
        Value::String("GTUSM|true|true".into()),
        None,
    );
}

#[test]
fn promise_catch_reflection_nonconstruction_and_saved_identity() {
    observe(
        r###"var fn=Promise.prototype.catch,p=Object.getOwnPropertyDescriptor(Promise.prototype,'catch'),n=Object.getOwnPropertyDescriptor(fn,'name'),l=Object.getOwnPropertyDescriptor(fn,'length');var flags=p.value===fn&&p.writable&&!p.enumerable&&p.configurable&&n.value==='catch'&&!n.writable&&!n.enumerable&&n.configurable&&l.value===1&&!l.writable&&!l.enumerable&&l.configurable&&Object.getOwnPropertyNames(fn).join(',')==='length,name'&&Object.getPrototypeOf(fn)===Function.prototype&&!Object.hasOwn(fn,'prototype');var errors=0;try{new fn();}catch(e){if(e instanceof TypeError)errors++;}try{Reflect.construct(fn,[]);}catch(e){if(e instanceof TypeError)errors++;}var receiver={then:function(a,b){return b;}};delete Promise.prototype.catch;var saved=fn.call(receiver,9)===9;Promise.prototype.catch=fn;"###,
        r###"flags&&errors===2&&saved&&Promise.prototype.catch===fn"###,
        Value::Bool(true),
        None,
    );
}
