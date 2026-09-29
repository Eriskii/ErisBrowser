// CASE: basic-var
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var out=[];for(var x of sequence([2,4]).source)out.push(x);assert.sameValue(text(out),"2,4");
// CASE: basic-let
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var out=[];for(let x of sequence([2,4]).source)out.push(x);assert.sameValue(text(out),"2,4");
// CASE: basic-const
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var out=[];for(const x of sequence([2,4]).source)out.push(x);assert.sameValue(text(out),"2,4");
// CASE: assignment-existing-binding
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var x=9,out=[];for(x of sequence([2,4]).source)out.push(x);assert.sameValue(x,4);assert.sameValue(text(out),"2,4");
// CASE: empty-preserves-assignment-target
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var x=9;for(x of sequence([]).source)throw "body";assert.sameValue(x,9);
// CASE: rhs-evaluated-once
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var calls=0;function rhs(){calls++;return sequence([1,2]).source;}var sum=0;for(var x of rhs())sum+=x;assert.sameValue(calls,1);assert.sameValue(sum,3);
// CASE: rhs-throw-identity
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var reason={},caught;function rhs(){throw reason;}try{for(var x of rhs())throw "body";}catch(e){caught=e;}assert.sameValue(caught,reason);
// CASE: iterable-method-zero-args-and-receiver
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var o={},count=0,it={next:function(){assert.sameValue(this,it);assert.sameValue(arguments.length,0);return {done:true};}};o[Symbol.iterator]=function(){assert.sameValue(this,o);assert.sameValue(arguments.length,0);count++;return it;};for(var x of o)throw "body";assert.sameValue(count,1);
// CASE: inherited-iterator-accessor-receiver
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var o={},p={},reads=0,it={next:function(){return {done:true};}};Object.defineProperty(p,Symbol.iterator,{get:function(){assert.sameValue(this,o);reads++;return function(){assert.sameValue(this,o);return it;};}});Object.setPrototypeOf(o,p);for(var x of o)throw "body";assert.sameValue(reads,1);
// CASE: primitive-iterator-method-receives-unboxed-value
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var seen=0;Number.prototype[Symbol.iterator]=function(){"use strict";assert.sameValue(this,17);seen++;return {next:function(){return {done:true};}};};for(var x of 17)throw "body";assert.sameValue(seen,1);
// CASE: only-symbol-iterator-is-consulted
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var reads=0,o={},s=sequence([5]);function poison(){reads++;throw "wrong property";}Object.defineProperty(o,"length",{get:poison});Object.defineProperty(o,"iterator",{get:poison});Object.defineProperty(o,"@@iterator",{get:poison});o[Symbol.iterator]=s.source[Symbol.iterator];var n=0;for(var x of o)n+=x;assert.sameValue(n,5);assert.sameValue(reads,0);
// CASE: async-iterator-is-not-a-sync-fallback
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
assert.sameValue(typeof Symbol.asyncIterator,"symbol");var o={},calls=0;o[Symbol.asyncIterator]=function(){calls++;return {next:function(){return {done:true};}};};assert.throws(TypeError,function(){for(var x of o){}});assert.sameValue(calls,0);
// CASE: noniterable-undefined
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
assert.throws(TypeError,function(){for(var x of undefined){}});
// CASE: noniterable-null
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
assert.throws(TypeError,function(){for(var x of null){}});
// CASE: noniterable-number
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
assert.throws(TypeError,function(){for(var x of 2){}});
// CASE: noniterable-boolean
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
assert.throws(TypeError,function(){for(var x of true){}});
// CASE: noniterable-symbol
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
assert.throws(TypeError,function(){for(var x of Symbol("x")){}});
// CASE: noniterable-ordinary
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
assert.throws(TypeError,function(){for(var x of ({length:0})){}});
// CASE: iterator-method-missing
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var o={};o[Symbol.iterator]=undefined;assert.throws(TypeError,function(){for(var x of o){}});
// CASE: iterator-method-null
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var o={};o[Symbol.iterator]=null;assert.throws(TypeError,function(){for(var x of o){}});
// CASE: iterator-method-number
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var o={};o[Symbol.iterator]=4;assert.throws(TypeError,function(){for(var x of o){}});
// CASE: iterator-method-object
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var o={};o[Symbol.iterator]=({});assert.throws(TypeError,function(){for(var x of o){}});
// CASE: iterator-getter-throw-before-next
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var reason={},caught,reads=0,o={};Object.defineProperty(o,Symbol.iterator,{get:function(){reads++;throw reason;}});try{for(var x of o){}}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(reads,1);
// CASE: iterator-call-throw
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var reason={},caught,o={};o[Symbol.iterator]=function(){throw reason;};try{for(var x of o){}}catch(e){caught=e;}assert.sameValue(caught,reason);
// CASE: iterator-result-must-be-object-undefined
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var o={};o[Symbol.iterator]=function(){return undefined;};assert.throws(TypeError,function(){for(var x of o){}});
// CASE: iterator-result-must-be-object-null
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var o={};o[Symbol.iterator]=function(){return null;};assert.throws(TypeError,function(){for(var x of o){}});
// CASE: iterator-result-must-be-object-number
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var o={};o[Symbol.iterator]=function(){return 3;};assert.throws(TypeError,function(){for(var x of o){}});
// CASE: iterator-result-must-be-object-string
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var o={};o[Symbol.iterator]=function(){return "it";};assert.throws(TypeError,function(){for(var x of o){}});
// CASE: iterator-result-must-be-object-boolean
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var o={};o[Symbol.iterator]=function(){return true;};assert.throws(TypeError,function(){for(var x of o){}});
// CASE: iterator-result-must-be-object-symbol
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var o={};o[Symbol.iterator]=function(){return Symbol("it");};assert.throws(TypeError,function(){for(var x of o){}});
// CASE: callable-iterator-object-is-accepted
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var o={},it=function(){throw "iterator called";};it.next=function(){return {done:true};};o[Symbol.iterator]=function(){return it;};for(var x of o)throw "body";
// CASE: next-getter-once-cached-call
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var reads=0,calls=0,log=[],it={},o={};Object.defineProperty(it,"next",{configurable:true,get:function(){reads++;return function(){assert.sameValue(this,it);assert.sameValue(arguments.length,0);var n=calls++;Object.defineProperty(it,"next",{value:function(){throw "late next";},configurable:true});return n<2?{value:n}:{done:true};};}});o[Symbol.iterator]=function(){return it;};for(var x of o)log.push(x);assert.sameValue(reads,1);assert.sameValue(calls,3);assert.sameValue(text(log),"0,1");
// CASE: next-getter-abrupt-does-not-close
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var reason={},caught,closes=0,it={},o={};Object.defineProperty(it,"next",{get:function(){throw reason;}});Object.defineProperty(it,"return",{get:function(){closes++;return function(){return {};};}});o[Symbol.iterator]=function(){return it;};try{for(var x of o){}}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(closes,0);
// CASE: noncallable-next-no-close-undefined
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var closes=0,it={next:undefined,return:function(){closes++;return {};}},o={};o[Symbol.iterator]=function(){return it;};assert.throws(TypeError,function(){for(var x of o){}});assert.sameValue(closes,0);
// CASE: noncallable-next-no-close-null
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var closes=0,it={next:null,return:function(){closes++;return {};}},o={};o[Symbol.iterator]=function(){return it;};assert.throws(TypeError,function(){for(var x of o){}});assert.sameValue(closes,0);
// CASE: noncallable-next-no-close-number
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var closes=0,it={next:3,return:function(){closes++;return {};}},o={};o[Symbol.iterator]=function(){return it;};assert.throws(TypeError,function(){for(var x of o){}});assert.sameValue(closes,0);
// CASE: noncallable-next-no-close-object
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var closes=0,it={next:({}),return:function(){closes++;return {};}},o={};o[Symbol.iterator]=function(){return it;};assert.throws(TypeError,function(){for(var x of o){}});assert.sameValue(closes,0);
// CASE: next-call-abrupt-does-not-close
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var reason={},caught,closes=0,it={next:function(){throw reason;},return:function(){closes++;return {};}},o={};o[Symbol.iterator]=function(){return it;};try{for(var x of o){}}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(closes,0);
// CASE: nonobject-step-does-not-close-undefined
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var closes=0,s=sequence([]);s.it.next=function(){return undefined;};s.it.return=function(){closes++;return {};};assert.throws(TypeError,function(){for(var x of s.source){}});assert.sameValue(closes,0);
// CASE: nonobject-step-does-not-close-null
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var closes=0,s=sequence([]);s.it.next=function(){return null;};s.it.return=function(){closes++;return {};};assert.throws(TypeError,function(){for(var x of s.source){}});assert.sameValue(closes,0);
// CASE: nonobject-step-does-not-close-number
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var closes=0,s=sequence([]);s.it.next=function(){return 3;};s.it.return=function(){closes++;return {};};assert.throws(TypeError,function(){for(var x of s.source){}});assert.sameValue(closes,0);
// CASE: nonobject-step-does-not-close-boolean
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var closes=0,s=sequence([]);s.it.next=function(){return true;};s.it.return=function(){closes++;return {};};assert.throws(TypeError,function(){for(var x of s.source){}});assert.sameValue(closes,0);
// CASE: nonobject-step-does-not-close-string
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var closes=0,s=sequence([]);s.it.next=function(){return "result";};s.it.return=function(){closes++;return {};};assert.throws(TypeError,function(){for(var x of s.source){}});assert.sameValue(closes,0);
// CASE: nonobject-step-does-not-close-symbol
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var closes=0,s=sequence([]);s.it.next=function(){return Symbol("r");};s.it.return=function(){closes++;return {};};assert.throws(TypeError,function(){for(var x of s.source){}});assert.sameValue(closes,0);
// CASE: done-getter-abrupt-does-not-close
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var reason={},caught,closes=0,r={done:false},s=sequence([]);Object.defineProperty(r,"done",{get:function(){throw reason;}});s.it.next=function(){return r;};s.it.return=function(){closes++;return {};};try{for(var x of s.source){}}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(closes,0);
// CASE: value-getter-abrupt-does-not-close
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var reason={},caught,closes=0,r={done:false},s=sequence([]);Object.defineProperty(r,"value",{get:function(){throw reason;}});s.it.next=function(){return r;};s.it.return=function(){closes++;return {};};try{for(var x of s.source){}}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(closes,0);
// CASE: done-true-skips-value-lhs-and-close
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var reads=0,r={done:true},s=sequence([]);Object.defineProperty(r,"value",{get:function(){throw "value";}});s.it.next=function(){return r;};Object.defineProperty(s.it,"return",{get:function(){throw "close";}});function base(){reads++;throw "lhs";}for(base().x of s.source)throw "body";assert.sameValue(reads,0);
// CASE: done-toboolean-does-not-coerce
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var poison={valueOf:function(){throw "valueOf";},toString:function(){throw "toString";}},s=sequence([]);poison[Symbol.toPrimitive]=function(){throw "primitive";};s.it.next=function(){return {done:poison,get value(){throw "value";}};};for(var x of s.source)throw "body";
// CASE: missing-done-and-value-mean-false-and-undefined
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var s=sequence([]),calls=0,body=0;s.it.next=function(){calls++;return calls===1?{}:{done:true};};for(var x of s.source){body++;assert.sameValue(x,undefined);}assert.sameValue(body,1);assert.sameValue(calls,2);
// CASE: result-inherited-accessors-use-result-receiver
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var p={},r=Object.create(p),log=[],s=sequence([]),calls=0;Object.defineProperty(p,"done",{get:function(){assert.sameValue(this,r);log.push("D");return calls>1;}});Object.defineProperty(p,"value",{get:function(){assert.sameValue(this,r);log.push("V");return 7;}});s.it.next=function(){calls++;return r;};for(var x of s.source){log.push(x);}assert.sameValue(text(log),"D,V,7,D");
// CASE: done-getter-can-replace-value-before-read
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var s=sequence([]),r={},calls=0;Object.defineProperty(r,"done",{get:function(){r.value=8;return false;}});s.it.next=function(){return calls++?{done:true}:r;};var out;for(var x of s.source)out=x;assert.sameValue(out,8);
// CASE: reused-result-object-read-live
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var r={done:false,value:0},s=sequence([]),i=0,out=[];s.it.next=function(){r.done=i===3;r.value=i++;return r;};for(var x of s.source)out.push(x);assert.sameValue(text(out),"0,1,2");
// CASE: normal-exhaustion-never-reads-return
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var s=sequence([1]);Object.defineProperty(s.it,"return",{get:function(){throw "return";}});for(var x of s.source)assert.sameValue(x,1);
// CASE: break-close-receiver-zero-args-once
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var trace=[],s=sequence([1,2],trace);Object.defineProperty(s.it,"return",{get:function(){assert.sameValue(this,s.it);trace.push("getReturn");return function(){assert.sameValue(this,s.it);assert.sameValue(arguments.length,0);trace.push("return");return {};};}});for(var x of s.source){trace.push("body");break;}assert.sameValue(text(trace),"next0,body,getReturn,return");
// CASE: close-result-object-fields-ignored
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var s=sequence([1]),calls=0;s.it.return=function(){calls++;return {get done(){throw "done";},get value(){throw "value";}};};for(var x of s.source)break;assert.sameValue(calls,1);
// CASE: close-break-absent
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var bodyToken={},closeError={},s=sequence([1]),reads=0,calls=0,result,caught,didThrow=false;function run(){for(var x of s.source){break;}return "finished";}try{result=run();}catch(e){didThrow=true;caught=e;}assert.sameValue(didThrow,false);assert.sameValue(result,"finished");assert.sameValue(reads,0);assert.sameValue(calls,0);
// CASE: close-return-absent
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var bodyToken={},closeError={},s=sequence([1]),reads=0,calls=0,result,caught,didThrow=false;function run(){for(var x of s.source){return bodyToken;}return "finished";}try{result=run();}catch(e){didThrow=true;caught=e;}assert.sameValue(didThrow,false);assert.sameValue(result,bodyToken);assert.sameValue(reads,0);assert.sameValue(calls,0);
// CASE: close-throw-absent
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var bodyToken={},closeError={},s=sequence([1]),reads=0,calls=0,result,caught,didThrow=false;function run(){for(var x of s.source){throw bodyToken;}return "finished";}try{result=run();}catch(e){didThrow=true;caught=e;}assert.sameValue(didThrow,true);assert.sameValue(caught,bodyToken);assert.sameValue(reads,0);assert.sameValue(calls,0);
// CASE: close-break-undefined
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var bodyToken={},closeError={},s=sequence([1]),reads=0,calls=0,result,caught,didThrow=false;s.it.return=undefined;function run(){for(var x of s.source){break;}return "finished";}try{result=run();}catch(e){didThrow=true;caught=e;}assert.sameValue(didThrow,false);assert.sameValue(result,"finished");assert.sameValue(reads,0);assert.sameValue(calls,0);
// CASE: close-return-undefined
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var bodyToken={},closeError={},s=sequence([1]),reads=0,calls=0,result,caught,didThrow=false;s.it.return=undefined;function run(){for(var x of s.source){return bodyToken;}return "finished";}try{result=run();}catch(e){didThrow=true;caught=e;}assert.sameValue(didThrow,false);assert.sameValue(result,bodyToken);assert.sameValue(reads,0);assert.sameValue(calls,0);
// CASE: close-throw-undefined
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var bodyToken={},closeError={},s=sequence([1]),reads=0,calls=0,result,caught,didThrow=false;s.it.return=undefined;function run(){for(var x of s.source){throw bodyToken;}return "finished";}try{result=run();}catch(e){didThrow=true;caught=e;}assert.sameValue(didThrow,true);assert.sameValue(caught,bodyToken);assert.sameValue(reads,0);assert.sameValue(calls,0);
// CASE: close-break-null
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var bodyToken={},closeError={},s=sequence([1]),reads=0,calls=0,result,caught,didThrow=false;s.it.return=null;function run(){for(var x of s.source){break;}return "finished";}try{result=run();}catch(e){didThrow=true;caught=e;}assert.sameValue(didThrow,false);assert.sameValue(result,"finished");assert.sameValue(reads,0);assert.sameValue(calls,0);
// CASE: close-return-null
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var bodyToken={},closeError={},s=sequence([1]),reads=0,calls=0,result,caught,didThrow=false;s.it.return=null;function run(){for(var x of s.source){return bodyToken;}return "finished";}try{result=run();}catch(e){didThrow=true;caught=e;}assert.sameValue(didThrow,false);assert.sameValue(result,bodyToken);assert.sameValue(reads,0);assert.sameValue(calls,0);
// CASE: close-throw-null
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var bodyToken={},closeError={},s=sequence([1]),reads=0,calls=0,result,caught,didThrow=false;s.it.return=null;function run(){for(var x of s.source){throw bodyToken;}return "finished";}try{result=run();}catch(e){didThrow=true;caught=e;}assert.sameValue(didThrow,true);assert.sameValue(caught,bodyToken);assert.sameValue(reads,0);assert.sameValue(calls,0);
// CASE: close-break-noncallable
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var bodyToken={},closeError={},s=sequence([1]),reads=0,calls=0,result,caught,didThrow=false;s.it.return=7;function run(){for(var x of s.source){break;}return "finished";}try{result=run();}catch(e){didThrow=true;caught=e;}assert.sameValue(didThrow,true);assert.sameValue(caught.constructor,TypeError);assert.sameValue(reads,0);assert.sameValue(calls,0);
// CASE: close-return-noncallable
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var bodyToken={},closeError={},s=sequence([1]),reads=0,calls=0,result,caught,didThrow=false;s.it.return=7;function run(){for(var x of s.source){return bodyToken;}return "finished";}try{result=run();}catch(e){didThrow=true;caught=e;}assert.sameValue(didThrow,true);assert.sameValue(caught.constructor,TypeError);assert.sameValue(reads,0);assert.sameValue(calls,0);
// CASE: close-throw-noncallable
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var bodyToken={},closeError={},s=sequence([1]),reads=0,calls=0,result,caught,didThrow=false;s.it.return=7;function run(){for(var x of s.source){throw bodyToken;}return "finished";}try{result=run();}catch(e){didThrow=true;caught=e;}assert.sameValue(didThrow,true);assert.sameValue(caught,bodyToken);assert.sameValue(reads,0);assert.sameValue(calls,0);
// CASE: close-break-getter-throw
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var bodyToken={},closeError={},s=sequence([1]),reads=0,calls=0,result,caught,didThrow=false;Object.defineProperty(s.it,"return",{get:function(){reads++;throw closeError;}});function run(){for(var x of s.source){break;}return "finished";}try{result=run();}catch(e){didThrow=true;caught=e;}assert.sameValue(didThrow,true);assert.sameValue(caught,closeError);assert.sameValue(reads,1);assert.sameValue(calls,0);
// CASE: close-return-getter-throw
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var bodyToken={},closeError={},s=sequence([1]),reads=0,calls=0,result,caught,didThrow=false;Object.defineProperty(s.it,"return",{get:function(){reads++;throw closeError;}});function run(){for(var x of s.source){return bodyToken;}return "finished";}try{result=run();}catch(e){didThrow=true;caught=e;}assert.sameValue(didThrow,true);assert.sameValue(caught,closeError);assert.sameValue(reads,1);assert.sameValue(calls,0);
// CASE: close-throw-getter-throw
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var bodyToken={},closeError={},s=sequence([1]),reads=0,calls=0,result,caught,didThrow=false;Object.defineProperty(s.it,"return",{get:function(){reads++;throw closeError;}});function run(){for(var x of s.source){throw bodyToken;}return "finished";}try{result=run();}catch(e){didThrow=true;caught=e;}assert.sameValue(didThrow,true);assert.sameValue(caught,bodyToken);assert.sameValue(reads,1);assert.sameValue(calls,0);
// CASE: close-break-call-throw
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var bodyToken={},closeError={},s=sequence([1]),reads=0,calls=0,result,caught,didThrow=false;s.it.return=function(){calls++;throw closeError;};function run(){for(var x of s.source){break;}return "finished";}try{result=run();}catch(e){didThrow=true;caught=e;}assert.sameValue(didThrow,true);assert.sameValue(caught,closeError);assert.sameValue(reads,0);assert.sameValue(calls,1);
// CASE: close-return-call-throw
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var bodyToken={},closeError={},s=sequence([1]),reads=0,calls=0,result,caught,didThrow=false;s.it.return=function(){calls++;throw closeError;};function run(){for(var x of s.source){return bodyToken;}return "finished";}try{result=run();}catch(e){didThrow=true;caught=e;}assert.sameValue(didThrow,true);assert.sameValue(caught,closeError);assert.sameValue(reads,0);assert.sameValue(calls,1);
// CASE: close-throw-call-throw
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var bodyToken={},closeError={},s=sequence([1]),reads=0,calls=0,result,caught,didThrow=false;s.it.return=function(){calls++;throw closeError;};function run(){for(var x of s.source){throw bodyToken;}return "finished";}try{result=run();}catch(e){didThrow=true;caught=e;}assert.sameValue(didThrow,true);assert.sameValue(caught,bodyToken);assert.sameValue(reads,0);assert.sameValue(calls,1);
// CASE: close-break-primitive
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var bodyToken={},closeError={},s=sequence([1]),reads=0,calls=0,result,caught,didThrow=false;s.it.return=function(){calls++;return 3;};function run(){for(var x of s.source){break;}return "finished";}try{result=run();}catch(e){didThrow=true;caught=e;}assert.sameValue(didThrow,true);assert.sameValue(caught.constructor,TypeError);assert.sameValue(reads,0);assert.sameValue(calls,1);
// CASE: close-return-primitive
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var bodyToken={},closeError={},s=sequence([1]),reads=0,calls=0,result,caught,didThrow=false;s.it.return=function(){calls++;return 3;};function run(){for(var x of s.source){return bodyToken;}return "finished";}try{result=run();}catch(e){didThrow=true;caught=e;}assert.sameValue(didThrow,true);assert.sameValue(caught.constructor,TypeError);assert.sameValue(reads,0);assert.sameValue(calls,1);
// CASE: close-throw-primitive
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var bodyToken={},closeError={},s=sequence([1]),reads=0,calls=0,result,caught,didThrow=false;s.it.return=function(){calls++;return 3;};function run(){for(var x of s.source){throw bodyToken;}return "finished";}try{result=run();}catch(e){didThrow=true;caught=e;}assert.sameValue(didThrow,true);assert.sameValue(caught,bodyToken);assert.sameValue(reads,0);assert.sameValue(calls,1);
// CASE: close-break-object
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var bodyToken={},closeError={},s=sequence([1]),reads=0,calls=0,result,caught,didThrow=false;s.it.return=function(){calls++;return {};};function run(){for(var x of s.source){break;}return "finished";}try{result=run();}catch(e){didThrow=true;caught=e;}assert.sameValue(didThrow,false);assert.sameValue(result,"finished");assert.sameValue(reads,0);assert.sameValue(calls,1);
// CASE: close-return-object
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var bodyToken={},closeError={},s=sequence([1]),reads=0,calls=0,result,caught,didThrow=false;s.it.return=function(){calls++;return {};};function run(){for(var x of s.source){return bodyToken;}return "finished";}try{result=run();}catch(e){didThrow=true;caught=e;}assert.sameValue(didThrow,false);assert.sameValue(result,bodyToken);assert.sameValue(reads,0);assert.sameValue(calls,1);
// CASE: close-throw-object
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var bodyToken={},closeError={},s=sequence([1]),reads=0,calls=0,result,caught,didThrow=false;s.it.return=function(){calls++;return {};};function run(){for(var x of s.source){throw bodyToken;}return "finished";}try{result=run();}catch(e){didThrow=true;caught=e;}assert.sameValue(didThrow,true);assert.sameValue(caught,bodyToken);assert.sameValue(reads,0);assert.sameValue(calls,1);
// CASE: live-return-lookup-after-body-mutation
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var s=sequence([1]),trace=[];s.it.return=function(){throw "old";};for(var x of s.source){s.it.return=function(){trace.push("new");return {};};break;}assert.sameValue(text(trace),"new");
// CASE: live-inherited-return-after-own-delete
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var s=sequence([1]),p={},calls=0;s.it.return=function(){throw "old";};p.return=function(){assert.sameValue(this,s.it);calls++;return {};};for(var x of s.source){delete s.it.return;Object.setPrototypeOf(s.it,p);break;}assert.sameValue(calls,1);
// CASE: return-value-is-saved-before-close
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var token={},replacement={},value=token,s=sequence([1]);s.it.return=function(){value=replacement;return {};};function run(){for(var x of s.source)return value;}assert.sameValue(run(),token);assert.sameValue(value,replacement);
// CASE: body-finally-runs-before-iterator-close
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var trace=[],s=sequence([1]);s.it.return=function(){trace.push("close");return {};};for(var x of s.source){try{trace.push("body");break;}finally{trace.push("finally");}}assert.sameValue(text(trace),"body,finally,close");
// CASE: outer-finally-runs-after-iterator-close
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var trace=[],s=sequence([1]);s.it.return=function(){trace.push("close");return {};};try{for(var x of s.source)break;}finally{trace.push("finally");}assert.sameValue(text(trace),"close,finally");
// CASE: finally-overrides-body-throw-with-break-before-close
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var bodyError={},closeError={},caught,s=sequence([1]);s.it.return=function(){throw closeError;};try{for(var x of s.source){try{throw bodyError;}finally{break;}}}catch(e){caught=e;}assert.sameValue(caught,closeError);
// CASE: same-loop-continue-does-not-close
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var closes=0,s=sequence([1,2]),seen=[];s.it.return=function(){closes++;return {};};for(var x of s.source){seen.push(x);continue;}assert.sameValue(text(seen),"1,2");assert.sameValue(closes,0);
// CASE: labeled-same-loop-continue-does-not-close
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var closes=0,s=sequence([1,2]),seen=[];s.it.return=function(){closes++;return {};};again:for(var x of s.source){seen.push(x);continue again;}assert.sameValue(text(seen),"1,2");assert.sameValue(closes,0);
// CASE: outward-continue-closes-inner-only
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var trace=[],outerSeq=sequence([1,2]);outerSeq.it.return=function(){trace.push("outer");return {};};outer:for(var x of outerSeq.source){var innerSeq=sequence([3,4]);innerSeq.it.return=function(){trace.push("inner"+x);return {};};for(var y of innerSeq.source)continue outer;}assert.sameValue(text(trace),"inner1,inner2");
// CASE: outward-break-closes-inner-before-outer
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var trace=[],a=sequence([1]),b=sequence([2]);a.it.return=function(){trace.push("outer");return {};};b.it.return=function(){trace.push("inner");return {};};outer:for(var x of a.source){for(var y of b.source)break outer;}assert.sameValue(text(trace),"inner,outer");
// CASE: nested-close-error-replaces-break-then-outer-throw-precedence
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var trace=[],first={},second={},caught,a=sequence([1]),b=sequence([2]);a.it.return=function(){trace.push("outer");throw second;};b.it.return=function(){trace.push("inner");throw first;};try{outer:for(var x of a.source){for(var y of b.source)break outer;}}catch(e){caught=e;}assert.sameValue(caught,first);assert.sameValue(text(trace),"inner,outer");
// CASE: break-enclosing-block-closes-loop
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var calls=0,s=sequence([1]);s.it.return=function(){calls++;return {};};block:{for(var x of s.source)break block;throw "after loop";}assert.sameValue(calls,1);
// CASE: switch-break-does-not-close-loop
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var calls=0,s=sequence([1,2]),sum=0;s.it.return=function(){calls++;return {};};for(var x of s.source){switch(x){case 1:sum+=1;break;default:sum+=2;break;}}assert.sameValue(sum,3);assert.sameValue(calls,0);
// CASE: fresh-closure-let
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var closures=[];for(let x of sequence([3,4,5]).source)closures.push(function(){return x;});assert.sameValue(closures[0](),3);assert.sameValue(closures[1](),4);assert.sameValue(closures[2](),5);
// CASE: head-tdz-let
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var x=sequence([1]).source;assert.throws(ReferenceError,function(){for(let x of x){}});assert.sameValue(typeof x,"object");
// CASE: head-typeof-tdz-let
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var x=1;assert.throws(ReferenceError,function(){for(let x of (typeof x,sequence([]).source)){}});assert.sameValue(x,1);
// CASE: head-captured-tdz-survives-loop-let
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var f,count=0;for(let x of (f=function(){return x;},sequence([1]).source)){count++;assert.sameValue(x,1);}assert.sameValue(count,1);assert.throws(ReferenceError,f);
// CASE: fresh-closure-const
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var closures=[];for(const x of sequence([3,4,5]).source)closures.push(function(){return x;});assert.sameValue(closures[0](),3);assert.sameValue(closures[1](),4);assert.sameValue(closures[2](),5);
// CASE: head-tdz-const
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var x=sequence([1]).source;assert.throws(ReferenceError,function(){for(const x of x){}});assert.sameValue(typeof x,"object");
// CASE: head-typeof-tdz-const
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var x=1;assert.throws(ReferenceError,function(){for(const x of (typeof x,sequence([]).source)){}});assert.sameValue(x,1);
// CASE: head-captured-tdz-survives-loop-const
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var f,count=0;for(const x of (f=function(){return x;},sequence([1]).source)){count++;assert.sameValue(x,1);}assert.sameValue(count,1);assert.throws(ReferenceError,f);
// CASE: var-closures-share-final-binding
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var closures=[];for(var x of sequence([3,4,5]).source)closures.push(function(){return x;});assert.sameValue(x,5);assert.sameValue(closures[0](),5);assert.sameValue(closures[1](),5);
// CASE: let-body-mutation-does-not-seed-next-binding
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var closures=[];for(let x of sequence([1,2]).source){x+=10;closures.push(function(){return x;});}assert.sameValue(closures[0](),11);assert.sameValue(closures[1](),12);
// CASE: const-write-closes-and-restores-outer-binding
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var x=9,s=sequence([1]),closes=0;s.it.return=function(){closes++;assert.sameValue(x,9);return {};};assert.throws(TypeError,function(){for(const x of s.source)x=2;});assert.sameValue(closes,1);assert.sameValue(x,9);
// CASE: body-shadow-does-not-alter-loop-binding
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var out=[];for(let x of sequence([1,2]).source){{let x=8;out.push(x);}out.push(x);}assert.sameValue(text(out),"8,1,8,2");
// CASE: var-is-hoisted-before-rhs
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
function run(){assert.sameValue(x,undefined);var rhs=function(){assert.sameValue(x,undefined);return sequence([8]).source;};for(var x of rhs()){}return x;}assert.sameValue(run(),8);
// CASE: outer-environment-observed-by-next-and-close
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var x=99,trace=[],s=sequence([1,2]);var next=s.it.next;s.it.next=function(){trace.push(x);return next.call(this);};s.it.return=function(){trace.push(x);return {};};for(let x of s.source){trace.push(x);if(x===2)break;}assert.sameValue(text(trace),"99,1,99,2,99");
// CASE: lexical-binding-restored-after-thrown-body
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var x=77,token={},caught;try{for(let x of sequence([2]).source)throw token;}catch(e){caught=e;assert.sameValue(x,77);}assert.sameValue(caught,token);assert.sameValue(x,77);
// CASE: lexical-binding-restored-before-close-throw
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var x=77,token={},caught,s=sequence([2]);s.it.return=function(){assert.sameValue(x,77);throw token;};try{for(let x of s.source)break;}catch(e){caught=e;}assert.sameValue(caught,token);assert.sameValue(x,77);
// CASE: const-of-binding-name
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var out=[];for(const of of sequence([1,2]).source)out.push(of);assert.sameValue(text(out),"1,2");
// CASE: let-of-binding-name
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var out=[];for(let of of sequence([1,2]).source)out.push(of);assert.sameValue(text(out),"1,2");
// CASE: for-of-function-callback-retains-iteration-binding
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var handlers=[];for(let x of sequence([1,2]).source){handlers.push(function(v=x){return v;});}assert.sameValue(handlers[0](),1);assert.sameValue(handlers[1](),2);
// CASE: member-lhs-after-value-and-before-body
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var trace=[],s=sequence([]),calls=0,target={};Object.defineProperty(target,"x",{set:function(v){trace.push("set"+v);}});function base(){trace.push("base");return target;}function key(){trace.push("key-expression");return {toString:function(){trace.push("key-string");return "x";}};}s.it.next=function(){trace.push("next");return calls++?{done:true}:{get done(){trace.push("done");return false;},get value(){trace.push("value");return 7;}};};for(base()[key()] of s.source)trace.push("body");assert.sameValue(text(trace),"next,done,value,base,key-expression,key-string,set7,body,next");
// CASE: member-base-reevaluated-per-iteration
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var targets=[{},{}],i=0;for(targets[i++].x of sequence([3,4]).source){}assert.sameValue(i,2);assert.sameValue(targets[0].x,3);assert.sameValue(targets[1].x,4);
// CASE: computed-symbol-target-key
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var target={},key=Symbol("k");for(target[key] of sequence([3]).source){}assert.sameValue(target[key],3);
// CASE: computed-key-string-hint
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var seen=[],target={},key={};key[Symbol.toPrimitive]=function(h){seen.push(h);return "x";};for(target[key] of sequence([4,5]).source){}assert.sameValue(target.x,5);assert.sameValue(text(seen),"string,string");
// CASE: lhs-base-throw-closes
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var reason={},caught,closes=0,s=sequence([1]);s.it.return=function(){closes++;return {};};function base(){throw reason;}try{for(base().x of s.source){}}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(closes,1);
// CASE: lhs-key-evaluation-throw-closes
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var reason={},caught,closes=0,s=sequence([1]),target={};s.it.return=function(){closes++;return {};};function key(){throw reason;}try{for(target[key()] of s.source){}}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(closes,1);
// CASE: lhs-key-coercion-throw-closes
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var reason={},caught,closes=0,s=sequence([1]),target={},key={toString:function(){throw reason;}};s.it.return=function(){closes++;return {};};try{for(target[key] of s.source){}}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(closes,1);
// CASE: null-base-evaluates-key-expression-but-not-key-coercion
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var trace=[],s=sequence([1]);s.it.return=function(){trace.push("close");return {};};function key(){trace.push("key");return {toString:function(){throw "coercion";}};}assert.throws(TypeError,function(){for(null[key()] of s.source){}});assert.sameValue(text(trace),"key,close");
// CASE: setter-throw-original-outweighs-close-throw
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var reason={},second={},caught,s=sequence([1]),target={};Object.defineProperty(target,"x",{set:function(){throw reason;}});s.it.return=function(){throw second;};try{for(target.x of s.source){}}catch(e){caught=e;}assert.sameValue(caught,reason);
// CASE: setter-sees-original-receiver-and-value
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var proto={},target=Object.create(proto),token={},calls=0;Object.defineProperty(proto,"x",{set:function(v){assert.sameValue(this,target);assert.sameValue(v,token);calls++;}});for(target.x of sequence([token]).source){}assert.sameValue(calls,1);
// CASE: strict-readonly-target-closes
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var target={},calls=0,s=sequence([1]);Object.defineProperty(target,"x",{value:9});s.it.return=function(){calls++;return {};};function run(){"use strict";for(target.x of s.source){}}assert.throws(TypeError,run);assert.sameValue(calls,1);assert.sameValue(target.x,9);
// CASE: sloppy-readonly-target-ignores-write-and-exhausts
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var target={},calls=0,s=sequence([1,2]);Object.defineProperty(target,"x",{value:9});s.it.return=function(){calls++;return {};};var run=Function("target","source","var n=0;for(target.x of source)n++;return n;");assert.sameValue(run(target,s.source),2);assert.sameValue(calls,0);assert.sameValue(target.x,9);
// CASE: value-captured-before-lhs-author-mutation
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var token={},replacement={},result={done:false,value:token},target={},s=sequence([]),n=0;s.it.next=function(){return n++?{done:true}:result;};function base(){result.value=replacement;return target;}for(base().x of s.source){}assert.sameValue(target.x,token);assert.sameValue(result.value,replacement);
// CASE: array-values-and-iterator-alias
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
assert.sameValue(typeof Array.prototype.values,"function");assert.sameValue(Array.prototype[Symbol.iterator],Array.prototype.values);var out=[];for(var x of [1,2])out.push(x);assert.sameValue(text(out),"1,2");
// CASE: array-iterator-override-wins-over-dense-storage
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var a=[1,2],s=sequence([8]);a[Symbol.iterator]=s.source[Symbol.iterator];var out=[];for(var x of a)out.push(x);assert.sameValue(text(out),"8");
// CASE: array-holes-are-visited-as-undefined
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var a=new Array(3),out=[];for(var x of a)out.push(x);assert.sameValue(out.length,3);assert.sameValue(out[0],undefined);assert.sameValue(out[1],undefined);assert.sameValue(out[2],undefined);
// CASE: array-inherited-hole-getter-live
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var a=new Array(2),p=Object.create(Array.prototype),reads=0,out=[];Object.defineProperty(p,"0",{get:function(){assert.sameValue(this,a);reads++;return 7;}});Object.setPrototypeOf(a,p);a[1]=8;for(var x of a)out.push(x);assert.sameValue(text(out),"7,8");assert.sameValue(reads,1);
// CASE: array-growth-after-first-value
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var a=[1],out=[];for(var x of a){out.push(x);if(x===1)a.push(2);}assert.sameValue(text(out),"1,2");
// CASE: array-shrink-stops-next-step
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var a=[1,2,3],out=[];for(var x of a){out.push(x);a.length=0;}assert.sameValue(text(out),"1");
// CASE: array-deleted-later-index-becomes-hole
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var a=[1,2],out=[];for(var x of a){out.push(x);delete a[1];}assert.sameValue(out.length,2);assert.sameValue(out[0],1);assert.sameValue(out[1],undefined);
// CASE: array-later-descriptor-mutation-is-live
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var a=[1,2],out=[];for(var x of a){out.push(x);if(out.length===1)Object.defineProperty(a,"1",{get:function(){return 9;}});}assert.sameValue(text(out),"1,9");
// CASE: array-iterator-length-not-cached
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var calls=0,o={0:1,1:2,get length(){calls++;return calls===1?2:1;}};o[Symbol.iterator]=Array.prototype.values;var out=[];for(var x of o)out.push(x);assert.sameValue(text(out),"1");assert.sameValue(calls,2);
// CASE: array-iterator-next-error-does-not-close
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var reason={},caught,closes=0,o={get length(){throw reason;}};var it=Array.prototype.values.call(o);it.return=function(){closes++;return {};};var iterable={};iterable[Symbol.iterator]=function(){return it;};try{for(var x of iterable){}}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(closes,0);
// CASE: array-max-u32-first-value-with-break
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var a=new Array(4294967295),n=0;for(var x of a){assert.sameValue(x,undefined);n++;break;}assert.sameValue(n,1);
// CASE: array-values-huge-generic-range-first-value
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var o={length:Infinity,0:9};o[Symbol.iterator]=Array.prototype.values;var n=0;for(var x of o){assert.sameValue(x,9);n++;break;}assert.sameValue(n,1);
// CASE: arguments-iterator-alias-and-mapped-live-value
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var f=Function("var out=[],a=arguments;for(var x of a){out.push(x);if(out.length===1)a[1]=9;}return out;");var out=f(1,2);assert.sameValue(text(out),"1,9");
// CASE: array-iterator-stays-completed-after-growth
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var a=[],it=a[Symbol.iterator]();assert.sameValue(it.next().done,true);a.push(1);assert.sameValue(it.next().done,true);
// CASE: array-iterator-self-iterable
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var it=[1,2][Symbol.iterator]();assert.sameValue(it[Symbol.iterator](),it);var out=[];for(var x of it)out.push(x);assert.sameValue(text(out),"1,2");
// CASE: string-empty
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var out=[];for(var x of "")out.push(x);var want=[];assert.sameValue(out.length,want.length);for(var i=0;i<want.length;i++)assert.sameValue(out[i],want[i]);
// CASE: string-ascii
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var out=[];for(var x of "ab")out.push(x);var want=["a","b"];assert.sameValue(out.length,want.length);for(var i=0;i<want.length;i++)assert.sameValue(out[i],want[i]);
// CASE: string-supplementary
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var out=[];for(var x of "A\uD83D\uDE00B")out.push(x);var want=["A","\ud83d\ude00","B"];assert.sameValue(out.length,want.length);for(var i=0;i<want.length;i++)assert.sameValue(out[i],want[i]);
// CASE: string-isolated-high
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var out=[];for(var x of "\uD800x")out.push(x);var want=["\ud800","x"];assert.sameValue(out.length,want.length);for(var i=0;i<want.length;i++)assert.sameValue(out[i],want[i]);
// CASE: string-isolated-low
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var out=[];for(var x of "x\uDC00")out.push(x);var want=["x","\udc00"];assert.sameValue(out.length,want.length);for(var i=0;i<want.length;i++)assert.sameValue(out[i],want[i]);
// CASE: string-high-high-low
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var out=[];for(var x of "\uD800\uD801\uDC00")out.push(x);var want=["\ud800","\ud801\udc00"];assert.sameValue(out.length,want.length);for(var i=0;i<want.length;i++)assert.sameValue(out[i],want[i]);
// CASE: string-no-normalization
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var out=[];for(var x of "e\u0301\u00e9")out.push(x);var want=["e","\u0301","\u00e9"];assert.sameValue(out.length,want.length);for(var i=0;i<want.length;i++)assert.sameValue(out[i],want[i]);
// CASE: string-nul
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var out=[];for(var x of "a\u0000b")out.push(x);var want=["a","\u0000","b"];assert.sameValue(out.length,want.length);for(var i=0;i<want.length;i++)assert.sameValue(out[i],want[i]);
// CASE: boxed-string-iteration
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var out=[];for(var x of Object("ab"))out.push(x);assert.sameValue(text(out),"a,b");
// CASE: string-iterator-converts-once-at-acquisition
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var calls=0,obj={toString:function(){calls++;return "ab";}};var it=String.prototype[Symbol.iterator].call(obj);assert.sameValue(calls,1);obj.toString=function(){throw "late conversion";};var out=[];for(var x of it)out.push(x);assert.sameValue(text(out),"a,b");assert.sameValue(calls,1);
// CASE: string-iterator-nullish-brand-and-symbol-conversion
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
assert.sameValue(typeof String.prototype[Symbol.iterator],"function");assert.sameValue(String.prototype[Symbol.iterator].call("a").next().value,"a");assert.throws(TypeError,function(){String.prototype[Symbol.iterator].call(null);});assert.throws(TypeError,function(){String.prototype[Symbol.iterator].call(undefined);});assert.throws(TypeError,function(){String.prototype[Symbol.iterator].call(Symbol("x"));});
// CASE: string-iterator-override-on-prototype
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var old=String.prototype[Symbol.iterator],s=sequence([9]);String.prototype[Symbol.iterator]=s.source[Symbol.iterator];var out=[];for(var x of "ab")out.push(x);assert.sameValue(text(out),"9");String.prototype[Symbol.iterator]=old;
// CASE: string-iterator-next-brand
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var it="a"[Symbol.iterator](),next=it.next;assert.throws(TypeError,function(){next.call({});});assert.sameValue(next.call(it).value,"a");assert.sameValue(next.call(it).done,true);
// CASE: var-loop-resolves-current-catch-binding
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
function run(){var x=1,seen;try{throw 7;}catch(x){for(var x of sequence([9]).source)seen=x;assert.sameValue(x,9);}assert.sameValue(x,1);return seen;}assert.sameValue(run(),9);
// CASE: regex-and-template-rhs-rescan
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var out=[];for(var x of (/[}]/.test("}")?sequence([`v${1}`]).source:sequence([]).source))out.push(x);assert.sameValue(text(out),"v1");
// CASE: labeled-loop-double-label-current-continue
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var out=[],s=sequence([1,2]),closes=0;s.it.return=function(){closes++;return {};};first:second:for(var x of s.source){out.push(x);continue first;}assert.sameValue(text(out),"1,2");assert.sameValue(closes,0);
// CASE: array-element-getter-reenters-next-after-index-advance
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var a=[0,2,3],it=a.values(),nested;Object.defineProperty(a,"0",{get:function(){nested=it.next();return 1;}});var first=it.next();assert.sameValue(first.value,1);assert.sameValue(nested.value,2);assert.sameValue(it.next().value,3);assert.sameValue(it.next().done,true);
// CASE: array-element-getter-throw-still-advances-index
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var reason={},caught,a=[1,2],it=a.values();Object.defineProperty(a,"0",{get:function(){throw reason;}});try{it.next();}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(it.next().value,2);assert.sameValue(it.next().done,true);
// CASE: array-length-getter-throw-does-not-advance-index
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var reason={},caught,calls=0,o={0:8,get length(){if(calls++===0)throw reason;return 1;}},it=Array.prototype.values.call(o);try{it.next();}catch(e){caught=e;}assert.sameValue(caught,reason);assert.sameValue(it.next().value,8);
// CASE: array-values-creation-does-not-read-length
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var calls=0,o={get length(){calls++;return 0;}},it=Array.prototype.values.call(o);assert.sameValue(calls,0);assert.sameValue(it.next().done,true);assert.sameValue(calls,1);assert.sameValue(it.next().done,true);assert.sameValue(calls,1);
// CASE: array-result-descriptors-and-fresh-identity
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var it=[7].values(),a=it.next(),b=it.next(),c=it.next();verifyProperty(a,"value",{value:7,writable:true,enumerable:true,configurable:true},{restore:true});verifyProperty(a,"done",{value:false,writable:true,enumerable:true,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(a),Object.prototype);assert.sameValue(b.value,undefined);assert.sameValue(b.done,true);assert.notSameValue(a,b);assert.notSameValue(b,c);
// CASE: array-next-rejects-fake-prototype-object
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var it=[1].values(),next=it.next;assert.sameValue(next.call(it).value,1);assert.throws(TypeError,function(){next.call(Object.create(Object.getPrototypeOf(it)));});assert.throws(TypeError,function(){next.call(null);});
// CASE: freezing-native-iterator-does-not-freeze-progress
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var it=[1,2].values();Object.freeze(it);assert.sameValue(it.next().value,1);assert.sameValue(it.next().value,2);assert.sameValue(it.next().done,true);var str="ab"[Symbol.iterator]();Object.freeze(str);assert.sameValue(str.next().value,"a");assert.sameValue(str.next().value,"b");
// CASE: array-values-alias-remains-after-property-replacement
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var values=Array.prototype.values,iterator=Array.prototype[Symbol.iterator];Array.prototype.values=1;var out=[];for(var x of [4])out.push(x);assert.sameValue(text(out),"4");assert.sameValue(iterator,values);assert.sameValue(values.call([9]).next().value,9);
// CASE: arguments-own-iterator-descriptor
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
function f(){verifyProperty(arguments,Symbol.iterator,{value:Array.prototype.values,writable:true,enumerable:false,configurable:true},{restore:true});var out=[];for(var x of arguments)out.push(x);return out;}assert.sameValue(text(f(2,3)),"2,3");
// CASE: strict-arguments-iteration
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
function f(a){"use strict";var out=[];for(var x of arguments){out.push(x);a=9;}return out;}assert.sameValue(text(f(2,3)),"2,3");
// CASE: mapped-parameter-mutation-is-observed-by-iterator
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var f=Function("a","b","var out=[];for(var x of arguments){out.push(x);b=9;}return out;");assert.sameValue(text(f(1,2)),"1,9");
// CASE: array-iterator-native-metadata
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var m=Array.prototype.values,it=[1].values(),proto=Object.getPrototypeOf(it),next=it.next;assert.sameValue(typeof m,"function");assert.sameValue(next.call(it).done,false);verifyProperty(Array.prototype,"values",{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,"name",{value:"values",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,"length",{value:0,writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(proto,"next",{value:next,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(next,"name",{value:"next",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(next,"length",{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getOwnPropertyDescriptor(m,"prototype"),undefined);assert.throws(TypeError,function(){new m();});assert.throws(TypeError,function(){new next();});verifyProperty(proto,Symbol.toStringTag,{value:"Array Iterator",writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.prototype.toString.call(it),"[object Array Iterator]");
// CASE: string-iterator-native-metadata
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var m=String.prototype[Symbol.iterator],it="x"[Symbol.iterator](),proto=Object.getPrototypeOf(it),next=it.next;assert.sameValue(typeof m,"function");assert.sameValue(next.call(it).done,false);verifyProperty(String.prototype,Symbol.iterator,{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,"name",{value:"[Symbol.iterator]",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,"length",{value:0,writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(proto,"next",{value:next,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(next,"name",{value:"next",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(next,"length",{value:0,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getOwnPropertyDescriptor(m,"prototype"),undefined);assert.throws(TypeError,function(){new m();});assert.throws(TypeError,function(){new next();});verifyProperty(proto,Symbol.toStringTag,{value:"String Iterator",writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.prototype.toString.call(it),"[object String Iterator]");
// CASE: common-iterator-prototype-self-method-is-generic
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var a=[1].values(),s="a"[Symbol.iterator](),ap=Object.getPrototypeOf(Object.getPrototypeOf(a)),sp=Object.getPrototypeOf(Object.getPrototypeOf(s));assert.sameValue(ap,sp);var m=ap[Symbol.iterator],token={};assert.sameValue(m.call(token),token);assert.sameValue(m.call(null),null);assert.sameValue(m.call(3),3);assert.sameValue(a[Symbol.iterator](),a);assert.sameValue(s[Symbol.iterator](),s);
// CASE: array-destructuring-binding
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var out=[];for(const [a,b] of sequence([[1,2],[3,4]]).source)out.push(a+b);assert.sameValue(text(out),"3,7");
// CASE: object-destructuring-binding
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var out=[];for(let {x:y} of sequence([{x:1},{x:2}]).source)out.push(y);assert.sameValue(text(out),"1,2");
// CASE: destructuring-assignment
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var a,b;for([a,b] of sequence([[1,2]]).source){}assert.sameValue(a,1);assert.sameValue(b,2);
// CASE: destructuring-default-tdz-closes
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var closes=0,s=sequence([[undefined]]);s.it.return=function(){closes++;return {};};assert.throws(ReferenceError,function(){for(let [x=x] of s.source){}});assert.sameValue(closes,1);
// CASE: generator-return-finally-on-break
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var log=[];function* gen(){try{yield 1;yield 2;}finally{log.push("finally");}}for(var x of gen()){assert.sameValue(x,1);break;}assert.sameValue(text(log),"finally");
// CASE: async-for-of-parser-prerequisite
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
async function f(source){for await(const x of source){return x;}}assert.sameValue(typeof f,"function");
// CASE: syntax-raw-async-target
var async;for(async of []){}
// CASE: syntax-lexical-let-binding
for(let let of []){}
// CASE: syntax-ordinary-const-needs-value
const x;
// CASE: syntax-classic-for-const-needs-value
for(const x;false;){}
// CASE: syntax-const-initializer
for(const x=1 of []){}
// CASE: syntax-let-initializer
for(let x=1 of []){}
// CASE: syntax-var-initializer
for(var x=1 of []){}
// CASE: syntax-multiple-var-bindings
for(var x,y of []){}
// CASE: syntax-multiple-let-bindings
for(let x,y of []){}
// CASE: syntax-multiple-const-bindings
for(const x,y of []){}
// CASE: syntax-escaped-of-separator
for(var x o\u0066 []){}
// CASE: syntax-comma-rhs
for(var x of [],[]){}
// CASE: syntax-invalid-assignment-binary
for(a+b of []){}
// CASE: syntax-duplicate-lexical-in-body
for(let x of []){var x;}
// CASE: syntax-continue-undefined-label
for(var x of []){continue missing;}
// CASE: syntax-break-undefined-label
for(var x of []){break missing;}
// CASE: syntax-duplicate-label
again:again:for(var x of []){}
// CASE: syntax-continue-noniteration-label
block:{for(var x of [])continue block;}
// CASE: parenthesized-async-target-is-valid
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var async=0;for((async) of sequence([7]).source){}assert.sameValue(async,7);
// CASE: declared-async-binding-is-valid
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var out=[];for(var async of sequence([7]).source)out.push(async);assert.sameValue(text(out),"7");
// CASE: parenthesized-comma-rhs-is-valid
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var out=[];for(var x of (0,sequence([4]).source))out.push(x);assert.sameValue(text(out),"4");
// CASE: budget-infinite-iterator-is-terminal
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var s=sequence([]);s.it.next=function(){return {value:1,done:false};};try{for(var x of s.source){}}catch(e){throw "resource was catchable";}throw "infinite loop completed";
// CASE: budget-next-callback-infinite-is-terminal
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var s=sequence([]);s.it.next=function(){while(true){}};try{for(var x of s.source){}}catch(e){throw "resource was catchable";}
// CASE: budget-close-callback-infinite-is-terminal
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var s=sequence([1]);s.it.return=function(){while(true){}};try{for(var x of s.source)break;}catch(e){throw "resource was catchable";}
// CASE: budget-recursive-next-is-terminal
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var s=sequence([]);s.it.next=function(){for(var x of s.source){}};try{for(var x of s.source){}}catch(e){throw "resource was catchable";}
// CASE: budget-retained-lexical-closures-is-terminal
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
var out=[],s=sequence([]),n=0;s.it.next=function(){return {value:n++};};try{for(let x of s.source)out.push(function(){return x;});}catch(e){throw "resource was catchable";}
// CASE: budget-large-array-hole-range-is-terminal
assert.sameValue(typeof Symbol,'function','Symbol prerequisite');
assert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');
var $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;
$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};
for(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}
assert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);
function sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}
function text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}
try{for(var x of new Array(4294967295)){}}catch(e){throw "resource was catchable";}
