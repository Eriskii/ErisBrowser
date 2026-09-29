// CASE: rightmost-and-argument-presence
assert.sameValue(typeof Array.prototype.lastIndexOf,'function');
assert.sameValue(['x','y','x'].lastIndexOf('x'),2);
assert.sameValue(['x','y','x'].lastIndexOf('x',undefined),0);
assert.sameValue(['x','y','x'].lastIndexOf('x',NaN),0);
assert.sameValue(['x','y','x'].lastIndexOf('x',null),0);
assert.sameValue(['x','y','x'].lastIndexOf('x',Infinity),2);
assert.sameValue(['x','y','x'].lastIndexOf('x',-Infinity),-1);
assert.sameValue(['x','y','x'].lastIndexOf('x',-1.5),2);
assert.sameValue(['x','y','x'].lastIndexOf('x',-3),0);
assert.sameValue(['x','y','x'].lastIndexOf('x',-4),-1);
// CASE: holes-undefined-and-inheritance
assert.sameValue(typeof Array.prototype.lastIndexOf,'function');
assert.sameValue([,,].lastIndexOf(undefined),-1);
assert.sameValue([,undefined].lastIndexOf(undefined),1);
var p={1:'hit'},o=Object.create(p);o.length=3;
assert.sameValue(Array.prototype.lastIndexOf.call(o,'hit'),1);
assert.sameValue(Array.prototype.lastIndexOf.call(o,undefined),-1);
Array.prototype[0]='inherited';
assert.sameValue([,].lastIndexOf('inherited'),0);
// CASE: strict-equality-without-conversion
assert.sameValue(typeof Array.prototype.lastIndexOf,'function');
var a={},b={},s=Symbol('x'),t=Symbol('x');
a.valueOf=function(){throw 'coercion';};a.toString=function(){throw 'coercion';};
assert.sameValue([a,b,a].lastIndexOf(a),2);
assert.sameValue([a,b].lastIndexOf({}),-1);
assert.sameValue([NaN].lastIndexOf(NaN),-1);
assert.sameValue([-0].lastIndexOf(0),0);
assert.sameValue([1].lastIndexOf('1'),-1);
assert.sameValue([s,t].lastIndexOf(s),0);
// CASE: primitive-boxing-and-utf16
assert.sameValue(typeof Array.prototype.lastIndexOf,'function');
var f=Array.prototype.lastIndexOf;
assert.sameValue(f.call('a\ud800\udfffa','a'),3);
assert.sameValue(f.call('a\ud800\udfffa','\udfff'),2);
assert.sameValue(f.call(true,undefined),-1);
assert.sameValue(f.call(7,undefined),-1);
assert.sameValue(f.call(Symbol('x'),undefined),-1);
// CASE: length-conversion-and-from-index-order
assert.sameValue(typeof Array.prototype.lastIndexOf,'function');
var log='',o={0:'hit',2:'hit'};
Object.defineProperty(o,'length',{get:function(){log+='l';return {valueOf:function(){log+='n';return 3;}};}});
var p={valueOf:function(){log+='p';return 1;}};
assert.sameValue(Array.prototype.lastIndexOf.call(o,'hit',p),0);
assert.sameValue(log,'lnp');
// CASE: empty-length-skips-from-index
assert.sameValue(typeof Array.prototype.lastIndexOf,'function');
var p={valueOf:function(){throw 'unexpected position';}};
assert.sameValue(Array.prototype.lastIndexOf.call({length:0},'x',p),-1);
assert.sameValue(Array.prototype.lastIndexOf.call({length:-4},'x',p),-1);
assert.sameValue(Array.prototype.lastIndexOf.call({length:NaN},'x',Symbol()),-1);
// CASE: saved-length-with-live-properties
assert.sameValue(typeof Array.prototype.lastIndexOf,'function');
var log='',o={length:3,1:'hit'};
Object.defineProperty(o,'2',{get:function(){log+='2';delete o[1];o[0]='hit';o.length=0;return 'miss';}});
assert.sameValue(Array.prototype.lastIndexOf.call(o,'hit'),0);
assert.sameValue(log,'2');
// CASE: position-conversion-mutates-receiver
assert.sameValue(typeof Array.prototype.lastIndexOf,'function');
var o={length:3,2:'old'},p={valueOf:function(){o.length=1;o[2]='new';return Infinity;}};
assert.sameValue(Array.prototype.lastIndexOf.call(o,'new',p),2);
// CASE: getters-receive-original-object
assert.sameValue(typeof Array.prototype.lastIndexOf,'function');
var p={},o=Object.create(p);o.length=2;var seen;
Object.defineProperty(p,'1',{get:function(){seen=this;return 'x';}});
assert.sameValue(Array.prototype.lastIndexOf.call(o,'x'),1);
assert.sameValue(seen,o);
// CASE: abrupt-order-and-identity
assert.sameValue(typeof Array.prototype.lastIndexOf,'function');
var marker={},caught,visits=0,o={};
Object.defineProperty(o,'length',{get:function(){throw marker;}});
var p={valueOf:function(){visits++;return 0;}};
try{Array.prototype.lastIndexOf.call(o,'x',p);}catch(e){caught=e;}
assert.sameValue(caught,marker);assert.sameValue(visits,0);
assert.throws(TypeError,function(){Array.prototype.lastIndexOf.call(null,'x',p);});
assert.sameValue(visits,0);
var r={length:1};Object.defineProperty(r,'0',{get:function(){throw marker;}});
caught=undefined;try{Array.prototype.lastIndexOf.call(r,'x');}catch(e){caught=e;}
assert.sameValue(caught,marker);
// CASE: full-safe-integer-length
assert.sameValue(typeof Array.prototype.lastIndexOf,'function');
var o={length:Infinity};o['9007199254740990']='x';
assert.sameValue(Array.prototype.lastIndexOf.call(o,'x'),9007199254740990);
assert.sameValue(Array.prototype.lastIndexOf.call(o,'x',-1),9007199254740990);
assert.sameValue(Array.prototype.lastIndexOf.call(o,'x',-Infinity),-1);
// CASE: descriptors-and-construction
assert.sameValue(typeof Array.prototype.lastIndexOf,'function');
var m=Array.prototype.lastIndexOf;
verifyProperty(Array.prototype,'lastIndexOf',{value:m,writable:true,enumerable:false,configurable:true},{restore:true});
verifyProperty(m,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});
verifyProperty(m,'name',{value:'lastIndexOf',writable:false,enumerable:false,configurable:true},{restore:true});
assert.sameValue(m.hasOwnProperty('prototype'),false);
assert.throws(TypeError,function(){new m();});
