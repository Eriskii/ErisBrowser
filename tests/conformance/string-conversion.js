// CASE: function-receiver-hooks
var log='',f=function(){};f.toString=function(){log+='t';return {};};f.valueOf=function(){log+='v';return 'abc';};
assert.sameValue(String.prototype.slice.call(f,1),'bc');assert.sameValue(log,'tv');
// CASE: array-receiver-hooks
var log='',a=[1,2];a.toString=function(){log+='t';return 'abc';};
assert.sameValue(String.prototype.slice.call(a,1),'bc');assert.sameValue(log,'t');
// CASE: function-noncallable-hook
var log='',f=function(){};f.toString=1;f.valueOf=function(){log+='v';return 'abc';};
assert.sameValue(String.prototype.charCodeAt.call(f,1),98);assert.sameValue(log,'v');
// CASE: array-abrupt-hook
var marker={},a=[],caught;a.toString=function(){throw marker;};
try{String.prototype.substring.call(a,1);}catch(e){caught=e;}assert.sameValue(caught,marker);
// CASE: ordinary-object-hooks
var log='',o={toString:function(){log+='t';return {};},valueOf:function(){log+='v';return 'abc';}};
assert.sameValue(String.prototype.slice.call(o,1),'bc');assert.sameValue(log,'tv');
// CASE: function-exotic-hook
var f=function(){};f[Symbol.toPrimitive]=function(h){assert.sameValue(h,'string');return 'abc';};
assert.sameValue(String.prototype.slice.call(f,1),'bc');
// CASE: receiver-across-methods
var f=function(){},count=0;f.toString=function(){assert.sameValue(this,f);count++;return 'abca';};
assert.sameValue(String.prototype.charAt.call(f,1),'b');assert.sameValue(String.prototype.codePointAt.call(f,1),98);
assert.sameValue(String.prototype.includes.call(f,'bc'),true);assert.sameValue(String.prototype.indexOf.call(f,'bc'),1);
assert.sameValue(String.prototype.startsWith.call(f,'ab'),true);assert.sameValue(String.prototype.endsWith.call(f,'ca'),true);
assert.sameValue(String.prototype.trim.call(f),'abca');assert.sameValue(String.prototype.toUpperCase.call(f),'ABCA');assert.sameValue(String.prototype.toLowerCase.call(f),'abca');
assert.sameValue(String.prototype.match.call(f,/bc/)[0],'bc');assert.sameValue(String.prototype.search.call(f,/bc/),1);
assert.sameValue(String.prototype.replace.call(f,'bc','X'),'aXa');assert.sameValue(String.prototype.split.call(f,'bc')[1],'a');assert.sameValue(count,13);
// CASE: inherited-and-bound-receiver-hooks
var base=function(){};base.toString=function(){return 'inherited';};var f=function(){};Object.setPrototypeOf(f,base);
assert.sameValue(String.prototype.substring.call(f,0,2),'in');
var bound=(function(){}).bind(null);bound.toString=function(){return 'bound';};assert.sameValue(String.prototype.charAt.call(bound,2),'u');
// CASE: nullish-receiver-precedence
var log='',arg={toString:function(){log+='a';return 'x';},valueOf:function(){log+='v';return 1;}};
assert.throws(TypeError,function(){String.prototype.slice.call(null,arg,arg);});assert.throws(TypeError,function(){String.prototype.includes.call(undefined,arg,arg);});
assert.sameValue(log,'');
// CASE: receiver-argument-order
var log='',f=function(){};f.toString=function(){log+='r';return 'abcdef';};
var start={valueOf:function(){log+='s';return 1;}},end={valueOf:function(){log+='e';return 4;}};
assert.sameValue(String.prototype.slice.call(f,start,end),'bcd');assert.sameValue(log,'rse');
// CASE: string-search-function-argument
var f=function(){};f.toString=function(){return 'bc';};
assert.sameValue('abcd'.includes(f),true);assert.sameValue('abcd'.indexOf(f),1);assert.sameValue('bcde'.startsWith(f),true);assert.sameValue('abc'.endsWith(f),true);
// CASE: string-search-array-argument
var a=[];a.toString=function(){return 'bc';};
assert.sameValue('abcd'.includes(a),true);assert.sameValue('abcd'.indexOf(a),1);assert.sameValue('bcde'.startsWith(a),true);assert.sameValue('abc'.endsWith(a),true);
// CASE: search-conversion-order
var log='',receiver={toString:function(){log+='r';return 'abc';}},needle=function(){},position={valueOf:function(){log+='p';return 1;}};
Object.defineProperty(needle,Symbol.match,{get:function(){log+='m';return false;}});needle.toString=function(){log+='s';return 'b';};
assert.sameValue(String.prototype.includes.call(receiver,needle,position),true);assert.sameValue(log,'rmsp');
log='';assert.sameValue(String.prototype.indexOf.call(receiver,needle,position),1);assert.sameValue(log,'rsp');
// CASE: search-regexp-reclassification
var methods=['includes','startsWith','endsWith'];
for(var i=0;i<methods.length;i++){
 var method=String.prototype[methods[i]],object={};object[Symbol.match]=true;
 assert.throws(TypeError,function(){method.call('x',object);});
 var re=/x/;re[Symbol.match]=false;re.toString=function(){return 'x';};assert.sameValue(method.call('x',re),true);
 re[Symbol.match]=undefined;assert.throws(TypeError,function(){method.call('x',re);});
}
// CASE: search-regexp-getter-abrupt
var marker={},caught,log='',needle={};Object.defineProperty(needle,Symbol.match,{get:function(){throw marker;}});
needle.toString=function(){log+='s';return 'x';};var p={valueOf:function(){log+='p';return 0;}};
try{'x'.startsWith(needle,p);}catch(e){caught=e;}assert.sameValue(caught,marker);assert.sameValue(log,'');
// CASE: utf16-hooks
var f=function(){};f.toString=function(){return '\ud800X\udfff';};
assert.sameValue(String.prototype.charCodeAt.call(f,0),0xd800);assert.sameValue(String.prototype.charCodeAt.call(f,2),0xdfff);
var a=[];a.toString=function(){return '\udfff';};assert.sameValue(String.prototype.includes.call(f,a),true);
assert.sameValue(String.prototype.slice.call(f,0,1).charCodeAt(0),0xd800);
// CASE: split-separator-and-limit-order
var log='',r=function(){},separator=function(){},limit={valueOf:function(){log+='l';return 2;}};
r.toString=function(){log+='r';return 'a,b,c';};separator.toString=function(){log+='s';return ',';};
var values=String.prototype.split.call(r,separator,limit);assert.sameValue(log,'rls');assert.sameValue(values.length,2);assert.sameValue(values[0],'a');assert.sameValue(values[1],'b');
// CASE: split-zero-still-converts-separator
var log='',s=[];s.toString=function(){log+='s';return ',';};var limit={valueOf:function(){log+='l';return 0;}};
assert.sameValue('a,b'.split(s,limit).length,0);assert.sameValue(log,'ls');
// CASE: split-limit-abrupt-before-separator
var marker={},caught,log='',s=function(){};s.toString=function(){log+='s';return ',';};
try{'a,b'.split(s,{valueOf:function(){throw marker;}});}catch(e){caught=e;}assert.sameValue(caught,marker);assert.sameValue(log,'');
// CASE: split-utf16-and-empty-fields
var a=[];a.toString=function(){return '';};var parts='\ud800\udfff'.split(a);assert.sameValue(parts.length,2);assert.sameValue(parts[0].charCodeAt(0),0xd800);assert.sameValue(parts[1].charCodeAt(0),0xdfff);
assert.sameValue(''.split(a).length,0);var s=function(){};s.toString=function(){return ',';};assert.sameValue(',a,'.split(s).length,3);assert.sameValue(',a,'.split(s)[2],'');
