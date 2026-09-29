// Independent concat draft; no engine execution.

// CASE: metadata-property-flags
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
verifyProperty(Array.prototype,'concat',{value:$cc,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty($cc,'name',{value:'concat',writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty($cc,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(own($cc,'prototype'),false);
// CASE: saved-alias-call-apply-bind-nonconstructor
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var f=$cc,a=[1],bound=f.bind(a,2);Array.prototype.concat=undefined;sameList(f.call(a,3),[1,3]);sameList(f.apply(a,[4,[5]]),[1,4,5]);sameList(bound(6),[1,2,6]);sameList(a,[1]);assert.throws(TypeError,function(){new f();});assert.throws(TypeError,function(){new bound();});assert.throws(TypeError,function(){Reflect.construct(f,[]);});
// CASE: dense-items-order-and-argument-evaluation
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var log='',a=[1],b=[2,3];function item(v){log+=v;return v;}var r=$cc.call(a,item(4),b,item(5),undefined,null);sameList(r,[1,4,2,3,5,undefined,null]);assert.sameValue(log,'45');sameList(a,[1]);sameList(b,[2,3]);assert.notSameValue(r,a);
// CASE: fresh-shallow-copy-and-no-argument
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var obj={},nested=[2],a=[obj,nested],r=$cc.call(a);assert.notSameValue(r,a);assert.sameValue(r[0],obj);assert.sameValue(r[1],nested);nested[0]=9;assert.sameValue(r[1][0],9);var empty=$cc.call([]);assert.sameValue(empty.length,0);assert.sameValue(Object.getPrototypeOf(empty),Array.prototype);
// CASE: nullish-receiver-rejected-without-item-hooks
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var n=0,o={};Object.defineProperty(o,Symbol.isConcatSpreadable,{get:function(){n++;throw 'item';}});assert.throws(TypeError,function(){$cc.call(null,o);});assert.throws(TypeError,function(){$cc.call(undefined,o);});assert.sameValue(n,0);
// CASE: primitive-receiver-boxes-once-and-string-codeunits
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var values=[7,false,Symbol('s'),'A\uD83D\uDE00'];for(var i=0;i<values.length;i++){var r=$cc.call(values[i],9);assert.sameValue(r.length,2);assert.sameValue(typeof r[0],'object');assert.sameValue(r[0].valueOf(),values[i]);assert.sameValue(r[1],9);}var s=Object('A\uD83D\uDE00\uD800');spread(s);var q=$cc.call(s);sameList(q,['A','\uD83D','\uDE00','\uD800']);
// CASE: primitive-items-ignore-prototype-spread-hooks
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var n=0,sp=Symbol.isConcatSpreadable,protos=[String.prototype,Number.prototype,Boolean.prototype,Symbol.prototype];for(var i=0;i<protos.length;i++)Object.defineProperty(protos[i],sp,{get:function(){n++;throw 'primitive hook';},configurable:true});try{var sym=Symbol('x'),r=$cc.call([],'ab',7,false,sym,null,undefined);sameList(r,['ab',7,false,sym,null,undefined]);assert.sameValue(n,0);}finally{for(var j=0;j<protos.length;j++)delete protos[j][sp];}
// CASE: default-array-spread-versus-ordinary-object
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var o={0:'x',length:1},a=[2],r=$cc.call(o,a);assert.sameValue(r[0],o);assert.sameValue(r[1],2);assert.sameValue(r.length,2);var f=function(a,b){};f[0]=3;var q=$cc.call([],f);assert.sameValue(q.length,1);assert.sameValue(q[0],f);spread(f);var t=$cc.call([],f);assert.sameValue(t.length,2);assert.sameValue(t[0],3);assert.sameValue(own(t,'1'),false);
// CASE: false-spread-flags-suppress-length-and-indices
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var flags=[false,null,0,-0,NaN,''];for(var i=0;i<flags.length;i++){var o={};o[Symbol.isConcatSpreadable]=flags[i];Object.defineProperty(o,'length',{get:function(){throw 'length';}});var r=$cc.call([],o);assert.sameValue(r.length,1);assert.sameValue(r[0],o);var a=[1];a[Symbol.isConcatSpreadable]=flags[i];var q=$cc.call(a);assert.sameValue(q.length,1);assert.sameValue(q[0],a);}
// CASE: truthy-spread-flags-never-coerce
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var poison={valueOf:function(){throw 'valueOf';},toString:function(){throw 'toString';}};poison[Symbol.toPrimitive]=function(){throw 'primitive';};var flags=[true,1,'yes',Symbol('truth'),poison];for(var i=0;i<flags.length;i++){var o={0:7,length:1};o[Symbol.isConcatSpreadable]=flags[i];sameList($cc.call([],o),[7]);}
// CASE: inherited-spread-getter-receiver-and-single-read
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var p={},o=Object.create(p),n=0;o[0]=4;o.length=1;Object.defineProperty(p,Symbol.isConcatSpreadable,{get:function(){n++;assert.sameValue(this,o);return true;}});sameList($cc.call([],o),[4]);assert.sameValue(n,1);
// CASE: undefined-spread-flag-falls-back-to-array-brand
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=[1],o={0:2,length:1};a[Symbol.isConcatSpreadable]=undefined;o[Symbol.isConcatSpreadable]=undefined;var r=$cc.call(a,o);assert.sameValue(r.length,2);assert.sameValue(r[0],1);assert.sameValue(r[1],o);delete a[Symbol.isConcatSpreadable];sameList($cc.call(a),[1]);
// CASE: tolength-truncation-nan-negative-and-symbol
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var inputs=[undefined,null,false,NaN,-0,-5,-Infinity,2.9,'2'],lengths=[0,0,0,0,0,0,0,2,2];for(var i=0;i<inputs.length;i++){var o=spread({0:7,1:8,length:inputs[i]}),r=$cc.call([],o);assert.sameValue(r.length,lengths[i]);if(lengths[i])sameList(r,[7,8]);}var bad=spread({length:Symbol('length')});assert.throws(TypeError,function(){$cc.call([],bad);});
// CASE: spread-then-length-then-number-conversion-order
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var log='',o={};Object.defineProperty(o,Symbol.isConcatSpreadable,{get:function(){log+='S';return true;}});Object.defineProperty(o,'length',{get:function(){log+='L';return {valueOf:function(){log+='V';return {};},toString:function(){log+='T';return '1';}};}});Object.defineProperty(o,'0',{get:function(){log+='G';return 7;}});sameList($cc.call([],o),[7]);assert.sameValue(log,'SLVTG');
// CASE: safe-integer-overflow-before-indexed-access
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var inputs=[9007199254740991,Infinity],log='';for(var i=0;i<inputs.length;i++){var o={};Object.defineProperty(o,Symbol.isConcatSpreadable,{get:function(){log+='S';return true;}});Object.defineProperty(o,'length',{get:function(){log+='L';return {valueOf:function(){log+='V';return inputs[i];}};}});Object.defineProperty(o,'0',{get:function(){throw 'index';}});assert.throws(TypeError,function(){$cc.call({},o);});}assert.sameValue(log,'SLVSLV');
// CASE: huge-safe-length-can-throw-at-first-index
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var token={},log='',o={};Object.defineProperty(o,Symbol.isConcatSpreadable,{get:function(){log+='S';return true;}});Object.defineProperty(o,'length',{get:function(){log+='L';return 9007199254740991;}});Object.defineProperty(o,'0',{get:function(){log+='G';throw token;}});sameThrow(token,function(){$cc.call([],o);});assert.sameValue(log,'SLG');
// CASE: holes-undefined-and-trailing-result-length
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=[,undefined,,3,,],r=$cc.call(a,[,5,,]);assert.sameValue(r.length,8);assert.sameValue(own(r,'0'),false);assert.sameValue(own(r,'1'),true);assert.sameValue(r[1],undefined);assert.sameValue(own(r,'2'),false);assert.sameValue(r[3],3);assert.sameValue(own(r,'4'),false);assert.sameValue(own(r,'5'),false);assert.sameValue(r[6],5);assert.sameValue(own(r,'7'),false);verifyProperty(r,'1',{value:undefined,writable:true,enumerable:true,configurable:true},{restore:true});
// CASE: inherited-indices-use-source-receiver
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var p={},o=Object.create(p),calls=0;Object.defineProperty(p,'1',{get:function(){calls++;assert.sameValue(this,o);return 9;}});o.length=3;o[2]=8;spread(o);var r=$cc.call([],o);assert.sameValue(r.length,3);assert.sameValue(own(r,'0'),false);assert.sameValue(r[1],9);assert.sameValue(own(r,'1'),true);assert.sameValue(r[2],8);assert.sameValue(calls,1);
// CASE: live-getter-deletes-and-adds-future-indices
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var o=spread({1:2,length:3});Object.defineProperty(o,'0',{get:function(){delete o[1];o[2]=9;return 1;}});var r=$cc.call([],o);assert.sameValue(r.length,3);assert.sameValue(r[0],1);assert.sameValue(own(r,'1'),false);assert.sameValue(r[2],9);
// CASE: captured-length-survives-shrink-and-growth
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=[1,2,3];Object.defineProperty(a,'0',{get:function(){a.length=1;a[3]=9;return 7;},configurable:true});var r=$cc.call(a);assert.sameValue(r.length,3);assert.sameValue(r[0],7);assert.sameValue(own(r,'1'),false);assert.sameValue(own(r,'2'),false);assert.sameValue(own(r,'3'),false);assert.sameValue(a.length,4);assert.sameValue(a[3],9);
// CASE: later-item-spreadability-and-length-are-live
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var later={0:2,length:1},o=spread({length:1});Object.defineProperty(o,'0',{get:function(){spread(later);later.length=2;later[1]=9;return 1;}});sameList($cc.call([],o,later),[1,2,9]);var shared=spread({length:1}),count=0;Object.defineProperty(shared,'0',{get:function(){count++;shared.length=2;shared[1]=8;return count;}});sameList($cc.call([],shared,shared),[1,2,8]);assert.sameValue(count,2);
// CASE: species-before-spreadability-constructor-zero-arity
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var log='',a=[7],out={},h={};Object.defineProperty(a,'constructor',{get:function(){log+='C';return h;},configurable:true});Object.defineProperty(h,Symbol.species,{get:function(){log+='P';assert.sameValue(this,h);return C;}});function C(n){log+='N';assert.sameValue(n,0);assert.sameValue(arguments.length,1);assert.sameValue(new.target,C);return out;}Object.defineProperty(a,Symbol.isConcatSpreadable,{get:function(){log+='S';return true;}});Object.defineProperty(a,'0',{get:function(){log+='G';return 7;},configurable:true});Object.defineProperty(out,'length',{set:function(n){log+='L';assert.sameValue(n,1);assert.sameValue(this,out);assert.sameValue(out[0],7);}});assert.sameValue($cc.call(a),out);assert.sameValue(log,'CPNSGL');
// CASE: default-species-keeps-intrinsic-after-global-poison
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var Original=Array,proto=Array.prototype;try{Array=function(){throw 'global Array';};var a=[1],b=[2],c=[3];a.constructor=undefined;species(b,null);c.constructor={};var ra=$cc.call(a),rb=$cc.call(b),rc=$cc.call(c);sameList(ra,[1]);sameList(rb,[2]);sameList(rc,[3]);assert.sameValue(Object.getPrototypeOf(ra),proto);assert.sameValue(Object.getPrototypeOf(rb),proto);assert.sameValue(Object.getPrototypeOf(rc),proto);}finally{Array=Original;}
// CASE: invalid-species-and-constructor-abrupt-before-spread
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var badCtor=[null,1,true,'x',Symbol('ctor')];for(var i=0;i<badCtor.length;i++){var a=[1];a.constructor=badCtor[i];Object.defineProperty(a,Symbol.isConcatSpreadable,{get:function(){throw 'spread';}});assert.throws(TypeError,function(){$cc.call(a);});}var badSpecies=[1,true,'x',{},Symbol('species'),()=>{}];for(var j=0;j<badSpecies.length;j++){var b=[2];species(b,badSpecies[j]);Object.defineProperty(b,Symbol.isConcatSpreadable,{get:function(){throw 'spread';}});assert.throws(TypeError,function(){$cc.call(b);});}var t={},c=[3];species(c,function(){throw t;});Object.defineProperty(c,Symbol.isConcatSpreadable,{get:function(){throw 'spread';}});sameThrow(t,function(){$cc.call(c);});
// CASE: generic-receiver-ignores-constructor-and-species
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var o=spread({0:4,length:1});Object.defineProperty(o,'constructor',{get:function(){throw 'constructor';}});Object.defineProperty(o,Symbol.species,{get:function(){throw 'species';}});var r=$cc.call(o);sameList(r,[4]);assert.sameValue(Object.getPrototypeOf(r),Array.prototype);
// CASE: source-holes-retain-preexisting-output-keys
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=[7,,9],out={1:'retained',5:'beyond',note:8};species(a,function(){return out;});assert.sameValue($cc.call(a),out);assert.sameValue(out[0],7);assert.sameValue(out[1],'retained');assert.sameValue(out[2],9);assert.sameValue(out[5],'beyond');assert.sameValue(out.length,3);assert.sameValue(out.note,8);
// CASE: species-source-alias-and-captured-length
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=[1,2];species(a,function(){return a;});var r=$cc.call(a,a);assert.sameValue(r,a);sameList(a,[1,2,1,2]);
// CASE: species-aliases-later-input-with-live-overwrite
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=[1],b=[2,3];species(a,function(){return b;});assert.sameValue($cc.call(a,b),b);sameList(b,[1,1,1]);sameList(a,[1]);
// CASE: bound-species-constructor-and-alternate-array-prototype
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var calls=0;function C(prefix,n){calls++;assert.sameValue(prefix,7);assert.sameValue(n,0);assert.sameValue(arguments.length,2);assert.sameValue(new.target,C);}var bound=C.bind(null,7),holder={};holder[Symbol.species]=bound;function Alt(){}Alt.prototype={constructor:holder};var a=Reflect.construct(Array,[2],Alt);a[0]=10;a[1]=20;assert.sameValue(Array.isArray(a),true);var r=$cc.call(a);assert.sameValue(Object.getPrototypeOf(r),C.prototype);sameList(r,[10,20]);assert.sameValue(calls,1);
// CASE: result-definitions-bypass-setters-and-replace-accessors
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var p={},out=Object.create(p),a=[7,8],calls=0;Object.defineProperty(p,'0',{set:function(){calls++;throw 'inherited setter';}});Object.defineProperty(out,'1',{get:function(){throw 'result getter';},set:function(){calls++;throw 'own setter';},configurable:true});species(a,function(){return out;});assert.sameValue($cc.call(a),out);assert.sameValue(calls,0);verifyProperty(out,'0',{value:7,writable:true,enumerable:true,configurable:true},{restore:true});verifyProperty(out,'1',{value:8,writable:true,enumerable:true,configurable:true},{restore:true});assert.sameValue(out.length,2);
// CASE: nonconfigurable-output-refusal-retains-prefix
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=[7,8,9],out={},reads=0;Object.defineProperty(out,'1',{value:8,writable:true,enumerable:true,configurable:false});Object.defineProperty(a,'2',{get:function(){reads++;throw 'later';},configurable:true});species(a,function(){return out;});assert.throws(TypeError,function(){$cc.call(a);});assert.sameValue(out[0],7);assert.sameValue(out[1],8);assert.sameValue(own(out,'length'),false);assert.sameValue(reads,0);
// CASE: nonextensible-output-get-effects-before-refusal
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var out=Object.preventExtensions({}),a=[1],reads=0;Object.defineProperty(a,'0',{get:function(){reads++;return 7;},configurable:true});species(a,function(){return out;});assert.throws(TypeError,function(){$cc.call(a);});assert.sameValue(reads,1);assert.sameValue(own(out,'0'),false);assert.sameValue(own(out,'length'),false);
// CASE: final-length-inherited-setter-and-abrupt-prefix
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var token={},p={},out=Object.create(p),a=[7,,9],seen=-1;Object.defineProperty(p,'length',{set:function(n){seen=n;assert.sameValue(this,out);assert.sameValue(out[0],7);assert.sameValue(own(out,'1'),false);assert.sameValue(out[2],9);throw token;}});species(a,function(){return out;});sameThrow(token,function(){$cc.call(a);});assert.sameValue(seen,3);assert.sameValue(own(out,'length'),false);assert.sameValue(out[0],7);assert.sameValue(out[2],9);
// CASE: readonly-or-getter-only-final-length-always-throws
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=[],out={};Object.defineProperty(out,'length',{value:0,writable:false});species(a,function(){return out;});assert.throws(TypeError,function(){$cc.call(a);});var b=[7],p={},q=Object.create(p);Object.defineProperty(p,'length',{get:function(){throw 'read length';}});species(b,function(){return q;});assert.throws(TypeError,function(){$cc.call(b);});assert.sameValue(q[0],7);assert.sameValue(own(q,'length'),false);var c=[],locked=[];Object.defineProperty(locked,'length',{writable:false});species(c,function(){return locked;});assert.throws(TypeError,function(){$cc.call(c);});
// CASE: final-array-shrink-refusal-keeps-descending-partial-effects
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var out=[9,8,7,6],a=[1];Object.defineProperty(out,'2',{configurable:false});species(a,function(){return out;});assert.throws(TypeError,function(){$cc.call(a);});assert.sameValue(out.length,3);assert.sameValue(out[0],1);assert.sameValue(out[1],8);assert.sameValue(out[2],7);assert.sameValue(own(out,'3'),false);sameList(a,[1]);var b=[4],ok=[9,8,7];species(b,function(){return ok;});assert.sameValue($cc.call(b),ok);sameList(ok,[4]);
// CASE: boxed-string-species-result-rejects-strict-definitions
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=[7],out=Object('x');species(a,function(){return out;});assert.throws(TypeError,function(){$cc.call(a);});assert.sameValue(out[0],'x');assert.sameValue(out.length,1);var b=[],empty=Object('');species(b,function(){return empty;});assert.throws(TypeError,function(){$cc.call(b);});assert.sameValue(empty.length,0);
// CASE: mapped-unmapped-and-spread-input-arguments
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
Function("a","b","var out=arguments,src=[7,8],h={};h[Symbol.species]=function(){return out;};src.constructor=h;assert.sameValue(Array.prototype.concat.call(src),out);assert.sameValue(a,7);assert.sameValue(b,8);assert.sameValue(out.length,2);")(1,2);Function("a","b","'use strict';var out=arguments,src=[7,8],h={};h[Symbol.species]=function(){return out;};src.constructor=h;assert.sameValue(Array.prototype.concat.call(src),out);assert.sameValue(a,1);assert.sameValue(b,2);assert.sameValue(out[0],7);assert.sameValue(out[1],8);")(1,2);Function("a","b","var o=arguments;o[Symbol.isConcatSpreadable]=true;Object.defineProperty(o,'0',{get:function(){b=9;return 7;},configurable:true});var r=Array.prototype.concat.call([],o);assert.sameValue(r[0],7);assert.sameValue(r[1],9);assert.sameValue(r.length,2);assert.sameValue(b,9);")(1,2);
// CASE: abrupt-getter-identities-preserve-earlier-output
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var t={},a=[1];Object.defineProperty(a,'constructor',{get:function(){throw t;}});sameThrow(t,function(){$cc.call(a);});var b=[1],h={};Object.defineProperty(h,Symbol.species,{get:function(){throw t;}});b.constructor=h;sameThrow(t,function(){$cc.call(b);});var out={},c=[4];species(c,function(){return out;});var o={};Object.defineProperty(o,Symbol.isConcatSpreadable,{get:function(){throw t;}});sameThrow(t,function(){$cc.call(c,o);});assert.sameValue(out[0],4);assert.sameValue(own(out,'length'),false);var d=spread({get length(){throw t;}});sameThrow(t,function(){$cc.call(c,d);});var e=spread({length:2,get 0(){return 7;},get 1(){throw t;}});sameThrow(t,function(){$cc.call(c,e);});assert.sameValue(out[0],4);assert.sameValue(out[1],7);assert.sameValue(own(out,'2'),false);assert.sameValue(own(out,'length'),false);
// CASE: sealed-frozen-sources-are-not-written
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=Object.freeze([1,,3]),b=Object.seal([4]),o=spread({0:5,length:1});Object.freeze(o);var r=$cc.call(a,b,o);assert.sameValue(r.length,5);assert.sameValue(r[0],1);assert.sameValue(own(r,'1'),false);assert.sameValue(r[2],3);assert.sameValue(r[3],4);assert.sameValue(r[4],5);assert.sameValue(Object.isFrozen(a),true);assert.sameValue(Object.isFrozen(o),true);assert.sameValue(b.length,1);
// CASE: iterator-and-close-hooks-are-never-read
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=[1],o=spread({0:2,length:1}),out={};function poison(){throw 'iterator or return';}Object.defineProperty(a,Symbol.iterator,{get:poison});Object.defineProperty(o,Symbol.iterator,{get:poison});Object.defineProperty(o,'return',{get:poison});Object.defineProperty(out,Symbol.iterator,{get:poison});species(a,function(){return out;});assert.sameValue($cc.call(a,o),out);sameList(out,[1,2]);var plain={};Object.defineProperty(plain,Symbol.iterator,{get:poison});var r=$cc.call([],plain);assert.sameValue(r.length,1);assert.sameValue(r[0],plain);
// CASE: proxy-source-has-get-order-and-result-definition-traps
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var log='',target={0:7,length:2};target[Symbol.isConcatSpreadable]=true;var p=new Proxy(target,{get:function(t,k){if(k===Symbol.isConcatSpreadable){log+='S';return true;}log+='g'+k+',';return t[k];},has:function(t,k){log+='h'+k+',';return k in t;}});var r=$cc.call([],p);assert.sameValue(r.length,2);assert.sameValue(r[0],7);assert.sameValue(own(r,'1'),false);assert.sameValue(log,'Sglength,h0,g0,h1,');var out={},trace='',q=new Proxy(out,{defineProperty:function(t,k,d){trace+='d'+k+',';Object.defineProperty(t,k,d);return true;},set:function(t,k,v){trace+='s'+k+',';t[k]=v;return true;}}),a=[1,,3];species(a,function(){return q;});assert.sameValue($cc.call(a),q);assert.sameValue(trace,'d0,d2,slength,');assert.sameValue(out.length,3);
// CASE: typed-array-default-and-explicit-spreadability
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=new Uint8Array([7,8]),r=$cc.call([],a);assert.sameValue(r.length,1);assert.sameValue(r[0],a);a[Symbol.isConcatSpreadable]=true;sameList($cc.call([],a),[7,8]);var src=[1,2],out=new Uint8Array(2);species(src,function(){return out;});assert.throws(TypeError,function(){$cc.call(src);});assert.sameValue(out[0],1);assert.sameValue(out[1],2);
// CASE: cross-realm-array-intrinsic-species-fallback
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var other=$262.createRealm().global,a=new other.Array(2);a[0]=7;a[1]=8;Object.defineProperty(other.Array,Symbol.species,{get:function(){throw 'foreign intrinsic species';},configurable:true});var r=$cc.call(a);sameList(r,[7,8]);assert.sameValue(Object.getPrototypeOf(r),Array.prototype);assert.notSameValue(Object.getPrototypeOf(r),other.Array.prototype);
// CASE: recursive-species-terminal-resource
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=[];species(a,function(){return $cc.call(a);});try{$cc.call(a);}catch(e){throw new Test262Error('terminal resource was caught');}finally{throw new Test262Error('terminal resource ran finally');}throw new Test262Error('unbounded recursion completed');
// CASE: large-hole-walk-terminal-resource-before-u32-final-length
assert.sameValue(typeof Array.prototype.concat,'function','concat availability');
var $cc=Array.prototype.concat,$ga=[1,2],$gr=$cc.call($ga,[3],4);assert.sameValue($gr.length,4);assert.sameValue($gr[0],1);assert.sameValue($gr[1],2);assert.sameValue($gr[2],3);assert.sameValue($gr[3],4);assert.sameValue($ga.length,2);
var $go={marker:7},$gd=$cc.call($go,8);assert.sameValue($gd.length,2);assert.sameValue($gd[0],$go);assert.sameValue($gd[1],8);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function spread(o){o[Symbol.isConcatSpreadable]=true;return o;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var o=spread({length:4294967296});try{$cc.call([],o);}catch(e){throw new Test262Error('terminal resource was caught or eager u32 exception');}finally{throw new Test262Error('terminal resource ran finally');}throw new Test262Error('huge hole walk completed');
