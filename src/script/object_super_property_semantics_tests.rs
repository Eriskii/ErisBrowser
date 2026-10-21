//! Frozen independent object-method SuperProperty literals.
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

fn observe(case_id: &str, source: &str, readback: &str, sloppy: &str, strict_expected: &str) {
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        execute_mode(&mut runtime, &mut document, strict, source)
            .unwrap_or_else(|error| panic!("{case_id} strict={strict} source: {error:?}"));
        let result = execute_mode(&mut runtime, &mut document, strict, readback)
            .unwrap_or_else(|error| panic!("{case_id} strict={strict} readback: {error:?}"));
        let expected = if strict { strict_expected } else { sloppy };
        assert_eq!(
            result,
            Value::String(expected.into()),
            "{case_id} strict={strict}"
        );
    }
}

fn early_error(case_id: &str, setup: &str, source: &str, readback: &str) {
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        execute_mode(&mut runtime, &mut document, strict, setup).unwrap();
        let error = execute_mode(&mut runtime, &mut document, strict, source)
            .expect_err("invalid SuperProperty context completed");
        assert_eq!(
            error.intrinsic_error_name(),
            Some("SyntaxError"),
            "{case_id} strict={strict}"
        );
        assert!(
            error.is_parse_error(),
            "{case_id} strict={strict}: {error:?}"
        );
        let observed = execute_mode(&mut runtime, &mut document, strict, readback).unwrap();
        assert_eq!(observed, Value::Number(0.0), "{case_id} strict={strict}");
    }
}

fn deferred(case_id: &str, source: &str) {
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        let error = execute_mode(&mut runtime, &mut document, strict, source)
            .expect_err("deferred valid syntax completed");
        assert!(
            error.is_unsupported(),
            "{case_id} strict={strict}: {error:?}"
        );
        // No JS readback after a terminal Unsupported refusal.
    }
}

#[test]
fn object_super_property_home_receiver_copy_bind_live() {
    observe(
        r####"home_receiver_copy_bind_live"####,
        r####"function check(v){if(!v)throw new Error('object super oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var a={get x(){return 'A'+this.tag;},f(){return this;}},b={get x(){return 'B'+this.tag;},f(){return this;}};
var home={__proto__:a,read(){return super.x;},call(){return super.f();}};
var receiver={tag:'R',x:'shadow',f:function(){throw 'shadow';}},copy={tag:'C',read:home.read};
Object.setPrototypeOf(receiver,{x:'wrong'});Object.setPrototypeOf(copy,{x:'wrong'});
var saved=home.read,bound=saved.bind(receiver);
check(saved.call(receiver)==='AR'&&copy.read()==='AC'&&bound()==='AR');
check(home.call.call(receiver)===receiver);
Object.setPrototypeOf(home,b);delete home.read;
check(saved.call(receiver)==='BR'&&copy.read()==='BC'&&bound()==='BR');
return 'home:copy:bind:live';
})();
"####,
        r####"result
"####,
        r####"home:copy:bind:live"####,
        r####"home:copy:bind:live"####,
    );
}

#[test]
fn object_super_property_repeated_evaluation_separate_homes() {
    observe(
        r####"repeated_evaluation_separate_homes"####,
        r####"function check(v){if(!v)throw new Error('object super oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
function make(parent){return {read(){return super.x;},__proto__:parent};}
var a=make({x:1}),b=make({x:2}),fa=a.read,fb=b.read;
check(fa!==fb&&fa()===1&&fb()===2);
Object.setPrototypeOf(a,{x:3});check(fa.call(b)===3&&fb.call(a)===2);
Object.setPrototypeOf(b,null);expect(TypeError,function(){fb();});check(fa()===3);
return '1:2:3:null';
})();
"####,
        r####"result
"####,
        r####"1:2:3:null"####,
        r####"1:2:3:null"####,
    );
}

#[test]
fn object_super_property_inherited_method_keeps_its_own_home() {
    observe(
        r####"inherited_method_keeps_its_own_home"####,
        r####"function check(v){if(!v)throw new Error('object super oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var grand={get x(){return 'G'+this.tag;}},mid={__proto__:grand,m(){return super.x;}};
var home={__proto__:mid,m(){return super.m();}},receiver={tag:'R'};
check(home.m.call(receiver)==='GR');
Object.setPrototypeOf(mid,{get x(){return 'N'+this.tag;}});check(home.m.call(receiver)==='NR');
Object.setPrototypeOf(home,{m:mid.m});check(home.m.call(receiver)==='NR');
return 'GR:NR:NR';
})();
"####,
        r####"result
"####,
        r####"GR:NR:NR"####,
        r####"GR:NR:NR"####,
    );
}

#[test]
fn object_super_property_accessor_merge_copy_and_receiver() {
    observe(
        r####"accessor_merge_copy_and_receiver"####,
        r####"function check(v){if(!v)throw new Error('object super oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',base={get x(){trace+='G';return this.tag;},set x(v){trace+='S';this.saved=v;}};
var home={__proto__:base,get slot(){return super.x;},set slot(v){super.x=v;},get slot(){trace+='g';return super.x;}};
var descriptor=Object.getOwnPropertyDescriptor(home,'slot'),copy={tag:7};
check(descriptor.enumerable&&descriptor.configurable&&descriptor.get.name==='get slot'&&descriptor.set.name==='set slot');
Object.defineProperty(copy,'slot',descriptor);check(copy.slot===7);copy.slot=8;check(copy.saved===8);
Object.setPrototypeOf(home,{get x(){trace+='N';return this.tag+1;},set x(v){trace+='T';this.saved=v+1;}});
check(copy.slot===8);copy.slot=9;check(copy.saved===10);
return trace;
})();
"####,
        r####"result
"####,
        r####"gGSgNT"####,
        r####"gGSgNT"####,
    );
}

#[test]
fn object_super_property_escaped_arrows_from_methods_and_accessors() {
    observe(
        r####"escaped_arrows_from_methods_and_accessors"####,
        r####"function check(v){if(!v)throw new Error('object super oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',base={get x(){trace+='G';return this.tag;},set x(v){trace+='S';this.saved=v;}};
var savedSetter,home={__proto__:base,make(){return ()=>super.x;},get arrow(){return ()=>super.x;},set arrow(v){savedSetter=()=>super.x=v;}};
var r={tag:4},get=home.make.call(r),d=Object.getOwnPropertyDescriptor(home,'arrow'),access=d.get.call(r);
d.set.call(r,5);check(get.call({tag:99})===4&&access()===4);savedSetter();check(r.saved===5);
Object.setPrototypeOf(home,{get x(){trace+='N';return this.tag+1;}});check(get()===5&&access()===5);
return trace;
})();
"####,
        r####"result
"####,
        r####"GGSNN"####,
        r####"GGSNN"####,
    );
}

#[test]
fn object_super_property_method_and_arrow_parameter_defaults() {
    observe(
        r####"method_and_arrow_parameter_defaults"####,
        r####"function check(v){if(!v)throw new Error('object super oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',base={get x(){trace+='G';return this.tag;}};
var home={__proto__:base,m(v=super.x,f=()=>super.x){trace+='B';return [v,f];},make(){return (v=super.x)=>v;},set slot(v=super.x){trace+='S';this.saved=v;}};
var r={tag:6},pair=home.m.call(r);check(pair[0]===6);r.tag=7;check(pair[1]()===7);
var arrow=home.make.call(r);check(arrow.call({tag:99})===7);check(arrow(8)===8);
Object.getOwnPropertyDescriptor(home,'slot').set.call(r,undefined);check(r.saved===7);
return trace;
})();
"####,
        r####"result
"####,
        r####"GBGGGS"####,
        r####"GBGGGS"####,
    );
}

#[test]
fn object_super_property_primitive_receivers_follow_mode() {
    observe(
        r####"primitive_receivers_follow_mode"####,
        r####"function check(v){if(!v)throw new Error('object super oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var base={get receiver(){'use strict';return this;}},home={__proto__:base,read(){return super.receiver;}};
var number=home.read.call(7),nil=home.read.call(null),undef=home.read.call(undefined);
if(typeof number==='object'){check(number.valueOf()===7&&nil===globalThis&&undef===globalThis);return 'boxed:global:global';}
check(number===7&&nil===null&&undef===undefined);return 'primitive:null:undefined';
})();
"####,
        r####"result
"####,
        r####"boxed:global:global"####,
        r####"primitive:null:undefined"####,
    );
}

#[test]
fn object_super_property_implicit_accessor_receivers_follow_mode() {
    observe(
        r####"implicit_accessor_receivers_follow_mode"####,
        r####"function check(v){if(!v)throw new Error('object super oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var seen,base={get x(){'use strict';return this;},set x(v){'use strict';seen=this;}},home={__proto__:base,get x(){return super.x;},set x(v){super.x=v;}};
var d=Object.getOwnPropertyDescriptor(home,'x'),got=d.get.call(7),nil=d.get.call(null);d.set.call(8,1);
if(typeof got==='object'){check(got.valueOf()===7&&typeof seen==='object'&&seen.valueOf()===8&&nil===globalThis);return 'boxed:boxed:global';}
check(got===7&&seen===8&&nil===null);return 'primitive:primitive:null';
})();
"####,
        r####"result
"####,
        r####"boxed:boxed:global"####,
        r####"primitive:primitive:null"####,
    );
}

#[test]
fn object_super_property_explicit_strict_method_and_accessor() {
    observe(
        r####"explicit_strict_method_and_accessor"####,
        r####"function check(v){if(!v)throw new Error('object super oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var seen,base={get x(){'use strict';return this;},set x(v){'use strict';seen=this;}};
var home={__proto__:base,m(){'use strict';return ()=>super.x;},get x(){'use strict';return super.x;},set x(v){'use strict';super.x=v;}};
check(home.m.call(7).call(99)===7);
var d=Object.getOwnPropertyDescriptor(home,'x');check(d.get.call(null)===null);d.set.call(8,1);check(seen===8);
return 'strict:7:null:8';
})();
"####,
        r####"result
"####,
        r####"strict:7:null:8"####,
        r####"strict:7:null:8"####,
    );
}

#[test]
fn object_super_property_failed_data_writes_follow_reference_strictness() {
    observe(
        r####"failed_data_writes_follow_reference_strictness"####,
        r####"function check(v){if(!v)throw new Error('object super oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var base={},home={__proto__:base,put(k,v){return super[k]=v;}};
Object.defineProperty(base,'locked',{value:1,writable:false});Object.defineProperty(base,'getonly',{get:function(){return 2;}});
var receiver={},other={};Object.defineProperty(other,'own',{get:function(){throw 'receiver getter';}});
var trace='';function attempt(r,k){try{check(home.put.call(r,k,9)===9);trace+='R';}catch(e){check(e instanceof TypeError);trace+='T';}}
attempt(receiver,'locked');attempt(receiver,'getonly');attempt(other,'own');
check(!Object.prototype.hasOwnProperty.call(receiver,'locked')&&!Object.prototype.hasOwnProperty.call(receiver,'getonly'));
check(typeof Object.getOwnPropertyDescriptor(other,'own').get==='function');return trace;
})();
"####,
        r####"result
"####,
        r####"RRR"####,
        r####"TTT"####,
    );
}

#[test]
fn object_super_property_primitive_data_write_boxing_and_strict_failure() {
    observe(
        r####"primitive_data_write_boxing_and_strict_failure"####,
        r####"function check(v){if(!v)throw new Error('object super oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var base={x:1},home={__proto__:base,write(){super.x=9;return this;}};
try{var r=home.write.call(7);check(typeof r==='object'&&r.valueOf()===7&&r.x===9);return 'boxed:9';}
catch(e){check(e instanceof TypeError);return 'TypeError';}
})();
"####,
        r####"result
"####,
        r####"boxed:9"####,
        r####"TypeError"####,
    );
}

#[test]
fn object_super_property_mapped_arguments_are_not_forced_strict() {
    observe(
        r####"mapped_arguments_are_not_forced_strict"####,
        r####"function check(v){if(!v)throw new Error('object super oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var home={__proto__:{'0':0},m(a){var args=arguments;var helper={__proto__:{'0':0},put(v){super[0]=v;}};helper.put.call(args,9);return a+':'+args[0];}};
return home.m(1);
})();
"####,
        r####"result
"####,
        r####"9:9"####,
        r####"1:9"####,
    );
}

#[test]
fn object_super_property_definition_key_outer_home_and_body_inner_home() {
    observe(
        r####"definition_key_outer_home_and_body_inner_home"####,
        r####"function check(v){if(!v)throw new Error('object super oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',key={};key[Symbol.toPrimitive]=function(hint){check(hint==='string');trace+='C';return 'made';};
var parent={get key(){trace+='K';return key;},x:'outer'};
var outer={__proto__:parent,build(){var inner={ [super.key](){trace+='M';return super.x;},__proto__:{x:'inner'}};trace+='B';check(super.x==='outer');return inner;}};
var inner=outer.build();check(inner.made()==='inner'&&inner.made.name==='made');return trace;
})();
"####,
        r####"result
"####,
        r####"KCBM"####,
        r####"KCBM"####,
    );
}

#[test]
fn object_super_property_key_expression_capture_and_conversion_order() {
    observe(
        r####"key_expression_capture_and_conversion_order"####,
        r####"function check(v){if(!v)throw new Error('object super oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',home,first={get x(){trace+='A';return 1;}},second={get x(){trace+='B';return 2;}},third={get x(){trace+='C';return 3;}},key={};
key[Symbol.toPrimitive]=function(){trace+='T';Object.setPrototypeOf(home,third);return 'x';};
home={__proto__:first,read(){return super[(trace+='K',Object.setPrototypeOf(home,second),key)];}};
check(home.read()===2);check(home.read.call({})===2);return trace;
})();
"####,
        r####"result
"####,
        r####"KTBKTB"####,
        r####"KTBKTB"####,
    );
}

#[test]
fn object_super_property_proto_colon_after_method_and_ordinary_proto_names() {
    observe(
        r####"proto_colon_after_method_and_ordinary_proto_names"####,
        r####"function check(v){if(!v)throw new Error('object super oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',base={x:4},home={m(){return super.x;},__proto__:(trace+='P',base),['__proto__']:(trace+='D',7)};
check(home.m()===4&&home.__proto__===7&&Object.getPrototypeOf(home)===base);
var accessor={__proto__:base,get __proto__(){return super.x;},set __proto__(v){super.x=v;}};
check(accessor.__proto__===4);accessor.__proto__=8;check(accessor.x===8&&base.x===4);
var method={__proto__:base,__proto__(){return super.x;}};check(method.__proto__()===4);
return trace;
})();
"####,
        r####"result
"####,
        r####"PD"####,
        r####"PD"####,
    );
}

#[test]
fn object_super_property_proto_primitive_ignored_null_and_no_coercion() {
    observe(
        r####"proto_primitive_ignored_null_and_no_coercion"####,
        r####"function check(v){if(!v)throw new Error('object super oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',p={x:5,toString(){throw 'coerced';},valueOf(){throw 'coerced';}};
var a={__proto__:p,m(){return super.x;}},b={__proto__:(trace+='P',17),m(){return super.toString;}},n={__proto__:null,m(){return super.x;}};
check(a.m()===5&&Object.getPrototypeOf(a)===p&&Object.getPrototypeOf(b)===Object.prototype);
check(b.m()===Object.prototype.toString);expect(TypeError,function(){n.m();});return trace;
})();
"####,
        r####"result
"####,
        r####"P"####,
        r####"P"####,
    );
}

#[test]
fn object_super_property_metadata_unicode_symbols_and_nonconstructability() {
    observe(
        r####"metadata_unicode_symbols_and_nonconstructability"####,
        r####"function check(v){if(!v)throw new Error('object super oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var symbol=Symbol('λ\uD800'),key='雪\uD800',base={x:3};
var home={__proto__:base,[key](a,b=super.x){return b;},get [symbol](){return super.x;},constructor(){return super.x;}};
var m=home[key],get=Object.getOwnPropertyDescriptor(home,symbol).get;
check(m.name===key&&m.length===1&&m(0)===3&&get.name==='get [λ\uD800]'&&get.length===0&&get.call(home)===3);
var d=Object.getOwnPropertyDescriptor(home,key),name=Object.getOwnPropertyDescriptor(m,'name');
check(d.value===m&&d.writable&&d.enumerable&&d.configurable&&!name.writable&&!name.enumerable&&name.configurable);
check(!Object.prototype.hasOwnProperty.call(m,'prototype')&&!Object.prototype.hasOwnProperty.call(home.constructor,'prototype'));
expect(TypeError,function(){new m();});expect(TypeError,function(){new home.constructor();});return 'metadata:unicode:symbol';
})();
"####,
        r####"result
"####,
        r####"metadata:unicode:symbol"####,
        r####"metadata:unicode:symbol"####,
    );
}

#[test]
fn object_super_property_nested_object_and_colon_arrow_contexts() {
    observe(
        r####"nested_object_and_colon_arrow_contexts"####,
        r####"function check(v){if(!v)throw new Error('object super oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var outer={__proto__:{x:'outer'},build(){var inner={__proto__:{x:'inner'},arrow:()=>super.x,m(){return super.x;}};check(super.x==='outer');return inner;}};
var inner=outer.build();check(inner.arrow()==='outer'&&inner.m()==='inner');
Object.setPrototypeOf(outer,{x:'live'});check(inner.arrow.call({})==='live'&&inner.m()==='inner');
return 'outer:inner:live';
})();
"####,
        r####"result
"####,
        r####"outer:inner:live"####,
        r####"outer:inner:live"####,
    );
}

#[test]
fn object_super_property_class_object_context_restoration_both_directions() {
    observe(
        r####"class_object_context_restoration_both_directions"####,
        r####"function check(v){if(!v)throw new Error('object super oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
class A { get x(){return 'class';} }
class C extends A { build(){var o={__proto__:{x:'object'},m(){return super.x;},arrow:()=>super.x};check(super.x==='class');return o;} }
var o=new C().build();check(o.m()==='object'&&o.arrow()==='class');
var outer={__proto__:{Base:A,key:'read',x:'outer'},build(){class D extends super.Base {[super.key](){return super.x;}}check(super.x==='outer');return D;}};
var D=outer.build();check(new D().read()==='class');return 'class:object:outer:class';
})();
"####,
        r####"result
"####,
        r####"class:object:outer:class"####,
        r####"class:object:outer:class"####,
    );
}

#[test]
fn object_super_property_ordinary_abrupts_preserve_identity_and_cleanup() {
    observe(
        r####"ordinary_abrupts_preserve_identity_and_cleanup"####,
        r####"function check(v){if(!v)throw new Error('object super oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',token={},base={get x(){trace+='G';throw token;},set x(v){trace+='S';throw token;}},home={__proto__:base,read(k){return super[k];},put(v){return super.x=v;}};
try{home.read('x');}catch(e){check(e===token);trace+='C';}finally{trace+='F';}
try{home.put(1);}catch(e){check(e===token);trace+='C';}finally{trace+='F';}
Object.setPrototypeOf(home,{x:8});check(home.read('x')===8);return trace;
})();
"####,
        r####"result
"####,
        r####"GCFSCF"####,
        r####"GCFSCF"####,
    );
}

#[test]
fn object_super_property_assignment_update_logical_and_loop_routes() {
    observe(
        r####"assignment_update_logical_and_loop_routes"####,
        r####"function check(v){if(!v)throw new Error('object super oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',stored=1,base={get x(){trace+='G';return stored;},set x(v){trace+='S';stored=v;}},home={__proto__:base,run(){
check((super.x+=2)===3);check(super.x++===3);check(++super.x===5);
check((super.x ||= (trace+='R',9))===5);check((super.x &&= (trace+='R',0))===0);check((super.x ??= (trace+='R',8))===0);
for(super.x of [6,7]){trace+='B';}for(super.x in {z:1}){trace+='I';}return stored;
}};check(home.run()==='z');return trace;
})();
"####,
        r####"result
"####,
        r####"GSGSGSGGRSGSBSBSI"####,
        r####"GSGSGSGGRSGSBSBSI"####,
    );
}

#[test]
fn object_super_property_call_new_and_delete_routes() {
    observe(
        r####"call_new_and_delete_routes"####,
        r####"function check(v){if(!v)throw new Error('object super oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',key={};key[Symbol.toPrimitive]=function(){trace+='T';return 'x';};
function Made(v){this.value=v;}
var base={get f(){trace+='G';return function(v){'use strict';check(this.tag==='R');trace+='F';return v;};},C:Made,x:1};
var home={__proto__:base,call(){return (super.f)((trace+='A',3));},make(){return new super.C(4);},del(){return delete super[(trace+='K',key)];}};
check(home.call.call({tag:'R'})===3);check(home.make().value===4);expect(ReferenceError,function(){home.del();});
check(base.x===1);return trace;
})();
"####,
        r####"result
"####,
        r####"GAFK"####,
        r####"GAFK"####,
    );
}

#[test]
fn object_super_property_definition_abrupt_and_accessor_replacement_order() {
    observe(
        r####"definition_abrupt_and_accessor_replacement_order"####,
        r####"function check(v){if(!v)throw new Error('object super oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',token={},key={};key[Symbol.toPrimitive]=function(){trace+='K';throw token;};var published='old';
try{published={m(){return super.x;},[key](){return super.x;},[(trace+='L','later')]:1};}catch(e){check(e===token);trace+='C';}
check(published==='old');
var home={__proto__:{x:4},get x(){return super.x;},set x(v){trace+='S';},x:9,get x(){trace+='G';return super.x;}};
var d=Object.getOwnPropertyDescriptor(home,'x');check(d.set===undefined&&home.x===4);return trace;
})();
"####,
        r####"result
"####,
        r####"KCG"####,
        r####"KCG"####,
    );
}

#[test]
fn object_super_property_super_call_method_forbidden() {
    early_error(
        r####"super_call_method_forbidden"####,
        r####"var touched=0;
"####,
        r####"touched++;({m(){super();}});
"####,
        r####"touched
"####,
    );
}

#[test]
fn object_super_property_super_call_default_forbidden() {
    early_error(
        r####"super_call_default_forbidden"####,
        r####"var touched=0;
"####,
        r####"touched++;({m(v=super()){return v;}});
"####,
        r####"touched
"####,
    );
}

#[test]
fn object_super_property_super_call_accessor_arrow_forbidden() {
    early_error(
        r####"super_call_accessor_arrow_forbidden"####,
        r####"var touched=0;
"####,
        r####"touched++;({get x(){return ()=>super();}});
"####,
        r####"touched
"####,
    );
}

#[test]
fn object_super_property_ordinary_function_value_resets_super() {
    early_error(
        r####"ordinary_function_value_resets_super"####,
        r####"var touched=0;
"####,
        r####"touched++;({m(){return {f:function(){return super.x;}};}});
"####,
        r####"touched
"####,
    );
}

#[test]
fn object_super_property_ordinary_nested_function_resets_super() {
    early_error(
        r####"ordinary_nested_function_resets_super"####,
        r####"var touched=0;
"####,
        r####"touched++;({m(){function f(){return ()=>super.x;}return f;}});
"####,
        r####"touched
"####,
    );
}

#[test]
fn object_super_property_computed_definition_name_needs_outer_context() {
    early_error(
        r####"computed_definition_name_needs_outer_context"####,
        r####"var touched=0;
"####,
        r####"touched++;({[super.x](){return 1;}});
"####,
        r####"touched
"####,
    );
}

#[test]
fn object_super_property_object_context_does_not_leak_to_next_function() {
    early_error(
        r####"object_context_does_not_leak_to_next_function"####,
        r####"var touched=0;
"####,
        r####"touched++;({m(){return super.x;}});function later(){return super.x;}
"####,
        r####"touched
"####,
    );
}

#[test]
fn object_super_property_duplicate_proto_colons_still_early_error() {
    early_error(
        r####"duplicate_proto_colons_still_early_error"####,
        r####"var touched=0;
"####,
        r####"touched++;({m(){return super.x;},__proto__:null,'__proto__':null});
"####,
        r####"touched
"####,
    );
}

#[test]
fn object_super_property_deferred_async_object_method() {
    deferred(
        r####"deferred_async_object_method"####,
        r####"({async m(){return super.x;}});
"####,
    );
}

#[test]
fn object_super_property_deferred_generator_object_method() {
    deferred(
        r####"deferred_generator_object_method"####,
        r####"({*m(){yield super.x;}});
"####,
    );
}

#[test]
fn object_super_property_deferred_object_spread() {
    deferred(
        r####"deferred_object_spread"####,
        r####"var base={};({m(){return super.x;},...base});
"####,
    );
}

#[test]
fn object_super_property_deferred_optional_super_result() {
    deferred(
        r####"deferred_optional_super_result"####,
        r####"({m(){return super.x?.y;}});
"####,
    );
}
