//! Additive Promise.any observable input/capability coverage.
//! Exact frozen source literals; prior public and terminal modules unchanged.
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
fn promise_any_primitive_then_receiver_and_ignored_return() {
    observe(
        "any_primitive_then_receiver_and_ignored_return",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
if(typeof Promise.any!=='function')throw new Error('Promise.any missing');
function check(v){if(!v)throw new Error('oracle');}
var trace='',holder={},returned={},payload={},reason={},out,error,receiver=false,identities=false,ignored=0;
function resolve(v){'use strict';check(this===undefined&&arguments.length===1&&v===payload);trace+='F';return returned;}
function C(ex){trace+='C';ex(resolve,function(e){'use strict';check(this===undefined&&arguments.length===1);trace+='J';error=e;});return holder;}
C.resolve=function(v){'use strict';check(this===C&&arguments.length===1&&v===payload);trace+='Q';return 7;};
Object.defineProperty(Number.prototype,'then',{configurable:true,value:function(f,r){'use strict';receiver=this===7;identities=f===resolve&&typeof r==='function'&&r!==f&&arguments.length===2;trace+='T';check(f(payload)===returned);check(r(reason)===undefined);return {get then(){ignored++;throw new Error('ignored then return');}};}});
out=Promise.any.call(C,[payload]);delete Number.prototype.then;
"###,
        r###"[trace,receiver,identities,ignored,out===holder,error instanceof AggregateError&&error.errors[0]===reason].join('|')"###,
        Value::String("CQTFJ|true|true|0|true|true".into()),
    );
}

#[test]
fn promise_any_strings_code_points_and_sparse_array_holes() {
    observe(
        "any_strings_code_points_and_sparse_array_holes",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
if(typeof Promise.any!=='function')throw new Error('Promise.any missing');
function check(v){if(!v)throw new Error('oracle');}
var text='pending',hole='pending';
Promise.any('𝄞a').then(function(v){text=v;},function(){text='wrong';});
Promise.any([,Promise.reject({})]).then(function(v){hole=v;},function(){hole='wrong';});
"###,
        r###"[text==='𝄞',text.length,hole===undefined].join('|')"###,
        Value::String("true|2|true".into()),
    );
}

#[test]
fn promise_any_invalid_inputs_reject_without_array_like_fallback_or_close() {
    observe(
        "any_invalid_inputs_reject_without_array_like_fallback_or_close",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
if(typeof Promise.any!=='function')throw new Error('Promise.any missing');
function check(v){if(!v)throw new Error('oracle');}
var closed=0,results=[],after=false;
var badMethod={};badMethod[Symbol.iterator]=1;
var badIterator={};badIterator[Symbol.iterator]=function(){return 1;};
var malformed={next:1};Object.defineProperty(malformed,'return',{get:function(){closed++;throw new Error('close');}});
var badNext={};badNext[Symbol.iterator]=function(){return malformed;};
var inputs=[undefined,null,1,{},{0:4,length:1},badMethod,badIterator,badNext];
for(var i=0;i<inputs.length;i++)(function(index){var p=Promise.any(inputs[index]);check(p instanceof Promise);p.then(function(){results[index]='fulfilled';},function(e){check(e instanceof TypeError);results[index]='TypeError';});})(i);
after=true;
"###,
        r###"[after,results.length,results.join(','),closed].join('|')"###,
        Value::String("true|8|TypeError,TypeError,TypeError,TypeError,TypeError,TypeError,TypeError,TypeError|0".into()),
    );
}

#[test]
fn promise_any_capability_and_noncallable_resolve_precede_iterator() {
    observe(
        "any_capability_and_noncallable_resolve_precede_iterator",
        r###"if(typeof AggregateError!=='function')throw new Error('AggregateError missing');
if(typeof Promise.any!=='function')throw new Error('Promise.any missing');
function check(v){if(!v)throw new Error('oracle');}
var rows=[];
for(var stage=0;stage<3;stage++){
 var trace='',token={},caught=false,holder={},input={};
 function C(ex){trace+='C';if(stage===0)throw token;if(stage===1)ex(0,function(){});else ex(function(){},0);return holder;}
 Object.defineProperty(C,'resolve',{get:function(){trace+='R';throw new Error('late resolve');}});
 Object.defineProperty(input,Symbol.iterator,{get:function(){trace+='I';throw new Error('late iterator');}});
 try{Promise.any.call(C,input);}catch(e){caught=stage===0?e===token:e instanceof TypeError;}
 rows.push(trace+':'+caught);
}
var badValues=[undefined,null,0,{}];
for(var index=0;index<badValues.length;index++){
 var trace='',seen,receiver=false,calls=0,holder={},input={};
 function C(ex){trace+='C';ex(function(){throw new Error('fulfill');},function(e){'use strict';receiver=this===undefined&&arguments.length===1;calls++;trace+='J';seen=e;});return holder;}
 Object.defineProperty(C,'resolve',{get:function(){trace+='R';return badValues[index];}});
 Object.defineProperty(input,Symbol.iterator,{get:function(){trace+='I';throw new Error('late iterator');}});
 var out=Promise.any.call(C,input);rows.push(trace+':'+(seen instanceof TypeError&&receiver&&calls===1&&out===holder));
}
var result=rows.join(',');
"###,
        r###"result"###,
        Value::String("C:true,C:true,C:true,CRJ:true,CRJ:true,CRJ:true,CRJ:true".into()),
    );
}
