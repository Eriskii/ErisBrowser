// CASE: function-call-and-new
var f=Function('a','b','return a+b'),g=new Function('a,b','return a*b');
assert.sameValue(f(2,3),5);assert.sameValue(g(2,3),6);assert.sameValue(Function()(),undefined);
assert.sameValue(f===g,false);assert.sameValue(f.name,'anonymous');assert.sameValue(f.length,2);
verifyProperty(f,'length',{value:2,writable:false,enumerable:false,configurable:true});
verifyProperty(f,'name',{value:'anonymous',writable:false,enumerable:false,configurable:true});
// CASE: global-environment
var globalValue=11;let globalLexical=13;
function outer(){var globalValue=21,globalLexical=22;return Function('return globalValue+globalLexical');}
assert.sameValue(outer()(),24);
// CASE: no-anonymous-self-binding
var anonymous=19;assert.sameValue(Function('return anonymous')(),19);
// CASE: own-strictness
assert.sameValue(Function('return this')(),globalThis);
assert.sameValue(Function('"use strict";return this')(),undefined);
assert.throws(SyntaxError,function(){Function('a,a','"use strict";return a');});
assert.sameValue(Function('a,a','return a')(1,2),2);
assert.throws(SyntaxError,function(){Function('eval','"use strict";');});
assert.throws(SyntaxError,function(){Function('arguments','"use strict";');});
// CASE: defaults-rest-and-lexical-conflicts
var f=Function('a=3','...rest','return a+rest.length');assert.sameValue(f(undefined,4,5),5);assert.sameValue(f.length,0);
assert.throws(SyntaxError,function(){Function('a=1','"use strict";');});
assert.throws(SyntaxError,function(){Function('a','let a;');});
assert.throws(SyntaxError,function(){Function('a,a=1','');});
assert.throws(SyntaxError,function(){Function('...a,','');});
assert.throws(SyntaxError,function(){Function('...a,b','');});
// CASE: separated-grammar-boundaries
assert.throws(SyntaxError,function(){Function('/*','*/ ) {');});
assert.throws(SyntaxError,function(){Function('a) { return 1; } //','return 2');});
assert.throws(SyntaxError,function(){Function('a','} ; globalValue=1; {');});
assert.throws(SyntaxError,function(){Function('a','/*');});
assert.throws(SyntaxError,function(){Function('a/*','*/');});
assert.sameValue(Function('a// comment','return a// comment')(17),17);
assert.sameValue(Function('a/* comment */','return a;')(18),18);
// CASE: no-construction-execution
var marker=0,f=Function('marker=7;return 9');assert.sameValue(marker,0);assert.sameValue(f(),9);assert.sameValue(marker,7);
// CASE: conversion-and-prototype-order
var log='',B=(function(){}).bind(null),proto={};
Object.defineProperty(B,'prototype',{get:function(){log+='p';return proto;}});
var a={toString:function(){log+='a';return 'x';}},b={toString:function(){log+='b';return 'return x';}};
var f=Reflect.construct(Function,[a,b],B);assert.sameValue(log,'abp');assert.sameValue(f(23),23);assert.sameValue(Object.getPrototypeOf(f),proto);
log='';assert.throws(SyntaxError,function(){Reflect.construct(Function,[{toString:function(){log+='a';return ')';}},b],B);});assert.sameValue(log,'ab');
// CASE: abrupt-conversion
var marker={},caught,log='';
try{Function({toString:function(){log+='a';throw marker;}},{toString:function(){log+='b';return '';}});}catch(e){caught=e;}
assert.sameValue(caught,marker);assert.sameValue(log,'a');
assert.throws(TypeError,function(){Function(Symbol());});
// CASE: prototype-fallback
var B=(function(){}).bind(null);Object.defineProperty(B,'prototype',{value:1});
var f=Reflect.construct(Function,['return 7'],B);assert.sameValue(Object.getPrototypeOf(f),Function.prototype);assert.sameValue(f(),7);
// CASE: generated-constructor
var F=Function('a','this.a=a;this.target=new.target'),x=new F(4);
assert.sameValue(x.a,4);assert.sameValue(x.target,F);assert.sameValue(Object.getPrototypeOf(x),F.prototype);assert.sameValue(F.prototype.constructor,F);
var G=function(){};var y=Reflect.construct(F,[5],G);assert.sameValue(y.target,G);assert.sameValue(Object.getPrototypeOf(y),G.prototype);
// CASE: target-in-parameter-arrow
var F=Function('get=()=>new.target','return get');assert.sameValue(F()(),undefined);assert.sameValue((new F())(),F);
// CASE: unicode-source-and-escapes
assert.sameValue(Function('π','return π+1')(3),4);assert.sameValue(Function('return "😀"')().length,2);
assert.sameValue(Function('return "\\ud800"')().charCodeAt(0),0xd800);
// CASE: bound-function-constructor
var B=Function.bind(null,'a');var f=new B('return a');assert.sameValue(f(37),37);assert.sameValue(Object.getPrototypeOf(f),Function.prototype);
// CASE: generated-property-order
var f=Function('return 1'),keys=Object.getOwnPropertyNames(f);assert.sameValue(keys[0],'length');assert.sameValue(keys[1],'name');assert.sameValue(keys[2],'prototype');
