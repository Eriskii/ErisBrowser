// CASE: host-local-fields-0
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(0),got=[d.getFullYear(),d.getMonth(),d.getDate(),d.getDay(),d.getHours(),d.getMinutes(),d.getSeconds(),d.getMilliseconds(),d.getTimezoneOffset()],want=[1969,11,31,3,19,0,0,0,300.0];for(var i=0;i<want.length;i++)assert.sameValue(got[i],want[i]);
// CASE: host-local-fields--1
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(-1),got=[d.getFullYear(),d.getMonth(),d.getDate(),d.getDay(),d.getHours(),d.getMinutes(),d.getSeconds(),d.getMilliseconds(),d.getTimezoneOffset()],want=[1969,11,31,3,18,59,59,999,300.0];for(var i=0;i<want.length;i++)assert.sameValue(got[i],want[i]);
// CASE: host-local-fields-1579091696789
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1579091696789),got=[d.getFullYear(),d.getMonth(),d.getDate(),d.getDay(),d.getHours(),d.getMinutes(),d.getSeconds(),d.getMilliseconds(),d.getTimezoneOffset()],want=[2020,0,15,3,7,34,56,789,300.0];for(var i=0;i<want.length;i++)assert.sameValue(got[i],want[i]);
// CASE: host-local-fields-1594816496789
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1594816496789),got=[d.getFullYear(),d.getMonth(),d.getDate(),d.getDay(),d.getHours(),d.getMinutes(),d.getSeconds(),d.getMilliseconds(),d.getTimezoneOffset()],want=[2020,6,15,3,8,34,56,789,240.0];for(var i=0;i<want.length;i++)assert.sameValue(got[i],want[i]);
// CASE: host-local-fields-951868799999
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(951868799999),got=[d.getFullYear(),d.getMonth(),d.getDate(),d.getDay(),d.getHours(),d.getMinutes(),d.getSeconds(),d.getMilliseconds(),d.getTimezoneOffset()],want=[2000,1,29,2,18,59,59,999,300.0];for(var i=0;i<want.length;i++)assert.sameValue(got[i],want[i]);
// CASE: constructor-metadata
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
verifyProperty(globalThis,'Date',{value:Date,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(Date,'length',{value:7,writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(Date,'name',{value:'Date',writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(Date,'prototype',{value:Date.prototype,writable:false,enumerable:false,configurable:false},{restore:true});verifyProperty(Date.prototype,'constructor',{value:Date,writable:true,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(Date),Function.prototype);assert.sameValue(Object.getPrototypeOf(Date.prototype),Object.prototype);assert.sameValue(Object.prototype.toString.call(new Date(0)),'[object Date]');
// CASE: prototype-is-not-a-date-instance
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
assert.throws(TypeError,function(){Date.prototype.getTime.call(Date.prototype);});assert.throws(TypeError,function(){Date.prototype.valueOf.call(Object.create(Date.prototype));});assert.sameValue(Object.prototype.toString.call(Date.prototype),'[object Object]');
// CASE: primitive-constructors-and-fresh-identity
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var a=new Date(0),b=new Date(0);assert.notSameValue(a,b);assert.sameValue(new Date(null).getTime(),0);assert.sameValue(new Date(false).getTime(),0);assert.sameValue(new Date(true).getTime(),1);assert.sameValue(new Date(undefined).getTime(),NaN);assert.sameValue(new Date(NaN).getTime(),NaN);assert.sameValue(new Date(Infinity).getTime(),NaN);assert.sameValue(new Date(-Infinity).getTime(),NaN);assert.sameValue(new Date(-0).getTime(),0);
// CASE: clip-truncation-and-boundaries
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var input=[1.9,-1.9,0.9,-0.9,8640000000000000,-8640000000000000,8640000000000001,-8640000000000001],want=[1,-1,0,0,8640000000000000,-8640000000000000,NaN,NaN];for(var i=0;i<input.length;i++)assert.sameValue(new Date(input[i]).valueOf(),want[i]);
// CASE: date-slot-copy-bypasses-all-author-hooks
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1234),reads=0;d.valueOf=function(){reads++;throw 'valueOf';};d.toString=function(){reads++;throw 'toString';};Object.defineProperty(d,Symbol.toPrimitive,{get:function(){reads++;throw 'symbol';}});assert.sameValue(new Date(d).getTime(),1234);assert.sameValue(reads,0);var bad=new Date(NaN);bad.valueOf=function(){throw 'bad';};assert.sameValue(new Date(bad).getTime(),NaN);
// CASE: one-argument-default-hint-and-string-branch
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var log='',o={};o[Symbol.toPrimitive]=function(h){assert.sameValue(this,o);assert.sameValue(h,'default');log+='P';return '2000-01-01T00:00:00.000Z';};assert.sameValue(new Date(o).getTime(),946684800000);assert.sameValue(log,'P');
// CASE: ordinary-primitive-number-order
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var log='',o={valueOf:function(){log+='V';return {};},toString:function(){log+='S';return '1970-01-01T00:00:00Z';}};assert.sameValue(new Date(o).getTime(),0);assert.sameValue(log,'VS');var n={valueOf:function(){return 123;},toString:function(){throw 'late';}};assert.sameValue(new Date(n).getTime(),123);
// CASE: constructor-abrupt-symbol-and-nonprimitive
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var reason={},caught,o={valueOf:function(){throw reason;}};try{new Date(o);}catch(e){caught=e;}assert.sameValue(caught,reason);assert.throws(TypeError,function(){new Date(Symbol('x'));});o[Symbol.toPrimitive]=function(){return {};};assert.throws(TypeError,function(){new Date(o);});
// CASE: constructor-component-conversion-order-and-seven-limit
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var log='';function v(i,n){return {valueOf:function(){log+=i;return n;}};}var d=new Date(v(0,2020),v(1,0),v(2,2),v(3,3),v(4,4),v(5,5),v(6,6),{valueOf:function(){throw 'eighth';}});assert.sameValue(log,'0123456');assert.sameValue(d.getFullYear(),2020);assert.sameValue(d.getMonth(),0);assert.sameValue(d.getDate(),2);assert.sameValue(d.getHours(),3);assert.sameValue(d.getMilliseconds(),6);
// CASE: constructor-converts-later-components-after-nan
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var log='',reason={},caught;try{new Date({valueOf:function(){log+='Y';return NaN;}},{valueOf:function(){log+='M';throw reason;}});}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(log,'YM');
// CASE: year-zero-through-ninety-nine-constructor-adjustment
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
assert.sameValue(new Date(0,0,1).getFullYear(),1900);assert.sameValue(new Date(99,0,1).getFullYear(),1999);assert.sameValue(new Date(100,0,1).getFullYear(),100);assert.sameValue(new Date(-1,0,1).getFullYear(),-1);assert.sameValue(new Date(2020,0).getDate(),1);assert.sameValue(new Date(2020,0,undefined).getTime(),NaN);
// CASE: constructor-overflow-local-components
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(2020,12,0,24,0,0,-1);assert.sameValue(d.getFullYear(),2020);assert.sameValue(d.getMonth(),11);assert.sameValue(d.getDate(),31);assert.sameValue(d.getHours(),23);assert.sameValue(d.getMinutes(),59);assert.sameValue(d.getSeconds(),59);assert.sameValue(d.getMilliseconds(),999);
// CASE: reflect-construction-prototype-after-conversions
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var trace='',reason={},prototype={},arg={valueOf:function(){trace+='V';return 1;}};var target=(function(){}).bind(null);Object.defineProperty(target,'prototype',{get:function(){trace+='P';return prototype;}});var d=Reflect.construct(Date,[arg],target);assert.sameValue(trace,'VP');assert.sameValue(Object.getPrototypeOf(d),prototype);assert.sameValue(Date.prototype.getTime.call(d),1);
// CASE: reflect-prototype-abrupt-after-conversion
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var log='',reason={},caught,target=(function(){}).bind(null);Object.defineProperty(target,'prototype',{get:function(){log+='P';throw reason;}});try{Reflect.construct(Date,[{valueOf:function(){log+='V';return 7;}}],target);}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(log,'VP');
// CASE: reflect-primitive-prototype-fallback
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
function T(){}T.prototype=1;var d=Reflect.construct(Date,[0],T);assert.sameValue(Object.getPrototypeOf(d),Date.prototype);assert.sameValue(d.getTime(),0);
// CASE: constructor-alias-call-apply-bind-and-global-replacement
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var D=Date,proto=Date.prototype,B=D.bind(null,42);assert.sameValue(new B().getTime(),42);assert.sameValue(typeof D.call({},123),'string');assert.sameValue(typeof D.apply(null,[123]),'string');globalThis.Date=1;var d=new D(9);assert.sameValue(proto.getTime.call(d),9);assert.sameValue(Object.getPrototypeOf(d),proto);
// CASE: date-call-ignores-coercion-and-receiver
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var reads=0,p={valueOf:function(){reads++;throw 'number';},toString:function(){reads++;throw 'string';}};Object.defineProperty(p,Symbol.toPrimitive,{get:function(){reads++;throw 'primitive';}});assert.sameValue(typeof Date.call(p,p,p),'string');assert.sameValue(reads,0);
// CASE: date-now-brackets-zero-argument-construction
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
assert.sameValue(typeof Date.now,'function');var first=Date.now(),d=new Date(),last=Date.now();assert.sameValue(typeof first,'number');assert.sameValue(Number.isInteger(first),true);assert.sameValue(d.getTime()>=first-1000&&d.getTime()<=last+1000,true);assert.sameValue(last>=first-1000,true);
// CASE: date-now-ignored-values-and-receiver
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var poison={valueOf:function(){throw 'ignored';},toString:function(){throw 'ignored';}};assert.sameValue(typeof Date.now.call(poison,poison),'number');
// CASE: UTC-defaults-year-and-clipping
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
assert.sameValue(Date.UTC(),NaN);assert.sameValue(Date.UTC(undefined),NaN);assert.sameValue(Date.UTC(1970),0);assert.sameValue(Date.UTC(99,0,1),915148800000);assert.sameValue(Date.UTC(0,0,1),-2208988800000);assert.sameValue(Date.UTC(2000,1,29),951782400000);assert.sameValue(Date.UTC(2001,1,29),983404800000);assert.sameValue(Date.UTC(1970,0,1,0,0,0,-1),-1);assert.sameValue(Date.UTC(Infinity,0),NaN);
// CASE: UTC-conversion-order-and-ignored-extra
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var log='';function x(i,n){return {valueOf:function(){log+=i;return n;}};}assert.sameValue(Date.UTC(x(0,1970),x(1,0),x(2,1),x(3,0),x(4,0),x(5,0),x(6,7),{valueOf:function(){throw 'eighth';}}),7);assert.sameValue(log,'0123456');
// CASE: UTC-symbol-and-throw-conversion-identity
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var reason={},caught;try{Date.UTC(NaN,{valueOf:function(){throw reason;}});}catch(e){caught=e;}assert.sameValue(caught,reason);assert.throws(TypeError,function(){Date.UTC(Symbol('y'));});
// CASE: parse-string-hint-and-abrupt-order
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
assert.sameValue(Date.parse('1970-01-01T00:00:00Z'),0);var trace='',o={valueOf:function(){throw 'number';},toString:function(){trace+='S';return '2000-01-01';}};assert.sameValue(Date.parse(o),946684800000);assert.sameValue(trace,'S');var reason={},caught;o[Symbol.toPrimitive]=function(h){assert.sameValue(h,'string');throw reason;};try{Date.parse(o);}catch(e){caught=e;}assert.sameValue(caught,reason);assert.throws(TypeError,function(){Date.parse(Symbol('x'));});
// CASE: ISO-parse-epoch
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
assert.sameValue(Date.parse("1970-01-01"),0);assert.sameValue(new Date("1970-01-01").getTime(),0);
// CASE: ISO-parse-negative
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
assert.sameValue(Date.parse("1969-12-31T23:59:59.999Z"),-1);assert.sameValue(new Date("1969-12-31T23:59:59.999Z").getTime(),-1);
// CASE: ISO-parse-offset-plus
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
assert.sameValue(Date.parse("2000-01-01T05:30:00+05:30"),946684800000);assert.sameValue(new Date("2000-01-01T05:30:00+05:30").getTime(),946684800000);
// CASE: ISO-parse-offset-minus
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
assert.sameValue(Date.parse("1999-12-31T19:00:00-05:00"),946684800000);assert.sameValue(new Date("1999-12-31T19:00:00-05:00").getTime(),946684800000);
// CASE: ISO-parse-24-hour
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
assert.sameValue(Date.parse("1999-12-31T24:00:00.000Z"),946684800000);assert.sameValue(new Date("1999-12-31T24:00:00.000Z").getTime(),946684800000);
// CASE: ISO-parse-year-zero
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
assert.sameValue(Date.parse("0000-01-01T00:00:00.000Z"),-62167219200000);assert.sameValue(new Date("0000-01-01T00:00:00.000Z").getTime(),-62167219200000);
// CASE: ISO-parse-expanded-positive
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
assert.sameValue(Date.parse("+010000-01-01T00:00:00.000Z"),253402300800000);assert.sameValue(new Date("+010000-01-01T00:00:00.000Z").getTime(),253402300800000);
// CASE: ISO-parse-expanded-negative
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
assert.sameValue(Date.parse("-000001-01-01T00:00:00.000Z"),-62198755200000);assert.sameValue(new Date("-000001-01-01T00:00:00.000Z").getTime(),-62198755200000);
// CASE: ISO-parse-minimum
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
assert.sameValue(Date.parse("-271821-04-20T00:00:00.000Z"),-8640000000000000);assert.sameValue(new Date("-271821-04-20T00:00:00.000Z").getTime(),-8640000000000000);
// CASE: ISO-parse-maximum
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
assert.sameValue(Date.parse("+275760-09-13T00:00:00.000Z"),8640000000000000);assert.sameValue(new Date("+275760-09-13T00:00:00.000Z").getTime(),8640000000000000);
// CASE: ISO-mandatory-rejections
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var values=['-000000-01-01T00:00:00Z','+275760-09-13T00:00:00.001Z','-271821-04-19T23:59:59.999Z','2000-13-01T00:00:00Z','2000-01-32T00:00:00Z','2000-01-01T24:00:00.001Z','2000-01-01T00:60:00Z','2000-01-01T00:00:60Z'];for(var i=0;i<values.length;i++)assert.sameValue(Date.parse(values[i]),NaN);
// CASE: UTC-ISO-and-string-roundtrip-required-forms
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var values=[0,946684800000,-2208988800000,253402300800000,-62167219200000];for(var i=0;i<values.length;i++){var d=new Date(values[i]);assert.sameValue(Date.parse(d.toISOString()),values[i]);assert.sameValue(Date.parse(d.toUTCString()),values[i]);assert.sameValue(Date.parse(d.toString()),values[i]);}
// CASE: exact-ISO-format-boundaries
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var values=[0,-1,-62167219200000,253402300800000,-8640000000000000,8640000000000000],want=['1970-01-01T00:00:00.000Z','1969-12-31T23:59:59.999Z','0000-01-01T00:00:00.000Z','+010000-01-01T00:00:00.000Z','-271821-04-20T00:00:00.000Z','+275760-09-13T00:00:00.000Z'];for(var i=0;i<values.length;i++)assert.sameValue(new Date(values[i]).toISOString(),want[i]);
// CASE: invalid-date-formatting-and-ISO-rangeerror
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(NaN),names=['toString','toDateString','toTimeString','toUTCString','toGMTString','toLocaleString','toLocaleDateString','toLocaleTimeString'];for(var i=0;i<names.length;i++){assert.sameValue(typeof Date.prototype[names[i]],'function');assert.sameValue(d[names[i]](),'Invalid Date');}assert.throws(RangeError,function(){d.toISOString();});assert.sameValue(d.toJSON(),null);
// CASE: UTC-string-known-weekday
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
assert.sameValue(new Date(0).toUTCString(),'Thu, 01 Jan 1970 00:00:00 GMT');assert.sameValue(new Date(946684800000).toUTCString(),'Sat, 01 Jan 2000 00:00:00 GMT');
// CASE: native-getTime-valueOf-are-distinct
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
assert.notSameValue(Date.prototype.getTime,Date.prototype.valueOf);assert.sameValue(new Date(321).valueOf(),321);assert.sameValue(Date.prototype.getTime.call(new Date(321)),321);
// CASE: UTC-getter-getUTCFullYear
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006);assert.sameValue(d.getUTCFullYear(),2020);var old=d.getUTCFullYear;Object.defineProperty(d,'valueOf',{get:function(){throw 'hook';}});assert.sameValue(old.call(d),2020);assert.sameValue(old.call(new Date(NaN)),NaN);
// CASE: UTC-getter-getUTCMonth
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006);assert.sameValue(d.getUTCMonth(),0);var old=d.getUTCMonth;Object.defineProperty(d,'valueOf',{get:function(){throw 'hook';}});assert.sameValue(old.call(d),0);assert.sameValue(old.call(new Date(NaN)),NaN);
// CASE: UTC-getter-getUTCDate
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006);assert.sameValue(d.getUTCDate(),2);var old=d.getUTCDate;Object.defineProperty(d,'valueOf',{get:function(){throw 'hook';}});assert.sameValue(old.call(d),2);assert.sameValue(old.call(new Date(NaN)),NaN);
// CASE: UTC-getter-getUTCDay
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006);assert.sameValue(d.getUTCDay(),4);var old=d.getUTCDay;Object.defineProperty(d,'valueOf',{get:function(){throw 'hook';}});assert.sameValue(old.call(d),4);assert.sameValue(old.call(new Date(NaN)),NaN);
// CASE: UTC-getter-getUTCHours
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006);assert.sameValue(d.getUTCHours(),3);var old=d.getUTCHours;Object.defineProperty(d,'valueOf',{get:function(){throw 'hook';}});assert.sameValue(old.call(d),3);assert.sameValue(old.call(new Date(NaN)),NaN);
// CASE: UTC-getter-getUTCMinutes
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006);assert.sameValue(d.getUTCMinutes(),4);var old=d.getUTCMinutes;Object.defineProperty(d,'valueOf',{get:function(){throw 'hook';}});assert.sameValue(old.call(d),4);assert.sameValue(old.call(new Date(NaN)),NaN);
// CASE: UTC-getter-getUTCSeconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006);assert.sameValue(d.getUTCSeconds(),5);var old=d.getUTCSeconds;Object.defineProperty(d,'valueOf',{get:function(){throw 'hook';}});assert.sameValue(old.call(d),5);assert.sameValue(old.call(new Date(NaN)),NaN);
// CASE: UTC-getter-getUTCMilliseconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006);assert.sameValue(d.getUTCMilliseconds(),6);var old=d.getUTCMilliseconds;Object.defineProperty(d,'valueOf',{get:function(){throw 'hook';}});assert.sameValue(old.call(d),6);assert.sameValue(old.call(new Date(NaN)),NaN);
// CASE: UTC-getter-getTime
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006);assert.sameValue(d.getTime(),1577934245006);var old=d.getTime;Object.defineProperty(d,'valueOf',{get:function(){throw 'hook';}});assert.sameValue(old.call(d),1577934245006);assert.sameValue(old.call(new Date(NaN)),NaN);
// CASE: UTC-getter-valueOf
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006);assert.sameValue(d.valueOf(),1577934245006);var old=d.valueOf;Object.defineProperty(d,'valueOf',{get:function(){throw 'hook';}});assert.sameValue(old.call(d),1577934245006);assert.sameValue(old.call(new Date(NaN)),NaN);
// CASE: metadata-getDate
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getDate"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"getDate",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"getDate",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-getDate
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getDate"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-getDay
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getDay"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"getDay",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"getDay",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-getDay
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getDay"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-getFullYear
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getFullYear"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"getFullYear",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"getFullYear",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-getFullYear
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getFullYear"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-getHours
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getHours"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"getHours",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"getHours",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-getHours
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getHours"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-getMilliseconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getMilliseconds"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"getMilliseconds",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"getMilliseconds",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-getMilliseconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getMilliseconds"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-getMinutes
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getMinutes"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"getMinutes",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"getMinutes",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-getMinutes
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getMinutes"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-getMonth
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getMonth"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"getMonth",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"getMonth",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-getMonth
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getMonth"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-getSeconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getSeconds"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"getSeconds",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"getSeconds",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-getSeconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getSeconds"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-getUTCDate
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getUTCDate"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"getUTCDate",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"getUTCDate",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-getUTCDate
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getUTCDate"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-getUTCDay
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getUTCDay"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"getUTCDay",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"getUTCDay",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-getUTCDay
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getUTCDay"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-getUTCFullYear
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getUTCFullYear"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"getUTCFullYear",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"getUTCFullYear",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-getUTCFullYear
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getUTCFullYear"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-getUTCHours
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getUTCHours"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"getUTCHours",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"getUTCHours",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-getUTCHours
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getUTCHours"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-getUTCMilliseconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getUTCMilliseconds"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"getUTCMilliseconds",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"getUTCMilliseconds",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-getUTCMilliseconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getUTCMilliseconds"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-getUTCMinutes
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getUTCMinutes"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"getUTCMinutes",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"getUTCMinutes",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-getUTCMinutes
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getUTCMinutes"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-getUTCMonth
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getUTCMonth"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"getUTCMonth",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"getUTCMonth",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-getUTCMonth
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getUTCMonth"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-getUTCSeconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getUTCSeconds"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"getUTCSeconds",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"getUTCSeconds",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-getUTCSeconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getUTCSeconds"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-getTime
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getTime"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"getTime",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"getTime",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-getTime
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getTime"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-getTimezoneOffset
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getTimezoneOffset"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"getTimezoneOffset",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"getTimezoneOffset",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-getTimezoneOffset
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getTimezoneOffset"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-valueOf
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["valueOf"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"valueOf",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"valueOf",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-valueOf
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["valueOf"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-toDateString
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["toDateString"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"toDateString",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"toDateString",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-toDateString
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["toDateString"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-toISOString
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["toISOString"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"toISOString",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"toISOString",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-toISOString
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["toISOString"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-toLocaleDateString
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["toLocaleDateString"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"toLocaleDateString",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"toLocaleDateString",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-toLocaleDateString
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["toLocaleDateString"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-toLocaleString
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["toLocaleString"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"toLocaleString",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"toLocaleString",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-toLocaleString
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["toLocaleString"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-toLocaleTimeString
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["toLocaleTimeString"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"toLocaleTimeString",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"toLocaleTimeString",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-toLocaleTimeString
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["toLocaleTimeString"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-toString
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["toString"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"toString",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"toString",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-toString
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["toString"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-toTimeString
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["toTimeString"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"toTimeString",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"toTimeString",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-toTimeString
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["toTimeString"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-toUTCString
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["toUTCString"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"toUTCString",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"toUTCString",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-toUTCString
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["toUTCString"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-setDate
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setDate"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);verifyProperty(Date.prototype,"setDate",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"setDate",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-setDate
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setDate"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-setFullYear
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setFullYear"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);verifyProperty(Date.prototype,"setFullYear",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"setFullYear",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:3,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-setFullYear
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setFullYear"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-setHours
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setHours"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);verifyProperty(Date.prototype,"setHours",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"setHours",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:4,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-setHours
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setHours"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-setMilliseconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setMilliseconds"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);verifyProperty(Date.prototype,"setMilliseconds",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"setMilliseconds",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-setMilliseconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setMilliseconds"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-setMinutes
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setMinutes"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);verifyProperty(Date.prototype,"setMinutes",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"setMinutes",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:3,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-setMinutes
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setMinutes"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-setMonth
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setMonth"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);verifyProperty(Date.prototype,"setMonth",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"setMonth",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:2,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-setMonth
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setMonth"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-setSeconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setSeconds"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);verifyProperty(Date.prototype,"setSeconds",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"setSeconds",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:2,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-setSeconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setSeconds"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-setTime
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setTime"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);verifyProperty(Date.prototype,"setTime",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"setTime",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-setTime
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setTime"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-setUTCDate
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setUTCDate"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);verifyProperty(Date.prototype,"setUTCDate",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"setUTCDate",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-setUTCDate
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setUTCDate"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-setUTCFullYear
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setUTCFullYear"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);verifyProperty(Date.prototype,"setUTCFullYear",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"setUTCFullYear",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:3,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-setUTCFullYear
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setUTCFullYear"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-setUTCHours
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setUTCHours"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);verifyProperty(Date.prototype,"setUTCHours",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"setUTCHours",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:4,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-setUTCHours
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setUTCHours"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-setUTCMilliseconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setUTCMilliseconds"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);verifyProperty(Date.prototype,"setUTCMilliseconds",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"setUTCMilliseconds",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-setUTCMilliseconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setUTCMilliseconds"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-setUTCMinutes
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setUTCMinutes"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);verifyProperty(Date.prototype,"setUTCMinutes",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"setUTCMinutes",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:3,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-setUTCMinutes
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setUTCMinutes"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-setUTCMonth
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setUTCMonth"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);verifyProperty(Date.prototype,"setUTCMonth",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"setUTCMonth",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:2,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-setUTCMonth
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setUTCMonth"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-setUTCSeconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setUTCSeconds"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);verifyProperty(Date.prototype,"setUTCSeconds",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"setUTCSeconds",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:2,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-setUTCSeconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setUTCSeconds"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-toJSON
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["toJSON"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"toJSON",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"toJSON",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: metadata-getYear
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getYear"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"getYear",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"getYear",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-getYear
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["getYear"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-setYear
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setYear"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);verifyProperty(Date.prototype,"setYear",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"setYear",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-setYear
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["setYear"];assert.sameValue(typeof m,'function');m.call(new Date(0),0);var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: metadata-toGMTString
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["toGMTString"];assert.sameValue(typeof m,'function');m.call(new Date(0));verifyProperty(Date.prototype,"toGMTString",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"toUTCString",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: brand-before-coercion-toGMTString
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype["toGMTString"];assert.sameValue(typeof m,'function');m.call(new Date(0));var values=[undefined,null,true,0,'x',Symbol('x'),{},[],Date.prototype,Object.create(Date.prototype)],coercions=0;var poison={valueOf:function(){coercions++;throw 'number';},toString:function(){coercions++;throw 'string';}};for(var i=0;i<values.length;i++){assert.throws(TypeError,function(){m.call(values[i],poison,poison,poison,poison);});}assert.sameValue(coercions,0);
// CASE: static-metadata-now
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.now;assert.sameValue(typeof m,'function');assert.sameValue(typeof m(),'number');verifyProperty(Date,"now",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"now",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: static-metadata-parse
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.parse;assert.sameValue(typeof m,'function');assert.sameValue(typeof m('1970-01-01'),'number');verifyProperty(Date,"parse",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"parse",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: static-metadata-UTC
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.UTC;assert.sameValue(typeof m,'function');assert.sameValue(typeof m(1970),'number');verifyProperty(Date,"UTC",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:"UTC",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:7,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});
// CASE: setter-fields-setUTCDate
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006),m=Date.prototype.setUTCDate;assert.sameValue(m.apply(d,[15]),1579057445006);assert.sameValue(d.getTime(),1579057445006);
// CASE: setter-coercion-and-extra-setUTCDate
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006),log='',m=Date.prototype.setUTCDate;m.call(new Date(0),0);function p(i,n){return {valueOf:function(){log+=i;return n;}};}var args=[15],converted=[];for(var i=0;i<args.length;i++)converted.push(p(i,args[i]));converted.push({valueOf:function(){throw 'extra';}});assert.sameValue(m.apply(d,converted),1579057445006);assert.sameValue(log,"0");
// CASE: setter-abrupt-preserves-reentrant-effect-setUTCDate
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006),reason={},caught,m=Date.prototype.setUTCDate;m.call(new Date(0),0);try{m.call(d,{valueOf:function(){d.setTime(77);throw reason;}});}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(d.getTime(),77);
// CASE: setter-fields-setUTCFullYear
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006),m=Date.prototype.setUTCFullYear;assert.sameValue(m.apply(d,[2001,1,3]),981169445006);assert.sameValue(d.getTime(),981169445006);
// CASE: setter-coercion-and-extra-setUTCFullYear
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006),log='',m=Date.prototype.setUTCFullYear;m.call(new Date(0),0);function p(i,n){return {valueOf:function(){log+=i;return n;}};}var args=[2001,1,3],converted=[];for(var i=0;i<args.length;i++)converted.push(p(i,args[i]));converted.push({valueOf:function(){throw 'extra';}});assert.sameValue(m.apply(d,converted),981169445006);assert.sameValue(log,"012");
// CASE: setter-abrupt-preserves-reentrant-effect-setUTCFullYear
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006),reason={},caught,m=Date.prototype.setUTCFullYear;m.call(new Date(0),0);try{m.call(d,{valueOf:function(){d.setTime(77);throw reason;}});}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(d.getTime(),77);
// CASE: setter-fields-setUTCHours
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006),m=Date.prototype.setUTCHours;assert.sameValue(m.apply(d,[20,21,22,23]),1577996482023);assert.sameValue(d.getTime(),1577996482023);
// CASE: setter-coercion-and-extra-setUTCHours
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006),log='',m=Date.prototype.setUTCHours;m.call(new Date(0),0);function p(i,n){return {valueOf:function(){log+=i;return n;}};}var args=[20,21,22,23],converted=[];for(var i=0;i<args.length;i++)converted.push(p(i,args[i]));converted.push({valueOf:function(){throw 'extra';}});assert.sameValue(m.apply(d,converted),1577996482023);assert.sameValue(log,"0123");
// CASE: setter-abrupt-preserves-reentrant-effect-setUTCHours
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006),reason={},caught,m=Date.prototype.setUTCHours;m.call(new Date(0),0);try{m.call(d,{valueOf:function(){d.setTime(77);throw reason;}});}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(d.getTime(),77);
// CASE: setter-fields-setUTCMilliseconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006),m=Date.prototype.setUTCMilliseconds;assert.sameValue(m.apply(d,[999]),1577934245999);assert.sameValue(d.getTime(),1577934245999);
// CASE: setter-coercion-and-extra-setUTCMilliseconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006),log='',m=Date.prototype.setUTCMilliseconds;m.call(new Date(0),0);function p(i,n){return {valueOf:function(){log+=i;return n;}};}var args=[999],converted=[];for(var i=0;i<args.length;i++)converted.push(p(i,args[i]));converted.push({valueOf:function(){throw 'extra';}});assert.sameValue(m.apply(d,converted),1577934245999);assert.sameValue(log,"0");
// CASE: setter-abrupt-preserves-reentrant-effect-setUTCMilliseconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006),reason={},caught,m=Date.prototype.setUTCMilliseconds;m.call(new Date(0),0);try{m.call(d,{valueOf:function(){d.setTime(77);throw reason;}});}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(d.getTime(),77);
// CASE: setter-fields-setUTCMinutes
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006),m=Date.prototype.setUTCMinutes;assert.sameValue(m.apply(d,[30,31,32]),1577935831032);assert.sameValue(d.getTime(),1577935831032);
// CASE: setter-coercion-and-extra-setUTCMinutes
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006),log='',m=Date.prototype.setUTCMinutes;m.call(new Date(0),0);function p(i,n){return {valueOf:function(){log+=i;return n;}};}var args=[30,31,32],converted=[];for(var i=0;i<args.length;i++)converted.push(p(i,args[i]));converted.push({valueOf:function(){throw 'extra';}});assert.sameValue(m.apply(d,converted),1577935831032);assert.sameValue(log,"012");
// CASE: setter-abrupt-preserves-reentrant-effect-setUTCMinutes
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006),reason={},caught,m=Date.prototype.setUTCMinutes;m.call(new Date(0),0);try{m.call(d,{valueOf:function(){d.setTime(77);throw reason;}});}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(d.getTime(),77);
// CASE: setter-fields-setUTCMonth
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006),m=Date.prototype.setUTCMonth;assert.sameValue(m.apply(d,[5,15]),1592190245006);assert.sameValue(d.getTime(),1592190245006);
// CASE: setter-coercion-and-extra-setUTCMonth
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006),log='',m=Date.prototype.setUTCMonth;m.call(new Date(0),0);function p(i,n){return {valueOf:function(){log+=i;return n;}};}var args=[5,15],converted=[];for(var i=0;i<args.length;i++)converted.push(p(i,args[i]));converted.push({valueOf:function(){throw 'extra';}});assert.sameValue(m.apply(d,converted),1592190245006);assert.sameValue(log,"01");
// CASE: setter-abrupt-preserves-reentrant-effect-setUTCMonth
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006),reason={},caught,m=Date.prototype.setUTCMonth;m.call(new Date(0),0);try{m.call(d,{valueOf:function(){d.setTime(77);throw reason;}});}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(d.getTime(),77);
// CASE: setter-fields-setUTCSeconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006),m=Date.prototype.setUTCSeconds;assert.sameValue(m.apply(d,[40,41]),1577934280041);assert.sameValue(d.getTime(),1577934280041);
// CASE: setter-coercion-and-extra-setUTCSeconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006),log='',m=Date.prototype.setUTCSeconds;m.call(new Date(0),0);function p(i,n){return {valueOf:function(){log+=i;return n;}};}var args=[40,41],converted=[];for(var i=0;i<args.length;i++)converted.push(p(i,args[i]));converted.push({valueOf:function(){throw 'extra';}});assert.sameValue(m.apply(d,converted),1577934280041);assert.sameValue(log,"01");
// CASE: setter-abrupt-preserves-reentrant-effect-setUTCSeconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006),reason={},caught,m=Date.prototype.setUTCSeconds;m.call(new Date(0),0);try{m.call(d,{valueOf:function(){d.setTime(77);throw reason;}});}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(d.getTime(),77);
// CASE: setter-fields-setTime
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006),m=Date.prototype.setTime;assert.sameValue(m.apply(d,[-1.9]),-1);assert.sameValue(d.getTime(),-1);
// CASE: setter-coercion-and-extra-setTime
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006),log='',m=Date.prototype.setTime;m.call(new Date(0),0);function p(i,n){return {valueOf:function(){log+=i;return n;}};}var args=[-1.9],converted=[];for(var i=0;i<args.length;i++)converted.push(p(i,args[i]));converted.push({valueOf:function(){throw 'extra';}});assert.sameValue(m.apply(d,converted),-1);assert.sameValue(log,"0");
// CASE: setter-abrupt-preserves-reentrant-effect-setTime
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006),reason={},caught,m=Date.prototype.setTime;m.call(new Date(0),0);try{m.call(d,{valueOf:function(){d.setTime(77);throw reason;}});}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(d.getTime(),77);
// CASE: setter-uses-initial-time-snapshot
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577934245006);var n={valueOf:function(){d.setTime(0);return 2021;}};assert.sameValue(d.setUTCFullYear(n),1609556645006);d=new Date(1577934245006);assert.sameValue(d.setUTCSeconds({valueOf:function(){d.setTime(0);return 9;}}),1577934249006);
// CASE: setter-invalid-snapshot-preserves-reentrant-slot
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(NaN);assert.sameValue(d.setUTCSeconds({valueOf:function(){d.setTime(77);return 9;}}),NaN);assert.sameValue(d.getTime(),77);
// CASE: setter-optional-undefined-and-omitted
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var a=new Date(1577934245006),b=new Date(1577934245006);assert.sameValue(a.setUTCFullYear(2021),1609556645006);assert.sameValue(b.setUTCFullYear(2021,undefined),NaN);assert.sameValue(b.getTime(),NaN);var c=new Date(1577934245006);assert.sameValue(c.setUTCSeconds(),NaN);
// CASE: setter-invalid-date-still-converts-all-supplied-components
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(NaN),log='';function p(n){return {valueOf:function(){log+=n;return 1;}};}assert.sameValue(d.setUTCHours(p('H'),p('M'),p('S'),p('m')),NaN);assert.sameValue(log,'HMSm');assert.sameValue(d.getTime(),NaN);
// CASE: setter-invalid-full-year-restores-epoch-fields
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(NaN);assert.sameValue(d.setUTCFullYear(2000),946684800000);assert.sameValue(d.toISOString(),'2000-01-01T00:00:00.000Z');d=new Date(NaN);assert.sameValue(d.setFullYear(2000,0,1),new Date(2000,0,1).getTime());
// CASE: setter-overflow-and-year-no-adjustment
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(0);assert.sameValue(d.setUTCFullYear(0),-62167219200000);assert.sameValue(d.getUTCFullYear(),0);d=new Date(0);assert.sameValue(d.setUTCMonth(-1),-2678400000);d=new Date(0);assert.sameValue(d.setUTCMilliseconds(-1),-1);
// CASE: setter-symbol-fails-without-overwrite
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(123);assert.throws(TypeError,function(){d.setTime(Symbol('x'));});assert.sameValue(d.getTime(),123);assert.throws(TypeError,function(){d.setUTCFullYear(Symbol('x'));});assert.sameValue(d.getTime(),123);
// CASE: frozen-date-internal-slot-remains-writable
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(0);Object.freeze(d);assert.sameValue(d.setTime(123),123);assert.sameValue(d.getTime(),123);assert.sameValue(Object.isFrozen(d),true);
// CASE: date-slot-survives-prototype-change
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(0),get=Date.prototype.getTime,set=Date.prototype.setTime;Object.setPrototypeOf(d,null);assert.sameValue(set.call(d,42),42);assert.sameValue(get.call(d),42);assert.sameValue(Object.getPrototypeOf(d),null);
// CASE: host-local-constructor-and-unzoned-ISO
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
assert.sameValue(new Date(2020,0,2,3,4,5,6).getTime(),1577952245006);assert.sameValue(Date.parse('2020-01-02T03:04:05.006'),1577952245006);assert.sameValue(Date.parse('2020-01-02'),1577923200000);
// CASE: host-local-setter-setDate
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577952245006),m=Date.prototype.setDate;assert.sameValue(typeof m,'function');assert.sameValue(m.apply(d,[15]),1579075445006);assert.sameValue(d.getTime(),1579075445006);
// CASE: host-local-setter-setFullYear
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577952245006),m=Date.prototype.setFullYear;assert.sameValue(typeof m,'function');assert.sameValue(m.apply(d,[2001,1,3]),981187445006);assert.sameValue(d.getTime(),981187445006);
// CASE: host-local-setter-setHours
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577952245006),m=Date.prototype.setHours;assert.sameValue(typeof m,'function');assert.sameValue(m.apply(d,[20,21,22,23]),1578014482023);assert.sameValue(d.getTime(),1578014482023);
// CASE: host-local-setter-setMilliseconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577952245006),m=Date.prototype.setMilliseconds;assert.sameValue(typeof m,'function');assert.sameValue(m.apply(d,[999]),1577952245999);assert.sameValue(d.getTime(),1577952245999);
// CASE: host-local-setter-setMinutes
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577952245006),m=Date.prototype.setMinutes;assert.sameValue(typeof m,'function');assert.sameValue(m.apply(d,[30,31,32]),1577953831032);assert.sameValue(d.getTime(),1577953831032);
// CASE: host-local-setter-setMonth
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577952245006),m=Date.prototype.setMonth;assert.sameValue(typeof m,'function');assert.sameValue(m.apply(d,[5,15]),1592204645006);assert.sameValue(d.getTime(),1592204645006);
// CASE: host-local-setter-setSeconds
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(1577952245006),m=Date.prototype.setSeconds;assert.sameValue(typeof m,'function');assert.sameValue(m.apply(d,[40,41]),1577952280041);assert.sameValue(d.getTime(),1577952280041);
// CASE: host-local-inverse-spring-gap
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
assert.sameValue(new Date(2017,2,12,2,30).getTime(),1489303800000);assert.sameValue(Date.parse("2017-03-12T02:30:00"),1489303800000);var d=new Date(2017,2,12,0,0);assert.sameValue(d.setHours(2,30),1489303800000);
// CASE: host-local-inverse-autumn-fold
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
assert.sameValue(new Date(2017,10,5,1,30).getTime(),1509859800000);assert.sameValue(Date.parse("2017-11-05T01:30:00"),1509859800000);var d=new Date(2017,10,5,0,0);assert.sameValue(d.setHours(1,30),1509859800000);
// CASE: annex-B-alias-and-year-semantics
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
assert.sameValue(Date.prototype.toGMTString,Date.prototype.toUTCString);assert.sameValue(Date.prototype.toGMTString.name,'toUTCString');var d=new Date(2000,0,1);assert.sameValue(d.getYear(),100);assert.sameValue(d.setYear(99),new Date(1999,0,1).getTime());assert.sameValue(d.getFullYear(),1999);d.setYear(100);assert.sameValue(d.getFullYear(),100);assert.sameValue(new Date(NaN).getYear(),NaN);d=new Date(NaN);d.setYear(2000);assert.sameValue(d.getFullYear(),2000);assert.sameValue(d.getMonth(),0);assert.sameValue(d.getDate(),1);
// CASE: symbol-toPrimitive-metadata-and-hints
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype[Symbol.toPrimitive];assert.sameValue(typeof m,'function');verifyProperty(Date.prototype,Symbol.toPrimitive,{value:m,writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:'[Symbol.toPrimitive]',writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(m.call(new Date(0),'number'),0);assert.sameValue(m.call(new Date(0),'string'),new Date(0).toString());assert.throws(TypeError,function(){new m();});
// CASE: symbol-toPrimitive-generic-ordinary-conversion
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype[Symbol.toPrimitive],log='',o={toString:function(){log+='S';return 'text';},valueOf:function(){log+='V';return 7;}};assert.sameValue(m.call(o,'default'),'text');assert.sameValue(m.call(o,'string'),'text');assert.sameValue(m.call(o,'number'),7);assert.sameValue(log,'SSV');o[Symbol.toPrimitive]=function(){throw 'recurse';};assert.sameValue(m.call(o,'number'),7);
// CASE: symbol-toPrimitive-receiver-before-hint-and-no-hint-coercion
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype[Symbol.toPrimitive];assert.sameValue(m.call(new Date(1),'number'),1);var reads=0,h={toString:function(){reads++;return 'number';}};assert.throws(TypeError,function(){m.call({},h);});assert.throws(TypeError,function(){m.call(null,h);});assert.throws(TypeError,function(){m.call({},'bogus');});assert.sameValue(reads,0);
// CASE: default-date-primitive-uses-live-overrides
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(0),log='';d.toString=function(){log+='S';return 'custom';};d.valueOf=function(){log+='V';return 7;};assert.sameValue(d+'','custom');assert.sameValue(+d,7);assert.sameValue(log,'SV');
// CASE: toJSON-finite-and-invalid
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
assert.sameValue(new Date(0).toJSON('ignored'),'1970-01-01T00:00:00.000Z');assert.sameValue(new Date(NaN).toJSON(),null);assert.sameValue(JSON.stringify({x:new Date(0),bad:new Date(NaN)}),'{"x":"1970-01-01T00:00:00.000Z","bad":null}');
// CASE: toJSON-generic-number-hint-live-iso-and-no-args
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype.toJSON;assert.sameValue(m.call(new Date(0)),'1970-01-01T00:00:00.000Z');var log='',o={};o[Symbol.toPrimitive]=function(h){assert.sameValue(this,o);assert.sameValue(h,'number');log+='P';Object.defineProperty(o,'toISOString',{get:function(){log+='G';return function(){log+='I';assert.sameValue(this,o);assert.sameValue(arguments.length,0);return 42;};}});return 0;};assert.sameValue(m.call(o,'key'),42);assert.sameValue(log,'PGI');
// CASE: toJSON-only-nonfinite-number-short-circuits
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype.toJSON;assert.sameValue(m.call(new Date(0)),'1970-01-01T00:00:00.000Z');var values=[NaN,Infinity,-Infinity],reads=0;for(var i=0;i<values.length;i++){var o={valueOf:function(){return values[i];},get toISOString(){reads++;throw 'late';}};assert.sameValue(m.call(o),null);}assert.sameValue(reads,0);var p={valueOf:function(){return 'not number';},toISOString:function(){return 7;}};assert.sameValue(m.call(p),7);p.valueOf=function(){return Symbol('primitive');};assert.sameValue(m.call(p),7);
// CASE: toJSON-abrupt-and-callability
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype.toJSON;assert.sameValue(m.call(new Date(0)),'1970-01-01T00:00:00.000Z');var reason={},caught,o={valueOf:function(){throw reason;}};try{m.call(o);}catch(e){caught=e;}assert.sameValue(caught,reason);assert.throws(TypeError,function(){m.call(null);});assert.throws(TypeError,function(){m.call({valueOf:function(){return 0;},toISOString:1});});
// CASE: JSON-calls-toJSON-before-replacer-with-key
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(0),log='';d.toJSON=function(k){assert.sameValue(this,d);assert.sameValue(k,'x');log+='J';return 'date';};var text=JSON.stringify({x:d},function(k,v){if(k==='x'){log+='R';assert.sameValue(v,'date');}return v;});assert.sameValue(text,'{"x":"date"}');assert.sameValue(log,'JR');
// CASE: toJSON-boxing-and-hook-receiver
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var m=Date.prototype.toJSON;assert.sameValue(m.call(new Date(0)),'1970-01-01T00:00:00.000Z');Object.defineProperty(Number.prototype,'toISOString',{value:function(){'use strict';assert.sameValue(typeof this,'object');assert.sameValue(this.valueOf(),7);return 'boxed';},configurable:true});assert.sameValue(m.call(7),'boxed');
// CASE: getter-alias-retains-slot-behavior-after-property-replacement
assert.sameValue(typeof Date,'function','Date availability prerequisite');
assert.sameValue(typeof Date.UTC,'function');assert.sameValue(Date.UTC(1970,0,1),0);
var guardDate=new Date(0);assert.sameValue(guardDate.getTime(),0);
assert.sameValue(guardDate.toISOString(),'1970-01-01T00:00:00.000Z');
var d=new Date(42),get=d.getTime;Date.prototype.getTime=7;assert.sameValue(get.call(d),42);delete Date.prototype.getTime;assert.sameValue(get.call(d),42);
