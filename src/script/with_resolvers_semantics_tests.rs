//! Public Promise.withResolvers source-derived contracts, frozen before production.
use super::*;

fn execute_mode(
    runtime: &mut Runtime,
    document: &mut Document,
    strict: bool,
    source: &str,
) -> Result<Value> {
    if strict {
        runtime.execute_strict(source, document)
    } else {
        runtime.execute(source, document)
    }
}

fn observe(case_id: &str, phases: &[(&str, &str, Value)]) {
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        for (phase, (source, expression, expected)) in phases.iter().enumerate() {
            execute_mode(&mut runtime, &mut document, strict, source).unwrap_or_else(|error| {
                panic!("{case_id} strict={strict} phase={phase} source: {error:?}")
            });
            let observed = execute_mode(&mut runtime, &mut document, strict, expression)
                .unwrap_or_else(|error| {
                    panic!("{case_id} strict={strict} phase={phase} observation: {error:?}")
                });
            assert_eq!(
                &observed, expected,
                "{case_id} strict={strict} phase={phase}"
            );
        }
    }
}

#[test]
fn promise_with_resolvers_metadata_and_detached_nonconstruction() {
    observe(
        "metadata_and_detached_nonconstruction",
        &[(
            r###"if(typeof Promise.withResolvers!=='function')throw new Error('Promise.withResolvers missing');
function check(v){if(!v)throw new Error('oracle');}
var f=Promise.withResolvers,O=Object,has=O.prototype.hasOwnProperty,d=O.getOwnPropertyDescriptor(Promise,'withResolvers');
check(d.value===f&&d.writable&&!d.enumerable&&d.configurable);
var n=O.getOwnPropertyDescriptor(f,'name'),l=O.getOwnPropertyDescriptor(f,'length');
check(n.value==='withResolvers'&&!n.writable&&!n.enumerable&&n.configurable);
check(l.value===0&&!l.writable&&!l.enumerable&&l.configurable);
check(O.getPrototypeOf(f)===Function.prototype&&!has.call(f,'prototype'));
var count=0;
try{f();}catch(e){check(e instanceof TypeError);count++;}
try{f.bind(undefined)();}catch(e){check(e instanceof TypeError);count++;}
try{new f();}catch(e){check(e instanceof TypeError);count++;}
try{Reflect.construct(f,[]);}catch(e){check(e instanceof TypeError);count++;}
check(count===4);var result='metadata:4';
"###,
            r###"result"###,
            Value::String("metadata:4".into()),
        )],
    );
}

#[test]
fn promise_with_resolvers_genuine_pending_then_deferred_first_settlement() {
    observe(
        "genuine_pending_then_deferred_first_settlement",
        &[
            (
                r###"if(typeof Promise.withResolvers!=='function')throw new Error('Promise.withResolvers missing');
function check(v){if(!v)throw new Error('oracle');}
var P=Promise,cap=P.withResolvers(),winner={},reason={},trace='S';
check(cap.promise instanceof P&&Object.getPrototypeOf(cap.promise)===P.prototype);
check(typeof cap.resolve==='function'&&typeof cap.reject==='function'&&cap.resolve!==cap.reject);
cap.promise.then(function(v){trace+=v===winner?'F':'x';},function(){trace+='J';});
queueMicrotask(function(){trace+='Q';});check(trace==='S');
"###,
                r###"trace"###,
                Value::String("SQ".into()),
            ),
            (
                r###"trace+='A';check(cap.resolve.call({ignored:true},winner)===undefined);check(trace==='SQA');
check(cap.reject(reason)===undefined&&cap.resolve(reason)===undefined);trace+='B';
"###,
                r###"trace"###,
                Value::String("SQABF".into()),
            ),
            (
                r###"check(cap.resolve(reason)===undefined&&cap.reject(reason)===undefined);
"###,
                r###"trace"###,
                Value::String("SQABF".into()),
            ),
        ],
    );
}

#[test]
fn promise_with_resolvers_genuine_reject_and_missing_resolution_argument() {
    observe(
        "genuine_reject_and_missing_resolution_argument",
        &[(
            r###"if(typeof Promise.withResolvers!=='function')throw new Error('Promise.withResolvers missing');
function check(v){if(!v)throw new Error('oracle');}
var a=Promise.withResolvers(),b=Promise.withResolvers(),reason={},trace='S';
a.promise.then(function(v){trace+=v===undefined?'U':'x';},function(){trace+='x';});
b.promise.then(function(){trace+='x';},function(v){trace+=v===reason?'R':'x';});
check(a.resolve()===undefined&&b.reject(reason)===undefined);
check(a.reject(reason)===undefined&&b.resolve(9)===undefined&&trace==='S');
"###,
            r###"trace"###,
            Value::String("SUR".into()),
        )],
    );
}

#[test]
fn promise_with_resolvers_thenable_assimilation_and_self_resolution() {
    observe(
        "thenable_assimilation_and_self_resolution",
        &[
            (
                r###"if(typeof Promise.withResolvers!=='function')throw new Error('Promise.withResolvers missing');
function check(v){if(!v)throw new Error('oracle');}
var cap=Promise.withResolvers(),winner={},trace='';
var thenable={get then(){trace+='G';return function(resolve,reject){check(this===thenable);trace+='T';resolve(winner);reject(1);resolve(2);};}};
cap.promise.then(function(v){trace+=v===winner?'F':'x';},function(){trace+='x';});
check(cap.resolve(thenable)===undefined&&trace==='G');trace+='S';
queueMicrotask(function(){trace+='Q';});
"###,
                r###"trace"###,
                Value::String("GSTQF".into()),
            ),
            (
                r###"var self=Promise.withResolvers();self.promise.then(function(){trace+='x';},function(e){trace+=e instanceof TypeError?'E':'x';});
check(self.resolve(self.promise)===undefined);trace+='A';
"###,
                r###"trace"###,
                Value::String("GSTQFAE".into()),
            ),
        ],
    );
}

#[test]
fn promise_with_resolvers_generic_constructor_executor_and_exact_callback_identity() {
    observe(
        "generic_constructor_executor_and_exact_callback_identity",
        &[(
            r###"if(typeof Promise.withResolvers!=='function')throw new Error('Promise.withResolvers missing');
function check(v){if(!v)throw new Error('oracle');}
var f=Promise.withResolvers,holder=Object.create(null),receiver={},token={},ret={},trace='',calls=0;
function resolve(v){check(this===receiver&&v===token);calls++;trace+='R';return ret;}
function reject(v){check(this===receiver&&v===token);calls++;trace+='J';return ret;}
function C(ex){
 check(new.target===C&&arguments.length===1&&typeof ex==='function');trace+='C';
 check(ex.length===2&&ex.name===''&&!Object.prototype.hasOwnProperty.call(ex,'prototype'));
 var bad=false;try{new ex();}catch(e){check(e instanceof TypeError);bad=true;}check(bad);
 check(ex.call(receiver,resolve,reject)===undefined);trace+='E';return holder;
}
var resultObject=f.call(C);check(trace==='CE'&&calls===0);
check(resultObject.promise===holder&&resultObject.resolve===resolve&&resultObject.reject===reject);
check(resultObject.resolve.call(receiver,token)===ret&&resultObject.reject.call(receiver,token)===ret);
check(calls===2);var result=trace;
"###,
            r###"result"###,
            Value::String("CERJ".into()),
        )],
    );
}

#[test]
fn promise_with_resolvers_bound_constructor_and_shared_callable_values() {
    observe(
        "bound_constructor_and_shared_callable_values",
        &[(
            r###"if(typeof Promise.withResolvers!=='function')throw new Error('Promise.withResolvers missing');
function check(v){if(!v)throw new Error('oracle');}
var f=Promise.withResolvers,prefix={},receiver={},token={},ret={},trace='',calls=0;
function callback(v){check(this===receiver&&v===token);calls++;trace+='R';return ret;}
var shared=callback.bind(receiver);
function C(first,ex){check(new.target===C&&first===prefix&&arguments.length===2);this.marker=prefix;trace+='C';ex(shared,shared);}
var Bound=C.bind({unused:true},prefix),run=f.bind(Bound),box=run();
check(box.promise.marker===prefix&&Object.getPrototypeOf(box.promise)===C.prototype);
check(box.resolve===shared&&box.reject===shared&&calls===0&&trace==='C');
check(box.resolve(token)===ret&&box.reject(token)===ret&&calls===2);var result=trace;
"###,
            r###"result"###,
            Value::String("CRR".into()),
        )],
    );
}

#[test]
fn promise_with_resolvers_saved_object_intrinsic_and_own_data_descriptors() {
    observe(
        "saved_object_intrinsic_and_own_data_descriptors",
        &[(
            r###"if(typeof Promise.withResolvers!=='function')throw new Error('Promise.withResolvers missing');
function check(v){if(!v)throw new Error('oracle');}
var P=Promise,O=Object,OP=O.prototype,f=P.withResolvers,holder={},trace='',setters=0;
function resolve(){}function reject(){}
var descriptor=O.create(null);descriptor.set=function(){setters++;};descriptor.configurable=true;
function C(ex){trace+='C';ex(resolve,reject);
 O.defineProperty(OP,'promise',descriptor);O.defineProperty(OP,'resolve',descriptor);O.defineProperty(OP,'reject',descriptor);
 Object=function(){throw new Error('global Object used');};Object.prototype={wrong:true};
 Promise=function(){throw new Error('global Promise used');};return holder;
}
var box=f.call(C);check(O.getPrototypeOf(box)===OP&&O.isExtensible(box));
check(Reflect.ownKeys(box).join(',')==='promise,resolve,reject'&&O.keys(box).join(',')==='promise,resolve,reject');
var names=['promise','resolve','reject'],values=[holder,resolve,reject];
for(var i=0;i<3;i++){var d=O.getOwnPropertyDescriptor(box,names[i]);check(d.value===values[i]&&d.writable&&d.enumerable&&d.configurable&&!('get' in d)&&!('set' in d));}
check(setters===0&&trace==='C');var result='C|promise,resolve,reject|0';
"###,
            r###"result"###,
            Value::String("C|promise,resolve,reject|0".into()),
        )],
    );
}

#[test]
fn promise_with_resolvers_fresh_results_mutation_and_resolver_metadata() {
    observe(
        "fresh_results_mutation_and_resolver_metadata",
        &[(
            r###"if(typeof Promise.withResolvers!=='function')throw new Error('Promise.withResolvers missing');
function check(v){if(!v)throw new Error('oracle');}
var a=Promise.withResolvers(),b=Promise.withResolvers(),p=a.promise,r=a.resolve,j=a.reject,x={},y={},trace='S';
check(a!==b&&a.promise!==b.promise&&a.resolve!==b.resolve&&a.reject!==b.reject&&a.resolve!==b.reject&&a.reject!==b.resolve);
var functions=[a.resolve,a.reject,b.resolve,b.reject];
for(var i=0;i<functions.length;i++){var f=functions[i],l=Object.getOwnPropertyDescriptor(f,'length'),n=Object.getOwnPropertyDescriptor(f,'name');
 check(l.value===1&&!l.writable&&!l.enumerable&&l.configurable&&n.value===''&&!n.writable&&!n.enumerable&&n.configurable);
 check(Object.getPrototypeOf(f)===Function.prototype&&!Object.prototype.hasOwnProperty.call(f,'prototype'));
 var caught=false;try{Reflect.construct(f,[]);}catch(e){check(e instanceof TypeError);caught=true;}check(caught);
}
a.promise={};check(delete a.resolve);Object.defineProperty(a,'reject',{value:17});a.extra=true;
check(a.promise!==p&&!Object.prototype.hasOwnProperty.call(a,'resolve')&&a.reject===17&&b.resolve!==undefined);
p.then(function(v){trace+=v===x?'A':'x';},function(){trace+='x';});
b.promise.then(function(){trace+='x';},function(v){trace+=v===y?'B':'x';});
check(r(x)===undefined&&j(y)===undefined&&b.reject(y)===undefined&&trace==='S');
"###,
            r###"trace"###,
            Value::String("SAB".into()),
        )],
    );
}

#[test]
fn promise_with_resolvers_invalid_receivers_and_missing_noncallable_capabilities() {
    observe(
        "invalid_receivers_and_missing_noncallable_capabilities",
        &[(
            r###"if(typeof Promise.withResolvers!=='function')throw new Error('Promise.withResolvers missing');
function check(v){if(!v)throw new Error('oracle');}
var f=Promise.withResolvers,count=0,bad=[undefined,null,true,1,'x',Symbol('x'),{},()=>{},Promise.resolve];
for(var i=0;i<bad.length;i++){var caught=false;try{f.call(bad[i]);}catch(e){check(e instanceof TypeError);caught=true;count++;}check(caught);}
function Missing(ex){}function Empty(ex){ex();}function BadResolve(ex){ex(1,function(){});}
function BadReject(ex){ex(function(){},1);}function Nulls(ex){ex(null,null);}function Returned(ex){return {};}
var constructors=[Missing,Empty,BadResolve,BadReject,Nulls,Returned];
for(var k=0;k<constructors.length;k++){var seen=false;try{f.call(constructors[k]);}catch(e){check(e instanceof TypeError);seen=true;count++;}check(seen);}
check(count===15);var result='type-errors:15';
"###,
            r###"result"###,
            Value::String("type-errors:15".into()),
        )],
    );
}

#[test]
fn promise_with_resolvers_ordinary_constructor_abrupt_identity_and_no_reject_conversion() {
    observe(
        "ordinary_constructor_abrupt_identity_and_no_reject_conversion",
        &[(
            r###"if(typeof Promise.withResolvers!=='function')throw new Error('Promise.withResolvers missing');
function check(v){if(!v)throw new Error('oracle');}
var f=Promise.withResolvers,one={},two={},trace='',calls=0,returned=false;
function resolve(){calls++;}function reject(){calls++;}
function Before(ex){trace+='B';throw one;}function After(ex){trace+='A';ex(resolve,reject);trace+='E';throw two;}
function Primitive(ex){trace+='P';throw 7;}
try{f.call(Before);returned=true;}catch(e){check(e===one);trace+='1';}finally{trace+='f';}
try{f.call(After);returned=true;}catch(e){check(e===two);trace+='2';}finally{trace+='f';}
try{f.call(Primitive);returned=true;}catch(e){check(e===7);trace+='3';}finally{trace+='f';}
check(!returned&&calls===0);var result=trace;
"###,
            r###"result"###,
            Value::String("B1fAE2fP3f".into()),
        )],
    );
}

#[test]
fn promise_with_resolvers_executor_repeat_uses_stored_values_not_once_flag() {
    observe(
        "executor_repeat_uses_stored_values_not_once_flag",
        &[(
            r###"if(typeof Promise.withResolvers!=='function')throw new Error('Promise.withResolvers missing');
function check(v){if(!v)throw new Error('oracle');}
var f=Promise.withResolvers,trace='',resolve=function(){},reject=function(){},other=function(){};
function Valid(ex){check(ex()===undefined&&ex(undefined,undefined)===undefined);trace+='U';check(ex(resolve,reject)===undefined);trace+='V';
 var caught=false;try{ex(other,other);}catch(e){check(e instanceof TypeError);caught=true;trace+='D';}check(caught);}
var box=f.call(Valid);check(box.resolve===resolve&&box.reject===reject);
var pairs=[[resolve,undefined],[undefined,reject],[null,undefined],[undefined,null]],outer=0,inner=0;
for(var i=0;i<pairs.length;i++){
 function Partial(ex){ex(pairs[i][0],pairs[i][1]);try{ex(resolve,reject);}catch(e){check(e instanceof TypeError);inner++;}}
 var caught=false;try{f.call(Partial);}catch(e){check(e instanceof TypeError);caught=true;outer++;}check(caught);
}
check(inner===4&&outer===4);var result=trace+'|'+inner+'|'+outer;
"###,
            r###"result"###,
            Value::String("UVD|4|4".into()),
        )],
    );
}

#[test]
fn promise_with_resolvers_ignored_arguments_evaluate_without_unrelated_gets() {
    observe(
        "ignored_arguments_evaluate_without_unrelated_gets",
        &[(
            r###"if(typeof Promise.withResolvers!=='function')throw new Error('Promise.withResolvers missing');
function check(v){if(!v)throw new Error('oracle');}
var f=Promise.withResolvers,trace='',holder={},resolve=function(){},reject=function(){},poison={};
function trap(){trace+='X';throw new Error('unrelated access');}
Object.defineProperty(poison,Symbol.iterator,{get:trap});Object.defineProperty(poison,'then',{get:trap});
poison.valueOf=trap;poison.toString=trap;
function C(ex){trace+='C';ex(resolve,reject);return holder;}
Object.defineProperty(C,'resolve',{get:trap});Object.defineProperty(C,Symbol.species,{get:trap});
function argument(){trace+='A';return poison;}
var box=f.call(C,argument(),argument());check(box.promise===holder&&trace==='AAC');
var caught=false;try{f.call({},argument());}catch(e){check(e instanceof TypeError);caught=true;trace+='T';}
check(caught&&trace==='AACAT');var result=trace;
"###,
            r###"result"###,
            Value::String("AACAT".into()),
        )],
    );
}
