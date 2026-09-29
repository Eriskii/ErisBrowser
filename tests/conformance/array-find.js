// CASE: find-intrinsic-metadata-and-nonconstructability
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
verifyProperty(Array.prototype,name,{value:m,writable:true,enumerable:false,configurable:true},{restore:true});
verifyProperty(m,'name',{value:name,writable:false,enumerable:false,configurable:true},{restore:true});
verifyProperty(m,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});
assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);
assert.throws(TypeError,function(){new m(function(){return true;});});
assert.notSameValue(Array.prototype.find,Array.prototype.findIndex);assert.notSameValue(Array.prototype.findLast,Array.prototype.findLastIndex);
assert.notSameValue(Array.prototype.find,Array.prototype.findLast);assert.notSameValue(Array.prototype.findIndex,Array.prototype.findLastIndex);
// CASE: findIndex-intrinsic-metadata-and-nonconstructability
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
verifyProperty(Array.prototype,name,{value:m,writable:true,enumerable:false,configurable:true},{restore:true});
verifyProperty(m,'name',{value:name,writable:false,enumerable:false,configurable:true},{restore:true});
verifyProperty(m,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});
assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);
assert.throws(TypeError,function(){new m(function(){return true;});});
assert.notSameValue(Array.prototype.find,Array.prototype.findIndex);assert.notSameValue(Array.prototype.findLast,Array.prototype.findLastIndex);
assert.notSameValue(Array.prototype.find,Array.prototype.findLast);assert.notSameValue(Array.prototype.findIndex,Array.prototype.findLastIndex);
// CASE: findLast-intrinsic-metadata-and-nonconstructability
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
verifyProperty(Array.prototype,name,{value:m,writable:true,enumerable:false,configurable:true},{restore:true});
verifyProperty(m,'name',{value:name,writable:false,enumerable:false,configurable:true},{restore:true});
verifyProperty(m,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});
assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);
assert.throws(TypeError,function(){new m(function(){return true;});});
assert.notSameValue(Array.prototype.find,Array.prototype.findIndex);assert.notSameValue(Array.prototype.findLast,Array.prototype.findLastIndex);
assert.notSameValue(Array.prototype.find,Array.prototype.findLast);assert.notSameValue(Array.prototype.findIndex,Array.prototype.findLastIndex);
// CASE: findLastIndex-intrinsic-metadata-and-nonconstructability
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
verifyProperty(Array.prototype,name,{value:m,writable:true,enumerable:false,configurable:true},{restore:true});
verifyProperty(m,'name',{value:name,writable:false,enumerable:false,configurable:true},{restore:true});
verifyProperty(m,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});
assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);
assert.throws(TypeError,function(){new m(function(){return true;});});
assert.notSameValue(Array.prototype.find,Array.prototype.findIndex);assert.notSameValue(Array.prototype.findLast,Array.prototype.findLastIndex);
assert.notSameValue(Array.prototype.find,Array.prototype.findLast);assert.notSameValue(Array.prototype.findIndex,Array.prototype.findLastIndex);
// CASE: find-saved-alias-call-apply-bind-and-property-recreation
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var token={},a=[token],predicate=function(){return true;},saved=m;
assert.sameValue(m.call(a,predicate),result(0,token));assert.sameValue(m.apply(a,[predicate]),result(0,token));
var bound=m.bind(a,predicate);assert.sameValue(bound(),result(0,token));
Array.prototype[name]=7;assert.sameValue(saved.call(a,predicate),result(0,token));
delete Array.prototype[name];assert.sameValue(saved.call(a,predicate),result(0,token));
Object.defineProperty(Array.prototype,name,{value:saved,writable:true,configurable:true});assert.sameValue(a[name](predicate),result(0,token));
// CASE: findIndex-saved-alias-call-apply-bind-and-property-recreation
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var token={},a=[token],predicate=function(){return true;},saved=m;
assert.sameValue(m.call(a,predicate),result(0,token));assert.sameValue(m.apply(a,[predicate]),result(0,token));
var bound=m.bind(a,predicate);assert.sameValue(bound(),result(0,token));
Array.prototype[name]=7;assert.sameValue(saved.call(a,predicate),result(0,token));
delete Array.prototype[name];assert.sameValue(saved.call(a,predicate),result(0,token));
Object.defineProperty(Array.prototype,name,{value:saved,writable:true,configurable:true});assert.sameValue(a[name](predicate),result(0,token));
// CASE: findLast-saved-alias-call-apply-bind-and-property-recreation
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var token={},a=[token],predicate=function(){return true;},saved=m;
assert.sameValue(m.call(a,predicate),result(0,token));assert.sameValue(m.apply(a,[predicate]),result(0,token));
var bound=m.bind(a,predicate);assert.sameValue(bound(),result(0,token));
Array.prototype[name]=7;assert.sameValue(saved.call(a,predicate),result(0,token));
delete Array.prototype[name];assert.sameValue(saved.call(a,predicate),result(0,token));
Object.defineProperty(Array.prototype,name,{value:saved,writable:true,configurable:true});assert.sameValue(a[name](predicate),result(0,token));
// CASE: findLastIndex-saved-alias-call-apply-bind-and-property-recreation
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var token={},a=[token],predicate=function(){return true;},saved=m;
assert.sameValue(m.call(a,predicate),result(0,token));assert.sameValue(m.apply(a,[predicate]),result(0,token));
var bound=m.bind(a,predicate);assert.sameValue(bound(),result(0,token));
Array.prototype[name]=7;assert.sameValue(saved.call(a,predicate),result(0,token));
delete Array.prototype[name];assert.sameValue(saved.call(a,predicate),result(0,token));
Object.defineProperty(Array.prototype,name,{value:saved,writable:true,configurable:true});assert.sameValue(a[name](predicate),result(0,token));
// CASE: find-method-and-author-argument-evaluation-before-length
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var log='',thisToken={},token={},o={get length(){log+='L';return 1;},get 0(){log+='G';return token;},get method(){log+='M';return m;}};
function predicateArgument(){log+='P';return function(v,k,obj){log+='C';assert.sameValue(this,thisToken);assert.sameValue(obj,o);return true;};}
function receiverArgument(){log+='T';return thisToken;}function extra(){log+='E';return {get valueOf(){throw 'ignored';}};}
assert.sameValue(o.method(predicateArgument(),receiverArgument(),extra()),result(0,token));assert.sameValue(log,'MPTELGC');
// CASE: findIndex-method-and-author-argument-evaluation-before-length
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var log='',thisToken={},token={},o={get length(){log+='L';return 1;},get 0(){log+='G';return token;},get method(){log+='M';return m;}};
function predicateArgument(){log+='P';return function(v,k,obj){log+='C';assert.sameValue(this,thisToken);assert.sameValue(obj,o);return true;};}
function receiverArgument(){log+='T';return thisToken;}function extra(){log+='E';return {get valueOf(){throw 'ignored';}};}
assert.sameValue(o.method(predicateArgument(),receiverArgument(),extra()),result(0,token));assert.sameValue(log,'MPTELGC');
// CASE: findLast-method-and-author-argument-evaluation-before-length
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var log='',thisToken={},token={},o={get length(){log+='L';return 1;},get 0(){log+='G';return token;},get method(){log+='M';return m;}};
function predicateArgument(){log+='P';return function(v,k,obj){log+='C';assert.sameValue(this,thisToken);assert.sameValue(obj,o);return true;};}
function receiverArgument(){log+='T';return thisToken;}function extra(){log+='E';return {get valueOf(){throw 'ignored';}};}
assert.sameValue(o.method(predicateArgument(),receiverArgument(),extra()),result(0,token));assert.sameValue(log,'MPTELGC');
// CASE: findLastIndex-method-and-author-argument-evaluation-before-length
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var log='',thisToken={},token={},o={get length(){log+='L';return 1;},get 0(){log+='G';return token;},get method(){log+='M';return m;}};
function predicateArgument(){log+='P';return function(v,k,obj){log+='C';assert.sameValue(this,thisToken);assert.sameValue(obj,o);return true;};}
function receiverArgument(){log+='T';return thisToken;}function extra(){log+='E';return {get valueOf(){throw 'ignored';}};}
assert.sameValue(o.method(predicateArgument(),receiverArgument(),extra()),result(0,token));assert.sameValue(log,'MPTELGC');
// CASE: find-abrupt-extra-argument-prevents-internal-reads
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reason={},caught,reads=0,o={get length(){reads++;return 1;}};
function extra(){throw reason;}try{m.call(o,function(){throw 'callback';},null,extra());}catch(e){caught=e;}
assert.sameValue(caught,reason);assert.sameValue(reads,0);
// CASE: findIndex-abrupt-extra-argument-prevents-internal-reads
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reason={},caught,reads=0,o={get length(){reads++;return 1;}};
function extra(){throw reason;}try{m.call(o,function(){throw 'callback';},null,extra());}catch(e){caught=e;}
assert.sameValue(caught,reason);assert.sameValue(reads,0);
// CASE: findLast-abrupt-extra-argument-prevents-internal-reads
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reason={},caught,reads=0,o={get length(){reads++;return 1;}};
function extra(){throw reason;}try{m.call(o,function(){throw 'callback';},null,extra());}catch(e){caught=e;}
assert.sameValue(caught,reason);assert.sameValue(reads,0);
// CASE: findLastIndex-abrupt-extra-argument-prevents-internal-reads
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reason={},caught,reads=0,o={get length(){reads++;return 1;}};
function extra(){throw reason;}try{m.call(o,function(){throw 'callback';},null,extra());}catch(e){caught=e;}
assert.sameValue(caught,reason);assert.sameValue(reads,0);
// CASE: find-null-and-undefined-receivers-reject-before-predicate-inspection
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reads=0,bad={get call(){reads++;throw 'call';},get valueOf(){reads++;throw 'valueOf';},get toString(){reads++;throw 'toString';}};
assert.throws(TypeError,function(){m.call(null,bad);});assert.throws(TypeError,function(){m.call(undefined,bad);});
assert.throws(TypeError,function(){m.call(null,function(){throw 'callback';});});assert.sameValue(reads,0);
// CASE: findIndex-null-and-undefined-receivers-reject-before-predicate-inspection
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reads=0,bad={get call(){reads++;throw 'call';},get valueOf(){reads++;throw 'valueOf';},get toString(){reads++;throw 'toString';}};
assert.throws(TypeError,function(){m.call(null,bad);});assert.throws(TypeError,function(){m.call(undefined,bad);});
assert.throws(TypeError,function(){m.call(null,function(){throw 'callback';});});assert.sameValue(reads,0);
// CASE: findLast-null-and-undefined-receivers-reject-before-predicate-inspection
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reads=0,bad={get call(){reads++;throw 'call';},get valueOf(){reads++;throw 'valueOf';},get toString(){reads++;throw 'toString';}};
assert.throws(TypeError,function(){m.call(null,bad);});assert.throws(TypeError,function(){m.call(undefined,bad);});
assert.throws(TypeError,function(){m.call(null,function(){throw 'callback';});});assert.sameValue(reads,0);
// CASE: findLastIndex-null-and-undefined-receivers-reject-before-predicate-inspection
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reads=0,bad={get call(){reads++;throw 'call';},get valueOf(){reads++;throw 'valueOf';},get toString(){reads++;throw 'toString';}};
assert.throws(TypeError,function(){m.call(null,bad);});assert.throws(TypeError,function(){m.call(undefined,bad);});
assert.throws(TypeError,function(){m.call(null,function(){throw 'callback';});});assert.sameValue(reads,0);
// CASE: find-length-getter-abrupt-precedes-callable-validation
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reason={},caught,reads=0,o={get length(){reads++;throw reason;}};
try{m.call(o,0);}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(reads,1);
// CASE: findIndex-length-getter-abrupt-precedes-callable-validation
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reason={},caught,reads=0,o={get length(){reads++;throw reason;}};
try{m.call(o,0);}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(reads,1);
// CASE: findLast-length-getter-abrupt-precedes-callable-validation
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reason={},caught,reads=0,o={get length(){reads++;throw reason;}};
try{m.call(o,0);}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(reads,1);
// CASE: findLastIndex-length-getter-abrupt-precedes-callable-validation
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reason={},caught,reads=0,o={get length(){reads++;throw reason;}};
try{m.call(o,0);}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(reads,1);
// CASE: find-length-conversion-before-invalid-predicate-and-index-get
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var log='',o={get length(){log+='L';return {valueOf:function(){log+='V';return 2;},toString:function(){throw 'string';}};}};
Object.defineProperty(o,'0',{get:function(){log+='G';throw 'index';}});Object.defineProperty(o,'1',{get:function(){log+='G';throw 'index';}});
assert.throws(TypeError,function(){m.call(o,{});});assert.sameValue(log,'LV');
// CASE: findIndex-length-conversion-before-invalid-predicate-and-index-get
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var log='',o={get length(){log+='L';return {valueOf:function(){log+='V';return 2;},toString:function(){throw 'string';}};}};
Object.defineProperty(o,'0',{get:function(){log+='G';throw 'index';}});Object.defineProperty(o,'1',{get:function(){log+='G';throw 'index';}});
assert.throws(TypeError,function(){m.call(o,{});});assert.sameValue(log,'LV');
// CASE: findLast-length-conversion-before-invalid-predicate-and-index-get
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var log='',o={get length(){log+='L';return {valueOf:function(){log+='V';return 2;},toString:function(){throw 'string';}};}};
Object.defineProperty(o,'0',{get:function(){log+='G';throw 'index';}});Object.defineProperty(o,'1',{get:function(){log+='G';throw 'index';}});
assert.throws(TypeError,function(){m.call(o,{});});assert.sameValue(log,'LV');
// CASE: findLastIndex-length-conversion-before-invalid-predicate-and-index-get
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var log='',o={get length(){log+='L';return {valueOf:function(){log+='V';return 2;},toString:function(){throw 'string';}};}};
Object.defineProperty(o,'0',{get:function(){log+='G';throw 'index';}});Object.defineProperty(o,'1',{get:function(){log+='G';throw 'index';}});
assert.throws(TypeError,function(){m.call(o,{});});assert.sameValue(log,'LV');
// CASE: find-length-symbol-toprimitive-number-hint-and-getter-receiver
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var log='',lengthValue={};lengthValue[Symbol.toPrimitive]=function(hint){assert.sameValue(this,lengthValue);assert.sameValue(hint,'number');log+='P';return 2;};
lengthValue.valueOf=function(){throw 'valueOf';};lengthValue.toString=function(){throw 'toString';};
var o={0:'a',1:'b',get length(){assert.sameValue(this,o);log+='L';return lengthValue;}},visited='';
assert.sameValue(m.call(o,function(v,k,obj){assert.sameValue(obj,o);visited+=k;return false;}),absent);
assert.sameValue(log,'LP');assert.sameValue(visited,reverse?'10':'01');
// CASE: findIndex-length-symbol-toprimitive-number-hint-and-getter-receiver
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var log='',lengthValue={};lengthValue[Symbol.toPrimitive]=function(hint){assert.sameValue(this,lengthValue);assert.sameValue(hint,'number');log+='P';return 2;};
lengthValue.valueOf=function(){throw 'valueOf';};lengthValue.toString=function(){throw 'toString';};
var o={0:'a',1:'b',get length(){assert.sameValue(this,o);log+='L';return lengthValue;}},visited='';
assert.sameValue(m.call(o,function(v,k,obj){assert.sameValue(obj,o);visited+=k;return false;}),absent);
assert.sameValue(log,'LP');assert.sameValue(visited,reverse?'10':'01');
// CASE: findLast-length-symbol-toprimitive-number-hint-and-getter-receiver
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var log='',lengthValue={};lengthValue[Symbol.toPrimitive]=function(hint){assert.sameValue(this,lengthValue);assert.sameValue(hint,'number');log+='P';return 2;};
lengthValue.valueOf=function(){throw 'valueOf';};lengthValue.toString=function(){throw 'toString';};
var o={0:'a',1:'b',get length(){assert.sameValue(this,o);log+='L';return lengthValue;}},visited='';
assert.sameValue(m.call(o,function(v,k,obj){assert.sameValue(obj,o);visited+=k;return false;}),absent);
assert.sameValue(log,'LP');assert.sameValue(visited,reverse?'10':'01');
// CASE: findLastIndex-length-symbol-toprimitive-number-hint-and-getter-receiver
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var log='',lengthValue={};lengthValue[Symbol.toPrimitive]=function(hint){assert.sameValue(this,lengthValue);assert.sameValue(hint,'number');log+='P';return 2;};
lengthValue.valueOf=function(){throw 'valueOf';};lengthValue.toString=function(){throw 'toString';};
var o={0:'a',1:'b',get length(){assert.sameValue(this,o);log+='L';return lengthValue;}},visited='';
assert.sameValue(m.call(o,function(v,k,obj){assert.sameValue(obj,o);visited+=k;return false;}),absent);
assert.sameValue(log,'LP');assert.sameValue(visited,reverse?'10':'01');
// CASE: find-length-ordinary-conversion-fallback-is-ordered
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var log='',o={length:{valueOf:function(){log+='V';return {};},toString:function(){log+='S';return '2.9';}},0:4,1:5,2:6},calls=0;
assert.sameValue(m.call(o,function(v,k){assert.sameValue(k,reverse?1-calls:calls);calls++;return false;}),absent);
assert.sameValue(log,'VS');assert.sameValue(calls,2);
// CASE: findIndex-length-ordinary-conversion-fallback-is-ordered
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var log='',o={length:{valueOf:function(){log+='V';return {};},toString:function(){log+='S';return '2.9';}},0:4,1:5,2:6},calls=0;
assert.sameValue(m.call(o,function(v,k){assert.sameValue(k,reverse?1-calls:calls);calls++;return false;}),absent);
assert.sameValue(log,'VS');assert.sameValue(calls,2);
// CASE: findLast-length-ordinary-conversion-fallback-is-ordered
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var log='',o={length:{valueOf:function(){log+='V';return {};},toString:function(){log+='S';return '2.9';}},0:4,1:5,2:6},calls=0;
assert.sameValue(m.call(o,function(v,k){assert.sameValue(k,reverse?1-calls:calls);calls++;return false;}),absent);
assert.sameValue(log,'VS');assert.sameValue(calls,2);
// CASE: findLastIndex-length-ordinary-conversion-fallback-is-ordered
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var log='',o={length:{valueOf:function(){log+='V';return {};},toString:function(){log+='S';return '2.9';}},0:4,1:5,2:6},calls=0;
assert.sameValue(m.call(o,function(v,k){assert.sameValue(k,reverse?1-calls:calls);calls++;return false;}),absent);
assert.sameValue(log,'VS');assert.sameValue(calls,2);
// CASE: find-length-coercion-error-identity-before-noncallable-predicate
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reason=Symbol('length'),caught,o={length:{valueOf:function(){throw reason;}}};
try{m.call(o,null);}catch(e){caught=e;}assert.sameValue(caught,reason);
assert.throws(TypeError,function(){m.call({length:Symbol('invalid')},function(){throw 'callback';});});
assert.throws(TypeError,function(){m.call({length:{valueOf:function(){return {};},toString:function(){return {};}}},0);});
// CASE: findIndex-length-coercion-error-identity-before-noncallable-predicate
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reason=Symbol('length'),caught,o={length:{valueOf:function(){throw reason;}}};
try{m.call(o,null);}catch(e){caught=e;}assert.sameValue(caught,reason);
assert.throws(TypeError,function(){m.call({length:Symbol('invalid')},function(){throw 'callback';});});
assert.throws(TypeError,function(){m.call({length:{valueOf:function(){return {};},toString:function(){return {};}}},0);});
// CASE: findLast-length-coercion-error-identity-before-noncallable-predicate
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reason=Symbol('length'),caught,o={length:{valueOf:function(){throw reason;}}};
try{m.call(o,null);}catch(e){caught=e;}assert.sameValue(caught,reason);
assert.throws(TypeError,function(){m.call({length:Symbol('invalid')},function(){throw 'callback';});});
assert.throws(TypeError,function(){m.call({length:{valueOf:function(){return {};},toString:function(){return {};}}},0);});
// CASE: findLastIndex-length-coercion-error-identity-before-noncallable-predicate
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reason=Symbol('length'),caught,o={length:{valueOf:function(){throw reason;}}};
try{m.call(o,null);}catch(e){caught=e;}assert.sameValue(caught,reason);
assert.throws(TypeError,function(){m.call({length:Symbol('invalid')},function(){throw 'callback';});});
assert.throws(TypeError,function(){m.call({length:{valueOf:function(){return {};},toString:function(){return {};}}},0);});
// CASE: find-zero-negative-nan-and-missing-length-do-not-visit
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var lengths=[undefined,null,false,0,-0,-1,-0.9,-Infinity,NaN,'','not a number'],calls=0;
for(var i=0;i<lengths.length;i++){assert.sameValue(m.call({length:lengths[i],0:1},function(){calls++;return true;}),absent);}
assert.sameValue(m.call({},function(){calls++;return true;}),absent);assert.sameValue(calls,0);
// CASE: findIndex-zero-negative-nan-and-missing-length-do-not-visit
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var lengths=[undefined,null,false,0,-0,-1,-0.9,-Infinity,NaN,'','not a number'],calls=0;
for(var i=0;i<lengths.length;i++){assert.sameValue(m.call({length:lengths[i],0:1},function(){calls++;return true;}),absent);}
assert.sameValue(m.call({},function(){calls++;return true;}),absent);assert.sameValue(calls,0);
// CASE: findLast-zero-negative-nan-and-missing-length-do-not-visit
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var lengths=[undefined,null,false,0,-0,-1,-0.9,-Infinity,NaN,'','not a number'],calls=0;
for(var i=0;i<lengths.length;i++){assert.sameValue(m.call({length:lengths[i],0:1},function(){calls++;return true;}),absent);}
assert.sameValue(m.call({},function(){calls++;return true;}),absent);assert.sameValue(calls,0);
// CASE: findLastIndex-zero-negative-nan-and-missing-length-do-not-visit
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var lengths=[undefined,null,false,0,-0,-1,-0.9,-Infinity,NaN,'','not a number'],calls=0;
for(var i=0;i<lengths.length;i++){assert.sameValue(m.call({length:lengths[i],0:1},function(){calls++;return true;}),absent);}
assert.sameValue(m.call({},function(){calls++;return true;}),absent);assert.sameValue(calls,0);
// CASE: find-fractional-string-and-boolean-lengths
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var lengths=[3.9,'3.9',true],counts=[3,3,1];
for(var i=0;i<lengths.length;i++){var calls=0,o={length:lengths[i],0:'a',1:'b',2:'c',3:'unvisited'};
assert.sameValue(m.call(o,function(v,k){assert.sameValue(k,reverse?counts[i]-1-calls:calls);calls++;return false;}),absent);assert.sameValue(calls,counts[i]);}
// CASE: findIndex-fractional-string-and-boolean-lengths
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var lengths=[3.9,'3.9',true],counts=[3,3,1];
for(var i=0;i<lengths.length;i++){var calls=0,o={length:lengths[i],0:'a',1:'b',2:'c',3:'unvisited'};
assert.sameValue(m.call(o,function(v,k){assert.sameValue(k,reverse?counts[i]-1-calls:calls);calls++;return false;}),absent);assert.sameValue(calls,counts[i]);}
// CASE: findLast-fractional-string-and-boolean-lengths
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var lengths=[3.9,'3.9',true],counts=[3,3,1];
for(var i=0;i<lengths.length;i++){var calls=0,o={length:lengths[i],0:'a',1:'b',2:'c',3:'unvisited'};
assert.sameValue(m.call(o,function(v,k){assert.sameValue(k,reverse?counts[i]-1-calls:calls);calls++;return false;}),absent);assert.sameValue(calls,counts[i]);}
// CASE: findLastIndex-fractional-string-and-boolean-lengths
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var lengths=[3.9,'3.9',true],counts=[3,3,1];
for(var i=0;i<lengths.length;i++){var calls=0,o={length:lengths[i],0:'a',1:'b',2:'c',3:'unvisited'};
assert.sameValue(m.call(o,function(v,k){assert.sameValue(k,reverse?counts[i]-1-calls:calls);calls++;return false;}),absent);assert.sameValue(calls,counts[i]);}
// CASE: find-empty-still-validates-callability-after-one-length-read
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reads=0,o={get length(){reads++;return 0;}},calls=0;
assert.sameValue(m.call(o,function(){calls++;}),absent);assert.sameValue(reads,1);assert.sameValue(calls,0);
assert.throws(TypeError,function(){m.call(o);});assert.sameValue(reads,2);
assert.throws(TypeError,function(){m.call(o,undefined);});assert.sameValue(reads,3);
// CASE: findIndex-empty-still-validates-callability-after-one-length-read
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reads=0,o={get length(){reads++;return 0;}},calls=0;
assert.sameValue(m.call(o,function(){calls++;}),absent);assert.sameValue(reads,1);assert.sameValue(calls,0);
assert.throws(TypeError,function(){m.call(o);});assert.sameValue(reads,2);
assert.throws(TypeError,function(){m.call(o,undefined);});assert.sameValue(reads,3);
// CASE: findLast-empty-still-validates-callability-after-one-length-read
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reads=0,o={get length(){reads++;return 0;}},calls=0;
assert.sameValue(m.call(o,function(){calls++;}),absent);assert.sameValue(reads,1);assert.sameValue(calls,0);
assert.throws(TypeError,function(){m.call(o);});assert.sameValue(reads,2);
assert.throws(TypeError,function(){m.call(o,undefined);});assert.sameValue(reads,3);
// CASE: findLastIndex-empty-still-validates-callability-after-one-length-read
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reads=0,o={get length(){reads++;return 0;}},calls=0;
assert.sameValue(m.call(o,function(){calls++;}),absent);assert.sameValue(reads,1);assert.sameValue(calls,0);
assert.throws(TypeError,function(){m.call(o);});assert.sameValue(reads,2);
assert.throws(TypeError,function(){m.call(o,undefined);});assert.sameValue(reads,3);
// CASE: find-all-noncallable-predicates-reject-without-coercion
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var bad=[undefined,null,true,0,'callback',{},[],/a/,Symbol('callback')],reads=0;
for(var i=0;i<bad.length;i++){assert.throws(TypeError,function(){m.call({length:0},bad[i]);});}
var poison={get call(){reads++;throw 'call';},get valueOf(){reads++;throw 'valueOf';}};
poison[Symbol.toPrimitive]=function(){reads++;throw 'primitive';};assert.throws(TypeError,function(){m.call([],poison);});assert.sameValue(reads,0);
// CASE: findIndex-all-noncallable-predicates-reject-without-coercion
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var bad=[undefined,null,true,0,'callback',{},[],/a/,Symbol('callback')],reads=0;
for(var i=0;i<bad.length;i++){assert.throws(TypeError,function(){m.call({length:0},bad[i]);});}
var poison={get call(){reads++;throw 'call';},get valueOf(){reads++;throw 'valueOf';}};
poison[Symbol.toPrimitive]=function(){reads++;throw 'primitive';};assert.throws(TypeError,function(){m.call([],poison);});assert.sameValue(reads,0);
// CASE: findLast-all-noncallable-predicates-reject-without-coercion
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var bad=[undefined,null,true,0,'callback',{},[],/a/,Symbol('callback')],reads=0;
for(var i=0;i<bad.length;i++){assert.throws(TypeError,function(){m.call({length:0},bad[i]);});}
var poison={get call(){reads++;throw 'call';},get valueOf(){reads++;throw 'valueOf';}};
poison[Symbol.toPrimitive]=function(){reads++;throw 'primitive';};assert.throws(TypeError,function(){m.call([],poison);});assert.sameValue(reads,0);
// CASE: findLastIndex-all-noncallable-predicates-reject-without-coercion
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var bad=[undefined,null,true,0,'callback',{},[],/a/,Symbol('callback')],reads=0;
for(var i=0;i<bad.length;i++){assert.throws(TypeError,function(){m.call({length:0},bad[i]);});}
var poison={get call(){reads++;throw 'call';},get valueOf(){reads++;throw 'valueOf';}};
poison[Symbol.toPrimitive]=function(){reads++;throw 'primitive';};assert.throws(TypeError,function(){m.call([],poison);});assert.sameValue(reads,0);
// CASE: find-primitive-number-boolean-and-symbol-empty-receivers
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var values=[0,-0,NaN,Infinity,true,false,Symbol('receiver')],calls=0;
for(var i=0;i<values.length;i++){assert.sameValue(m.call(values[i],function(){calls++;return true;}),absent);}
assert.sameValue(calls,0);
// CASE: findIndex-primitive-number-boolean-and-symbol-empty-receivers
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var values=[0,-0,NaN,Infinity,true,false,Symbol('receiver')],calls=0;
for(var i=0;i<values.length;i++){assert.sameValue(m.call(values[i],function(){calls++;return true;}),absent);}
assert.sameValue(calls,0);
// CASE: findLast-primitive-number-boolean-and-symbol-empty-receivers
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var values=[0,-0,NaN,Infinity,true,false,Symbol('receiver')],calls=0;
for(var i=0;i<values.length;i++){assert.sameValue(m.call(values[i],function(){calls++;return true;}),absent);}
assert.sameValue(calls,0);
// CASE: findLastIndex-primitive-number-boolean-and-symbol-empty-receivers
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var values=[0,-0,NaN,Infinity,true,false,Symbol('receiver')],calls=0;
for(var i=0;i<values.length;i++){assert.sameValue(m.call(values[i],function(){calls++;return true;}),absent);}
assert.sameValue(calls,0);
// CASE: find-number-receiver-is-boxed-once-and-shares-callback-object
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var seen,reads=0,calls=0;Object.defineProperty(Number.prototype,'length',{get:function(){assert.sameValue(typeof this,'object');assert.sameValue(this.valueOf(),7);seen=this;reads++;return 2;},configurable:true});
Object.defineProperty(Number.prototype,'1',{get:function(){assert.sameValue(this,seen);return 'inherited';},configurable:true});
assert.sameValue(m.call(7,function(v,k,o){assert.sameValue(o,seen);assert.sameValue(v,k===1?'inherited':undefined);calls++;return false;}),absent);
assert.sameValue(reads,1);assert.sameValue(calls,2);
// CASE: findIndex-number-receiver-is-boxed-once-and-shares-callback-object
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var seen,reads=0,calls=0;Object.defineProperty(Number.prototype,'length',{get:function(){assert.sameValue(typeof this,'object');assert.sameValue(this.valueOf(),7);seen=this;reads++;return 2;},configurable:true});
Object.defineProperty(Number.prototype,'1',{get:function(){assert.sameValue(this,seen);return 'inherited';},configurable:true});
assert.sameValue(m.call(7,function(v,k,o){assert.sameValue(o,seen);assert.sameValue(v,k===1?'inherited':undefined);calls++;return false;}),absent);
assert.sameValue(reads,1);assert.sameValue(calls,2);
// CASE: findLast-number-receiver-is-boxed-once-and-shares-callback-object
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var seen,reads=0,calls=0;Object.defineProperty(Number.prototype,'length',{get:function(){assert.sameValue(typeof this,'object');assert.sameValue(this.valueOf(),7);seen=this;reads++;return 2;},configurable:true});
Object.defineProperty(Number.prototype,'1',{get:function(){assert.sameValue(this,seen);return 'inherited';},configurable:true});
assert.sameValue(m.call(7,function(v,k,o){assert.sameValue(o,seen);assert.sameValue(v,k===1?'inherited':undefined);calls++;return false;}),absent);
assert.sameValue(reads,1);assert.sameValue(calls,2);
// CASE: findLastIndex-number-receiver-is-boxed-once-and-shares-callback-object
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var seen,reads=0,calls=0;Object.defineProperty(Number.prototype,'length',{get:function(){assert.sameValue(typeof this,'object');assert.sameValue(this.valueOf(),7);seen=this;reads++;return 2;},configurable:true});
Object.defineProperty(Number.prototype,'1',{get:function(){assert.sameValue(this,seen);return 'inherited';},configurable:true});
assert.sameValue(m.call(7,function(v,k,o){assert.sameValue(o,seen);assert.sameValue(v,k===1?'inherited':undefined);calls++;return false;}),absent);
assert.sameValue(reads,1);assert.sameValue(calls,2);
// CASE: find-boolean-and-symbol-wrapper-prototype-indices
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var symbol=Symbol('input'),types=[Boolean,Symbol],values=[false,symbol];
for(var i=0;i<types.length;i++){Object.defineProperty(types[i].prototype,'length',{value:1,configurable:true});Object.defineProperty(types[i].prototype,'0',{value:'item',configurable:true});
assert.sameValue(m.call(values[i],function(v,k,o){assert.sameValue(v,'item');assert.sameValue(k,0);assert.sameValue(typeof o,'object');assert.sameValue(o.valueOf(),values[i]);return true;}),result(0,'item'));}
// CASE: findIndex-boolean-and-symbol-wrapper-prototype-indices
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var symbol=Symbol('input'),types=[Boolean,Symbol],values=[false,symbol];
for(var i=0;i<types.length;i++){Object.defineProperty(types[i].prototype,'length',{value:1,configurable:true});Object.defineProperty(types[i].prototype,'0',{value:'item',configurable:true});
assert.sameValue(m.call(values[i],function(v,k,o){assert.sameValue(v,'item');assert.sameValue(k,0);assert.sameValue(typeof o,'object');assert.sameValue(o.valueOf(),values[i]);return true;}),result(0,'item'));}
// CASE: findLast-boolean-and-symbol-wrapper-prototype-indices
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var symbol=Symbol('input'),types=[Boolean,Symbol],values=[false,symbol];
for(var i=0;i<types.length;i++){Object.defineProperty(types[i].prototype,'length',{value:1,configurable:true});Object.defineProperty(types[i].prototype,'0',{value:'item',configurable:true});
assert.sameValue(m.call(values[i],function(v,k,o){assert.sameValue(v,'item');assert.sameValue(k,0);assert.sameValue(typeof o,'object');assert.sameValue(o.valueOf(),values[i]);return true;}),result(0,'item'));}
// CASE: findLastIndex-boolean-and-symbol-wrapper-prototype-indices
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var symbol=Symbol('input'),types=[Boolean,Symbol],values=[false,symbol];
for(var i=0;i<types.length;i++){Object.defineProperty(types[i].prototype,'length',{value:1,configurable:true});Object.defineProperty(types[i].prototype,'0',{value:'item',configurable:true});
assert.sameValue(m.call(values[i],function(v,k,o){assert.sameValue(v,'item');assert.sameValue(k,0);assert.sameValue(typeof o,'object');assert.sameValue(o.valueOf(),values[i]);return true;}),result(0,'item'));}
// CASE: find-primitive-string-uses-utf16-indices-and-one-wrapper
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var text='A\ud800\udc00B',units=['A','\ud800','\udc00','B'],seen,calls=0;
assert.sameValue(m.call(text,function(v,k,o){assert.sameValue(arguments.length,3);assert.sameValue(v,units[k]);assert.sameValue(typeof o,'object');assert.sameValue(o.valueOf(),text);if(calls)assert.sameValue(o,seen);seen=o;calls++;return false;}),absent);
assert.sameValue(calls,4);
// CASE: findIndex-primitive-string-uses-utf16-indices-and-one-wrapper
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var text='A\ud800\udc00B',units=['A','\ud800','\udc00','B'],seen,calls=0;
assert.sameValue(m.call(text,function(v,k,o){assert.sameValue(arguments.length,3);assert.sameValue(v,units[k]);assert.sameValue(typeof o,'object');assert.sameValue(o.valueOf(),text);if(calls)assert.sameValue(o,seen);seen=o;calls++;return false;}),absent);
assert.sameValue(calls,4);
// CASE: findLast-primitive-string-uses-utf16-indices-and-one-wrapper
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var text='A\ud800\udc00B',units=['A','\ud800','\udc00','B'],seen,calls=0;
assert.sameValue(m.call(text,function(v,k,o){assert.sameValue(arguments.length,3);assert.sameValue(v,units[k]);assert.sameValue(typeof o,'object');assert.sameValue(o.valueOf(),text);if(calls)assert.sameValue(o,seen);seen=o;calls++;return false;}),absent);
assert.sameValue(calls,4);
// CASE: findLastIndex-primitive-string-uses-utf16-indices-and-one-wrapper
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var text='A\ud800\udc00B',units=['A','\ud800','\udc00','B'],seen,calls=0;
assert.sameValue(m.call(text,function(v,k,o){assert.sameValue(arguments.length,3);assert.sameValue(v,units[k]);assert.sameValue(typeof o,'object');assert.sameValue(o.valueOf(),text);if(calls)assert.sameValue(o,seen);seen=o;calls++;return false;}),absent);
assert.sameValue(calls,4);
// CASE: find-boxed-string-and-null-prototype-receivers-preserve-identity
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var boxed=new String('\ud800x'),o=Object.create(null);o.length=2;o[0]='a';o[1]='b';
var inputs=[boxed,o];for(var i=0;i<inputs.length;i++){var input=inputs[i],calls=0;
assert.sameValue(m.call(input,function(v,k,receiver){assert.sameValue(receiver,input);assert.sameValue(v,input[k]);calls++;return false;}),absent);assert.sameValue(calls,2);}
// CASE: findIndex-boxed-string-and-null-prototype-receivers-preserve-identity
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var boxed=new String('\ud800x'),o=Object.create(null);o.length=2;o[0]='a';o[1]='b';
var inputs=[boxed,o];for(var i=0;i<inputs.length;i++){var input=inputs[i],calls=0;
assert.sameValue(m.call(input,function(v,k,receiver){assert.sameValue(receiver,input);assert.sameValue(v,input[k]);calls++;return false;}),absent);assert.sameValue(calls,2);}
// CASE: findLast-boxed-string-and-null-prototype-receivers-preserve-identity
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var boxed=new String('\ud800x'),o=Object.create(null);o.length=2;o[0]='a';o[1]='b';
var inputs=[boxed,o];for(var i=0;i<inputs.length;i++){var input=inputs[i],calls=0;
assert.sameValue(m.call(input,function(v,k,receiver){assert.sameValue(receiver,input);assert.sameValue(v,input[k]);calls++;return false;}),absent);assert.sameValue(calls,2);}
// CASE: findLastIndex-boxed-string-and-null-prototype-receivers-preserve-identity
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var boxed=new String('\ud800x'),o=Object.create(null);o.length=2;o[0]='a';o[1]='b';
var inputs=[boxed,o];for(var i=0;i<inputs.length;i++){var input=inputs[i],calls=0;
assert.sameValue(m.call(input,function(v,k,receiver){assert.sameValue(receiver,input);assert.sameValue(v,input[k]);calls++;return false;}),absent);assert.sameValue(calls,2);}
// CASE: find-function-receiver-uses-arity-and-live-indexed-data
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
function input(a,b,c){}input[0]='a';input[1]='b';input[2]='c';var log='';
assert.sameValue(m.call(input,function(v,k,o){assert.sameValue(o,input);log+=k;return false;}),absent);assert.sameValue(log,reverse?'210':'012');
// CASE: findIndex-function-receiver-uses-arity-and-live-indexed-data
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
function input(a,b,c){}input[0]='a';input[1]='b';input[2]='c';var log='';
assert.sameValue(m.call(input,function(v,k,o){assert.sameValue(o,input);log+=k;return false;}),absent);assert.sameValue(log,reverse?'210':'012');
// CASE: findLast-function-receiver-uses-arity-and-live-indexed-data
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
function input(a,b,c){}input[0]='a';input[1]='b';input[2]='c';var log='';
assert.sameValue(m.call(input,function(v,k,o){assert.sameValue(o,input);log+=k;return false;}),absent);assert.sameValue(log,reverse?'210':'012');
// CASE: findLastIndex-function-receiver-uses-arity-and-live-indexed-data
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
function input(a,b,c){}input[0]='a';input[1]='b';input[2]='c';var log='';
assert.sameValue(m.call(input,function(v,k,o){assert.sameValue(o,input);log+=k;return false;}),absent);assert.sameValue(log,reverse?'210':'012');
// CASE: find-mapped-and-unmapped-arguments-receivers
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var sloppy=Function('method','reverse','var args=arguments,calls=0;return method.call(args,function(value,index,obj){if(obj!==args)throw new Error("wrong arguments identity");calls++;return index===0;});');
var found=sloppy(m,reverse);assert.sameValue(found,result(0,m));
function strictInput(a,b){'use strict';var args=arguments;return m.call(args,function(v,k,o){assert.sameValue(o,args);assert.sameValue(v,k===0?a:b);return k===1;});}
assert.sameValue(strictInput('a','b'),result(1,'b'));
// CASE: findIndex-mapped-and-unmapped-arguments-receivers
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var sloppy=Function('method','reverse','var args=arguments,calls=0;return method.call(args,function(value,index,obj){if(obj!==args)throw new Error("wrong arguments identity");calls++;return index===0;});');
var found=sloppy(m,reverse);assert.sameValue(found,result(0,m));
function strictInput(a,b){'use strict';var args=arguments;return m.call(args,function(v,k,o){assert.sameValue(o,args);assert.sameValue(v,k===0?a:b);return k===1;});}
assert.sameValue(strictInput('a','b'),result(1,'b'));
// CASE: findLast-mapped-and-unmapped-arguments-receivers
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var sloppy=Function('method','reverse','var args=arguments,calls=0;return method.call(args,function(value,index,obj){if(obj!==args)throw new Error("wrong arguments identity");calls++;return index===0;});');
var found=sloppy(m,reverse);assert.sameValue(found,result(0,m));
function strictInput(a,b){'use strict';var args=arguments;return m.call(args,function(v,k,o){assert.sameValue(o,args);assert.sameValue(v,k===0?a:b);return k===1;});}
assert.sameValue(strictInput('a','b'),result(1,'b'));
// CASE: findLastIndex-mapped-and-unmapped-arguments-receivers
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var sloppy=Function('method','reverse','var args=arguments,calls=0;return method.call(args,function(value,index,obj){if(obj!==args)throw new Error("wrong arguments identity");calls++;return index===0;});');
var found=sloppy(m,reverse);assert.sameValue(found,result(0,m));
function strictInput(a,b){'use strict';var args=arguments;return m.call(args,function(v,k,o){assert.sameValue(o,args);assert.sameValue(v,k===0?a:b);return k===1;});}
assert.sameValue(strictInput('a','b'),result(1,'b'));
// CASE: find-direction-and-exact-three-callback-arguments
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=['a','b','c'],log='',calls=0;
assert.sameValue(m.call(a,function(v,k,o){assert.sameValue(arguments.length,3);assert.sameValue(arguments[3],undefined);assert.sameValue(typeof k,'number');assert.sameValue(o,a);assert.sameValue(v,a[k]);log+=k;calls++;return false;}),absent);
assert.sameValue(log,reverse?'210':'012');assert.sameValue(calls,3);
// CASE: findIndex-direction-and-exact-three-callback-arguments
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=['a','b','c'],log='',calls=0;
assert.sameValue(m.call(a,function(v,k,o){assert.sameValue(arguments.length,3);assert.sameValue(arguments[3],undefined);assert.sameValue(typeof k,'number');assert.sameValue(o,a);assert.sameValue(v,a[k]);log+=k;calls++;return false;}),absent);
assert.sameValue(log,reverse?'210':'012');assert.sameValue(calls,3);
// CASE: findLast-direction-and-exact-three-callback-arguments
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=['a','b','c'],log='',calls=0;
assert.sameValue(m.call(a,function(v,k,o){assert.sameValue(arguments.length,3);assert.sameValue(arguments[3],undefined);assert.sameValue(typeof k,'number');assert.sameValue(o,a);assert.sameValue(v,a[k]);log+=k;calls++;return false;}),absent);
assert.sameValue(log,reverse?'210':'012');assert.sameValue(calls,3);
// CASE: findLastIndex-direction-and-exact-three-callback-arguments
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=['a','b','c'],log='',calls=0;
assert.sameValue(m.call(a,function(v,k,o){assert.sameValue(arguments.length,3);assert.sameValue(arguments[3],undefined);assert.sameValue(typeof k,'number');assert.sameValue(o,a);assert.sameValue(v,a[k]);log+=k;calls++;return false;}),absent);
assert.sameValue(log,reverse?'210':'012');assert.sameValue(calls,3);
// CASE: find-first-matching-index-in-direction-short-circuits
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=['hit','miss','hit'],calls=0;
assert.sameValue(m.call(a,function(v,k){calls++;assert.sameValue(k,reverse?2:0);return v==='hit';}),result(reverse?2:0,'hit'));assert.sameValue(calls,1);
// CASE: findIndex-first-matching-index-in-direction-short-circuits
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=['hit','miss','hit'],calls=0;
assert.sameValue(m.call(a,function(v,k){calls++;assert.sameValue(k,reverse?2:0);return v==='hit';}),result(reverse?2:0,'hit'));assert.sameValue(calls,1);
// CASE: findLast-first-matching-index-in-direction-short-circuits
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=['hit','miss','hit'],calls=0;
assert.sameValue(m.call(a,function(v,k){calls++;assert.sameValue(k,reverse?2:0);return v==='hit';}),result(reverse?2:0,'hit'));assert.sameValue(calls,1);
// CASE: findLastIndex-first-matching-index-in-direction-short-circuits
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=['hit','miss','hit'],calls=0;
assert.sameValue(m.call(a,function(v,k){calls++;assert.sameValue(k,reverse?2:0);return v==='hit';}),result(reverse?2:0,'hit'));assert.sameValue(calls,1);
// CASE: find-short-circuit-never-gets-later-index
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var start=reverse?2:0,end=reverse?0:2,o={length:3},token={},reads=0;o[start]=token;
Object.defineProperty(o,'1',{get:function(){reads++;throw 'later getter';}});Object.defineProperty(o,end,{get:function(){reads++;throw 'last getter';}});
assert.sameValue(m.call(o,function(){return true;}),result(start,token));assert.sameValue(reads,0);
// CASE: findIndex-short-circuit-never-gets-later-index
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var start=reverse?2:0,end=reverse?0:2,o={length:3},token={},reads=0;o[start]=token;
Object.defineProperty(o,'1',{get:function(){reads++;throw 'later getter';}});Object.defineProperty(o,end,{get:function(){reads++;throw 'last getter';}});
assert.sameValue(m.call(o,function(){return true;}),result(start,token));assert.sameValue(reads,0);
// CASE: findLast-short-circuit-never-gets-later-index
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var start=reverse?2:0,end=reverse?0:2,o={length:3},token={},reads=0;o[start]=token;
Object.defineProperty(o,'1',{get:function(){reads++;throw 'later getter';}});Object.defineProperty(o,end,{get:function(){reads++;throw 'last getter';}});
assert.sameValue(m.call(o,function(){return true;}),result(start,token));assert.sameValue(reads,0);
// CASE: findLastIndex-short-circuit-never-gets-later-index
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var start=reverse?2:0,end=reverse?0:2,o={length:3},token={},reads=0;o[start]=token;
Object.defineProperty(o,'1',{get:function(){reads++;throw 'later getter';}});Object.defineProperty(o,end,{get:function(){reads++;throw 'last getter';}});
assert.sameValue(m.call(o,function(){return true;}),result(start,token));assert.sameValue(reads,0);
// CASE: find-holes-are-visited-with-undefined-not-skipped
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=new Array(3),log='',calls=0;
assert.sameValue(m.call(a,function(v,k,o){assert.sameValue(v,undefined);assert.sameValue(k in o,false);log+=k;calls++;return false;}),absent);
assert.sameValue(log,reverse?'210':'012');assert.sameValue(calls,3);
// CASE: findIndex-holes-are-visited-with-undefined-not-skipped
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=new Array(3),log='',calls=0;
assert.sameValue(m.call(a,function(v,k,o){assert.sameValue(v,undefined);assert.sameValue(k in o,false);log+=k;calls++;return false;}),absent);
assert.sameValue(log,reverse?'210':'012');assert.sameValue(calls,3);
// CASE: findLast-holes-are-visited-with-undefined-not-skipped
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=new Array(3),log='',calls=0;
assert.sameValue(m.call(a,function(v,k,o){assert.sameValue(v,undefined);assert.sameValue(k in o,false);log+=k;calls++;return false;}),absent);
assert.sameValue(log,reverse?'210':'012');assert.sameValue(calls,3);
// CASE: findLastIndex-holes-are-visited-with-undefined-not-skipped
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=new Array(3),log='',calls=0;
assert.sameValue(m.call(a,function(v,k,o){assert.sameValue(v,undefined);assert.sameValue(k in o,false);log+=k;calls++;return false;}),absent);
assert.sameValue(log,reverse?'210':'012');assert.sameValue(calls,3);
// CASE: find-a-hole-can-match-and-stop
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=new Array(3),calls=0;
assert.sameValue(m.call(a,function(v,k){calls++;assert.sameValue(v,undefined);assert.sameValue(k,reverse?2:0);return true;}),result(reverse?2:0,undefined));assert.sameValue(calls,1);
// CASE: findIndex-a-hole-can-match-and-stop
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=new Array(3),calls=0;
assert.sameValue(m.call(a,function(v,k){calls++;assert.sameValue(v,undefined);assert.sameValue(k,reverse?2:0);return true;}),result(reverse?2:0,undefined));assert.sameValue(calls,1);
// CASE: findLast-a-hole-can-match-and-stop
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=new Array(3),calls=0;
assert.sameValue(m.call(a,function(v,k){calls++;assert.sameValue(v,undefined);assert.sameValue(k,reverse?2:0);return true;}),result(reverse?2:0,undefined));assert.sameValue(calls,1);
// CASE: findLastIndex-a-hole-can-match-and-stop
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=new Array(3),calls=0;
assert.sameValue(m.call(a,function(v,k){calls++;assert.sameValue(v,undefined);assert.sameValue(k,reverse?2:0);return true;}),result(reverse?2:0,undefined));assert.sameValue(calls,1);
// CASE: find-inherited-array-index-getter-uses-original-receiver
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=[1,,3],reads=0,p={};Object.defineProperty(p,'1',{get:function(){reads++;assert.sameValue(this,a);return 7;}});Object.setPrototypeOf(a,p);
var log='';assert.sameValue(m.call(a,function(v,k,o){assert.sameValue(o,a);log+=k;return k===1;}),result(1,7));
assert.sameValue(log,reverse?'21':'01');assert.sameValue(reads,1);assert.sameValue(Object.getOwnPropertyDescriptor(a,'1'),undefined);
// CASE: findIndex-inherited-array-index-getter-uses-original-receiver
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=[1,,3],reads=0,p={};Object.defineProperty(p,'1',{get:function(){reads++;assert.sameValue(this,a);return 7;}});Object.setPrototypeOf(a,p);
var log='';assert.sameValue(m.call(a,function(v,k,o){assert.sameValue(o,a);log+=k;return k===1;}),result(1,7));
assert.sameValue(log,reverse?'21':'01');assert.sameValue(reads,1);assert.sameValue(Object.getOwnPropertyDescriptor(a,'1'),undefined);
// CASE: findLast-inherited-array-index-getter-uses-original-receiver
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=[1,,3],reads=0,p={};Object.defineProperty(p,'1',{get:function(){reads++;assert.sameValue(this,a);return 7;}});Object.setPrototypeOf(a,p);
var log='';assert.sameValue(m.call(a,function(v,k,o){assert.sameValue(o,a);log+=k;return k===1;}),result(1,7));
assert.sameValue(log,reverse?'21':'01');assert.sameValue(reads,1);assert.sameValue(Object.getOwnPropertyDescriptor(a,'1'),undefined);
// CASE: findLastIndex-inherited-array-index-getter-uses-original-receiver
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=[1,,3],reads=0,p={};Object.defineProperty(p,'1',{get:function(){reads++;assert.sameValue(this,a);return 7;}});Object.setPrototypeOf(a,p);
var log='';assert.sameValue(m.call(a,function(v,k,o){assert.sameValue(o,a);log+=k;return k===1;}),result(1,7));
assert.sameValue(log,reverse?'21':'01');assert.sameValue(reads,1);assert.sameValue(Object.getOwnPropertyDescriptor(a,'1'),undefined);
// CASE: find-inherited-length-and-index-getters-are-live
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var log='',o,p={get length(){assert.sameValue(this,o);log+='L';return 1;},get 0(){assert.sameValue(this,o);log+='G';return 9;}};o=Object.create(p);
assert.sameValue(m.call(o,function(v,k,receiver){log+='C';assert.sameValue(receiver,o);return true;}),result(0,9));assert.sameValue(log,'LGC');
// CASE: findIndex-inherited-length-and-index-getters-are-live
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var log='',o,p={get length(){assert.sameValue(this,o);log+='L';return 1;},get 0(){assert.sameValue(this,o);log+='G';return 9;}};o=Object.create(p);
assert.sameValue(m.call(o,function(v,k,receiver){log+='C';assert.sameValue(receiver,o);return true;}),result(0,9));assert.sameValue(log,'LGC');
// CASE: findLast-inherited-length-and-index-getters-are-live
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var log='',o,p={get length(){assert.sameValue(this,o);log+='L';return 1;},get 0(){assert.sameValue(this,o);log+='G';return 9;}};o=Object.create(p);
assert.sameValue(m.call(o,function(v,k,receiver){log+='C';assert.sameValue(receiver,o);return true;}),result(0,9));assert.sameValue(log,'LGC');
// CASE: findLastIndex-inherited-length-and-index-getters-are-live
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var log='',o,p={get length(){assert.sameValue(this,o);log+='L';return 1;},get 0(){assert.sameValue(this,o);log+='G';return 9;}};o=Object.create(p);
assert.sameValue(m.call(o,function(v,k,receiver){log+='C';assert.sameValue(receiver,o);return true;}),result(0,9));assert.sameValue(log,'LGC');
// CASE: find-index-getter-runs-once-before-predicate
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var token={},reads=0,calls=0,o={length:1,get 0(){reads++;return token;}};
assert.sameValue(m.call(o,function(v){calls++;assert.sameValue(v,token);assert.sameValue(reads,1);return true;}),result(0,token));assert.sameValue(reads,1);assert.sameValue(calls,1);
// CASE: findIndex-index-getter-runs-once-before-predicate
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var token={},reads=0,calls=0,o={length:1,get 0(){reads++;return token;}};
assert.sameValue(m.call(o,function(v){calls++;assert.sameValue(v,token);assert.sameValue(reads,1);return true;}),result(0,token));assert.sameValue(reads,1);assert.sameValue(calls,1);
// CASE: findLast-index-getter-runs-once-before-predicate
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var token={},reads=0,calls=0,o={length:1,get 0(){reads++;return token;}};
assert.sameValue(m.call(o,function(v){calls++;assert.sameValue(v,token);assert.sameValue(reads,1);return true;}),result(0,token));assert.sameValue(reads,1);assert.sameValue(calls,1);
// CASE: findLastIndex-index-getter-runs-once-before-predicate
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var token={},reads=0,calls=0,o={length:1,get 0(){reads++;return token;}};
assert.sameValue(m.call(o,function(v){calls++;assert.sameValue(v,token);assert.sameValue(reads,1);return true;}),result(0,token));assert.sameValue(reads,1);assert.sameValue(calls,1);
// CASE: find-getter-mutations-change-future-values-including-deleted-holes
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var start=reverse?2:0,end=reverse?0:2,o={length:3},values=[];o[1]='old';o[end]='old end';
Object.defineProperty(o,start,{get:function(){o[1]='new';delete o[end];return 'first';}});
assert.sameValue(m.call(o,function(v){values.push(v);return false;}),absent);
assert.sameValue(values.length,3);assert.sameValue(values[0],'first');assert.sameValue(values[1],'new');assert.sameValue(values[2],undefined);
// CASE: findIndex-getter-mutations-change-future-values-including-deleted-holes
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var start=reverse?2:0,end=reverse?0:2,o={length:3},values=[];o[1]='old';o[end]='old end';
Object.defineProperty(o,start,{get:function(){o[1]='new';delete o[end];return 'first';}});
assert.sameValue(m.call(o,function(v){values.push(v);return false;}),absent);
assert.sameValue(values.length,3);assert.sameValue(values[0],'first');assert.sameValue(values[1],'new');assert.sameValue(values[2],undefined);
// CASE: findLast-getter-mutations-change-future-values-including-deleted-holes
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var start=reverse?2:0,end=reverse?0:2,o={length:3},values=[];o[1]='old';o[end]='old end';
Object.defineProperty(o,start,{get:function(){o[1]='new';delete o[end];return 'first';}});
assert.sameValue(m.call(o,function(v){values.push(v);return false;}),absent);
assert.sameValue(values.length,3);assert.sameValue(values[0],'first');assert.sameValue(values[1],'new');assert.sameValue(values[2],undefined);
// CASE: findLastIndex-getter-mutations-change-future-values-including-deleted-holes
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var start=reverse?2:0,end=reverse?0:2,o={length:3},values=[];o[1]='old';o[end]='old end';
Object.defineProperty(o,start,{get:function(){o[1]='new';delete o[end];return 'first';}});
assert.sameValue(m.call(o,function(v){values.push(v);return false;}),absent);
assert.sameValue(values.length,3);assert.sameValue(values[0],'first');assert.sameValue(values[1],'new');assert.sameValue(values[2],undefined);
// CASE: find-callback-mutations-use-captured-length-and-live-values
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var start=reverse?2:0,end=reverse?0:2,a=['a','b','c'],values=[];
assert.sameValue(m.call(a,function(v,k){values.push(v);if(k===start){a[1]='changed';delete a[end];a[3]='outside';}return false;}),absent);
assert.sameValue(values.length,3);assert.sameValue(values[0],reverse?'c':'a');assert.sameValue(values[1],'changed');assert.sameValue(values[2],undefined);assert.sameValue(a.length,4);
// CASE: findIndex-callback-mutations-use-captured-length-and-live-values
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var start=reverse?2:0,end=reverse?0:2,a=['a','b','c'],values=[];
assert.sameValue(m.call(a,function(v,k){values.push(v);if(k===start){a[1]='changed';delete a[end];a[3]='outside';}return false;}),absent);
assert.sameValue(values.length,3);assert.sameValue(values[0],reverse?'c':'a');assert.sameValue(values[1],'changed');assert.sameValue(values[2],undefined);assert.sameValue(a.length,4);
// CASE: findLast-callback-mutations-use-captured-length-and-live-values
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var start=reverse?2:0,end=reverse?0:2,a=['a','b','c'],values=[];
assert.sameValue(m.call(a,function(v,k){values.push(v);if(k===start){a[1]='changed';delete a[end];a[3]='outside';}return false;}),absent);
assert.sameValue(values.length,3);assert.sameValue(values[0],reverse?'c':'a');assert.sameValue(values[1],'changed');assert.sameValue(values[2],undefined);assert.sameValue(a.length,4);
// CASE: findLastIndex-callback-mutations-use-captured-length-and-live-values
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var start=reverse?2:0,end=reverse?0:2,a=['a','b','c'],values=[];
assert.sameValue(m.call(a,function(v,k){values.push(v);if(k===start){a[1]='changed';delete a[end];a[3]='outside';}return false;}),absent);
assert.sameValue(values.length,3);assert.sameValue(values[0],reverse?'c':'a');assert.sameValue(values[1],'changed');assert.sameValue(values[2],undefined);assert.sameValue(a.length,4);
// CASE: find-deletion-reveals-newly-read-inherited-property
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var start=reverse?2:0,p={1:'inherited'},o=Object.create(p),values=[];o.length=3;o[0]='a';o[1]='own';o[2]='c';
assert.sameValue(m.call(o,function(v,k){values.push(v);if(k===start)delete o[1];return k===1;}),result(1,'inherited'));
assert.sameValue(values.length,2);assert.sameValue(values[1],'inherited');
// CASE: findIndex-deletion-reveals-newly-read-inherited-property
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var start=reverse?2:0,p={1:'inherited'},o=Object.create(p),values=[];o.length=3;o[0]='a';o[1]='own';o[2]='c';
assert.sameValue(m.call(o,function(v,k){values.push(v);if(k===start)delete o[1];return k===1;}),result(1,'inherited'));
assert.sameValue(values.length,2);assert.sameValue(values[1],'inherited');
// CASE: findLast-deletion-reveals-newly-read-inherited-property
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var start=reverse?2:0,p={1:'inherited'},o=Object.create(p),values=[];o.length=3;o[0]='a';o[1]='own';o[2]='c';
assert.sameValue(m.call(o,function(v,k){values.push(v);if(k===start)delete o[1];return k===1;}),result(1,'inherited'));
assert.sameValue(values.length,2);assert.sameValue(values[1],'inherited');
// CASE: findLastIndex-deletion-reveals-newly-read-inherited-property
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var start=reverse?2:0,p={1:'inherited'},o=Object.create(p),values=[];o.length=3;o[0]='a';o[1]='own';o[2]='c';
assert.sameValue(m.call(o,function(v,k){values.push(v);if(k===start)delete o[1];return k===1;}),result(1,'inherited'));
assert.sameValue(values.length,2);assert.sameValue(values[1],'inherited');
// CASE: find-shrinking-array-length-still-visits-captured-range
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=['a','b','c'],values=[],indices='';
assert.sameValue(m.call(a,function(v,k){values.push(v);indices+=k;if(values.length===1)a.length=0;return false;}),absent);
assert.sameValue(indices,reverse?'210':'012');assert.sameValue(values.length,3);assert.sameValue(values[0],reverse?'c':'a');assert.sameValue(values[1],undefined);assert.sameValue(values[2],undefined);assert.sameValue(a.length,0);
// CASE: findIndex-shrinking-array-length-still-visits-captured-range
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=['a','b','c'],values=[],indices='';
assert.sameValue(m.call(a,function(v,k){values.push(v);indices+=k;if(values.length===1)a.length=0;return false;}),absent);
assert.sameValue(indices,reverse?'210':'012');assert.sameValue(values.length,3);assert.sameValue(values[0],reverse?'c':'a');assert.sameValue(values[1],undefined);assert.sameValue(values[2],undefined);assert.sameValue(a.length,0);
// CASE: findLast-shrinking-array-length-still-visits-captured-range
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=['a','b','c'],values=[],indices='';
assert.sameValue(m.call(a,function(v,k){values.push(v);indices+=k;if(values.length===1)a.length=0;return false;}),absent);
assert.sameValue(indices,reverse?'210':'012');assert.sameValue(values.length,3);assert.sameValue(values[0],reverse?'c':'a');assert.sameValue(values[1],undefined);assert.sameValue(values[2],undefined);assert.sameValue(a.length,0);
// CASE: findLastIndex-shrinking-array-length-still-visits-captured-range
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=['a','b','c'],values=[],indices='';
assert.sameValue(m.call(a,function(v,k){values.push(v);indices+=k;if(values.length===1)a.length=0;return false;}),absent);
assert.sameValue(indices,reverse?'210':'012');assert.sameValue(values.length,3);assert.sameValue(values[0],reverse?'c':'a');assert.sameValue(values[1],undefined);assert.sameValue(values[2],undefined);assert.sameValue(a.length,0);
// CASE: find-replaced-length-getter-is-not-reconsulted
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reads=0,o={0:'a',1:'b',2:'c',get length(){reads++;return 3;}},calls=0;
assert.sameValue(m.call(o,function(){calls++;if(calls===1)Object.defineProperty(o,'length',{get:function(){throw 'second length';}});return false;}),absent);
assert.sameValue(reads,1);assert.sameValue(calls,3);
// CASE: findIndex-replaced-length-getter-is-not-reconsulted
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reads=0,o={0:'a',1:'b',2:'c',get length(){reads++;return 3;}},calls=0;
assert.sameValue(m.call(o,function(){calls++;if(calls===1)Object.defineProperty(o,'length',{get:function(){throw 'second length';}});return false;}),absent);
assert.sameValue(reads,1);assert.sameValue(calls,3);
// CASE: findLast-replaced-length-getter-is-not-reconsulted
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reads=0,o={0:'a',1:'b',2:'c',get length(){reads++;return 3;}},calls=0;
assert.sameValue(m.call(o,function(){calls++;if(calls===1)Object.defineProperty(o,'length',{get:function(){throw 'second length';}});return false;}),absent);
assert.sameValue(reads,1);assert.sameValue(calls,3);
// CASE: findLastIndex-replaced-length-getter-is-not-reconsulted
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reads=0,o={0:'a',1:'b',2:'c',get length(){reads++;return 3;}},calls=0;
assert.sameValue(m.call(o,function(){calls++;if(calls===1)Object.defineProperty(o,'length',{get:function(){throw 'second length';}});return false;}),absent);
assert.sameValue(reads,1);assert.sameValue(calls,3);
// CASE: find-length-conversion-mutation-precedes-captured-range
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var o={0:'a',1:'b',2:'c',length:{valueOf:function(){o[1]='converted';o[2]='outside';return 2;}}},log='';
assert.sameValue(m.call(o,function(v,k){log+=k;assert.sameValue(v,k===1?'converted':'a');return false;}),absent);assert.sameValue(log,reverse?'10':'01');
// CASE: findIndex-length-conversion-mutation-precedes-captured-range
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var o={0:'a',1:'b',2:'c',length:{valueOf:function(){o[1]='converted';o[2]='outside';return 2;}}},log='';
assert.sameValue(m.call(o,function(v,k){log+=k;assert.sameValue(v,k===1?'converted':'a');return false;}),absent);assert.sameValue(log,reverse?'10':'01');
// CASE: findLast-length-conversion-mutation-precedes-captured-range
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var o={0:'a',1:'b',2:'c',length:{valueOf:function(){o[1]='converted';o[2]='outside';return 2;}}},log='';
assert.sameValue(m.call(o,function(v,k){log+=k;assert.sameValue(v,k===1?'converted':'a');return false;}),absent);assert.sameValue(log,reverse?'10':'01');
// CASE: findLastIndex-length-conversion-mutation-precedes-captured-range
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var o={0:'a',1:'b',2:'c',length:{valueOf:function(){o[1]='converted';o[2]='outside';return 2;}}},log='';
assert.sameValue(m.call(o,function(v,k){log+=k;assert.sameValue(v,k===1?'converted':'a');return false;}),absent);assert.sameValue(log,reverse?'10':'01');
// CASE: find-prototype-replacement-is-observed-on-future-get
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var start=reverse?2:0,o=Object.create({1:'old'});o.length=3;o[0]='a';o[2]='c';var next={1:'new'};
assert.sameValue(m.call(o,function(v,k){if(k===start)Object.setPrototypeOf(o,next);return k===1;}),result(1,'new'));
// CASE: findIndex-prototype-replacement-is-observed-on-future-get
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var start=reverse?2:0,o=Object.create({1:'old'});o.length=3;o[0]='a';o[2]='c';var next={1:'new'};
assert.sameValue(m.call(o,function(v,k){if(k===start)Object.setPrototypeOf(o,next);return k===1;}),result(1,'new'));
// CASE: findLast-prototype-replacement-is-observed-on-future-get
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var start=reverse?2:0,o=Object.create({1:'old'});o.length=3;o[0]='a';o[2]='c';var next={1:'new'};
assert.sameValue(m.call(o,function(v,k){if(k===start)Object.setPrototypeOf(o,next);return k===1;}),result(1,'new'));
// CASE: findLastIndex-prototype-replacement-is-observed-on-future-get
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var start=reverse?2:0,o=Object.create({1:'old'});o.length=3;o[0]='a';o[2]='c';var next={1:'new'};
assert.sameValue(m.call(o,function(v,k){if(k===start)Object.setPrototypeOf(o,next);return k===1;}),result(1,'new'));
// CASE: find-already-visited-indices-are-not-revisited
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=['a','b','c'],start=reverse?2:0,calls=0,log='';
assert.sameValue(m.call(a,function(v,k){calls++;log+=k;if(k===1)a[start]='changed after visit';return false;}),absent);assert.sameValue(calls,3);assert.sameValue(log,reverse?'210':'012');
// CASE: findIndex-already-visited-indices-are-not-revisited
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=['a','b','c'],start=reverse?2:0,calls=0,log='';
assert.sameValue(m.call(a,function(v,k){calls++;log+=k;if(k===1)a[start]='changed after visit';return false;}),absent);assert.sameValue(calls,3);assert.sameValue(log,reverse?'210':'012');
// CASE: findLast-already-visited-indices-are-not-revisited
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=['a','b','c'],start=reverse?2:0,calls=0,log='';
assert.sameValue(m.call(a,function(v,k){calls++;log+=k;if(k===1)a[start]='changed after visit';return false;}),absent);assert.sameValue(calls,3);assert.sameValue(log,reverse?'210':'012');
// CASE: findLastIndex-already-visited-indices-are-not-revisited
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=['a','b','c'],start=reverse?2:0,calls=0,log='';
assert.sameValue(m.call(a,function(v,k){calls++;log+=k;if(k===1)a[start]='changed after visit';return false;}),absent);assert.sameValue(calls,3);assert.sameValue(log,reverse?'210':'012');
// CASE: find-truthy-object-result-never-invokes-coercion-or-then
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reads=0,truth={get valueOf(){reads++;throw 'valueOf';},get toString(){reads++;throw 'toString';},get then(){reads++;throw 'then';}};
Object.defineProperty(truth,Symbol.toPrimitive,{get:function(){reads++;throw 'primitive';}});var token={};
assert.sameValue(m.call([token],function(){return truth;}),result(0,token));assert.sameValue(reads,0);
// CASE: findIndex-truthy-object-result-never-invokes-coercion-or-then
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reads=0,truth={get valueOf(){reads++;throw 'valueOf';},get toString(){reads++;throw 'toString';},get then(){reads++;throw 'then';}};
Object.defineProperty(truth,Symbol.toPrimitive,{get:function(){reads++;throw 'primitive';}});var token={};
assert.sameValue(m.call([token],function(){return truth;}),result(0,token));assert.sameValue(reads,0);
// CASE: findLast-truthy-object-result-never-invokes-coercion-or-then
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reads=0,truth={get valueOf(){reads++;throw 'valueOf';},get toString(){reads++;throw 'toString';},get then(){reads++;throw 'then';}};
Object.defineProperty(truth,Symbol.toPrimitive,{get:function(){reads++;throw 'primitive';}});var token={};
assert.sameValue(m.call([token],function(){return truth;}),result(0,token));assert.sameValue(reads,0);
// CASE: findLastIndex-truthy-object-result-never-invokes-coercion-or-then
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reads=0,truth={get valueOf(){reads++;throw 'valueOf';},get toString(){reads++;throw 'toString';},get then(){reads++;throw 'then';}};
Object.defineProperty(truth,Symbol.toPrimitive,{get:function(){reads++;throw 'primitive';}});var token={};
assert.sameValue(m.call([token],function(){return truth;}),result(0,token));assert.sameValue(reads,0);
// CASE: find-all-truthy-results-stop-without-returning-predicate-result
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var answers=[true,1,-1,'yes',{},[],function(){},Symbol('answer'),new Boolean(false)],token={};
for(var i=0;i<answers.length;i++){var calls=0;assert.sameValue(m.call([token,token],function(v,k){calls++;return answers[i];}),result(reverse?1:0,token));assert.sameValue(calls,1);}
// CASE: findIndex-all-truthy-results-stop-without-returning-predicate-result
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var answers=[true,1,-1,'yes',{},[],function(){},Symbol('answer'),new Boolean(false)],token={};
for(var i=0;i<answers.length;i++){var calls=0;assert.sameValue(m.call([token,token],function(v,k){calls++;return answers[i];}),result(reverse?1:0,token));assert.sameValue(calls,1);}
// CASE: findLast-all-truthy-results-stop-without-returning-predicate-result
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var answers=[true,1,-1,'yes',{},[],function(){},Symbol('answer'),new Boolean(false)],token={};
for(var i=0;i<answers.length;i++){var calls=0;assert.sameValue(m.call([token,token],function(v,k){calls++;return answers[i];}),result(reverse?1:0,token));assert.sameValue(calls,1);}
// CASE: findLastIndex-all-truthy-results-stop-without-returning-predicate-result
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var answers=[true,1,-1,'yes',{},[],function(){},Symbol('answer'),new Boolean(false)],token={};
for(var i=0;i<answers.length;i++){var calls=0;assert.sameValue(m.call([token,token],function(v,k){calls++;return answers[i];}),result(reverse?1:0,token));assert.sameValue(calls,1);}
// CASE: find-all-falsy-results-continue-through-holes
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var answers=[false,0,-0,NaN,'',null,undefined];
for(var i=0;i<answers.length;i++){var calls=0;assert.sameValue(m.call(new Array(3),function(){calls++;return answers[i];}),absent);assert.sameValue(calls,3);}
// CASE: findIndex-all-falsy-results-continue-through-holes
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var answers=[false,0,-0,NaN,'',null,undefined];
for(var i=0;i<answers.length;i++){var calls=0;assert.sameValue(m.call(new Array(3),function(){calls++;return answers[i];}),absent);assert.sameValue(calls,3);}
// CASE: findLast-all-falsy-results-continue-through-holes
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var answers=[false,0,-0,NaN,'',null,undefined];
for(var i=0;i<answers.length;i++){var calls=0;assert.sameValue(m.call(new Array(3),function(){calls++;return answers[i];}),absent);assert.sameValue(calls,3);}
// CASE: findLastIndex-all-falsy-results-continue-through-holes
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var answers=[false,0,-0,NaN,'',null,undefined];
for(var i=0;i<answers.length;i++){var calls=0;assert.sameValue(m.call(new Array(3),function(){calls++;return answers[i];}),absent);assert.sameValue(calls,3);}
// CASE: find-strict-callback-receives-exact-thisarg
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var receivers=[undefined,null,0,-0,false,'text',Symbol('this'),{}],token={};
for(var i=0;i<receivers.length;i++){var expectedThis=receivers[i];assert.sameValue(m.call([token],function(v,k,o){'use strict';assert.sameValue(this,expectedThis);assert.sameValue(arguments.length,3);return true;},expectedThis),result(0,token));}
assert.sameValue(m.call([token],function(){'use strict';assert.sameValue(this,undefined);return true;}),result(0,token));
// CASE: findIndex-strict-callback-receives-exact-thisarg
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var receivers=[undefined,null,0,-0,false,'text',Symbol('this'),{}],token={};
for(var i=0;i<receivers.length;i++){var expectedThis=receivers[i];assert.sameValue(m.call([token],function(v,k,o){'use strict';assert.sameValue(this,expectedThis);assert.sameValue(arguments.length,3);return true;},expectedThis),result(0,token));}
assert.sameValue(m.call([token],function(){'use strict';assert.sameValue(this,undefined);return true;}),result(0,token));
// CASE: findLast-strict-callback-receives-exact-thisarg
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var receivers=[undefined,null,0,-0,false,'text',Symbol('this'),{}],token={};
for(var i=0;i<receivers.length;i++){var expectedThis=receivers[i];assert.sameValue(m.call([token],function(v,k,o){'use strict';assert.sameValue(this,expectedThis);assert.sameValue(arguments.length,3);return true;},expectedThis),result(0,token));}
assert.sameValue(m.call([token],function(){'use strict';assert.sameValue(this,undefined);return true;}),result(0,token));
// CASE: findLastIndex-strict-callback-receives-exact-thisarg
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var receivers=[undefined,null,0,-0,false,'text',Symbol('this'),{}],token={};
for(var i=0;i<receivers.length;i++){var expectedThis=receivers[i];assert.sameValue(m.call([token],function(v,k,o){'use strict';assert.sameValue(this,expectedThis);assert.sameValue(arguments.length,3);return true;},expectedThis),result(0,token));}
assert.sameValue(m.call([token],function(){'use strict';assert.sameValue(this,undefined);return true;}),result(0,token));
// CASE: find-sloppy-callback-normalizes-this-independently-of-caller-mode
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var expectedThis,sloppy=Function('v','k','o','if(expectedThis===null||expectedThis===undefined){assert.sameValue(this,globalThis);}else if(typeof expectedThis==="object"){assert.sameValue(this,expectedThis);}else{assert.sameValue(typeof this,"object");assert.sameValue(this.valueOf(),expectedThis);}assert.sameValue(arguments.length,3);return true;');
var receivers=[undefined,null,0,false,'text',Symbol('this'),{}],token={};
for(var i=0;i<receivers.length;i++){expectedThis=receivers[i];assert.sameValue(m.call([token],sloppy,expectedThis),result(0,token));}
expectedThis=undefined;assert.sameValue(m.call([token],sloppy),result(0,token));
// CASE: findIndex-sloppy-callback-normalizes-this-independently-of-caller-mode
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var expectedThis,sloppy=Function('v','k','o','if(expectedThis===null||expectedThis===undefined){assert.sameValue(this,globalThis);}else if(typeof expectedThis==="object"){assert.sameValue(this,expectedThis);}else{assert.sameValue(typeof this,"object");assert.sameValue(this.valueOf(),expectedThis);}assert.sameValue(arguments.length,3);return true;');
var receivers=[undefined,null,0,false,'text',Symbol('this'),{}],token={};
for(var i=0;i<receivers.length;i++){expectedThis=receivers[i];assert.sameValue(m.call([token],sloppy,expectedThis),result(0,token));}
expectedThis=undefined;assert.sameValue(m.call([token],sloppy),result(0,token));
// CASE: findLast-sloppy-callback-normalizes-this-independently-of-caller-mode
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var expectedThis,sloppy=Function('v','k','o','if(expectedThis===null||expectedThis===undefined){assert.sameValue(this,globalThis);}else if(typeof expectedThis==="object"){assert.sameValue(this,expectedThis);}else{assert.sameValue(typeof this,"object");assert.sameValue(this.valueOf(),expectedThis);}assert.sameValue(arguments.length,3);return true;');
var receivers=[undefined,null,0,false,'text',Symbol('this'),{}],token={};
for(var i=0;i<receivers.length;i++){expectedThis=receivers[i];assert.sameValue(m.call([token],sloppy,expectedThis),result(0,token));}
expectedThis=undefined;assert.sameValue(m.call([token],sloppy),result(0,token));
// CASE: findLastIndex-sloppy-callback-normalizes-this-independently-of-caller-mode
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var expectedThis,sloppy=Function('v','k','o','if(expectedThis===null||expectedThis===undefined){assert.sameValue(this,globalThis);}else if(typeof expectedThis==="object"){assert.sameValue(this,expectedThis);}else{assert.sameValue(typeof this,"object");assert.sameValue(this.valueOf(),expectedThis);}assert.sameValue(arguments.length,3);return true;');
var receivers=[undefined,null,0,false,'text',Symbol('this'),{}],token={};
for(var i=0;i<receivers.length;i++){expectedThis=receivers[i];assert.sameValue(m.call([token],sloppy,expectedThis),result(0,token));}
expectedThis=undefined;assert.sameValue(m.call([token],sloppy),result(0,token));
// CASE: find-arrow-predicate-keeps-lexical-this
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var owner={},other={},token={};function make(){return (v,k,o)=>{assert.sameValue(this,owner);assert.sameValue(v,token);return true;};}
var predicate=make.call(owner);assert.sameValue(m.call([token],predicate,other),result(0,token));
// CASE: findIndex-arrow-predicate-keeps-lexical-this
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var owner={},other={},token={};function make(){return (v,k,o)=>{assert.sameValue(this,owner);assert.sameValue(v,token);return true;};}
var predicate=make.call(owner);assert.sameValue(m.call([token],predicate,other),result(0,token));
// CASE: findLast-arrow-predicate-keeps-lexical-this
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var owner={},other={},token={};function make(){return (v,k,o)=>{assert.sameValue(this,owner);assert.sameValue(v,token);return true;};}
var predicate=make.call(owner);assert.sameValue(m.call([token],predicate,other),result(0,token));
// CASE: findLastIndex-arrow-predicate-keeps-lexical-this
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var owner={},other={},token={};function make(){return (v,k,o)=>{assert.sameValue(this,owner);assert.sameValue(v,token);return true;};}
var predicate=make.call(owner);assert.sameValue(m.call([token],predicate,other),result(0,token));
// CASE: find-bound-predicate-keeps-bound-this-and-prefix
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var owner={},ignored={},prefix={},token={},a=[token];
function predicate(p,v,k,o){'use strict';assert.sameValue(this,owner);assert.sameValue(arguments.length,4);assert.sameValue(p,prefix);assert.sameValue(v,token);assert.sameValue(k,0);assert.sameValue(o,a);return true;}
assert.sameValue(m.call(a,predicate.bind(owner,prefix),ignored),result(0,token));
// CASE: findIndex-bound-predicate-keeps-bound-this-and-prefix
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var owner={},ignored={},prefix={},token={},a=[token];
function predicate(p,v,k,o){'use strict';assert.sameValue(this,owner);assert.sameValue(arguments.length,4);assert.sameValue(p,prefix);assert.sameValue(v,token);assert.sameValue(k,0);assert.sameValue(o,a);return true;}
assert.sameValue(m.call(a,predicate.bind(owner,prefix),ignored),result(0,token));
// CASE: findLast-bound-predicate-keeps-bound-this-and-prefix
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var owner={},ignored={},prefix={},token={},a=[token];
function predicate(p,v,k,o){'use strict';assert.sameValue(this,owner);assert.sameValue(arguments.length,4);assert.sameValue(p,prefix);assert.sameValue(v,token);assert.sameValue(k,0);assert.sameValue(o,a);return true;}
assert.sameValue(m.call(a,predicate.bind(owner,prefix),ignored),result(0,token));
// CASE: findLastIndex-bound-predicate-keeps-bound-this-and-prefix
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var owner={},ignored={},prefix={},token={},a=[token];
function predicate(p,v,k,o){'use strict';assert.sameValue(this,owner);assert.sameValue(arguments.length,4);assert.sameValue(p,prefix);assert.sameValue(v,token);assert.sameValue(k,0);assert.sameValue(o,a);return true;}
assert.sameValue(m.call(a,predicate.bind(owner,prefix),ignored),result(0,token));
// CASE: find-native-predicates-use-callability-not-user-call-property
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
assert.sameValue(m.call([0,7,0],Boolean),result(1,7));assert.sameValue(m.call([Infinity,4,Infinity],Number.isFinite),result(1,4));
assert.sameValue(m.call([1,2],Function.prototype),absent);
// CASE: findIndex-native-predicates-use-callability-not-user-call-property
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
assert.sameValue(m.call([0,7,0],Boolean),result(1,7));assert.sameValue(m.call([Infinity,4,Infinity],Number.isFinite),result(1,4));
assert.sameValue(m.call([1,2],Function.prototype),absent);
// CASE: findLast-native-predicates-use-callability-not-user-call-property
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
assert.sameValue(m.call([0,7,0],Boolean),result(1,7));assert.sameValue(m.call([Infinity,4,Infinity],Number.isFinite),result(1,4));
assert.sameValue(m.call([1,2],Function.prototype),absent);
// CASE: findLastIndex-native-predicates-use-callability-not-user-call-property
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
assert.sameValue(m.call([0,7,0],Boolean),result(1,7));assert.sameValue(m.call([Infinity,4,Infinity],Number.isFinite),result(1,4));
assert.sameValue(m.call([1,2],Function.prototype),absent);
// CASE: find-predicate-call-apply-properties-are-not-read
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reads=0,predicate=function(){return true;},token={};
Object.defineProperty(predicate,'call',{get:function(){reads++;throw 'call';}});Object.defineProperty(predicate,'apply',{get:function(){reads++;throw 'apply';}});
assert.sameValue(m.call([token],predicate),result(0,token));assert.sameValue(reads,0);
// CASE: findIndex-predicate-call-apply-properties-are-not-read
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reads=0,predicate=function(){return true;},token={};
Object.defineProperty(predicate,'call',{get:function(){reads++;throw 'call';}});Object.defineProperty(predicate,'apply',{get:function(){reads++;throw 'apply';}});
assert.sameValue(m.call([token],predicate),result(0,token));assert.sameValue(reads,0);
// CASE: findLast-predicate-call-apply-properties-are-not-read
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reads=0,predicate=function(){return true;},token={};
Object.defineProperty(predicate,'call',{get:function(){reads++;throw 'call';}});Object.defineProperty(predicate,'apply',{get:function(){reads++;throw 'apply';}});
assert.sameValue(m.call([token],predicate),result(0,token));assert.sameValue(reads,0);
// CASE: findLastIndex-predicate-call-apply-properties-are-not-read
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reads=0,predicate=function(){return true;},token={};
Object.defineProperty(predicate,'call',{get:function(){reads++;throw 'call';}});Object.defineProperty(predicate,'apply',{get:function(){reads++;throw 'apply';}});
assert.sameValue(m.call([token],predicate),result(0,token));assert.sameValue(reads,0);
// CASE: find-callback-reentrancy-does-not-corrupt-outer-traversal
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=['a','b','c'],log='',innerCalls=0,token={};
assert.sameValue(m.call(a,function(v,k){log+=k;assert.sameValue(m.call([token],function(){innerCalls++;return true;}),result(0,token));return false;}),absent);
assert.sameValue(log,reverse?'210':'012');assert.sameValue(innerCalls,3);
// CASE: findIndex-callback-reentrancy-does-not-corrupt-outer-traversal
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=['a','b','c'],log='',innerCalls=0,token={};
assert.sameValue(m.call(a,function(v,k){log+=k;assert.sameValue(m.call([token],function(){innerCalls++;return true;}),result(0,token));return false;}),absent);
assert.sameValue(log,reverse?'210':'012');assert.sameValue(innerCalls,3);
// CASE: findLast-callback-reentrancy-does-not-corrupt-outer-traversal
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=['a','b','c'],log='',innerCalls=0,token={};
assert.sameValue(m.call(a,function(v,k){log+=k;assert.sameValue(m.call([token],function(){innerCalls++;return true;}),result(0,token));return false;}),absent);
assert.sameValue(log,reverse?'210':'012');assert.sameValue(innerCalls,3);
// CASE: findLastIndex-callback-reentrancy-does-not-corrupt-outer-traversal
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=['a','b','c'],log='',innerCalls=0,token={};
assert.sameValue(m.call(a,function(v,k){log+=k;assert.sameValue(m.call([token],function(){innerCalls++;return true;}),result(0,token));return false;}),absent);
assert.sameValue(log,reverse?'210':'012');assert.sameValue(innerCalls,3);
// CASE: find-length-and-index-getter-reentrancy-preserves-outer-value
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var token={},log='',o={get length(){log+='L';assert.sameValue(m.call([token],function(){return true;}),result(0,token));return 1;},get 0(){log+='G';assert.sameValue(m.call([],function(){throw 'empty';}),absent);return token;}};
assert.sameValue(m.call(o,function(){log+='C';return true;}),result(0,token));assert.sameValue(log,'LGC');
// CASE: findIndex-length-and-index-getter-reentrancy-preserves-outer-value
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var token={},log='',o={get length(){log+='L';assert.sameValue(m.call([token],function(){return true;}),result(0,token));return 1;},get 0(){log+='G';assert.sameValue(m.call([],function(){throw 'empty';}),absent);return token;}};
assert.sameValue(m.call(o,function(){log+='C';return true;}),result(0,token));assert.sameValue(log,'LGC');
// CASE: findLast-length-and-index-getter-reentrancy-preserves-outer-value
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var token={},log='',o={get length(){log+='L';assert.sameValue(m.call([token],function(){return true;}),result(0,token));return 1;},get 0(){log+='G';assert.sameValue(m.call([],function(){throw 'empty';}),absent);return token;}};
assert.sameValue(m.call(o,function(){log+='C';return true;}),result(0,token));assert.sameValue(log,'LGC');
// CASE: findLastIndex-length-and-index-getter-reentrancy-preserves-outer-value
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var token={},log='',o={get length(){log+='L';assert.sameValue(m.call([token],function(){return true;}),result(0,token));return 1;},get 0(){log+='G';assert.sameValue(m.call([],function(){throw 'empty';}),absent);return token;}};
assert.sameValue(m.call(o,function(){log+='C';return true;}),result(0,token));assert.sameValue(log,'LGC');
// CASE: find-returns-value-read-before-callback-replaces-property
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var original={},replacement={},a=[original];
assert.sameValue(m.call(a,function(v,k,o){assert.sameValue(v,original);o[k]=replacement;return true;}),result(0,original));assert.sameValue(a[0],replacement);
// CASE: findIndex-returns-value-read-before-callback-replaces-property
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var original={},replacement={},a=[original];
assert.sameValue(m.call(a,function(v,k,o){assert.sameValue(v,original);o[k]=replacement;return true;}),result(0,original));assert.sameValue(a[0],replacement);
// CASE: findLast-returns-value-read-before-callback-replaces-property
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var original={},replacement={},a=[original];
assert.sameValue(m.call(a,function(v,k,o){assert.sameValue(v,original);o[k]=replacement;return true;}),result(0,original));assert.sameValue(a[0],replacement);
// CASE: findLastIndex-returns-value-read-before-callback-replaces-property
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var original={},replacement={},a=[original];
assert.sameValue(m.call(a,function(v,k,o){assert.sameValue(v,original);o[k]=replacement;return true;}),result(0,original));assert.sameValue(a[0],replacement);
// CASE: find-self-deleting-getter-return-is-retained
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var original={},reads=0,o={length:1};Object.defineProperty(o,'0',{get:function(){reads++;delete o[0];return original;},configurable:true});
assert.sameValue(m.call(o,function(v,k){assert.sameValue(v,original);assert.sameValue(k in o,false);return true;}),result(0,original));assert.sameValue(reads,1);
// CASE: findIndex-self-deleting-getter-return-is-retained
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var original={},reads=0,o={length:1};Object.defineProperty(o,'0',{get:function(){reads++;delete o[0];return original;},configurable:true});
assert.sameValue(m.call(o,function(v,k){assert.sameValue(v,original);assert.sameValue(k in o,false);return true;}),result(0,original));assert.sameValue(reads,1);
// CASE: findLast-self-deleting-getter-return-is-retained
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var original={},reads=0,o={length:1};Object.defineProperty(o,'0',{get:function(){reads++;delete o[0];return original;},configurable:true});
assert.sameValue(m.call(o,function(v,k){assert.sameValue(v,original);assert.sameValue(k in o,false);return true;}),result(0,original));assert.sameValue(reads,1);
// CASE: findLastIndex-self-deleting-getter-return-is-retained
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var original={},reads=0,o={length:1};Object.defineProperty(o,'0',{get:function(){reads++;delete o[0];return original;},configurable:true});
assert.sameValue(m.call(o,function(v,k){assert.sameValue(v,original);assert.sameValue(k in o,false);return true;}),result(0,original));assert.sameValue(reads,1);
// CASE: find-returned-reference-observes-mutation-of-the-same-object
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var original={n:1},a=[original];var found=m.call(a,function(v){v.n=9;return true;});assert.sameValue(found,result(0,original));assert.sameValue(original.n,9);
if(!indexResult)assert.sameValue(found.n,9);
// CASE: findIndex-returned-reference-observes-mutation-of-the-same-object
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var original={n:1},a=[original];var found=m.call(a,function(v){v.n=9;return true;});assert.sameValue(found,result(0,original));assert.sameValue(original.n,9);
if(!indexResult)assert.sameValue(found.n,9);
// CASE: findLast-returned-reference-observes-mutation-of-the-same-object
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var original={n:1},a=[original];var found=m.call(a,function(v){v.n=9;return true;});assert.sameValue(found,result(0,original));assert.sameValue(original.n,9);
if(!indexResult)assert.sameValue(found.n,9);
// CASE: findLastIndex-returned-reference-observes-mutation-of-the-same-object
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var original={n:1},a=[original];var found=m.call(a,function(v){v.n=9;return true;});assert.sameValue(found,result(0,original));assert.sameValue(original.n,9);
if(!indexResult)assert.sameValue(found.n,9);
// CASE: find-samevalue-of-found-nan-negative-zero-and-symbol
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var values=[NaN,-0,0,undefined,null,Symbol('element')];
for(var i=0;i<values.length;i++){assert.sameValue(m.call([values[i]],function(){return true;}),result(0,values[i]));}
// CASE: findIndex-samevalue-of-found-nan-negative-zero-and-symbol
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var values=[NaN,-0,0,undefined,null,Symbol('element')];
for(var i=0;i<values.length;i++){assert.sameValue(m.call([values[i]],function(){return true;}),result(0,values[i]));}
// CASE: findLast-samevalue-of-found-nan-negative-zero-and-symbol
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var values=[NaN,-0,0,undefined,null,Symbol('element')];
for(var i=0;i<values.length;i++){assert.sameValue(m.call([values[i]],function(){return true;}),result(0,values[i]));}
// CASE: findLastIndex-samevalue-of-found-nan-negative-zero-and-symbol
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var values=[NaN,-0,0,undefined,null,Symbol('element')];
for(var i=0;i<values.length;i++){assert.sameValue(m.call([values[i]],function(){return true;}),result(0,values[i]));}
// CASE: find-abrupt-predicate-identity-and-prior-effects
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reasons=[{},undefined,null,NaN,Symbol('throw')];
for(var i=0;i<reasons.length;i++){var reason=reasons[i],caught=false,seen,prior=0,o={length:2},start=reverse?1:0,end=reverse?0:1;o[start]=3;
Object.defineProperty(o,end,{get:function(){throw 'should not visit';}});
try{m.call(o,function(){prior++;throw reason;});}catch(e){caught=true;seen=e;}assert.sameValue(caught,true);assert.sameValue(seen,reason);assert.sameValue(prior,1);}
// CASE: findIndex-abrupt-predicate-identity-and-prior-effects
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reasons=[{},undefined,null,NaN,Symbol('throw')];
for(var i=0;i<reasons.length;i++){var reason=reasons[i],caught=false,seen,prior=0,o={length:2},start=reverse?1:0,end=reverse?0:1;o[start]=3;
Object.defineProperty(o,end,{get:function(){throw 'should not visit';}});
try{m.call(o,function(){prior++;throw reason;});}catch(e){caught=true;seen=e;}assert.sameValue(caught,true);assert.sameValue(seen,reason);assert.sameValue(prior,1);}
// CASE: findLast-abrupt-predicate-identity-and-prior-effects
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reasons=[{},undefined,null,NaN,Symbol('throw')];
for(var i=0;i<reasons.length;i++){var reason=reasons[i],caught=false,seen,prior=0,o={length:2},start=reverse?1:0,end=reverse?0:1;o[start]=3;
Object.defineProperty(o,end,{get:function(){throw 'should not visit';}});
try{m.call(o,function(){prior++;throw reason;});}catch(e){caught=true;seen=e;}assert.sameValue(caught,true);assert.sameValue(seen,reason);assert.sameValue(prior,1);}
// CASE: findLastIndex-abrupt-predicate-identity-and-prior-effects
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reasons=[{},undefined,null,NaN,Symbol('throw')];
for(var i=0;i<reasons.length;i++){var reason=reasons[i],caught=false,seen,prior=0,o={length:2},start=reverse?1:0,end=reverse?0:1;o[start]=3;
Object.defineProperty(o,end,{get:function(){throw 'should not visit';}});
try{m.call(o,function(){prior++;throw reason;});}catch(e){caught=true;seen=e;}assert.sameValue(caught,true);assert.sameValue(seen,reason);assert.sameValue(prior,1);}
// CASE: find-abrupt-index-getter-prevents-predicate
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reason={},caught,calls=0,reads=0,o={length:2},start=reverse?1:0;
Object.defineProperty(o,start,{get:function(){reads++;throw reason;}});try{m.call(o,function(){calls++;return true;});}catch(e){caught=e;}
assert.sameValue(caught,reason);assert.sameValue(calls,0);assert.sameValue(reads,1);
// CASE: findIndex-abrupt-index-getter-prevents-predicate
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reason={},caught,calls=0,reads=0,o={length:2},start=reverse?1:0;
Object.defineProperty(o,start,{get:function(){reads++;throw reason;}});try{m.call(o,function(){calls++;return true;});}catch(e){caught=e;}
assert.sameValue(caught,reason);assert.sameValue(calls,0);assert.sameValue(reads,1);
// CASE: findLast-abrupt-index-getter-prevents-predicate
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reason={},caught,calls=0,reads=0,o={length:2},start=reverse?1:0;
Object.defineProperty(o,start,{get:function(){reads++;throw reason;}});try{m.call(o,function(){calls++;return true;});}catch(e){caught=e;}
assert.sameValue(caught,reason);assert.sameValue(calls,0);assert.sameValue(reads,1);
// CASE: findLastIndex-abrupt-index-getter-prevents-predicate
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reason={},caught,calls=0,reads=0,o={length:2},start=reverse?1:0;
Object.defineProperty(o,start,{get:function(){reads++;throw reason;}});try{m.call(o,function(){calls++;return true;});}catch(e){caught=e;}
assert.sameValue(caught,reason);assert.sameValue(calls,0);assert.sameValue(reads,1);
// CASE: find-constructor-species-and-iterator-protocols-are-unused
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reads=0,token={},a=[token];function poison(){reads++;throw 'protocol';}
Object.defineProperty(a,'constructor',{get:poison});Object.defineProperty(a,Symbol.iterator,{get:poison});Object.defineProperty(a,Symbol.species,{get:poison});
assert.sameValue(m.call(a,function(){return true;}),result(0,token));
var constructor={};Object.defineProperty(constructor,Symbol.species,{get:poison});var b=[token];b.constructor=constructor;assert.sameValue(m.call(b,function(){return true;}),result(0,token));assert.sameValue(reads,0);
// CASE: findIndex-constructor-species-and-iterator-protocols-are-unused
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reads=0,token={},a=[token];function poison(){reads++;throw 'protocol';}
Object.defineProperty(a,'constructor',{get:poison});Object.defineProperty(a,Symbol.iterator,{get:poison});Object.defineProperty(a,Symbol.species,{get:poison});
assert.sameValue(m.call(a,function(){return true;}),result(0,token));
var constructor={};Object.defineProperty(constructor,Symbol.species,{get:poison});var b=[token];b.constructor=constructor;assert.sameValue(m.call(b,function(){return true;}),result(0,token));assert.sameValue(reads,0);
// CASE: findLast-constructor-species-and-iterator-protocols-are-unused
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reads=0,token={},a=[token];function poison(){reads++;throw 'protocol';}
Object.defineProperty(a,'constructor',{get:poison});Object.defineProperty(a,Symbol.iterator,{get:poison});Object.defineProperty(a,Symbol.species,{get:poison});
assert.sameValue(m.call(a,function(){return true;}),result(0,token));
var constructor={};Object.defineProperty(constructor,Symbol.species,{get:poison});var b=[token];b.constructor=constructor;assert.sameValue(m.call(b,function(){return true;}),result(0,token));assert.sameValue(reads,0);
// CASE: findLastIndex-constructor-species-and-iterator-protocols-are-unused
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reads=0,token={},a=[token];function poison(){reads++;throw 'protocol';}
Object.defineProperty(a,'constructor',{get:poison});Object.defineProperty(a,Symbol.iterator,{get:poison});Object.defineProperty(a,Symbol.species,{get:poison});
assert.sameValue(m.call(a,function(){return true;}),result(0,token));
var constructor={};Object.defineProperty(constructor,Symbol.species,{get:poison});var b=[token];b.constructor=constructor;assert.sameValue(m.call(b,function(){return true;}),result(0,token));assert.sameValue(reads,0);
// CASE: find-readonly-and-nonextensible-receivers-need-no-writes
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=Object.freeze([1,,3]),log='';assert.sameValue(m.call(a,function(v,k){log+=k;return false;}),absent);assert.sameValue(log,reverse?'210':'012');assert.sameValue(Object.isFrozen(a),true);
var o={length:1,0:'item'};Object.freeze(o);assert.sameValue(m.call(o,function(){return true;}),result(0,'item'));assert.sameValue(Object.isFrozen(o),true);
// CASE: findIndex-readonly-and-nonextensible-receivers-need-no-writes
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=Object.freeze([1,,3]),log='';assert.sameValue(m.call(a,function(v,k){log+=k;return false;}),absent);assert.sameValue(log,reverse?'210':'012');assert.sameValue(Object.isFrozen(a),true);
var o={length:1,0:'item'};Object.freeze(o);assert.sameValue(m.call(o,function(){return true;}),result(0,'item'));assert.sameValue(Object.isFrozen(o),true);
// CASE: findLast-readonly-and-nonextensible-receivers-need-no-writes
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=Object.freeze([1,,3]),log='';assert.sameValue(m.call(a,function(v,k){log+=k;return false;}),absent);assert.sameValue(log,reverse?'210':'012');assert.sameValue(Object.isFrozen(a),true);
var o={length:1,0:'item'};Object.freeze(o);assert.sameValue(m.call(o,function(){return true;}),result(0,'item'));assert.sameValue(Object.isFrozen(o),true);
// CASE: findLastIndex-readonly-and-nonextensible-receivers-need-no-writes
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=Object.freeze([1,,3]),log='';assert.sameValue(m.call(a,function(v,k){log+=k;return false;}),absent);assert.sameValue(log,reverse?'210':'012');assert.sameValue(Object.isFrozen(a),true);
var o={length:1,0:'item'};Object.freeze(o);assert.sameValue(m.call(o,function(){return true;}),result(0,'item'));assert.sameValue(Object.isFrozen(o),true);
// CASE: find-noncanonical-and-symbol-keys-do-not-participate
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var o={length:3},reads=0;function poison(){reads++;throw 'unvisited key';}
var keys=['-1','01','1.0','3',Symbol('index')];for(var i=0;i<keys.length;i++)Object.defineProperty(o,keys[i],{get:poison});
var calls=0;assert.sameValue(m.call(o,function(v){calls++;assert.sameValue(v,undefined);return false;}),absent);assert.sameValue(calls,3);assert.sameValue(reads,0);
// CASE: findIndex-noncanonical-and-symbol-keys-do-not-participate
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var o={length:3},reads=0;function poison(){reads++;throw 'unvisited key';}
var keys=['-1','01','1.0','3',Symbol('index')];for(var i=0;i<keys.length;i++)Object.defineProperty(o,keys[i],{get:poison});
var calls=0;assert.sameValue(m.call(o,function(v){calls++;assert.sameValue(v,undefined);return false;}),absent);assert.sameValue(calls,3);assert.sameValue(reads,0);
// CASE: findLast-noncanonical-and-symbol-keys-do-not-participate
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var o={length:3},reads=0;function poison(){reads++;throw 'unvisited key';}
var keys=['-1','01','1.0','3',Symbol('index')];for(var i=0;i<keys.length;i++)Object.defineProperty(o,keys[i],{get:poison});
var calls=0;assert.sameValue(m.call(o,function(v){calls++;assert.sameValue(v,undefined);return false;}),absent);assert.sameValue(calls,3);assert.sameValue(reads,0);
// CASE: findLastIndex-noncanonical-and-symbol-keys-do-not-participate
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var o={length:3},reads=0;function poison(){reads++;throw 'unvisited key';}
var keys=['-1','01','1.0','3',Symbol('index')];for(var i=0;i<keys.length;i++)Object.defineProperty(o,keys[i],{get:poison});
var calls=0;assert.sameValue(m.call(o,function(v){calls++;assert.sameValue(v,undefined);return false;}),absent);assert.sameValue(calls,3);assert.sameValue(reads,0);
// CASE: find-huge-logical-length-first-visit-is-decisive
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var lengths=[65537,4294967295,4294967296,9007199254740991,Infinity,1e100];
for(var i=0;i<lengths.length;i++){var length=lengths[i],effective=length>9007199254740991?9007199254740991:length,start=reverse?effective-1:0,token={},o={length:length},calls=0;o[start]=token;
assert.sameValue(m.call(o,function(v,k,obj){calls++;assert.sameValue(k,start);assert.sameValue(v,token);assert.sameValue(obj,o);return true;}),result(start,token));assert.sameValue(calls,1);}
// CASE: findIndex-huge-logical-length-first-visit-is-decisive
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var lengths=[65537,4294967295,4294967296,9007199254740991,Infinity,1e100];
for(var i=0;i<lengths.length;i++){var length=lengths[i],effective=length>9007199254740991?9007199254740991:length,start=reverse?effective-1:0,token={},o={length:length},calls=0;o[start]=token;
assert.sameValue(m.call(o,function(v,k,obj){calls++;assert.sameValue(k,start);assert.sameValue(v,token);assert.sameValue(obj,o);return true;}),result(start,token));assert.sameValue(calls,1);}
// CASE: findLast-huge-logical-length-first-visit-is-decisive
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var lengths=[65537,4294967295,4294967296,9007199254740991,Infinity,1e100];
for(var i=0;i<lengths.length;i++){var length=lengths[i],effective=length>9007199254740991?9007199254740991:length,start=reverse?effective-1:0,token={},o={length:length},calls=0;o[start]=token;
assert.sameValue(m.call(o,function(v,k,obj){calls++;assert.sameValue(k,start);assert.sameValue(v,token);assert.sameValue(obj,o);return true;}),result(start,token));assert.sameValue(calls,1);}
// CASE: findLastIndex-huge-logical-length-first-visit-is-decisive
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var lengths=[65537,4294967295,4294967296,9007199254740991,Infinity,1e100];
for(var i=0;i<lengths.length;i++){var length=lengths[i],effective=length>9007199254740991?9007199254740991:length,start=reverse?effective-1:0,token={},o={length:length},calls=0;o[start]=token;
assert.sameValue(m.call(o,function(v,k,obj){calls++;assert.sameValue(k,start);assert.sameValue(v,token);assert.sameValue(obj,o);return true;}),result(start,token));assert.sameValue(calls,1);}
// CASE: find-huge-first-hole-is-read-as-undefined-without-scanning-gaps
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var o={length:Infinity},calls=0,start=reverse?9007199254740990:0;
assert.sameValue(m.call(o,function(v,k){calls++;assert.sameValue(v,undefined);assert.sameValue(k,start);return true;}),result(start,undefined));assert.sameValue(calls,1);
// CASE: findIndex-huge-first-hole-is-read-as-undefined-without-scanning-gaps
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var o={length:Infinity},calls=0,start=reverse?9007199254740990:0;
assert.sameValue(m.call(o,function(v,k){calls++;assert.sameValue(v,undefined);assert.sameValue(k,start);return true;}),result(start,undefined));assert.sameValue(calls,1);
// CASE: findLast-huge-first-hole-is-read-as-undefined-without-scanning-gaps
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var o={length:Infinity},calls=0,start=reverse?9007199254740990:0;
assert.sameValue(m.call(o,function(v,k){calls++;assert.sameValue(v,undefined);assert.sameValue(k,start);return true;}),result(start,undefined));assert.sameValue(calls,1);
// CASE: findLastIndex-huge-first-hole-is-read-as-undefined-without-scanning-gaps
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var o={length:Infinity},calls=0,start=reverse?9007199254740990:0;
assert.sameValue(m.call(o,function(v,k){calls++;assert.sameValue(v,undefined);assert.sameValue(k,start);return true;}),result(start,undefined));assert.sameValue(calls,1);
// CASE: find-huge-first-get-abrupt-preserves-identity
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reason={},caught,calls=0,o={length:Infinity},start=reverse?9007199254740990:0;
Object.defineProperty(o,start,{get:function(){throw reason;}});try{m.call(o,function(){calls++;return false;});}catch(e){caught=e;}
assert.sameValue(caught,reason);assert.sameValue(calls,0);
// CASE: findIndex-huge-first-get-abrupt-preserves-identity
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reason={},caught,calls=0,o={length:Infinity},start=reverse?9007199254740990:0;
Object.defineProperty(o,start,{get:function(){throw reason;}});try{m.call(o,function(){calls++;return false;});}catch(e){caught=e;}
assert.sameValue(caught,reason);assert.sameValue(calls,0);
// CASE: findLast-huge-first-get-abrupt-preserves-identity
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reason={},caught,calls=0,o={length:Infinity},start=reverse?9007199254740990:0;
Object.defineProperty(o,start,{get:function(){throw reason;}});try{m.call(o,function(){calls++;return false;});}catch(e){caught=e;}
assert.sameValue(caught,reason);assert.sameValue(calls,0);
// CASE: findLastIndex-huge-first-get-abrupt-preserves-identity
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var reason={},caught,calls=0,o={length:Infinity},start=reverse?9007199254740990:0;
Object.defineProperty(o,start,{get:function(){throw reason;}});try{m.call(o,function(){calls++;return false;});}catch(e){caught=e;}
assert.sameValue(caught,reason);assert.sameValue(calls,0);
// CASE: find-max-u32-sparse-array-first-hole-is-decisive
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=new Array(4294967295),calls=0,start=reverse?4294967294:0;
assert.sameValue(m.call(a,function(v,k,o){calls++;assert.sameValue(v,undefined);assert.sameValue(k,start);assert.sameValue(o,a);return true;}),result(start,undefined));assert.sameValue(calls,1);assert.sameValue(a.length,4294967295);
// CASE: findIndex-max-u32-sparse-array-first-hole-is-decisive
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=new Array(4294967295),calls=0,start=reverse?4294967294:0;
assert.sameValue(m.call(a,function(v,k,o){calls++;assert.sameValue(v,undefined);assert.sameValue(k,start);assert.sameValue(o,a);return true;}),result(start,undefined));assert.sameValue(calls,1);assert.sameValue(a.length,4294967295);
// CASE: findLast-max-u32-sparse-array-first-hole-is-decisive
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=new Array(4294967295),calls=0,start=reverse?4294967294:0;
assert.sameValue(m.call(a,function(v,k,o){calls++;assert.sameValue(v,undefined);assert.sameValue(k,start);assert.sameValue(o,a);return true;}),result(start,undefined));assert.sameValue(calls,1);assert.sameValue(a.length,4294967295);
// CASE: findLastIndex-max-u32-sparse-array-first-hole-is-decisive
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var a=new Array(4294967295),calls=0,start=reverse?4294967294:0;
assert.sameValue(m.call(a,function(v,k,o){calls++;assert.sameValue(v,undefined);assert.sameValue(k,start);assert.sameValue(o,a);return true;}),result(start,undefined));assert.sameValue(calls,1);assert.sameValue(a.length,4294967295);
// CASE: find-predicate-value-is-captured-before-length-getter-mutation
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var calls=0,original=function(){calls++;return true;},predicate=original,o={0:'item',get length(){predicate=function(){throw 'replacement called';};return 1;}};
assert.sameValue(m.call(o,predicate),result(0,'item'));assert.sameValue(calls,1);assert.notSameValue(predicate,original);
// CASE: findIndex-predicate-value-is-captured-before-length-getter-mutation
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var calls=0,original=function(){calls++;return true;},predicate=original,o={0:'item',get length(){predicate=function(){throw 'replacement called';};return 1;}};
assert.sameValue(m.call(o,predicate),result(0,'item'));assert.sameValue(calls,1);assert.notSameValue(predicate,original);
// CASE: findLast-predicate-value-is-captured-before-length-getter-mutation
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var calls=0,original=function(){calls++;return true;},predicate=original,o={0:'item',get length(){predicate=function(){throw 'replacement called';};return 1;}};
assert.sameValue(m.call(o,predicate),result(0,'item'));assert.sameValue(calls,1);assert.notSameValue(predicate,original);
// CASE: findLastIndex-predicate-value-is-captured-before-length-getter-mutation
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var calls=0,original=function(){calls++;return true;},predicate=original,o={0:'item',get length(){predicate=function(){throw 'replacement called';};return 1;}};
assert.sameValue(m.call(o,predicate),result(0,'item'));assert.sameValue(calls,1);assert.notSameValue(predicate,original);
// CASE: find-changing-callback-parameters-does-not-replace-the-selected-value
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="find",m=Array.prototype[name],reverse=false,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var original={},replacement={},third={},a=[original];
assert.sameValue(m.call(a,function(v,k,o){arguments[0]=replacement;v=third;arguments[1]=99;return true;}),result(0,original));assert.sameValue(a[0],original);
// CASE: findIndex-changing-callback-parameters-does-not-replace-the-selected-value
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findIndex",m=Array.prototype[name],reverse=false,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var original={},replacement={},third={},a=[original];
assert.sameValue(m.call(a,function(v,k,o){arguments[0]=replacement;v=third;arguments[1]=99;return true;}),result(0,original));assert.sameValue(a[0],original);
// CASE: findLast-changing-callback-parameters-does-not-replace-the-selected-value
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLast",m=Array.prototype[name],reverse=true,indexResult=false;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var original={},replacement={},third={},a=[original];
assert.sameValue(m.call(a,function(v,k,o){arguments[0]=replacement;v=third;arguments[1]=99;return true;}),result(0,original));assert.sameValue(a[0],original);
// CASE: findLastIndex-changing-callback-parameters-does-not-replace-the-selected-value
assert.sameValue(typeof Array.prototype.find,'function','find prerequisite');
assert.sameValue(typeof Array.prototype.findIndex,'function','findIndex prerequisite');
assert.sameValue(typeof Array.prototype.findLast,'function','findLast prerequisite');
assert.sameValue(typeof Array.prototype.findLastIndex,'function','findLastIndex prerequisite');
var guardToken={},guardArray=[0,guardToken,0],guardPredicate=function(v){return v===guardToken;};
assert.sameValue(guardArray.find(guardPredicate),guardToken);assert.sameValue(guardArray.findIndex(guardPredicate),1);
assert.sameValue(guardArray.findLast(guardPredicate),guardToken);assert.sameValue(guardArray.findLastIndex(guardPredicate),1);
assert.sameValue([].find(function(){throw 'empty guard';}),undefined);assert.sameValue([].findIndex(function(){throw 'empty guard';}),-1);
assert.sameValue([].findLast(function(){throw 'empty guard';}),undefined);assert.sameValue([].findLastIndex(function(){throw 'empty guard';}),-1);
var name="findLastIndex",m=Array.prototype[name],reverse=true,indexResult=true;var absent=indexResult?-1:undefined;function result(index,value){return indexResult?index:value;}
var original={},replacement={},third={},a=[original];
assert.sameValue(m.call(a,function(v,k,o){arguments[0]=replacement;v=third;arguments[1]=99;return true;}),result(0,original));assert.sameValue(a[0],original);
