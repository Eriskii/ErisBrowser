//! Public class contracts frozen independently before production.
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

fn observe(case_id: &str, source: &str, expression: &str, expected: Value) {
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        execute_mode(&mut runtime, &mut document, strict, source)
            .unwrap_or_else(|error| panic!("{case_id} strict={strict} source: {error:?}"));
        let observed = execute_mode(&mut runtime, &mut document, strict, expression)
            .unwrap_or_else(|error| panic!("{case_id} strict={strict} observation: {error:?}"));
        assert_eq!(&observed, &expected, "{case_id} strict={strict}");
    }
}

fn early_error(case_id: &str, setup: &str, source: &str, expression: &str, expected: Value) {
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        execute_mode(&mut runtime, &mut document, strict, setup)
            .unwrap_or_else(|error| panic!("{case_id} strict={strict} setup: {error:?}"));
        let error = execute_mode(&mut runtime, &mut document, strict, source)
            .expect_err("class early-error source unexpectedly completed");
        assert_eq!(
            error.intrinsic_error_name(),
            Some("SyntaxError"),
            "{case_id} strict={strict}: {error:?}"
        );
        assert!(
            error.is_parse_error(),
            "{case_id} strict={strict}: {error:?}"
        );
        let observed = execute_mode(&mut runtime, &mut document, strict, expression)
            .unwrap_or_else(|error| panic!("{case_id} strict={strict} observation: {error:?}"));
        assert_eq!(&observed, &expected, "{case_id} strict={strict}");
    }
}

fn unsupported(case_id: &str, source: &str) {
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        let error = execute_mode(&mut runtime, &mut document, strict, source)
            .expect_err("deferred class source unexpectedly completed");
        assert!(
            error.is_unsupported(),
            "{case_id} strict={strict}: {error:?}"
        );
    }
}

#[test]
fn class_public_lexical_names_and_tdz() {
    observe(
        "lexical_names_and_tdz",
        r###"function check(v){if(!v)throw new Error('class oracle');}
var result=(function(){
  var t='';
  try { typeof Later; class Later {} } catch(e){check(e instanceof ReferenceError);t+='T';}
  try { class Self extends Self {} } catch(e){check(e instanceof ReferenceError);t+='H';}
  try { class Key { [Key](){} } } catch(e){check(e instanceof ReferenceError);t+='K';}
  class Outer { static identity(){return Outer;} static write(){Outer=7;} }
  var saved=Outer;Outer=9;check(saved.identity()===saved&&Outer===9);
  try { saved.write(); } catch(e){check(e instanceof TypeError);t+='I';}
  var Expr=class Inner {static identity(){return Inner;}};
  check(Expr.identity()===Expr&&typeof Inner==='undefined');
  var Inferred=class {};var Shadow=class {static name(){return 3;} static length(){return 4;}};
  check(Inferred.name==='Inferred'&&Shadow.name()===3&&Shadow.length()===4);
  return t;
})();
"###,
        r###"result
"###,
        Value::String("THKI".into()),
    );
}

#[test]
fn class_public_global_and_block_lexical() {
    observe(
        "global_and_block_lexical",
        r###"function check(v){if(!v)throw new Error('class oracle');}
class ClassScopeWitness {}
var result=(function(){
  check(window.ClassScopeWitness===undefined);
  var first=ClassScopeWitness,read;
  { class ClassScopeWitness {static value(){return 4;}} read=()=>ClassScopeWitness; }
  check(read()!==first&&read().value()===4&&ClassScopeWitness===first);
  ClassScopeWitness=7;check(ClassScopeWitness===7);return 'lexical';
})();
"###,
        r###"result
"###,
        Value::String("lexical".into()),
    );
}

#[test]
fn class_public_constructor_and_method_metadata() {
    observe(
        "constructor_and_method_metadata",
        r###"function check(v){if(!v)throw new Error('class oracle');}
var result=(function(){
  var O=Object,H=O.prototype.hasOwnProperty;
  class Base {constructor(a,b=2,...rest){} m(a,b){} get x(){return 1;} set x(v){} static s(a){} }
  class Default extends Base {}
  var p=O.getOwnPropertyDescriptor(Base,'prototype'),n=O.getOwnPropertyDescriptor(Base,'name'),l=O.getOwnPropertyDescriptor(Base,'length');
  var c=O.getOwnPropertyDescriptor(Base.prototype,'constructor'),m=O.getOwnPropertyDescriptor(Base.prototype,'m'),x=O.getOwnPropertyDescriptor(Base.prototype,'x');
  check(p.value===Base.prototype&&!p.writable&&!p.enumerable&&!p.configurable);
  check(n.value==='Base'&&!n.writable&&!n.enumerable&&n.configurable);
  check(l.value===1&&!l.writable&&!l.enumerable&&l.configurable&&Default.length===0);
  check(c.value===Base&&c.writable&&!c.enumerable&&c.configurable);
  check(m.writable&&!m.enumerable&&m.configurable&&m.value.length===2&&!H.call(m.value,'prototype'));
  check(!x.enumerable&&x.configurable&&x.get.name==='get x'&&x.set.name==='set x');
  check(O.getPrototypeOf(Base)===Function.prototype&&O.getPrototypeOf(Base.prototype)===O.prototype);
  check(O.getPrototypeOf(Default)===Base&&O.getPrototypeOf(Default.prototype)===Base.prototype);
  var errors=0;try{new m.value();}catch(e){check(e instanceof TypeError);errors++;}
  try{new x.get();}catch(e){check(e instanceof TypeError);errors++;}
  check(errors===2);return 'metadata:2';
})();
"###,
        r###"result
"###,
        Value::String("metadata:2".into()),
    );
}

#[test]
fn class_public_public_keys_accessors_and_name_disambiguation() {
    observe(
        "public_keys_accessors_and_name_disambiguation",
        r###"function check(v){if(!v)throw new Error('class oracle');}
var result=(function(){
 var sym=Symbol('tag'),writes=0;
 class C { ; static(){return 's';} get(){return 'g';} set(){return 't';}
   ['constructor'](){return 'ordinary';} [sym](){return this;} 'x-y'(){return 2;} 7(){return 7;}
   get pair(){return this.v;} set pair(v){writes++;this.v=v;}
   m(){return 1;} m(){return 2;} static constructor(){return 3;}
 }
 var x=new C();x.pair=9;var d=Object.getOwnPropertyDescriptor(C.prototype,'pair');
 check(x.static()==='s'&&x.get()==='g'&&x.set()==='t'&&x.constructor()==='ordinary');
 check(x[sym]()===x&&C.prototype[sym].name==='[tag]'&&x['x-y']()===2&&x[7]()===7);
 check(x.pair===9&&writes===1&&typeof d.get==='function'&&typeof d.set==='function');
 check(x.m()===2&&C.constructor()===3&&Object.keys(C.prototype).length===0);
 return 'members';
})();
"###,
        r###"result
"###,
        Value::String("members".into()),
    );
}

#[test]
fn class_public_computed_key_order_and_abrupt_identity() {
    observe(
        "computed_key_order_and_abrupt_identity",
        r###"function check(v){if(!v)throw new Error('class oracle');}
var result=(function(){
 var trace='',token={},key={toString:function(){trace+='C';return 'm';}};
 function mark(s,k){trace+=s;return k;}
 class C { [mark('A',key)](){} static [mark('B','s')](){} [mark('D','n')](){} }
 check(trace==='ACBD');
 try { class Fail {[(trace+='E',key)](){} [(trace+='F',function(){throw token;}())](){} [(trace+='X','late')](){} } }
 catch(e){check(e===token);trace+='T';}
 check(typeof C.prototype.m==='function'&&typeof C.s==='function');return trace;
})();
"###,
        r###"result
"###,
        Value::String("ACBDECFT".into()),
    );
}

#[test]
fn class_public_heritage_validation_order() {
    observe(
        "heritage_validation_order",
        r###"function check(v){if(!v)throw new Error('class oracle');}
var result=(function(){
 var trace='',token={};function B(v){this.v=v;}
 var Bound=B.bind(null);Object.defineProperty(Bound,'prototype',{get:function(){trace+='P';return B.prototype;}});
 function heritage(){trace+='H';return Bound;}
 class C extends heritage(){[(trace+='K','m')](){} }
 var x=new C(7);check(x.v===7&&Object.getPrototypeOf(C)===Bound&&Object.getPrototypeOf(C.prototype)===B.prototype);
 try{class Bad extends (trace+='N',{}) {[(trace+='X','x')](){}}}catch(e){check(e instanceof TypeError);trace+='T';}
 var Throwing=B.bind(null);Object.defineProperty(Throwing,'prototype',{get:function(){trace+='G';throw token;}});
 try{class Bad extends Throwing {[(trace+='Y','y')](){}}}catch(e){check(e===token);trace+='I';}
 B.prototype=3;try{class Bad extends B {[(trace+='Z','z')](){}}}catch(e){check(e instanceof TypeError);trace+='V';}
 return trace;
})();
"###,
        r###"result
"###,
        Value::String("HPKNTGIV".into()),
    );
}

#[test]
fn class_public_intrinsic_parents_and_setter_bypass() {
    observe(
        "intrinsic_parents_and_setter_bypass",
        r###"function check(v){if(!v)throw new Error('class oracle');}
var result=(function(){
 var O=Object,F=Function,hits=0;
 O.defineProperty(O.prototype,'classMember',{set:function(){hits++;},configurable:true});
 O.defineProperty(F.prototype,'classStatic',{set:function(){hits++;},configurable:true});
 Object=function(){throw 1;};Function=function(){throw 2;};
 var C=class {classMember(){return 1;} static classStatic(){return 2;}};
 check(O.getPrototypeOf(C)===F.prototype&&O.getPrototypeOf(C.prototype)===O.prototype);
 check(new C().classMember()===1&&C.classStatic()===2&&hits===0);
 Object=O;Function=F;delete O.prototype.classMember;delete F.prototype.classStatic;return 'intrinsics:0';
})();
"###,
        r###"result
"###,
        Value::String("intrinsics:0".into()),
    );
}

#[test]
fn class_public_class_call_guard_precedes_parameters() {
    observe(
        "class_call_guard_precedes_parameters",
        r###"function check(v){if(!v)throw new Error('class oracle');}
var result=(function(){
 var trace='';class C {constructor(x=(trace+='D',7)){trace+='B';this.x=x;}}
 function rejected(f){try{f();throw 'miss';}catch(e){check(e instanceof TypeError);trace+='T';}}
 rejected(()=>C((trace+='A',undefined)));rejected(()=>C.call(null));
 rejected(()=>C.apply(null,[]));rejected(()=>C.bind(null)());
 var x=new C();check(x.x===7&&typeof C==='function');return trace;
})();
"###,
        r###"result
"###,
        Value::String("ATTTTDB".into()),
    );
}

#[test]
fn class_public_class_strictness_and_detached_receiver() {
    observe(
        "class_strictness_and_detached_receiver",
        r###"function check(v){if(!v)throw new Error('class oracle');}
var result=(function(){
 class C {m(){return this;} nested(){return function(){return this;};} write(){ClassMissingWitness=1;}}
 var x=new C(),m=x.m,n=x.nested();check(m()===undefined&&n()===undefined);
 var strict=false;try{x.write();}catch(e){strict=e instanceof ReferenceError;}
 check(strict&&typeof ClassMissingWitness==='undefined');return 'strict';
})();
"###,
        r###"result
"###,
        Value::String("strict".into()),
    );
}

#[test]
fn class_public_base_return_and_alternate_newtarget() {
    observe(
        "base_return_and_alternate_newtarget",
        r###"function check(v){if(!v)throw new Error('class oracle');}
var result=(function(){
 var token={},seen;class B {constructor(v){seen=new.target;this.v=v;return v;}}
 var a=new B(3);check(a.v===3&&seen===B);check(new B(token)===token);
 function N(){}var proto={};N.prototype=proto;
 var b=Reflect.construct(B,[4],N);check(seen===N&&Object.getPrototypeOf(b)===proto&&b.v===4);
 var Bound=B.bind({bad:true},5),c=new Bound();check(seen===B&&Object.getPrototypeOf(c)===B.prototype&&c.v===5);
 return 'base-and-newtarget';
})();
"###,
        r###"result
"###,
        Value::String("base-and-newtarget".into()),
    );
}

#[test]
fn class_public_derived_return_and_this_states() {
    observe(
        "derived_return_and_this_states",
        r###"function check(v){if(!v)throw new Error('class oracle');}
var result=(function(){
 var trace='',token={};function B(){trace+='B';this.ok=true;}
 function error(C,T){try{new C();throw 'miss';}catch(e){check(e instanceof T);trace+=T===ReferenceError?'R':'T';}}
 error(class extends B {constructor(){this.x=1;}},ReferenceError);
 error(class extends B {constructor(a=this){}},ReferenceError);
 error(class extends B {constructor(){}},ReferenceError);
 error(class extends B {constructor(){return undefined;}},ReferenceError);
 error(class extends B {constructor(){return 1;}},TypeError);
 error(class extends B {constructor(){super();return null;}},TypeError);
 var C=class extends B {constructor(){return token;}};check(new C()===token);
 var D=class extends B {constructor(){super();return undefined;}};check(new D().ok);
 return trace;
})();
"###,
        r###"result
"###,
        Value::String("RRRRTBTB".into()),
    );
}

#[test]
fn class_public_repeated_super_and_retry_after_throw() {
    observe(
        "repeated_super_and_retry_after_throw",
        r###"function check(v){if(!v)throw new Error('class oracle');}
var result=(function(){
 var trace='',count=0,token={};function B(){count++;trace+='B';this.n=count;}
 class D extends B {constructor(){super();try{super();}catch(e){check(e instanceof ReferenceError);trace+='R';}check(this.n===1);}}
 new D();check(count===2);
 function ThrowOnce(){trace+='X';if(count++===2)throw token;this.ok=true;}
 class Retry extends ThrowOnce {constructor(){try{super();}catch(e){check(e===token);trace+='C';}super();check(this.ok);}}
 new Retry();return trace;
})();
"###,
        r###"result
"###,
        Value::String("BBRXCX".into()),
    );
}

#[test]
fn class_public_dynamic_super_capture_before_arguments() {
    observe(
        "dynamic_super_capture_before_arguments",
        r###"function check(v){if(!v)throw new Error('class oracle');}
var result=(function(){
 var trace='';function A(v){trace+='A';this.v=v;}function B(v){trace+='B';this.v=v;}
 class D extends A {constructor(){super((trace+='X',Object.setPrototypeOf(D,A),7));}}
 Object.setPrototypeOf(D,B);var a=new D(),b=new D();
 check(a.v===7&&b.v===7&&Object.getPrototypeOf(a)===D.prototype&&Object.getPrototypeOf(D.prototype)===A.prototype);
 Object.setPrototypeOf(D,{});try{new D();}catch(e){check(e instanceof TypeError);trace+='T';}
 return trace;
})();
"###,
        r###"result
"###,
        Value::String("XBXAXT".into()),
    );
}

#[test]
fn class_public_lexical_arrow_super_and_escaped_owner() {
    observe(
        "lexical_arrow_super_and_escaped_owner",
        r###"function check(v){if(!v)throw new Error('class oracle');}
var result=(function(){
 var trace='',saved,later,token={};function B(v){trace+='B'+v;this.v=v;}
 class D extends B {constructor(){var get=()=>this;try{get();}catch(e){check(e instanceof ReferenceError);trace+='R';}
   var call=()=>super(4);call();check(get()===this);saved=()=>[this,new.target];}}
 var d=new D(),pair=saved.call({});check(pair[0]===d&&pair[1]===D);
 class Late extends B {constructor(){later=()=>super(8);return token;}}
 check(new Late()===token);var x=later();check(x.v===8&&Object.getPrototypeOf(x)===Late.prototype);
 try{later();}catch(e){check(e instanceof ReferenceError);trace+='R';}return trace;
})();
"###,
        r###"result
"###,
        Value::String("RB4B8B8R".into()),
    );
}

#[test]
fn class_public_default_forwarding_without_array_iteration() {
    observe(
        "default_forwarding_without_array_iteration",
        r###"function check(v){if(!v)throw new Error('class oracle');}
var result=(function(){
 var token={},seen,old=Array.prototype[Symbol.iterator],hits=0;
 function B(a,b){seen=[a,b,arguments.length,new.target];}
 class D extends B {}var d;
 Array.prototype[Symbol.iterator]=function(){hits++;throw token;};
 try{d=new D(token,9);}finally{Array.prototype[Symbol.iterator]=old;}
 check(seen[0]===token&&seen[1]===9&&seen[2]===2&&seen[3]===D&&hits===0&&D.length===0);
 class E extends B {constructor(a=token,...rest){super(a,...rest);}}
 new E(undefined,6);check(seen[0]===token&&seen[1]===6&&seen[2]===2&&seen[3]===E);
 return 'forward:0';
})();
"###,
        r###"result
"###,
        Value::String("forward:0".into()),
    );
}

#[test]
fn class_public_extends_null_and_new_class_expression() {
    observe(
        "extends_null_and_new_class_expression",
        r###"function check(v){if(!v)throw new Error('class oracle');}
var result=(function(){
 class N extends null {}var caught=false;
 check(Object.getPrototypeOf(N)===Function.prototype&&Object.getPrototypeOf(N.prototype)===null);
 try{new N();}catch(e){caught=e instanceof TypeError;}check(caught);
 var x=new class extends null {constructor(v){var o=Object.create(new.target.prototype);o.v=v;return o;}}(7);
 check(x.v===7&&Object.getPrototypeOf(Object.getPrototypeOf(x))===null);return 'null:TypeError|object:7';
})();
"###,
        r###"result
"###,
        Value::String("null:TypeError|object:7".into()),
    );
}

#[test]
fn class_public_computed_static_prototype_stops_at_runtime() {
    observe(
        "computed_static_prototype_stops_at_runtime",
        r###"function check(v){if(!v)throw new Error('class oracle');}
var result=(function(){
 var trace='';try{class C {static [(trace+='A','ok')](){} static [(trace+='B','prototype')](){} [(trace+='X','late')](){}}}
 catch(e){check(e instanceof TypeError);trace+='T';}return trace;
})();
"###,
        r###"result
"###,
        Value::String("ABT".into()),
    );
}

#[test]
fn class_public_outer_this_and_newtarget_in_class_evaluation() {
    observe(
        "outer_this_and_newtarget_in_class_evaluation",
        r###"function check(v){if(!v)throw new Error('class oracle');}
var result=(function(){
 var marker={},seen=0;function B(){}
 function make(){var self=this,target=new.target;class C extends (check(this===self),check(new.target===target),seen++,B) {
   [(check(this===self),check(new.target===target),seen++,'m')](){return this;}}
   return C;}
 var C=make.call(marker),D=new make();check(seen===4&&new C().m() instanceof C&&new D().m() instanceof D);
 return 'outer-context:4';
})();
"###,
        r###"result
"###,
        Value::String("outer-context:4".into()),
    );
}

#[test]
fn class_public_promise_default_subclass_withresolvers_and_try() {
    observe(
        "promise_default_subclass_withresolvers_and_try",
        r###"function check(v){if(!v)throw new Error('class oracle');}
class P extends Promise {}
var c=P.withResolvers(),token={},trace='S';
check(c.promise instanceof P&&c.promise instanceof Promise&&Object.getPrototypeOf(c.promise)===P.prototype);
check(P.try(()=>c.promise)===c.promise);
c.promise.then(function(v){check(v===token);trace+='F';});
queueMicrotask(function(){trace+='Q';});c.resolve(token);trace+='E';
"###,
        r###"trace
"###,
        Value::String("SEQF".into()),
    );
}

#[test]
fn class_public_promise_explicit_super_executor_order() {
    observe(
        "promise_explicit_super_executor_order",
        r###"function check(v){if(!v)throw new Error('class oracle');}
var trace='',token={},seenTarget,done=false;
class P extends Promise {constructor(executor){trace+='C';seenTarget=new.target;super(executor);trace+='D';}}
var p=new P(function(resolve){trace+='E';resolve(token);}),first=trace;
check(seenTarget===P&&p instanceof P&&p instanceof Promise);
p.then(function(v){check(v===token);done=true;});
"###,
        r###"first+'|'+done
"###,
        Value::String("CED|true".into()),
    );
}

#[test]
fn class_public_promise_combinators_inherited_statics() {
    observe(
        "promise_combinators_inherited_statics",
        r###"function check(v){if(!v)throw new Error('class oracle');}
class P extends Promise {}
var a={},b={},seen=['0','0','0','0','0','0','0'];
var ps=[P.resolve(a),P.reject(b),P.all([a]),P.race([a]),P.allSettled([P.reject(b)]),P.any([P.reject(b),a]),P.any([])];
for(var i=0;i<ps.length;i++)check(ps[i] instanceof P&&ps[i] instanceof Promise);
ps[0].then(v=>{check(v===a);seen[0]='1';});
ps[1].then(undefined,v=>{check(v===b);seen[1]='1';});
ps[2].then(v=>{check(v.length===1&&v[0]===a);seen[2]='1';});
ps[3].then(v=>{check(v===a);seen[3]='1';});
ps[4].then(v=>{check(v.length===1&&v[0].status==='rejected'&&v[0].reason===b);seen[4]='1';});
ps[5].then(v=>{check(v===a);seen[5]='1';});
ps[6].then(undefined,v=>{check(v instanceof AggregateError&&v.errors.length===0);seen[6]='1';});
"###,
        r###"seen.join('')
"###,
        Value::String("1111111".into()),
    );
}

#[test]
fn class_public_promise_species_object_return_without_super() {
    observe(
        "promise_species_object_return_without_super",
        r###"function check(v){if(!v)throw new Error('class oracle');}
var trace='',holder={},token={},resolved,receiver;
class P extends Promise {}
class C extends P {constructor(executor){trace+='C';executor(function(v){'use strict';trace+='R';resolved=v;receiver=this;},function(){throw 'unexpected';});return holder;}}
var p=Promise.resolve(token);p.constructor={get [Symbol.species](){trace+='G';return C;}};
var out=p.then(function(v){check(v===token);trace+='H';return token;}),before=trace;
check(out===holder);
"###,
        r###"before+'|'+trace+'|'+(resolved===token)+'|'+(receiver===undefined)
"###,
        Value::String("GC|GCHR|true|true".into()),
    );
}

#[test]
fn class_public_class_is_callable_but_rejects_invocation() {
    observe(
        "class_is_callable_but_rejects_invocation",
        r###"function check(v){if(!v)throw new Error('class oracle');}
var entered=0,tryRejected=false,executorRejected=false;
class Callback {constructor(x=(entered++,1)){entered++;}}
Promise.try(Callback).then(undefined,e=>{check(e instanceof TypeError);tryRejected=true;});
new Promise(Callback).then(undefined,e=>{check(e instanceof TypeError);executorRejected=true;});
"###,
        r###"tryRejected+'|'+executorRejected+'|'+entered
"###,
        Value::String("true|true|0".into()),
    );
}

#[test]
fn class_public_aggregateerror_native_brand_through_default_super() {
    observe(
        "aggregateerror_native_brand_through_default_super",
        r###"function check(v){if(!v)throw new Error('class oracle');}
var result=(function(){
 class E extends AggregateError {}var token={},reason={},e=new E([token],'message',{cause:reason});
 check(e instanceof E&&e instanceof AggregateError&&e instanceof Error&&Object.getPrototypeOf(e)===E.prototype);
 check(e.errors.length===1&&e.errors[0]===token&&e.message==='message'&&e.cause===reason);
 check(Object.prototype.toString.call(e)==='[object Error]');return 'native-brand';
})();
"###,
        r###"result
"###,
        Value::String("native-brand".into()),
    );
}

#[test]
fn class_public_duplicate_constructor() {
    early_error(
        "duplicate_constructor",
        r###"var touched=0;
"###,
        r###"touched=1;
class C {constructor(){} constructor(){}}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_getter_constructor() {
    early_error(
        "getter_constructor",
        r###"var touched=0;
"###,
        r###"touched=1;
class C {get constructor(){}}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_setter_constructor() {
    early_error(
        "setter_constructor",
        r###"var touched=0;
"###,
        r###"touched=1;
class C {set constructor(v){}}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_literal_static_prototype() {
    early_error(
        "literal_static_prototype",
        r###"var touched=0;
"###,
        r###"touched=1;
class C {static prototype(){}}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_quoted_static_prototype() {
    early_error(
        "quoted_static_prototype",
        r###"var touched=0;
"###,
        r###"touched=1;
class C {static "prototype"(){}}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_base_super_call() {
    early_error(
        "base_super_call",
        r###"var touched=0;
"###,
        r###"touched=1;
class C {constructor(){super();}}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_method_super_call() {
    early_error(
        "method_super_call",
        r###"var touched=0;
"###,
        r###"touched=1;
class C extends Object {m(){super();}}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_static_super_call() {
    early_error(
        "static_super_call",
        r###"var touched=0;
"###,
        r###"touched=1;
class C extends Object {static m(){super();}}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_nested_function_super_call() {
    early_error(
        "nested_function_super_call",
        r###"var touched=0;
"###,
        r###"touched=1;
class C extends Object {constructor(){function f(){super();}}}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_duplicate_constructor_parameters() {
    early_error(
        "duplicate_constructor_parameters",
        r###"var touched=0;
"###,
        r###"touched=1;
class C {constructor(a,a){}}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_duplicate_method_parameters() {
    early_error(
        "duplicate_method_parameters",
        r###"var touched=0;
"###,
        r###"touched=1;
class C {m(a,a){}}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_getter_arity() {
    early_error(
        "getter_arity",
        r###"var touched=0;
"###,
        r###"touched=1;
class C {get x(a){}}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_setter_no_argument() {
    early_error(
        "setter_no_argument",
        r###"var touched=0;
"###,
        r###"touched=1;
class C {set x(){}}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_setter_two_arguments() {
    early_error(
        "setter_two_arguments",
        r###"var touched=0;
"###,
        r###"touched=1;
class C {set x(a,b){}}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_setter_rest_argument() {
    early_error(
        "setter_rest_argument",
        r###"var touched=0;
"###,
        r###"touched=1;
class C {set x(...a){}}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_strict_with() {
    early_error(
        "strict_with",
        r###"var touched=0;
"###,
        r###"touched=1;
class C {m(){with({}){}}}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_strict_delete_identifier() {
    early_error(
        "strict_delete_identifier",
        r###"var touched=0;
"###,
        r###"touched=1;
class C {m(){var x;delete x;}}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_strict_class_binding() {
    early_error(
        "strict_class_binding",
        r###"var touched=0;
"###,
        r###"touched=1;
class eval {}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_controlled_statement_class() {
    early_error(
        "controlled_statement_class",
        r###"var touched=0;
"###,
        r###"touched=1;
if(true) class C {}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_labelled_class() {
    early_error(
        "labelled_class",
        r###"var touched=0;
"###,
        r###"touched=1;
label: class C {}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_lexical_collision() {
    early_error(
        "lexical_collision",
        r###"var touched=0;
"###,
        r###"touched=1;
let C;class C {}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_var_collision() {
    early_error(
        "var_collision",
        r###"var touched=0;
"###,
        r###"touched=1;
var C;class C {}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_parameter_collision() {
    early_error(
        "parameter_collision",
        r###"var touched=0;
"###,
        r###"touched=1;
function f(C){class C {}}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_catch_collision() {
    early_error(
        "catch_collision",
        r###"var touched=0;
"###,
        r###"touched=1;
try{}catch(C){class C {}}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_switch_collision() {
    early_error(
        "switch_collision",
        r###"var touched=0;
"###,
        r###"touched=1;
switch(0){case 0:class C {}break;default:class C {}}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_anonymous_declaration() {
    early_error(
        "anonymous_declaration",
        r###"var touched=0;
"###,
        r###"touched=1;
class {}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_public_field() {
    unsupported(
        "public_field",
        r###"class C {x=1;}
"###,
    );
}

#[test]
fn class_public_static_field() {
    unsupported(
        "static_field",
        r###"class C {static x=1;}
"###,
    );
}

#[test]
fn class_public_private_field() {
    unsupported(
        "private_field",
        r###"class C {#x;}
"###,
    );
}

#[test]
fn class_public_private_method() {
    unsupported(
        "private_method",
        r###"class C {#m(){}}
"###,
    );
}

#[test]
fn class_public_static_block() {
    unsupported(
        "static_block",
        r###"class C {static {}}
"###,
    );
}

#[test]
fn class_public_generator_method() {
    unsupported(
        "generator_method",
        r###"class C {*m(){}}
"###,
    );
}

#[test]
fn class_public_async_method() {
    unsupported(
        "async_method",
        r###"class C {async m(){}}
"###,
    );
}

#[test]
fn class_public_async_generator_method() {
    unsupported(
        "async_generator_method",
        r###"class C {async *m(){}}
"###,
    );
}

#[test]
fn class_public_super_named_property() {
    observe(
        "super_named_property",
        r###"class C {m(){return super.x;}}
Object.setPrototypeOf(C.prototype, {x:17});
var result=(new C()).m();
"###,
        "result",
        Value::Number(17.0),
    );
}

#[test]
fn class_public_super_computed_assignment() {
    observe(
        "super_computed_assignment",
        r###"class C {m(k,v){super[k]=v;}}
var item=new C();item.m('x',23);var result=item.x;
"###,
        "result",
        Value::Number(23.0),
    );
}

#[test]
fn class_public_inherited_static_accessors_keep_receiver() {
    observe(
        "inherited_static_accessors_keep_receiver",
        r###"function check(v){if(!v)throw new Error('class oracle');}
var result=(function(){
 var sym=Symbol('tag'),token={},writes=0;
 class C {static get x(){return this.v;} static set x(v){writes++;this.v=v;}
   static get [sym](){return this;} static set [sym](v){this.saved=v;}}
 class D extends C {}
 C.x=7;D.x=8;D[sym]=token;
 var d=Object.getOwnPropertyDescriptor(C,'x'),s=Object.getOwnPropertyDescriptor(C,sym);
 check(C.x===7&&D.x===8&&writes===2&&D[sym]===D&&D.saved===token&&!Object.prototype.hasOwnProperty.call(C,'saved'));
 check(!d.enumerable&&d.configurable&&d.get.name==='get x'&&d.set.name==='set x');
 check(!s.enumerable&&s.configurable&&s.get.name==='get [tag]'&&s.set.name==='set [tag]');
 return 'static-receivers:2';
})();
"###,
        r###"result
"###,
        Value::String("static-receivers:2".into()),
    );
}

#[test]
fn class_public_computed_static_species_creates_selected_promise() {
    observe(
        "computed_static_species_creates_selected_promise",
        r###"function check(v){if(!v)throw new Error('class oracle');}
var trace='',token={},done=false;
class Q extends Promise {}
class P extends Promise {static get [Symbol.species](){trace+='G';return Q;}}
var p=new P(resolve=>resolve(token));
var q=p.then(v=>{check(v===token);trace+='R';done=true;return v;});
check(p instanceof P&&q instanceof Q&&q instanceof Promise&&!(q instanceof P));
"###,
        r###"trace+'|'+done
"###,
        Value::String("GR|true".into()),
    );
}

#[test]
fn class_public_naming_identifier_and_logical_assignment() {
    observe(
        "naming_identifier_and_logical_assignment",
        r###"function check(v){if(!v)throw new Error('class naming oracle');}
var result=(function(){
 var trace='',assigned,andName=1,orName=0,nullName=null;
 var first=(assigned=class {[(trace+='A','m')](){}});
 var second=(andName&&=class {[(trace+='B','m')](){}});
 var third=(orName||=class {[(trace+='C','m')](){}});
 var fourth=(nullName??=class {[(trace+='D','m')](){}});
 check(first===assigned&&second===andName&&third===orName&&fourth===nullName);
 check(assigned.name==='assigned'&&andName.name==='andName'&&orName.name==='orName'&&nullName.name==='nullName');
 var skipAnd=0,marker={},skipOr=marker,skipNull=false;
 check((skipAnd&&=class {[(trace+='X','m')](){}})===0);
 check((skipOr||=class {[(trace+='Y','m')](){}})===marker);
 check((skipNull??=class {[(trace+='Z','m')](){}})===false);
 check(skipAnd===0&&skipOr===marker&&skipNull===false);
 var hinted;hinted=class {static read(){return hinted;}};
 var saved=hinted;check(saved.name==='hinted');hinted=7;check(saved.read()===7);
 var explicit;explicit=class Inner {};check(explicit.name==='Inner');
 return trace+'|names';
})();
"###,
        r###"result
"###,
        Value::String("ABCD|names".into()),
    );
}

#[test]
fn class_public_naming_parameter_defaults() {
    observe(
        "naming_parameter_defaults",
        r###"function check(v){if(!v)throw new Error('class naming oracle');}
var result=(function(){
 function ordinary(Param=class {}){return Param;}
 var arrow=(Arrow=(class {}))=>Arrow;
 class Holder {
  constructor(Ctor=class {}){this.C=Ctor;}
  method(Method=class {}){return Method;}
  static take(Static=class {}){return Static;}
 }
 var p=ordinary(),a=arrow(),h=new Holder(),m=h.method(),s=Holder.take();
 check(p.name==='Param'&&a.name==='Arrow'&&h.C.name==='Ctor'&&m.name==='Method'&&s.name==='Static');
 var marker={},count=0;
 function optional(Skip=class {[(count++,'m')](){}}){return Skip;}
 check(optional(marker)===marker&&count===0);
 check(optional().name==='Skip'&&count===1);
 var d=Object.getOwnPropertyDescriptor(p,'name');
 check(d.value==='Param'&&!d.writable&&!d.enumerable&&d.configurable);
 return p.name+'|'+a.name+'|'+h.C.name+'|'+m.name+'|'+s.name+'|'+count;
})();
"###,
        r###"result
"###,
        Value::String("Param|Arrow|Ctor|Method|Static|1".into()),
    );
}

#[test]
fn class_public_naming_object_data_property_keys() {
    observe(
        "naming_object_data_property_keys",
        r###"function check(v){if(!v)throw new Error('class naming oracle');}
var result=(function(){
 var trace='',key={toString:function(){trace+='K';return 'computed';}},sym=Symbol('slot'),empty=Symbol();
 var o={
  plain:class {},
  'spaced key':(class {}),
  [key]:class {[(trace+='C','m')](){}},
  [(trace+='S',sym)]:class {[(trace+='D','m')](){}},
  [empty]:class {}
 };
 check(o.plain.name==='plain'&&o['spaced key'].name==='spaced key'&&o.computed.name==='computed');
 check(o[sym].name==='[slot]'&&o[empty].name==='');
 var d=Object.getOwnPropertyDescriptor(o[sym],'name');
 check(d.value==='[slot]'&&!d.writable&&!d.enumerable&&d.configurable);
 check(Object.getOwnPropertyDescriptor(o,sym).value===o[sym]);
 return trace+'|'+o.plain.name+'|'+o['spaced key'].name+'|'+o.computed.name+'|'+o[sym].name+'|'+o[empty].name;
})();
"###,
        r###"result
"###,
        Value::String("KCSD|plain|spaced key|computed|[slot]|".into()),
    );
}

#[test]
fn class_public_naming_parentheses_comma_conditional_boundaries() {
    observe(
        "naming_parentheses_comma_conditional_boundaries",
        r###"function check(v){if(!v)throw new Error('class naming oracle');}
var result=(function(){
 var Wrapped=(((class {}))),Comma=(0,class {}),Conditional=(true?class {}:class {}),Alternative=(false?class {}:class {});
 var Assigned;Assigned=((class {}));
 var o={wrapped:(class {}),comma:(0,class {}),conditional:(true?class {}:class {})};
 check(Wrapped.name==='Wrapped'&&Assigned.name==='Assigned'&&o.wrapped.name==='wrapped');
 check(Comma.name===''&&Conditional.name===''&&Alternative.name===''&&o.comma.name===''&&o.conditional.name==='');
 return Wrapped.name+'|'+Assigned.name+'|'+o.wrapped.name+'|empty';
})();
"###,
        r###"result
"###,
        Value::String("Wrapped|Assigned|wrapped|empty".into()),
    );
}

#[test]
fn class_public_naming_hints_preserve_static_name_and_length() {
    observe(
        "naming_hints_preserve_static_name_and_length",
        r###"function check(v){if(!v)throw new Error('class naming oracle');}
var result=(function(){
 var reads=0,Assigned;Assigned=class {static name(){return 1;} static get length(){reads++;return 2;}};
 var Logical=0;Logical||=class {static get name(){reads++;return 3;} static length(){return 4;}};
 function defaults(Default=class {static name(){return 5;} static length(){return 6;}}){return Default;}
 var Default=defaults(),o={field:class {static name(){return 7;} static length(){return 8;}}};
 var Wrapped=(class {static name(){return 9;} static length(){return 10;}});
 check(reads===0);
 var n=Object.getOwnPropertyDescriptor(Assigned,'name'),l=Object.getOwnPropertyDescriptor(Assigned,'length');
 check(n.value===Assigned.name&&n.writable&&!n.enumerable&&n.configurable);
 check(typeof l.get==='function'&&l.get.name==='get length'&&!l.enumerable&&l.configurable);
 check(Assigned.name()===1&&Assigned.length===2&&Logical.name===3&&Logical.length()===4);
 check(Default.name()===5&&Default.length()===6&&o.field.name()===7&&o.field.length()===8);
 check(Wrapped.name()===9&&Wrapped.length()===10&&reads===2);
 return 'members:'+reads;
})();
"###,
        r###"result
"###,
        Value::String("members:2".into()),
    );
}

#[test]
fn class_public_context_strict_heritage_and_computed_keys() {
    observe(
        "context_strict_heritage_and_computed_keys",
        r###"function check(v){if(!v)throw new Error('class context oracle');}
var result=(function(){
 var trace='',box={};Object.defineProperty(box,'x',{value:0,writable:false,configurable:true});
 try {class H extends (trace+='H',box.x=1,Object) {}trace+='X';}
 catch(e){check(e instanceof TypeError);trace+='T';}
 try {class K {[(trace+='K',box.x=2,'m')](){}}trace+='Y';}
 catch(e){check(e instanceof TypeError);trace+='T';}
 try {class S {static [(trace+='S',box.x=3,'m')](){}}trace+='Z';}
 catch(e){check(e instanceof TypeError);trace+='T';}
 check(box.x===0);return trace+'|'+box.x;
})();
"###,
        r###"result
"###,
        Value::String("HTKTST|0".into()),
    );
}

#[test]
fn class_public_context_nested_class_evaluation_uses_outer_super() {
    observe(
        "context_nested_class_evaluation_uses_outer_super",
        r###"function check(v){if(!v)throw new Error('class context oracle');}
var result=(function(){
 var trace='';function B(v){trace+='B'+v;this.value=v;}
 class H extends B {constructor(){
  class Inner extends (trace+='H',super(1),Object) {method(){return [this,new.target];}}
  check(this.value===1);this.Inner=Inner;trace+='h';
 }}
 var h=new H(),hi=new h.Inner(),hp=hi.method();
 check(hp[0]===hi&&hp[1]===undefined&&Object.getPrototypeOf(h)===H.prototype);
 class K extends B {constructor(){
  class Inner {
   [(trace+='K',super(2),'m')](){return [this,new.target];}
   static [(check(new.target===K),trace+='S','s')](){return [this,new.target];}
  }
  check(this.value===2);this.Inner=Inner;trace+='k';
 }}
 var k=new K(),ki=new k.Inner(),kp=ki.m(),sp=k.Inner.s();
 check(kp[0]===ki&&kp[1]===undefined&&sp[0]===k.Inner&&sp[1]===undefined);
 check(Object.getPrototypeOf(k)===K.prototype);return trace;
})();
"###,
        r###"result
"###,
        Value::String("HB1hKB2Sk".into()),
    );
}

#[test]
fn class_public_context_escaped_super_after_throw_preserves_owner() {
    observe(
        "context_escaped_super_after_throw_preserves_owner",
        r###"function check(v){if(!v)throw new Error('class context oracle');}
var result=(function(){
 var trace='',saved,read,token={};
 function A(v){trace+='A'+v;this.value=v;this.target=new.target;}
 function B(v){trace+='B'+v;this.value=v;this.target=new.target;}
 function Alternate(){}
 class D extends A {constructor(){
  trace+='D';saved=(v)=>super(v);read=()=>[this,new.target];throw token;
 }}
 try{Reflect.construct(D,[],Alternate);trace+='X';}catch(e){check(e===token);trace+='T';}
 try{read();trace+='Y';}catch(e){check(e instanceof ReferenceError);trace+='R';}
 Object.setPrototypeOf(D,B);var first=saved.call({},7),pair=read.call({});
 check(first.value===7&&first.target===Alternate&&Object.getPrototypeOf(first)===Alternate.prototype);
 check(pair[0]===first&&pair[1]===Alternate);
 Object.setPrototypeOf(D,A);
 try{saved(8);trace+='Z';}catch(e){check(e instanceof ReferenceError);trace+='R';}
 check(read()[0]===first);return trace;
})();
"###,
        r###"result
"###,
        Value::String("DTRB7A8R".into()),
    );
}

#[test]
fn class_public_context_derived_default_super_before_body() {
    observe(
        "context_derived_default_super_before_body",
        r###"function check(v){if(!v)throw new Error('class context oracle');}
var result=(function(){
 var trace='';function B(v){trace+='B'+v;this.value=v;this.target=new.target;}
 class D extends B {constructor(x=(trace+='P',super(4)),same=(trace+='Q',this)){
  trace+='D';check(x===this&&same===this&&this.value===4);
 }}
 var d=new D();check(d.target===D&&Object.getPrototypeOf(d)===D.prototype);
 try{new D({});trace+='X';}catch(e){check(e instanceof ReferenceError);trace+='R';}
 return trace;
})();
"###,
        r###"result
"###,
        Value::String("PB4QDQR".into()),
    );
}

#[test]
fn class_public_context_nested_base_constructor_rejects_outer_super() {
    early_error(
        "context_nested_base_constructor_rejects_outer_super",
        r###"var touched=0;
"###,
        r###"touched=1;
class Outer extends Object {constructor(){class Inner {constructor(){super();}}super();}}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_context_nested_method_arrow_rejects_outer_super() {
    early_error(
        "context_nested_method_arrow_rejects_outer_super",
        r###"var touched=0;
"###,
        r###"touched=1;
class Outer extends Object {constructor(){class Inner extends Object {m(){return ()=>super();}}super();}}
"###,
        r###"touched
"###,
        Value::Number(0.0),
    );
}

#[test]
fn class_public_naming_unicode_identifiers_and_computed_keys() {
    observe(
        "naming_unicode_identifiers_and_computed_keys",
        r###"function check(v){if(!v)throw new Error('class Unicode oracle');}
var result=(function(){
 function units(s,a){check(s.length===a.length);for(var i=0;i<a.length;i++){check(s.charCodeAt(i)===a[i]);}}
 class Café {}
 var π;π=class {};
 function defaults(字=class {}){return 字;}
 var parameter=defaults(),explicit=class 𐐀 {},é=class {};
 check(Café.name==='Caf\u00E9'&&π.name==='\u03C0'&&parameter.name==='\u5B57');
 check(explicit.name==='\u{10400}'&&é.name==='e\u0301'&&é.name!=='\u00E9');
 units(Café.name,[67,97,102,233]);units(π.name,[960]);units(parameter.name,[23383]);
 units(explicit.name,[55297,56320]);units(é.name,[101,769]);
 var key='雪😀',symbol=Symbol('λ\uD800'),values={[key]:class {},[symbol]:class {}};
 class Methods {[key](){} static [symbol](){}}
 check(values[key].name===key&&Methods.prototype[key].name===key);
 check(values[symbol].name==='[λ\uD800]'&&Methods[symbol].name==='[λ\uD800]');
 units(values[key].name,[38634,55357,56832]);units(Methods.prototype[key].name,[38634,55357,56832]);
 units(values[symbol].name,[91,955,55296,93]);units(Methods[symbol].name,[91,955,55296,93]);
 return 'utf16-names';
})();
"###,
        r###"result
"###,
        Value::String("utf16-names".into()),
    );
}
