// CASE: split-hook-raw-arguments
var receiver={},separator={},limit={},result={},log='';receiver.toString=function(){throw 'unused';};limit.valueOf=function(){throw 'unused';};
Object.defineProperty(separator,Symbol.split,{get:function(){log+='g';assert.sameValue(this,separator);return function(r,l){log+='c';assert.sameValue(this,separator);assert.sameValue(arguments.length,2);assert.sameValue(r,receiver);assert.sameValue(l,limit);return result;};}});
assert.sameValue(String.prototype.split.call(receiver,separator,limit),result);assert.sameValue(log,'gc');
// CASE: split-hook-abrupt-order
var marker={},caught,log='',receiver={toString:function(){log+='r';return 'abc';}},separator={};
Object.defineProperty(separator,Symbol.split,{get:function(){log+='g';throw marker;}});
try{String.prototype.split.call(receiver,separator);}catch(e){caught=e;}assert.sameValue(caught,marker);assert.sameValue(log,'g');
log='';assert.throws(TypeError,function(){String.prototype.split.call(null,separator);});assert.sameValue(log,'');
// CASE: split-hook-noncallable
var separator={};separator[Symbol.split]=1;var log='',receiver={toString:function(){log+='r';return 'abc';}};
assert.throws(TypeError,function(){String.prototype.split.call(receiver,separator);});assert.sameValue(log,'');
// CASE: split-hook-nullish-fallback
var log='',separator=[],receiver={toString:function(){log+='r';return 'a,b';}},limit={valueOf:function(){log+='l';return 2;}};
Object.defineProperty(separator,Symbol.split,{get:function(){log+='g';return null;}});separator.toString=function(){log+='s';return ',';};
var result=String.prototype.split.call(receiver,separator,limit);assert.sameValue(log,'grls');assert.sameValue(result.length,2);assert.sameValue(result[1],'b');
// CASE: split-hook-undefined-result
var separator={};separator[Symbol.split]=function(){return undefined;};assert.sameValue('abc'.split(separator),undefined);
separator[Symbol.split]=function(){return 3;};assert.sameValue('abc'.split(separator),3);
// CASE: split-hook-primitives-are-not-boxed
Object.defineProperty(String.prototype,Symbol.split,{get:function(){throw 'unused';}});
Object.defineProperty(Number.prototype,Symbol.split,{get:function(){throw 'unused';}});
Object.defineProperty(Boolean.prototype,Symbol.split,{get:function(){throw 'unused';}});
assert.sameValue('a1b'.split('1')[1],'b');assert.sameValue('a1b'.split(1)[1],'b');assert.sameValue('atrueb'.split(true)[1],'b');
// CASE: split-hook-on-regexp-override
var re=/x/,result={},receiver={toString:function(){throw 'unused';}};
re[Symbol.split]=function(r,l){assert.sameValue(this,re);assert.sameValue(r,receiver);assert.sameValue(l,7);return result;};
assert.sameValue(String.prototype.split.call(receiver,re,7),result);
