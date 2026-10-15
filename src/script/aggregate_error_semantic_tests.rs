//! Frozen AggregateError literals and existing Error brand/tag controls.
//! Each mode uses a fresh Runtime and a separate post-execution observation.
use super::*;

fn observe(case_id: &str, source: &str, expression: &str, expected: &str) {
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
        assert_eq!(
            observed,
            Value::String(expected.into()),
            "{case_id} strict={strict}"
        );
    }
}

#[test]
fn aggregate_metadata_and_call_construct() {
    observe(
        "aggregate_metadata_and_call_construct",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
function check(v){if(!v)throw new Error('oracle');}
var A=AggregateError,O=Object;
function flags(o,k,v,w,e,c){var d=O.getOwnPropertyDescriptor(o,k);check(d.value===v&&d.writable===w&&d.enumerable===e&&d.configurable===c);}
flags(globalThis,'AggregateError',A,true,false,true);flags(A,'length',2,false,false,true);flags(A,'name','AggregateError',false,false,true);flags(A,'prototype',A.prototype,false,false,false);
check(O.getPrototypeOf(A)===Error&&O.getPrototypeOf(A.prototype)===Error.prototype);
flags(A.prototype,'constructor',A,true,false,true);flags(A.prototype,'name','AggregateError',true,false,true);flags(A.prototype,'message','',true,false,true);
var a=A([]),b=new A([]);check(a!==b&&a.errors!==b.errors&&a instanceof A&&a instanceof Error&&b instanceof A);check(!O.prototype.hasOwnProperty.call(A.prototype,'errors'));
var result='ok';
"###,
        r###"result"###,
        "ok",
    );
}

#[test]
fn aggregate_prototype_message_cause_iteration_order() {
    observe(
        "aggregate_prototype_message_cause_iteration_order",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
function check(v){if(!v)throw new Error('oracle');}
var t='',token={},cause={},proto={},N=(function(){}).bind(null);
Object.defineProperty(N,'prototype',{get:function(){t+='P';return proto;}});
var msg={};msg[Symbol.toPrimitive]=function(h){check(h==='string');t+='M';return 'text';};
var inherited={};Object.defineProperty(inherited,'cause',{get:function(){t+='C';return cause;}});var options=Object.create(inherited);
var count=0,it={};Object.defineProperty(it,'next',{get:function(){t+='N';return function(){t+='S';if(count++===0)return {get done(){t+='D';return false;},get value(){t+='V';return token;}};return {get done(){t+='D';return true;}};};}});
var items={};Object.defineProperty(items,Symbol.iterator,{get:function(){t+='I';return function(){t+='i';return it;};}});
var e=Reflect.construct(AggregateError,[items,msg,options],N);
check(Object.getPrototypeOf(e)===proto&&e.message==='text'&&e.cause===cause&&e.errors[0]===token);
check(Object.getOwnPropertyNames(e).join(',')==='message,cause,errors');var result=t;
"###,
        r###"result"###,
        "PMCIiNSDVSD",
    );
}

#[test]
fn aggregate_abrupt_precedence_preserves_payload() {
    observe(
        "aggregate_abrupt_precedence_preserves_payload",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
function check(v){if(!v)throw new Error('oracle');}
var rows=[];
for(var stage=0;stage<4;stage++){
 var t='',token={},N=(function(){}).bind(null);Object.defineProperty(N,'prototype',{get:function(){t+='P';if(stage===0)throw token;return {};}});
 var msg={toString:function(){t+='M';if(stage===1)throw token;return 'm';}};
 var opts={get cause(){t+='C';if(stage===2)throw token;return 1;}};
 var items={};Object.defineProperty(items,Symbol.iterator,{get:function(){t+='I';throw token;}});
 var same=false;try{Reflect.construct(AggregateError,[items,msg,opts],N);}catch(e){same=e===token;}check(same);rows.push(t);
}
var result=rows.join('|');
"###,
        r###"result"###,
        "P|PM|PMC|PMCI",
    );
}

#[test]
fn aggregate_options_presence_and_dense_errors() {
    observe(
        "aggregate_options_presence_and_dense_errors",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
function check(v){if(!v)throw new Error('oracle');}
var A=AggregateError,h=Object.prototype.hasOwnProperty,src=[,undefined],e=A(src,undefined,{cause:undefined});
check(!h.call(e,'message')&&h.call(e,'cause')&&e.cause===undefined&&e.errors!==src&&e.errors.length===2&&h.call(e.errors,'0')&&h.call(e.errors,'1'));
var d=Object.getOwnPropertyDescriptor(e,'errors');check(d.writable&&!d.enumerable&&d.configurable);
var c=Object.getOwnPropertyDescriptor(e,'cause');check(c.writable&&!c.enumerable&&c.configurable);
Object.defineProperty(String.prototype,'cause',{get:function(){throw new Error('primitive options inspected');},configurable:true});
check(!h.call(A([],undefined,'x'),'cause'));delete String.prototype.cause;
check(!h.call(A([],undefined,null),'cause')&&!h.call(A([],undefined,undefined),'cause'));
var result='ok';
"###,
        r###"result"###,
        "ok",
    );
}

#[test]
fn aggregate_iterator_abrupt_paths_do_not_close() {
    observe(
        "aggregate_iterator_abrupt_paths_do_not_close",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
function check(v){if(!v)throw new Error('oracle');}
var rows=[];
for(var stage=0;stage<4;stage++){
 var token={},closed=0,it={next:function(){if(stage===0)throw token;if(stage===3)return 1;return {get done(){if(stage===1)throw token;return false;},get value(){throw token;}};}};
 Object.defineProperty(it,'return',{get:function(){closed++;throw new Error('close');}});var items={};items[Symbol.iterator]=function(){return it;};
 var same=false;try{AggregateError(items);}catch(e){same=stage===3?e instanceof TypeError:e===token;}check(same&&closed===0);rows.push('0');
}
var result=rows.join(',');
"###,
        r###"result"###,
        "0,0,0,0",
    );
}

#[test]
fn aggregate_saved_constructor_and_intrinsic_array() {
    observe(
        "aggregate_saved_constructor_and_intrinsic_array",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
function check(v){if(!v)throw new Error('oracle');}
var A=AggregateError,O=Object,Arr=Array,p=A.prototype,a={},b={};
AggregateError=function(){throw new Error('global AggregateError');};Array=function(){throw new Error('global Array');};
var e=A([a,b]);check(O.getPrototypeOf(e)===p&&Arr.isArray(e.errors)&&O.getPrototypeOf(e.errors)===Arr.prototype&&e.errors[0]===a&&e.errors[1]===b);
var result='ok';
"###,
        r###"result"###,
        "ok",
    );
}

#[test]
fn aggregate_newtarget_fallback_and_validation() {
    observe(
        "aggregate_newtarget_fallback_and_validation",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
function check(v){if(!v)throw new Error('oracle');}
var A=AggregateError,N=(function(){}).bind(null),seen=0;Object.defineProperty(N,'prototype',{get:function(){seen++;return 1;}});
var e=Reflect.construct(A,[[]],N);check(seen===1&&Object.getPrototypeOf(e)===A.prototype);
var items={};Object.defineProperty(items,Symbol.iterator,{get:function(){throw new Error('must not inspect iterable');}});
var type=false;try{Reflect.construct(A,[items],()=>{});}catch(x){type=x instanceof TypeError;}check(type);var result='ok';
"###,
        r###"result"###,
        "ok",
    );
}

#[test]
fn aggregate_strings_and_noniterables() {
    observe(
        "aggregate_strings_and_noniterables",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
function check(v){if(!v)throw new Error('oracle');}
var e=AggregateError('A\uD83D\uDE00');check(e.errors.length===2&&e.errors[0]==='A'&&e.errors[1]==='\uD83D\uDE00');
var inputs=[undefined,null,1,false],n=0;for(var i=0;i<inputs.length;i++){try{AggregateError(inputs[i]);}catch(x){check(x instanceof TypeError);n++;}}
check(n===4);var result='ok';
"###,
        r###"result"###,
        "ok",
    );
}

#[test]
fn aggregate_own_data_bypasses_inherited_setters() {
    observe(
        "aggregate_own_data_bypasses_inherited_setters",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
function check(v){if(!v)throw new Error('oracle');}
var A=AggregateError,called=0,keys=['message','cause','errors'];
for(var i=0;i<keys.length;i++){var d=Object.create(null);d.set=function(){called++;};d.configurable=true;Object.defineProperty(A.prototype,keys[i],d);}
var payload={},cause={},e=A([payload],'m',{cause:cause});check(called===0&&e.message==='m'&&e.cause===cause&&e.errors[0]===payload);
check(Object.getOwnPropertyNames(e).join(',')==='message,cause,errors');var result='ok';
"###,
        r###"result"###,
        "ok",
    );
}

#[test]
fn aggregate_brand_and_message_conversion() {
    observe(
        "aggregate_brand_and_message_conversion",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
function check(v){if(!v)throw new Error('oracle');}
var A=AggregateError,tag=Object.prototype.toString,e=A([]),fake=Object.create(A.prototype);
check(tag.call(e)==='[object Error]'&&tag.call(fake)==='[object Object]'&&tag.call(A.prototype)==='[object Object]');
Object.setPrototypeOf(e,null);check(tag.call(e)==='[object Error]');e[Symbol.toStringTag]='chosen';check(tag.call(e)==='[object chosen]');
var m=[];m.toString=function(){return 'override';};check(A([],m).message==='override');
var touched=0,type=false;try{A([],Symbol('message'),{get cause(){touched++;}});}catch(x){type=x instanceof TypeError;}check(type&&touched===0);var result='ok';
"###,
        r###"result"###,
        "ok",
    );
}

#[test]
fn error_brand_genuine_fake_and_prototype() {
    observe(
        "error_brand_genuine_fake_and_prototype",
        r###"function check(v){if(!v)throw new Error('oracle');}
var O=Object,tag=O.prototype.toString,ctors=[Error,EvalError,RangeError,ReferenceError,SyntaxError,TypeError,URIError],called=[],built=[],fake=[];
for(var i=0;i<ctors.length;i++){
 var C=ctors[i],e=C('called'),b=new C('built'),f=O.create(C.prototype);
 check(tag.call(C.prototype)==='[object Object]'&&tag.call(f)==='[object Object]');
 f.name=C.name;f.message='called';check(tag.call(f)==='[object Object]');
 called.push(e);built.push(b);fake.push(f);
}
for(var i=0;i<called.length;i++){
 check(tag.call(called[i])==='[object Error]'&&tag.call(built[i])==='[object Error]');
 O.setPrototypeOf(called[i],null);check(tag.call(called[i])==='[object Error]');
}
var result=called.length+':'+built.length+':'+fake.length;
"###,
        r###"result"###,
        "7:7:7",
    );
}

#[test]
fn error_tag_string_override_and_getter_identity() {
    observe(
        "error_tag_string_override_and_getter_identity",
        r###"function check(v){if(!v)throw new Error('oracle');}
var O=Object,tag=O.prototype.toString,key=Symbol.toStringTag,subjects=[Error('m'),O.create(Error.prototype),Error.prototype],gets=0,throws=0;
for(var i=0;i<subjects.length;i++){
 var target=subjects[i],token={};
 O.defineProperty(target,key,{get:function(){check(this===target);gets++;return 'chosen';},configurable:true});
 check(tag.call(target)==='[object chosen]');
 O.defineProperty(target,key,{value:'',writable:true,configurable:true});check(tag.call(target)==='[object ]');
 O.defineProperty(target,key,{get:function(){check(this===target);throws++;throw token;},configurable:true});
 var same=false;try{tag.call(target);}catch(e){same=e===token;}check(same);delete target[key];
}
var result=gets+':'+throws;
"###,
        r###"result"###,
        "3:3",
    );
}

#[test]
fn error_tag_non_string_falls_back_without_coercion() {
    observe(
        "error_tag_non_string_falls_back_without_coercion",
        r###"function check(v){if(!v)throw new Error('oracle');}
var O=Object,tag=O.prototype.toString,key=Symbol.toStringTag,token={},poison={toString:function(){throw token;},valueOf:function(){throw token;}};
var genuine=Error('m'),fake=O.create(Error.prototype),subjects=[genuine,fake,Error.prototype],values=[undefined,null,1,false,Symbol('tag'),poison],checked=0;
for(var i=0;i<values.length;i++){
 for(var j=0;j<subjects.length;j++){
  O.defineProperty(subjects[j],key,{value:values[i],writable:true,configurable:true});
  check(tag.call(subjects[j])===(j===0?'[object Error]':'[object Object]'));checked++;
 }
}
var gets=0;O.defineProperty(genuine,key,{get:function(){check(this===genuine);gets++;return poison;},configurable:true});
check(tag.call(genuine)==='[object Error]');
for(var j=0;j<subjects.length;j++)delete subjects[j][key];
check(tag.call(genuine)==='[object Error]'&&tag.call(fake)==='[object Object]'&&tag.call(Error.prototype)==='[object Object]');
var result=checked+':'+gets;
"###,
        r###"result"###,
        "18:1",
    );
}

#[test]
fn aggregate_iterator_caches_next_and_skips_exhausted_value() {
    observe(
        "aggregate_iterator_caches_next_and_skips_exhausted_value",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
function check(v){if(!v)throw new Error('oracle');}
var trace='',calls=0,gets=0,poison=0,it={},item={};
Object.defineProperty(it,'next',{get:function(){gets++;trace+='N';return function(){
 trace+='S';if(calls++===0){
  Object.defineProperty(it,'next',{get:function(){poison++;throw new Error('replacement next');},configurable:true});
  return {get done(){trace+='D';return false;},get value(){trace+='V';return item;}};
 }
 return {get done(){trace+='D';return true;},get value(){poison++;throw new Error('exhausted value');}};
};},configurable:true});
var items={};Object.defineProperty(items,Symbol.iterator,{get:function(){trace+='I';return function(){trace+='i';return it;};}});
var e=AggregateError(items);check(e.errors.length===1&&e.errors[0]===item&&gets===1&&calls===2&&poison===0);
var result=trace+'|'+gets+'|'+calls+'|'+poison;
"###,
        r###"result"###,
        "IiNSDVSD|1|2|0",
    );
}
