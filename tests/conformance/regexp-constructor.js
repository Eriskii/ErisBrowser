// CASE: regexp-like-call-identity
var r={constructor:RegExp};r[Symbol.match]=true;assert.sameValue(RegExp(r),r);
// CASE: disabled-match-clones-real-regexp
var r=/x/g,n=0;r[Symbol.match]=false;Object.defineProperty(r,'constructor',{get:function(){n++;throw 1;}});var c=RegExp(r);assert.notSameValue(c,r);assert.sameValue(c.source,'x');assert.sameValue(c.flags,'g');assert.sameValue(n,0);
// CASE: regexp-like-source-and-flags
var r={source:'a',flags:'i'};r[Symbol.match]=true;var c=new RegExp(r);assert.sameValue(c.source,'a');assert.sameValue(c.flags,'i');assert.sameValue(c.test('A'),true);
// CASE: classify-get-allocate-convert-order
var log='',r={},nt=(function(){}).bind(null);Object.defineProperty(r,Symbol.match,{get:function(){log+='m';return true;}});Object.defineProperty(r,'source',{get:function(){log+='s';return {toString:function(){log+='t';return 'a';}};}});Object.defineProperty(r,'flags',{get:function(){log+='f';return {toString:function(){log+='u';return 'g';}};}});Object.defineProperty(nt,'prototype',{get:function(){log+='p';return Object.prototype;}});Reflect.construct(RegExp,[r],nt);assert.sameValue(log,'msfptu');
// CASE: match-abrupt-precedes-allocation
var marker={},caught,n=0,r={},nt=(function(){}).bind(null);Object.defineProperty(r,Symbol.match,{get:function(){throw marker;}});Object.defineProperty(nt,'prototype',{get:function(){n++;return Object.prototype;}});try{Reflect.construct(RegExp,[r],nt);}catch(e){caught=e;}assert.sameValue(caught,marker);assert.sameValue(n,0);
// CASE: explicit-flags-skip-pattern-flags
var r={source:'a'},n=0;r[Symbol.match]=true;Object.defineProperty(r,'flags',{get:function(){n++;throw 1;}});var c=new RegExp(r,'i');assert.sameValue(c.source,'a');assert.sameValue(c.flags,'i');assert.sameValue(n,0);
// CASE: function-source-and-array-flags-convert
var p=function(){},f=[];p.toString=function(){return 'a';};f.toString=function(){return 'g';};var c=new RegExp(p,f);assert.sameValue(c.source,'a');assert.sameValue(c.flags,'g');
// CASE: real-slot-copy-still-classifies
var n=0,r=/x/g;Object.defineProperty(r,Symbol.match,{get:function(){n++;return false;}});Object.defineProperty(r,'source',{get:function(){throw 1;}});Object.defineProperty(r,'flags',{get:function(){throw 1;}});var c=new RegExp(r);assert.sameValue(c.source,'x');assert.sameValue(c.flags,'g');assert.sameValue(n,1);
// CASE: prototype-get-before-ordinary-conversion
var log='',p={toString:function(){log+='s';return 'a';}},nt=(function(){}).bind(null);Object.defineProperty(nt,'prototype',{get:function(){log+='p';return Object.prototype;}});Reflect.construct(RegExp,[p],nt);assert.sameValue(log,'ps');
// CASE: default-split-regexp-like-source
var r={source:'b',flags:''};r[Symbol.match]=true;assert.sameValue(RegExp.prototype[Symbol.split].call(r,'abc').join('|'),'a|c');
// CASE: constructor-identity-getter-abrupt
var marker={},caught,r={};r[Symbol.match]=true;Object.defineProperty(r,'constructor',{get:function(){throw marker;}});try{RegExp(r);}catch(e){caught=e;}assert.sameValue(caught,marker);
// CASE: new-skips-constructor-getter
var r={source:'a',flags:'g'};r[Symbol.match]=true;Object.defineProperty(r,'constructor',{get:function(){throw 1;}});var c=new RegExp(r);assert.sameValue(c.source,'a');assert.sameValue(c.flags,'g');
// CASE: false-match-uses-ordinary-conversion
var r={toString:function(){return 'a';}},n=0;Object.defineProperty(r,Symbol.match,{get:function(){n++;return false;}});Object.defineProperty(r,'source',{get:function(){throw 1;}});Object.defineProperty(r,'flags',{get:function(){throw 1;}});var c=new RegExp(r);assert.sameValue(c.source,'a');assert.sameValue(n,1);
// CASE: flags-get-abrupt-before-source-conversion
var marker={},caught,n=0,r={source:{toString:function(){n++;return 'a';}}};r[Symbol.match]=true;Object.defineProperty(r,'flags',{get:function(){throw marker;}});try{new RegExp(r);}catch(e){caught=e;}assert.sameValue(caught,marker);assert.sameValue(n,0);
// CASE: explicit-flags-still-observe-classification
var marker={},caught,n=0,r={},flags={toString:function(){n++;return 'g';}};Object.defineProperty(r,Symbol.match,{get:function(){throw marker;}});try{new RegExp(r,flags);}catch(e){caught=e;}assert.sameValue(caught,marker);assert.sameValue(n,0);
// CASE: pattern-values-captured-before-prototype
var r={source:'a',flags:'g'},nt=(function(){}).bind(null);r[Symbol.match]=true;Object.defineProperty(nt,'prototype',{get:function(){r.source='b';r.flags='i';return RegExp.prototype;}});var c=Reflect.construct(RegExp,[r],nt);assert.sameValue(RegExp.prototype.toString.call(c),'/a/g');
// CASE: undefined-source-and-flags-defaults
var r={};r[Symbol.match]=true;var c=new RegExp(r);assert.sameValue(c.source,'(?:)');assert.sameValue(c.flags,'');var a=[];a.toString=function(){return '\ud800';};c=new RegExp(a);assert.sameValue(c.source.charCodeAt(0),0xd800);assert.sameValue(c.test('\ud800'),true);
// CASE: source-symbol-precedes-flag-conversion
var n=0,r={source:Symbol()},f={toString:function(){n++;return 'g';}};r[Symbol.match]=true;assert.throws(TypeError,function(){new RegExp(r,f);});assert.sameValue(n,0);
