// CASE: overlapping-and-position
assert.sameValue(typeof String.prototype.lastIndexOf,'function');
assert.sameValue('ababa'.lastIndexOf('aba'),2);
assert.sameValue('ababa'.lastIndexOf('aba',1),0);
assert.sameValue('ababa'.lastIndexOf('aba',2.9),2);
assert.sameValue('ababa'.lastIndexOf('x'),-1);
// CASE: empty-search-and-clamping
assert.sameValue(typeof String.prototype.lastIndexOf,'function');
assert.sameValue('abc'.lastIndexOf(''),3);
assert.sameValue('abc'.lastIndexOf('',Infinity),3);
assert.sameValue('abc'.lastIndexOf('',-Infinity),0);
assert.sameValue('abc'.lastIndexOf('',-0),0);
assert.sameValue('abc'.lastIndexOf('',1.9),1);
assert.sameValue(''.lastIndexOf(''),0);
assert.sameValue(''.lastIndexOf('a'),-1);
// CASE: nan-defaults-to-end
assert.sameValue(typeof String.prototype.lastIndexOf,'function');
assert.sameValue('aba'.lastIndexOf('a',NaN),2);
assert.sameValue('aba'.lastIndexOf('a',undefined),2);
assert.sameValue('aba'.lastIndexOf('a','no-number'),2);
assert.sameValue('aba'.lastIndexOf('a',null),0);
assert.sameValue('aba'.lastIndexOf('a',false),0);
assert.sameValue('aba'.lastIndexOf('a',true),0);
// CASE: omitted-search-value
assert.sameValue(typeof String.prototype.lastIndexOf,'function');
assert.sameValue('undefined-undefined'.lastIndexOf(),10);
assert.sameValue('null-null'.lastIndexOf(null),5);
assert.sameValue('123-123'.lastIndexOf(123),4);
// CASE: utf16-and-unpaired-surrogates
assert.sameValue(typeof String.prototype.lastIndexOf,'function');
assert.sameValue('A\ud83e\udd80B\ud83e\udd80'.lastIndexOf('\udd80'),5);
assert.sameValue('A\ud83e\udd80B\ud83e\udd80'.lastIndexOf('\ud83e',3),1);
assert.sameValue('A\ud83e\udd80B\ud83e\udd80'.lastIndexOf('\ud83e\udd80'),4);
assert.sameValue('\ud800x\ud800'.lastIndexOf('\ud800'),2);
assert.sameValue('\udfff'.lastIndexOf(''),1);
// CASE: generic-receiver-and-string-hints
assert.sameValue(typeof String.prototype.lastIndexOf,'function');
var log='';
var receiver={toString:function(){log+='r';return 'abcabc';},valueOf:function(){throw 'bad receiver hint';}};
var search={toString:function(){log+='s';return 'bc';},valueOf:function(){throw 'bad search hint';}};
var position={valueOf:function(){log+='p';return 3;},toString:function(){throw 'bad position hint';}};
assert.sameValue(String.prototype.lastIndexOf.call(receiver,search,position),1);
assert.sameValue(log,'rsp');
assert.sameValue(String.prototype.lastIndexOf.call(123123,123),3);
assert.sameValue(String.prototype.lastIndexOf.call(new String('ababa'),'aba'),2);
// CASE: primitive-symbol-hints
assert.sameValue(typeof String.prototype.lastIndexOf,'function');
var log='';var receiver={};var search={};var position={};
receiver[Symbol.toPrimitive]=function(h){log+='r'+h;return 'ababa';};
search[Symbol.toPrimitive]=function(h){log+='s'+h;return 'aba';};
position[Symbol.toPrimitive]=function(h){log+='p'+h;return NaN;};
assert.sameValue(String.prototype.lastIndexOf.call(receiver,search,position),2);
assert.sameValue(log,'rstringsstringpnumber');
// CASE: nullish-receiver-first
assert.sameValue(typeof String.prototype.lastIndexOf,'function');
var visits=0;var search={toString:function(){visits++;return 'x';}};
assert.throws(TypeError,function(){String.prototype.lastIndexOf.call(null,search);});
assert.throws(TypeError,function(){String.prototype.lastIndexOf.call(undefined,search);});
assert.sameValue(visits,0);
// CASE: receiver-error-stops-conversion
assert.sameValue(typeof String.prototype.lastIndexOf,'function');
var visits=0;var search={toString:function(){visits++;return 'x';}};
var receiver={toString:function(){throw new RangeError('receiver');}};
assert.throws(RangeError,function(){String.prototype.lastIndexOf.call(receiver,search);});
assert.sameValue(visits,0);
// CASE: search-error-stops-position
assert.sameValue(typeof String.prototype.lastIndexOf,'function');
var visits=0;var position={valueOf:function(){visits++;return 0;}};
var search={toString:function(){throw new RangeError('search');}};
assert.throws(RangeError,function(){'x'.lastIndexOf(search,position);});
assert.sameValue(visits,0);
// CASE: position-converts-before-shortcuts
assert.sameValue(typeof String.prototype.lastIndexOf,'function');
var position={valueOf:function(){throw new RangeError('position');}};
assert.throws(RangeError,function(){''.lastIndexOf('longer',position);});
assert.throws(RangeError,function(){''.lastIndexOf('',position);});
assert.throws(TypeError,function(){'x'.lastIndexOf('',Symbol());});
// CASE: symbol-string-errors
assert.sameValue(typeof String.prototype.lastIndexOf,'function');
assert.throws(TypeError,function(){String.prototype.lastIndexOf.call(Symbol(),'x');});
assert.throws(TypeError,function(){'x'.lastIndexOf(Symbol());});
// CASE: regexp-string-without-match-lookup
assert.sameValue(typeof String.prototype.lastIndexOf,'function');
var pattern=/x/;
Object.defineProperty(pattern,Symbol.match,{get:function(){throw 'unexpected IsRegExp';}});
assert.sameValue('/x/-/x/'.lastIndexOf(pattern),4);
// CASE: descriptors-and-construction
assert.sameValue(typeof String.prototype.lastIndexOf,'function');
verifyProperty(String.prototype,'lastIndexOf',{writable:true,enumerable:false,configurable:true},{restore:true});
verifyProperty(String.prototype.lastIndexOf,'length',{value:1,writable:false,enumerable:false,configurable:true});
verifyProperty(String.prototype.lastIndexOf,'name',{value:'lastIndexOf',writable:false,enumerable:false,configurable:true});
assert.sameValue(String.prototype.lastIndexOf.hasOwnProperty('prototype'),false);
assert.throws(TypeError,function(){new String.prototype.lastIndexOf();});
