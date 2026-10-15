//! Public-API Promise semantic literals frozen before production.
use super::*;

fn observe(
    script: &str,
    expression: &str,
    expected: Value,
    entry: Option<Value>,
    console: &[&str],
) {
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
    assert_eq!(
        runtime
            .console
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        console
    );
}

#[test]
fn promise_synchronous_executor_shared_fifo_and_chained_append() {
    observe(
        r###"var log='';queueMicrotask(function(){log+='Q';});var p=new Promise(function(resolve){log+='E';resolve(10);});p.then(function(v){log+='P'+v;return v+1;}).then(function(v){log+='D'+v;});queueMicrotask(function(){log+='C';});log+='S';42;"###,
        r###"log"###,
        Value::String("ESQP10CD11".into()),
        Some(Value::Number(42.0)),
        &[],
    );
}

#[test]
fn promise_then_getter_synchronous_call_asynchronous() {
    observe(
        r###"var log='';var token={};Object.defineProperty(token,'then',{get:function(){log+='G';return function(resolve,reject){log+='T';resolve(4);reject(9);throw 11;};}});Promise.resolve(token).then(function(v){log+='P'+v;},function(){log+='X';});queueMicrotask(function(){log+='Q';});log+='S';"###,
        r###"log"###,
        Value::String("GSTQP4".into()),
        None,
        &[],
    );
}

#[test]
fn promise_original_resolver_consumed_before_then_getter() {
    observe(
        r###"var log='';var resolve,reject;var p=new Promise(function(r,j){resolve=r;reject=j;});var token={};Object.defineProperty(token,'then',{get:function(){log+='G';reject('wrong');return function(r){log+='T';r(99);};}});resolve(token);reject('wrong2');resolve(7);p.then(function(v){log+='P'+v;},function(){log+='X';});log+='S';"###,
        r###"log"###,
        Value::String("GSTP99".into()),
        None,
        &[],
    );
}

#[test]
fn promise_handler_throw_original_value_without_diagnostic_getters() {
    observe(
        r###"var token={};var touched=0;Object.defineProperty(token,'constructor',{get:function(){touched++;throw 99;}});var seen=false;Promise.resolve(1).then(function(){throw token;}).then(undefined,function(e){seen=e===token;});"###,
        r###"seen&&touched===0"###,
        Value::Bool(true),
        None,
        &[],
    );
}

#[test]
fn promise_self_resolution_and_chained_cycle_reject() {
    observe(
        r###"var r;var p=new Promise(function(resolve){r=resolve;});var a=false,b=false;r(p);p.then(undefined,function(e){a=e instanceof TypeError;});var q=Promise.resolve(1).then(function(){return q;});q.then(undefined,function(e){b=e instanceof TypeError;});"###,
        r###"a&&b"###,
        Value::Bool(true),
        None,
        &[],
    );
}

#[test]
fn promise_noncallable_then_fulfills_exact_object() {
    observe(
        r###"var token={then:7};var same=false;Promise.resolve(token).then(function(v){same=v===token;});"###,
        r###"same"###,
        Value::Bool(true),
        None,
        &[],
    );
}

#[test]
fn promise_generic_resolve_identity_precedes_constructor_validation() {
    observe(
        r###"var p=Promise.resolve(1);var marker={};p.constructor=marker;var same=Promise.resolve.call(marker,p)===p;var refused=false;try{Promise.reject.call(marker,1);}catch(e){refused=e instanceof TypeError;}"###,
        r###"same&&refused"###,
        Value::Bool(true),
        None,
        &[],
    );
}

#[test]
fn promise_generic_capability_can_accept_second_call_after_undefined_pair() {
    observe(
        r###"var log='';var target={};function C(executor){executor(undefined,undefined);executor(function(v){'use strict';log+=(this===undefined&&arguments.length===1?'R':'X')+v;},function(){log+='J';});return target;}var same=Promise.resolve.call(C,3)===target;"###,
        r###"same&&log==='R3'"###,
        Value::Bool(true),
        None,
        &[],
    );
}

#[test]
fn promise_species_callbacks_are_observable_and_child_is_generic_capability() {
    observe(
        r###"var log='';var target={};function C(executor){log+='C';executor(function(v){log+='R'+v;},function(v){log+='J'+v;});return target;}var marker={};Object.defineProperty(marker,Symbol.species,{get:function(){log+='G';return C;}});var p=Promise.resolve(2);Object.defineProperty(p,'constructor',{get:function(){log+='K';return marker;}});var same=p.then(function(v){log+='H';return v+1;})===target;log+='S';"###,
        r###"same&&log==='KGCSHR3'"###,
        Value::Bool(true),
        None,
        &[],
    );
}

#[test]
fn promise_absent_handlers_forward_exact_fulfillment_and_rejection() {
    observe(
        r###"var token={};var a=false,b=false;Promise.resolve(token).then(3,null).then(function(v){a=v===token;});Promise.reject(token).then(undefined,7).then(undefined,function(v){b=v===token;});"###,
        r###"a&&b"###,
        Value::Bool(true),
        None,
        &[],
    );
}

#[test]
fn promise_pending_fanout_and_reentrant_attachment() {
    observe(
        r###"var log='',release;var p=new Promise(function(r){release=r;});p.then(function(){log+='A';p.then(function(){log+='C';});});p.then(function(){log+='B';});release(1);queueMicrotask(function(){log+='M';});log+='S';"###,
        r###"log"###,
        Value::String("SABMC".into()),
        None,
        &[],
    );
}

#[test]
fn promise_captured_then_receiver_arguments_and_nested_adoption() {
    observe(
        r###"var log='',token={};Object.defineProperty(token,'then',{configurable:true,get:function(){log+='G';return function(r,j){'use strict';log+=(this===token&&arguments.length===2?'T':'X');r(5);return 99;};}});Promise.resolve(token).then(function(v){log+='P'+v;});Object.defineProperty(token,'then',{value:function(){log+='X';},configurable:true});log+='S';"###,
        r###"log"###,
        Value::String("GSTP5".into()),
        None,
        &[],
    );
    observe(
        r###"var log='',inner={then:function(r){log+='I';r(7);}};var outer={then:function(r,j){log+='O';r(inner);j(1);}};Promise.resolve(outer).then(function(v){log+='P'+v;});queueMicrotask(function(){log+='Q';});log+='S';"###,
        r###"log"###,
        Value::String("SOQIP7".into()),
        None,
        &[],
    );
}

#[test]
fn promise_abrupt_executor_and_then_getter() {
    observe(
        r###"var token={},a=false,b=false,c=false,x={};Object.defineProperty(x,'then',{get:function(){throw token;}});Promise.resolve(x).then(undefined,function(e){a=e===token;});new Promise(function(){throw token;}).then(undefined,function(e){b=e===token;});new Promise(function(r){r(6);throw token;}).then(function(v){c=v===6;});"###,
        r###"a&&b&&c"###,
        Value::Bool(true),
        None,
        &[],
    );
}

#[test]
fn promise_generic_capability_failure_boundaries() {
    observe(
        r###"var captures=0,outer=0,calls=0;
function firstResolve(e){e(function(){calls++;},undefined);try{e(function(){calls++;},function(){calls++;});}catch(x){if(x instanceof TypeError)captures++;}}
function firstReject(e){e(undefined,function(){calls++;});try{e(function(){calls++;},function(){calls++;});}catch(x){if(x instanceof TypeError)captures++;}}
function badResolve(e){e(7,function(){calls++;});}
function badReject(e){e(function(){calls++;},7);}
var constructors=[firstResolve,firstReject,badResolve,badReject];
for(var i=0;i<constructors.length;i++){try{Promise.resolve.call(constructors[i],1);}catch(x){if(x instanceof TypeError)outer++;}}"###,
        r###"captures+'|'+outer+'|'+calls"###,
        Value::String("2|4|0".into()),
        None,
        &[],
    );
    observe(
        r###"var token={},seen=0,wrong=0;
function ResolveThrow(e){e(function(){throw token;},function(){wrong++;});return {};}
function RejectThrow(e){e(function(){wrong++;},function(){throw token;});return {};}
try{Promise.resolve.call(ResolveThrow,1);}catch(x){if(x===token)seen++;}
try{Promise.reject.call(RejectThrow,1);}catch(x){if(x===token)seen++;}"###,
        r###"seen+'|'+wrong"###,
        Value::String("2|0".into()),
        None,
        &[],
    );
    observe(
        r###"var log='',token={};function C(e){e(function(){log+='R';throw token;},function(){log+='J';});return {};}var p=Promise.resolve(1);p.constructor={};p.constructor[Symbol.species]=C;p.then(function(){log+='H';return 2;});queueMicrotask(function(){log+='Q';});log+='S';"###,
        r###"log"###,
        Value::String("SHRQ".into()),
        None,
        &["Uncaught Promise job exception: uncaught exception: [object Object]"],
    );
    observe(
        r###"var log='',token={};function C(e){e(function(){log+='R';},function(){log+='J';throw token;});return {};}
var p=Promise.reject(token);p.constructor={};p.constructor[Symbol.species]=C;
p.then(undefined,function(e){log+='H';throw e;});queueMicrotask(function(){log+='Q';});log+='S';"###,
        r###"log"###,
        Value::String("SHJQ".into()),
        None,
        &["Uncaught Promise job exception: uncaught exception: [object Object]"],
    );
}

#[test]
fn promise_selected_surface_brand_newtarget_and_callback_controls() {
    observe(
        r###"function check(v){if(!v)throw new Error('Promise selected descriptor surface');}
function data(o,k,v,w,e,c){var d=Object.getOwnPropertyDescriptor(o,k);check(d.value===v&&d.writable===w&&d.enumerable===e&&d.configurable===c);}
var P=Promise,proto=P.prototype;
data(window,'Promise',P,true,false,true);check(P===globalThis.Promise);
check(Object.getPrototypeOf(P)===Function.prototype&&Object.getPrototypeOf(proto)===Object.prototype);
data(P,'name','Promise',false,false,true);data(P,'length',1,false,false,true);data(P,'prototype',proto,false,false,false);
data(P,'resolve',P.resolve,true,false,true);data(P,'reject',P.reject,true,false,true);
data(proto,'constructor',P,true,false,true);data(proto,'then',proto.then,true,false,true);
data(proto,Symbol.toStringTag,'Promise',false,false,true);
check(Object.getOwnPropertyNames(P).join(',')==='length,name,prototype,resolve,reject,all,race,allSettled,any,withResolvers,try');
check(Object.getOwnPropertyNames(proto).join(',')==='constructor,then,catch,finally');
check(Object.getOwnPropertySymbols(P).length===1&&Object.getOwnPropertySymbols(P)[0]===Symbol.species);
check(Object.getOwnPropertySymbols(proto).length===1&&Object.getOwnPropertySymbols(proto)[0]===Symbol.toStringTag);
var species=Object.getOwnPropertyDescriptor(P,Symbol.species);check(species.set===undefined&&!species.enumerable&&species.configurable);
var marker={};check(species.get.call(marker)===marker&&species.get.call(null)===null&&species.get.call(undefined)===undefined);
var rows=[[P.resolve,'resolve',1],[P.reject,'reject',1],[proto.then,'then',2],[species.get,'get [Symbol.species]',0]];
var refused=0;
for(var i=0;i<rows.length;i++){var f=rows[i][0];check(Object.getPrototypeOf(f)===Function.prototype);check(Object.getOwnPropertyNames(f).join(',')==='length,name');check(Object.getOwnPropertySymbols(f).length===0);data(f,'length',rows[i][2],false,false,true);data(f,'name',rows[i][1],false,false,true);check(Object.getOwnPropertyDescriptor(f,'prototype')===undefined);try{new f();}catch(x){if(x instanceof TypeError)refused++;}for(var j=0;j<i;j++)check(f!==rows[j][0]);}
check(P.resolve===rows[0][0]&&P.reject===rows[1][0]&&proto.then===rows[2][0]);
check(Object.prototype.toString.call(proto)==='[object Promise]');"###,
        r###"refused"###,
        Value::Number(4.0),
        None,
        &[],
    );
    observe(
        r###"var failures=0,poison=0;
function typeError(f){try{f();}catch(x){if(x instanceof TypeError)failures++;}}
typeError(function(){Promise(function(){});});typeError(function(){new Promise(7);});
var forged=Object.create(Promise.prototype);
Object.defineProperty(forged,'constructor',{get:function(){poison++;throw 7;}});
Object.defineProperty(Promise.prototype,'constructor',{get:function(){poison++;throw 7;},configurable:true});
typeError(function(){Promise.prototype.then.call(forged);});typeError(function(){Promise.prototype.then.call(Promise.prototype);});"###,
        r###"failures+'|'+poison"###,
        Value::String("4|0".into()),
        None,
        &[],
    );
    observe(
        r###"var P=Promise,p=P.resolve(1),token={},seen=0,wrong=0;
Object.defineProperty(p,'constructor',{get:function(){throw token;},configurable:true});
try{p.then(function(){wrong++;});}catch(x){if(x===token)seen++;}
var marker={};Object.defineProperty(marker,Symbol.species,{get:function(){throw token;},configurable:true});
Object.defineProperty(p,'constructor',{value:marker,writable:true,configurable:true});
try{p.then(function(){wrong++;});}catch(x){if(x===token)seen++;}
Object.defineProperty(marker,Symbol.species,{value:{},writable:true,configurable:true});
try{p.then();}catch(x){if(x instanceof TypeError)seen++;}
p.constructor=7;try{p.then();}catch(x){if(x instanceof TypeError)seen++;}
p.constructor={};p.constructor[Symbol.species]=null;var nullChild=p.then();
p.constructor={};p.constructor[Symbol.species]=undefined;var undefinedChild=p.then();
p.constructor=undefined;window.Promise=17;var noConstructorChild=p.then();window.Promise=P;
var defaults=Object.getPrototypeOf(nullChild)===P.prototype&&Object.getPrototypeOf(undefinedChild)===P.prototype&&Object.getPrototypeOf(noConstructorChild)===P.prototype;"###,
        r###"seen===4&&wrong===0&&defaults"###,
        Value::Bool(true),
        None,
        &[],
    );
    observe(
        r###"var log='',executorOK=false,handlerOK=false,poison=0,custom={};
var Alt=(function(){}).bind(null);
Object.defineProperty(Alt,'prototype',{get:function(){log+='P';return custom;}});
try{Reflect.construct(Promise,[7],Alt);}catch(x){if(x instanceof TypeError)log+='I';}
var ignored={};Object.defineProperty(ignored,'then',{get:function(){poison++;throw 7;}});
var p=Reflect.construct(Promise,[function(r,j){'use strict';log+='E';executorOK=this===undefined&&arguments.length===2;r(8);return ignored;}],Alt);
var changedPrototype=Object.getPrototypeOf(p)===custom;
Promise.prototype.then.call(p,function(v){'use strict';log+='H';handlerOK=this===undefined&&arguments.length===1&&v===8;});
function Other(){}Other.prototype=7;var fallback=Reflect.construct(Promise,[function(){}],Other);
var fallbackOK=Object.getPrototypeOf(fallback)===Promise.prototype;"###,
        r###"log==='IPEH'&&executorOK&&handlerOK&&changedPrototype&&fallbackOK&&poison===0"###,
        Value::Bool(true),
        None,
        &[],
    );
    observe(
        r###"function check(v){if(!v)throw new Error('Promise callback identity');}
var r1,j1,r2,j2,a=0,b=0;
var p1=new Promise(function(r,j){r1=r;j1=j;});var p2=new Promise(function(r,j){r2=r;j2=j;});
var fs=[r1,j1,r2,j2];for(var i=0;i<fs.length;i++){check(fs[i].name===''&&fs[i].length===1);check(Object.getOwnPropertyDescriptor(fs[i],'prototype')===undefined);for(var k=0;k<i;k++)check(fs[i]!==fs[k]);}
var refused=0;for(var n=0;n<fs.length;n++){try{new fs[n]();}catch(x){if(x instanceof TypeError)refused++;}}
check(r1.call({wrong:true},11)===undefined);check(j1.apply(null,[22])===undefined);
var bound=r2.bind({wrong:true});check(bound(33)===undefined);check(j2.call(undefined,44)===undefined);
try{new bound();}catch(x){if(x instanceof TypeError)refused++;}
p1.then(function(v){a=v;});p2.then(function(v){b=v;});
var captures=[],received=0,target={};function C(e){captures.push(e);e.call({wrong:true},function(v){received+=v;},function(){received=-99;});return target;}
check(Promise.resolve.call(C,5)===target);check(Promise.resolve.call(C,7)===target);
check(captures[0]!==captures[1]);for(var z=0;z<captures.length;z++){check(captures[z].name===''&&captures[z].length===2);check(Object.getOwnPropertyDescriptor(captures[z],'prototype')===undefined);try{new captures[z]();}catch(x){if(x instanceof TypeError)refused++;}}"###,
        r###"a+'|'+b+'|'+received+'|'+refused"###,
        Value::String("11|33|12|7".into()),
        None,
        &[],
    );
}

#[test]
fn promise_species_constructor_settles_pending_source_before_then_state_read() {
    observe(
        r###"var log='',release;var p=new Promise(function(r){release=r;});function C(e){log+='C';release(4);e(function(v){log+='R'+v;},function(){log+='J';});return {};}p.constructor={};p.constructor[Symbol.species]=C;p.then(function(v){log+='H'+v;return v+1;});log+='S';"###,
        r###"log"###,
        Value::String("CSH4R5".into()),
        None,
        &[],
    );
}
