// CASE: every-metadata-and-nonconstructability
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var d=Object.getOwnPropertyDescriptor(Array.prototype,name);
assert.sameValue(d.value,m);assert.sameValue(d.writable,true);assert.sameValue(d.enumerable,false);assert.sameValue(d.configurable,true);
var n=Object.getOwnPropertyDescriptor(m,'name'),l=Object.getOwnPropertyDescriptor(m,'length');
assert.sameValue(n.value,name);assert.sameValue(n.writable,false);assert.sameValue(n.enumerable,false);assert.sameValue(n.configurable,true);
assert.sameValue(l.value,1);assert.sameValue(l.writable,false);assert.sameValue(l.enumerable,false);assert.sameValue(l.configurable,true);
assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);
assert.throws(TypeError,function(){new m(function(){return true;});});
assert.notSameValue(Array.prototype.every,Array.prototype.some);
// CASE: some-metadata-and-nonconstructability
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var d=Object.getOwnPropertyDescriptor(Array.prototype,name);
assert.sameValue(d.value,m);assert.sameValue(d.writable,true);assert.sameValue(d.enumerable,false);assert.sameValue(d.configurable,true);
var n=Object.getOwnPropertyDescriptor(m,'name'),l=Object.getOwnPropertyDescriptor(m,'length');
assert.sameValue(n.value,name);assert.sameValue(n.writable,false);assert.sameValue(n.enumerable,false);assert.sameValue(n.configurable,true);
assert.sameValue(l.value,1);assert.sameValue(l.writable,false);assert.sameValue(l.enumerable,false);assert.sameValue(l.configurable,true);
assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);
assert.throws(TypeError,function(){new m(function(){return true;});});
assert.notSameValue(Array.prototype.every,Array.prototype.some);
// CASE: every-empty-result-and-callback-validation
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var calls=0;assert.sameValue(m.call([],function(){calls++;}),isEvery);assert.sameValue(calls,0);
var bad=[undefined,null,false,1,'callback',{},[],Symbol('callback')];
for(var i=0;i<bad.length;i++){assert.throws(TypeError,function(){m.call([],bad[i]);});}
assert.throws(TypeError,function(){m.call([]);});
// CASE: some-empty-result-and-callback-validation
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var calls=0;assert.sameValue(m.call([],function(){calls++;}),isEvery);assert.sameValue(calls,0);
var bad=[undefined,null,false,1,'callback',{},[],Symbol('callback')];
for(var i=0;i<bad.length;i++){assert.throws(TypeError,function(){m.call([],bad[i]);});}
assert.throws(TypeError,function(){m.call([]);});
// CASE: every-nullish-receiver-before-callback
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var calls=0,poison={get call(){calls++;throw 'call';},get valueOf(){calls++;throw 'value';}};
assert.throws(TypeError,function(){m.call(null,poison);});assert.throws(TypeError,function(){m.call(undefined,poison);});assert.sameValue(calls,0);
// CASE: some-nullish-receiver-before-callback
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var calls=0,poison={get call(){calls++;throw 'call';},get valueOf(){calls++;throw 'value';}};
assert.throws(TypeError,function(){m.call(null,poison);});assert.throws(TypeError,function(){m.call(undefined,poison);});assert.sameValue(calls,0);
// CASE: every-length-access-before-invalid-callback
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var reason={},caught,log='',o={get length(){log+='L';throw reason;},get 0(){throw 'index';}};
try{m.call(o,null);}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(log,'L');
// CASE: some-length-access-before-invalid-callback
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var reason={},caught,log='',o={get length(){log+='L';throw reason;},get 0(){throw 'index';}};
try{m.call(o,null);}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(log,'L');
// CASE: every-length-conversion-order-and-fraction
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var log='',calls=0,o={0:4,1:5,2:6,get length(){log+='L';return {valueOf:function(){log+='V';return {};},toString:function(){log+='S';return '2.9';}};}};
assert.sameValue(m.call(o,function(v,k,obj){log+=k;assert.sameValue(obj,o);calls++;return keepGoing;}),isEvery);
assert.sameValue(log,'LVS01');assert.sameValue(calls,2);
// CASE: some-length-conversion-order-and-fraction
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var log='',calls=0,o={0:4,1:5,2:6,get length(){log+='L';return {valueOf:function(){log+='V';return {};},toString:function(){log+='S';return '2.9';}};}};
assert.sameValue(m.call(o,function(v,k,obj){log+=k;assert.sameValue(obj,o);calls++;return keepGoing;}),isEvery);
assert.sameValue(log,'LVS01');assert.sameValue(calls,2);
// CASE: every-length-symbol-conversion-and-abrupt-identity
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var reason={},caught,log='',v={};v[Symbol.toPrimitive]=function(hint){assert.sameValue(hint,'number');log+='N';throw reason;};
try{m.call({length:v},null);}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(log,'N');
assert.throws(TypeError,function(){m.call({length:Symbol('length')},function(){throw 'callback';});});
// CASE: some-length-symbol-conversion-and-abrupt-identity
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var reason={},caught,log='',v={};v[Symbol.toPrimitive]=function(hint){assert.sameValue(hint,'number');log+='N';throw reason;};
try{m.call({length:v},null);}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(log,'N');
assert.throws(TypeError,function(){m.call({length:Symbol('length')},function(){throw 'callback';});});
// CASE: every-nonpositive-and-missing-lengths
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var values=[undefined,null,false,0,-0,-2,-Infinity,NaN],calls=0;
for(var i=0;i<values.length;i++){assert.sameValue(m.call({length:values[i],0:9},function(){calls++;return stop;}),isEvery);}
assert.sameValue(m.call({},function(){calls++;}),isEvery);assert.sameValue(calls,0);
// CASE: some-nonpositive-and-missing-lengths
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var values=[undefined,null,false,0,-0,-2,-Infinity,NaN],calls=0;
for(var i=0;i<values.length;i++){assert.sameValue(m.call({length:values[i],0:9},function(){calls++;return stop;}),isEvery);}
assert.sameValue(m.call({},function(){calls++;}),isEvery);assert.sameValue(calls,0);
// CASE: every-inherited-length-getter-original-receiver
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var p={get length(){assert.sameValue(this,o);return 2;}},o=Object.create(p);o[0]=4;o[1]=5;var seen='';
assert.sameValue(m.call(o,function(v,k,obj){assert.sameValue(obj,o);seen+=v;return keepGoing;}),isEvery);assert.sameValue(seen,'45');
// CASE: some-inherited-length-getter-original-receiver
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var p={get length(){assert.sameValue(this,o);return 2;}},o=Object.create(p);o[0]=4;o[1]=5;var seen='';
assert.sameValue(m.call(o,function(v,k,obj){assert.sameValue(obj,o);seen+=v;return keepGoing;}),isEvery);assert.sameValue(seen,'45');
// CASE: every-invalid-callback-does-not-coerce-or-read-index
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var calls=0,cb={get call(){calls++;throw 'call';},get valueOf(){calls++;throw 'value';}},o={length:1,get 0(){calls++;throw 'index';}};
assert.throws(TypeError,function(){m.call(o,cb);});assert.sameValue(calls,0);
// CASE: some-invalid-callback-does-not-coerce-or-read-index
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var calls=0,cb={get call(){calls++;throw 'call';},get valueOf(){calls++;throw 'value';}},o={length:1,get 0(){calls++;throw 'index';}};
assert.throws(TypeError,function(){m.call(o,cb);});assert.sameValue(calls,0);
// CASE: every-truthiness-without-result-coercion
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var calls=0,poison={get valueOf(){calls++;throw 'value';},get toString(){calls++;throw 'string';}};
Object.defineProperty(poison,Symbol.toPrimitive,{get:function(){calls++;throw 'primitive';}});
var truthy=[true,1,-1,Infinity,'0',{},[],function(){},Symbol('truthy'),new Boolean(false),poison];
var falsy=[undefined,null,false,0,-0,NaN,''];
for(var i=0;i<truthy.length;i++){assert.sameValue(m.call([1],function(){return truthy[i];}),true);}
for(var j=0;j<falsy.length;j++){assert.sameValue(m.call([1],function(){return falsy[j];}),false);}
assert.sameValue(calls,0);
// CASE: some-truthiness-without-result-coercion
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var calls=0,poison={get valueOf(){calls++;throw 'value';},get toString(){calls++;throw 'string';}};
Object.defineProperty(poison,Symbol.toPrimitive,{get:function(){calls++;throw 'primitive';}});
var truthy=[true,1,-1,Infinity,'0',{},[],function(){},Symbol('truthy'),new Boolean(false),poison];
var falsy=[undefined,null,false,0,-0,NaN,''];
for(var i=0;i<truthy.length;i++){assert.sameValue(m.call([1],function(){return truthy[i];}),true);}
for(var j=0;j<falsy.length;j++){assert.sameValue(m.call([1],function(){return falsy[j];}),false);}
assert.sameValue(calls,0);
// CASE: every-short-circuit-skips-later-getters
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var log='',o={length:3,get 0(){log+='G';return 7;},get 1(){throw 'later getter';},get 2(){throw 'last getter';}};
assert.sameValue(m.call(o,function(v,k,obj){log+='C';assert.sameValue(v,7);assert.sameValue(k,0);assert.sameValue(obj,o);return stop;}),stop);
assert.sameValue(log,'GC');
// CASE: some-short-circuit-skips-later-getters
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var log='',o={length:3,get 0(){log+='G';return 7;},get 1(){throw 'later getter';},get 2(){throw 'last getter';}};
assert.sameValue(m.call(o,function(v,k,obj){log+='C';assert.sameValue(v,7);assert.sameValue(k,0);assert.sameValue(obj,o);return stop;}),stop);
assert.sameValue(log,'GC');
// CASE: every-ascending-visits-and-exact-three-arguments
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var a=[2,3,4],log='';assert.sameValue(m.call(a,function(v,k,o){assert.sameValue(arguments.length,3);assert.sameValue(arguments[0],v);assert.sameValue(arguments[1],k);assert.sameValue(arguments[2],a);assert.sameValue(o,a);assert.sameValue(v,k+2);log+=k;return keepGoing;}),isEvery);assert.sameValue(log,'012');
// CASE: some-ascending-visits-and-exact-three-arguments
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var a=[2,3,4],log='';assert.sameValue(m.call(a,function(v,k,o){assert.sameValue(arguments.length,3);assert.sameValue(arguments[0],v);assert.sameValue(arguments[1],k);assert.sameValue(arguments[2],a);assert.sameValue(o,a);assert.sameValue(v,k+2);log+=k;return keepGoing;}),isEvery);assert.sameValue(log,'012');
// CASE: every-holes-versus-present-undefined
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var calls=0;assert.sameValue(m.call([,,,],function(){calls++;throw 'hole';}),isEvery);assert.sameValue(calls,0);
var a=[,undefined,,],seen='';assert.sameValue(m.call(a,function(v,k,o){assert.sameValue(v,undefined);assert.sameValue(k,1);assert.sameValue(o,a);seen+=k;return keepGoing;}),isEvery);assert.sameValue(seen,'1');
// CASE: some-holes-versus-present-undefined
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var calls=0;assert.sameValue(m.call([,,,],function(){calls++;throw 'hole';}),isEvery);assert.sameValue(calls,0);
var a=[,undefined,,],seen='';assert.sameValue(m.call(a,function(v,k,o){assert.sameValue(v,undefined);assert.sameValue(k,1);assert.sameValue(o,a);seen+=k;return keepGoing;}),isEvery);assert.sameValue(seen,'1');
// CASE: every-inherited-nonenumerable-indices
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var p={},a=[1,,3],seen='';Object.defineProperty(p,'1',{get:function(){assert.sameValue(this,a);return 8;}});Object.setPrototypeOf(a,p);
assert.sameValue(m.call(a,function(v){seen+=v;return keepGoing;}),isEvery);assert.sameValue(seen,'183');assert.sameValue(Object.getOwnPropertyDescriptor(a,'1'),undefined);
// CASE: some-inherited-nonenumerable-indices
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var p={},a=[1,,3],seen='';Object.defineProperty(p,'1',{get:function(){assert.sameValue(this,a);return 8;}});Object.setPrototypeOf(a,p);
assert.sameValue(m.call(a,function(v){seen+=v;return keepGoing;}),isEvery);assert.sameValue(seen,'183');assert.sameValue(Object.getOwnPropertyDescriptor(a,'1'),undefined);
// CASE: every-callback-delete-and-add-future-properties
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var a=[1,2,,4],seen='';assert.sameValue(m.call(a,function(v,k){seen+=k+':'+v+';';if(k===0){delete a[1];a[2]=7;}return keepGoing;}),isEvery);assert.sameValue(seen,'0:1;2:7;3:4;');
// CASE: some-callback-delete-and-add-future-properties
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var a=[1,2,,4],seen='';assert.sameValue(m.call(a,function(v,k){seen+=k+':'+v+';';if(k===0){delete a[1];a[2]=7;}return keepGoing;}),isEvery);assert.sameValue(seen,'0:1;2:7;3:4;');
// CASE: every-getter-mutation-is-live-before-callback
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var log='',o={length:3,1:2};Object.defineProperty(o,'0',{get:function(){log+='g';delete o[1];Object.defineProperty(o,'2',{get:function(){log+='h';return 9;}});return 4;}});
assert.sameValue(m.call(o,function(v,k){log+='c'+k+v;return keepGoing;}),isEvery);assert.sameValue(log,'gc04hc29');
// CASE: some-getter-mutation-is-live-before-callback
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var log='',o={length:3,1:2};Object.defineProperty(o,'0',{get:function(){log+='g';delete o[1];Object.defineProperty(o,'2',{get:function(){log+='h';return 9;}});return 4;}});
assert.sameValue(m.call(o,function(v,k){log+='c'+k+v;return keepGoing;}),isEvery);assert.sameValue(log,'gc04hc29');
// CASE: every-saved-length-ignores-appended-properties
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var a=[1,2],calls=0;assert.sameValue(m.call(a,function(v,k){calls++;if(k===0){a[2]=8;a[3]=9;}return keepGoing;}),isEvery);assert.sameValue(calls,2);assert.sameValue(a.length,4);
// CASE: some-saved-length-ignores-appended-properties
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var a=[1,2],calls=0;assert.sameValue(m.call(a,function(v,k){calls++;if(k===0){a[2]=8;a[3]=9;}return keepGoing;}),isEvery);assert.sameValue(calls,2);assert.sameValue(a.length,4);
// CASE: every-array-shrink-deletes-own-but-retains-inherited-visits
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var p={2:9},a=[1,2,3],seen='';Object.setPrototypeOf(a,p);
assert.sameValue(m.call(a,function(v,k){seen+=k+':'+v+';';if(k===0)a.length=1;return keepGoing;}),isEvery);assert.sameValue(seen,'0:1;2:9;');assert.sameValue(a.length,1);
// CASE: some-array-shrink-deletes-own-but-retains-inherited-visits
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var p={2:9},a=[1,2,3],seen='';Object.setPrototypeOf(a,p);
assert.sameValue(m.call(a,function(v,k){seen+=k+':'+v+';';if(k===0)a.length=1;return keepGoing;}),isEvery);assert.sameValue(seen,'0:1;2:9;');assert.sameValue(a.length,1);
// CASE: every-ordinary-length-shrink-keeps-indexed-properties
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var o={length:3,0:1,1:2,2:3},seen='';assert.sameValue(m.call(o,function(v,k){seen+=k;o.length=0;return keepGoing;}),isEvery);assert.sameValue(seen,'012');
// CASE: some-ordinary-length-shrink-keeps-indexed-properties
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var o={length:3,0:1,1:2,2:3},seen='';assert.sameValue(m.call(o,function(v,k){seen+=k;o.length=0;return keepGoing;}),isEvery);assert.sameValue(seen,'012');
// CASE: every-prototype-switch-affects-next-lookup
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var p={1:8},o={length:3,0:1,2:3},seen='';assert.sameValue(m.call(o,function(v,k){seen+=v;if(k===0)Object.setPrototypeOf(o,p);return keepGoing;}),isEvery);assert.sameValue(seen,'183');
// CASE: some-prototype-switch-affects-next-lookup
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var p={1:8},o={length:3,0:1,2:3},seen='';assert.sameValue(m.call(o,function(v,k){seen+=v;if(k===0)Object.setPrototypeOf(o,p);return keepGoing;}),isEvery);assert.sameValue(seen,'183');
// CASE: every-captured-callback-survives-length-getter-replacement
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var calls=0,cb=function(){calls++;return keepGoing;},o={0:1,get length(){cb=null;return 1;}};
assert.sameValue(m.call(o,cb),isEvery);assert.sameValue(calls,1);assert.sameValue(cb,null);
// CASE: some-captured-callback-survives-length-getter-replacement
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var calls=0,cb=function(){calls++;return keepGoing;},o={0:1,get length(){cb=null;return 1;}};
assert.sameValue(m.call(o,cb),isEvery);assert.sameValue(calls,1);assert.sameValue(cb,null);
// CASE: every-omitted-thisarg-respects-callback-strictness
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var strict=(function(){return this;})()===undefined,calls=0;
assert.sameValue(m.call([1,2],function(){calls++;assert.sameValue(this,strict?undefined:window);return keepGoing;}),isEvery);assert.sameValue(calls,2);
// CASE: some-omitted-thisarg-respects-callback-strictness
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var strict=(function(){return this;})()===undefined,calls=0;
assert.sameValue(m.call([1,2],function(){calls++;assert.sameValue(this,strict?undefined:window);return keepGoing;}),isEvery);assert.sameValue(calls,2);
// CASE: every-explicit-thisarg-and-poison-hooks
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var marker={valueOf:function(){throw 'coerce';},toString:function(){throw 'coerce';}},calls=0;
marker[Symbol.toPrimitive]=function(){throw 'coerce';};
assert.sameValue(m.call([1,2],function(){calls++;assert.sameValue(this,marker);return keepGoing;},marker),isEvery);assert.sameValue(calls,2);
assert.sameValue(m.call([1],function(){'use strict';assert.sameValue(this,null);return keepGoing;},null),isEvery);
assert.sameValue(m.call([1],function(){'use strict';assert.sameValue(this,7);return keepGoing;},7),isEvery);
// CASE: some-explicit-thisarg-and-poison-hooks
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var marker={valueOf:function(){throw 'coerce';},toString:function(){throw 'coerce';}},calls=0;
marker[Symbol.toPrimitive]=function(){throw 'coerce';};
assert.sameValue(m.call([1,2],function(){calls++;assert.sameValue(this,marker);return keepGoing;},marker),isEvery);assert.sameValue(calls,2);
assert.sameValue(m.call([1],function(){'use strict';assert.sameValue(this,null);return keepGoing;},null),isEvery);
assert.sameValue(m.call([1],function(){'use strict';assert.sameValue(this,7);return keepGoing;},7),isEvery);
// CASE: every-primitive-thisarg-boxing-is-mode-dependent
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var strict=(function(){return this;})()===undefined;
assert.sameValue(m.call([1],function(){if(strict){assert.sameValue(this,7);}else{assert.sameValue(typeof this,'object');assert.sameValue(this.valueOf(),7);}return keepGoing;},7),isEvery);
// CASE: some-primitive-thisarg-boxing-is-mode-dependent
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var strict=(function(){return this;})()===undefined;
assert.sameValue(m.call([1],function(){if(strict){assert.sameValue(this,7);}else{assert.sameValue(typeof this,'object');assert.sameValue(this.valueOf(),7);}return keepGoing;},7),isEvery);
// CASE: every-bound-callback-and-native-callback
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var marker={},other={},a=[1,2],calls=0,cb=function(prefix,v,k,o){assert.sameValue(this,marker);assert.sameValue(prefix,'P');assert.sameValue(arguments.length,4);assert.sameValue(o,a);calls++;return keepGoing;}.bind(marker,'P');
assert.sameValue(m.call(a,cb,other),isEvery);assert.sameValue(calls,2);
assert.sameValue(m.call([1,2],Number.isFinite),true);assert.sameValue(m.call(['x','y'],Number.isFinite),false);
// CASE: some-bound-callback-and-native-callback
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var marker={},other={},a=[1,2],calls=0,cb=function(prefix,v,k,o){assert.sameValue(this,marker);assert.sameValue(prefix,'P');assert.sameValue(arguments.length,4);assert.sameValue(o,a);calls++;return keepGoing;}.bind(marker,'P');
assert.sameValue(m.call(a,cb,other),isEvery);assert.sameValue(calls,2);
assert.sameValue(m.call([1,2],Number.isFinite),true);assert.sameValue(m.call(['x','y'],Number.isFinite),false);
// CASE: every-arrow-callback-lexical-this
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var owner={run:function(){return m.call([1,2],(v,k,o)=>{assert.sameValue(this,owner);return keepGoing;},{unrelated:true});}};assert.sameValue(owner.run(),isEvery);
// CASE: some-arrow-callback-lexical-this
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var owner={run:function(){return m.call([1,2],(v,k,o)=>{assert.sameValue(this,owner);return keepGoing;},{unrelated:true});}};assert.sameValue(owner.run(),isEvery);
// CASE: every-primitive-string-boxing-and-utf16
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var s='A\ud800\udfff',seen='',box;
assert.sameValue(m.call(s,function(v,k,o){assert.sameValue(typeof o,'object');assert.sameValue(Object.getPrototypeOf(o),String.prototype);assert.sameValue(o.valueOf(),s);if(box)assert.sameValue(o,box);box=o;assert.sameValue(v,s[k]);assert.sameValue(v.length,1);seen+=k;return keepGoing;}),isEvery);assert.sameValue(seen,'012');
// CASE: some-primitive-string-boxing-and-utf16
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var s='A\ud800\udfff',seen='',box;
assert.sameValue(m.call(s,function(v,k,o){assert.sameValue(typeof o,'object');assert.sameValue(Object.getPrototypeOf(o),String.prototype);assert.sameValue(o.valueOf(),s);if(box)assert.sameValue(o,box);box=o;assert.sameValue(v,s[k]);assert.sameValue(v.length,1);seen+=k;return keepGoing;}),isEvery);assert.sameValue(seen,'012');
// CASE: every-number-boolean-symbol-boxing
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var calls=0,cb=function(){calls++;return stop;};assert.sameValue(m.call(7,cb),isEvery);assert.sameValue(m.call(false,cb),isEvery);assert.sameValue(m.call(Symbol('receiver'),cb),isEvery);assert.sameValue(calls,0);
Number.prototype.length=1;Number.prototype[0]=9;
assert.sameValue(m.call(7,function(v,k,o){assert.sameValue(v,9);assert.sameValue(k,0);assert.sameValue(o.valueOf(),7);assert.sameValue(Object.getPrototypeOf(o),Number.prototype);return stop;}),stop);
// CASE: some-number-boolean-symbol-boxing
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var calls=0,cb=function(){calls++;return stop;};assert.sameValue(m.call(7,cb),isEvery);assert.sameValue(m.call(false,cb),isEvery);assert.sameValue(m.call(Symbol('receiver'),cb),isEvery);assert.sameValue(calls,0);
Number.prototype.length=1;Number.prototype[0]=9;
assert.sameValue(m.call(7,function(v,k,o){assert.sameValue(v,9);assert.sameValue(k,0);assert.sameValue(o.valueOf(),7);assert.sameValue(Object.getPrototypeOf(o),Number.prototype);return stop;}),stop);
// CASE: every-arguments-mapped-state-remains-live
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var strict=(function(){return this;})()===undefined;
function f(a,b){var seen='';assert.sameValue(m.call(arguments,function(v,k,o){seen+=v;if(k===0)b=9;return keepGoing;}),isEvery);assert.sameValue(seen,strict?'12':'19');}f(1,2);
// CASE: some-arguments-mapped-state-remains-live
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var strict=(function(){return this;})()===undefined;
function f(a,b){var seen='';assert.sameValue(m.call(arguments,function(v,k,o){seen+=v;if(k===0)b=9;return keepGoing;}),isEvery);assert.sameValue(seen,strict?'12':'19');}f(1,2);
// CASE: every-callback-abrupt-identity-and-prior-effects
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var reason={},caught,calls=0,o={length:2,0:1,get 1(){throw 'later getter';}};
try{m.call(o,function(){calls++;o.marker=7;throw reason;});}catch(e){caught=e;}
assert.sameValue(caught,reason);assert.sameValue(calls,1);assert.sameValue(o.marker,7);
// CASE: some-callback-abrupt-identity-and-prior-effects
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var reason={},caught,calls=0,o={length:2,0:1,get 1(){throw 'later getter';}};
try{m.call(o,function(){calls++;o.marker=7;throw reason;});}catch(e){caught=e;}
assert.sameValue(caught,reason);assert.sameValue(calls,1);assert.sameValue(o.marker,7);
// CASE: every-index-getter-abrupt-identity
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var reason=Symbol('abrupt'),caught,calls=0,o={length:2,get 0(){this.marker=4;throw reason;},1:2};
try{m.call(o,function(){calls++;return keepGoing;});}catch(e){caught=e;}
assert.sameValue(caught,reason);assert.sameValue(calls,0);assert.sameValue(o.marker,4);
// CASE: some-index-getter-abrupt-identity
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var reason=Symbol('abrupt'),caught,calls=0,o={length:2,get 0(){this.marker=4;throw reason;},1:2};
try{m.call(o,function(){calls++;return keepGoing;});}catch(e){caught=e;}
assert.sameValue(caught,reason);assert.sameValue(calls,0);assert.sameValue(o.marker,4);
// CASE: every-call-argument-evaluation-before-length-and-callback
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var log='',marker={},o={get length(){log+='L';return 1;},get 0(){log+='G';return 4;}};
Object.defineProperty(o,'method',{get:function(){log+='M';return m;}});
function callback(){log+='C';return function(v,k,obj){log+='B';assert.sameValue(this,marker);assert.sameValue(v,4);assert.sameValue(obj,o);return keepGoing;};}
function receiver(){log+='T';return marker;}function extra(){log+='E';return {valueOf:function(){throw 'unused';}};}
assert.sameValue(o.method(callback(),receiver(),extra()),isEvery);assert.sameValue(log,'MCTELGB');
// CASE: some-call-argument-evaluation-before-length-and-callback
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var log='',marker={},o={get length(){log+='L';return 1;},get 0(){log+='G';return 4;}};
Object.defineProperty(o,'method',{get:function(){log+='M';return m;}});
function callback(){log+='C';return function(v,k,obj){log+='B';assert.sameValue(this,marker);assert.sameValue(v,4);assert.sameValue(obj,o);return keepGoing;};}
function receiver(){log+='T';return marker;}function extra(){log+='E';return {valueOf:function(){throw 'unused';}};}
assert.sameValue(o.method(callback(),receiver(),extra()),isEvery);assert.sameValue(log,'MCTELGB');
// CASE: every-abrupt-extra-argument-prevents-length-access
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var reason={},caught,log='',o={get length(){log+='L';return 1;}};
function extra(){log+='E';throw reason;}
try{m.call(o,function(){throw 'callback';},null,extra());}catch(e){caught=e;}
assert.sameValue(caught,reason);assert.sameValue(log,'E');
// CASE: some-abrupt-extra-argument-prevents-length-access
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var reason={},caught,log='',o={get length(){log+='L';return 1;}};
function extra(){log+='E';throw reason;}
try{m.call(o,function(){throw 'callback';},null,extra());}catch(e){caught=e;}
assert.sameValue(caught,reason);assert.sameValue(log,'E');
// CASE: every-symbol-values-and-canonical-index-keys
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var s=Symbol('value'),o={length:2,0:s,1:9,'01':99,'1.0':99,'-0':99},seen='';
assert.sameValue(m.call(o,function(v,k){if(k===0)assert.sameValue(v,s);else assert.sameValue(v,9);seen+=k;return keepGoing;}),isEvery);assert.sameValue(seen,'01');
// CASE: some-symbol-values-and-canonical-index-keys
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var s=Symbol('value'),o={length:2,0:s,1:9,'01':99,'1.0':99,'-0':99},seen='';
assert.sameValue(m.call(o,function(v,k){if(k===0)assert.sameValue(v,s);else assert.sameValue(v,9);seen+=k;return keepGoing;}),isEvery);assert.sameValue(seen,'01');
// CASE: every-safe-integer-logical-length-early-return
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var lengths=[9007199254740991,9007199254740992,Infinity];
for(var i=0;i<lengths.length;i++){var o={length:lengths[i],0:7},calls=0;assert.sameValue(m.call(o,function(v,k,obj){calls++;assert.sameValue(v,7);assert.sameValue(k,0);assert.sameValue(obj,o);return stop;}),stop);assert.sameValue(calls,1);}
// CASE: some-safe-integer-logical-length-early-return
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var lengths=[9007199254740991,9007199254740992,Infinity];
for(var i=0;i<lengths.length;i++){var o={length:lengths[i],0:7},calls=0;assert.sameValue(m.call(o,function(v,k,obj){calls++;assert.sameValue(v,7);assert.sameValue(k,0);assert.sameValue(obj,o);return stop;}),stop);assert.sameValue(calls,1);}
// CASE: every-sparse-u32-array-short-return-after-holes
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var a=new Array(4294967295),calls=0;a[2]=8;
assert.sameValue(m.call(a,function(v,k,o){calls++;assert.sameValue(v,8);assert.sameValue(k,2);assert.sameValue(o,a);return stop;}),stop);assert.sameValue(calls,1);assert.sameValue(a.length,4294967295);
// CASE: some-sparse-u32-array-short-return-after-holes
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var a=new Array(4294967295),calls=0;a[2]=8;
assert.sameValue(m.call(a,function(v,k,o){calls++;assert.sameValue(v,8);assert.sameValue(k,2);assert.sameValue(o,a);return stop;}),stop);assert.sameValue(calls,1);assert.sameValue(a.length,4294967295);
// CASE: every-readonly-nonextensible-array-is-only-read
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var a=[1,,3],seen='';Object.defineProperty(a,'length',{writable:false});Object.preventExtensions(a);
assert.sameValue(m.call(a,function(v,k){seen+=k;return keepGoing;}),isEvery);assert.sameValue(seen,'02');assert.sameValue(a.length,3);assert.sameValue(Object.isExtensible(a),false);
// CASE: some-readonly-nonextensible-array-is-only-read
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var a=[1,,3],seen='';Object.defineProperty(a,'length',{writable:false});Object.preventExtensions(a);
assert.sameValue(m.call(a,function(v,k){seen+=k;return keepGoing;}),isEvery);assert.sameValue(seen,'02');assert.sameValue(a.length,3);assert.sameValue(Object.isExtensible(a),false);
// CASE: every-does-not-access-constructor-species-or-iterator
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var a=[1,2];Object.defineProperty(a,'constructor',{get:function(){throw 'constructor';}});Object.defineProperty(a,Symbol.iterator,{get:function(){throw 'iterator';}});
assert.sameValue(m.call(a,function(){return keepGoing;}),isEvery);
// CASE: some-does-not-access-constructor-species-or-iterator
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var a=[1,2];Object.defineProperty(a,'constructor',{get:function(){throw 'constructor';}});Object.defineProperty(a,Symbol.iterator,{get:function(){throw 'iterator';}});
assert.sameValue(m.call(a,function(){return keepGoing;}),isEvery);
// CASE: every-retained-alias-after-replacement-and-delete
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
Array.prototype[name]=null;assert.sameValue(m.call([1],function(){return keepGoing;}),isEvery);
delete Array.prototype[name];assert.sameValue(m.call([1],function(){return stop;}),stop);
Object.defineProperty(m,'name',{value:'renamed'});assert.sameValue(m.call([1],function(){return keepGoing;}),isEvery);
// CASE: some-retained-alias-after-replacement-and-delete
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
Array.prototype[name]=null;assert.sameValue(m.call([1],function(){return keepGoing;}),isEvery);
delete Array.prototype[name];assert.sameValue(m.call([1],function(){return stop;}),stop);
Object.defineProperty(m,'name',{value:'renamed'});assert.sameValue(m.call([1],function(){return keepGoing;}),isEvery);
// CASE: every-reentrant-callback-keeps-outer-state
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var o={length:2,0:1,1:2},seen='',nested=false;
assert.sameValue(m.call(o,function(v,k){seen+=v;if(!nested){nested=true;assert.sameValue(m.call([4],function(){return stop;}),stop);o[1]=7;}return keepGoing;}),isEvery);assert.sameValue(seen,'17');
// CASE: some-reentrant-callback-keeps-outer-state
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var o={length:2,0:1,1:2},seen='',nested=false;
assert.sameValue(m.call(o,function(v,k){seen+=v;if(!nested){nested=true;assert.sameValue(m.call([4],function(){return stop;}),stop);o[1]=7;}return keepGoing;}),isEvery);assert.sameValue(seen,'17');
// CASE: every-call-apply-bind-and-empty-null-prototype
var name="every";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=true,keepGoing=true,stop=false;
var o=Object.create(null);o.length=1;o[0]=8;var cb=function(v){assert.sameValue(v,8);return keepGoing;};
assert.sameValue(m.apply(o,[cb]),isEvery);assert.sameValue(m.bind(o,cb)(),isEvery);
assert.sameValue(m.call(Object.create(null),function(){throw 'empty';}),isEvery);
// CASE: some-call-apply-bind-and-empty-null-prototype
var name="some";assert.sameValue(typeof Array.prototype[name],'function','method prerequisite');var m=Array.prototype[name];
assert.sameValue(m.call([1,2],function(v){return v>0;}),true,'positive prerequisite');
assert.sameValue(m.call([0,0],function(v){return v>0;}),false,'negative result prerequisite');
var isEvery=false,keepGoing=false,stop=true;
var o=Object.create(null);o.length=1;o[0]=8;var cb=function(v){assert.sameValue(v,8);return keepGoing;};
assert.sameValue(m.apply(o,[cb]),isEvery);assert.sameValue(m.bind(o,cb)(),isEvery);
assert.sameValue(m.call(Object.create(null),function(){throw 'empty';}),isEvery);
