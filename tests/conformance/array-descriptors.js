// CASE: initial-length-and-element-descriptors
var a=[1,,undefined],d=Object.getOwnPropertyDescriptor(a,'length');
assert.sameValue(d.value,3);assert.sameValue(d.writable,true);
assert.sameValue(d.enumerable,false);assert.sameValue(d.configurable,false);
d=Object.getOwnPropertyDescriptor(a,'0');
assert.sameValue(d.value,1);assert.sameValue(d.writable,true);
assert.sameValue(d.enumerable,true);assert.sameValue(d.configurable,true);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'1'),undefined);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'2').value,undefined);
// CASE: length-converts-twice-with-number-hint
var a=[1,2,3],log='',v={valueOf:function(){log+='v';return 2;},toString:function(){throw 'unexpected string conversion';}};
Object.defineProperty(a,'length',{value:v});
assert.sameValue(log,'vv');assert.sameValue(a.length,2);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'2'),undefined);
// CASE: length-rereads-conversion-method
var a=[1,2,3],log='',v={valueOf:function(){log+='a';v.valueOf=function(){log+='b';return 1;};return 1;}};
Object.defineProperty(a,'length',{value:v});
assert.sameValue(log,'ab');assert.sameValue(a.length,1);assert.sameValue(a[0],1);
// CASE: length-double-conversion-mismatch
var a=[1,2,3],calls=0,v={valueOf:function(){calls++;a.mark=calls;return calls==1?2:3;}};
assert.throws(RangeError,function(){Object.defineProperty(a,'length',{value:v});});
assert.sameValue(calls,2);assert.sameValue(a.mark,2);assert.sameValue(a.length,3);
assert.sameValue(a[2],3);
// CASE: length-first-conversion-is-uint32
var a=[1,2,3],calls=0,v={valueOf:function(){calls++;return calls==1?4294967297:1;}};
Object.defineProperty(a,'length',{value:v});
assert.sameValue(calls,2);assert.sameValue(a.length,1);assert.sameValue(a[0],1);
// CASE: length-conversion-abrupt-preserves-earlier-effects
var a=[1,2],calls=0,reason={},caught,v={valueOf:function(){calls++;if(calls==1){a[3]=9;return 1;}throw reason;}};
try{Object.defineProperty(a,'length',{value:v});}catch(e){caught=e;}
assert.sameValue(caught,reason);assert.sameValue(calls,2);assert.sameValue(a.length,4);
assert.sameValue(a[3],9);assert.sameValue(a[1],2);
// CASE: length-reads-old-state-after-both-conversions
var a=[1,2,3],calls=0,v={valueOf:function(){calls++;if(calls==2)a[4]=9;return 2;}};
Object.defineProperty(a,'length',{value:v});
assert.sameValue(calls,2);assert.sameValue(a.length,2);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'2'),undefined);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'4'),undefined);
// CASE: conversion-can-lock-length-before-shrink
var a=[1,2,3],calls=0,v={valueOf:function(){calls++;if(calls==1)Object.defineProperty(a,'length',{writable:false});return 1;}};
assert.throws(TypeError,function(){Object.defineProperty(a,'length',{value:v});});
assert.sameValue(calls,2);assert.sameValue(a.length,3);assert.sameValue(a[2],3);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'length').writable,false);
// CASE: readonly-length-define-converts-but-set-does-not
var a=[1,2,3],calls=0,v={valueOf:function(){calls++;return 3;}};
Object.defineProperty(a,'length',{writable:false});
Object.defineProperty(a,'length',{value:v});assert.sameValue(calls,2);
function strictSet(){'use strict';a.length=v;}
assert.throws(TypeError,strictSet);assert.sameValue(calls,2);assert.sameValue(a.length,3);
// CASE: descriptor-fields-before-length-conversion
var a=[1,2,3],log='',v={valueOf:function(){log+='n';return 1;}},d={};
Object.defineProperty(d,'enumerable',{get:function(){log+='e';return false;}});
Object.defineProperty(d,'configurable',{get:function(){log+='c';return false;}});
Object.defineProperty(d,'value',{get:function(){log+='v';return v;}});
Object.defineProperty(d,'writable',{get:function(){log+='w';return true;}});
Object.defineProperty(a,'length',d);
assert.sameValue(log,'ecvwnn');assert.sameValue(a.length,1);
// CASE: invalid-descriptor-skips-length-conversion
var a=[1,2],calls=0,v={valueOf:function(){calls++;return 0;}};
assert.throws(TypeError,function(){Object.defineProperty(a,'length',{value:v,get:function(){}});});
assert.sameValue(calls,0);assert.sameValue(a.length,2);
// CASE: invalid-lengths-preserve-array
var values=[-1,0.5,NaN,Infinity,-Infinity,4294967296],a=[7];
for(var i=0;i<values.length;i++){
 assert.throws(RangeError,function(){Object.defineProperty(a,'length',{value:values[i]});});
 assert.sameValue(a.length,1);assert.sameValue(a[0],7);
}
Object.defineProperty(a,'length',{value:-0});
assert.sameValue(a.length,0);assert.sameValue(Object.getOwnPropertyDescriptor(a,'0'),undefined);
// CASE: descending-shrink-partial-failure
var a=[0,1,2,3,4,5];
Object.defineProperty(a,'2',{configurable:false});
Object.defineProperty(a,'4',{configurable:false});
assert.throws(TypeError,function(){Object.defineProperty(a,'length',{value:1});});
assert.sameValue(a.length,5);assert.sameValue(a[4],4);assert.sameValue(a[3],3);
assert.sameValue(a[2],2);assert.sameValue(a[1],1);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'5'),undefined);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'length').writable,true);
// CASE: failed-shrink-finalizes-readonly-length
var a=[0,1,2,3,4];Object.defineProperty(a,'2',{configurable:false});
assert.throws(TypeError,function(){Object.defineProperty(a,'length',{value:1,writable:false});});
assert.sameValue(a.length,3);assert.sameValue(a[1],1);assert.sameValue(a[2],2);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'3'),undefined);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'4'),undefined);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'length').writable,false);
assert.throws(TypeError,function(){Object.defineProperty(a,'length',{writable:true});});
// CASE: shrink-does-not-call-index-accessors
var a=[],calls=0;
Object.defineProperty(a,'5',{get:function(){calls++;throw 'getter';},configurable:true});
Object.defineProperty(a,'3',{get:function(){calls++;throw 'getter';},configurable:false});
assert.sameValue(a.length,6);
assert.throws(TypeError,function(){Object.defineProperty(a,'length',{value:1});});
assert.sameValue(calls,0);assert.sameValue(a.length,4);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'5'),undefined);
assert.sameValue(typeof Object.getOwnPropertyDescriptor(a,'3').get,'function');
// CASE: sparse-u32-index-and-ordinary-boundary
var a=[];
Object.defineProperty(a,'4294967294',{value:'index',writable:true,enumerable:true,configurable:true});
assert.sameValue(a.length,4294967295);assert.sameValue(a[4294967294],'index');
Object.defineProperty(a,'4294967295',{value:'ordinary',writable:true,enumerable:true,configurable:true});
assert.sameValue(a.length,4294967295);
Object.defineProperty(a,'length',{value:0});
assert.sameValue(a.length,0);assert.sameValue(a[4294967295],'ordinary');
assert.sameValue(Object.getOwnPropertyDescriptor(a,'4294967294'),undefined);
// CASE: maximum-logical-length-with-holes
var a=new Array(4294967295);
assert.sameValue(a.length,4294967295);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'4294967294'),undefined);
Object.defineProperty(a,'length',{value:0});assert.sameValue(a.length,0);
// CASE: noncanonical-index-names-and-symbols
var a=[],s=Symbol('element');a['01']=1;a['1.0']=2;a['-0']=3;a['4294967295']=4;a[s]=5;
assert.sameValue(a.length,0);a[0]=6;assert.sameValue(a.length,1);
a.length=0;assert.sameValue(a['01'],1);assert.sameValue(a['1.0'],2);
assert.sameValue(a['-0'],3);assert.sameValue(a['4294967295'],4);assert.sameValue(a[s],5);
// CASE: indexed-accessor-receiver-and-descriptor
var a=[],seen,setValue,get=function(){seen=this;return 7;},set=function(v){seen=this;setValue=v;};
Object.defineProperty(a,'2',{get:get,set:set,enumerable:false,configurable:true});
assert.sameValue(a.length,3);assert.sameValue(a[2],7);assert.sameValue(seen,a);
a[2]=9;assert.sameValue(setValue,9);assert.sameValue(seen,a);
var d=Object.getOwnPropertyDescriptor(a,'2');assert.sameValue(d.get,get);assert.sameValue(d.set,set);
assert.sameValue(d.enumerable,false);assert.sameValue(d.configurable,true);assert.sameValue(d.value,undefined);
assert.sameValue(delete a[2],true);assert.sameValue(a.length,3);
// CASE: inherited-index-setter-does-not-grow-receiver
var p=[],a=[],seen,value;
Object.defineProperty(p,'5',{set:function(v){seen=this;value=v;},configurable:true});
Object.setPrototypeOf(a,p);a[5]=11;
assert.sameValue(seen,a);assert.sameValue(value,11);assert.sameValue(a.length,0);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'5'),undefined);assert.sameValue(p.length,6);
// CASE: inherited-readonly-index-refuses-own-definition-by-assignment
var p=[],a=[];Object.defineProperty(p,'4',{value:7,writable:false});Object.setPrototypeOf(a,p);
if((function(){return this;})()===undefined){assert.throws(TypeError,function(){a[4]=9;});}
else{a[4]=9;}
assert.sameValue(a[4],7);assert.sameValue(a.length,0);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'4'),undefined);
Object.defineProperty(a,'4',{value:8});assert.sameValue(a[4],8);assert.sameValue(a.length,5);
// CASE: readonly-length-allows-below-length-properties
var a=[1,,3];Object.defineProperty(a,'length',{writable:false});
Object.defineProperty(a,'1',{value:2,writable:true,configurable:true});a[0]=9;
assert.sameValue(a.length,3);assert.sameValue(a[0],9);assert.sameValue(a[1],2);
assert.throws(TypeError,function(){Object.defineProperty(a,'3',{value:4});});
assert.sameValue(Object.getOwnPropertyDescriptor(a,'3'),undefined);
Object.defineProperty(a,'4294967295',{value:7});assert.sameValue(a[4294967295],7);
// CASE: nonextensible-array-existing-updates-and-length-only-growth
var a=[1,,3];Object.preventExtensions(a);assert.sameValue(Object.isExtensible(a),false);
a[0]=9;assert.sameValue(a[0],9);
assert.throws(TypeError,function(){Object.defineProperty(a,'1',{value:2});});
assert.throws(TypeError,function(){Object.defineProperty(a,'3',{value:4});});
assert.sameValue(a.length,3);assert.sameValue(Object.getOwnPropertyDescriptor(a,'1'),undefined);
Object.defineProperty(a,'length',{value:5});assert.sameValue(a.length,5);
assert.sameValue(delete a[2],true);assert.sameValue(a.length,5);
assert.throws(TypeError,function(){Object.defineProperty(a,'2',{value:3});});
// CASE: index-data-accessor-data-transitions
var a=[];Object.defineProperty(a,'0',{value:1,writable:true,enumerable:true,configurable:true});
var get=function(){return 4;};Object.defineProperty(a,'0',{get:get});
var d=Object.getOwnPropertyDescriptor(a,'0');assert.sameValue(d.get,get);assert.sameValue(d.set,undefined);
assert.sameValue(d.enumerable,true);assert.sameValue(d.configurable,true);assert.sameValue(a[0],4);
Object.defineProperty(a,'0',{value:8});d=Object.getOwnPropertyDescriptor(a,'0');
assert.sameValue(d.value,8);assert.sameValue(d.writable,false);assert.sameValue(d.enumerable,true);
assert.sameValue(d.configurable,true);assert.sameValue(d.get,undefined);assert.sameValue(a.length,1);
// CASE: nonconfigurable-index-samevalue-and-delete
var a=[];Object.defineProperty(a,'0',{value:NaN});Object.defineProperty(a,'0',{value:0/0});
assert.sameValue(a.length,1);assert.sameValue(a[0],NaN);
assert.throws(TypeError,function(){Object.defineProperty(a,'0',{value:1});});
assert.throws(TypeError,function(){Object.defineProperty(a,'0',{writable:true});});
if((function(){return this;})()===undefined){assert.throws(TypeError,function(){delete a[0];});}
else{assert.sameValue(delete a[0],false);}
assert.sameValue(Object.getOwnPropertyDescriptor(a,'0').configurable,false);
Object.defineProperty(a,'1',{value:-0});
assert.throws(TypeError,function(){Object.defineProperty(a,'1',{value:0});});assert.sameValue(a[1],-0);
// CASE: holes-read-through-prototype-but-shrink-deletes-only-own
var p={1:8,4:9},a=[1,,3];Object.setPrototypeOf(a,p);
assert.sameValue(a[1],8);assert.sameValue(Object.getOwnPropertyDescriptor(a,'1'),undefined);
a.length=1;assert.sameValue(a[1],8);assert.sameValue(a[4],9);assert.sameValue(p[4],9);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'2'),undefined);
// CASE: json-reviver-replaces-configurable-accessor-with-data
var reads=0,writes=0,seen='',a=JSON.parse('[1,2]',function(k,v){
 if(k==='0'){Object.defineProperty(this,'1',{get:function(){reads++;return 7;},set:function(){writes++;throw 'setter';},enumerable:false,configurable:true});}
 if(k==='1'){seen=v;return 9;}return v;
});
assert.sameValue(reads,1);assert.sameValue(writes,0);assert.sameValue(seen,7);assert.sameValue(a[1],9);
var d=Object.getOwnPropertyDescriptor(a,'1');assert.sameValue(d.value,9);assert.sameValue(d.writable,true);
assert.sameValue(d.enumerable,true);assert.sameValue(d.configurable,true);assert.sameValue(d.get,undefined);
// CASE: json-reviver-ignores-incompatible-data-redefinition
var visits='',a=JSON.parse('[1,2]',function(k,v){
 if(k==='0'){Object.defineProperty(this,'1',{value:8,writable:false,configurable:false});}
 if(k==='1'){visits+=v;return 9;}return v;
});
assert.sameValue(visits,'8');assert.sameValue(a[1],8);assert.sameValue(a.length,2);
var d=Object.getOwnPropertyDescriptor(a,'1');assert.sameValue(d.writable,false);assert.sameValue(d.configurable,false);
// CASE: json-reviver-ignores-failed-delete
var a=JSON.parse('[1,2]',function(k,v){if(k==='0'){Object.defineProperty(this,'1',{configurable:false});}if(k==='1')return undefined;return v;});
assert.sameValue(a[1],2);assert.sameValue(a.length,2);assert.sameValue(Object.getOwnPropertyDescriptor(a,'1').configurable,false);
// CASE: json-reviver-cannot-recreate-nonextensible-hole
var a=JSON.parse('[1,2]',function(k,v){if(k==='0'){delete this[0];Object.preventExtensions(this);return 9;}return v;});
assert.sameValue(a.length,2);assert.sameValue(Object.getOwnPropertyDescriptor(a,'0'),undefined);
assert.sameValue(a[1],2);assert.sameValue(Object.isExtensible(a),false);
// CASE: json-reviver-keeps-saved-length-after-readonly-shrink
var log='',a=JSON.parse('[1,2]',function(k,v){
 if(k==='0'){log+='0';Object.defineProperty(this,'length',{value:0,writable:false});return 8;}
 if(k==='1'){log+='1';assert.sameValue(v,undefined);return 9;}return v;
});
assert.sameValue(log,'01');assert.sameValue(a.length,0);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'0'),undefined);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'1'),undefined);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'length').writable,false);
// CASE: push-and-pop-respect-descriptor-failures
var a=[1];Object.defineProperty(a,'length',{writable:false});
assert.throws(TypeError,function(){a.push(2);});assert.sameValue(a.length,1);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'1'),undefined);
var b=[1,2];Object.defineProperty(b,'1',{configurable:false});
assert.throws(TypeError,function(){b.pop();});assert.sameValue(b.length,2);assert.sameValue(b[1],2);
// CASE: reverse-uses-live-index-accessors
var a=[1,2,3],log='',value=1;
Object.defineProperty(a,'0',{get:function(){log+='g';return value;},set:function(v){log+='s';value=v;},configurable:true});
assert.sameValue(a.reverse(),a);assert.sameValue(log,'gs');assert.sameValue(value,3);
assert.sameValue(a[2],1);assert.sameValue(a.length,3);
// CASE: sort-copyback-failure-preserves-earlier-writes
var a=[3,2,1];Object.defineProperty(a,'1',{writable:false});
assert.throws(TypeError,function(){a.sort();});assert.sameValue(a[0],1);
assert.sameValue(a[1],2);assert.sameValue(a[2],1);assert.sameValue(a.length,3);
// CASE: map-sees-live-inherited-and-own-accessors
var p={},a=[1,,3],log='';Object.setPrototypeOf(p,Array.prototype);Object.setPrototypeOf(a,p);
Object.defineProperty(p,'1',{get:function(){assert.sameValue(this,a);log+='g';return 8;},configurable:true});
var b=a.map(function(v,k){log+=k;if(k===0)Object.defineProperty(a,'2',{get:function(){log+='h';return 9;},configurable:true});return v*2;});
assert.sameValue(log,'0g1h2');assert.sameValue(b.length,3);
assert.sameValue(b[0],2);assert.sameValue(b[1],16);assert.sameValue(b[2],18);
// CASE: json-stringify-reads-nonenumerable-array-index
var a=[1,2],calls=0;Object.defineProperty(a,'1',{get:function(){calls++;return 7;},enumerable:false,configurable:true});
assert.sameValue(JSON.stringify(a),'[1,7]');assert.sameValue(calls,1);
