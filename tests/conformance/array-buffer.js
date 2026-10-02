// Independent ArrayBuffer draft; no engine execution.

// CASE: constructor-prototype-and-static-metadata
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
verifyProperty($AB,'name',{value:'ArrayBuffer',writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty($AB,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty($AB,'prototype',{value:$bp,writable:false,enumerable:false,configurable:false},{restore:true});verifyProperty($bp,'constructor',{value:$AB,writable:true,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf($AB),Function.prototype);assert.sameValue(Object.getPrototypeOf($bp),Object.prototype);var f=$AB.isView;assert.sameValue(typeof f,'function');verifyProperty($AB,'isView',{value:f,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(f,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(f($g),false);
// CASE: getter-method-flags-and-nonconstructability
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var names=['byteLength','maxByteLength','resizable','detached'];for(var i=0;i<names.length;i++){var f=get(names[i]);verifyProperty($bp,names[i],{get:f,set:undefined,enumerable:false,configurable:true},{restore:true});verifyProperty(f,'name',{value:'get '+names[i],writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(f,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(own(f,'prototype'),false);assert.throws(TypeError,function(){new f();});}var ms=['resize','slice','transfer','transferToFixedLength'],lens=[1,2,0,0];for(var j=0;j<ms.length;j++){var m=method(ms[j]);verifyProperty($bp,ms[j],{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:ms[j],writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:lens[j],writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(own(m,'prototype'),false);assert.throws(TypeError,function(){Reflect.construct(m,[]);});}
// CASE: species-getter-is-generic-and-preserves-receiver
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var d=Object.getOwnPropertyDescriptor($AB,Symbol.species),f=d.get;assert.sameValue(typeof f,'function');verifyProperty($AB,Symbol.species,{get:f,set:undefined,enumerable:false,configurable:true},{restore:true});verifyProperty(f,'name',{value:'get [Symbol.species]',writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(f,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});var values=[undefined,null,7,false,'x',Symbol('s'),{},$AB];for(var i=0;i<values.length;i++)assert.sameValue(f.call(values[i]),values[i]);
// CASE: constructor-needs-new-before-length-or-options-hooks
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var n=0,p={valueOf:function(){n++;throw 'length';}},o={};Object.defineProperty(o,'maxByteLength',{get:function(){n++;throw 'max';}});assert.throws(TypeError,function(){$AB(p,o);});assert.throws(TypeError,function(){$AB.call({},p,o);});assert.sameValue(n,0);
// CASE: constructor-small-toindex-and-invalid-range
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var values=[undefined,null,false,true,NaN,-0,-0.75,2.9,'3'],sizes=[0,0,0,1,0,0,0,2,3];for(var i=0;i<values.length;i++){var b=new $AB(values[i]);meta(b,sizes[i],sizes[i],false,false);}var invalid=[-1,-Infinity,Infinity,9007199254740992];for(var j=0;j<invalid.length;j++)assert.throws(RangeError,function(){new $AB(invalid[j]);});assert.throws(TypeError,function(){new $AB(Symbol('length'));});
// CASE: primitive-options-are-ignored-without-boxing
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var n=0,protos=[Number.prototype,String.prototype,Boolean.prototype,Symbol.prototype];for(var i=0;i<protos.length;i++)Object.defineProperty(protos[i],'maxByteLength',{get:function(){n++;throw 'boxed option';},configurable:true});try{var options=[undefined,null,false,7,'x',Symbol('o')];for(var j=0;j<options.length;j++)meta(new $AB(2,options[j]),2,2,false,false);assert.sameValue(n,0);}finally{for(var k=0;k<protos.length;k++)delete protos[k].maxByteLength;}
// CASE: max-option-undefined-versus-explicit-zero-and-truncation
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
meta(new $AB(2,{}),2,2,false,false);meta(new $AB(2,{maxByteLength:undefined}),2,2,false,false);var zeros=[0,-0,null,false,NaN,-0.5];for(var i=0;i<zeros.length;i++)meta(new $AB(0,{maxByteLength:zeros[i]}),0,0,true,false);meta(new $AB(2,{maxByteLength:'4.9'}),2,4,true,false);meta(new $AB(1,{maxByteLength:true}),1,1,true,false);
// CASE: length-option-and-prototype-get-order
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var log='',proto={},N=(function(){}).bind(null);Object.defineProperty(N,'prototype',{get:function(){log+='P';return proto;}});var length={valueOf:function(){log+='L';return 2;}},options={};Object.defineProperty(options,'maxByteLength',{get:function(){log+='M';assert.sameValue(this,options);return {valueOf:function(){log+='V';return 4;}};}});var b=Reflect.construct($AB,[length,options],N);assert.sameValue(log,'LMVP');assert.sameValue(Object.getPrototypeOf(b),proto);meta(b,2,4,true,false);
// CASE: invalid-length-and-max-before-newtarget-prototype
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var log='',N=(function(){}).bind(null);Object.defineProperty(N,'prototype',{get:function(){log+='P';throw 'prototype';}});var o={};Object.defineProperty(o,'maxByteLength',{get:function(){log+='M';return 2;}});assert.throws(RangeError,function(){Reflect.construct($AB,[-1,o],N);});assert.sameValue(log,'');assert.throws(RangeError,function(){Reflect.construct($AB,[3,o],N);});assert.sameValue(log,'M');assert.throws(RangeError,function(){Reflect.construct($AB,[0,{maxByteLength:-1}],N);});assert.sameValue(log,'M');
// CASE: constructor-abrupt-conversions-preserve-identity-and-prefix
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var t={},log='',o={};Object.defineProperty(o,'maxByteLength',{get:function(){log+='M';throw t;}});sameThrow(t,function(){new $AB({valueOf:function(){log+='L';throw t;}},o);});assert.sameValue(log,'L');sameThrow(t,function(){new $AB({valueOf:function(){log+='l';return 0;}},o);});assert.sameValue(log,'LlM');sameThrow(t,function(){new $AB(0,{maxByteLength:{valueOf:function(){log+='V';throw t;}}});});assert.sameValue(log,'LlMV');
// CASE: inherited-max-option-is-read-once-with-original-receiver
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var proto={},options=Object.create(proto),gets=0,conversions=0;Object.defineProperty(proto,'maxByteLength',{get:function(){gets++;assert.sameValue(this,options);Object.defineProperty(options,'maxByteLength',{value:99});return {valueOf:function(){conversions++;return 5;}};}});meta(new $AB(2,options),2,5,true,false);assert.sameValue(gets,1);assert.sameValue(conversions,1);assert.sameValue(options.maxByteLength,99);
// CASE: newtarget-prototype-fallback-and-saved-constructor
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var Original=ArrayBuffer,N=(function(){}).bind(null);Object.defineProperty(N,'prototype',{value:7});try{ArrayBuffer=function(){throw 'global';};var b=Reflect.construct($AB,[3],N),c=new $AB(1);assert.sameValue(Object.getPrototypeOf(b),$bp);assert.sameValue(Object.getPrototypeOf(c),$bp);meta(b,3,3,false,false);}finally{ArrayBuffer=Original;}
// CASE: getter-brands-ignore-coercion-and-inherited-buffer
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var n=0,poison={valueOf:function(){n++;throw 'valueOf';},toString:function(){n++;throw 'toString';}};poison[Symbol.toStringTag]='ArrayBuffer';var bad=[undefined,null,0,false,'x',Symbol('x'),{},$bp,Object.create($g),poison];var names=['byteLength','maxByteLength','resizable','detached'];for(var i=0;i<names.length;i++){var f=get(names[i]);f.call($g);for(var j=0;j<bad.length;j++)assert.throws(TypeError,function(){f.call(bad[j]);});}assert.sameValue(n,0);
// CASE: ordinary-keys-shadow-metadata-without-changing-slots
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var b=new $AB(4,{maxByteLength:8}),s=Symbol('own');assert.sameValue(Object.getOwnPropertyNames(b).length,0);b[2]=7;b[0]=9;b.z=1;b.a=2;b[s]=3;Object.defineProperty(b,'byteLength',{value:99,writable:true,enumerable:true,configurable:true});assert.sameValue(Object.keys(b).join(','),'0,2,z,a,byteLength');assert.sameValue(Object.getOwnPropertySymbols(b)[0],s);assert.sameValue(b.byteLength,99);meta(b,4,8,true,false);delete b.z;b.z=5;assert.sameValue(Object.keys(b).join(','),'0,2,a,byteLength,z');assert.sameValue(JSON.stringify(new $AB(2)),'{}');
// CASE: integrity-freeze-does-not-freeze-internal-buffer-state
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var resize=resizeMethod(),transfer=transferMethod('transfer'),b=new $AB(2,{maxByteLength:6});b.note=7;Object.freeze(b);assert.sameValue(Object.isFrozen(b),true);assert.sameValue(resize.call(b,4),undefined);meta(b,4,6,true,false);assert.throws(TypeError,function(){Object.defineProperty(b,'newKey',{value:1});});var r=transfer.call(b,3);meta(r,3,6,true,false);meta(b,0,0,true,true);assert.sameValue(b.note,7);assert.sameValue(Object.isFrozen(b),true);
// CASE: isview-false-without-any-coercion-or-duck-branding
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var f=$AB.isView;assert.sameValue(typeof f,'function');var n=0,o={};Object.defineProperty(o,'buffer',{get:function(){n++;throw 'buffer';}});Object.defineProperty(o,Symbol.toStringTag,{get:function(){n++;throw 'tag';}});o.valueOf=function(){n++;throw 'valueOf';};var values=[undefined,null,0,false,'x',Symbol('x'),{},[],function(){},$g,$bp,Object.create($g),o];for(var i=0;i<values.length;i++)assert.sameValue(f.call({ignored:1},values[i],o),false);assert.sameValue(n,0);assert.throws(TypeError,function(){new f();});
// CASE: saved-method-aliases-call-apply-bind
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var resize=resizeMethod(),slice=sliceMethod(),transfer=transferMethod('transfer'),fixed=transferMethod('transferToFixedLength');$bp.resize=undefined;$bp.slice=undefined;$bp.transfer=undefined;$bp.transferToFixedLength=undefined;var b=new $AB(2,{maxByteLength:5});resize.apply(b,[3]);meta(b,3,5,true,false);var r=slice.bind(b,1)(3);meta(r,2,2,false,false);var moved=transfer.call(b,4);meta(moved,4,5,true,false);var final=fixed.apply(moved,[2]);meta(final,2,2,false,false);assert.throws(TypeError,function(){new resize();});assert.throws(TypeError,function(){new slice();});assert.throws(TypeError,function(){new transfer();});assert.throws(TypeError,function(){new fixed();});
// CASE: resize-shrink-regrow-zero-and-toindex
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var resize=resizeMethod(),b=new $AB(4,{maxByteLength:8});assert.sameValue(resize.call(b,1),undefined);meta(b,1,8,true,false);resize.call(b,6.9);meta(b,6,8,true,false);resize.call(b,undefined);meta(b,0,8,true,false);resize.call(b,8);meta(b,8,8,true,false);resize.call(b,-0.5);meta(b,0,8,true,false);
// CASE: resize-brand-and-fixed-buffer-refusal-before-conversion
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var resize=resizeMethod(),n=0,arg={valueOf:function(){n++;throw 'length';}},bad=[{},$bp,Object.create($g),new $AB(0),new $AB(2),null,undefined];for(var i=0;i<bad.length;i++)assert.throws(TypeError,function(){resize.call(bad[i],arg);});assert.sameValue(n,0);
// CASE: resize-conversion-can-reenter-before-final-length-update
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var resize=resizeMethod(),b=new $AB(5,{maxByteLength:8}),log='';resize.call(b,{valueOf:function(){log+='V';resize.call(b,1);assert.sameValue($bl.call(b),1);return 4;}});meta(b,4,8,true,false);assert.sameValue(log,'V');
// CASE: resize-detached-rab-converts-but-detached-fixed-does-not
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var resize=resizeMethod(),transfer=transferMethod('transfer'),r=new $AB(2,{maxByteLength:4}),f=new $AB(2),n=0;transfer.call(r);transfer.call(f);meta(r,0,0,true,true);meta(f,0,0,false,true);var arg={valueOf:function(){n++;return 1;}};assert.throws(TypeError,function(){resize.call(r,arg);});assert.sameValue(n,1);assert.throws(TypeError,function(){resize.call(f,arg);});assert.sameValue(n,1);assert.throws(RangeError,function(){resize.call(r,-1);});
// CASE: resize-conversion-detachment-and-abrupt-prefix
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var resize=resizeMethod(),transfer=transferMethod('transfer'),b=new $AB(3,{maxByteLength:6}),moved;assert.throws(TypeError,function(){resize.call(b,{valueOf:function(){moved=transfer.call(b);return 2;}});});meta(moved,3,6,true,false);meta(b,0,0,true,true);var t={},c=new $AB(3,{maxByteLength:6});sameThrow(t,function(){resize.call(c,{valueOf:function(){resize.call(c,1);throw t;}});});meta(c,1,6,true,false);
// CASE: resize-range-refusal-retains-earlier-author-mutation
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var resize=resizeMethod(),b=new $AB(4,{maxByteLength:6});assert.throws(RangeError,function(){resize.call(b,7);});meta(b,4,6,true,false);assert.throws(RangeError,function(){resize.call(b,{valueOf:function(){resize.call(b,2);return 7;}});});meta(b,2,6,true,false);Object.defineProperty(b,'maxByteLength',{get:function(){throw 'own max';}});resize.call(b,6);meta(b,6,6,true,false);
// CASE: slice-default-range-conversions-and-fixed-result
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var slice=sliceMethod(),b=new $AB(6,{maxByteLength:8}),starts=[undefined,1.9,-2,-Infinity,Infinity,NaN],ends=[undefined,4.9,undefined,2,undefined,0],sizes=[6,3,2,2,0,0];for(var i=0;i<starts.length;i++){var r=slice.call(b,starts[i],ends[i]);meta(r,sizes[i],sizes[i],false,false);assert.notSameValue(r,b);}meta(b,6,8,true,false);Object.defineProperty(b,'byteLength',{get:function(){throw 'own length';}});meta(slice.call(b,1,3),2,2,false,false);
// CASE: slice-brand-and-detached-refuse-before-bound-conversion
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var slice=sliceMethod(),transfer=transferMethod('transfer'),b=new $AB(2),n=0;transfer.call(b);var arg={valueOf:function(){n++;throw 'bound';}},bad=[{},$bp,Object.create($g),null,undefined,b];for(var i=0;i<bad.length;i++)assert.throws(TypeError,function(){slice.call(bad[i],arg,arg);});assert.sameValue(n,0);
// CASE: slice-bound-species-construction-order
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var slice=sliceMethod(),b=new $AB(6),log='',h={};Object.defineProperty(b,'constructor',{get:function(){log+='C';return h;}});Object.defineProperty(h,Symbol.species,{get:function(){log+='S';return C;}});function C(n){log+='N';assert.sameValue(arguments.length,1);assert.sameValue(n,3);assert.sameValue(new.target,C);return new $AB(n);}var r=slice.call(b,{valueOf:function(){log+='A';return 1;}},{valueOf:function(){log+='E';return 4;}});assert.sameValue(log,'AECSN');meta(r,3,3,false,false);
// CASE: slice-custom-resizable-larger-result-is-not-truncated
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var slice=sliceMethod(),b=new $AB(5),out=new $AB(6,{maxByteLength:9});out.marker=7;species(b,function(n){assert.sameValue(n,2);return out;});assert.sameValue(slice.call(b,1,3),out);meta(out,6,9,true,false);assert.sameValue(out.marker,7);meta(b,5,5,false,false);
// CASE: slice-default-species-intrinsic-survives-global-poison
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var slice=sliceMethod(),saved=ArrayBuffer,a=new $AB(3),b=new $AB(3),c=new $AB(3);a.constructor=undefined;species(b,null);c.constructor={};try{ArrayBuffer=function(){throw 'global';};var inputs=[a,b,c];for(var i=0;i<inputs.length;i++){var r=slice.call(inputs[i],1);assert.sameValue(Object.getPrototypeOf(r),$bp);meta(r,2,2,false,false);}}finally{ArrayBuffer=saved;}
// CASE: slice-invalid-constructor-and-species-no-coercion
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var slice=sliceMethod(),bad=[null,0,false,'x',Symbol('constructor')];for(var i=0;i<bad.length;i++){var b=new $AB(2);b.constructor=bad[i];assert.throws(TypeError,function(){slice.call(b);});}var invalid=[0,false,'x',Symbol('species'),{},()=>{}];for(var j=0;j<invalid.length;j++){var c=new $AB(2);species(c,invalid[j]);assert.throws(TypeError,function(){slice.call(c);});}var t={},d=new $AB(2);Object.defineProperty(d,'constructor',{get:function(){throw t;}});sameThrow(t,function(){slice.call(d);});
// CASE: slice-result-must-be-genuine-distinct-attached-and-large-enough
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var slice=sliceMethod(),transfer=transferMethod('transfer'),b=new $AB(4),detached=new $AB(4);transfer.call(detached);var fake={};Object.defineProperty(fake,'byteLength',{get:function(){throw 'fake length';}});var small=new $AB(1);Object.defineProperty(small,'byteLength',{value:99});var bad=[fake,Object.create($bp),b,detached,small];for(var i=0;i<bad.length;i++){species(b,function(){return bad[i];});assert.throws(TypeError,function(){slice.call(b,0,2);});}species(b,function(){return b;});assert.throws(TypeError,function(){slice.call(b,0,0);});meta(b,4,4,false,false);
// CASE: slice-start-detachment-still-runs-end-and-species
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var slice=sliceMethod(),transfer=transferMethod('transfer'),b=new $AB(4),log='',out;species(b,function(n){log+='S';assert.sameValue(n,2);out=new $AB(n);return out;});assert.throws(TypeError,function(){slice.call(b,{valueOf:function(){log+='A';transfer.call(b);return 1;}},{valueOf:function(){log+='E';return 3;}});});assert.sameValue(log,'AES');meta(b,0,0,false,true);meta(out,2,2,false,false);
// CASE: slice-captures-old-length-across-resize-in-bounds-and-species
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var slice=sliceMethod(),resize=resizeMethod(),b=new $AB(6,{maxByteLength:10}),out;var r=slice.call(b,{valueOf:function(){resize.call(b,1);return 2;}});meta(r,4,4,false,false);meta(b,1,10,true,false);resize.call(b,6);species(b,function(n){assert.sameValue(n,4);resize.call(b,0);out=new $AB(n);return out;});assert.sameValue(slice.call(b,2),out);meta(out,4,4,false,false);meta(b,0,10,true,false);
// CASE: slice-species-can-resize-result-before-result-validation
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var slice=sliceMethod(),resize=resizeMethod(),b=new $AB(4),out=new $AB(4,{maxByteLength:8});species(b,function(n){assert.sameValue(n,3);resize.call(out,2);return out;});assert.throws(TypeError,function(){slice.call(b,1);});meta(out,2,8,true,false);meta(b,4,4,false,false);species(b,function(n){resize.call(out,6);return out;});assert.sameValue(slice.call(b,1),out);meta(out,6,8,true,false);
// CASE: slice-abrupt-bound-and-species-effects-preserve-identity
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var slice=sliceMethod(),resize=resizeMethod(),b=new $AB(4,{maxByteLength:8}),t={},log='';Object.defineProperty(b,'constructor',{get:function(){log+='C';throw t;},configurable:true});sameThrow(t,function(){slice.call(b,{valueOf:function(){log+='A';throw t;}},{valueOf:function(){log+='E';return 1;}});});assert.sameValue(log,'A');sameThrow(t,function(){slice.call(b,0,{valueOf:function(){log+='E';resize.call(b,2);throw t;}});});assert.sameValue(log,'AE');meta(b,2,8,true,false);sameThrow(t,function(){slice.call(b,0,1);});assert.sameValue(log,'AEC');
// CASE: transfer-fixed-omitted-undefined-shrink-and-grow
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var names=['transfer','transferToFixedLength'];for(var i=0;i<names.length;i++){var f=transferMethod(names[i]),a=new $AB(3),b=new $AB(3),c=new $AB(3),d=new $AB(3);meta(f.call(a),3,3,false,false);meta(f.call(b,undefined),3,3,false,false);meta(f.call(c,1),1,1,false,false);meta(f.call(d,5),5,5,false,false);meta(a,0,0,false,true);meta(b,0,0,false,true);meta(c,0,0,false,true);meta(d,0,0,false,true);}
// CASE: transfer-preserves-rab-max-and-fixed-transfer-removes-it
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var transfer=transferMethod('transfer'),fixed=transferMethod('transferToFixedLength'),a=new $AB(3,{maxByteLength:6}),b=new $AB(3,{maxByteLength:6});var r=transfer.call(a,5);meta(r,5,6,true,false);meta(a,0,0,true,true);var s=fixed.call(b,8);meta(s,8,8,false,false);meta(b,0,0,true,true);
// CASE: transfer-ignores-constructor-species-and-source-prototype
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var transfer=transferMethod('transfer'),fixed=transferMethod('transferToFixedLength'),a=new $AB(2),b=new $AB(2,{maxByteLength:4}),saved=ArrayBuffer;function poison(){throw 'constructor or species';}Object.defineProperty(a,'constructor',{get:poison});Object.defineProperty(b,'constructor',{get:poison});Object.defineProperty(a,Symbol.species,{get:poison});Object.setPrototypeOf(a,null);try{ArrayBuffer=poison;var r=transfer.call(a),s=fixed.call(b);assert.sameValue(Object.getPrototypeOf(r),$bp);assert.sameValue(Object.getPrototypeOf(s),$bp);meta(r,2,2,false,false);meta(s,2,2,false,false);}finally{ArrayBuffer=saved;}
// CASE: transfer-brand-refusal-before-length-conversion
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var names=['transfer','transferToFixedLength'],n=0,arg={valueOf:function(){n++;throw 'length';}},bad=[{},$bp,Object.create($g),null,undefined,7];for(var i=0;i<names.length;i++){var f=transferMethod(names[i]);for(var j=0;j<bad.length;j++)assert.throws(TypeError,function(){f.call(bad[j],arg);});}assert.sameValue(n,0);
// CASE: transfer-detached-still-converts-explicit-length-first
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var names=['transfer','transferToFixedLength'];for(var i=0;i<names.length;i++){var f=transferMethod(names[i]),b=new $AB(2,{maxByteLength:4}),n=0;f.call(b);assert.throws(TypeError,function(){f.call(b,{valueOf:function(){n++;return 1;}});});assert.sameValue(n,1);assert.throws(RangeError,function(){f.call(b,-1);});var t={};sameThrow(t,function(){f.call(b,{valueOf:function(){throw t;}});});meta(b,0,0,true,true);}
// CASE: transfer-length-conversion-reenters-resize-before-copy
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var resize=resizeMethod(),names=['transfer','transferToFixedLength'];for(var i=0;i<names.length;i++){var f=transferMethod(names[i]),b=new $AB(4,{maxByteLength:8}),log='';var r=f.call(b,{valueOf:function(){log+='V';resize.call(b,1);return 3;}});assert.sameValue(log,'V');meta(r,3,i===0?8:3,i===0,false);meta(b,0,0,true,true);}
// CASE: transfer-reentrant-detachment-preserves-inner-result
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var names=['transfer','transferToFixedLength'];for(var i=0;i<names.length;i++){var f=transferMethod(names[i]),b=new $AB(3),inner;assert.throws(TypeError,function(){f.call(b,{valueOf:function(){inner=f.call(b,1);return 2;}});});meta(inner,1,1,false,false);meta(b,0,0,false,true);}
// CASE: transfer-range-refusal-is-transactional-after-author-effects
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var transfer=transferMethod('transfer'),fixed=transferMethod('transferToFixedLength'),resize=resizeMethod(),b=new $AB(4,{maxByteLength:6});assert.throws(RangeError,function(){transfer.call(b,7);});meta(b,4,6,true,false);assert.throws(RangeError,function(){transfer.call(b,{valueOf:function(){resize.call(b,2);return 7;}});});meta(b,2,6,true,false);var t={};sameThrow(t,function(){fixed.call(b,{valueOf:function(){resize.call(b,1);throw t;}});});meta(b,1,6,true,false);meta(fixed.call(b,7),7,7,false,false);meta(b,0,0,true,true);
// CASE: ignored-extra-arguments-are-evaluated-but-not-converted
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var resize=resizeMethod(),slice=sliceMethod(),transfer=transferMethod('transfer'),b=new $AB(3,{maxByteLength:6}),log='',extra={valueOf:function(){throw 'extra conversion';}};function value(mark,v){log+=mark;return v;}resize.call(b,value('A',2),value('B',extra));assert.sameValue(log,'AB');log='';meta(slice.call(b,value('C',0),value('D',1),value('E',extra)),1,1,false,false);assert.sameValue(log,'CDE');log='';meta(transfer.call(b,value('F',undefined),value('G',extra)),2,6,true,false);assert.sameValue(log,'FG');
// CASE: custom-prototype-retains-slot-brand-and-slice-species
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var slice=sliceMethod(),N=function(){},p={marker:7};N.prototype=p;var b=Reflect.construct($AB,[4],N);assert.sameValue(Object.getPrototypeOf(b),p);meta(b,4,4,false,false);species(b,function(n){return new $AB(n,{maxByteLength:8});});var r=slice.call(b,1,3);meta(r,2,8,true,false);assert.sameValue(Object.getPrototypeOf(r),$bp);
// CASE: tags-are-ordinary-and-detachment-preserves-own-properties
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var f=transferMethod('transfer'),b=new $AB(2,{maxByteLength:4});verifyProperty($bp,Symbol.toStringTag,{value:'ArrayBuffer',writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.prototype.toString.call($bp),'[object ArrayBuffer]');assert.sameValue(Object.prototype.toString.call(b),'[object ArrayBuffer]');Object.defineProperty(b,Symbol.toStringTag,{value:'custom',configurable:true});b.note=7;Object.preventExtensions(b);var r=f.call(b);assert.sameValue(Object.prototype.toString.call(b),'[object custom]');assert.sameValue(b.note,7);assert.sameValue(Object.isExtensible(b),false);meta(b,0,0,true,true);meta(r,2,4,true,false);assert.sameValue(own(r,'note'),false);
// CASE: prerequisite-typedarray-bytes-and-isview
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var resize=resizeMethod(),slice=sliceMethod(),transfer=transferMethod('transfer'),b=new $AB(4,{maxByteLength:8}),v=new Uint8Array(b);assert.sameValue($AB.isView(v),true);v[0]=17;v[1]=29;v[2]=43;v[3]=61;resize.call(b,2);resize.call(b,4);var grown=new Uint8Array(b);assert.sameValue(grown[0],17);assert.sameValue(grown[1],29);assert.sameValue(grown[2],0);assert.sameValue(grown[3],0);var cut=new Uint8Array(slice.call(b,1,4));assert.sameValue(cut[0],29);assert.sameValue(cut[1],0);assert.sameValue(cut[2],0);var moved=transfer.call(b,6),out=new Uint8Array(moved);assert.sameValue(out[0],17);assert.sameValue(out[1],29);assert.sameValue(out[4],0);assert.sameValue(out[5],0);assert.sameValue(v.length,0);assert.sameValue($AB.isView(v),true);
// CASE: prerequisite-dataview-bytes-and-detached-isview
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var transfer=transferMethod('transferToFixedLength'),b=new $AB(2),v=new DataView(b);assert.sameValue($AB.isView(v),true);v.setUint8(0,37);v.setUint8(1,91);var moved=transfer.call(b),out=new DataView(moved);assert.sameValue(out.getUint8(0),37);assert.sameValue(out.getUint8(1),91);assert.sameValue($AB.isView(v),true);assert.throws(TypeError,function(){v.getUint8(0);});
// CASE: prerequisite-shared-buffer-is-rejected-by-nonshared-methods
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var resize=resizeMethod(),slice=sliceMethod(),transfer=transferMethod('transfer'),fixed=transferMethod('transferToFixedLength'),b=new SharedArrayBuffer(2,{maxByteLength:4}),n=0,p={valueOf:function(){n++;throw 'length';}},fs=[resize,slice,transfer,fixed];assert.sameValue($AB.isView(b),false);var names=['byteLength','maxByteLength','resizable','detached'];for(var i=0;i<names.length;i++){var g=get(names[i]);assert.throws(TypeError,function(){g.call(b);});}for(var j=0;j<fs.length;j++)assert.throws(TypeError,function(){fs[j].call(b,p);});assert.sameValue(n,0);
// CASE: prerequisite-proxy-does-not-forward-private-brand
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var n=0,p=new Proxy($g,{get:function(){n++;throw 'get';},getPrototypeOf:function(){n++;throw 'prototype';}});assert.sameValue($AB.isView(p),false);assert.throws(TypeError,function(){$bl.call(p);});var slice=sliceMethod();assert.throws(TypeError,function(){slice.call(p);});assert.sameValue(n,0);var r=Proxy.revocable($g,{});r.revoke();assert.sameValue($AB.isView(r.proxy),false);assert.throws(TypeError,function(){$bl.call(r.proxy);});
// CASE: prerequisite-foreign-realm-constructor-species-is-not-discarded
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var slice=sliceMethod(),other=$262.createRealm().global,b=new other.ArrayBuffer(4),r=slice.call(b,1,3);assert.sameValue(Object.getPrototypeOf(r),other.ArrayBuffer.prototype);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),4);assert.notSameValue(Object.getPrototypeOf(r),$bp);var transfer=transferMethod('transfer'),m=transfer.call(b);assert.sameValue(Object.getPrototypeOf(m),$bp);assert.sameValue($bl.call(m),4);
// CASE: recursive-length-conversion-terminal-resource
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
var length={valueOf:function(){return new $AB(length);}};try{new $AB(length);}catch(e){throw new Test262Error('terminal resource was caught');}finally{throw new Test262Error('terminal resource ran finally');}throw new Test262Error('unbounded recursion completed');
// CASE: repeated-tiny-buffer-allocation-terminal-resource
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
var $AB=ArrayBuffer,$bp=$AB.prototype,$bd=Object.getOwnPropertyDescriptor($bp,'byteLength');assert.sameValue(typeof $bd.get,'function','genuine buffer getter');var $bl=$bd.get,$g=new $AB(2);assert.sameValue($bl.call($g),2);assert.sameValue(Object.getPrototypeOf($g),$bp);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function get(name){var d=Object.getOwnPropertyDescriptor($bp,name);assert.sameValue(typeof d.get,'function');return d.get;}
function meta(b,n,max,res,det){assert.sameValue($bl.call(b),n);assert.sameValue(get('maxByteLength').call(b),max);assert.sameValue(get('resizable').call(b),res);assert.sameValue(get('detached').call(b),det);}
function method(name){var f=$bp[name];assert.sameValue(typeof f,'function',name+' availability');return f;}
function resizeMethod(){var f=method('resize'),b=new $AB(1,{maxByteLength:2});assert.sameValue(f.call(b,2),undefined);assert.sameValue($bl.call(b),2);return f;}
function sliceMethod(){var f=method('slice'),b=new $AB(2),r=f.call(b,1);assert.sameValue($bl.call(r),1);assert.notSameValue(r,b);return f;}
function transferMethod(name){var f=method(name),b=new $AB(2),r=f.call(b);assert.sameValue($bl.call(r),2);assert.sameValue($bl.call(b),0);assert.sameValue(get('detached').call(b),true);return f;}
function species(b,C){var holder={};holder[Symbol.species]=C;b.constructor=holder;}
try{while(true){new $AB(1);}}catch(e){throw new Test262Error('terminal resource was caught');}finally{throw new Test262Error('terminal resource ran finally');}throw new Test262Error('unbounded allocation completed');
