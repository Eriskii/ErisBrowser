// Independent own-key oracle. No engine executed during preparation.

// CASE: mixed-numeric-string-symbol-order
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var o={},s1=Symbol('x'),s2=Symbol('x');o.b=1;o['01']=2;o[4294967295]=3;o.a=4;o[4294967294]=5;o[2]=6;o[s1]=7;o.z=8;o[s2]=9;
sameList(Reflect.ownKeys(o),['2','4294967294','b','01','4294967295','a','z',s1,s2]);sameList(Object.keys(o),['2','4294967294','b','01','4294967295','a','z']);sameList(Object.getOwnPropertySymbols(o),[s1,s2]);
// CASE: noncanonical-numeric-spellings
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var o={};o['-0']=1;o['00']=2;o['1.0']=3;o['1e0']=4;o['+1']=5;o['4294967295']=6;o[1]=7;o[0]=8;o['9007199254740991']=9;sameList(Object.getOwnPropertyNames(o),['0','1','-0','00','1.0','1e0','+1','4294967295','9007199254740991']);
// CASE: delete-reinsert-moves-only-string-ordinal
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var o={a:1,b:2,c:3};o[9]=9;o[2]=2;delete o.b;delete o[2];o.b=20;o[2]=22;sameList(Object.keys(o),['2','9','a','c','b']);
// CASE: descriptor-kind-and-flag-changes-preserve-ordinal
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var o={a:1,b:2,c:3};Object.defineProperty(o,'b',{get:function(){return 9;},enumerable:false,configurable:true});sameList(Object.keys(o),['a','c']);Object.defineProperty(o,'b',{value:8,writable:true,enumerable:true});sameList(Object.keys(o),['a','b','c']);sameList(Object.values(o),[1,8,3]);
// CASE: symbol-identity-delete-reinsert
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var a=Symbol('same'),b=Symbol('same'),o={x:1};o[a]=2;o[b]=3;delete o[a];o[a]=4;Object.defineProperty(o,b,{enumerable:false});sameList(Reflect.ownKeys(o),['x',b,a]);sameList(Object.getOwnPropertySymbols(o),[b,a]);sameList(Object.keys(o),['x']);
// CASE: utf16-nul-and-normalization-distinct-keys
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var o={};o['\uD800']=1;o['\uDC00']=2;o['a\0b']=3;o['a']=4;o['e\u0301']=5;o['\u00e9']=6;sameList(Reflect.ownKeys(o),['\uD800','\uDC00','a\0b','a','e\u0301','\u00e9']);sameList(Object.values(o),[1,2,3,4,5,6]);
// CASE: long-common-prefix-names-retain-exact-order
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var p='',o={};for(var i=0;i<256;i++)p+='x';o[p+'b']=1;o[p+'a']=2;o[p+'c']=3;delete o[p+'a'];o[p+'a']=4;sameList(Object.keys(o),[p+'b',p+'c',p+'a']);
// CASE: empty-primitives-and-reflect-rejection
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
sameList(Object.keys(Object.create(null)),[]);sameList(Object.getOwnPropertyNames(7),[]);sameList(Object.values(false),[]);sameList(Object.keys(Symbol('v')),[]);assert.throws(TypeError,function(){Reflect.ownKeys('x');});assert.throws(TypeError,function(){Object.keys(null);});
// CASE: array-holes-do-not-copy-prototype-keys
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var a=[10,20,30],p={1:99};delete a[1];Object.setPrototypeOf(a,p);sameList(Object.keys(a),['0','2']);sameList(Object.getOwnPropertyNames(a),['0','2','length']);sameList(Object.values(a),[10,30]);
// CASE: array-dense-sidecar-overlap-listed-once
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var a=[10,20,30],reads=0;Object.defineProperty(a,'0',{get:function(){reads++;return 11;},configurable:true,enumerable:true});Object.defineProperty(a,'1',{value:21,enumerable:false});sameList(Object.getOwnPropertyNames(a),['0','1','2','length']);sameList(Object.keys(a),['0','2']);assert.sameValue(reads,0);sameList(Object.values(a),[11,30]);assert.sameValue(reads,1);
// CASE: array-hole-refill-and-numeric-reinsert
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var a=[0,1,2];delete a[1];Object.defineProperty(a,'1',{get:function(){return 7;},enumerable:true,configurable:true});delete a[0];a[0]=9;sameList(Object.keys(a),['0','1','2']);sameList(Object.values(a),[9,7,2]);
// CASE: array-max-logical-length-sparse-own-shape
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var a=[];a[4294967294]=7;a[2]=3;a['4294967295']=8;a.tail=9;assert.sameValue(a.length,4294967295);sameList(Reflect.ownKeys(a),['2','4294967294','length','4294967295','tail']);sameList(Object.values(a),[3,7,8,9]);
// CASE: array-shrink-regrow-does-not-resurrect-keys
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var a=[0,1,2,3];a.note=5;a.length=1;a.length=4;a[3]=9;sameList(Object.getOwnPropertyNames(a),['0','3','length','note']);sameList(Object.keys(a),['0','3','note']);
// CASE: array-length-compatible-definition-no-duplicate
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var a=[1,2];Object.defineProperty(a,'length',{value:2,writable:false});Object.defineProperty(a,'0',{value:1,writable:false,configurable:false});sameList(Reflect.ownKeys(a),['0','1','length']);assert.sameValue(Object.getOwnPropertyDescriptor(a,'length').writable,false);
// CASE: primitive-string-enumerates-code-units
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var s='A\uD83D\uDE00\uD800';sameList(Object.keys(s),['0','1','2','3']);sameList(Object.values(s),['A','\uD83D','\uDE00','\uD800']);sameList(Object.getOwnPropertyNames(s),['0','1','2','3','length']);
// CASE: boxed-string-compatible-virtual-definitions
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var o=Object('\uD800x');Object.defineProperty(o,'0',{value:'\uD800',writable:false,enumerable:true,configurable:false});Object.defineProperty(o,'length',{value:2,writable:false,enumerable:false,configurable:false});sameList(Reflect.ownKeys(o),['0','1','length']);sameList(Object.values(o),['\uD800','x']);
// CASE: boxed-string-extra-index-and-symbol-order
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var o=Object('ab'),s=Symbol('s');o.z=1;o[10]=10;o[2]=2;o['01']=3;o[s]=4;sameList(Reflect.ownKeys(o),['0','1','2','10','length','z','01',s]);sameList(Object.keys(o),['0','1','2','10','z','01']);
// CASE: primitive-string-own-enumeration-skips-inherited
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var calls=0;Object.defineProperty(String.prototype,'extra',{get:function(){calls++;throw 1;},enumerable:true,configurable:true});sameList(Object.keys('x'),['0']);sameList(Object.values('x'),['x']);assert.sameValue(calls,0);
// CASE: key-only-operations-never-read-getters
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var n=0,s=Symbol('s'),o={};Object.defineProperty(o,'x',{get:function(){n++;throw 1;},enumerable:true});Object.defineProperty(o,s,{get:function(){n++;throw 2;},enumerable:true});sameList(Object.keys(o),['x']);sameList(Object.getOwnPropertyNames(o),['x']);sameList(Reflect.ownKeys(o),['x',s]);assert.sameValue(n,0);
// CASE: values-delete-later-and-ignore-new-key
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var o={get a(){delete o.b;o.c=3;return 1;},b:2};sameList(Object.values(o),[1]);sameList(Object.keys(o),['a','c']);
// CASE: values-hide-later-key-before-read
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var n=0,o={get a(){Object.defineProperty(o,'b',{enumerable:false});return 1;},get b(){n++;return 2;}};sameList(Object.values(o),[1]);assert.sameValue(n,0);
// CASE: values-observe-replaced-later-getter
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var t='',o={get a(){Object.defineProperty(o,'b',{get:function(){t+='new';return 9;},enumerable:true,configurable:true});return 1;},get b(){throw 'old';}};sameList(Object.values(o),[1,9]);assert.sameValue(t,'new');
// CASE: values-deleted-own-key-does-not-read-prototype
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var p={get b(){throw 'prototype';}},o=Object.create(p);Object.defineProperty(o,'a',{get:function(){delete o.b;return 1;},enumerable:true});Object.defineProperty(o,'b',{value:2,enumerable:true,configurable:true});sameList(Object.values(o),[1]);
// CASE: values-abrupt-getter-preserves-effects-and-stops
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var token={},got,n=0,o={get a(){o.mark=1;throw token;},get b(){n++;return 2;}};try{Object.values(o);}catch(e){got=e;}assert.sameValue(got,token);assert.sameValue(o.mark,1);assert.sameValue(n,0);
// CASE: defineproperties-collects-before-applying
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var o={},trace='',p={};Object.defineProperty(p,'a',{enumerable:true,get:function(){trace+='a';assert.sameValue(own(o,'a'),false);return {value:1,enumerable:true};}});Object.defineProperty(p,'b',{enumerable:true,get:function(){trace+='b';assert.sameValue(own(o,'a'),false);return {value:2,enumerable:true};}});assert.sameValue(Object.defineProperties(o,p),o);sameList(Object.keys(o),['a','b']);assert.sameValue(trace,'ab');
// CASE: defineproperties-conversion-failure-is-before-writes
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var o={},trace='',p={};Object.defineProperty(p,'a',{get:function(){trace+='a';return {value:1};},enumerable:true});Object.defineProperty(p,'b',{get:function(){trace+='b';return {get:7};},enumerable:true});assert.throws(TypeError,function(){Object.defineProperties(o,p);});sameList(Reflect.ownKeys(o),[]);assert.sameValue(trace,'ab');
// CASE: defineproperties-live-delete-hide-and-add
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var target={},p={};Object.defineProperty(p,'a',{enumerable:true,get:function(){delete p.b;Object.defineProperty(p,'c',{enumerable:false});p.d={value:4};return {value:1};}});p.b={value:2};p.c={value:3};Object.defineProperties(target,p);sameList(Reflect.ownKeys(target),['a']);
// CASE: defineproperties-numeric-string-symbol-order
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var p={},o={},s=Symbol('s'),t='';function d(k){return {enumerable:true,get:function(){t+=k;return {value:k,enumerable:true};}};}Object.defineProperty(p,'b',d('b'));Object.defineProperty(p,'2',d('2'));Object.defineProperty(p,'1',d('1'));Object.defineProperty(p,s,d('s'));Object.defineProperties(o,p);assert.sameValue(t,'12bs');sameList(Reflect.ownKeys(o),['1','2','b',s]);
// CASE: defineproperties-definition-failure-keeps-applied-prefix
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var o={};Object.defineProperty(o,'b',{value:0,writable:false,configurable:false});assert.throws(TypeError,function(){Object.defineProperties(o,{a:{value:1},b:{value:2},c:{value:3}});});assert.sameValue(o.a,1);assert.sameValue(o.b,0);assert.sameValue(own(o,'c'),false);sameList(Object.getOwnPropertyNames(o),['b','a']);
// CASE: object-create-descriptor-throw-keeps-source-effects
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var token={},got,n=0,p={a:{value:1}};Object.defineProperty(p,'b',{enumerable:true,get:function(){n++;throw token;}});try{Object.create(null,p);}catch(e){got=e;}assert.sameValue(got,token);assert.sameValue(n,1);
// CASE: array-shrink-converts-twice-before-delete
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var a=[0,1,2,3],n=0,v={valueOf:function(){n++;assert.sameValue(a.length,4);assert.sameValue(own(a,'3'),true);if(n===1)a.note=7;return 2;}};Object.defineProperty(a,'length',{value:v});assert.sameValue(n,2);sameList(Reflect.ownKeys(a),['0','1','length','note']);
// CASE: array-conversion-makes-length-readonly-before-refusal
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var a=[0,1,2],n=0,v={valueOf:function(){n++;if(n===2)Object.defineProperty(a,'length',{writable:false});return 1;}};assert.sameValue(Reflect.defineProperty(a,'length',{value:v}),false);assert.sameValue(n,2);assert.sameValue(a.length,3);sameList(Object.keys(a),['0','1','2']);assert.sameValue(Object.getOwnPropertyDescriptor(a,'length').writable,false);
// CASE: array-second-conversion-throw-preserves-first-effects
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var a=[0,1,2],n=0,token={},got,v={valueOf:function(){n++;if(n===1){a[4]=9;return 1;}throw token;}};try{Object.defineProperty(a,'length',{value:v});}catch(e){got=e;}assert.sameValue(got,token);assert.sameValue(n,2);assert.sameValue(a.length,5);sameList(Object.keys(a),['0','1','2','4']);
// CASE: array-inconsistent-conversions-range-error-no-delete
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var a=[0,1,2],n=0,v={valueOf:function(){return ++n;}};assert.throws(RangeError,function(){Object.defineProperty(a,'length',{value:v});});assert.sameValue(n,2);assert.sameValue(a.length,3);sameList(Object.keys(a),['0','1','2']);
// CASE: array-partial-descending-shrink-final-readonly
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var a=[0,1,2,3,4];Object.defineProperty(a,'2',{configurable:false});assert.sameValue(Reflect.defineProperty(a,'length',{value:1,writable:false}),false);assert.sameValue(a.length,3);sameList(Object.keys(a),['0','1','2']);assert.sameValue(Object.getOwnPropertyDescriptor(a,'length').writable,false);assert.sameValue(Reflect.defineProperty(a,'3',{value:9}),false);
// CASE: array-sparse-shrink-never-invokes-element-getters
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var a=[],n=0;Object.defineProperty(a,'4294967294',{get:function(){n++;throw 1;},enumerable:true,configurable:true});a[2]=3;a.length=1;assert.sameValue(a.length,1);sameList(Reflect.ownKeys(a),['length']);assert.sameValue(n,0);
// CASE: for-in-prototype-order-and-nonenumerable-shadows
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var p={2:2,x:3,y:4},o=Object.create(p),seen=[];o[1]=1;o.a=5;Object.defineProperty(o,'x',{value:7,enumerable:false});for(var k in o)seen.push(k);sameList(seen,['1','a','2','y']);
// CASE: for-in-does-not-read-values-or-include-symbols
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var n=0,s=Symbol('s'),p={},o=Object.create(p),seen=[];Object.defineProperty(o,'a',{get:function(){n++;throw 1;},enumerable:true});Object.defineProperty(p,'b',{get:function(){n++;throw 2;},enumerable:true});o[s]=3;for(var k in o)seen.push(k);sameList(seen,['a','b']);assert.sameValue(n,0);
// CASE: for-in-delete-before-visit-allows-prototype-name
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var p={b:9},o=Object.create(p),seen=[];o.a=1;o.b=2;for(var k in o){seen.push(k);if(k==='a')delete o.b;}sameList(seen,['a','b']);
// CASE: for-in-new-own-key-not-in-current-snapshot
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var o={a:1,b:2},seen=[];for(var k in o){seen.push(k);if(k==='a')o.c=3;}sameList(seen,['a','b']);
// CASE: for-in-lazily-snapshots-added-prototype-key
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var p={p:2},o=Object.create(p),seen=[];o.a=1;for(var k in o){seen.push(k);if(k==='a')p.q=3;}sameList(seen,['a','p','q']);
// CASE: for-in-lazily-follows-replaced-prototype
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var p={old:1},q={next:2},o=Object.create(p),seen=[];o.a=1;for(var k in o){seen.push(k);if(k==='a')Object.setPrototypeOf(o,q);}sameList(seen,['a','next']);
// CASE: for-in-nullish-skips-target-assignment
assert.sameValue(typeof Object.keys,'function');assert.sameValue(typeof Object.values,'function');assert.sameValue(typeof Object.getOwnPropertyNames,'function');assert.sameValue(typeof Reflect.ownKeys,'function');
var $ok=Object.keys({x:7});assert.sameValue($ok.length,1);assert.sameValue($ok[0],'x');assert.sameValue(Object.values({x:7})[0],7);assert.sameValue(Reflect.ownKeys({x:7})[0],'x');
function sameList(a,b){assert.sameValue(a.length,b.length);for(var i=0;i<b.length;i++)assert.sameValue(a[i],b[i]);}
function own(o,k){return Object.prototype.hasOwnProperty.call(o,k);}
var n=0,box={set key(v){n++;}};for(box.key in null){throw 1;}for(box.key in undefined){throw 2;}assert.sameValue(n,0);
