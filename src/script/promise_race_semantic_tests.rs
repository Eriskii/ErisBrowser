//! Frozen Promise.race public literals from the independently reviewed proposal.
//! Fresh Runtime per strict mode; source and post-drain observation are separate.
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
fn promise_race_fulfilled_fifo() {
    observe(
        "fulfilled_fifo",
        r###"var trace='S', winner;
Promise.race([Promise.resolve(7),Promise.resolve(9)]).then(function(v){winner=v;trace+='W'+v;});
Promise.resolve().then(function(){trace+='P';});
queueMicrotask(function(){trace+='Q';});
trace+='E';"###,
        r###"trace+'|'+(winner===7)"###,
        Value::String("SEPQW7|true".into()),
    );
}

#[test]
fn promise_race_thenable_vs_plain_value() {
    observe(
        "thenable_vs_plain_value",
        r###"var trace='S', winner;
var item={};
Object.defineProperty(item,'then',{get:function(){trace+='G';return function(resolve){trace+='T';resolve('t');};}});
Promise.race([item,'v']).then(function(v){winner=v;trace+='W'+v;});
queueMicrotask(function(){trace+='Q';});
trace+='E';"###,
        r###"trace+'|'+(winner==='v')"###,
        Value::String("SGETQWv|true".into()),
    );
}

#[test]
fn promise_race_intrinsic_empty_pending() {
    observe(
        "intrinsic_empty_pending",
        r###"var trace='S';
Promise.race([]).then(function(){trace+='BAD';},function(){trace+='BAD';});
queueMicrotask(function(){trace+='Q';});
trace+='E';"###,
        r###"trace"###,
        Value::String("SEQ".into()),
    );
}

#[test]
fn promise_race_rejection_identity_fifo() {
    observe(
        "rejection_identity_fifo",
        r###"var trace='S', token={}, same=false;
Promise.race([Promise.reject(token),Promise.resolve(9)]).then(function(){trace+='BAD';},function(e){same=e===token;trace+='R';});
queueMicrotask(function(){trace+='Q';});
trace+='E';"###,
        r###"trace+'|'+same"###,
        Value::String("SEQR|true".into()),
    );
}

#[test]
fn promise_race_generic_identity_cached_lookup_and_repeated_callbacks() {
    observe(
        "generic_identity_cached_lookup_and_repeated_callbacks",
        r###"var trace='',result={},bad=false,seen;
function resolve(v){trace+='F'+v;}
function reject(v){trace+='J'+v;}
function C(executor){trace+='C';executor(resolve,reject);return result;}
Object.defineProperty(C,Symbol.species,{get:function(){bad=true;throw 'species';}});
Object.defineProperty(C,'resolve',{configurable:true,get:function(){trace+='R';return function(v){
  if(this!==C)bad=true;
  trace+='A'+v;
  return {then:function(f,j){
    if(f!==resolve||j!==reject||arguments.length!==2)bad=true;
    trace+='T';f(v);f(v+10);j(v+20);
    Object.defineProperty(C,'resolve',{value:function(){bad=true;throw 'late resolve';}});
  }};
};}});
var index=0,iterator={};
Object.defineProperty(iterator,'next',{configurable:true,get:function(){trace+='N';return function(){
  if(this!==iterator)bad=true;
  trace+='X';var done=index===2,value=++index;
  Object.defineProperty(iterator,'next',{value:function(){bad=true;throw 'late next';}});
  var step={};
  Object.defineProperty(step,'done',{get:function(){trace+='D';return done;}});
  Object.defineProperty(step,'value',{get:function(){trace+='V';return value;}});
  return step;
};}});
var input={};
Object.defineProperty(input,Symbol.iterator,{get:function(){trace+='I';return function(){if(this!==input)bad=true;trace+='M';return iterator;};}});
seen=Promise.race.call(C,input);"###,
        r###"trace+'|'+(seen===result)+'|'+bad"###,
        Value::String("CRIMNXDVA1TF1F11J21XDVA2TF2F12J22XD|true|false".into()),
    );
}

#[test]
fn promise_race_generic_empty_returns_without_callback() {
    observe(
        "generic_empty_returns_without_callback",
        r###"var trace='',result={},seen;
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(){trace+='BAD';});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='BAD';};}});
var iterator={next:function(){trace+='N';return {done:true,get value(){trace+='BAD';}};}};
var input={};input[Symbol.iterator]=function(){trace+='I';return iterator;};
seen=Promise.race.call(C,input);"###,
        r###"trace+'|'+(seen===result)"###,
        Value::String("CRIN|true".into()),
    );
}

// Additive ordinary metadata, error and reentrancy cases; the six original groups above remain exact.

#[test]
fn promise_race_metadata_and_nonconstructor() {
    observe(
        "metadata_and_nonconstructor/metadata",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var d=Object.getOwnPropertyDescriptor(Promise,'race'),f=d.value;
var n=Object.getOwnPropertyDescriptor(f,'name'),l=Object.getOwnPropertyDescriptor(f,'length');
var noNew=false;try{new f([]);}catch(e){noNew=e instanceof TypeError;}
var old=[Promise,Promise.all,Promise.resolve,Promise.reject,Promise.prototype.then,Promise.prototype.catch,Promise.prototype.finally];
var distinct=true;for(var i=0;i<old.length;i++){if(f===old[i])distinct=false;}
var check=[f===Promise.race,d.writable,!d.enumerable,d.configurable,n.value==='race',l.value===1,
!n.writable,!n.enumerable,n.configurable,!l.writable,!l.enumerable,l.configurable,
!Object.prototype.hasOwnProperty.call(f,'prototype'),Object.getPrototypeOf(f)===Function.prototype,noNew,distinct,
Object.getOwnPropertyNames(Promise).join(',')==='length,name,prototype,resolve,reject,all,race,allSettled'].every(function(v){return v;});
Object.defineProperty(f,'name',{value:'changed race'});Object.defineProperty(f,'length',{value:99});
check=check&&Promise.all.name==='all'&&Promise.all.length===1&&Promise.resolve.name==='resolve'&&Promise.resolve.length===1;
"###,
        r###"check
"###,
        Value::Bool(true),
    );
}

#[test]
fn promise_race_capability_precedes_later_getters() {
    observe(
        "capability_precedes_later_getters/nonconstructor",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},caught=false,input={};
var C={};
Object.defineProperty(C,'resolve',{get:function(){trace+='R';throw 'late resolve';}});
Object.defineProperty(input,Symbol.iterator,{get:function(){trace+='I';throw 'late iterator';}});
try{Promise.race.call(C,input);}catch(e){caught=e instanceof TypeError;}
"###,
        r###"trace+'|'+caught
"###,
        Value::String("|true".into()),
    );
    observe(
        "capability_precedes_later_getters/constructor_throw",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},caught=false,input={};
function C(executor){trace+='C';throw token;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';throw 'late resolve';}});
Object.defineProperty(input,Symbol.iterator,{get:function(){trace+='I';throw 'late iterator';}});
try{Promise.race.call(C,input);}catch(e){caught=e===token;}
"###,
        r###"trace+'|'+caught
"###,
        Value::String("C|true".into()),
    );
    observe(
        "capability_precedes_later_getters/invalid_resolve",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},caught=false,input={};
function C(executor){trace+='C';executor(0,function(){});}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';throw 'late resolve';}});
Object.defineProperty(input,Symbol.iterator,{get:function(){trace+='I';throw 'late iterator';}});
try{Promise.race.call(C,input);}catch(e){caught=e instanceof TypeError;}
"###,
        r###"trace+'|'+caught
"###,
        Value::String("C|true".into()),
    );
    observe(
        "capability_precedes_later_getters/invalid_reject",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},caught=false,input={};
function C(executor){trace+='C';executor(function(){},0);}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';throw 'late resolve';}});
Object.defineProperty(input,Symbol.iterator,{get:function(){trace+='I';throw 'late iterator';}});
try{Promise.race.call(C,input);}catch(e){caught=e instanceof TypeError;}
"###,
        r###"trace+'|'+caught
"###,
        Value::String("C|true".into()),
    );
}

#[test]
fn promise_race_resolve_get_or_callable_failure_rejects_before_iterator() {
    observe(
        "resolve_get_or_callable_failure_rejects_before_iterator/getter_throw",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},seen,result={},out,receiver=false,input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=this===undefined;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';throw token;}});
Object.defineProperty(input,Symbol.iterator,{get:function(){trace+='I';throw 'late iterator';}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+ '|'+receiver+'|'+(out===result)
"###,
        Value::String("CRJ|true|true|true".into()),
    );
    observe(
        "resolve_get_or_callable_failure_rejects_before_iterator/not_callable",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},seen,result={},out,receiver=false,input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=this===undefined;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return 0;}});
Object.defineProperty(input,Symbol.iterator,{get:function(){trace+='I';throw 'late iterator';}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen instanceof TypeError)+ '|'+receiver+'|'+(out===result)
"###,
        Value::String("CRJ|true|true|true".into()),
    );
}

#[test]
fn promise_race_acquisition_failure_never_closes() {
    observe(
        "acquisition_failure_never_closes/absent_method",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},seen,result={},out,gets=0,input={},it={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='BAD';};}});
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';throw 'close must not run';}});
Object.defineProperty(it,'next',{get:function(){trace+='G';return function(){trace+='BAD';return {done:true};};}});

out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen instanceof TypeError)+'|'+gets+'|'+(out===result)
"###,
        Value::String("CRJ|true|0|true".into()),
    );
    observe(
        "acquisition_failure_never_closes/null_method",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},seen,result={},out,gets=0,input={},it={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='BAD';};}});
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';throw 'close must not run';}});
Object.defineProperty(it,'next',{get:function(){trace+='G';return function(){trace+='BAD';return {done:true};};}});
Object.defineProperty(input,Symbol.iterator,{get:function(){trace+='I';return null;}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen instanceof TypeError)+'|'+gets+'|'+(out===result)
"###,
        Value::String("CRIJ|true|0|true".into()),
    );
    observe(
        "acquisition_failure_never_closes/noncallable_method",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},seen,result={},out,gets=0,input={},it={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='BAD';};}});
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';throw 'close must not run';}});
Object.defineProperty(it,'next',{get:function(){trace+='G';return function(){trace+='BAD';return {done:true};};}});
Object.defineProperty(input,Symbol.iterator,{get:function(){trace+='I';return 0;}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen instanceof TypeError)+'|'+gets+'|'+(out===result)
"###,
        Value::String("CRIJ|true|0|true".into()),
    );
    observe(
        "acquisition_failure_never_closes/iterator_getter_throw",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},seen,result={},out,gets=0,input={},it={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='BAD';};}});
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';throw 'close must not run';}});
Object.defineProperty(it,'next',{get:function(){trace+='G';return function(){trace+='BAD';return {done:true};};}});
Object.defineProperty(input,Symbol.iterator,{get:function(){trace+='I';throw token;}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+(out===result)
"###,
        Value::String("CRIJ|true|0|true".into()),
    );
    observe(
        "acquisition_failure_never_closes/iterator_call_throw",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},seen,result={},out,gets=0,input={},it={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='BAD';};}});
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';throw 'close must not run';}});
Object.defineProperty(it,'next',{get:function(){trace+='G';return function(){trace+='BAD';return {done:true};};}});
Object.defineProperty(input,Symbol.iterator,{get:function(){trace+='I';return function(){trace+='M';throw token;};}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+(out===result)
"###,
        Value::String("CRIMJ|true|0|true".into()),
    );
    observe(
        "acquisition_failure_never_closes/iterator_result_primitive",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},seen,result={},out,gets=0,input={},it={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='BAD';};}});
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';throw 'close must not run';}});
Object.defineProperty(it,'next',{get:function(){trace+='G';return function(){trace+='BAD';return {done:true};};}});
Object.defineProperty(input,Symbol.iterator,{get:function(){trace+='I';return function(){trace+='M';return 3;};}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen instanceof TypeError)+'|'+gets+'|'+(out===result)
"###,
        Value::String("CRIMJ|true|0|true".into()),
    );
    observe(
        "acquisition_failure_never_closes/next_getter_throw",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},seen,result={},out,gets=0,input={},it={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='BAD';};}});
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';throw 'close must not run';}});
Object.defineProperty(it,'next',{get:function(){trace+='G';throw token;}});
Object.defineProperty(input,Symbol.iterator,{get:function(){trace+='I';return function(){trace+='M';return it;};}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+(out===result)
"###,
        Value::String("CRIMGJ|true|0|true".into()),
    );
}

#[test]
fn promise_race_step_failures_mark_done_without_close() {
    observe(
        "step_failures_mark_done_without_close/noncallable_next",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},seen,result={},out,gets=0,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='BAD';};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
Object.defineProperty(it,'next',{get:function(){trace+='G';return 0;}});
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';throw 'close must not run';}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen instanceof TypeError)+'|'+gets+'|'+(out===result)
"###,
        Value::String("CRIGJ|true|0|true".into()),
    );
    observe(
        "step_failures_mark_done_without_close/throwing_next",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},seen,result={},out,gets=0,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='BAD';};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
Object.defineProperty(it,'next',{get:function(){trace+='G';return function(){trace+='N';throw token;};}});
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';throw 'close must not run';}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+(out===result)
"###,
        Value::String("CRIGNJ|true|0|true".into()),
    );
    observe(
        "step_failures_mark_done_without_close/primitive_step",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},seen,result={},out,gets=0,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='BAD';};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
Object.defineProperty(it,'next',{get:function(){trace+='G';return function(){trace+='N';return 0;};}});
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';throw 'close must not run';}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen instanceof TypeError)+'|'+gets+'|'+(out===result)
"###,
        Value::String("CRIGNJ|true|0|true".into()),
    );
    observe(
        "step_failures_mark_done_without_close/throwing_done",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},seen,result={},out,gets=0,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='BAD';};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
Object.defineProperty(it,'next',{get:function(){trace+='G';return function(){trace+='N';return {get done(){trace+='D';throw token;},get value(){trace+='BAD';}};};}});
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';throw 'close must not run';}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+(out===result)
"###,
        Value::String("CRIGNDJ|true|0|true".into()),
    );
    observe(
        "step_failures_mark_done_without_close/throwing_value",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},seen,result={},out,gets=0,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='BAD';};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
Object.defineProperty(it,'next',{get:function(){trace+='G';return function(){trace+='N';return {get done(){trace+='D';return false;},get value(){trace+='V';throw token;}};};}});
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';throw 'close must not run';}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+(out===result)
"###,
        Value::String("CRIGNDVJ|true|0|true".into()),
    );
}

#[test]
fn promise_race_post_yield_failures_close_preserving_original_reason() {
    observe(
        "post_yield_failures_close_preserving_original_reason/resolve_call__absent",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},closeToken={},seen,result={},out,gets=0,calls=0,rejects=0,receiver=true,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=receiver&&this===undefined;rejects++;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='A';throw token;};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
it.next=function(){trace+='N';return {get done(){trace+='D';return false;},get value(){trace+='V';return 1;}};};

out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+calls+'|'+rejects+'|'+receiver+'|'+(out===result)
"###,
        Value::String("CRINDVAJ|true|0|0|1|true|true".into()),
    );
    observe(
        "post_yield_failures_close_preserving_original_reason/resolve_call__undefined",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},closeToken={},seen,result={},out,gets=0,calls=0,rejects=0,receiver=true,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=receiver&&this===undefined;rejects++;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='A';throw token;};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
it.next=function(){trace+='N';return {get done(){trace+='D';return false;},get value(){trace+='V';return 1;}};};
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';return undefined;}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+calls+'|'+rejects+'|'+receiver+'|'+(out===result)
"###,
        Value::String("CRINDVAXJ|true|1|0|1|true|true".into()),
    );
    observe(
        "post_yield_failures_close_preserving_original_reason/resolve_call__null",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},closeToken={},seen,result={},out,gets=0,calls=0,rejects=0,receiver=true,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=receiver&&this===undefined;rejects++;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='A';throw token;};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
it.next=function(){trace+='N';return {get done(){trace+='D';return false;},get value(){trace+='V';return 1;}};};
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';return null;}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+calls+'|'+rejects+'|'+receiver+'|'+(out===result)
"###,
        Value::String("CRINDVAXJ|true|1|0|1|true|true".into()),
    );
    observe(
        "post_yield_failures_close_preserving_original_reason/resolve_call__noncallable",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},closeToken={},seen,result={},out,gets=0,calls=0,rejects=0,receiver=true,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=receiver&&this===undefined;rejects++;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='A';throw token;};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
it.next=function(){trace+='N';return {get done(){trace+='D';return false;},get value(){trace+='V';return 1;}};};
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';return 0;}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+calls+'|'+rejects+'|'+receiver+'|'+(out===result)
"###,
        Value::String("CRINDVAXJ|true|1|0|1|true|true".into()),
    );
    observe(
        "post_yield_failures_close_preserving_original_reason/resolve_call__getter_throw",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},closeToken={},seen,result={},out,gets=0,calls=0,rejects=0,receiver=true,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=receiver&&this===undefined;rejects++;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='A';throw token;};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
it.next=function(){trace+='N';return {get done(){trace+='D';return false;},get value(){trace+='V';return 1;}};};
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';throw closeToken;}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+calls+'|'+rejects+'|'+receiver+'|'+(out===result)
"###,
        Value::String("CRINDVAXJ|true|1|0|1|true|true".into()),
    );
    observe(
        "post_yield_failures_close_preserving_original_reason/resolve_call__call_throw",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},closeToken={},seen,result={},out,gets=0,calls=0,rejects=0,receiver=true,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=receiver&&this===undefined;rejects++;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='A';throw token;};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
it.next=function(){trace+='N';return {get done(){trace+='D';return false;},get value(){trace+='V';return 1;}};};
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';return function(){'use strict';receiver=receiver&&this===it;calls++;trace+='Y';throw closeToken;};}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+calls+'|'+rejects+'|'+receiver+'|'+(out===result)
"###,
        Value::String("CRINDVAXYJ|true|1|1|1|true|true".into()),
    );
    observe(
        "post_yield_failures_close_preserving_original_reason/resolve_call__primitive_result",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},closeToken={},seen,result={},out,gets=0,calls=0,rejects=0,receiver=true,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=receiver&&this===undefined;rejects++;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='A';throw token;};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
it.next=function(){trace+='N';return {get done(){trace+='D';return false;},get value(){trace+='V';return 1;}};};
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';return function(){'use strict';receiver=receiver&&this===it;calls++;trace+='Y';return 0;};}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+calls+'|'+rejects+'|'+receiver+'|'+(out===result)
"###,
        Value::String("CRINDVAXYJ|true|1|1|1|true|true".into()),
    );
    observe(
        "post_yield_failures_close_preserving_original_reason/resolve_call__object_result",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},closeToken={},seen,result={},out,gets=0,calls=0,rejects=0,receiver=true,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=receiver&&this===undefined;rejects++;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='A';throw token;};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
it.next=function(){trace+='N';return {get done(){trace+='D';return false;},get value(){trace+='V';return 1;}};};
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';return function(){'use strict';receiver=receiver&&this===it;calls++;trace+='Y';return {};};}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+calls+'|'+rejects+'|'+receiver+'|'+(out===result)
"###,
        Value::String("CRINDVAXYJ|true|1|1|1|true|true".into()),
    );
    observe(
        "post_yield_failures_close_preserving_original_reason/then_get__absent",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},closeToken={},seen,result={},out,gets=0,calls=0,rejects=0,receiver=true,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=receiver&&this===undefined;rejects++;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='A';return {get then(){trace+='G';throw token;}};};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
it.next=function(){trace+='N';return {get done(){trace+='D';return false;},get value(){trace+='V';return 1;}};};

out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+calls+'|'+rejects+'|'+receiver+'|'+(out===result)
"###,
        Value::String("CRINDVAGJ|true|0|0|1|true|true".into()),
    );
    observe(
        "post_yield_failures_close_preserving_original_reason/then_get__undefined",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},closeToken={},seen,result={},out,gets=0,calls=0,rejects=0,receiver=true,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=receiver&&this===undefined;rejects++;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='A';return {get then(){trace+='G';throw token;}};};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
it.next=function(){trace+='N';return {get done(){trace+='D';return false;},get value(){trace+='V';return 1;}};};
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';return undefined;}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+calls+'|'+rejects+'|'+receiver+'|'+(out===result)
"###,
        Value::String("CRINDVAGXJ|true|1|0|1|true|true".into()),
    );
    observe(
        "post_yield_failures_close_preserving_original_reason/then_get__null",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},closeToken={},seen,result={},out,gets=0,calls=0,rejects=0,receiver=true,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=receiver&&this===undefined;rejects++;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='A';return {get then(){trace+='G';throw token;}};};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
it.next=function(){trace+='N';return {get done(){trace+='D';return false;},get value(){trace+='V';return 1;}};};
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';return null;}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+calls+'|'+rejects+'|'+receiver+'|'+(out===result)
"###,
        Value::String("CRINDVAGXJ|true|1|0|1|true|true".into()),
    );
    observe(
        "post_yield_failures_close_preserving_original_reason/then_get__noncallable",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},closeToken={},seen,result={},out,gets=0,calls=0,rejects=0,receiver=true,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=receiver&&this===undefined;rejects++;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='A';return {get then(){trace+='G';throw token;}};};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
it.next=function(){trace+='N';return {get done(){trace+='D';return false;},get value(){trace+='V';return 1;}};};
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';return 0;}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+calls+'|'+rejects+'|'+receiver+'|'+(out===result)
"###,
        Value::String("CRINDVAGXJ|true|1|0|1|true|true".into()),
    );
    observe(
        "post_yield_failures_close_preserving_original_reason/then_get__getter_throw",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},closeToken={},seen,result={},out,gets=0,calls=0,rejects=0,receiver=true,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=receiver&&this===undefined;rejects++;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='A';return {get then(){trace+='G';throw token;}};};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
it.next=function(){trace+='N';return {get done(){trace+='D';return false;},get value(){trace+='V';return 1;}};};
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';throw closeToken;}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+calls+'|'+rejects+'|'+receiver+'|'+(out===result)
"###,
        Value::String("CRINDVAGXJ|true|1|0|1|true|true".into()),
    );
    observe(
        "post_yield_failures_close_preserving_original_reason/then_get__call_throw",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},closeToken={},seen,result={},out,gets=0,calls=0,rejects=0,receiver=true,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=receiver&&this===undefined;rejects++;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='A';return {get then(){trace+='G';throw token;}};};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
it.next=function(){trace+='N';return {get done(){trace+='D';return false;},get value(){trace+='V';return 1;}};};
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';return function(){'use strict';receiver=receiver&&this===it;calls++;trace+='Y';throw closeToken;};}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+calls+'|'+rejects+'|'+receiver+'|'+(out===result)
"###,
        Value::String("CRINDVAGXYJ|true|1|1|1|true|true".into()),
    );
    observe(
        "post_yield_failures_close_preserving_original_reason/then_get__primitive_result",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},closeToken={},seen,result={},out,gets=0,calls=0,rejects=0,receiver=true,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=receiver&&this===undefined;rejects++;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='A';return {get then(){trace+='G';throw token;}};};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
it.next=function(){trace+='N';return {get done(){trace+='D';return false;},get value(){trace+='V';return 1;}};};
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';return function(){'use strict';receiver=receiver&&this===it;calls++;trace+='Y';return 0;};}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+calls+'|'+rejects+'|'+receiver+'|'+(out===result)
"###,
        Value::String("CRINDVAGXYJ|true|1|1|1|true|true".into()),
    );
    observe(
        "post_yield_failures_close_preserving_original_reason/then_get__object_result",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},closeToken={},seen,result={},out,gets=0,calls=0,rejects=0,receiver=true,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=receiver&&this===undefined;rejects++;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='A';return {get then(){trace+='G';throw token;}};};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
it.next=function(){trace+='N';return {get done(){trace+='D';return false;},get value(){trace+='V';return 1;}};};
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';return function(){'use strict';receiver=receiver&&this===it;calls++;trace+='Y';return {};};}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+calls+'|'+rejects+'|'+receiver+'|'+(out===result)
"###,
        Value::String("CRINDVAGXYJ|true|1|1|1|true|true".into()),
    );
    observe(
        "post_yield_failures_close_preserving_original_reason/then_call__absent",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},closeToken={},seen,result={},out,gets=0,calls=0,rejects=0,receiver=true,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=receiver&&this===undefined;rejects++;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='A';return {get then(){trace+='G';return function(){trace+='T';throw token;};}};};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
it.next=function(){trace+='N';return {get done(){trace+='D';return false;},get value(){trace+='V';return 1;}};};

out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+calls+'|'+rejects+'|'+receiver+'|'+(out===result)
"###,
        Value::String("CRINDVAGTJ|true|0|0|1|true|true".into()),
    );
    observe(
        "post_yield_failures_close_preserving_original_reason/then_call__undefined",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},closeToken={},seen,result={},out,gets=0,calls=0,rejects=0,receiver=true,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=receiver&&this===undefined;rejects++;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='A';return {get then(){trace+='G';return function(){trace+='T';throw token;};}};};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
it.next=function(){trace+='N';return {get done(){trace+='D';return false;},get value(){trace+='V';return 1;}};};
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';return undefined;}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+calls+'|'+rejects+'|'+receiver+'|'+(out===result)
"###,
        Value::String("CRINDVAGTXJ|true|1|0|1|true|true".into()),
    );
    observe(
        "post_yield_failures_close_preserving_original_reason/then_call__null",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},closeToken={},seen,result={},out,gets=0,calls=0,rejects=0,receiver=true,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=receiver&&this===undefined;rejects++;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='A';return {get then(){trace+='G';return function(){trace+='T';throw token;};}};};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
it.next=function(){trace+='N';return {get done(){trace+='D';return false;},get value(){trace+='V';return 1;}};};
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';return null;}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+calls+'|'+rejects+'|'+receiver+'|'+(out===result)
"###,
        Value::String("CRINDVAGTXJ|true|1|0|1|true|true".into()),
    );
    observe(
        "post_yield_failures_close_preserving_original_reason/then_call__noncallable",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},closeToken={},seen,result={},out,gets=0,calls=0,rejects=0,receiver=true,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=receiver&&this===undefined;rejects++;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='A';return {get then(){trace+='G';return function(){trace+='T';throw token;};}};};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
it.next=function(){trace+='N';return {get done(){trace+='D';return false;},get value(){trace+='V';return 1;}};};
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';return 0;}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+calls+'|'+rejects+'|'+receiver+'|'+(out===result)
"###,
        Value::String("CRINDVAGTXJ|true|1|0|1|true|true".into()),
    );
    observe(
        "post_yield_failures_close_preserving_original_reason/then_call__getter_throw",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},closeToken={},seen,result={},out,gets=0,calls=0,rejects=0,receiver=true,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=receiver&&this===undefined;rejects++;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='A';return {get then(){trace+='G';return function(){trace+='T';throw token;};}};};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
it.next=function(){trace+='N';return {get done(){trace+='D';return false;},get value(){trace+='V';return 1;}};};
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';throw closeToken;}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+calls+'|'+rejects+'|'+receiver+'|'+(out===result)
"###,
        Value::String("CRINDVAGTXJ|true|1|0|1|true|true".into()),
    );
    observe(
        "post_yield_failures_close_preserving_original_reason/then_call__call_throw",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},closeToken={},seen,result={},out,gets=0,calls=0,rejects=0,receiver=true,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=receiver&&this===undefined;rejects++;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='A';return {get then(){trace+='G';return function(){trace+='T';throw token;};}};};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
it.next=function(){trace+='N';return {get done(){trace+='D';return false;},get value(){trace+='V';return 1;}};};
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';return function(){'use strict';receiver=receiver&&this===it;calls++;trace+='Y';throw closeToken;};}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+calls+'|'+rejects+'|'+receiver+'|'+(out===result)
"###,
        Value::String("CRINDVAGTXYJ|true|1|1|1|true|true".into()),
    );
    observe(
        "post_yield_failures_close_preserving_original_reason/then_call__primitive_result",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},closeToken={},seen,result={},out,gets=0,calls=0,rejects=0,receiver=true,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=receiver&&this===undefined;rejects++;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='A';return {get then(){trace+='G';return function(){trace+='T';throw token;};}};};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
it.next=function(){trace+='N';return {get done(){trace+='D';return false;},get value(){trace+='V';return 1;}};};
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';return function(){'use strict';receiver=receiver&&this===it;calls++;trace+='Y';return 0;};}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+calls+'|'+rejects+'|'+receiver+'|'+(out===result)
"###,
        Value::String("CRINDVAGTXYJ|true|1|1|1|true|true".into()),
    );
    observe(
        "post_yield_failures_close_preserving_original_reason/then_call__object_result",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},closeToken={},seen,result={},out,gets=0,calls=0,rejects=0,receiver=true,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){'use strict';receiver=receiver&&this===undefined;rejects++;trace+='J';seen=e;});return result;}
Object.defineProperty(C,'resolve',{get:function(){trace+='R';return function(){trace+='A';return {get then(){trace+='G';return function(){trace+='T';throw token;};}};};}});
input[Symbol.iterator]=function(){trace+='I';return it;};
it.next=function(){trace+='N';return {get done(){trace+='D';return false;},get value(){trace+='V';return 1;}};};
Object.defineProperty(it,'return',{get:function(){gets++;trace+='X';return function(){'use strict';receiver=receiver&&this===it;calls++;trace+='Y';return {};};}});
out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+gets+'|'+calls+'|'+rejects+'|'+receiver+'|'+(out===result)
"###,
        Value::String("CRINDVAGTXYJ|true|1|1|1|true|true".into()),
    );
}

#[test]
fn promise_race_primitive_then_uses_getv_and_original_receiver() {
    observe(
        "primitive_then_uses_getv_and_original_receiver/number_receiver",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',result={},out,receiver=false,identities=false,ignored=0;
function resolve(v){trace+='F'+v;}
function reject(v){trace+='J'+v;}
function C(executor){executor(resolve,reject);return result;}
C.resolve=function(){return 7;};
Object.defineProperty(Number.prototype,'then',{configurable:true,value:function(f,j){'use strict';receiver=this===7;identities=f===resolve&&j===reject&&arguments.length===2;f(9);j(10);return {get then(){ignored++;throw 'ignored then return';}};}});
out=Promise.race.call(C,[1]);delete Number.prototype.then;
"###,
        r###"trace+'|'+receiver+'|'+identities+'|'+ignored+'|'+(out===result)
"###,
        Value::String("F9J10|true|true|0|true".into()),
    );
}

#[test]
fn promise_race_resolve_then_throw_obeys_capability_behavior() {
    observe(
        "resolve_then_throw_obeys_capability_behavior/custom_callbacks",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},seen,result={},out,closed=0,it={},input={};
function C(executor){executor(function(v){trace+='F'+v;},function(e){trace+='J';seen=e;});return result;}
C.resolve=function(){return {then:function(f){f(7);throw token;}};};
it.next=function(){return {done:false,value:1};};it.return=function(){closed++;return {};};
input[Symbol.iterator]=function(){return it;};out=Promise.race.call(C,input);
"###,
        r###"trace+'|'+(seen===token)+'|'+closed+'|'+(out===result)
"###,
        Value::String("F7J|true|1|true".into()),
    );
    observe(
        "resolve_then_throw_obeys_capability_behavior/intrinsic_first_settlement",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},closed=0,it={},input={},original=Promise.resolve;
Promise.resolve=function(){return {then:function(f){f(7);throw token;}};};
it.next=function(){return {done:false,value:1};};it.return=function(){closed++;return {};};
input[Symbol.iterator]=function(){return it;};var result=Promise.race(input);
Promise.resolve=original;
result.then(function(v){trace+='W'+v;},function(){trace+='BAD';});
"###,
        r###"trace+'|'+closed
"###,
        Value::String("W7|1".into()),
    );
}

#[test]
fn promise_race_custom_reject_throw_escapes_once() {
    observe(
        "custom_reject_throw_escapes_once/reject_throw",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},replacement={},seen=false,caught=false,closed=0,rejects=0,it={},input={};
function C(executor){trace+='C';executor(function(){trace+='BAD';},function(e){rejects++;trace+='J';seen=e===token;throw replacement;});}
C.resolve=function(){trace+='A';throw token;};
it.next=function(){trace+='N';return {done:false,value:1};};it.return=function(){closed++;trace+='R';return {};};
input[Symbol.iterator]=function(){trace+='I';return it;};
try{Promise.race.call(C,input);}catch(e){caught=e===replacement;trace+='E';}
"###,
        r###"trace+'|'+seen+'|'+caught+'|'+closed+'|'+rejects
"###,
        Value::String("CINARJE|true|true|1|1".into()),
    );
}

#[test]
fn promise_race_saved_callbacks_preserve_identity_repetition_returns_and_throw() {
    observe(
        "saved_callbacks_preserve_identity_repetition_returns_and_throw/saved",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',token={},caught=false,result={},out,f,j,returns=[];
function resolve(v){trace+='F'+v;if(v===4)throw token;return v+100;}
function reject(v){trace+='J'+v;return v+200;}
function C(executor){executor(resolve,reject);return result;}
C.resolve=function(v){return v;};
out=Promise.race.call(C,[{then:function(a,b){f=a;j=b;}}]);
returns.push(f(1));returns.push(f(2));returns.push(j(3));try{f(4);}catch(e){caught=e===token;}
"###,
        r###"trace+'|'+returns.join(',')+'|'+caught+'|'+(f===resolve&&j===reject)+'|'+(out===result)
"###,
        Value::String("F1F2J3F4|101,102,203|true|true|true".into()),
    );
}

#[test]
fn promise_race_nested_race_keeps_outer_iterator_and_capability_independent() {
    observe(
        "nested_race_keeps_outer_iterator_and_capability_independent/nested",
        r###"if(typeof Promise.race!=='function')throw new Error('Promise.race missing');
var trace='',outer={},inner={},out,bad=false;
function outerResolve(v){trace+='F'+v;}
function innerResolve(v){trace+='d'+v;}
function reject(){bad=true;}
function C(executor){trace+='C';executor(outerResolve,reject);return outer;}
function D(executor){trace+='D';executor(innerResolve,reject);return inner;}
D.resolve=function(v){trace+='B'+v;return {then:function(f,j){if(f!==innerResolve||j!==reject)bad=true;trace+='U';f(v);}};};
C.resolve=function(v){trace+='A'+v;if(v===1&&Promise.race.call(D,[9])!==inner)bad=true;return {then:function(f,j){if(f!==outerResolve||j!==reject)bad=true;trace+='T';f(v);}};};
out=Promise.race.call(C,[1,2]);
"###,
        r###"trace+'|'+bad+'|'+(out===outer)
"###,
        Value::String("CA1DB9Ud9TF1A2TF2|false|true".into()),
    );
}
