// Frozen local semantic cases; each is run independently in sloppy and strict mode.
// CASE: descriptors
var split=RegExp.prototype[Symbol.split];
verifyProperty(RegExp.prototype,Symbol.split,{value:split,writable:true,enumerable:false,configurable:true});
verifyProperty(split,'name',{value:'[Symbol.split]',writable:false,enumerable:false,configurable:true});
verifyProperty(split,'length',{value:2,writable:false,enumerable:false,configurable:true});
assert.sameValue(Object.prototype.hasOwnProperty.call(split,'prototype'),false);
assert.throws(TypeError,function(){new split('x');});
// CASE: species-getter
var d=Object.getOwnPropertyDescriptor(RegExp,Symbol.species);
assert.sameValue(d.enumerable,false);assert.sameValue(d.configurable,true);assert.sameValue(d.set,undefined);
verifyProperty(d.get,'name',{value:'get [Symbol.species]',writable:false,enumerable:false,configurable:true});
verifyProperty(d.get,'length',{value:0,writable:false,enumerable:false,configurable:true});
assert.sameValue(d.get.call(null),null);assert.sameValue(d.get.call(undefined),undefined);assert.sameValue(d.get.call(12),12);assert.sameValue(RegExp[Symbol.species],RegExp);
// CASE: order-raw-species-arguments
var log='',rx={},ctor={},str={toString:function(){log+='s';return 'ab';}},lim={valueOf:function(){log+='l';return 0;}};
Object.defineProperty(rx,'constructor',{get:function(){log+='c';return ctor;}});
Object.defineProperty(ctor,Symbol.species,{get:function(){log+='p';return function(r,f){log+='n';assert.sameValue(r,rx);assert.sameValue(f,'iy');return {exec:function(){throw 1;}};};}});
Object.defineProperty(rx,'flags',{get:function(){log+='f';return {toString:function(){log+='t';return 'i';}};}});
assert.sameValue(RegExp.prototype[Symbol.split].call(rx,str,lim).length,0);assert.sameValue(log,'scpftnl');
// CASE: receiver-before-string
var called=false,s={toString:function(){called=true;throw 1;}};
assert.throws(TypeError,function(){RegExp.prototype[Symbol.split].call(null,s);});
assert.throws(TypeError,function(){RegExp.prototype[Symbol.split].call('x',s);});assert.sameValue(called,false);
// CASE: string-before-constructor
var marker={},caught,rx={};Object.defineProperty(rx,'constructor',{get:function(){throw 1;}});
try{RegExp.prototype[Symbol.split].call(rx,{toString:function(){throw marker;}});}catch(e){caught=e;}assert.sameValue(caught,marker);
// CASE: invalid-species-before-flags
var rx={constructor:{}},called=false;rx.constructor[Symbol.species]=()=>{};
Object.defineProperty(rx,'flags',{get:function(){called=true;throw 1;}});
assert.throws(TypeError,function(){RegExp.prototype[Symbol.split].call(rx,'a');});assert.sameValue(called,false);
rx.constructor=null;assert.throws(TypeError,function(){RegExp.prototype[Symbol.split].call(rx,'a');});assert.sameValue(called,false);
// CASE: intrinsic-default-and-lastindex
var original=RegExp,rx=/(b)/g;rx.lastIndex=7;rx.constructor=undefined;RegExp=function(){throw 1;};
var a=original.prototype[Symbol.split].call(rx,'abc');assert.sameValue(a.join('|'),'a|b|c');assert.sameValue(rx.lastIndex,7);
rx.constructor={};rx.constructor[Symbol.species]=null;assert.sameValue('abc'.split(rx).join('|'),'a|b|c');
// CASE: sticky-flag-preserved
var rx={flags:'gy',constructor:{}};rx.constructor[Symbol.species]=function(r,f){assert.sameValue(f,'gy');return {exec:function(){return null;}};};
assert.sameValue(RegExp.prototype[Symbol.split].call(rx,'abc').join('|'),'abc');
// CASE: empty-input-does-not-set-index
var rx={flags:'',constructor:{}},count=0,splitter={exec:function(s){count++;assert.sameValue(s,'');return null;}};
Object.defineProperty(splitter,'lastIndex',{set:function(){throw 1;}});rx.constructor[Symbol.species]=function(){return splitter;};
var a=RegExp.prototype[Symbol.split].call(rx,'');assert.sameValue(a.length,1);assert.sameValue(a[0],'');assert.sameValue(count,1);
splitter.exec=function(){return {};};assert.sameValue(RegExp.prototype[Symbol.split].call(rx,'').length,0);
// CASE: captures-raw-and-limit-before-length
var rx={flags:'',constructor:{}},marker={},lengthCalls=0,captureCalls=0,result={};
Object.defineProperty(result,'length',{get:function(){lengthCalls++;return Infinity;}});
Object.defineProperty(result,'1',{get:function(){captureCalls++;return marker;}});
rx.constructor[Symbol.species]=function(){return {exec:function(){this.lastIndex=1;return result;}};};
var a=RegExp.prototype[Symbol.split].call(rx,'ab',1);assert.sameValue(a.length,1);assert.sameValue(lengthCalls,0);
a=RegExp.prototype[Symbol.split].call(rx,'ab',2);assert.sameValue(a[1],marker);assert.sameValue(lengthCalls,1);assert.sameValue(captureCalls,1);
// CASE: lastindex-clamped-to-input
var rx={flags:'',constructor:{}},coerced=0;rx.constructor[Symbol.species]=function(){return {exec:function(){this.lastIndex={valueOf:function(){coerced++;return Infinity;}};return {length:0};}};};
var a=RegExp.prototype[Symbol.split].call(rx,'abc');assert.sameValue(a.length,2);assert.sameValue(a.join('|'),'|');assert.sameValue(coerced,1);
// CASE: exec-invalid-and-set-index-errors
var rx={flags:'',constructor:{}},called=false;rx.constructor[Symbol.species]=function(){return {exec:function(){return 1;}};};
assert.throws(TypeError,function(){RegExp.prototype[Symbol.split].call(rx,'a');});
rx.constructor[Symbol.species]=function(){var s={exec:function(){called=true;return null;}};Object.defineProperty(s,'lastIndex',{value:0,writable:false});return s;};
assert.throws(TypeError,function(){RegExp.prototype[Symbol.split].call(rx,'a');});assert.sameValue(called,false);
// CASE: disabled-regexp-split-is-literal
var rx=/x/;rx[Symbol.split]=null;assert.sameValue('axb'.split(rx).join('|'),'axb');
rx[Symbol.split]=undefined;assert.sameValue('a/x/b'.split(rx).join('|'),'a|b');
// CASE: unicode-and-unicode-sets-advancement
function positions(flags,empty){var log='',rx={flags:flags,constructor:{}};rx.constructor[Symbol.species]=function(){return {exec:function(){log+=this.lastIndex;this.lastIndex=0;return empty?{}:null;}};};var a=RegExp.prototype[Symbol.split].call(rx,'\ud800\udc00x');assert.sameValue(a[0],'\ud800\udc00x');return log;}
assert.sameValue(positions('u',false),'02');assert.sameValue(positions('v',false),'02');assert.sameValue(positions('',false),'012');assert.sameValue(positions('u',true),'02');
// CASE: captures-ignore-array-prototype-setter
var called=false;Object.defineProperty(Array.prototype,'1',{set:function(){called=true;},configurable:true});
var a='abc'.split(/(b)/);assert.sameValue(a[1],'b');assert.sameValue(called,false);assert.sameValue(a.length,3);
// CASE: exact-utf16-and-empty-matches
var a='\ud800a\udfff'.split(/a/);assert.sameValue(a[0].charCodeAt(0),0xd800);assert.sameValue(a[1].charCodeAt(0),0xdfff);
assert.sameValue('ab'.split(/(?:)/).join('|'),'a|b');assert.sameValue(''.split(/(?:)/).length,0);assert.sameValue(''.split(/a/).length,1);
// CASE: abrupt-result-length-and-capture
var marker={},caught,rx={flags:'',constructor:{}},r={};Object.defineProperty(r,'length',{get:function(){throw marker;}});
rx.constructor[Symbol.species]=function(){return {exec:function(){this.lastIndex=1;return r;}};};
try{RegExp.prototype[Symbol.split].call(rx,'ab');}catch(e){caught=e;}assert.sameValue(caught,marker);
r={length:2};Object.defineProperty(r,'1',{get:function(){throw marker;}});caught=undefined;
try{RegExp.prototype[Symbol.split].call(rx,'ab');}catch(e){caught=e;}assert.sameValue(caught,marker);
