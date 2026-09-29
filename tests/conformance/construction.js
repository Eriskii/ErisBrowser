// Local semantic coverage; upstream corpus files are kept separately unchanged.
// CASE: lexical-target
var saved, defaultRead;
function F(a=new.target, read=()=>new.target) {
  assert.sameValue(a,new.target); defaultRead=read;
  saved=()=>()=>new.target;
  function nested(){return new.target;}
  assert.sameValue(nested(),undefined);
  if(this!==undefined){this.target=new.target;}
}
var f=new F(); assert.sameValue(f.target,F);
assert.sameValue(saved()(),F); assert.sameValue(defaultRead(),F);
F();assert.sameValue(saved()(),undefined);assert.sameValue(defaultRead(),undefined);
var receiver={};F.call(receiver);assert.sameValue(receiver.target,undefined);
var m={f(){return ()=>new.target;}};assert.sameValue(m.f()(),undefined);
// CASE: alternate-target
function F(){this.target=new.target;this.self=this;}
function G(){throw new Test262Error('newTarget body executed');}
var p={tag:1};G.prototype=p;
var x=Reflect.construct(F,[],G);
assert.sameValue(Object.getPrototypeOf(x),p);assert.sameValue(x.target,G);assert.sameValue(x.self,x);
G.prototype=null;x=Reflect.construct(F,[],G);assert.sameValue(Object.getPrototypeOf(x),Object.prototype);
function result(){return p;} assert.sameValue(Reflect.construct(result,[],G),p);
function primitive(){return 4;} assert.sameValue(Object.getPrototypeOf(new primitive()),primitive.prototype);
// CASE: bound-target
function F(a,b,c){this.args=[a,b,c];this.target=new.target;}
var wrong={}, B=F.bind(wrong,1), C=B.bind(wrong,2);
Object.defineProperty(C,'prototype',{get:function(){throw new Test262Error('bound prototype read');}});
var x=new C(3);assert.compareArray(x.args,[1,2,3]);assert.sameValue(x.target,F);
assert.sameValue(Object.getPrototypeOf(x),F.prototype);assert.sameValue(wrong.args,undefined);
function G(){};G.prototype={};x=Reflect.construct(C,[3],G);
assert.sameValue(x.target,G);assert.sameValue(Object.getPrototypeOf(x),G.prototype);
var D=G.bind(null),p={};Object.defineProperty(D,'prototype',{value:p});
x=Reflect.construct(F,[],D);assert.sameValue(x.target,D);assert.sameValue(Object.getPrototypeOf(x),p);
assert.throws(TypeError,function(){Reflect.construct((()=>1).bind(null),[]);});
// CASE: validation-order
var log='',list={get length(){log+='l';return 0;}};
assert.sameValue(typeof Reflect.apply,'function');assert.sameValue(typeof Reflect.construct,'function');
assert.throws(TypeError,function(){Reflect.apply({},null,list);});
assert.throws(TypeError,function(){Reflect.construct({},list);});
assert.throws(TypeError,function(){Reflect.construct(function(){},list,undefined);});
assert.throws(TypeError,function(){Reflect.construct(function(){},list,()=>0);});
assert.sameValue(log,'');
var values=[null,undefined,1,'abc',true,Symbol()];for(var i=0;i<values.length;i++){(function(v){assert.throws(TypeError,function(){Reflect.construct(function(){},v);});assert.throws(TypeError,function(){Reflect.apply(function(){},null,v);});})(values[i]);}
// CASE: argument-order
var log='',targetProto={},B=(function(){}).bind(null);
Object.defineProperty(B,'prototype',{get:function(){log+='p';return targetProto;}});
var list={get length(){log+='l';return {valueOf:function(){log+='v';return 2.9;}};},get 0(){log+='0';delete this[1];return 7;},1:9};
function F(a,b){log+='f';assert.sameValue(a,7);assert.sameValue(b,undefined);assert.sameValue(new.target,B);}
var x=Reflect.construct(F,list,B);assert.sameValue(log,'lv0pf');assert.sameValue(Object.getPrototypeOf(x),targetProto);
log='';Object.defineProperty(list,'length',{get:function(){log+='l';return -1;}});
Reflect.apply(function(){assert.sameValue(arguments.length,0);},null,list);assert.sameValue(log,'l');
// CASE: apply-receivers
var object={};function f(a,b){'use strict';assert.sameValue(new.target,undefined);return [this,a,b];}
var x=Reflect.apply(f,object,{0:1,1:2,length:2});assert.compareArray(x,[object,1,2]);
assert.sameValue(Reflect.apply(f,3,[])[0],3);assert.sameValue(Reflect.apply(f,null,[])[0],null);
assert.sameValue(f.apply(object,null)[0],object);
assert.throws(TypeError,function(){Reflect.apply(f,object,null);});
assert.throws(TypeError,function(){Reflect.apply(f,object,'ab');});
var marker={};assert.throws(Test262Error,function(){Reflect.apply(function(){throw new Test262Error();},null,[]);});
// CASE: abrupt-order
var marker={},log='',list={get length(){throw marker;}};
function F(){log+='f';}
try{Reflect.construct(F,list);}catch(e){assert.sameValue(e,marker);log+='c';}assert.sameValue(log,'c');
list={length:2,get 0(){throw marker;},get 1(){log+='1';}};
try{Reflect.apply(F,null,list);}catch(e){assert.sameValue(e,marker);log+='c';}assert.sameValue(log,'cc');
var B=F.bind(null);Object.defineProperty(B,'prototype',{get:function(){throw marker;}});
try{Reflect.construct(F,[],B);}catch(e){assert.sameValue(e,marker);log+='c';}assert.sameValue(log,'ccc');
function nested(){try{throw marker;}finally{assert.sameValue(new.target,nested);}}
try{new nested();}catch(e){assert.sameValue(e,marker);}
assert.sameValue(Reflect.apply(function(){return new.target;},null,[]),undefined);
// CASE: boxed-native-order
var log='',proto={},B=(function(){}).bind(null);
Object.defineProperty(B,'prototype',{get:function(){log+='p';return proto;}});
var value={toString:function(){log+='s';return 'ab';},valueOf:function(){log+='n';return 7;}};
var s=Reflect.construct(String,[value],B);assert.sameValue(log,'sp');assert.sameValue(String.prototype.valueOf.call(s),'ab');assert.sameValue(s[1],'b');assert.sameValue(s.length,2);assert.sameValue(Object.getPrototypeOf(s),proto);
log='';var n=Reflect.construct(Number,[value],B);assert.sameValue(log,'np');assert.sameValue(Number.prototype.valueOf.call(n),7);assert.sameValue(Object.getPrototypeOf(n),proto);
log='';var b=Reflect.construct(Boolean,[value],B);assert.sameValue(log,'p');assert.sameValue(Boolean.prototype.valueOf.call(b),true);
log='';assert.throws(TypeError,function(){Reflect.construct(String,[Symbol()],B);});assert.sameValue(log,'');
// CASE: native-allocation-order
var log='',proto={},B=(function(){}).bind(null);
Object.defineProperty(B,'prototype',{get:function(){log+='p';return proto;}});
var o={};assert.sameValue(Reflect.construct(Object,[o]),o);
var x=Reflect.construct(Object,[o],B);assert.notSameValue(x,o);assert.sameValue(Object.getPrototypeOf(x),proto);assert.sameValue(log,'p');
log='';assert.throws(RangeError,function(){Reflect.construct(Array,[-1],B);});assert.sameValue(log,'p');
log='';x=Reflect.construct(Array,[1,2],B);assert.sameValue(x.length,2);assert.sameValue(x[1],2);assert.sameValue(Array.isArray(x),true);assert.sameValue(Object.getPrototypeOf(x),proto);
log='';x=Reflect.construct(TypeError,[{toString:function(){log+='m';return 'oops';}}],B);assert.sameValue(log,'pm');assert.sameValue(x.message,'oops');assert.sameValue(Object.getPrototypeOf(x),proto);
log='';x=Reflect.construct(RegExp,[{toString:function(){log+='s';return 'a';}}],B);assert.sameValue(log,'ps');assert.sameValue(RegExp.prototype.test.call(x,'a'),true);assert.sameValue(Object.getPrototypeOf(x),proto);
// CASE: intrinsic-fallback
function B(){};B.prototype=3;
for(var i=0;i<7;i++){var C=[Object,Array,String,Number,Boolean,TypeError,RegExp][i];var x=Reflect.construct(C,[],B);assert.sameValue(Object.getPrototypeOf(x),C.prototype);}
// CASE: metadata
for(var i=0;i<2;i++){var name=['apply','construct'][i],method=Reflect[name];
verifyProperty(Reflect,name,{value:method,writable:true,enumerable:false,configurable:true},{restore:true});
verifyProperty(method,'length',{value:[3,2][i],writable:false,enumerable:false,configurable:true},{restore:true});
verifyProperty(method,'name',{value:name,writable:false,enumerable:false,configurable:true},{restore:true});
assert.sameValue(Object.getPrototypeOf(method),Function.prototype);
assert.sameValue(Object.prototype.hasOwnProperty.call(method,'prototype'),false);assert.throws(TypeError,function(){new method();});}
var saved=Reflect.construct;delete Reflect.construct;assert.sameValue(Reflect.construct,undefined);function F(){this.target=new.target;}assert.sameValue(saved(F,[]).target,F);
// CASE: unary-and-member
function F(){assert.sameValue(delete new.target,true);assert.sameValue(typeof new.target,'function');assert.sameValue(new.target.prototype,F.prototype);assert.sameValue((new.target),F);}
new F();function G(){return !new.target;}assert.sameValue(G(),true);
function H(done){if(!done){return new new.target(true);}this.x=1;}assert.sameValue(new H().x,1);
// CASE: property-isolation
var global=globalThis;
Object.defineProperty(global,'new.target',{get:function(){throw new Test262Error('global getter reached');},configurable:true});
function F(){assert.sameValue(new.target,F);assert.sameValue(Object.prototype.hasOwnProperty.call(this,'new.target'),false);}
new F();delete global['new.target'];
