//! Promise.all public literals copied exactly from the held source-only plan.
//! Each case uses a fresh Runtime per mode; no new private APIs or quota cuts.
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
fn promise_all_ordinary_method_metadata() {
    observe(
        "ordinary_method_metadata",
        r###"var d=Object.getOwnPropertyDescriptor(Promise,'all');var f=d.value;var n=Object.getOwnPropertyDescriptor(f,'name');var l=Object.getOwnPropertyDescriptor(f,'length');var noNew=false;try{new f([]);}catch(e){noNew=e instanceof TypeError;}var check=[f===Promise.all,d.writable,!d.enumerable,d.configurable,f.name==='all',f.length===1,!n.writable,!n.enumerable,n.configurable,!l.writable,!l.enumerable,l.configurable,!Object.prototype.hasOwnProperty.call(f,'prototype'),Object.getPrototypeOf(f)===Function.prototype,noNew].every(function(v){return v;});"###,
        r###"check"###,
        Value::Bool(true),
    );
}

#[test]
fn promise_all_empty_fulfillment_precedes_later_job() {
    observe(
        "empty_fulfillment_precedes_later_job",
        r###"var log='';Promise.all([]).then(function(v){log+='E'+v.length;});queueMicrotask(function(){log+='Q';});log+='S';"###,
        r###"log"###,
        Value::String("SE0Q".into()),
    );
}

#[test]
fn promise_all_ordered_results_after_out_of_order_settlement() {
    observe(
        "ordered_results_after_out_of_order_settlement",
        r###"var log='';var release;var marker={};var p=new Promise(function(r){release=r;});Promise.all([p,Promise.resolve(marker),3]).then(function(v){log+='A'+v[0]+':'+(v[1]===marker)+':'+v[2];});queueMicrotask(function(){log+='Q';});release(7);log+='S';"###,
        r###"log"###,
        Value::String("SQA7:true:3".into()),
    );
}

#[test]
fn promise_all_exact_rejection_with_pending_sibling() {
    observe(
        "exact_rejection_with_pending_sibling",
        r###"var marker={};var log='';Promise.all([Promise.reject(marker),new Promise(function(){})]).then(function(){log+='bad';},function(e){log+='R'+(e===marker);});queueMicrotask(function(){log+='Q';});log+='S';"###,
        r###"log"###,
        Value::String("SQRtrue".into()),
    );
}

#[test]
fn promise_all_default_resolve_thenable_getter_and_job() {
    observe(
        "default_resolve_thenable_getter_and_job",
        r###"var log='';var t={get then(){log+='G';return function(r){log+='T';r(4);r(9);};}};Promise.all([t]).then(function(v){log+='A'+v[0];});queueMicrotask(function(){log+='Q';});log+='S';"###,
        r###"log"###,
        Value::String("GSTQA4".into()),
    );
}

#[test]
fn promise_all_generic_capability_cached_resolve_iterator_order() {
    observe(
        "generic_capability_cached_resolve_iterator_order",
        r###"var log='',bad=false,reject,values;function C(executor){log+='C';executor(function(v){values=v;log+='F'+v.join(',');},reject=function(){});}Object.defineProperty(C,Symbol.species,{get:function(){throw 'species must not run';}});Object.defineProperty(C,'resolve',{configurable:true,get:function(){log+='G';return function(x){'use strict';bad=bad||this!==C||arguments.length!==1;log+='R'+x;Object.defineProperty(C,'resolve',{value:function(){throw 'uncached resolve';},configurable:true});var next={then:function(f,r){'use strict';bad=bad||this!==next||arguments.length!==2||r!==reject;log+='T'+x;f.call({ignored:true},x);f(99);}};return next;};}});var index=0,it={};Object.defineProperty(it,'next',{get:function(){log+='N';return function(){'use strict';bad=bad||this!==it||arguments.length!==0;log+='S';return index<2?{done:false,value:++index}:{done:true};};}});var iterable={};Object.defineProperty(iterable,Symbol.iterator,{get:function(){log+='I';return function(){'use strict';bad=bad||this!==iterable;log+='M';return it;};}});var result=Promise.all.call(C,iterable);log+='Z';"###,
        r###"log+'|'+bad+'|'+(result instanceof C)"###,
        Value::String("CGIMNSR1T1SR2T2SF1,2Z|false|true".into()),
    );
}

#[test]
fn promise_all_saved_fulfillment_once_return_and_shared_reject() {
    observe(
        "saved_fulfillment_once_return_and_shared_reject",
        r###"var callbacks=[],rejects=[],values,resolutions=0,rejections=0,marker={},returned={},meta=true;function C(executor){executor(function(v){resolutions++;values=v;callbacks[1]('ignored');return returned;},function(v){if(v===marker)rejections++;});}C.resolve=function(x){return {then:function(f,r){callbacks.push(f);rejects.push(r);meta=meta&&f.name===''&&f.length===1&&!Object.prototype.hasOwnProperty.call(f,'prototype');}};};var result=Promise.all.call(C,[0,1]);var first=callbacks[1].call({wrong:true},'B');rejects[0](marker);rejects[1](marker);var last=callbacks[0].call(null,'A');var duplicate=callbacks[0]('bad');"###,
        r###"[values.join(''),resolutions,rejections,rejects[0]===rejects[1],first===undefined,last===returned,duplicate===undefined,meta].join('|')"###,
        Value::String("AB|1|2|true|true|true|true|true".into()),
    );
}

#[test]
fn promise_all_resolve_getter_error_rejects_before_iterator() {
    observe(
        "resolve_getter_error_rejects_before_iterator",
        r###"var marker={},seen,log='';function C(executor){log+='C';executor(function(){},function(e){log+='J';seen=e;});}Object.defineProperty(C,'resolve',{get:function(){log+='G';throw marker;}});var iterable={};Object.defineProperty(iterable,Symbol.iterator,{get:function(){log+='I';throw 'must not run';}});var result=Promise.all.call(C,iterable);"###,
        r###"log+'|'+(seen===marker)+'|'+(result instanceof C)"###,
        Value::String("CGJ|true|true".into()),
    );
}

#[test]
fn promise_all_invalid_constructor_precedes_property_access() {
    observe(
        "invalid_constructor_precedes_property_access",
        r###"var count=0,bad={};Object.defineProperty(bad,'resolve',{get:function(){count++;throw 1;}});var caught=false;try{Promise.all.call(bad,[]);}catch(e){caught=e instanceof TypeError;}"###,
        r###"caught+'|'+count"###,
        Value::String("true|0".into()),
    );
}

#[test]
fn promise_all_capability_constructor_throw_is_exact() {
    observe(
        "capability_constructor_throw_is_exact",
        r###"var marker={},seen=false,reads=0;function C(){throw marker;}Object.defineProperty(C,'resolve',{get:function(){reads++;return function(){};}});try{Promise.all.call(C,[]);}catch(e){seen=e===marker;}"###,
        r###"seen+'|'+reads"###,
        Value::String("true|0".into()),
    );
}

#[test]
fn promise_all_custom_reject_does_not_short_circuit_iteration() {
    observe(
        "custom_reject_does_not_short_circuit_iteration",
        r###"var marker={},seen=[],rejections=0,resolutions=0;function C(executor){executor(function(){resolutions++;},function(e){if(e===marker)rejections++;});}C.resolve=function(x){seen.push(x);return {then:function(f,r){if(x===1)r(marker);else f(x);}};};var result=Promise.all.call(C,[1,2,3]);"###,
        r###"seen.join(',')+'|'+rejections+'|'+resolutions+'|'+(result instanceof C)"###,
        Value::String("1,2,3|1|0|true".into()),
    );
}

#[test]
fn promise_all_primitive_next_promise_uses_getv_and_receiver() {
    observe(
        "primitive_next_promise_uses_getv_and_receiver",
        r###"var values,correct=false;function C(executor){executor(function(v){values=v;},function(e){throw e;});}C.resolve=function(){return 7;};Object.defineProperty(Number.prototype,'then',{configurable:true,value:function(f){'use strict';correct=this===7;f(9);}});Promise.all.call(C,[1]);delete Number.prototype.then;"###,
        r###"correct+'|'+values[0]"###,
        Value::String("true|9".into()),
    );
}

#[test]
fn promise_all_intrinsic_result_array_ignores_global_constructor_and_index_setter() {
    observe(
        "intrinsic_result_array_ignores_global_constructor_and_index_setter",
        r###"var NativeArray=Array,values,setter=0;function C(executor){executor(function(v){values=v;},function(e){throw e;});}C.resolve=function(x){return {then:function(f){f(x);}};};Object.defineProperty(NativeArray.prototype,'0',{configurable:true,set:function(){setter++;}});Array=function(){throw 'global Array must not run';};Promise.all.call(C,[4,5]);Array=NativeArray;delete NativeArray.prototype[0];"###,
        r###"[NativeArray.isArray(values),Object.getPrototypeOf(values)===NativeArray.prototype,values.length,values[0],values[1],Object.prototype.hasOwnProperty.call(values,'0'),setter].join('|')"###,
        Value::String("true|true|2|4|5|true|0".into()),
    );
}

#[test]
fn promise_all_strings_and_sparse_arrays_use_sync_iterator() {
    observe(
        "strings_and_sparse_arrays_use_sync_iterator",
        r###"var a,b;Promise.all('a𝄞').then(function(v){a=v;});Promise.all([,3]).then(function(v){b=v;});"###,
        r###"[a.length,a[0],a[1].length,b.length,b[0]===undefined,b[1]].join('|')"###,
        Value::String("2|a|2|2|true|3".into()),
    );
}

#[test]
fn promise_all_array_like_is_not_an_iterable_fallback() {
    observe(
        "array_like_is_not_an_iterable_fallback",
        r###"var ok=false;Promise.all({0:4,length:1}).then(function(){},function(e){ok=e instanceof TypeError;});"###,
        r###"ok"###,
        Value::Bool(true),
    );
}

#[test]
fn promise_all_empty_custom_resolve_throw_is_rejected() {
    observe(
        "empty_custom_resolve_throw_is_rejected",
        r###"var marker={},seen,log='';function C(executor){executor(function(v){log+='R'+v.length;throw marker;},function(e){log+='J';seen=e;});}C.resolve=function(){throw 'empty input must not call resolve';};var result=Promise.all.call(C,[]);"###,
        r###"log+'|'+(seen===marker)+'|'+(result instanceof C)"###,
        Value::String("R0J|true|true".into()),
    );
}

#[test]
fn promise_all_custom_reject_throw_escapes_exactly() {
    observe(
        "custom_reject_throw_escapes_exactly",
        r###"var original={},replacement={},seen=false;function C(executor){executor(function(){},function(e){if(e!==original)throw 'wrong reason';throw replacement;});}Object.defineProperty(C,'resolve',{get:function(){throw original;}});try{Promise.all.call(C,[]);}catch(e){seen=e===replacement;}"###,
        r###"seen"###,
        Value::Bool(true),
    );
}

#[test]
fn promise_all_iterator_next_throw_does_not_close() {
    observe(
        "iterator_next_throw_does_not_close",
        r###"var marker={},closeMarker={},seen,returns=0;function C(executor){executor(function(){},function(e){seen=e;});}var it={next:function(){return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returns++;throw closeMarker;}});var src={};src[Symbol.iterator]=function(){return it;};C.resolve=function(x){return {then:function(){throw marker;}};};it.next=function(){throw marker;};Promise.all.call(C,src);"###,
        r###"(seen===marker)+'|'+returns"###,
        Value::String("true|0".into()),
    );
}

#[test]
fn promise_all_iterator_done_getter_throw_does_not_close() {
    observe(
        "iterator_done_getter_throw_does_not_close",
        r###"var marker={},closeMarker={},seen,returns=0;function C(executor){executor(function(){},function(e){seen=e;});}var it={next:function(){return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returns++;throw closeMarker;}});var src={};src[Symbol.iterator]=function(){return it;};C.resolve=function(x){return {then:function(){throw marker;}};};it.next=function(){return {get done(){throw marker;}};};Promise.all.call(C,src);"###,
        r###"(seen===marker)+'|'+returns"###,
        Value::String("true|0".into()),
    );
}

#[test]
fn promise_all_iterator_value_getter_throw_does_not_close() {
    observe(
        "iterator_value_getter_throw_does_not_close",
        r###"var marker={},closeMarker={},seen,returns=0;function C(executor){executor(function(){},function(e){seen=e;});}var it={next:function(){return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returns++;throw closeMarker;}});var src={};src[Symbol.iterator]=function(){return it;};C.resolve=function(x){return {then:function(){throw marker;}};};it.next=function(){return {done:false,get value(){throw marker;}};};Promise.all.call(C,src);"###,
        r###"(seen===marker)+'|'+returns"###,
        Value::String("true|0".into()),
    );
}

#[test]
fn promise_all_resolve_call_throw_closes_preserving_reason() {
    observe(
        "resolve_call_throw_closes_preserving_reason",
        r###"var marker={},closeMarker={},seen,returns=0;function C(executor){executor(function(){},function(e){seen=e;});}var it={next:function(){return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returns++;throw closeMarker;}});var src={};src[Symbol.iterator]=function(){return it;};C.resolve=function(x){return {then:function(){throw marker;}};};C.resolve=function(){throw marker;};Promise.all.call(C,src);"###,
        r###"(seen===marker)+'|'+returns"###,
        Value::String("true|1".into()),
    );
}

#[test]
fn promise_all_then_getter_throw_closes_preserving_reason() {
    observe(
        "then_getter_throw_closes_preserving_reason",
        r###"var marker={},closeMarker={},seen,returns=0;function C(executor){executor(function(){},function(e){seen=e;});}var it={next:function(){return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returns++;throw closeMarker;}});var src={};src[Symbol.iterator]=function(){return it;};C.resolve=function(x){return {then:function(){throw marker;}};};C.resolve=function(){return {get then(){throw marker;}};};Promise.all.call(C,src);"###,
        r###"(seen===marker)+'|'+returns"###,
        Value::String("true|1".into()),
    );
}

#[test]
fn promise_all_then_call_throw_closes_preserving_reason() {
    observe(
        "then_call_throw_closes_preserving_reason",
        r###"var marker={},closeMarker={},seen,returns=0;function C(executor){executor(function(){},function(e){seen=e;});}var it={next:function(){return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returns++;throw closeMarker;}});var src={};src[Symbol.iterator]=function(){return it;};C.resolve=function(x){return {then:function(){throw marker;}};};Promise.all.call(C,src);"###,
        r###"(seen===marker)+'|'+returns"###,
        Value::String("true|1".into()),
    );
}

#[test]
fn promise_all_iterator_next_getter_acquisition_throw_does_not_close() {
    observe(
        "iterator_next_getter_acquisition_throw_does_not_close",
        r###"var marker={},closeMarker={},seen,returns=0;function C(executor){executor(function(){},function(e){seen=e;});}var it={next:function(){return {done:false,value:1};}};Object.defineProperty(it,'return',{get:function(){returns++;throw closeMarker;}});var src={};src[Symbol.iterator]=function(){return it;};C.resolve=function(x){return {then:function(){throw marker;}};};Object.defineProperty(it,'next',{get:function(){throw marker;}});Promise.all.call(C,src);"###,
        r###"(seen===marker)+'|'+returns"###,
        Value::String("true|0".into()),
    );
}
