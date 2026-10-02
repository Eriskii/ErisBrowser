// Independent DataView draft; no engine execution.

// CASE: constructor-prototype-and-tag-metadata
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
verifyProperty($DV,'name',{value:'DataView',writable:false,enumerable:false,configurable:true},{restore:true});
verifyProperty($DV,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});
verifyProperty($DV,'prototype',{value:$dp,writable:false,enumerable:false,configurable:false},{restore:true});
verifyProperty($dp,'constructor',{value:$DV,writable:true,enumerable:false,configurable:true},{restore:true});
verifyProperty($dp,Symbol.toStringTag,{value:'DataView',writable:false,enumerable:false,configurable:true},{restore:true});
assert.sameValue(Object.getPrototypeOf($DV),Function.prototype);assert.sameValue(Object.getPrototypeOf($dp),Object.prototype);
assert.sameValue(Object.prototype.toString.call($dp),'[object DataView]');assert.sameValue(Object.prototype.toString.call($view),'[object DataView]');
assert.sameValue(own($DV,Symbol.species),false);
// CASE: accessor-buffer-metadata
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var k="buffer",f=getter(k);verifyProperty($dp,k,{get:f,set:undefined,enumerable:false,configurable:true},{restore:true});
verifyProperty(f,'name',{value:'get '+k,writable:false,enumerable:false,configurable:true},{restore:true});
verifyProperty(f,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});
assert.sameValue(own(f,'prototype'),false);assert.throws(TypeError,function(){Reflect.construct(f,[]);});
// CASE: accessor-byteLength-metadata
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var k="byteLength",f=getter(k);verifyProperty($dp,k,{get:f,set:undefined,enumerable:false,configurable:true},{restore:true});
verifyProperty(f,'name',{value:'get '+k,writable:false,enumerable:false,configurable:true},{restore:true});
verifyProperty(f,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});
assert.sameValue(own(f,'prototype'),false);assert.throws(TypeError,function(){Reflect.construct(f,[]);});
// CASE: accessor-byteOffset-metadata
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var k="byteOffset",f=getter(k);verifyProperty($dp,k,{get:f,set:undefined,enumerable:false,configurable:true},{restore:true});
verifyProperty(f,'name',{value:'get '+k,writable:false,enumerable:false,configurable:true},{restore:true});
verifyProperty(f,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});
assert.sameValue(own(f,'prototype'),false);assert.throws(TypeError,function(){Reflect.construct(f,[]);});
// CASE: method-pair-Int8-metadata
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var names=['getInt8','setInt8'],lens=[1,2];for(var i=0;i<2;i++){var k=names[i],f=method(k);
verifyProperty($dp,k,{value:f,writable:true,enumerable:false,configurable:true},{restore:true});
verifyProperty(f,'name',{value:k,writable:false,enumerable:false,configurable:true},{restore:true});
verifyProperty(f,'length',{value:lens[i],writable:false,enumerable:false,configurable:true},{restore:true});
assert.sameValue(own(f,'prototype'),false);assert.throws(TypeError,function(){Reflect.construct(f,[]);});}
// CASE: method-pair-Uint8-metadata
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var names=['getUint8','setUint8'],lens=[1,2];for(var i=0;i<2;i++){var k=names[i],f=method(k);
verifyProperty($dp,k,{value:f,writable:true,enumerable:false,configurable:true},{restore:true});
verifyProperty(f,'name',{value:k,writable:false,enumerable:false,configurable:true},{restore:true});
verifyProperty(f,'length',{value:lens[i],writable:false,enumerable:false,configurable:true},{restore:true});
assert.sameValue(own(f,'prototype'),false);assert.throws(TypeError,function(){Reflect.construct(f,[]);});}
// CASE: method-pair-Int16-metadata
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var names=['getInt16','setInt16'],lens=[1,2];for(var i=0;i<2;i++){var k=names[i],f=method(k);
verifyProperty($dp,k,{value:f,writable:true,enumerable:false,configurable:true},{restore:true});
verifyProperty(f,'name',{value:k,writable:false,enumerable:false,configurable:true},{restore:true});
verifyProperty(f,'length',{value:lens[i],writable:false,enumerable:false,configurable:true},{restore:true});
assert.sameValue(own(f,'prototype'),false);assert.throws(TypeError,function(){Reflect.construct(f,[]);});}
// CASE: method-pair-Uint16-metadata
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var names=['getUint16','setUint16'],lens=[1,2];for(var i=0;i<2;i++){var k=names[i],f=method(k);
verifyProperty($dp,k,{value:f,writable:true,enumerable:false,configurable:true},{restore:true});
verifyProperty(f,'name',{value:k,writable:false,enumerable:false,configurable:true},{restore:true});
verifyProperty(f,'length',{value:lens[i],writable:false,enumerable:false,configurable:true},{restore:true});
assert.sameValue(own(f,'prototype'),false);assert.throws(TypeError,function(){Reflect.construct(f,[]);});}
// CASE: method-pair-Int32-metadata
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var names=['getInt32','setInt32'],lens=[1,2];for(var i=0;i<2;i++){var k=names[i],f=method(k);
verifyProperty($dp,k,{value:f,writable:true,enumerable:false,configurable:true},{restore:true});
verifyProperty(f,'name',{value:k,writable:false,enumerable:false,configurable:true},{restore:true});
verifyProperty(f,'length',{value:lens[i],writable:false,enumerable:false,configurable:true},{restore:true});
assert.sameValue(own(f,'prototype'),false);assert.throws(TypeError,function(){Reflect.construct(f,[]);});}
// CASE: method-pair-Uint32-metadata
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var names=['getUint32','setUint32'],lens=[1,2];for(var i=0;i<2;i++){var k=names[i],f=method(k);
verifyProperty($dp,k,{value:f,writable:true,enumerable:false,configurable:true},{restore:true});
verifyProperty(f,'name',{value:k,writable:false,enumerable:false,configurable:true},{restore:true});
verifyProperty(f,'length',{value:lens[i],writable:false,enumerable:false,configurable:true},{restore:true});
assert.sameValue(own(f,'prototype'),false);assert.throws(TypeError,function(){Reflect.construct(f,[]);});}
// CASE: method-pair-Float16-metadata
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var names=['getFloat16','setFloat16'],lens=[1,2];for(var i=0;i<2;i++){var k=names[i],f=method(k);
verifyProperty($dp,k,{value:f,writable:true,enumerable:false,configurable:true},{restore:true});
verifyProperty(f,'name',{value:k,writable:false,enumerable:false,configurable:true},{restore:true});
verifyProperty(f,'length',{value:lens[i],writable:false,enumerable:false,configurable:true},{restore:true});
assert.sameValue(own(f,'prototype'),false);assert.throws(TypeError,function(){Reflect.construct(f,[]);});}
// CASE: method-pair-Float32-metadata
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var names=['getFloat32','setFloat32'],lens=[1,2];for(var i=0;i<2;i++){var k=names[i],f=method(k);
verifyProperty($dp,k,{value:f,writable:true,enumerable:false,configurable:true},{restore:true});
verifyProperty(f,'name',{value:k,writable:false,enumerable:false,configurable:true},{restore:true});
verifyProperty(f,'length',{value:lens[i],writable:false,enumerable:false,configurable:true},{restore:true});
assert.sameValue(own(f,'prototype'),false);assert.throws(TypeError,function(){Reflect.construct(f,[]);});}
// CASE: method-pair-Float64-metadata
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var names=['getFloat64','setFloat64'],lens=[1,2];for(var i=0;i<2;i++){var k=names[i],f=method(k);
verifyProperty($dp,k,{value:f,writable:true,enumerable:false,configurable:true},{restore:true});
verifyProperty(f,'name',{value:k,writable:false,enumerable:false,configurable:true},{restore:true});
verifyProperty(f,'length',{value:lens[i],writable:false,enumerable:false,configurable:true},{restore:true});
assert.sameValue(own(f,'prototype'),false);assert.throws(TypeError,function(){Reflect.construct(f,[]);});}
// CASE: constructor-new-and-buffer-brand-before-index-hooks
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var n=0,arg={valueOf:function(){n++;throw 'conversion';}},fake={};Object.defineProperty(fake,'byteLength',{get:function(){n++;throw 'length';}});
assert.throws(TypeError,function(){$DV($buf,arg,arg);});
var bad=[undefined,null,0,false,'x',{},$view,$AB.prototype,Object.create($buf),fake];
for(var i=0;i<bad.length;i++)assert.throws(TypeError,function(){new $DV(bad[i],arg,arg);});assert.sameValue(n,0);
// CASE: constructor-omitted-undefined-zero-and-end-offset
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(4),r=new $AB(4,{maxByteLength:8});
meta(new $DV(b),b,0,4);meta(new $DV(b,1),b,1,3);meta(new $DV(b,1,undefined),b,1,3);meta(new $DV(b,4),b,4,0);meta(new $DV(b,4,0),b,4,0);
var a=new $DV(r,1),u=new $DV(r,1,undefined),z=new $DV(r,1,0),f=new $DV(r,1,2);r.resize(7);
meta(a,r,1,6);meta(u,r,1,6);meta(z,r,1,0);meta(f,r,1,2);assert.throws(RangeError,function(){new $DV(b,5);});
// CASE: constructor-toindex-small-and-invalid-ranges
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(6),inputs=[undefined,null,false,true,NaN,-0,-0.75,2.9,'3'],wanted=[0,0,0,1,0,0,0,2,3];
for(var i=0;i<inputs.length;i++){meta(new $DV(b,inputs[i]),b,wanted[i],6-wanted[i]);meta(new $DV(b,0,inputs[i]),b,0,inputs[i]===undefined?6:wanted[i]);}
var invalid=[-1,-Infinity,Infinity,9007199254740992];for(var j=0;j<invalid.length;j++){assert.throws(RangeError,function(){new $DV(b,invalid[j]);});assert.throws(RangeError,function(){new $DV(b,0,invalid[j]);});}
assert.throws(TypeError,function(){new $DV(b,Symbol('offset'));});assert.throws(TypeError,function(){new $DV(b,0,Symbol('length'));});
// CASE: constructor-offset-length-prototype-order-and-custom-brand
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var log='',b=new $AB(6),p={marker:9},N=(function(){}).bind(null);Object.defineProperty(N,'prototype',{get:function(){log+='P';return p;}});
var v=Reflect.construct($DV,[b,{valueOf:function(){log+='O';return 1;}},{valueOf:function(){log+='L';return 3;}}],N);
assert.sameValue(log,'OLP');assert.sameValue(Object.getPrototypeOf(v),p);meta(v,b,1,3);assert.sameValue($AB.isView(v),true);
method('setUint8').call(v,0,61);assert.sameValue(new $DV(b).getUint8(1),61);
// CASE: constructor-detached-offset-conversion-before-refusal
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(4),log='',N=(function(){}).bind(null);b.transfer();Object.defineProperty(N,'prototype',{get:function(){log+='P';return {};}});
assert.throws(TypeError,function(){Reflect.construct($DV,[b,{valueOf:function(){log+='O';return 0;}},{valueOf:function(){log+='L';return 0;}}],N);});assert.sameValue(log,'O');
assert.throws(RangeError,function(){new $DV(b,-1);});var t={};sameThrow(t,function(){new $DV(b,{valueOf:function(){throw t;}});});
// CASE: constructor-length-detachment-precedes-throwing-prototype
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(4),log='',t={},N=(function(){}).bind(null);Object.defineProperty(N,'prototype',{get:function(){log+='P';throw t;}});
sameThrow(t,function(){Reflect.construct($DV,[b,1,{valueOf:function(){log+='L';b.transfer();return 2;}}],N);});
assert.sameValue(log,'LP');assert.sameValue(b.detached,true);
// CASE: constructor-captured-length-growth-does-not-rescue-overflow
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(4,{maxByteLength:8}),log='',N=(function(){}).bind(null);Object.defineProperty(N,'prototype',{get:function(){log+='P';return {};}});
assert.throws(RangeError,function(){Reflect.construct($DV,[b,1,{valueOf:function(){log+='L';b.resize(8);return 4;}}],N);});
assert.sameValue(log,'L');assert.sameValue(b.byteLength,8);
// CASE: constructor-fresh-check-after-prototype-resize-or-detach
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(6,{maxByteLength:8}),N=(function(){}).bind(null),n=0;Object.defineProperty(N,'prototype',{get:function(){n++;b.resize(2);return {};}});
assert.throws(RangeError,function(){Reflect.construct($DV,[b,1,3],N);});assert.sameValue(n,1);assert.sameValue(b.byteLength,2);
var c=new $AB(4),M=(function(){}).bind(null);Object.defineProperty(M,'prototype',{get:function(){c.transfer();return {};}});
assert.throws(TypeError,function(){Reflect.construct($DV,[c,0,2],M);});assert.sameValue(c.detached,true);
// CASE: constructor-tracking-uses-fresh-length-after-prototype
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(4,{maxByteLength:8}),N=(function(){}).bind(null);Object.defineProperty(N,'prototype',{get:function(){b.resize(7);return $dp;}});
var v=Reflect.construct($DV,[b,1],N);meta(v,b,1,6);b.resize(3);meta(v,b,1,2);
// CASE: constructor-poisoned-species-and-global-alias-are-ignored
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(4),saved=DataView,N=(function(){}).bind(null);Object.defineProperty(N,'prototype',{value:7});
Object.defineProperty(b,'constructor',{get:function(){throw 'buffer constructor';}});Object.defineProperty($DV,Symbol.species,{get:function(){throw 'species';},configurable:true});
try{DataView=function(){throw 'global';};var v=Reflect.construct($DV,[b,1,2],N);assert.sameValue(Object.getPrototypeOf(v),$dp);meta(v,b,1,2);}finally{DataView=saved;delete $DV[Symbol.species];}
// CASE: constructor-callback-view-table-growth-keeps-outer-identity
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(6,{maxByteLength:8}),N=(function(){}).bind(null),saved=[],p={};Object.defineProperty(N,'prototype',{get:function(){for(var i=0;i<32;i++)saved.push(new $DV(new $AB(1)));b.resize(7);return p;}});
var v=Reflect.construct($DV,[b,{valueOf:function(){for(var i=0;i<16;i++)saved.push(new $DV(b,0,1));return 2;}},3],N);
assert.sameValue(saved.length,48);assert.sameValue(Object.getPrototypeOf(v),p);meta(v,b,2,3);method('setUint8').call(v,0,79);assert.sameValue(new $DV(b).getUint8(2),79);
// CASE: accessor-brands-ignore-prototype-and-own-shadows
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var bad=[undefined,null,0,false,'x',{},$dp,Object.create($view)],names=['buffer','byteLength','byteOffset'];
for(var i=0;i<names.length;i++){var f=getter(names[i]);f.call($view);for(var j=0;j<bad.length;j++)assert.throws(TypeError,function(){f.call(bad[j]);});}
var b=new $AB(5),v=new $DV(b,1,3);Object.defineProperty(v,'buffer',{value:{}});Object.defineProperty(v,'byteOffset',{value:99});Object.defineProperty(v,'byteLength',{value:99});meta(v,b,1,3);
// CASE: method-brands-precede-index-and-value-conversion
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var n=0,arg={valueOf:function(){n++;throw 'conversion';}},g=method('getUint16'),s=method('setUint16');g.call($view,0);s.call($view,0,1);
var bad=[undefined,null,0,false,'x',{},$dp,Object.create($view)];for(var i=0;i<bad.length;i++){assert.throws(TypeError,function(){g.call(bad[i],arg,arg);});assert.throws(TypeError,function(){s.call(bad[i],arg,arg,arg);});}assert.sameValue(n,0);
// CASE: ordinary-properties-and-integrity-do-not-freeze-bytes
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(4),v=new $DV(b);assert.sameValue(Object.getOwnPropertyNames(v).length,0);v[1]=81;v[0]=17;v.note=9;
assert.sameValue(Object.keys(v).join(','),'0,1,note');assert.sameValue(v.getUint8(0),0);Object.freeze(v);assert.sameValue(Object.isFrozen(v),true);
assert.sameValue(v.setUint16(1,4660),undefined);bytes(v,0,[0,18,52,0]);assert.sameValue(v[0],17);assert.sameValue(v[1],81);assert.throws(TypeError,function(){Object.defineProperty(v,'extra',{value:1});});
assert.sameValue($AB.isView(v),true);assert.sameValue($AB.isView(Object.create(v)),false);
// CASE: saved-methods-call-apply-bind-and-prototype-change
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(6),v=new $DV(b,1,4),s=method('setUint16'),g=method('getUint16');$dp.setUint16=undefined;$dp.getUint16=undefined;Object.setPrototypeOf(v,null);
assert.sameValue(s.apply(v,[1,4660,true]),undefined);assert.sameValue(g.bind(v,1)(true),4660);assert.sameValue(g.call(v,1,false),13330);meta(v,b,1,4);assert.sameValue($AB.isView(v),true);
// CASE: literal-Int8-bytes-endians-and-unaligned-boundary
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(1+4),all=new $DV(b),v=new $DV(b,1,1+2),g=method('getInt8'),s=method('setInt8');
seed(all,0,[165, 165, 165, 165, 165]);seed(v,1,[254]);assert.sameValue(g.call(v,1),-2);assert.sameValue(g.call(v,1,false),-2);
seed(v,1,[254]);assert.sameValue(g.call(v,1,true),-2);
assert.sameValue(s.call(v,1,-2.9),undefined);bytes(v,1,[254]);assert.sameValue(all.getUint8(0),165);assert.sameValue(v.getUint8(0),165);assert.sameValue(v.getUint8(1+1),165);assert.sameValue(all.getUint8(1+3),165);
assert.sameValue(s.call(v,1,-2.9,true),undefined);bytes(v,1,[254]);s.call(v,1,257.9);bytes(v,1,[1]);
assert.throws(RangeError,function(){g.call(v,3);});assert.throws(RangeError,function(){s.call(v,3,0);});bytes(v,1,[1]);
// CASE: literal-Uint8-bytes-endians-and-unaligned-boundary
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(1+4),all=new $DV(b),v=new $DV(b,1,1+2),g=method('getUint8'),s=method('setUint8');
seed(all,0,[165, 165, 165, 165, 165]);seed(v,1,[254]);assert.sameValue(g.call(v,1),254);assert.sameValue(g.call(v,1,false),254);
seed(v,1,[254]);assert.sameValue(g.call(v,1,true),254);
assert.sameValue(s.call(v,1,-2.9),undefined);bytes(v,1,[254]);assert.sameValue(all.getUint8(0),165);assert.sameValue(v.getUint8(0),165);assert.sameValue(v.getUint8(1+1),165);assert.sameValue(all.getUint8(1+3),165);
assert.sameValue(s.call(v,1,-2.9,true),undefined);bytes(v,1,[254]);s.call(v,1,257.9);bytes(v,1,[1]);
assert.throws(RangeError,function(){g.call(v,3);});assert.throws(RangeError,function(){s.call(v,3,0);});bytes(v,1,[1]);
// CASE: literal-Int16-bytes-endians-and-unaligned-boundary
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(2+4),all=new $DV(b),v=new $DV(b,1,2+2),g=method('getInt16'),s=method('setInt16');
seed(all,0,[165, 165, 165, 165, 165, 165]);seed(v,1,[237, 204]);assert.sameValue(g.call(v,1),-4660);assert.sameValue(g.call(v,1,false),-4660);
seed(v,1,[204, 237]);assert.sameValue(g.call(v,1,true),-4660);
assert.sameValue(s.call(v,1,-4660),undefined);bytes(v,1,[237, 204]);assert.sameValue(all.getUint8(0),165);assert.sameValue(v.getUint8(0),165);assert.sameValue(v.getUint8(2+1),165);assert.sameValue(all.getUint8(2+3),165);
assert.sameValue(s.call(v,1,-4660,true),undefined);bytes(v,1,[204, 237]);s.call(v,1,65537.9);bytes(v,1,[0, 1]);
assert.throws(RangeError,function(){g.call(v,3);});assert.throws(RangeError,function(){s.call(v,3,0);});bytes(v,1,[0, 1]);
// CASE: literal-Uint16-bytes-endians-and-unaligned-boundary
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(2+4),all=new $DV(b),v=new $DV(b,1,2+2),g=method('getUint16'),s=method('setUint16');
seed(all,0,[165, 165, 165, 165, 165, 165]);seed(v,1,[237, 204]);assert.sameValue(g.call(v,1),60876);assert.sameValue(g.call(v,1,false),60876);
seed(v,1,[204, 237]);assert.sameValue(g.call(v,1,true),60876);
assert.sameValue(s.call(v,1,-4660),undefined);bytes(v,1,[237, 204]);assert.sameValue(all.getUint8(0),165);assert.sameValue(v.getUint8(0),165);assert.sameValue(v.getUint8(2+1),165);assert.sameValue(all.getUint8(2+3),165);
assert.sameValue(s.call(v,1,-4660,true),undefined);bytes(v,1,[204, 237]);s.call(v,1,65537.9);bytes(v,1,[0, 1]);
assert.throws(RangeError,function(){g.call(v,3);});assert.throws(RangeError,function(){s.call(v,3,0);});bytes(v,1,[0, 1]);
// CASE: literal-Int32-bytes-endians-and-unaligned-boundary
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(4+4),all=new $DV(b),v=new $DV(b,1,4+2),g=method('getInt32'),s=method('setInt32');
seed(all,0,[165, 165, 165, 165, 165, 165, 165, 165]);seed(v,1,[254, 220, 186, 153]);assert.sameValue(g.call(v,1),-19088743);assert.sameValue(g.call(v,1,false),-19088743);
seed(v,1,[153, 186, 220, 254]);assert.sameValue(g.call(v,1,true),-19088743);
assert.sameValue(s.call(v,1,-19088743),undefined);bytes(v,1,[254, 220, 186, 153]);assert.sameValue(all.getUint8(0),165);assert.sameValue(v.getUint8(0),165);assert.sameValue(v.getUint8(4+1),165);assert.sameValue(all.getUint8(4+3),165);
assert.sameValue(s.call(v,1,-19088743,true),undefined);bytes(v,1,[153, 186, 220, 254]);s.call(v,1,4294967297.9);bytes(v,1,[0, 0, 0, 1]);
assert.throws(RangeError,function(){g.call(v,3);});assert.throws(RangeError,function(){s.call(v,3,0);});bytes(v,1,[0, 0, 0, 1]);
// CASE: literal-Uint32-bytes-endians-and-unaligned-boundary
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(4+4),all=new $DV(b),v=new $DV(b,1,4+2),g=method('getUint32'),s=method('setUint32');
seed(all,0,[165, 165, 165, 165, 165, 165, 165, 165]);seed(v,1,[254, 220, 186, 153]);assert.sameValue(g.call(v,1),4275878553);assert.sameValue(g.call(v,1,false),4275878553);
seed(v,1,[153, 186, 220, 254]);assert.sameValue(g.call(v,1,true),4275878553);
assert.sameValue(s.call(v,1,-19088743),undefined);bytes(v,1,[254, 220, 186, 153]);assert.sameValue(all.getUint8(0),165);assert.sameValue(v.getUint8(0),165);assert.sameValue(v.getUint8(4+1),165);assert.sameValue(all.getUint8(4+3),165);
assert.sameValue(s.call(v,1,-19088743,true),undefined);bytes(v,1,[153, 186, 220, 254]);s.call(v,1,4294967297.9);bytes(v,1,[0, 0, 0, 1]);
assert.throws(RangeError,function(){g.call(v,3);});assert.throws(RangeError,function(){s.call(v,3,0);});bytes(v,1,[0, 0, 0, 1]);
// CASE: literal-Float16-bytes-endians-and-unaligned-boundary
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(2+4),all=new $DV(b),v=new $DV(b,1,2+2),g=method('getFloat16'),s=method('setFloat16');
seed(all,0,[165, 165, 165, 165, 165, 165]);seed(v,1,[190, 0]);assert.sameValue(g.call(v,1),-1.5);assert.sameValue(g.call(v,1,false),-1.5);
seed(v,1,[0, 190]);assert.sameValue(g.call(v,1,true),-1.5);
assert.sameValue(s.call(v,1,-1.5),undefined);bytes(v,1,[190, 0]);assert.sameValue(all.getUint8(0),165);assert.sameValue(v.getUint8(0),165);assert.sameValue(v.getUint8(2+1),165);assert.sameValue(all.getUint8(2+3),165);
assert.sameValue(s.call(v,1,-1.5,true),undefined);bytes(v,1,[0, 190]);s.call(v,1,1.5);bytes(v,1,[62, 0]);
assert.throws(RangeError,function(){g.call(v,3);});assert.throws(RangeError,function(){s.call(v,3,0);});bytes(v,1,[62, 0]);
// CASE: literal-Float32-bytes-endians-and-unaligned-boundary
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(4+4),all=new $DV(b),v=new $DV(b,1,4+2),g=method('getFloat32'),s=method('setFloat32');
seed(all,0,[165, 165, 165, 165, 165, 165, 165, 165]);seed(v,1,[191, 192, 0, 0]);assert.sameValue(g.call(v,1),-1.5);assert.sameValue(g.call(v,1,false),-1.5);
seed(v,1,[0, 0, 192, 191]);assert.sameValue(g.call(v,1,true),-1.5);
assert.sameValue(s.call(v,1,-1.5),undefined);bytes(v,1,[191, 192, 0, 0]);assert.sameValue(all.getUint8(0),165);assert.sameValue(v.getUint8(0),165);assert.sameValue(v.getUint8(4+1),165);assert.sameValue(all.getUint8(4+3),165);
assert.sameValue(s.call(v,1,-1.5,true),undefined);bytes(v,1,[0, 0, 192, 191]);s.call(v,1,1.5);bytes(v,1,[63, 192, 0, 0]);
assert.throws(RangeError,function(){g.call(v,3);});assert.throws(RangeError,function(){s.call(v,3,0);});bytes(v,1,[63, 192, 0, 0]);
// CASE: literal-Float64-bytes-endians-and-unaligned-boundary
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(8+4),all=new $DV(b),v=new $DV(b,1,8+2),g=method('getFloat64'),s=method('setFloat64');
seed(all,0,[165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165]);seed(v,1,[191, 248, 0, 0, 0, 0, 0, 0]);assert.sameValue(g.call(v,1),-1.5);assert.sameValue(g.call(v,1,false),-1.5);
seed(v,1,[0, 0, 0, 0, 0, 0, 248, 191]);assert.sameValue(g.call(v,1,true),-1.5);
assert.sameValue(s.call(v,1,-1.5),undefined);bytes(v,1,[191, 248, 0, 0, 0, 0, 0, 0]);assert.sameValue(all.getUint8(0),165);assert.sameValue(v.getUint8(0),165);assert.sameValue(v.getUint8(8+1),165);assert.sameValue(all.getUint8(8+3),165);
assert.sameValue(s.call(v,1,-1.5,true),undefined);bytes(v,1,[0, 0, 0, 0, 0, 0, 248, 191]);s.call(v,1,1.5);bytes(v,1,[63, 248, 0, 0, 0, 0, 0, 0]);
assert.throws(RangeError,function(){g.call(v,3);});assert.throws(RangeError,function(){s.call(v,3,0);});bytes(v,1,[63, 248, 0, 0, 0, 0, 0, 0]);
// CASE: integer-nan-infinity-and-modulo-are-not-saturating
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var v=new $DV(new $AB(8)),names=['Int8','Uint8','Int16','Uint16','Int32','Uint32'],widths=[1,1,2,2,4,4],zero=[0,0,0,0];
for(var i=0;i<names.length;i++){var f=method('set'+names[i]);f.call(v,0,NaN);bytes(v,0,zero.slice(0,widths[i]));f.call(v,0,Infinity);bytes(v,0,zero.slice(0,widths[i]));f.call(v,0,-Infinity);bytes(v,0,zero.slice(0,widths[i]));}
v.setUint32(1,4294967295);bytes(v,1,[255,255,255,255]);assert.sameValue(v.getInt32(1),-1);v.setInt32(1,4294967296);bytes(v,1,[0,0,0,0]);
v.setUint32(1,-4294967297);bytes(v,1,[255,255,255,255]);v.setInt32(1,9007199254740991);bytes(v,1,[255,255,255,255]);
// CASE: float32-literal-rounding-subnormal-zero-and-infinity
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var v=new $DV(new $AB(4));
var inputs=[0,-0,Infinity,-Infinity,1.000000059604644775390625,1.000000178813934326171875,1.401298464324817e-45,7.006492321624085e-46,-7.006492321624085e-46];
var wants=[[0,0,0,0],[128,0,0,0],[127,128,0,0],[255,128,0,0],[63,128,0,0],[63,128,0,2],[0,0,0,1],[0,0,0,0],[128,0,0,0]];
for(var i=0;i<inputs.length;i++){v.setFloat32(0,inputs[i]);bytes(v,0,wants[i]);}
seed(v,0,[0,0,0,1]);assert.sameValue(v.getFloat32(0),1.401298464324817e-45);seed(v,0,[128,0,0,0]);assert.sameValue(v.getFloat32(0),-0);
seed(v,0,[127,127,255,255]);assert.sameValue(v.getFloat32(0),3.4028234663852886e38);
// CASE: float64-literal-signed-zero-extremes-and-endians
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var v=new $DV(new $AB(8)),inputs=[0,-0,Infinity,-Infinity,5e-324,2.2250738585072014e-308,1.7976931348623157e308],wants=[[0,0,0,0,0,0,0,0],[128,0,0,0,0,0,0,0],[127,240,0,0,0,0,0,0],[255,240,0,0,0,0,0,0],[0,0,0,0,0,0,0,1],[0,16,0,0,0,0,0,0],[127,239,255,255,255,255,255,255]];
for(var i=0;i<inputs.length;i++){v.setFloat64(0,inputs[i]);bytes(v,0,wants[i]);seed(v,0,wants[i]);assert.sameValue(v.getFloat64(0),inputs[i]);}
seed(v,0,[0,0,0,0,0,0,240,63]);assert.sameValue(v.getFloat64(0,true),1);
// CASE: floating-nan-classification-and-same-input-encoding-stability
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var v=new $DV(new $AB(16)),types=['Float16','Float32','Float64'],widths=[2,4,8],nanBytes=[[126,1],[127,192,0,1],[127,248,0,0,0,0,0,1]];
for(var i=0;i<types.length;i++){var g=method('get'+types[i]),s=method('set'+types[i]);seed(v,0,nanBytes[i]);var n=g.call(v,0);assert.sameValue(n!==n,true);s.call(v,0,n);s.call(v,8,n);for(var j=0;j<widths[i];j++)assert.sameValue(v.getUint8(j),v.getUint8(8+j));assert.sameValue(g.call(v,0)!==g.call(v,0),true);}
// CASE: float16-direct-binary64-literals-1
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var source=new $DV(new $AB(8)),out=new $DV(new $AB(6)),rows=[["positive-zero",[0,0,0,0,0,0,0,0],[0,0]],["negative-zero",[128,0,0,0,0,0,0,0],[128,0]],["binary64-min-subnormal",[0,0,0,0,0,0,0,1],[0,0]],["negative-binary64-min-subnormal",[128,0,0,0,0,0,0,1],[128,0]],["binary64-max-subnormal",[0,15,255,255,255,255,255,255],[0,0]],["binary64-min-normal",[0,16,0,0,0,0,0,0],[0,0]],["below-zero-midpoint",[62,95,255,255,255,255,255,255],[0,0]]];
for(var i=0;i<rows.length;i++){var row=rows[i];seed(source,0,row[1]);var number=source.getFloat64(0);out.setFloat16(1,number,false);
if(row[2]===null){assert.sameValue(number!==number,true);var n=out.getFloat16(1);assert.sameValue(n!==n,true);out.setFloat16(3,number,false);bytes(out,3,[out.getUint8(1),out.getUint8(2)]);var high=out.getUint8(1),low=out.getUint8(2);assert.sameValue((high&124)===124&&((high&3)!==0||low!==0),true);}
else{bytes(out,1,row[2]);out.setFloat16(3,number,true);bytes(out,3,[row[2][1],row[2][0]]);}
assert.sameValue(out.getUint8(0),0);assert.sameValue(out.getUint8(5),0);}
// CASE: float16-direct-binary64-literals-2
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var source=new $DV(new $AB(8)),out=new $DV(new $AB(6)),rows=[["zero-midpoint",[62,96,0,0,0,0,0,0],[0,0]],["above-zero-midpoint",[62,96,0,0,0,0,0,1],[0,1]],["negative-zero-midpoint",[190,96,0,0,0,0,0,0],[128,0]],["negative-above-zero-midpoint-magnitude",[190,96,0,0,0,0,0,1],[128,1]],["minimum-subnormal",[62,112,0,0,0,0,0,0],[0,1]],["odd-subnormal-lower-tie",[62,120,0,0,0,0,0,0],[0,2]],["even-subnormal-lower-tie",[62,132,0,0,0,0,0,0],[0,2]]];
for(var i=0;i<rows.length;i++){var row=rows[i];seed(source,0,row[1]);var number=source.getFloat64(0);out.setFloat16(1,number,false);
if(row[2]===null){assert.sameValue(number!==number,true);var n=out.getFloat16(1);assert.sameValue(n!==n,true);out.setFloat16(3,number,false);bytes(out,3,[out.getUint8(1),out.getUint8(2)]);var high=out.getUint8(1),low=out.getUint8(2);assert.sameValue((high&124)===124&&((high&3)!==0||low!==0),true);}
else{bytes(out,1,row[2]);out.setFloat16(3,number,true);bytes(out,3,[row[2][1],row[2][0]]);}
assert.sameValue(out.getUint8(0),0);assert.sameValue(out.getUint8(5),0);}
// CASE: float16-direct-binary64-literals-3
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var source=new $DV(new $AB(8)),out=new $DV(new $AB(6)),rows=[["maximum-subnormal",[63,15,248,0,0,0,0,0],[3,255]],["below-normal-midpoint",[63,15,251,255,255,255,255,255],[3,255]],["normal-midpoint",[63,15,252,0,0,0,0,0],[4,0]],["above-normal-midpoint",[63,15,252,0,0,0,0,1],[4,0]],["minimum-normal",[63,16,0,0,0,0,0,0],[4,0]],["negative-normal-midpoint",[191,15,252,0,0,0,0,0],[132,0]],["one",[63,240,0,0,0,0,0,0],[60,0]]];
for(var i=0;i<rows.length;i++){var row=rows[i];seed(source,0,row[1]);var number=source.getFloat64(0);out.setFloat16(1,number,false);
if(row[2]===null){assert.sameValue(number!==number,true);var n=out.getFloat16(1);assert.sameValue(n!==n,true);out.setFloat16(3,number,false);bytes(out,3,[out.getUint8(1),out.getUint8(2)]);var high=out.getUint8(1),low=out.getUint8(2);assert.sameValue((high&124)===124&&((high&3)!==0||low!==0),true);}
else{bytes(out,1,row[2]);out.setFloat16(3,number,true);bytes(out,3,[row[2][1],row[2][0]]);}
assert.sameValue(out.getUint8(0),0);assert.sameValue(out.getUint8(5),0);}
// CASE: float16-direct-binary64-literals-4
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var source=new $DV(new $AB(8)),out=new $DV(new $AB(6)),rows=[["below-even-normal-tie",[63,240,1,255,255,255,255,255],[60,0]],["even-normal-tie",[63,240,2,0,0,0,0,0],[60,0]],["double-rounding-counterexample",[63,240,2,0,0,0,0,1],[60,1]],["negative-double-rounding-counterexample",[191,240,2,0,0,0,0,1],[188,1]],["below-odd-normal-tie",[63,240,5,255,255,255,255,255],[60,1]],["odd-normal-tie",[63,240,6,0,0,0,0,0],[60,2]],["above-odd-normal-tie",[63,240,6,0,0,0,0,1],[60,2]]];
for(var i=0;i<rows.length;i++){var row=rows[i];seed(source,0,row[1]);var number=source.getFloat64(0);out.setFloat16(1,number,false);
if(row[2]===null){assert.sameValue(number!==number,true);var n=out.getFloat16(1);assert.sameValue(n!==n,true);out.setFloat16(3,number,false);bytes(out,3,[out.getUint8(1),out.getUint8(2)]);var high=out.getUint8(1),low=out.getUint8(2);assert.sameValue((high&124)===124&&((high&3)!==0||low!==0),true);}
else{bytes(out,1,row[2]);out.setFloat16(3,number,true);bytes(out,3,[row[2][1],row[2][0]]);}
assert.sameValue(out.getUint8(0),0);assert.sameValue(out.getUint8(5),0);}
// CASE: float16-direct-binary64-literals-5
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var source=new $DV(new $AB(8)),out=new $DV(new $AB(6)),rows=[["binary64-nearest-one-third",[63,213,85,85,85,85,85,85],[53,85]],["maximum-finite",[64,239,252,0,0,0,0,0],[123,255]],["below-overflow-midpoint",[64,239,253,255,255,255,255,255],[123,255]],["overflow-midpoint",[64,239,254,0,0,0,0,0],[124,0]],["above-overflow-midpoint",[64,239,254,0,0,0,0,1],[124,0]],["negative-overflow-midpoint",[192,239,254,0,0,0,0,0],[252,0]],["next-unbounded-exponent-value",[64,240,0,0,0,0,0,0],[124,0]]];
for(var i=0;i<rows.length;i++){var row=rows[i];seed(source,0,row[1]);var number=source.getFloat64(0);out.setFloat16(1,number,false);
if(row[2]===null){assert.sameValue(number!==number,true);var n=out.getFloat16(1);assert.sameValue(n!==n,true);out.setFloat16(3,number,false);bytes(out,3,[out.getUint8(1),out.getUint8(2)]);var high=out.getUint8(1),low=out.getUint8(2);assert.sameValue((high&124)===124&&((high&3)!==0||low!==0),true);}
else{bytes(out,1,row[2]);out.setFloat16(3,number,true);bytes(out,3,[row[2][1],row[2][0]]);}
assert.sameValue(out.getUint8(0),0);assert.sameValue(out.getUint8(5),0);}
// CASE: float16-direct-binary64-literals-6
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var source=new $DV(new $AB(8)),out=new $DV(new $AB(6)),rows=[["binary64-maximum-finite",[127,239,255,255,255,255,255,255],[124,0]],["positive-infinity",[127,240,0,0,0,0,0,0],[124,0]],["negative-infinity",[255,240,0,0,0,0,0,0],[252,0]],["nan-low-payload",[127,240,0,0,0,0,0,1],null],["nan-quiet",[127,248,0,0,0,0,0,0],null],["nan-maximum-payload",[127,255,255,255,255,255,255,255],null],["nan-negative-payload",[255,248,0,0,0,0,0,1],null]];
for(var i=0;i<rows.length;i++){var row=rows[i];seed(source,0,row[1]);var number=source.getFloat64(0);out.setFloat16(1,number,false);
if(row[2]===null){assert.sameValue(number!==number,true);var n=out.getFloat16(1);assert.sameValue(n!==n,true);out.setFloat16(3,number,false);bytes(out,3,[out.getUint8(1),out.getUint8(2)]);var high=out.getUint8(1),low=out.getUint8(2);assert.sameValue((high&124)===124&&((high&3)!==0||low!==0),true);}
else{bytes(out,1,row[2]);out.setFloat16(3,number,true);bytes(out,3,[row[2][1],row[2][0]]);}
assert.sameValue(out.getUint8(0),0);assert.sameValue(out.getUint8(5),0);}
// CASE: float16-read-literal-boundaries-independent-of-writer
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var v=new $DV(new $AB(2)),rows=[[[0,0],0],[[128,0],-0],[[0,1],0.000000059604644775390625],[[3,255],0.000060975551605224609375],[[4,0],0.00006103515625],[[60,0],1],[[60,1],1.0009765625],[[53,85],0.333251953125],[[123,255],65504],[[124,0],Infinity],[[252,0],-Infinity]];
for(var i=0;i<rows.length;i++){seed(v,0,rows[i][0]);assert.sameValue(v.getFloat16(0),rows[i][1]);seed(v,0,[rows[i][0][1],rows[i][0][0]]);assert.sameValue(v.getFloat16(0,true),rows[i][1]);}
// CASE: access-index-toindex-and-endian-toboolean-no-callback
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var v=new $DV(new $AB(4));seed(v,0,[18,52,86,120]);var inputs=[undefined,null,false,NaN,-0,-0.75,0.9],n=0,endian={valueOf:function(){n++;throw 'value';},toString:function(){n++;throw 'string';}};
for(var i=0;i<inputs.length;i++)assert.sameValue(v.getUint16(inputs[i]),4660);assert.sameValue(v.getUint16(true),13398);assert.sameValue(v.getUint16('1.9'),13398);
assert.sameValue(v.getUint16(0,endian),13330);v.setUint16(0,4660,endian);bytes(v,0,[52,18]);assert.sameValue(n,0);
assert.throws(RangeError,function(){v.getUint8(-1);});assert.throws(RangeError,function(){v.setUint8(Infinity,0);});assert.throws(TypeError,function(){v.getUint8(Symbol('index'));});
// CASE: setter-index-then-value-before-range-and-no-partial-write
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var v=new $DV(new $AB(4)),log='';seed(v,0,[11,22,33,44]);
assert.throws(RangeError,function(){v.setUint32({valueOf:function(){log+='I';v.setUint8(0,77);return 1;}},{valueOf:function(){log+='V';return 4294967295;}});});assert.sameValue(log,'IV');bytes(v,0,[77,22,33,44]);
var n=0;assert.throws(RangeError,function(){v.setUint8(-1,{valueOf:function(){n++;return 1;}});});assert.sameValue(n,0);
assert.throws(TypeError,function(){v.setUint16(99,Symbol('value'));});bytes(v,0,[77,22,33,44]);
// CASE: setter-value-symboltoprimitive-number-hint-and-abrupt-prefix
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var v=new $DV(new $AB(4)),log='',t={},value={};value[Symbol.toPrimitive]=function(h){assert.sameValue(h,'number');log+='V';v.setUint8(3,99);return '258.9';};
v.setUint16({valueOf:function(){log+='I';return 1;}},value);assert.sameValue(log,'IV');bytes(v,0,[0,1,2,99]);
sameThrow(t,function(){v.setUint16(1,{valueOf:function(){v.setUint8(0,81);throw t;}});});bytes(v,0,[81,1,2,99]);
// CASE: getter-index-callback-mutates-bytes-before-fresh-read
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(4),v=new $DV(b),n=0;seed(v,0,[1,2,3,4]);var got=v.getUint16({valueOf:function(){n++;v.setUint8(1,90);v.setUint8(2,91);return 1;}});
assert.sameValue(got,23131);assert.sameValue(n,1);bytes(v,0,[1,90,91,4]);
// CASE: getter-shrink-out-of-bounds-beats-element-range
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(6,{maxByteLength:8}),v=new $DV(b,2,4),n=0;
assert.throws(TypeError,function(){v.getUint32({valueOf:function(){n++;b.resize(3);return 99;}});});assert.sameValue(n,1);assert.sameValue($AB.isView(v),true);assert.sameValue(getter('buffer').call(v),b);
assert.throws(TypeError,function(){getter('byteOffset').call(v);});assert.throws(TypeError,function(){getter('byteLength').call(v);});
// CASE: setter-value-can-restore-out-of-bounds-fixed-view
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(6,{maxByteLength:8}),v=new $DV(b,2,4);b.resize(1);var log='';
assert.sameValue(v.setUint16({valueOf:function(){log+='I';return 1;}},{valueOf:function(){log+='V';b.resize(6);return 4660;}}),undefined);
assert.sameValue(log,'IV');meta(v,b,2,4);bytes(new $DV(b),0,[0,0,0,18,52,0]);
// CASE: getter-index-can-restore-out-of-bounds-tracking-view
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(4,{maxByteLength:8}),v=new $DV(b,3);b.resize(2);
var value=v.getUint16({valueOf:function(){b.resize(6);var all=new $DV(b);all.setUint8(3,18);all.setUint8(4,52);return 0;}});
assert.sameValue(value,4660);meta(v,b,3,3);
// CASE: setter-index-detaches-but-value-still-runs-and-can-throw
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(4),v=new $DV(b),log='',t={},moved;
sameThrow(t,function(){v.setUint16({valueOf:function(){log+='I';moved=b.transfer();return 0;}},{valueOf:function(){log+='V';throw t;}});});
assert.sameValue(log,'IV');assert.sameValue(moved.byteLength,4);assert.sameValue(getter('buffer').call(v),b);assert.sameValue($AB.isView(v),true);
var n=0;assert.throws(TypeError,function(){v.setUint16(0,{valueOf:function(){n++;return 1;}});});assert.sameValue(n,1);assert.throws(RangeError,function(){v.getUint8(-1);});
// CASE: setter-value-detachment-keeps-author-prefix-without-write
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(4),v=new $DV(b),moved;seed(v,0,[11,22,33,44]);
assert.throws(TypeError,function(){v.setUint32(0,{valueOf:function(){v.setUint8(1,77);moved=b.transferToFixedLength();return 4294967295;}});});bytes(new $DV(moved),0,[11,77,33,44]);
assert.sameValue($AB.isView(v),true);assert.sameValue(getter('buffer').call(v),b);assert.throws(TypeError,function(){getter('byteLength').call(v);});
// CASE: fixed-and-tracking-resize-regrow-and-zeroed-tail
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(6,{maxByteLength:10}),all=new $DV(b),fixed=new $DV(b,2,4),tracking=new $DV(b,2);seed(all,0,[11,22,33,44,55,66]);b.resize(4);
assert.throws(TypeError,function(){fixed.getUint8(0);});meta(tracking,b,2,2);bytes(tracking,0,[33,44]);assert.throws(RangeError,function(){tracking.getUint32(0);});
b.resize(8);meta(fixed,b,2,4);meta(tracking,b,2,6);bytes(all,0,[11,22,33,44,0,0,0,0]);fixed.setUint8(3,91);assert.sameValue(tracking.getUint8(3),91);
// CASE: zero-length-at-end-revives-after-shrink-regrow
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(4,{maxByteLength:6}),fixed=new $DV(b,4,0),tracking=new $DV(b,4);meta(fixed,b,4,0);meta(tracking,b,4,0);
assert.throws(RangeError,function(){fixed.getUint8(0);});b.resize(3);assert.throws(TypeError,function(){getter('byteLength').call(fixed);});assert.throws(TypeError,function(){getter('byteOffset').call(tracking);});
b.resize(6);meta(fixed,b,4,0);meta(tracking,b,4,2);tracking.setUint16(0,4660);bytes(new $DV(b),4,[18,52]);assert.throws(RangeError,function(){fixed.setUint8(0,1);});
// CASE: transfer-keeps-old-view-identity-and-copies-current-bytes
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(4,{maxByteLength:8}),v=new $DV(b,1,2),all=new $DV(b);seed(all,0,[17,29,43,61]);var moved=b.transfer(6),out=new $DV(moved);
bytes(out,0,[17,29,43,61,0,0]);assert.sameValue(moved.resizable,true);assert.sameValue(getter('buffer').call(v),b);assert.sameValue($AB.isView(v),true);assert.throws(TypeError,function(){v.getUint8(0);});
var final=moved.transferToFixedLength(3);bytes(new $DV(final),0,[17,29,43]);assert.sameValue(final.resizable,false);assert.sameValue($AB.isView(out),true);assert.throws(TypeError,function(){out.setUint8(0,1);});
// CASE: overlapping-views-alias-without-offset-or-endian-confusion
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(8),a=new $DV(b,1,6),c=new $DV(b,2,4),all=new $DV(b);a.setUint32(1,305419896);bytes(c,0,[18,52,86,120]);
c.setUint16(1,43981,true);bytes(all,0,[0,0,18,205,171,120,0,0]);assert.sameValue(a.getUint32(1),315468664);assert.sameValue(c.getUint16(1),52651);
// CASE: callback-growth-of-buffer-and-view-tables-keeps-byte-access
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(8,{maxByteLength:12}),v=new $DV(b,1,6),saved=[];
v.setUint32({valueOf:function(){for(var i=0;i<24;i++)saved.push(new $DV(new $AB(2)));b.resize(10);return 1;}},{valueOf:function(){for(var j=0;j<24;j++)saved.push(new $DV(b,0,1));return 305419896;}});
assert.sameValue(saved.length,48);bytes(new $DV(b),0,[0,0,18,52,86,120,0,0,0,0]);assert.sameValue(v.getUint32(1),305419896);
// CASE: prerequisite-bigint-codecs-and-number-rejection
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var v=new $DV(new $AB(8));v.setBigInt64(0,-1n);bytes(v,0,[255,255,255,255,255,255,255,255]);assert.sameValue(v.getBigUint64(0),18446744073709551615n);
assert.throws(TypeError,function(){v.setUint8(0,1n);});assert.throws(TypeError,function(){v.setBigInt64(0,1);});
// CASE: prerequisite-shared-buffer-view-and-alias
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new SharedArrayBuffer(4),a=new $DV(b),v=new $DV(b,1,2);a.setUint16(1,4660);meta(v,b,1,2);assert.sameValue(v.getUint16(0),4660);assert.sameValue($AB.isView(v),true);
// CASE: prerequisite-typedarray-byte-alias
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(4),v=new $DV(b),u=new Uint8Array(b);u[0]=18;u[1]=52;assert.sameValue(v.getUint16(0),4660);v.setUint16(2,43981,true);assert.sameValue(u[2],205);assert.sameValue(u[3],171);assert.sameValue($AB.isView(u),true);
// CASE: prerequisite-proxy-never-forwards-private-view-brand
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var n=0,p=new Proxy($view,{get:function(){n++;throw 'get';},getPrototypeOf:function(){n++;throw 'prototype';}});
assert.sameValue($AB.isView(p),false);assert.throws(TypeError,function(){getter('buffer').call(p);});assert.throws(TypeError,function(){method('getUint8').call(p,0);});assert.throws(TypeError,function(){method('setUint8').call(p,0,1);});assert.sameValue(n,0);
var r=Proxy.revocable($view,{});r.revoke();assert.sameValue($AB.isView(r.proxy),false);assert.throws(TypeError,function(){getter('byteLength').call(r.proxy);});
// CASE: prerequisite-foreign-realm-view-slots-and-intrinsic-prototype
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var other=$262.createRealm().global,b=new other.ArrayBuffer(4),v=new other.DataView(b,1,2);method('setUint16').call(v,0,4660);assert.sameValue(method('getUint16').call(v,0),4660);meta(v,b,1,2);assert.sameValue($AB.isView(v),true);
var local=new $DV(b,1,2);assert.sameValue(Object.getPrototypeOf(local),$dp);assert.sameValue(Object.getPrototypeOf(v),other.DataView.prototype);assert.sameValue(local.getUint16(0),4660);
// CASE: recursive-index-conversion-terminal-resource
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var v=new $DV(new $AB(1)),index={valueOf:function(){return v.getUint8(index);}};
try{v.getUint8(index);}catch(e){throw new Test262Error('terminal resource was caught');}finally{throw new Test262Error('terminal resource ran finally');}throw new Test262Error('unbounded recursion completed');
// CASE: repeated-zero-byte-view-metadata-terminal-resource
assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');
assert.sameValue(typeof DataView,'function','DataView availability');
var $AB=ArrayBuffer,$DV=DataView,$dp=$DV.prototype,$buf=new $AB(2),$view=new $DV($buf);
assert.sameValue($view.buffer,$buf);assert.sameValue($view.byteLength,2);assert.sameValue($AB.isView($view),true);
assert.sameValue(typeof $dp.setUint8,'function');assert.sameValue(typeof $dp.getUint8,'function');
assert.sameValue($view.setUint8(0,37),undefined);assert.sameValue($view.getUint8(0),37);assert.sameValue($view.getUint8(1),0);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameThrow(t,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,t);}assert.sameValue(seen,true);}
function getter(k){var d=Object.getOwnPropertyDescriptor($dp,k);assert.sameValue(typeof d.get,'function');return d.get;}
function method(k){var f=$dp[k];assert.sameValue(typeof f,'function',k+' availability');return f;}
function seed(v,at,bytes){for(var i=0;i<bytes.length;i++)v.setUint8(at+i,bytes[i]);}
function bytes(v,at,want){for(var i=0;i<want.length;i++)assert.sameValue(v.getUint8(at+i),want[i],'byte '+i);}
function meta(v,b,at,n){assert.sameValue(getter('buffer').call(v),b);assert.sameValue(getter('byteOffset').call(v),at);assert.sameValue(getter('byteLength').call(v),n);}
var b=new $AB(0);try{while(true){new $DV(b);}}catch(e){throw new Test262Error('terminal resource was caught');}finally{throw new Test262Error('terminal resource ran finally');}throw new Test262Error('unbounded metadata allocation completed');
