// Independent splice draft; no engine execution.

// CASE: metadata-property-flags
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
verifyProperty(Array.prototype,'splice',{value:$sp,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty($sp,'name',{value:'splice',writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty($sp,'length',{value:2,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(own($sp,'prototype'),false);
// CASE: saved-alias-call-apply-bind-nonconstructor
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=[1,2],b=[3,4],c=[5,6],f=$sp;Array.prototype.splice=undefined;sameList(f.call(a,0,1),[1]);sameList(f.apply(b,[1,1]),[4]);var bound=f.bind(c,0,1);sameList(bound(),[5]);sameList(a,[2]);sameList(b,[3]);sameList(c,[6]);assert.throws(TypeError,function(){new f();});assert.throws(TypeError,function(){new bound();});assert.throws(TypeError,function(){Reflect.construct(f,[]);});
// CASE: absent-start-delete-versus-undefined
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=[1,2],b=[1,2],c=[1,2],d=[1,2];sameList($sp.call(a),[]);sameList(a,[1,2]);sameList($sp.call(b,undefined),[1,2]);sameList(b,[]);sameList($sp.call(c,undefined,undefined),[]);sameList(c,[1,2]);sameList($sp.call(d,1,undefined,9),[]);sameList(d,[1,9,2]);
// CASE: start-clamps-fractions-signed-zero-and-infinities
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var starts=[NaN,-0,-Infinity,Infinity,-10,-1.9,1.9,20],removed=[[0],[0],[0],[],[0],[3],[1],[]];for(var i=0;i<starts.length;i++){var a=[0,1,2,3];sameList($sp.call(a,starts[i],1),removed[i]);}assert.throws(TypeError,function(){$sp.call([1],Symbol('start'),0);});
// CASE: deletecount-clamps-and-symbol-error
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var counts=[NaN,-0,-Infinity,-2,0,1.9,Infinity,99],removed=[[],[],[],[],[],[1],[1,2],[1,2]];for(var i=0;i<counts.length;i++){var a=[0,1,2];sameList($sp.call(a,1,counts[i]),removed[i]);}assert.throws(TypeError,function(){$sp.call([1],0,Symbol('count'));});
// CASE: length-start-delete-conversion-order
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var log='',finalLength=-1,o={0:10,1:20,get length(){log+='L';return {valueOf:function(){log+='V';return 2;}};},set length(v){log+='F';finalLength=v;}},s={valueOf:function(){log+='S';return 1;}},d={valueOf:function(){log+='D';return 1;}};sameList($sp.call(o,s,d),[20]);assert.sameValue(log,'LVSDF');assert.sameValue(finalLength,1);assert.sameValue(o[0],10);assert.sameValue(own(o,'1'),false);
// CASE: tolength-fraction-negative-and-infinity
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var inputs=[undefined,null,false,-0,-5,NaN,-Infinity,2.9,'3',Infinity],want=[0,0,0,0,0,0,0,2,3,9007199254740991];for(var i=0;i<inputs.length;i++){var o={length:inputs[i]};sameList($sp.call(o,Infinity,0),[]);assert.sameValue(o.length,want[i]);}assert.throws(TypeError,function(){$sp.call({length:Symbol('n')},0,0);});
// CASE: safe-integer-overflow-before-index-effects
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var reads=0,writes=0,o={length:9007199254740991};Object.defineProperty(o,'9007199254740990',{get:function(){reads++;throw 'index';},set:function(){writes++;}});Object.defineProperty(o,'constructor',{get:function(){throw 'generic constructor';}});assert.throws(TypeError,function(){$sp.call(o,9007199254740990,0,'x');});assert.sameValue(reads,0);assert.sameValue(writes,0);assert.sameValue(o.length,9007199254740991);
// CASE: captured-length-survives-start-mutation
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=[0,1,2,3],s={valueOf:function(){a.length=1;a[2]=9;return 1;}},r=$sp.call(a,s,1);assert.sameValue(r.length,1);assert.sameValue(own(r,'0'),false);assert.sameValue(a.length,3);assert.sameValue(a[0],0);assert.sameValue(a[1],9);assert.sameValue(own(a,'2'),false);
// CASE: fresh-deleted-result-holes-versus-undefined
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=[,undefined,,3],r=$sp.call(a,0,3);assert.sameValue(r.length,3);assert.sameValue(own(r,'0'),false);assert.sameValue(own(r,'1'),true);assert.sameValue(r[1],undefined);assert.sameValue(own(r,'2'),false);verifyProperty(r,'1',{value:undefined,writable:true,enumerable:true,configurable:true},{restore:true});sameList(a,[3]);
// CASE: inherited-index-read-original-receiver
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var p={},o=Object.create(p),calls=0;Object.defineProperty(p,'1',{get:function(){assert.sameValue(this,o);calls++;return 42;},configurable:true});o.length=3;o[2]=9;var r=$sp.call(o,0,2);assert.sameValue(r.length,2);assert.sameValue(own(r,'0'),false);assert.sameValue(r[1],42);assert.sameValue(own(r,'1'),true);assert.sameValue(calls,1);assert.sameValue(o[0],9);assert.sameValue(o.length,1);assert.sameValue(own(o,'1'),false);
// CASE: deleted-range-getter-mutates-future-positions
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var o={1:20,length:3};Object.defineProperty(o,'0',{get:function(){delete o[1];o[2]=99;return 10;},configurable:true});var r=$sp.call(o,0,3);assert.sameValue(r[0],10);assert.sameValue(own(r,'1'),false);assert.sameValue(r[2],99);assert.sameValue(r.length,3);assert.sameValue(o.length,0);assert.sameValue(own(o,'0'),false);assert.sameValue(own(o,'2'),false);
// CASE: result-definitions-bypass-inherited-setters
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var calls=0,p={},a=[7];Object.defineProperty(p,'0',{set:function(){calls++;throw 'inherited setter';}});function C(){return Object.create(p);}species(a,C);var r=$sp.call(a,0,1);assert.sameValue(calls,0);verifyProperty(r,'0',{value:7,writable:true,enumerable:true,configurable:true},{restore:true});assert.sameValue(r.length,1);sameList(a,[]);
// CASE: species-constructor-order-arity-and-newtarget
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var log='',stored=-1,out={},a=[7],holder={};Object.defineProperty(a,'0',{get:function(){log+='G';return 7;},set:function(v){log+='W';stored=v;},configurable:true});Object.defineProperty(a,'constructor',{get:function(){log+='C';return holder;},configurable:true});Object.defineProperty(holder,Symbol.species,{get:function(){log+='P';assert.sameValue(this,holder);return C;}});Object.defineProperty(out,'length',{set:function(n){log+='L';assert.sameValue(n,1);assert.sameValue(this[0],7);}});function C(n){log+='N';assert.sameValue(arguments.length,1);assert.sameValue(n,1);assert.sameValue(new.target,C);return out;}var s={valueOf:function(){log+='S';return 0;}},d={valueOf:function(){log+='D';return 1;}};assert.sameValue($sp.call(a,s,d,8),out);assert.sameValue(log,'SDCPNGLW');assert.sameValue(stored,8);assert.sameValue(a.length,1);
// CASE: default-species-keeps-intrinsic-after-global-poison
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var Original=Array,proto=Array.prototype;try{Array=function(){throw 'poisoned global Array';};var a=[1],b=[2],c=[3];a.constructor=undefined;species(b,null);c.constructor={};var ra=$sp.call(a,0,1),rb=$sp.call(b,0,1),rc=$sp.call(c,0,1);sameList(ra,[1]);sameList(rb,[2]);sameList(rc,[3]);assert.sameValue(Object.getPrototypeOf(ra),proto);assert.sameValue(Object.getPrototypeOf(rb),proto);assert.sameValue(Object.getPrototypeOf(rc),proto);}finally{Array=Original;}
// CASE: invalid-species-constructor-and-abrupt-identity
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var invalid=[null,1,true,'x',Symbol('ctor')];for(var i=0;i<invalid.length;i++){var a=[1];a.constructor=invalid[i];assert.throws(TypeError,function(){$sp.call(a,0,1);});sameList(a,[1]);}var bad=[1,true,'x',{},Symbol('species'),()=>{}];for(var j=0;j<bad.length;j++){var b=[2];species(b,bad[j]);assert.throws(TypeError,function(){$sp.call(b,0,1);});sameList(b,[2]);}var token={},c=[3],h={};Object.defineProperty(h,Symbol.species,{get:function(){throw token;}});c.constructor=h;sameThrow(token,function(){$sp.call(c,0,1);});sameList(c,[3]);
// CASE: generic-receiver-ignores-constructor-and-protocols
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var o={0:5,length:1};Object.defineProperty(o,'constructor',{get:function(){throw 'constructor';}});Object.defineProperty(o,Symbol.species,{get:function(){throw 'species';}});Object.defineProperty(o,Symbol.iterator,{get:function(){throw 'iterator';}});Object.defineProperty(o,Symbol.isConcatSpreadable,{get:function(){throw 'spreadable';}});var r=$sp.call(o,0,1);sameList(r,[5]);assert.sameValue(Object.getPrototypeOf(r),Array.prototype);assert.sameValue(o.length,0);
// CASE: ordinary-species-result-keeps-preexisting-hole-key
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=[7,,9],out={1:'retained',note:8};function C(n){assert.sameValue(n,2);return out;}species(a,C);assert.sameValue($sp.call(a,0,2),out);assert.sameValue(out[0],7);assert.sameValue(out[1],'retained');assert.sameValue(out.note,8);assert.sameValue(out.length,2);sameList(a,[9]);
// CASE: species-source-alias-retains-observable-shrink
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=[0,1,2,3];function C(){return a;}species(a,C);var r=$sp.call(a,1,2,'x');assert.sameValue(r,a);assert.sameValue(a.length,3);assert.sameValue(a[0],1);assert.sameValue(a[1],'x');assert.sameValue(own(a,'2'),false);assert.sameValue(own(a,'3'),false);
// CASE: species-returns-different-existing-array
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=['a','b','c'],out=[9,8,7,6];function C(n){assert.sameValue(n,1);return out;}species(a,C);assert.sameValue($sp.call(a,1,1),out);sameList(out,['b']);assert.sameValue(own(out,'1'),false);sameList(a,['a','c']);
// CASE: bound-species-and-reflect-created-array-prototype
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var calls=0;function C(prefix,n){calls++;assert.sameValue(prefix,7);assert.sameValue(n,1);assert.sameValue(arguments.length,2);assert.sameValue(new.target,C);}var bound=C.bind(null,7);function Alt(){}var holder={};holder[Symbol.species]=bound;Alt.prototype={constructor:holder};var a=Reflect.construct(Array,[2],Alt);a[0]=10;a[1]=20;assert.sameValue(Array.isArray(a),true);var r=$sp.call(a,0,1);assert.sameValue(Object.getPrototypeOf(r),C.prototype);assert.sameValue(r[0],10);assert.sameValue(r.length,1);assert.sameValue(calls,1);assert.sameValue(a[0],20);assert.sameValue(a.length,1);
// CASE: left-shift-ascending-live-get-set-order
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var log='',o={length:5};function put(i){Object.defineProperty(o,''+i,{get:function(){log+='g'+i+',';return i;},set:function(v){log+='s'+i+':'+v+',';},configurable:true});}for(var i=0;i<5;i++)put(i);sameList($sp.call(o,0,2),[0,1]);assert.sameValue(log,'g0,g1,g2,s0:2,g3,s1:3,g4,s2:4,');assert.sameValue(o.length,3);assert.sameValue(own(o,'3'),false);assert.sameValue(own(o,'4'),false);
// CASE: right-shift-descending-live-get-set-order
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var log='',o={length:3};Object.defineProperty(o,'0',{get:function(){throw 'unrelated';},configurable:true});function put(i){Object.defineProperty(o,''+i,{get:function(){log+='g'+i+',';return i;},set:function(v){log+='s'+i+':'+v+',';},configurable:true});}for(var i=1;i<5;i++)put(i);sameList($sp.call(o,1,0,'X','Y'),[]);assert.sameValue(log,'g2,s4:2,g1,s3:1,s1:X,s2:Y,');assert.sameValue(o.length,5);
// CASE: equal-count-skips-unrelated-index-reads
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var o={1:9,length:3};Object.defineProperty(o,'0',{get:function(){throw 'before';}});Object.defineProperty(o,'2',{get:function(){throw 'after';}});sameList($sp.call(o,1,1,'x'),[9]);assert.sameValue(o[1],'x');assert.sameValue(o.length,3);
// CASE: holes-delete-destinations-in-both-directions
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a={0:'a',2:'c',length:3},b={1:'b',length:2};sameList($sp.call(a,0,1),['a']);assert.sameValue(own(a,'0'),false);assert.sameValue(a[1],'c');assert.sameValue(own(a,'2'),false);assert.sameValue(a.length,2);sameList($sp.call(b,0,0,'x'),[]);assert.sameValue(b[0],'x');assert.sameValue(own(b,'1'),false);assert.sameValue(b[2],'b');assert.sameValue(b.length,3);
// CASE: excess-tail-deletion-descends-before-failure
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var o={0:'a',1:'b',2:'c',3:'d',4:'e',length:5};Object.defineProperty(o,'2',{configurable:false});assert.throws(TypeError,function(){$sp.call(o,0,4);});assert.sameValue(o[0],'e');assert.sameValue(o[1],'b');assert.sameValue(o[2],'c');assert.sameValue(own(o,'3'),false);assert.sameValue(own(o,'4'),false);assert.sameValue(o.length,5);
// CASE: inserted-object-symbol-identity-without-coercion
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var token={},item={valueOf:function(){throw token;},toString:function(){throw token;}},s=Symbol('item'),a=[0];Object.defineProperty(item,Symbol.iterator,{get:function(){throw token;}});Object.defineProperty(item,Symbol.isConcatSpreadable,{get:function(){throw token;}});sameList($sp.call(a,1,0,item,s),[]);assert.sameValue(a[1],item);assert.sameValue(a[2],s);assert.sameValue(a.length,3);
// CASE: source-inherited-setter-sees-original-receiver
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var p={},o=Object.create(p),calls=0;Object.defineProperty(p,'0',{set:function(v){assert.sameValue(this,o);assert.sameValue(v,7);calls++;}});o.length=0;sameList($sp.call(o,0,0,7),[]);assert.sameValue(calls,1);assert.sameValue(own(o,'0'),false);assert.sameValue(o.length,1);
// CASE: reentrant-index-getter-uses-captured-outer-bounds
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var o={1:20,2:30,length:3},inner;Object.defineProperty(o,'0',{get:function(){delete o[0];inner=$sp.call(o,1,1);return 10;},configurable:true});var outer=$sp.call(o,0,1);sameList(inner,[20]);sameList(outer,[10]);assert.sameValue(o[0],30);assert.sameValue(own(o,'1'),false);assert.sameValue(o.length,2);
// CASE: readonly-destination-preserves-grown-prefix
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var o={0:'a',1:'b',2:'c',length:3};Object.defineProperty(o,'2',{writable:false});assert.throws(TypeError,function(){$sp.call(o,0,0,'x');});assert.sameValue(o[3],'c');assert.sameValue(o[2],'c');assert.sameValue(o[1],'b');assert.sameValue(o[0],'a');assert.sameValue(o.length,3);
// CASE: nonconfigurable-hole-destination-preserves-left-prefix
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var o={0:'a',1:'b',length:4};Object.defineProperty(o,'1',{configurable:false});Object.defineProperty(o,'3',{get:function(){throw 'late read';}});assert.throws(TypeError,function(){$sp.call(o,0,1);});assert.sameValue(o[0],'b');assert.sameValue(o[1],'b');assert.sameValue(o.length,4);
// CASE: nonextensible-receiver-rejects-new-destination
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var o=Object.preventExtensions({0:'a',length:1});assert.throws(TypeError,function(){$sp.call(o,0,0,'x');});assert.sameValue(o[0],'a');assert.sameValue(own(o,'1'),false);assert.sameValue(o.length,1);
// CASE: final-source-length-setter-throws-after-edits
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var token={},o={0:'a',1:'b',get length(){return 2;},set length(v){assert.sameValue(v,1);throw token;}};sameThrow(token,function(){$sp.call(o,0,1);});assert.sameValue(o[0],'b');assert.sameValue(own(o,'1'),false);assert.sameValue(o.length,2);
// CASE: result-definition-failure-retains-output-prefix-only
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=[7,8,9],out={};Object.defineProperty(out,'1',{value:8,writable:true,enumerable:true,configurable:false});function C(){return out;}species(a,C);assert.throws(TypeError,function(){$sp.call(a,0,2);});verifyProperty(out,'0',{value:7,writable:true,enumerable:true,configurable:true},{restore:true});assert.sameValue(out[1],8);assert.sameValue(own(out,'length'),false);sameList(a,[7,8,9]);
// CASE: result-length-setter-mutates-before-source-movement
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=[0,1,2,3],out={},calls=0;Object.defineProperty(out,'length',{set:function(v){calls++;assert.sameValue(v,1);assert.sameValue(this[0],1);assert.sameValue(a.length,4);assert.sameValue(a[1],1);a[2]=99;delete a[3];}});function C(){return out;}species(a,C);assert.sameValue($sp.call(a,1,1),out);assert.sameValue(calls,1);assert.sameValue(a.length,3);assert.sameValue(a[0],0);assert.sameValue(a[1],99);assert.sameValue(own(a,'2'),false);
// CASE: string-utf16-readonly-length-and-partial-growth
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
assert.throws(TypeError,function(){$sp.call('',0,0);});assert.throws(TypeError,function(){$sp.call('A\uD83D\uDE00',1,1,'x');});var o=Object('\uD83D\uDE00');assert.throws(TypeError,function(){$sp.call(o,2,0,'x');});assert.sameValue(o[0],'\uD83D');assert.sameValue(o[1],'\uDE00');assert.sameValue(o[2],'x');assert.sameValue(o.length,2);var n=0,s={valueOf:function(){n++;return 0;}};assert.throws(TypeError,function(){$sp.call(null,s,0);});assert.throws(TypeError,function(){$sp.call(undefined,s,0);});assert.sameValue(n,0);
// CASE: number-boolean-symbol-boxing
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var values=[7,false,Symbol('v')];for(var i=0;i<values.length;i++){sameList($sp.call(values[i],0,0,'x'),[]);var o=Object(values[i]);sameList($sp.call(o,0,0,'y'),[]);assert.sameValue(o[0],'y');assert.sameValue(o.length,1);assert.sameValue(o.valueOf(),values[i]);}
// CASE: mapped-unmapped-and-species-output-arguments
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
Function("a","b","var r=Array.prototype.splice.call(arguments,0,1);assert.sameValue(r[0],1);assert.sameValue(a,2);assert.sameValue(b,2);assert.sameValue(arguments.length,1);assert.sameValue(Object.prototype.hasOwnProperty.call(arguments,'1'),false);")(1,2);Function("a","b","'use strict';var r=Array.prototype.splice.call(arguments,0,1);assert.sameValue(r[0],1);assert.sameValue(a,1);assert.sameValue(b,2);assert.sameValue(arguments[0],2);assert.sameValue(arguments.length,1);")(1,2);Function("a","b","var out=arguments,src=[7,8],h={};h[Symbol.species]=function(){return out;};src.constructor=h;assert.sameValue(Array.prototype.splice.call(src,0,2),out);assert.sameValue(a,7);assert.sameValue(b,8);assert.sameValue(out.length,2);")(1,2);
// CASE: sealed-frozen-noop-and-replacement
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=Object.seal([1,2]);sameList($sp.call(a,0,1,9),[1]);sameList(a,[9,2]);sameList($sp.call(Object.seal([])),[]);assert.throws(TypeError,function(){$sp.call(Object.freeze([]));});assert.throws(TypeError,function(){$sp.call(Object.freeze([1]),0,0);});assert.throws(TypeError,function(){$sp.call(Object.freeze([1]),0,1,1);});
// CASE: generic-ordinary-index-keys-above-u32
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var o={length:4294967297};o[4294967294]='a';o[4294967295]='b';o[4294967296]='c';sameList($sp.call(o,4294967295,1,'x','y'),['b']);assert.sameValue(o[4294967294],'a');assert.sameValue(o[4294967295],'x');assert.sameValue(o[4294967296],'y');assert.sameValue(o[4294967297],'c');assert.sameValue(o.length,4294967298);
// CASE: actual-array-late-length-rangeerror-keeps-ordinary-key
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=[];a.length=4294967295;assert.throws(RangeError,function(){$sp.call(a,4294967295,0,'x');});assert.sameValue(a.length,4294967295);assert.sameValue(a[4294967295],'x');assert.sameValue(own(a,'4294967295'),true);
// CASE: huge-constant-work-and-default-result-rangeerror
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var o={length:9007199254740991};sameList($sp.call(o,Infinity,0),[]);assert.sameValue(o.length,9007199254740991);var r=$sp.call(o,9007199254740990,1,'x');assert.sameValue(r.length,1);assert.sameValue(own(r,'0'),false);assert.sameValue(o[9007199254740990],'x');assert.sameValue(o.length,9007199254740991);var reads=0,b={length:4294967296};Object.defineProperty(b,'0',{get:function(){reads++;throw 'copy';}});assert.throws(RangeError,function(){$sp.call(b,0,4294967296);});assert.sameValue(reads,0);
// CASE: abrupt-getter-constructor-setter-identities
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var t={},o={get length(){throw t;}};sameThrow(t,function(){$sp.call(o,0,0);});sameThrow(t,function(){$sp.call([], {valueOf:function(){throw t;}},0);});sameThrow(t,function(){$sp.call([],0,{valueOf:function(){throw t;}});});var a=[1];Object.defineProperty(a,'constructor',{get:function(){throw t;}});sameThrow(t,function(){$sp.call(a,0,1);});var b=[2];species(b,function(){throw t;});sameThrow(t,function(){$sp.call(b,0,1);});var c={length:1,get 0(){throw t;}};sameThrow(t,function(){$sp.call(c,0,1);});var d={length:0,set 0(v){throw t;}};sameThrow(t,function(){$sp.call(d,0,0,8);});
// CASE: recursive-species-terminal-resource
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=[];function C(){return $sp.call(a,0,0);}species(a,C);try{$sp.call(a,0,0);}catch(e){throw new Test262Error('terminal resource was caught');}finally{throw new Test262Error('terminal resource ran finally');}throw new Test262Error('recursion completed');
// CASE: proxy-live-traps-and-overflow-before-species
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var log='',target=[1,2,3],p=new Proxy(target,{get:function(t,k){log+='g'+k+',';return t[k];},has:function(t,k){log+='h'+k+',';return k in t;},set:function(t,k,v){log+='s'+k+',';t[k]=v;return true;}});sameList($sp.call(p,1,1,9),[2]);assert.sameValue(log,'glength,gconstructor,h1,g1,s1,slength,');sameList(target,[1,9,3]);var constructorReads=0,overflow=new Proxy([],{get:function(t,k){if(k==='length')return 9007199254740991;if(k==='constructor')constructorReads++;throw 'unexpected read';}});assert.throws(TypeError,function(){$sp.call(overflow,9007199254740991,0,1);});assert.sameValue(constructorReads,0);
// CASE: foreign-realm-array-intrinsic-species-exception
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var foreign=$262.createRealm().global.Array;Object.defineProperty(foreign,Symbol.species,{get:function(){throw 'foreign species';},configurable:true});var a=[7];a.constructor=foreign;var r=$sp.call(a,0,1);sameList(r,[7]);assert.sameValue(Object.getPrototypeOf(r),Array.prototype);
// CASE: typed-array-generic-splice-partial-errors
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var a=new Uint8Array([1,2,3]);Object.defineProperty(a,'constructor',{get:function(){throw 'not an Array';}});assert.throws(TypeError,function(){$sp.call(a,0,1);});assert.sameValue(a[0],2);assert.sameValue(a[1],3);assert.sameValue(a[2],3);assert.sameValue(a.length,3);var b=new Uint8Array([4,5]);assert.throws(TypeError,function(){$sp.call(b,0,1,9);});assert.sameValue(b[0],9);assert.sameValue(b[1],5);assert.sameValue(b.length,2);
// CASE: million-hole-shift-terminal-resource
assert.sameValue(typeof Array.prototype.splice,'function','splice availability');
var $sp=Array.prototype.splice,$ga=[1,2,3],$gr=$sp.call($ga,1,1,9);assert.sameValue($gr.length,1);assert.sameValue($gr[0],2);assert.sameValue($ga.length,3);assert.sameValue($ga[1],9);
var $go={0:4,1:5,length:2},$gd=$sp.call($go,0,1);assert.sameValue($gd.length,1);assert.sameValue($gd[0],4);assert.sameValue($go[0],5);assert.sameValue($go.length,1);
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function species(a,C){var h={};h[Symbol.species]=C;a.constructor=h;}
function sameThrow(token,f){var seen=false;try{f();}catch(e){seen=true;assert.sameValue(e,token);}assert.sameValue(seen,true);}
var o={length:1000001};try{$sp.call(o,0,0,1);}catch(e){throw new Test262Error('terminal resource was caught');}finally{throw new Test262Error('terminal resource ran finally');}throw new Test262Error('million visits completed');
