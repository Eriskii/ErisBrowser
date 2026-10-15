//! Promise.try public contracts, frozen independently before production.
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
fn promise_try_metadata_cached_alias_nonconstruction() {
    observe(
        "metadata_cached_alias_nonconstruction",
        &[(
            r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
function check(v){if(!v)throw new Error('oracle');}
var P=Promise,O=Object,f=P.try,alias=P.try,has=O.prototype.hasOwnProperty;
var d=O.getOwnPropertyDescriptor(P,'try'),n=O.getOwnPropertyDescriptor(f,'name'),l=O.getOwnPropertyDescriptor(f,'length');
check(f===alias&&d.value===f&&d.writable&&!d.enumerable&&d.configurable);
check(n.value==='try'&&!n.writable&&!n.enumerable&&n.configurable);
check(l.value===1&&!l.writable&&!l.enumerable&&l.configurable);
check(O.getPrototypeOf(f)===Function.prototype&&!has.call(f,'prototype'));
var entered=0,caught=0;function callback(){entered++;return 1;}
try{f(callback);}catch(e){check(e instanceof TypeError);caught++;}
try{f.bind(undefined)(callback);}catch(e){check(e instanceof TypeError);caught++;}
try{new f(callback);}catch(e){check(e instanceof TypeError);caught++;}
try{Reflect.construct(f,[callback]);}catch(e){check(e instanceof TypeError);caught++;}
check(caught===4&&entered===0);
O.defineProperty(f,'name',{value:'renamed'});O.defineProperty(f,'length',{value:9});
check(alias===P.try&&alias.name==='renamed'&&alias.length===9);
var p=P.resolve(7),run=alias.bind(P);check(run(function(){entered++;return p;})===p&&entered===1);
var result='metadata:4|alias:true|callback:1';
"###,
            r###"result"###,
            Value::String("metadata:4|alias:true|callback:1".into()),
        )],
    );
}

#[test]
fn promise_try_receiver_guard_and_late_constructor_validation() {
    observe(
        "receiver_guard_and_late_constructor_validation",
        &[(
            r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
function check(v){if(!v)throw new Error('oracle');}
var f=Promise.try,bad=[undefined,null,true,1,'x',Symbol('x')],trace='',args=0,entered=0,primitive=0,objects=0,token={};
function argument(){args++;trace+='A';return token;}
function callback(){entered++;trace+='F';return 9;}
for(var i=0;i<bad.length;i++){
 trace='';var caught=false;
 try{f.call(bad[i],callback,argument());}catch(e){check(e instanceof TypeError);trace+='T';caught=true;primitive++;}
 check(caught&&trace==='AT'&&entered===0);
}
var nonconstructors=[{},()=>{}];
for(var j=0;j<nonconstructors.length;j++){
 trace='';var normal=false;
 try{f.call(nonconstructors[j],callback,argument());}catch(e){check(e instanceof TypeError);trace+='T';normal=true;objects++;}
 check(normal&&trace==='AFT');
 trace='';var abrupt=false;
 try{f.call(nonconstructors[j],function(){entered++;trace+='F';throw token;},argument());}
 catch(e){check(e instanceof TypeError&&e!==token);trace+='T';abrupt=true;objects++;}
 check(abrupt&&trace==='AFT');
}
check(primitive===6&&objects===4&&entered===4&&args===10);
var result='primitive:'+primitive+'|object:'+objects+'|callbacks:'+entered+'|args:'+args;
"###,
            r###"result"###,
            Value::String("primitive:6|object:4|callbacks:4|args:10".into()),
        )],
    );
}

#[test]
fn promise_try_generic_normal_callback_before_constructor() {
    observe(
        "generic_normal_callback_before_constructor",
        &[(
            r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
function check(v){if(!v)throw new Error('oracle');}
var f=Promise.try,token={},holder={},ignored={},trace='',resolves=0,rejects=0;
function resolve(v){'use strict';check(this===undefined&&arguments.length===1&&v===token);resolves++;trace+='R';return ignored;}
function reject(){rejects++;trace+='J';}
function C(ex){check(new.target===C&&arguments.length===1&&typeof ex==='function');trace+='C';check(ex(resolve,reject)===undefined);return holder;}
var value=f.call(C,function(){'use strict';check(this===undefined&&arguments.length===0);trace+='F';return token;});
check(value===holder&&trace==='FCR'&&resolves===1&&rejects===0);
var result=trace+'|'+resolves+'|'+rejects;
"###,
            r###"result"###,
            Value::String("FCR|1|0".into()),
        )],
    );
}

#[test]
fn promise_try_callback_throw_constructor_and_reject_precedence() {
    observe(
        "callback_throw_constructor_and_reject_precedence",
        &[(
            r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
function check(v){if(!v)throw new Error('oracle');}
var f=Promise.try,reason={},constructorToken={},rejectToken={},holder={},ignored={},trace='',resolves=0,rejects=0,seen=[];
function resolve(){resolves++;trace+='R';}
function reject(v){'use strict';check(this===undefined&&arguments.length===1&&v===reason);rejects++;trace+='J';return ignored;}
function C(ex){trace+='C';ex(resolve,reject);return holder;}
var reasons=[reason,7];
for(var i=0;i<reasons.length;i++){
 reason=reasons[i];trace='';
 var value=f.call(C,function(){trace+='F';throw reason;});
 check(value===holder&&trace==='FCJ');seen.push(trace);
}
function BadC(ex){trace+='C';throw constructorToken;}
trace='';var caught=false;
try{f.call(BadC,function(){trace+='F';throw reason;});}catch(e){check(e===constructorToken);caught=true;}
check(caught&&trace==='FC');seen.push(trace);
function ThrowReject(v){'use strict';check(this===undefined&&arguments.length===1&&v===reason);rejects++;trace+='J';throw rejectToken;}
function RejectC(ex){trace+='C';ex(resolve,ThrowReject);return holder;}
trace='';caught=false;
try{f.call(RejectC,function(){trace+='F';throw reason;});}catch(e){check(e===rejectToken);caught=true;}
check(caught&&trace==='FCJ'&&resolves===0&&rejects===3);seen.push(trace);
var result=seen.join(',')+'|rejects:'+rejects;
"###,
            r###"result"###,
            Value::String("FCJ,FCJ,FC,FCJ|rejects:3".into()),
        )],
    );
}

#[test]
fn promise_try_genuine_identity_before_constructor_check() {
    observe(
        "genuine_identity_before_constructor_check",
        &[(
            r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
function check(v){if(!v)throw new Error('oracle');}
var P=Promise,f=P.try,O=Object,token={},reason={},handled=false,trace='',reads=0,seen=[];
var pending=new P(function(){}),fulfilled=P.resolve(token),rejected=P.reject(reason);
rejected.then(function(){throw new Error('wrong fulfillment');},function(v){check(v===reason);handled=true;});
var promises=[pending,fulfilled,rejected],targets=[P,{},()=>{}],target;
for(var i=0;i<promises.length;i++){
 target=targets[i];trace='';
 O.defineProperty(promises[i],'constructor',{configurable:true,get:function(){reads++;trace+='G';return target;}});
 var resultObject=f.call(target,function(){trace+='F';return promises[i];});
 check(resultObject===promises[i]&&trace==='FG');seen.push(trace);
}
check(reads===3&&!handled);
"###,
            r###"seen.join(',')+'|'+handled"###,
            Value::String("FG,FG,FG|true".into()),
        )],
    );
}

#[test]
fn promise_try_genuine_constructor_getter_mismatch_and_throw() {
    observe(
        "genuine_constructor_getter_mismatch_and_throw",
        &[(
            r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
function check(v){if(!v)throw new Error('oracle');}
var P=Promise,f=P.try,O=Object,p=P.resolve(7),other={},holder={},token={},trace='',reads=0,constructors=0,resolves=0,rejects=0,seen=[];
function resolve(v){'use strict';check(this===undefined&&arguments.length===1&&v===p);trace+='R';resolves++;}
function reject(){trace+='J';rejects++;}
function C(ex){trace+='C';constructors++;ex(resolve,reject);return holder;}
O.defineProperty(p,'constructor',{configurable:true,get:function(){reads++;trace+='G';return other;}});
check(f.call(C,function(){trace+='F';return p;})===holder&&trace==='FGCR');seen.push(trace);
O.defineProperty(p,'constructor',{configurable:true,get:function(){reads++;trace+='G';throw token;}});
var receivers=[C,{}];
for(var i=0;i<receivers.length;i++){
 trace='';var caught=false;
 try{f.call(receivers[i],function(){trace+='F';return p;});}catch(e){check(e===token);caught=true;}
 check(caught&&trace==='FG');seen.push(trace);
}
check(reads===3&&constructors===1&&resolves===1&&rejects===0);
var result=seen.join(',')+'|'+reads+'|'+constructors+'|'+resolves+'|'+rejects;
"###,
            r###"result"###,
            Value::String("FGCR,FG,FG|3|1|1|0".into()),
        )],
    );
}

#[test]
fn promise_try_invalid_callback_and_capability_capture() {
    observe(
        "invalid_callback_and_capability_capture",
        &[(
            r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
function check(v){if(!v)throw new Error('oracle');}
var f=Promise.try,holder={},token={},trace='',badCallbacks=0,badCapabilities=0,settlers=0,unexpected=0;
function resolve(){unexpected++;trace+='R';}
function reject(v){'use strict';check(this===undefined&&arguments.length===1&&v instanceof TypeError);settlers++;trace+='J';}
function C(ex){trace+='C';ex(resolve,reject);return holder;}
var callbacks=[undefined,null,{},17];
for(var i=0;i<callbacks.length;i++){
 trace='';var resultObject=i===0?f.call(C):f.call(C,callbacks[i]);
 check(resultObject===holder&&trace==='CJ');badCallbacks++;
}
function Missing(ex){trace+='C';return holder;}
function BadResolve(ex){trace+='C';ex(1,reject);return holder;}
function BadReject(ex){trace+='C';ex(resolve,1);return holder;}
var constructors=[Missing,BadResolve,BadReject];
for(var j=0;j<constructors.length;j++){
 trace='';var caught=false;
 try{f.call(constructors[j],function(){trace+='F';return token;});}catch(e){check(e instanceof TypeError);caught=true;}
 check(caught&&trace==='FC');badCapabilities++;
}
function AfterCapture(ex){trace+='C';ex(resolve,reject);throw token;}
trace='';var captureThrow=false;
try{f.call(AfterCapture,function(){trace+='F';throw 7;});}catch(e){check(e===token);captureThrow=true;}
check(captureThrow&&trace==='FC'&&settlers===4&&unexpected===0);
var result='bad-callbacks:'+badCallbacks+'|bad-capabilities:'+badCapabilities+'|capture-throw:1|settlers:'+settlers;
"###,
            r###"result"###,
            Value::String("bad-callbacks:4|bad-capabilities:3|capture-throw:1|settlers:4".into()),
        )],
    );
}

#[test]
fn promise_try_exact_this_tail_arguments_and_evaluation_order() {
    observe(
        "exact_this_tail_arguments_and_evaluation_order",
        &[(
            r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
function check(v){if(!v)throw new Error('oracle');}
var f=Promise.try,O=Object,object={},fn=function(){},symbol=Symbol('arg'),returned={},holder={},trace='',expected=[],counts=[],seen=[],resolves=0;
O.defineProperty(object,'length',{get:function(){throw new Error('argument length Get');}});
O.defineProperty(object,Symbol.iterator,{get:function(){throw new Error('argument iterator Get');}});
function resolve(v){'use strict';check(this===undefined&&arguments.length===1&&v===returned);trace+='R';resolves++;}
function reject(){throw new Error('unexpected reject');}
function C(ex){trace+='C';ex(resolve,reject);return holder;}
function arg(v){trace+='A';return v;}
function callback(){'use strict';check(this===undefined&&arguments.length===expected.length);trace+='F';counts.push(arguments.length);
 for(var i=0;i<expected.length;i++){check(O.is(arguments[i],expected[i]));}return returned;}
check(f.call(C,callback)===holder&&trace==='FCR');seen.push(trace);
trace='';expected=[object];check(f.call(C,callback,arg(object))===holder&&trace==='AFCR');seen.push(trace);
trace='';expected=[object,fn,symbol,undefined,-0,'tail'];
check(f.call(C,callback,arg(object),arg(fn),arg(symbol),arg(undefined),arg(-0),arg('tail'))===holder&&trace==='AAAAAAFCR');seen.push(trace);
var receiver={},prefix={};
function boundTarget(first,second,third){'use strict';check(this===receiver&&arguments.length===3&&first===prefix&&second===object&&third===symbol);trace+='F';counts.push(arguments.length);return returned;}
var bound=boundTarget.bind(receiver,prefix);trace='';
check(f.call(C,bound,arg(object),arg(symbol))===holder&&trace==='AAFCR');seen.push(trace);
check(resolves===4);var result=counts.join(',')+'|'+seen.join(',');
"###,
            r###"result"###,
            Value::String("0,1,6,3|FCR,AFCR,AAAAAAFCR,AAFCR".into()),
        )],
    );
}

#[test]
fn promise_try_normal_resolve_abrupt_is_not_callback_rejection() {
    observe(
        "normal_resolve_abrupt_is_not_callback_rejection",
        &[(
            r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
function check(v){if(!v)throw new Error('oracle');}
var f=Promise.try,token={},holder={},ignored={},resolveToken={},rejectToken={},fail=false,trace='',resolves=0,rejects=0,seen=[];
function resolve(v){'use strict';check(this===undefined&&arguments.length===1&&v===token);resolves++;trace+='R';if(fail)throw resolveToken;return ignored;}
function reject(){rejects++;trace+='J';throw rejectToken;}
function C(ex){trace+='C';ex(resolve,reject);return holder;}
check(f.call(C,function(){trace+='F';return token;})===holder&&trace==='FCR');seen.push(trace);
fail=true;trace='';var caught=false;
try{f.call(C,function(){trace+='F';return token;});}catch(e){check(e===resolveToken);caught=true;}
check(caught&&trace==='FCR'&&resolves===2&&rejects===0);seen.push(trace);
var result=seen.join(',')+'|'+resolves+'|'+rejects;
"###,
            r###"result"###,
            Value::String("FCR,FCR|2|0".into()),
        )],
    );
}

#[test]
fn promise_try_fake_brand_and_thenable_getter_boundary() {
    observe(
        "fake_brand_and_thenable_getter_boundary",
        &[(
            r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
function check(v){if(!v)throw new Error('oracle');}
var P=Promise,f=P.try,O=Object,fake=O.create(P.prototype),holder={},fakeTrace='',fakeReads=0,token={},reason={},ignored={};
O.defineProperty(fake,'constructor',{get:function(){fakeReads++;throw new Error('fake constructor Get');}});
function resolve(v){'use strict';check(this===undefined&&arguments.length===1&&v===fake);fakeTrace+='R';}
function reject(){throw new Error('fake reject');}
function C(ex){fakeTrace+='C';ex(resolve,reject);return holder;}
check(f.call(C,function(){fakeTrace+='F';return fake;})===holder&&fakeTrace==='FCR'&&fakeReads===0);
var trace='',throwTrace='',thenable={};
function then(resolveValue,rejectValue){'use strict';check(this===thenable&&arguments.length===2);trace+='T';resolveValue(token);rejectValue(reason);resolveValue(ignored);throw ignored;}
O.defineProperty(thenable,'then',{get:function(){trace+='G';return then;}});
var p=f.call(P,function(){trace+='F';return thenable;});check(trace==='FG');
p.then(function(v){check(v===token);trace+='V';},function(){trace+='x';});
trace+='E';queueMicrotask(function(){trace+='Q';});
var bad={};O.defineProperty(bad,'then',{get:function(){throwTrace+='G';throw reason;}});
var rejected=f.call(P,function(){throwTrace+='F';return bad;});check(throwTrace==='FG');
rejected.then(function(){throwTrace+='x';},function(v){check(v===reason);throwTrace+='J';});throwTrace+='E';
check(trace==='FGE'&&throwTrace==='FGE');
"###,
            r###"[fakeTrace,trace,throwTrace,fakeReads].join('|')"###,
            Value::String("FCR|FGETQV|FGEJ|0".into()),
        )],
    );
}

#[test]
fn promise_try_genuine_pending_deferred_and_handled_jobs() {
    observe(
        "genuine_pending_deferred_and_handled_jobs",
        &[
            (
                r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
function check(v){if(!v)throw new Error('oracle');}

var P=Promise,f=P.try,release,token={},reason={},trace='S';
var pending=new P(function(resolve){release=resolve;});
var returned=f.call(P,function(){trace+='F';return pending;});check(returned===pending);
returned.then(function(v){check(v===token);trace+='V';},function(){trace+='x';});
trace+='E';queueMicrotask(function(){trace+='Q';});check(trace==='SFE');
"###,
                r###"trace"###,
                Value::String("SFEQ".into()),
            ),
            (
                r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
trace+='A';check(release(token)===undefined);trace+='B';check(trace==='SFEQAB');
"###,
                r###"trace"###,
                Value::String("SFEQABV".into()),
            ),
            (
                r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
var jobs='S';
var a=f.call(P,function(){jobs+='F';return token;});
a.then(function(v){check(v===token);jobs+='V';},function(){jobs+='x';});
var b=f.call(P,function(){jobs+='T';throw reason;});
b.then(function(){jobs+='x';},function(v){check(v===reason);jobs+='J';});
jobs+='E';queueMicrotask(function(){jobs+='Q';});check(jobs==='SFTE');
"###,
                r###"jobs"###,
                Value::String("SFTEVJQ".into()),
            ),
        ],
    );
}

#[test]
fn promise_try_captured_receiver_and_poisoned_lookup_bypass() {
    observe(
        "captured_receiver_and_poisoned_lookup_bypass",
        &[(
            r###"if(typeof Promise.try!=='function')throw new Error('Promise.try missing');
function check(v){if(!v)throw new Error('oracle');}
var P=Promise,O=Object,f=P.try,token={},trace='',poisons=0,newPrototype={fresh:true},p=P.resolve(7);
function trap(){poisons++;throw new Error('unrelated lookup');}
function resolve(v){'use strict';check(this===undefined&&arguments.length===1&&v===token);trace+='R';}
function reject(){throw new Error('unexpected reject');}
function C(ex){check(new.target===original);trace+='C';ex(resolve,reject);this.marker=token;}
var original=C;
O.defineProperty(original,'resolve',{get:trap});O.defineProperty(original,Symbol.species,{get:trap});
O.defineProperty(P,'resolve',{configurable:true,get:trap});O.defineProperty(P,Symbol.species,{configurable:true,get:trap});
var resultObject=f.call(C,function(){
 trace+='F';original.prototype=newPrototype;C={replaced:true};
 Promise=function(){throw new Error('global Promise used');};Object=function(){throw new Error('global Object used');};return token;
});
check(O.getPrototypeOf(resultObject)===newPrototype&&resultObject.marker===token&&trace==='FCR');
check(f.call(P,function(){trace+='F';return p;})===p&&trace==='FCRF'&&poisons===0);
var result=trace+'|'+poisons+'|'+(C!==original);
"###,
            r###"result"###,
            Value::String("FCRF|0|true".into()),
        )],
    );
}
