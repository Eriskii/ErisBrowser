// CASE: regexp-match-method-descriptor
verifyProperty(RegExp.prototype[Symbol.match],'name',{value:'[Symbol.match]',writable:false,enumerable:false,configurable:true});verifyProperty(RegExp.prototype[Symbol.match],'length',{value:1,writable:false,enumerable:false,configurable:true});
// CASE: regexp-search-method-descriptor
verifyProperty(RegExp.prototype[Symbol.search],'name',{value:'[Symbol.search]',writable:false,enumerable:false,configurable:true});verifyProperty(RegExp.prototype[Symbol.search],'length',{value:1,writable:false,enumerable:false,configurable:true});
// CASE: match-hook-gets-raw-receiver
var raw={toString:function(){throw 1;}},p={},marker={};p[Symbol.match]=function(v){assert.sameValue(this,p);assert.sameValue(v,raw);return marker;};assert.sameValue(String.prototype.match.call(raw,p),marker);
// CASE: search-hook-gets-raw-receiver
var raw={toString:function(){throw 1;}},p={},marker={};p[Symbol.search]=function(v){assert.sameValue(this,p);assert.sameValue(v,raw);return marker;};assert.sameValue(String.prototype.search.call(raw,p),marker);
// CASE: match-fallback-invokes-prototype-hook
var marker={};RegExp.prototype[Symbol.match]=function(s){assert.sameValue(s,'abc');assert.sameValue(this.source,'b');return marker;};assert.sameValue('abc'.match('b'),marker);
// CASE: search-fallback-invokes-prototype-hook
var marker={};RegExp.prototype[Symbol.search]=function(s){assert.sameValue(s,'abc');assert.sameValue(this.source,'b');return marker;};assert.sameValue('abc'.search('b'),marker);
// CASE: match-flags-and-input-conversion-order
var log='',r={exec:function(s){log+='e';assert.sameValue(s,'abc');return null;}};Object.defineProperty(r,'flags',{get:function(){log+='f';return {toString:function(){log+='t';return '';}};}});RegExp.prototype[Symbol.match].call(r,{toString:function(){log+='s';return 'abc';}});assert.sameValue(log,'sfte');
// CASE: match-custom-unicode-empty-advance
var log='',r={flags:'gu',lastIndex:91,exec:function(){if(this.lastIndex>3)return null;log+=this.lastIndex+',';return {0:''};}};assert.sameValue(RegExp.prototype[Symbol.match].call(r,'\ud83d\ude00x').length,3);assert.sameValue(log,'0,2,3,');
// CASE: match-custom-unicode-sets-empty-advance
var log='',r={flags:'gv',lastIndex:91,exec:function(){if(this.lastIndex>3)return null;log+=this.lastIndex+',';return {0:''};}};assert.sameValue(RegExp.prototype[Symbol.match].call(r,'\ud83d\ude00x').length,3);assert.sameValue(log,'0,2,3,');
// CASE: search-restores-negative-zero
var marker={},r={lastIndex:-0,exec:function(){assert.sameValue(this.lastIndex,0);this.lastIndex=8;return {index:marker};}};assert.sameValue(RegExp.prototype[Symbol.search].call(r,'abc'),marker);assert.sameValue(r.lastIndex,-0);
// CASE: search-does-not-restore-after-abrupt-exec
var marker={},caught,r={lastIndex:9,exec:function(){this.lastIndex=3;throw marker;}};try{RegExp.prototype[Symbol.search].call(r,'abc');}catch(e){caught=e;}assert.sameValue(caught,marker);assert.sameValue(r.lastIndex,3);
// CASE: match-result-string-conversion
var n=0,r={flags:'g',exec:function(){n++;return n==1?{0:{toString:function(){return 'x';}}}:null;}};assert.sameValue(RegExp.prototype[Symbol.match].call(r,'x')[0],'x');
// CASE: null-receiver-precedes-hook-getter
var n=0,p={};Object.defineProperty(p,Symbol.match,{get:function(){n++;throw 1;}});Object.defineProperty(p,Symbol.search,{get:function(){n++;throw 1;}});assert.throws(TypeError,function(){String.prototype.match.call(null,p);});assert.throws(TypeError,function(){String.prototype.search.call(undefined,p);});assert.sameValue(n,0);
// CASE: noncallable-hooks-throw
var p={};p[Symbol.match]=false;p[Symbol.search]=0;assert.throws(TypeError,function(){'a'.match(p);});assert.throws(TypeError,function(){'a'.search(p);});
// CASE: primitive-argument-prototypes-ignored
String.prototype[Symbol.match]=function(){throw 1;};String.prototype[Symbol.search]=function(){throw 2;};assert.sameValue('abc'.match('b')[0],'b');assert.sameValue('abc'.search('b'),1);
// CASE: fallback-conversion-order
var log='',r={toString:function(){log+='r';return 'abc';}},p={toString:function(){log+='p';return 'b';}};Object.defineProperty(p,Symbol.match,{get:function(){log+='m';return null;}});assert.sameValue(String.prototype.match.call(r,p)[0],'b');assert.sameValue(log,'mrp');
// CASE: actual-regexp-null-hook-uses-regexp-create
var r=/a/;r[Symbol.match]=null;r[Symbol.search]=null;r.toString=function(){return 'b';};assert.sameValue('abc'.match(r)[0],'b');assert.sameValue('abc'.search(r),1);
// CASE: nonglobal-match-returns-exec-object-unchanged
var out=function(){},r={flags:'',exec:function(s){assert.sameValue(s,'x');return out;}};Object.defineProperty(out,'0',{get:function(){throw 1;}});assert.sameValue(RegExp.prototype[Symbol.match].call(r,'x'),out);
// CASE: match-function-and-array-string-conversion
var f=function(){},flags=[],n=0,matched=[];f.toString=function(){return 'x';};flags.toString=function(){return 'g';};matched.toString=function(){return 'x';};var r={flags:flags,exec:function(s){assert.sameValue(s,'x');n++;return n==1?{0:matched}:null;}};assert.sameValue(RegExp.prototype[Symbol.match].call(r,f)[0],'x');
// CASE: match-result-ignores-array-prototype-setter
Object.defineProperty(Array.prototype,'0',{set:function(){throw 1;},configurable:true});var m=RegExp.prototype[Symbol.match].call(/./g,'ab');assert.sameValue(m.length,2);assert.sameValue(m[0],'a');assert.sameValue(m[1],'b');
// CASE: search-restores-before-reading-index
var r={lastIndex:7,exec:function(){this.lastIndex=9;var out={};Object.defineProperty(out,'index',{get:function(){assert.sameValue(r.lastIndex,7);return 'raw';}});return out;}};assert.sameValue(RegExp.prototype[Symbol.search].call(r,'x'),'raw');
// CASE: symbol-receiver-validation-before-conversion
var n=0,s={toString:function(){n++;throw 1;}};assert.throws(TypeError,function(){RegExp.prototype[Symbol.match].call(1,s);});assert.throws(TypeError,function(){RegExp.prototype[Symbol.search].call(null,s);});assert.sameValue(n,0);
// CASE: match-result-ignores-object-prototype-setter
Object.defineProperty(Object.prototype,'0',{set:function(){throw 1;},configurable:true});var m=RegExp.prototype[Symbol.match].call(/./g,'ab');assert.sameValue(m.length,2);assert.sameValue(m[0],'a');assert.sameValue(m[1],'b');
