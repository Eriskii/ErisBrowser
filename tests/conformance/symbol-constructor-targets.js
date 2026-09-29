// CASE: symbol-alternate
var target;function F(){target=new.target;this.x=1;}
var object=Reflect.construct(F,[],Symbol);
assert.sameValue(target,Symbol);assert.sameValue(Object.getPrototypeOf(object),Symbol.prototype);
assert.sameValue(object.x,1);assert.throws(TypeError,function(){Symbol.prototype.valueOf.call(object);});
assert.sameValue(Object.getPrototypeOf(Reflect.construct(Object,[],Symbol)),Symbol.prototype);
// CASE: symbol-construction-order
var log='',B=(function(){}).bind(null);
Object.defineProperty(B,'prototype',{get:function(){log+='p';return {};}});
var description={toString:function(){log+='s';return 'x';}};
var list={get length(){log+='l';return 1;},get 0(){log+='0';return description;}};
assert.throws(TypeError,function(){Reflect.construct(Symbol,list,B);});assert.sameValue(log,'l0');
log='';assert.throws(TypeError,function(){Reflect.construct(Symbol,list,()=>0);});assert.sameValue(log,'');
assert.throws(TypeError,function(){new Symbol(description);});assert.sameValue(log,'');
// CASE: symbol-argument-abrupt
var marker={},caught,list={get length(){throw marker;}};
try{Reflect.construct(Symbol,list);}catch(e){caught=e;}assert.sameValue(caught,marker);
caught=undefined;list={length:1,get 0(){throw marker;}};
try{Reflect.construct(Symbol,list);}catch(e){caught=e;}assert.sameValue(caught,marker);
// CASE: bound-symbol-target
var B=Symbol.bind(null,'bound');
assert.sameValue(Object.getPrototypeOf(Reflect.construct(Object,[],B)),Object.prototype);
var proto={};Object.defineProperty(B,'prototype',{value:proto});
assert.sameValue(Object.getPrototypeOf(Reflect.construct(Object,[],B)),proto);
var target;function F(){target=new.target;}
assert.sameValue(Object.getPrototypeOf(Reflect.construct(F,[],B)),proto);assert.sameValue(target,B);
assert.throws(TypeError,function(){new B();});
// CASE: bound-symbol-construction-order
var log='',G=(function(){}).bind(null);
Object.defineProperty(G,'prototype',{get:function(){log+='p';return {};}});
var B=Symbol.bind(null,{toString:function(){log+='s';return 'x';}});
assert.throws(TypeError,function(){Reflect.construct(B,{get length(){log+='l';return 0;}},G);});
assert.sameValue(log,'l');
