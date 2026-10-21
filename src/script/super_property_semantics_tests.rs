//! Independent public SuperProperty literals, frozen before implementation.
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
fn super_property_instance_home_and_borrowed_receiver() {
    observe(
        r####"instance_home_and_borrowed_receiver"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
class A { get value(){return this.tag;} method(){return this;} }
class C extends A { read(){return super.value;} call(){return super.method();} }
var receiver={tag:'receiver',value:'shadow',method:function(){throw 'shadow';}};
var inherited={value:'wrong',method:function(){throw 'receiver prototype';}};
Object.setPrototypeOf(receiver,inherited);
check(C.prototype.read.call(receiver)==='receiver');
check(C.prototype.call.apply(receiver,[])===receiver);
check(C.prototype.call.bind(receiver)()===receiver);
var replacement={get value(){return 'live:'+this.tag;},method:function(){return this;}};
Object.setPrototypeOf(C.prototype,replacement);
check(C.prototype.read.call(receiver)==='live:receiver');
return 'home:receiver:live';
})();
"####,
        r####"result
"####,
        r####"home:receiver:live"####,
        r####"home:receiver:live"####,
    );
}

#[test]
fn super_property_static_home_is_original_constructor() {
    observe(
        r####"static_home_is_original_constructor"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
class A { static get value(){return this.tag;} static method(){return this;} }
class C extends A { static read(){return super.value;} static call(){return super.method();} }
var receiver={tag:'R',value:'shadow'},saved=C.read;
check(saved.call(receiver)==='R'&&C.call.call(receiver)===receiver);
var replacement={get value(){return 'N'+this.tag;}};
Object.setPrototypeOf(C,replacement);
check(saved.call(receiver)==='NR');
Object.setPrototypeOf(receiver,{value:'wrong'});
check(saved.call(receiver)==='NR');
return 'static:live';
})();
"####,
        r####"result
"####,
        r####"static:live"####,
        r####"static:live"####,
    );
}

#[test]
fn super_property_accessor_receiver_and_setter_result() {
    observe(
        r####"accessor_receiver_and_setter_result"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',seen;
class A { get x(){trace+='G';seen=this;return this.mark;} set x(v){trace+='S';seen=this;this.received=v;return 900;} }
class C extends A { get x(){return super.x;} set x(v){super.x=v;} write(v){return super.x=v;} }
var receiver={mark:7};Object.defineProperty(receiver,'x',{value:40,writable:false});
var d=Object.getOwnPropertyDescriptor(C.prototype,'x');
check(d.get.call(receiver)===7&&seen===receiver);
d.set.call(receiver,8);check(receiver.received===8&&seen===receiver);
check(C.prototype.write.call(receiver,9)===9&&receiver.received===9&&receiver.x===40);
return trace;
})();
"####,
        r####"result
"####,
        r####"GSS"####,
        r####"GSS"####,
    );
}

#[test]
fn super_property_repeated_evaluation_and_saved_homes() {
    observe(
        r####"repeated_evaluation_and_saved_homes"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
function make(parent){class C { read(){return super.x;} }Object.setPrototypeOf(C.prototype,parent);return C;}
var A=make({x:'A'}),B=make({x:'B'}),a=A.prototype.read,b=B.prototype.read;
check(a.call(B.prototype)==='A'&&b.call(A.prototype)==='B');
A.prototype.read=function(){return 'replacement';};delete B.prototype.read;
check(a.call({})==='A'&&b.call({})==='B');
Object.setPrototypeOf(A.prototype,{x:'C'});
check(a.call({})==='C'&&b.call({})==='B');
return 'distinct:saved';
})();
"####,
        r####"result
"####,
        r####"distinct:saved"####,
        r####"distinct:saved"####,
    );
}

#[test]
fn super_property_escaped_method_and_accessor_arrows() {
    observe(
        r####"escaped_method_and_accessor_arrows"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
class A { get x(){return this.tag;} }
class C extends A { arrow(){return ()=>super.x;} get arrowGetter(){return ()=>super.x;} }
var receiver={tag:'R'},a=C.prototype.arrow.call(receiver);
var b=Object.getOwnPropertyDescriptor(C.prototype,'arrowGetter').get.call(receiver);
check(a.call({tag:'wrong'})==='R'&&b()==='R');
Object.setPrototypeOf(C.prototype,{get x(){return 'N'+this.tag;}});
check(a()==='NR'&&b.call(null)==='NR');
return 'arrow:receiver:live';
})();
"####,
        r####"result
"####,
        r####"arrow:receiver:live"####,
        r####"arrow:receiver:live"####,
    );
}

#[test]
fn super_property_constructor_call_parent_and_property_home_are_distinct() {
    observe(
        r####"constructor_call_parent_and_property_home_are_distinct"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='';
class A { constructor(){trace+='A';this.tag='a';} }
class B { constructor(){trace+='B';this.tag='b';} }
class C extends A { constructor(){super();trace+=super.label;this.read=()=>super.label;} }
Object.setPrototypeOf(C,B);Object.setPrototypeOf(C.prototype,{get label(){return this.tag+'P';}});
var c=new C();check(c.read()==='bP'&&Object.getPrototypeOf(c)===C.prototype);
Object.setPrototypeOf(C.prototype,{label:'Q'});check(c.read()==='Q');
return trace;
})();
"####,
        r####"result
"####,
        r####"BbP"####,
        r####"BbP"####,
    );
}

#[test]
fn super_property_constructor_method_and_arrow_parameter_defaults() {
    observe(
        r####"constructor_method_and_arrow_parameter_defaults"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='';
class Base { constructor(v=super.seed){this.v=v;trace+='B';} }
Object.setPrototypeOf(Base.prototype,{seed:3});check(new Base().v===3);
class A { constructor(){trace+='A';this.tag=4;} get seed(){trace+='G';return this.tag;} }
class C extends A { constructor(v=(super(),super.seed)){trace+='C';this.v=v;} method(v=super.seed){return v;} arrow(){return (v=super.seed)=>v;} }
var c=new C();check(c.v===4&&c.method()===4&&c.arrow()()===4);
return trace;
})();
"####,
        r####"result
"####,
        r####"BAGCGG"####,
        r####"BAGCGG"####,
    );
}

#[test]
fn super_property_nested_class_outer_evaluation_and_inner_home() {
    observe(
        r####"nested_class_outer_evaluation_and_inner_home"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='';class Parent { value(){return 'inner';} }
class A { get parent(){trace+='H';return Parent;} get key(){trace+='K';return 'm';} }
class C extends A { make(){return class D extends super.parent { [super.key](){return super.value();} };} }
var D=new C().make();check(new D().m()==='inner');return trace;
})();
"####,
        r####"result
"####,
        r####"HK"####,
        r####"HK"####,
    );
}

#[test]
fn super_property_derived_this_check_precedes_key_expression() {
    observe(
        r####"derived_this_check_precedes_key_expression"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='';class A { get x(){trace+='G';return this;} }
class C extends A { constructor(){try{super[(trace+='K','x')];}catch(e){check(e instanceof ReferenceError);trace+='T';}super();check(super[(trace+='k','x')]===this);trace+='B';} }
new C();return trace;
})();
"####,
        r####"result
"####,
        r####"TkGB"####,
        r####"TkGB"####,
    );
}

#[test]
fn super_property_escaped_uninitialized_constructor_activation() {
    observe(
        r####"escaped_uninitialized_constructor_activation"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var initialize,read,trace='',alternate={};
class A { constructor(){trace+='A';this.tag='A';} }
class B { constructor(){trace+='B';this.tag='B';check(new.target===Target);} }
class C extends A { constructor(){initialize=()=>super();read=()=>super.x;return alternate;} }
function Target(){}Target.prototype={target:true};
check(Reflect.construct(C,[],Target)===alternate);
expect(ReferenceError,()=>read());
Object.setPrototypeOf(C,B);Object.setPrototypeOf(C.prototype,{get x(){return this.tag;}});
var instance=initialize();check(Object.getPrototypeOf(instance)===Target.prototype&&read()==='B');
expect(ReferenceError,()=>initialize());
return trace;
})();
"####,
        r####"result
"####,
        r####"BB"####,
        r####"BB"####,
    );
}

#[test]
fn super_property_key_expression_vs_key_coercion_base_capture() {
    observe(
        r####"key_expression_vs_key_coercion_base_capture"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='';class C { read(key){return super[key];} changed(){return super[(trace+='E',Object.setPrototypeOf(C.prototype,B),'x')];} }
var A={x:'A'},B={x:'B'},D={x:'D'};
Object.setPrototypeOf(C.prototype,A);check(C.prototype.changed.call({})==='B');
Object.setPrototypeOf(C.prototype,A);
var key={};key[Symbol.toPrimitive]=function(h){check(h==='string');trace+='C';Object.setPrototypeOf(C.prototype,D);return 'x';};
check(C.prototype.read.call({},key)==='A');check(C.prototype.read.call({},'x')==='D');
return trace;
})();
"####,
        r####"result
"####,
        r####"EC"####,
        r####"EC"####,
    );
}

#[test]
fn super_property_simple_assignment_delays_conversion_until_after_rhs() {
    observe(
        r####"simple_assignment_delays_conversion_until_after_rhs"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',receiver={},key={},A={},B={},D={},seen;
Object.defineProperty(B,'x',{set:function(v){trace+='S';seen=this;check(v===9);}});
Object.defineProperty(D,'x',{set:function(){throw 'retarget';}});
class C { write(){return super[(trace+='K',Object.setPrototypeOf(C.prototype,B),key)]=(trace+='R',Object.setPrototypeOf(C.prototype,D),9);} }
Object.setPrototypeOf(C.prototype,A);
key[Symbol.toPrimitive]=function(h){check(h==='string');trace+='C';Object.setPrototypeOf(C.prototype,A);return 'x';};
check(C.prototype.write.call(receiver)===9&&seen===receiver);return trace;
})();
"####,
        r####"result
"####,
        r####"KRCS"####,
        r####"KRCS"####,
    );
}

#[test]
fn super_property_abrupt_key_rhs_conversion_and_setter_identity() {
    observe(
        r####"abrupt_key_rhs_conversion_and_setter_identity"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',marker={},stage=0,key={};
function keyValue(){trace+='K';if(stage===0)throw marker;return key;}
function rhs(){trace+='R';if(stage===1)throw marker;return 1;}
key[Symbol.toPrimitive]=function(){trace+='C';if(stage===2)throw marker;return 'x';};
class A { set x(v){trace+='S';throw marker;} }
class C extends A { write(){super[keyValue()]=rhs();} }
var c=new C(),parts=[];for(stage=0;stage<4;stage++){trace='';var caught=false;try{c.write();}catch(e){caught=true;check(e===marker);}check(caught);parts.push(trace);}return parts.join('|');
})();
"####,
        r####"result
"####,
        r####"K|KR|KRC|KRCS"####,
        r####"K|KR|KRC|KRCS"####,
    );
}

#[test]
fn super_property_null_base_order_and_delete_reference_error() {
    observe(
        r####"null_base_order_and_delete_reference_error"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',key={};key[Symbol.toPrimitive]=function(){trace+='C';return 'x';};
class C { read(){return super[(trace+='K',key)];} write(){return super[(trace+='K',key)]=(trace+='R',1);} remove(){return delete super[(trace+='K',key)];} type(){return typeof super[(trace+='K',key)];} }
Object.setPrototypeOf(C.prototype,null);var c=new C(),parts=[];
expect(TypeError,()=>c.read());parts.push(trace);trace='';expect(TypeError,()=>c.write());parts.push(trace);trace='';expect(ReferenceError,()=>c.remove());parts.push(trace);trace='';expect(TypeError,()=>c.type());parts.push(trace);
return parts.join('|');
})();
"####,
        r####"result
"####,
        r####"K|KR|K|K"####,
        r####"K|KR|K|K"####,
    );
}

#[test]
fn super_property_compound_reference_converts_key_once_and_keeps_base() {
    observe(
        r####"compound_reference_converts_key_once_and_keeps_base"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',key={},receiver={},seen,old={valueOf:function(){trace+='V';return 4;}};
class A { get x(){trace+='G';return old;} set x(v){trace+='S';seen=this;check(v===7);} }
class C extends A { add(){return super[(trace+='K',key)]+=(trace+='R',Object.setPrototypeOf(C.prototype,{set x(v){throw 'retarget';}}),3);} }
key[Symbol.toPrimitive]=function(){trace+='C';return 'x';};
check(C.prototype.add.call(receiver)===7&&seen===receiver);return trace;
})();
"####,
        r####"result
"####,
        r####"KCGRVS"####,
        r####"KCGRVS"####,
    );
}

#[test]
fn super_property_logical_assignment_taken_and_skipped_paths() {
    observe(
        r####"logical_assignment_taken_and_skipped_paths"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',stored,key={};key[Symbol.toPrimitive]=function(){trace+='C';return 'x';};
class A { get x(){trace+='G';return stored;} set x(v){trace+='S';stored=v;} }
class C extends A { and(){return super[(trace+='K',key)]&&=(trace+='R',7);} or(){return super[(trace+='K',key)]||=(trace+='R',7);} nullish(){return super[(trace+='K',key)]??=(trace+='R',7);} }
var c=new C(),out=[];
function run(name,initial,want){trace='';stored=initial;check(c[name]()===want&&stored===want);out.push(trace);}
run('and',0,0);run('and',2,7);run('or',2,2);run('or',0,7);run('nullish',false,false);run('nullish',null,7);run('nullish',undefined,7);
return out.join('|');
})();
"####,
        r####"result
"####,
        r####"KCG|KCGRS|KCG|KCGRS|KCG|KCGRS|KCGRS"####,
        r####"KCG|KCGRS|KCG|KCGRS|KCG|KCGRS|KCGRS"####,
    );
}

#[test]
fn super_property_prefix_and_postfix_updates() {
    observe(
        r####"prefix_and_postfix_updates"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',stored=4,key={};key[Symbol.toPrimitive]=function(){trace+='C';return 'x';};
class A { get x(){trace+='G';return stored;} set x(v){trace+='S';stored=v;} }
class C extends A { pre(){return ++super[(trace+='K',key)];} post(){return super[(trace+='K',key)]++;} down(){return --super[(trace+='K',key)];} postDown(){return super[(trace+='K',key)]--;} }
var c=new C();check(c.pre()===5&&c.post()===5&&c.down()===5&&c.postDown()===5&&stored===4);return trace;
})();
"####,
        r####"result
"####,
        r####"KCGSKCGSKCGSKCGS"####,
        r####"KCGSKCGSKCGSKCGS"####,
    );
}

#[test]
fn super_property_call_reference_parentheses_and_comma() {
    observe(
        r####"call_reference_parentheses_and_comma"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
class A { method(){check(new.target===undefined);return this;} }
class C extends A { paren(){return (super.method)();} comma(){return (0,super.method)();} }
var receiver={};check(C.prototype.paren.call(receiver)===receiver);check(C.prototype.comma.call(receiver)===undefined);return 'reference:lost';
})();
"####,
        r####"result
"####,
        r####"reference:lost"####,
        r####"reference:lost"####,
    );
}

#[test]
fn super_property_callee_get_precedes_arguments_but_callable_check_follows() {
    observe(
        r####"callee_get_precedes_arguments_but_callable_check_follows"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',marker={};class A { get x(){trace+='G';return 1;} get y(){trace+='H';throw marker;} }
class C extends A { bad(){return super.x((trace+='A',2));} abrupt(){return super.y((trace+='X',2));} }
var c=new C();try{c.bad();}catch(e){check(e instanceof TypeError);trace+='T';}var caught=false;try{c.abrupt();}catch(e){check(e===marker);caught=true;}check(caught);return trace;
})();
"####,
        r####"result
"####,
        r####"GATH"####,
        r####"GATH"####,
    );
}

#[test]
fn super_property_call_spread_order_and_receiver() {
    observe(
        r####"call_spread_order_and_receiver"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',receiver={},iterator={},source={};var n=0;
iterator.next=function(){trace+='N';n++;return n===1?{value:2,done:false}:{done:true};};
source[Symbol.iterator]=function(){trace+='I';return iterator;};
class A { get f(){trace+='G';return function(a,b,c){trace+='F';check(this===receiver&&a===1&&b===2&&c===3);return 9;};} }
class C extends A { call(){return super.f((trace+='A',1),...source,(trace+='Z',3));} }
check(C.prototype.call.call(receiver)===9);return trace;
})();
"####,
        r####"result
"####,
        r####"GAINNZF"####,
        r####"GAINNZF"####,
    );
}

#[test]
fn super_property_spread_step_abrupt_preserves_identity_without_close() {
    observe(
        r####"spread_step_abrupt_preserves_identity_without_close"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',marker={},source={},iterator={next:function(){trace+='N';throw marker;},return:function(){trace+='R';return {};}};
source[Symbol.iterator]=function(){trace+='I';return iterator;};
class A { get f(){trace+='G';return function(){trace+='F';};} }
class C extends A { call(){return super.f(...source);} }
var caught=false;try{new C().call();}catch(e){caught=true;check(e===marker);}check(caught);return trace;
})();
"####,
        r####"result
"####,
        r####"GIN"####,
        r####"GIN"####,
    );
}

#[test]
fn super_property_construct_super_property_callee() {
    observe(
        r####"construct_super_property_callee"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',seen;function Target(v){trace+='C';check(new.target===Target);this.v=v;seen=this;}
class A { get ctor(){trace+='G';return Target;} get bad(){trace+='B';return ()=>0;} }
class C extends A { build(){return new super.ctor((trace+='A',6));} bad(){return new super.bad((trace+='X',7));} }
var c=new C(),made=c.build();check(made===seen&&made.v===6&&Object.getPrototypeOf(made)===Target.prototype);
expect(TypeError,()=>c.bad());return trace;
})();
"####,
        r####"result
"####,
        r####"GACBX"####,
        r####"GACBX"####,
    );
}

#[test]
fn super_property_receiver_descriptor_matrix() {
    observe(
        r####"receiver_descriptor_matrix"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var setterCalls=0,protoSetter=0;class C { write(k,v){return super[k]=v;} }
var base={};Object.defineProperty(base,'w',{value:1,writable:true});Object.defineProperty(base,'r',{value:2,writable:false});Object.defineProperty(base,'g',{get:function(){return 3;}});Object.defineProperty(base,'s',{set:function(v){setterCalls++;this.received=v;}});Object.setPrototypeOf(C.prototype,base);
var write=C.prototype.write,own={};Object.defineProperty(own,'w',{value:4,writable:true,enumerable:false,configurable:false});check(write.call(own,'w',5)===5);var d=Object.getOwnPropertyDescriptor(own,'w');check(d.value===5&&d.writable&&!d.enumerable&&!d.configurable&&base.w===1);
var readonly={};Object.defineProperty(readonly,'w',{value:6,writable:false});expect(TypeError,()=>write.call(readonly,'w',9));check(readonly.w===6);
var accessor={};Object.defineProperty(accessor,'w',{set:function(){setterCalls++;}});expect(TypeError,()=>write.call(accessor,'w',9));check(setterCalls===0);
var inherited=Object.create({set w(v){protoSetter++;}});write.call(inherited,'w',8);d=Object.getOwnPropertyDescriptor(inherited,'w');check(d.value===8&&d.writable&&d.enumerable&&d.configurable&&protoSetter===0);
var fixed={w:7};Object.preventExtensions(fixed);write.call(fixed,'w',8);check(fixed.w===8);expect(TypeError,()=>write.call(fixed,'missing',9));
expect(TypeError,()=>write.call({},'r',9));expect(TypeError,()=>write.call({},'g',9));
Object.defineProperty(readonly,'s',{value:0,writable:false});write.call(readonly,'s',11);check(setterCalls===1&&readonly.received===11&&readonly.s===0);
return 'own:inherited:fixed:accessor';
})();
"####,
        r####"result
"####,
        r####"own:inherited:fixed:accessor"####,
        r####"own:inherited:fixed:accessor"####,
    );
}

#[test]
fn super_property_primitive_receivers_getters_setters_and_data() {
    observe(
        r####"primitive_receivers_getters_setters_and_data"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var expected,seen,received;class A { get x(){seen=this;return this;} set x(v){seen=this;received=v;} }
class C extends A { read(){return super.x;} write(v){return super.x=v;} data(v){return super.d=v;} missing(){return super.absent;} }
Object.defineProperty(A.prototype,'d',{value:1,writable:true});var values=[undefined,null,7,'s',Symbol('r')];
for(var i=0;i<values.length;i++){expected=values[i];check(C.prototype.read.call(expected)===expected&&seen===expected);check(C.prototype.write.call(expected,9)===9&&seen===expected&&received===9);expect(TypeError,()=>C.prototype.data.call(expected,9));check(C.prototype.missing.call(expected)===undefined);}
return 'five:primitive:receivers';
})();
"####,
        r####"result
"####,
        r####"five:primitive:receivers"####,
        r####"five:primitive:receivers"####,
    );
}

#[test]
fn super_property_reentrant_getter_preserves_outer_reference() {
    observe(
        r####"reentrant_getter_preserves_outer_reference"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',outer={},inner={},seen;class C { add(){return super.x+=(trace+='R',2);} side(){return super.y;} }
var A={get x(){trace+='G';check(C.prototype.side.call(inner)===inner);Object.setPrototypeOf(C.prototype,B);return 3;},get y(){trace+='Y';return this;},set x(v){trace+='S';seen=this;check(v===5);}};
var B={set x(v){throw 'retarget';}};Object.setPrototypeOf(C.prototype,A);
check(C.prototype.add.call(outer)===5&&seen===outer);return trace;
})();
"####,
        r####"result
"####,
        r####"GYRS"####,
        r####"GYRS"####,
    );
}

#[test]
fn super_property_delete_evaluates_key_expression_not_coercion() {
    observe(
        r####"delete_evaluates_key_expression_not_coercion"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',key={};key[Symbol.toPrimitive]=function(){trace+='C';throw 'conversion';};
class A { get x(){trace+='G';return 1;} }
class C extends A { remove(){return delete (super[(trace+='K',key)]);} type(){return typeof super.x;} }
var c=new C();expect(ReferenceError,()=>c.remove());check(c.type()==='number'&&Object.getOwnPropertyDescriptor(A.prototype,'x')!==undefined);return trace;
})();
"####,
        r####"result
"####,
        r####"KG"####,
        r####"KG"####,
    );
}

#[test]
fn super_property_for_of_super_target_and_iterator_close() {
    observe(
        r####"for_of_super_target_and_iterator_close"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',marker={},key={},source={},index=0;
key[Symbol.toPrimitive]=function(){trace+='C';return 'x';};
var iterator={next:function(){trace+='N';index++;return {value:index,done:false};},return:function(){trace+='R';return {};}};source[Symbol.iterator]=function(){trace+='I';return iterator;};
class A { set x(v){trace+='S';if(v===2)throw marker;} }
class C extends A { run(){for(super[(trace+='K',key)] of source){trace+='B';}} }
var caught=false;try{new C().run();}catch(e){caught=true;check(e===marker);}check(caught);return trace;
})();
"####,
        r####"result
"####,
        r####"INKCSBNKCSR"####,
        r####"INKCSBNKCSR"####,
    );
}

#[test]
fn super_property_for_in_super_assignment_target() {
    observe(
        r####"for_in_super_assignment_target"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',values=[],key={};key[Symbol.toPrimitive]=function(){trace+='C';return 'x';};
class A { set x(v){trace+='S';values.push(v);} }
class C extends A { run(){for(super[(trace+='K',key)] in {a:1,b:2}){trace+='B';}} }
new C().run();check(values.join(',')==='a,b');return trace;
})();
"####,
        r####"result
"####,
        r####"KCSBKCSB"####,
        r####"KCSBKCSB"####,
    );
}

#[test]
fn super_property_array_receiver_indices_length_and_native_method() {
    observe(
        r####"array_receiver_indices_length_and_native_method"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
class C { write(k,v){return super[k]=v;} push(v){return super.push(v);} }
var base={};Object.defineProperty(base,'2',{value:0,writable:true});Object.defineProperty(base,'length',{value:0,writable:true});base.push=Array.prototype.push;Object.setPrototypeOf(C.prototype,base);
var a=[];C.prototype.write.call(a,'2',7);check(a.length===3&&a[2]===7);C.prototype.write.call(a,'length',1);check(a.length===1&&a[2]===undefined);check(C.prototype.push.call(a,8)===2&&a[1]===8);
Object.defineProperty(a,'length',{writable:false});expect(TypeError,()=>C.prototype.write.call(a,'2',9));check(a.length===2&&a[2]===undefined);return 'array:2:8';
})();
"####,
        r####"result
"####,
        r####"array:2:8"####,
        r####"array:2:8"####,
    );
}

#[test]
fn super_property_array_base_reads_and_distinct_receiver_writes() {
    observe(
        r####"array_base_reads_and_distinct_receiver_writes"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
class C { read(k){return super[k];} write(k,v){return super[k]=v;} }
var base=[3],receiver={};Object.setPrototypeOf(C.prototype,base);
check(C.prototype.read.call(receiver,'0')===3&&C.prototype.read.call(receiver,'length')===1);C.prototype.write.call(receiver,'0',8);C.prototype.write.call(receiver,'length',9);
check(receiver[0]===8&&receiver.length===9&&base[0]===3&&base.length===1);return 'array:base:receiver';
})();
"####,
        r####"result
"####,
        r####"array:base:receiver"####,
        r####"array:base:receiver"####,
    );
}

#[test]
fn super_property_typed_array_receiver_conversion_and_invalid_index() {
    observe(
        r####"typed_array_receiver_conversion_and_invalid_index"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',value={valueOf:function(){trace+='V';return 260;}};class C { write(k,v){return super[k]=v;} }
var base={};Object.defineProperty(base,'0',{value:0,writable:true});Object.defineProperty(base,'2',{value:0,writable:true});Object.setPrototypeOf(C.prototype,base);
var a=new Uint8Array(2);check(C.prototype.write.call(a,'0',value)===value&&a[0]===4);expect(TypeError,()=>C.prototype.write.call(a,'2',value));check(trace==='V'&&a.length===2&&a[2]===undefined);
Object.defineProperty(base,'1',{value:0,writable:false});expect(TypeError,()=>C.prototype.write.call(a,'1',value));check(a[1]===0);return trace;
})();
"####,
        r####"result
"####,
        r####"V"####,
        r####"V"####,
    );
}

#[test]
fn super_property_typed_array_base_same_and_distinct_receivers() {
    observe(
        r####"typed_array_base_same_and_distinct_receivers"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',value={valueOf:function(){trace+='V';return 9;}};class C { read(k){return super[k];} write(k,v){return super[k]=v;} }
var base=new Uint8Array([3]),receiver={};Object.setPrototypeOf(C.prototype,base);
check(C.prototype.read.call(receiver,'0')===3);check(C.prototype.write.call(receiver,'0',value)===value&&receiver[0]===value&&base[0]===3&&trace==='');
C.prototype.write.call(receiver,'1',value);C.prototype.write.call(receiver,'-0',value);check(!Object.prototype.hasOwnProperty.call(receiver,'1')&&!Object.prototype.hasOwnProperty.call(receiver,'-0')&&trace==='');
C.prototype.write.call(base,'1',value);check(trace==='V'&&base[0]===3&&base[1]===undefined);
var symbol=Symbol('extra');C.prototype.write.call(receiver,'01',7);C.prototype.write.call(receiver,symbol,8);check(receiver['01']===7&&receiver[symbol]===8);return trace;
})();
"####,
        r####"result
"####,
        r####"V"####,
        r####"V"####,
    );
}

#[test]
fn super_property_mapped_and_unmapped_arguments_receiver() {
    observe(
        r####"mapped_and_unmapped_arguments_receiver"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
class C { write(v){return super[0]=v;} }Object.setPrototypeOf(C.prototype,{0:0});
return (function(a){C.prototype.write.call(arguments,9);var first=a,saved=arguments[0];a=5;return first+':'+saved+':'+arguments[0];})(1);
})();
"####,
        r####"result
"####,
        r####"9:9:5"####,
        r####"1:9:9"####,
    );
}

#[test]
fn super_property_dom_character_data_brand_and_actual_receiver() {
    observe(
        r####"dom_character_data_brand_and_actual_receiver"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
class C { read(){return super.data;} write(v){return super.data=v;} append(v){return super.appendData(v);} length(){return super.length;} }
Object.setPrototypeOf(C.prototype,CharacterData.prototype);var text=new Text('ab');
check(C.prototype.read.call(text)==='ab');check(C.prototype.write.call(text,'x')==='x');check(C.prototype.append.call(text,'yz')===undefined);check(C.prototype.read.call(text)==='xyz'&&C.prototype.length.call(text)===3);
expect(TypeError,()=>C.prototype.read.call({}));expect(TypeError,()=>C.prototype.write.call({},'bad'));expect(TypeError,()=>C.prototype.append.call({},'bad'));return 'xyz:3';
})();
"####,
        r####"result
"####,
        r####"xyz:3"####,
        r####"xyz:3"####,
    );
}

#[test]
fn super_property_dom_receiver_own_expandos_and_setter_identity() {
    observe(
        r####"dom_receiver_own_expandos_and_setter_identity"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var seen;class C { write(k,v){return super[k]=v;} }
var symbol=Symbol('node'),base={};Object.defineProperty(base,'extra',{value:0,writable:true});Object.defineProperty(base,'sink',{set:function(v){seen=this;this.received=v;}});Object.setPrototypeOf(C.prototype,base);
var text=new Text('ab');C.prototype.write.call(text,'extra',7);C.prototype.write.call(text,symbol,8);C.prototype.write.call(text,'sink',9);
check(text.extra===7&&text[symbol]===8&&seen===text&&text.received===9&&text.data==='ab');var d=Object.getOwnPropertyDescriptor(text,'extra');check(d.value===7&&d.writable&&d.enumerable&&d.configurable);return 'node:identity';
})();
"####,
        r####"result
"####,
        r####"node:identity"####,
        r####"node:identity"####,
    );
}

#[test]
fn super_property_promise_native_receiver_and_species() {
    observe(
        r####"promise_native_receiver_and_species"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
class C extends Promise { bridge(f){return super.then(f);} }
var trace='S',p=C.resolve(4),q=p.bridge(function(v){check(v===4);trace+='F';});check(q instanceof C);
expect(TypeError,()=>C.prototype.bridge.call({},function(){trace+='bad';}));trace+='X';trace+='E';
return {read:function(){return trace;}};
})();
"####,
        r####"result.read()
"####,
        r####"SXEF"####,
        r####"SXEF"####,
    );
}

#[test]
fn super_property_unicode_and_symbol_keys_exact_units() {
    observe(
        r####"unicode_and_symbol_keys_exact_units"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var lone='\uD800',symbol=Symbol(lone),other=Symbol(lone),trace='',key={};
class C { read(k){return super[k];} write(k,v){return super[k]=v;} }
var base={};base['雪']=1;base[lone]=2;base[symbol]=3;Object.setPrototypeOf(C.prototype,base);
check(C.prototype.read.call({},'雪')===1&&C.prototype.read.call({},lone)===2&&C.prototype.read.call({},symbol)===3&&C.prototype.read.call({},other)===undefined);
key[Symbol.toPrimitive]=function(h){check(h==='string');trace+='K';return symbol;};var receiver={};C.prototype.write.call(receiver,key,4);check(receiver[symbol]===4&&base[symbol]===3&&lone.charCodeAt(0)===55296);return trace;
})();
"####,
        r####"result
"####,
        r####"K"####,
        r####"K"####,
    );
}

#[test]
fn super_property_alternate_new_target_does_not_replace_constructor_home() {
    observe(
        r####"alternate_new_target_does_not_replace_constructor_home"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
class A { get x(){return this.marker;} }
class C extends A { constructor(){super();this.marker=6;this.seen=super.x;} }
function Target(){}Target.prototype={get x(){throw 'wrong prototype';}};
var made=Reflect.construct(C,[],Target);check(Object.getPrototypeOf(made)===Target.prototype&&made.seen===6);return 'newtarget:receiver:home';
})();
"####,
        r####"result
"####,
        r####"newtarget:receiver:home"####,
        r####"newtarget:receiver:home"####,
    );
}

#[test]
fn super_property_accessor_pair_preserves_independent_homes() {
    observe(
        r####"accessor_pair_preserves_independent_homes"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var seen;class A { get x(){return super.value;} }class B { set x(v){super.value=v;} }
Object.setPrototypeOf(A.prototype,{value:'A'});Object.setPrototypeOf(B.prototype,{set value(v){seen=this;this.received='B'+v;}});
var descriptor={get:Object.getOwnPropertyDescriptor(A.prototype,'x').get,set:Object.getOwnPropertyDescriptor(B.prototype,'x').set,configurable:true};
var receiver={};Object.defineProperty(receiver,'x',descriptor);check(receiver.x==='A');receiver.x=7;check(seen===receiver&&receiver.received==='B7');
Object.setPrototypeOf(A.prototype,{value:'C'});check(receiver.x==='C');receiver.x=8;check(receiver.received==='B8');return 'get:A:C|set:B';
})();
"####,
        r####"result
"####,
        r####"get:A:C|set:B"####,
        r####"get:A:C|set:B"####,
    );
}

#[test]
fn super_property_base_class_default_prototype_links() {
    observe(
        r####"base_class_default_prototype_links"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
class C { objectString(){return super.toString();} static invoke(receiver){return super.call(receiver);} }
var c=new C();check(c.objectString()==='[object Object]');expect(TypeError,()=>C.invoke({}));
var seen;function fn(){seen=this;return 8;}var receiver={};check(C.invoke.call(fn,receiver)===8&&seen===receiver);return 'Object:Function';
})();
"####,
        r####"result
"####,
        r####"Object:Function"####,
        r####"Object:Function"####,
    );
}

#[test]
fn super_property_early_top_level_super_property() {
    early_error(
        r####"early_top_level_super_property"####,
        r####"var sentinel=0;
"####,
        r####"sentinel=1;
super.x;
"####,
        r####"sentinel
"####,
    );
}

#[test]
fn super_property_early_ordinary_function_super_property() {
    early_error(
        r####"early_ordinary_function_super_property"####,
        r####"var sentinel=0;
"####,
        r####"sentinel=1;
function f(){return super.x;}
"####,
        r####"sentinel
"####,
    );
}

#[test]
fn super_property_early_nested_function_resets_super() {
    early_error(
        r####"early_nested_function_resets_super"####,
        r####"var sentinel=0;
"####,
        r####"sentinel=1;
class C { m(){return function(){return super.x;};} }
"####,
        r####"sentinel
"####,
    );
}

#[test]
fn super_property_early_base_constructor_super_call() {
    early_error(
        r####"early_base_constructor_super_call"####,
        r####"var sentinel=0;
"####,
        r####"sentinel=1;
class C { constructor(){super();} }
"####,
        r####"sentinel
"####,
    );
}

#[test]
fn super_property_early_method_super_call() {
    early_error(
        r####"early_method_super_call"####,
        r####"var sentinel=0;
"####,
        r####"sentinel=1;
class C extends Object { m(){super();} }
"####,
        r####"sentinel
"####,
    );
}

#[test]
fn super_property_early_bare_super() {
    early_error(
        r####"early_bare_super"####,
        r####"var sentinel=0;
"####,
        r####"sentinel=1;
class C { m(){return super;} }
"####,
        r####"sentinel
"####,
    );
}

#[test]
fn super_property_early_new_bare_super_call() {
    early_error(
        r####"early_new_bare_super_call"####,
        r####"var sentinel=0;
"####,
        r####"sentinel=1;
class C extends Object { constructor(){return new super();} }
"####,
        r####"sentinel
"####,
    );
}

#[test]
fn super_property_early_optional_super_base() {
    early_error(
        r####"early_optional_super_base"####,
        r####"var sentinel=0;
"####,
        r####"sentinel=1;
class C { m(){return super?.x;} }
"####,
        r####"sentinel
"####,
    );
}

#[test]
fn super_property_accepted_object_literal_method_super() {
    observe(
        r####"accepted_object_literal_method_super"####,
        r####"var object={m(){return super.x;}};
"####,
        r####"[typeof object.m,object.m()===undefined].join(':')"####,
        "function:true",
        "function:true",
    );
}

#[test]
fn super_property_accepted_object_literal_accessor_arrow_super() {
    observe(
        r####"accepted_object_literal_accessor_arrow_super"####,
        r####"var object={get x(){return ()=>super.x;}};
"####,
        r####"var arrow=object.x;[typeof arrow,arrow()===undefined].join(':')"####,
        "function:true",
        "function:true",
    );
}

#[test]
fn super_property_deferred_class_field_super() {
    deferred(
        r####"deferred_class_field_super"####,
        r####"class C { x=super.x; }
"####,
    );
}

#[test]
fn super_property_deferred_class_static_block_super() {
    deferred(
        r####"deferred_class_static_block_super"####,
        r####"class C { static { super.x; } }
"####,
    );
}

#[test]
fn super_property_deferred_class_async_super() {
    deferred(
        r####"deferred_class_async_super"####,
        r####"class C { async m(){return super.x;} }
"####,
    );
}

#[test]
fn super_property_static_accessor_and_arrow_default_home() {
    observe(
        r####"static_accessor_and_arrow_default_home"####,
        r####"function check(v){if(!v)throw new Error('super property oracle');}
function expect(C,f){var caught=false;try{f();}catch(e){caught=true;check(e instanceof C);}check(caught);}
var result=(function(){
var trace='',receiver={tag:'R'};
class A { static get x(){trace+='G';return this.tag;} static set x(v){trace+='S';this.tag=v;} }
class C extends A { static get x(){trace+='g';return super.x;} static set x(v){trace+='s';super.x=v;} static arrow(){return (v=super.x)=>super.x=v;} }
var d=Object.getOwnPropertyDescriptor(C,'x');check(d.get.call(receiver)==='R');d.set.call(receiver,'V');check(receiver.tag==='V');
var arrow=C.arrow.call(receiver);Object.setPrototypeOf(C,{get x(){trace+='H';return this.tag+'!';},set x(v){trace+='T';this.tag=v;}});
check(arrow.call({tag:'wrong'})==='V!'&&receiver.tag==='V!');return trace;
})();
"####,
        r####"result
"####,
        r####"gGsSHT"####,
        r####"gGsSHT"####,
    );
}
