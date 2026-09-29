// Independent Array.from oracle; expectations frozen before any adapter execution.

// CASE: array-like-basic
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
sameList(Array.from({0:2,1:4,length:2}),[2,4]);
// CASE: custom-iterable-basic
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
sameList(Array.from(sequence([2,4]).source),[2,4]);
// CASE: iterable-wins-over-length
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var s=sequence([7]),reads=0;Object.defineProperty(s.source,"length",{get:function(){reads++;throw "length";}});sameList(Array.from(s.source),[7]);assert.sameValue(reads,0);
// CASE: iterator-absent-array-like
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={0:8,length:1};o[Symbol.iterator]=undefined;sameList(Array.from(o),[8]);
// CASE: iterator-null-array-like
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={0:8,length:1};o[Symbol.iterator]=null;sameList(Array.from(o),[8]);
// CASE: noncallable-iterator-number
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={length:1},reads=0;o[Symbol.iterator]=1;Object.defineProperty(o,"length",{get:function(){reads++;return 1;}});assert.throws(TypeError,function(){Array.from(o);});assert.sameValue(reads,0);
// CASE: noncallable-iterator-boolean
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={length:1},reads=0;o[Symbol.iterator]=false;Object.defineProperty(o,"length",{get:function(){reads++;return 1;}});assert.throws(TypeError,function(){Array.from(o);});assert.sameValue(reads,0);
// CASE: noncallable-iterator-string
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={length:1},reads=0;o[Symbol.iterator]="x";Object.defineProperty(o,"length",{get:function(){reads++;return 1;}});assert.throws(TypeError,function(){Array.from(o);});assert.sameValue(reads,0);
// CASE: noncallable-iterator-object
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={length:1},reads=0;o[Symbol.iterator]=({});Object.defineProperty(o,"length",{get:function(){reads++;return 1;}});assert.throws(TypeError,function(){Array.from(o);});assert.sameValue(reads,0);
// CASE: noncallable-iterator-symbol
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={length:1},reads=0;o[Symbol.iterator]=Symbol("x");Object.defineProperty(o,"length",{get:function(){reads++;return 1;}});assert.throws(TypeError,function(){Array.from(o);});assert.sameValue(reads,0);
// CASE: invalid-mapper-before-items-null
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={},reads=0;Object.defineProperty(o,Symbol.iterator,{get:function(){reads++;throw "iterator";}});Object.defineProperty(o,"length",{get:function(){reads++;throw "length";}});assert.throws(TypeError,function(){Array.from(o,null);});assert.sameValue(reads,0);
// CASE: invalid-mapper-before-items-number
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={},reads=0;Object.defineProperty(o,Symbol.iterator,{get:function(){reads++;throw "iterator";}});Object.defineProperty(o,"length",{get:function(){reads++;throw "length";}});assert.throws(TypeError,function(){Array.from(o,1);});assert.sameValue(reads,0);
// CASE: invalid-mapper-before-items-boolean
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={},reads=0;Object.defineProperty(o,Symbol.iterator,{get:function(){reads++;throw "iterator";}});Object.defineProperty(o,"length",{get:function(){reads++;throw "length";}});assert.throws(TypeError,function(){Array.from(o,true);});assert.sameValue(reads,0);
// CASE: invalid-mapper-before-items-string
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={},reads=0;Object.defineProperty(o,Symbol.iterator,{get:function(){reads++;throw "iterator";}});Object.defineProperty(o,"length",{get:function(){reads++;throw "length";}});assert.throws(TypeError,function(){Array.from(o,"map");});assert.sameValue(reads,0);
// CASE: invalid-mapper-before-items-object
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={},reads=0;Object.defineProperty(o,Symbol.iterator,{get:function(){reads++;throw "iterator";}});Object.defineProperty(o,"length",{get:function(){reads++;throw "length";}});assert.throws(TypeError,function(){Array.from(o,({}));});assert.sameValue(reads,0);
// CASE: invalid-mapper-before-items-symbol
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={},reads=0;Object.defineProperty(o,Symbol.iterator,{get:function(){reads++;throw "iterator";}});Object.defineProperty(o,"length",{get:function(){reads++;throw "length";}});assert.throws(TypeError,function(){Array.from(o,Symbol("map"));});assert.sameValue(reads,0);
// CASE: nullish-items-null
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
assert.throws(TypeError,function(){Array.from(null);});
// CASE: nullish-items-undefined
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
assert.throws(TypeError,function(){Array.from(undefined);});
// CASE: undefined-mapper-does-not-call-thisarg
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={toString:function(){throw "coerce";},valueOf:function(){throw "coerce";}};sameList(Array.from(sequence([9]).source,undefined,o),[9]);
// CASE: ignored-extra-arguments
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={toString:function(){throw "coerce";},valueOf:function(){throw "coerce";}};sameList(Array.from({0:3,length:1},undefined,undefined,o,o),[3]);
// CASE: only-symbol-iterator
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={0:3,length:1};Object.defineProperty(o,"iterator",{get:function(){throw "wrong";}});Object.defineProperty(o,"@@iterator",{get:function(){throw "wrong";}});sameList(Array.from(o),[3]);
// CASE: async-iterator-is-not-sync-fallback
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
assert.sameValue(typeof Symbol.asyncIterator,"symbol");var n=0,o={0:4,length:1};o[Symbol.asyncIterator]=function(){n++;throw "async";};sameList(Array.from(o),[4]);assert.sameValue(n,0);
// CASE: iterator-getter-original-receiver
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var reads=0,p={},o=Object.create(p),s=sequence([6]);Object.defineProperty(p,Symbol.iterator,{get:function(){assert.sameValue(this,o);reads++;return s.source[Symbol.iterator];}});sameList(Array.from(o),[6]);assert.sameValue(reads,1);
// CASE: primitive-iterator-getter-and-call-unboxed
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var seen=0;Object.defineProperty(Number.prototype,Symbol.iterator,{get:function(){"use strict";assert.sameValue(this,7);seen++;return function(){"use strict";assert.sameValue(this,7);seen++;return {next:function(){return {done:true};}};};}});sameList(Array.from(7),[]);assert.sameValue(seen,2);
// CASE: iterable-order-construction-before-method-call
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var t=[],o={},it={next:function(){t.push("next");return {done:true};}};Object.defineProperty(o,Symbol.iterator,{get:function(){t.push("get-iterator");return function(){assert.sameValue(this,o);assert.sameValue(arguments.length,0);t.push("call-iterator");return it;};}});function C(){t.push("construct"+arguments.length);assert.sameValue(new.target,C);}var a=Array.from.call(C,o);assert.sameValue(traceText(t),"get-iterator,construct0,call-iterator,next");assert.sameValue(a.length,0);
// CASE: array-like-order-length-before-construction
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var t=[],o={get length(){t.push("length");return {valueOf:function(){t.push("convert");return 1;}};},get 0(){t.push("index");return 8;}};Object.defineProperty(o,Symbol.iterator,{get:function(){t.push("iterator");return null;}});function C(n){t.push("construct"+n);assert.sameValue(arguments.length,1);}var a=Array.from.call(C,o,function(v,i){t.push("map"+i);return v;});sameList(a,[8]);assert.sameValue(traceText(t),"iterator,length,convert,construct1,index,map0");
// CASE: cached-iterator-method-survives-constructor-mutation
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var s=sequence([6]),calls=0;function C(){s.source[Symbol.iterator]=function(){throw "late iterator";};calls++;}sameList(Array.from.call(C,s.source),[6]);assert.sameValue(calls,1);
// CASE: constructor-mutation-before-iterator-start
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=[2];function C(){a.push(4);}sameList(Array.from.call(C,a),[2,4]);
// CASE: iterator-getter-throw-skips-construction
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},got,o={},n=0;Object.defineProperty(o,Symbol.iterator,{get:function(){throw token;}});function C(){n++;}try{Array.from.call(C,o);}catch(e){got=e;}assert.sameValue(got,token);assert.sameValue(n,0);
// CASE: iterable-constructor-throw-skips-acquisition
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},got,n=0,s=sequence([1]);s.source[Symbol.iterator]=function(){n++;return s.it;};function C(){throw token;}try{Array.from.call(C,s.source);}catch(e){got=e;}assert.sameValue(got,token);assert.sameValue(n,0);
// CASE: array-like-constructor-throw-skips-index
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},got,o={length:1,get 0(){throw "index";}};function C(){throw token;}try{Array.from.call(C,o);}catch(e){got=e;}assert.sameValue(got,token);
// CASE: iterator-call-throw-identity
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},got,o={};o[Symbol.iterator]=function(){throw token;};try{Array.from(o);}catch(e){got=e;}assert.sameValue(got,token);
// CASE: nonobject-iterator-null
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={};o[Symbol.iterator]=function(){return null;};assert.throws(TypeError,function(){Array.from(o);});
// CASE: nonobject-iterator-undefined
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={};o[Symbol.iterator]=function(){return undefined;};assert.throws(TypeError,function(){Array.from(o);});
// CASE: nonobject-iterator-number
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={};o[Symbol.iterator]=function(){return 3;};assert.throws(TypeError,function(){Array.from(o);});
// CASE: nonobject-iterator-boolean
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={};o[Symbol.iterator]=function(){return false;};assert.throws(TypeError,function(){Array.from(o);});
// CASE: nonobject-iterator-string
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={};o[Symbol.iterator]=function(){return "x";};assert.throws(TypeError,function(){Array.from(o);});
// CASE: nonobject-iterator-symbol
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={};o[Symbol.iterator]=function(){return Symbol("x");};assert.throws(TypeError,function(){Array.from(o);});
// CASE: function-is-object-iterator-and-step
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var it=function(){},r=function(){};r.value=7;r.done=false;var n=0;it.next=function(){return n++?{done:true}:r;};var o={};o[Symbol.iterator]=function(){return it;};sameList(Array.from(o),[7]);
// CASE: cached-next-getter-and-call-receiver
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var n=0,reads=0,s=sequence([]);Object.defineProperty(s.it,"next",{configurable:true,get:function(){reads++;return function(){assert.sameValue(this,s.it);assert.sameValue(arguments.length,0);Object.defineProperty(s.it,"next",{value:function(){throw "late next";},configurable:true});return n++<2?{value:n}:{done:true};};}});sameList(Array.from(s.source),[1,2]);assert.sameValue(reads,1);assert.sameValue(n,3);
// CASE: no-close-next-get
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},got,n=0,s=sequence([]);Object.defineProperty(s.it,"return",{get:function(){n++;return function(){return {};};}});Object.defineProperty(s.it,"next",{get:function(){throw token;}});try{Array.from(s.source);}catch(e){got=e;}assert.sameValue(got,token);assert.sameValue(n,0);
// CASE: no-close-next-call
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},got,n=0,s=sequence([]);Object.defineProperty(s.it,"return",{get:function(){n++;return function(){return {};};}});s.it.next=function(){throw token;};try{Array.from(s.source);}catch(e){got=e;}assert.sameValue(got,token);assert.sameValue(n,0);
// CASE: no-close-done-get
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},got,n=0,s=sequence([]);Object.defineProperty(s.it,"return",{get:function(){n++;return function(){return {};};}});s.it.next=function(){return {get done(){throw token;}};};try{Array.from(s.source);}catch(e){got=e;}assert.sameValue(got,token);assert.sameValue(n,0);
// CASE: no-close-value-get
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},got,n=0,s=sequence([]);Object.defineProperty(s.it,"return",{get:function(){n++;return function(){return {};};}});s.it.next=function(){return {done:false,get value(){throw token;}};};try{Array.from(s.source);}catch(e){got=e;}assert.sameValue(got,token);assert.sameValue(n,0);
// CASE: no-close-noncallable-next-undefined
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var n=0,s=sequence([]);s.it.next=undefined;s.it.return=function(){n++;return {};};assert.throws(TypeError,function(){Array.from(s.source);});assert.sameValue(n,0);
// CASE: no-close-noncallable-next-null
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var n=0,s=sequence([]);s.it.next=null;s.it.return=function(){n++;return {};};assert.throws(TypeError,function(){Array.from(s.source);});assert.sameValue(n,0);
// CASE: no-close-noncallable-next-number
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var n=0,s=sequence([]);s.it.next=1;s.it.return=function(){n++;return {};};assert.throws(TypeError,function(){Array.from(s.source);});assert.sameValue(n,0);
// CASE: no-close-noncallable-next-object
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var n=0,s=sequence([]);s.it.next=({});s.it.return=function(){n++;return {};};assert.throws(TypeError,function(){Array.from(s.source);});assert.sameValue(n,0);
// CASE: no-close-nonobject-step-undefined
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var n=0,s=sequence([]);s.it.next=function(){return undefined;};s.it.return=function(){n++;return {};};assert.throws(TypeError,function(){Array.from(s.source);});assert.sameValue(n,0);
// CASE: no-close-nonobject-step-null
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var n=0,s=sequence([]);s.it.next=function(){return null;};s.it.return=function(){n++;return {};};assert.throws(TypeError,function(){Array.from(s.source);});assert.sameValue(n,0);
// CASE: no-close-nonobject-step-number
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var n=0,s=sequence([]);s.it.next=function(){return 3;};s.it.return=function(){n++;return {};};assert.throws(TypeError,function(){Array.from(s.source);});assert.sameValue(n,0);
// CASE: no-close-nonobject-step-string
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var n=0,s=sequence([]);s.it.next=function(){return "x";};s.it.return=function(){n++;return {};};assert.throws(TypeError,function(){Array.from(s.source);});assert.sameValue(n,0);
// CASE: no-close-nonobject-step-boolean
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var n=0,s=sequence([]);s.it.next=function(){return false;};s.it.return=function(){n++;return {};};assert.throws(TypeError,function(){Array.from(s.source);});assert.sameValue(n,0);
// CASE: no-close-nonobject-step-symbol
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var n=0,s=sequence([]);s.it.next=function(){return Symbol("x");};s.it.return=function(){n++;return {};};assert.throws(TypeError,function(){Array.from(s.source);});assert.sameValue(n,0);
// CASE: done-before-value-and-no-coercion
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var s=sequence([]),done={valueOf:function(){throw "coerce";},toString:function(){throw "coerce";}};s.it.next=function(){return {done:done,get value(){throw "value";}};};sameList(Array.from(s.source),[]);
// CASE: missing-done-is-false-missing-value-is-undefined
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var s=sequence([]),n=0;s.it.next=function(){return n++?{done:true}:{};};var a=Array.from(s.source);sameList(a,[undefined]);assert.sameValue(Object.prototype.hasOwnProperty.call(a,"0"),true);
// CASE: normal-exhaustion-never-gets-return
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var s=sequence([1]);Object.defineProperty(s.it,"return",{get:function(){throw "return";}});sameList(Array.from(s.source),[1]);
// CASE: mapper-value-index-two-arguments-iterable
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var ctx={},calls=0;var a=Array.from(sequence([3,5]).source,function(v,k){"use strict";assert.sameValue(this,ctx);assert.sameValue(arguments.length,2);assert.sameValue(k,calls++);return v+k;},ctx);sameList(a,[3,6]);assert.sameValue(calls,2);
// CASE: mapper-strict-undefined-this-iterable
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=Array.from(sequence([3,5]).source,function(v){"use strict";assert.sameValue(this,undefined);return v;});sameList(a,[3,5]);
// CASE: mapper-strict-null-this-iterable
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
sameList(Array.from(sequence([3,5]).source,function(v){"use strict";assert.sameValue(this,null);return v;},null),[3,5]);
// CASE: mapper-sloppy-global-this-iterable
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var f=Function("v","assert.sameValue(this,globalThis);return v;");sameList(Array.from(sequence([3,5]).source,f,null),[3,5]);
// CASE: mapper-sloppy-boxed-this-iterable
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var f=Function("v","assert.sameValue(typeof this,\"object\");assert.sameValue(this.valueOf(),7);return v;");sameList(Array.from(sequence([3,5]).source,f,7),[3,5]);
// CASE: mapper-bound-this-iterable
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var ctx={};function f(v,k){assert.sameValue(this,ctx);return v+k;}sameList(Array.from(sequence([3,5]).source,f.bind(ctx),{}),[3,6]);
// CASE: mapper-arrow-lexical-this-iterable
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var ctx={};function f(){return Array.from(sequence([3,5]).source,v=>{assert.sameValue(this,ctx);return v;},{});}sameList(f.call(ctx),[3,5]);
// CASE: mapper-value-index-two-arguments-array-like
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var ctx={},calls=0;var a=Array.from(({0:3,1:5,length:2}),function(v,k){"use strict";assert.sameValue(this,ctx);assert.sameValue(arguments.length,2);assert.sameValue(k,calls++);return v+k;},ctx);sameList(a,[3,6]);assert.sameValue(calls,2);
// CASE: mapper-strict-undefined-this-array-like
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=Array.from(({0:3,1:5,length:2}),function(v){"use strict";assert.sameValue(this,undefined);return v;});sameList(a,[3,5]);
// CASE: mapper-strict-null-this-array-like
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
sameList(Array.from(({0:3,1:5,length:2}),function(v){"use strict";assert.sameValue(this,null);return v;},null),[3,5]);
// CASE: mapper-sloppy-global-this-array-like
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var f=Function("v","assert.sameValue(this,globalThis);return v;");sameList(Array.from(({0:3,1:5,length:2}),f,null),[3,5]);
// CASE: mapper-sloppy-boxed-this-array-like
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var f=Function("v","assert.sameValue(typeof this,\"object\");assert.sameValue(this.valueOf(),7);return v;");sameList(Array.from(({0:3,1:5,length:2}),f,7),[3,5]);
// CASE: mapper-bound-this-array-like
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var ctx={};function f(v,k){assert.sameValue(this,ctx);return v+k;}sameList(Array.from(({0:3,1:5,length:2}),f.bind(ctx),{}),[3,6]);
// CASE: mapper-arrow-lexical-this-array-like
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var ctx={};function f(){return Array.from(({0:3,1:5,length:2}),v=>{assert.sameValue(this,ctx);return v;},{});}sameList(f.call(ctx),[3,5]);
// CASE: mapper-native-callable
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
sameList(Array.from({0:"2",1:"3",length:2},Number),[2,3]);
// CASE: mapper-value-captured-before-mutation
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var item={},other={},r={value:item,done:false},s=sequence([]),n=0;s.it.next=function(){return n++?{done:true}:r;};var a=Array.from(s.source,function(v){r.value=other;assert.sameValue(v,item);return v;});assert.sameValue(a[0],item);
// CASE: mapper-returns-undefined-and-negative-zero
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=Array.from(sequence([1,2]).source,function(v){return v===1?undefined:-0;});assert.sameValue(a[0],undefined);assert.sameValue(a[1],-0);
// CASE: mapper-output-defined-before-next-step
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var out={},n=0,s=sequence([]);function C(){return out;}s.it.next=function(){if(n===1)assert.sameValue(out[0],8);return n++?{done:true}:{value:4};};assert.sameValue(Array.from.call(C,s.source,function(v){return v*2;}),out);assert.sameValue(out[0],8);
// CASE: mapper-reentrant-array-from
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var calls=0;var a=Array.from(sequence([2,3]).source,function(v){calls++;return Array.from({0:v+1,length:1})[0];});sameList(a,[3,4]);assert.sameValue(calls,2);
// CASE: mapper-native-iterator-growth-live
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=[2];sameList(Array.from(a,function(v){if(v===2)a.push(4);return v;}),[2,4]);
// CASE: mapper-native-iterator-shrink-live
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=[2,4];sameList(Array.from(a,function(v){a.length=0;return v;}),[2]);
// CASE: mapper-next-replacement-does-not-replace-cache
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var s=sequence([2,4]);sameList(Array.from(s.source,function(v){s.it.next=function(){throw "replacement";};return v;}),[2,4]);
// CASE: constructor-args-newtarget-iterable
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var seen=0;function C(n){seen++;assert.sameValue(arguments.length,0);assert.sameValue(n,undefined);assert.sameValue(new.target,C);}var a=Array.from.call(C,sequence([2,4]).source);assert.sameValue(seen,1);assert.sameValue(Object.getPrototypeOf(a),C.prototype);sameList(a,[2,4]);
// CASE: constructor-return-existing-object-iterable
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var out={keep:9};function C(){return out;}var a=Array.from.call(C,sequence([2,4]).source);assert.sameValue(a,out);assert.sameValue(a.keep,9);sameList(a,[2,4]);
// CASE: constructor-return-primitive-ignored-iterable
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
function C(){this.keep=8;return 3;}var a=Array.from.call(C,sequence([2,4]).source);assert.sameValue(a.keep,8);assert.sameValue(Object.getPrototypeOf(a),C.prototype);sameList(a,[2,4]);
// CASE: bound-constructor-iterable
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var seen=0;function C(prefix,n){assert.sameValue(prefix,"p");assert.sameValue(arguments.length,1);assert.sameValue(n,undefined);assert.sameValue(new.target,C);seen++;}var bound=C.bind({ignored:true},"p");var a=Array.from.call(bound,sequence([2,4]).source);assert.sameValue(seen,1);assert.sameValue(Object.getPrototypeOf(a),C.prototype);sameList(a,[2,4]);
// CASE: native-Object-constructor-iterable
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=Array.from.call(Object,sequence([2,4]).source);assert.sameValue(Array.isArray(a),false);sameList(a,[2,4]);
// CASE: constructor-args-newtarget-array-like
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var seen=0;function C(n){seen++;assert.sameValue(arguments.length,1);assert.sameValue(n,2);assert.sameValue(new.target,C);}var a=Array.from.call(C,({0:2,1:4,length:2}));assert.sameValue(seen,1);assert.sameValue(Object.getPrototypeOf(a),C.prototype);sameList(a,[2,4]);
// CASE: constructor-return-existing-object-array-like
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var out={keep:9};function C(){return out;}var a=Array.from.call(C,({0:2,1:4,length:2}));assert.sameValue(a,out);assert.sameValue(a.keep,9);sameList(a,[2,4]);
// CASE: constructor-return-primitive-ignored-array-like
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
function C(){this.keep=8;return 3;}var a=Array.from.call(C,({0:2,1:4,length:2}));assert.sameValue(a.keep,8);assert.sameValue(Object.getPrototypeOf(a),C.prototype);sameList(a,[2,4]);
// CASE: bound-constructor-array-like
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var seen=0;function C(prefix,n){assert.sameValue(prefix,"p");assert.sameValue(arguments.length,2);assert.sameValue(n,2);assert.sameValue(new.target,C);seen++;}var bound=C.bind({ignored:true},"p");var a=Array.from.call(bound,({0:2,1:4,length:2}));assert.sameValue(seen,1);assert.sameValue(Object.getPrototypeOf(a),C.prototype);sameList(a,[2,4]);
// CASE: native-Object-constructor-array-like
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=Array.from.call(Object,({0:2,1:4,length:2}));assert.sameValue(Array.isArray(a),false);sameList(a,[2,4]);
// CASE: nonconstructor-receiver-intrinsic-fallback-null
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=Array.from.call(null,sequence([7]).source);assert.sameValue(Array.isArray(a),true);assert.sameValue(Object.getPrototypeOf(a),Array.prototype);sameList(a,[7]);
// CASE: nonconstructor-receiver-intrinsic-fallback-undefined
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=Array.from.call(undefined,sequence([7]).source);assert.sameValue(Array.isArray(a),true);assert.sameValue(Object.getPrototypeOf(a),Array.prototype);sameList(a,[7]);
// CASE: nonconstructor-receiver-intrinsic-fallback-object
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=Array.from.call(({}),sequence([7]).source);assert.sameValue(Array.isArray(a),true);assert.sameValue(Object.getPrototypeOf(a),Array.prototype);sameList(a,[7]);
// CASE: nonconstructor-receiver-intrinsic-fallback-number
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=Array.from.call(4,sequence([7]).source);assert.sameValue(Array.isArray(a),true);assert.sameValue(Object.getPrototypeOf(a),Array.prototype);sameList(a,[7]);
// CASE: nonconstructor-receiver-intrinsic-fallback-arrow
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=Array.from.call((()=>{throw "called";}),sequence([7]).source);assert.sameValue(Array.isArray(a),true);assert.sameValue(Object.getPrototypeOf(a),Array.prototype);sameList(a,[7]);
// CASE: nonconstructor-receiver-intrinsic-fallback-native-nonconstructor
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=Array.from.call(Math.abs,sequence([7]).source);assert.sameValue(Array.isArray(a),true);assert.sameValue(Object.getPrototypeOf(a),Array.prototype);sameList(a,[7]);
// CASE: source-constructor-species-not-observed
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
assert.sameValue(typeof Symbol.species,"symbol");var a=[3];Object.defineProperty(a,"constructor",{get:function(){throw "constructor";}});Object.defineProperty(a,Symbol.species,{get:function(){throw "species";}});sameList(Array.from(a),[3]);
// CASE: receiver-species-not-observed
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
assert.sameValue(typeof Symbol.species,"symbol");function C(){}Object.defineProperty(C,Symbol.species,{get:function(){throw "species";}});sameList(Array.from.call(C,sequence([2]).source),[2]);
// CASE: saved-from-and-intrinsic-array-survive-global-replacement
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var from=Array.from,proto=Array.prototype;Array=function(){throw "global";};var a=from.call(null,{0:5,length:1});assert.sameValue(Object.getPrototypeOf(a),proto);sameList(a,[5]);
// CASE: constructor-mutates-arraylike-after-length-capture
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={0:2,1:4,length:2};function C(n){assert.sameValue(n,2);o.length=0;o[1]=9;}sameList(Array.from.call(C,o),[2,9]);
// CASE: constructor-output-can-be-source
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={0:1,1:2,length:2};function C(){return o;}var a=Array.from.call(C,o,function(v,k){if(k===0)o[1]=9;return v*2;});assert.sameValue(a,o);sameList(a,[2,18]);
// CASE: constructor-returns-mapped-arguments
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var captured;var f=Function("a","b","captured=arguments;function C(){return captured;}var r=Array.from.call(C,{0:7,1:8,length:2});assert.sameValue(a,7);assert.sameValue(b,8);return r;");var r=f(1,2);sameList(r,[7,8]);
// CASE: to-length-missing
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=Array.from({0:3,1:4,length:undefined});assert.sameValue(a.length,0);if(a.length)assert.sameValue(a[0],3);
// CASE: to-length-zero
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=Array.from({0:3,1:4,length:0});assert.sameValue(a.length,0);if(a.length)assert.sameValue(a[0],3);
// CASE: to-length-negative-zero
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=Array.from({0:3,1:4,length:-0});assert.sameValue(a.length,0);if(a.length)assert.sameValue(a[0],3);
// CASE: to-length-negative
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=Array.from({0:3,1:4,length:-2});assert.sameValue(a.length,0);if(a.length)assert.sameValue(a[0],3);
// CASE: to-length-nan
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=Array.from({0:3,1:4,length:NaN});assert.sameValue(a.length,0);if(a.length)assert.sameValue(a[0],3);
// CASE: to-length-negative-infinity
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=Array.from({0:3,1:4,length:-Infinity});assert.sameValue(a.length,0);if(a.length)assert.sameValue(a[0],3);
// CASE: to-length-fraction
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=Array.from({0:3,1:4,length:2.9});assert.sameValue(a.length,2);if(a.length)assert.sameValue(a[0],3);
// CASE: to-length-string
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=Array.from({0:3,1:4,length:"2"});assert.sameValue(a.length,2);if(a.length)assert.sameValue(a[0],3);
// CASE: to-length-null
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=Array.from({0:3,1:4,length:null});assert.sameValue(a.length,0);if(a.length)assert.sameValue(a[0],3);
// CASE: to-length-boolean
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=Array.from({0:3,1:4,length:true});assert.sameValue(a.length,1);if(a.length)assert.sameValue(a[0],3);
// CASE: to-length-number-hint-once
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var log=[],o={0:7,get length(){log.push("length");return {valueOf:function(){log.push("valueOf");return {};},toString:function(){log.push("toString");return "1";}};}};sameList(Array.from(o),[7]);assert.sameValue(traceText(log),"length,valueOf,toString");
// CASE: length-get-throw-before-constructor
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},got,n=0,o={get length(){throw token;}};function C(){n++;}try{Array.from.call(C,o);}catch(e){got=e;}assert.sameValue(got,token);assert.sameValue(n,0);
// CASE: symbol-length-rejected-before-constructor
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var n=0;function C(){n++;}assert.throws(TypeError,function(){Array.from.call(C,{length:Symbol("n")});});assert.sameValue(n,0);
// CASE: arraylike-length-once-indices-live
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var reads=0,o={0:2,1:4,get length(){reads++;return 2;}};var a=Array.from(o,function(v,k){if(k===0){o[1]=9;Object.defineProperty(o,"length",{value:0});}return v;});sameList(a,[2,9]);assert.sameValue(reads,1);
// CASE: arraylike-growth-does-not-extend
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={0:1,length:1};sameList(Array.from(o,function(v){o[1]=8;o.length=2;return v;}),[1]);
// CASE: arraylike-deletion-becomes-undefined-own-data
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={0:1,1:2,length:2};var a=Array.from(o,function(v,k){if(k===0)delete o[1];return v;});sameList(a,[1,undefined]);assert.sameValue(Object.prototype.hasOwnProperty.call(a,"1"),true);
// CASE: holes-inherited-getter-original-receiver
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var p={},o=Object.create(p),n=0;o.length=2;Object.defineProperty(p,"1",{get:function(){assert.sameValue(this,o);n++;return 8;}});var a=Array.from(o);sameList(a,[undefined,8]);assert.sameValue(n,1);verifyProperty(a,"0",{value:undefined,writable:true,enumerable:true,configurable:true},{restore:true});
// CASE: arraylike-index-getter-mutates-next
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var o={length:2,get 0(){o[1]=9;return 2;}};sameList(Array.from(o),[2,9]);
// CASE: arraylike-mapper-abrupt-keeps-prefix
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var out={},token={},got,o={0:1,1:2,length:2};function C(){return out;}try{Array.from.call(C,o,function(v,k){if(k===1)throw token;return v*3;});}catch(e){got=e;}assert.sameValue(got,token);assert.sameValue(out[0],3);assert.sameValue(Object.prototype.hasOwnProperty.call(out,"1"),false);assert.sameValue(Object.prototype.hasOwnProperty.call(out,"length"),false);
// CASE: output-own-data-bypasses-inherited-setter-iterable
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var n=0,p={};Object.defineProperty(p,"0",{set:function(){n++;throw "setter";}});function C(){}C.prototype=p;var a=Array.from.call(C,sequence([7]).source);assert.sameValue(n,0);verifyProperty(a,"0",{value:7,writable:true,enumerable:true,configurable:true},{restore:true});
// CASE: output-shadow-inherited-nonwritable-iterable
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var p={};Object.defineProperty(p,"0",{value:1});function C(){}C.prototype=p;var a=Array.from.call(C,sequence([7]).source);verifyProperty(a,"0",{value:7,writable:true,enumerable:true,configurable:true},{restore:true});
// CASE: output-replaces-configurable-accessor-iterable
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var out={};Object.defineProperty(out,"0",{get:function(){throw "getter";},set:function(){throw "setter";},configurable:true});function C(){return out;}assert.sameValue(Array.from.call(C,sequence([7]).source),out);verifyProperty(out,"0",{value:7,writable:true,enumerable:true,configurable:true},{restore:true});
// CASE: output-length-inherited-setter-iterable
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var seen=0,p={};Object.defineProperty(p,"length",{set:function(v){seen++;assert.sameValue(v,1);assert.sameValue(this[0],7);}});function C(){}C.prototype=p;var a=Array.from.call(C,sequence([7]).source);assert.sameValue(seen,1);assert.sameValue(Object.prototype.hasOwnProperty.call(a,"length"),false);
// CASE: output-length-nonwritable-same-value-throws-iterable
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var out={};Object.defineProperty(out,"length",{value:1});function C(){return out;}assert.throws(TypeError,function(){Array.from.call(C,sequence([7]).source);});assert.sameValue(out[0],7);
// CASE: output-length-getter-only-throws-iterable
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var out={get length(){return 1;}};function C(){return out;}assert.throws(TypeError,function(){Array.from.call(C,sequence([7]).source);});assert.sameValue(out[0],7);
// CASE: nonextensible-output-cannot-add-index-iterable
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var out=Object.preventExtensions({});function C(){return out;}assert.throws(TypeError,function(){Array.from.call(C,sequence([7]).source);});assert.sameValue(Object.prototype.hasOwnProperty.call(out,"0"),false);
// CASE: nonconfigurable-output-index-cannot-redefine-iterable
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var out={};Object.defineProperty(out,"0",{value:7,writable:true,enumerable:true,configurable:false});function C(){return out;}assert.throws(TypeError,function(){Array.from.call(C,sequence([7]).source);});assert.sameValue(out[0],7);
// CASE: output-own-data-bypasses-inherited-setter-array-like
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var n=0,p={};Object.defineProperty(p,"0",{set:function(){n++;throw "setter";}});function C(){}C.prototype=p;var a=Array.from.call(C,({0:7,length:1}));assert.sameValue(n,0);verifyProperty(a,"0",{value:7,writable:true,enumerable:true,configurable:true},{restore:true});
// CASE: output-shadow-inherited-nonwritable-array-like
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var p={};Object.defineProperty(p,"0",{value:1});function C(){}C.prototype=p;var a=Array.from.call(C,({0:7,length:1}));verifyProperty(a,"0",{value:7,writable:true,enumerable:true,configurable:true},{restore:true});
// CASE: output-replaces-configurable-accessor-array-like
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var out={};Object.defineProperty(out,"0",{get:function(){throw "getter";},set:function(){throw "setter";},configurable:true});function C(){return out;}assert.sameValue(Array.from.call(C,({0:7,length:1})),out);verifyProperty(out,"0",{value:7,writable:true,enumerable:true,configurable:true},{restore:true});
// CASE: output-length-inherited-setter-array-like
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var seen=0,p={};Object.defineProperty(p,"length",{set:function(v){seen++;assert.sameValue(v,1);assert.sameValue(this[0],7);}});function C(){}C.prototype=p;var a=Array.from.call(C,({0:7,length:1}));assert.sameValue(seen,1);assert.sameValue(Object.prototype.hasOwnProperty.call(a,"length"),false);
// CASE: output-length-nonwritable-same-value-throws-array-like
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var out={};Object.defineProperty(out,"length",{value:1});function C(){return out;}assert.throws(TypeError,function(){Array.from.call(C,({0:7,length:1}));});assert.sameValue(out[0],7);
// CASE: output-length-getter-only-throws-array-like
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var out={get length(){return 1;}};function C(){return out;}assert.throws(TypeError,function(){Array.from.call(C,({0:7,length:1}));});assert.sameValue(out[0],7);
// CASE: nonextensible-output-cannot-add-index-array-like
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var out=Object.preventExtensions({});function C(){return out;}assert.throws(TypeError,function(){Array.from.call(C,({0:7,length:1}));});assert.sameValue(Object.prototype.hasOwnProperty.call(out,"0"),false);
// CASE: nonconfigurable-output-index-cannot-redefine-array-like
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var out={};Object.defineProperty(out,"0",{value:7,writable:true,enumerable:true,configurable:false});function C(){return out;}assert.throws(TypeError,function(){Array.from.call(C,({0:7,length:1}));});assert.sameValue(out[0],7);
// CASE: output-definition-sees-mapper-frozen-property
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var out={},s=sequence([1]),n=0;function C(){return out;}s.it.return=function(){n++;return {};};assert.throws(TypeError,function(){Array.from.call(C,s.source,function(v){Object.defineProperty(out,"0",{value:v});return v;});});assert.sameValue(n,1);
// CASE: boxed-string-output-index-rejects
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var out=Object("x"),s=sequence(["x"]),n=0;function C(){return out;}s.it.return=function(){n++;return {};};assert.throws(TypeError,function(){Array.from.call(C,s.source);});assert.sameValue(n,1);
// CASE: iterable-final-length-throw-no-close
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},got,n=0,out={},s=sequence([1]);Object.defineProperty(out,"length",{set:function(v){assert.sameValue(v,1);throw token;}});s.it.return=function(){n++;return {};};function C(){return out;}try{Array.from.call(C,s.source);}catch(e){got=e;}assert.sameValue(got,token);assert.sameValue(n,0);assert.sameValue(out[0],1);
// CASE: empty-iterable-final-length-throw-no-close
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var n=0,out={},s=sequence([]);Object.defineProperty(out,"length",{value:0,writable:false});s.it.return=function(){n++;return {};};function C(){return out;}assert.throws(TypeError,function(){Array.from.call(C,s.source);});assert.sameValue(n,0);
// CASE: close-mapper-missing
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},closeToken={},got,closes=0,s=sequence([7]),out={};function C(){return out;}try{Array.from.call(C,s.source,function(){throw token;});}catch(e){got=e;}assert.sameValue(got,token);assert.sameValue(closes,0);
// CASE: close-mapper-undefined
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},closeToken={},got,closes=0,s=sequence([7]),out={};function C(){return out;}s.it.return=undefined;try{Array.from.call(C,s.source,function(){throw token;});}catch(e){got=e;}assert.sameValue(got,token);assert.sameValue(closes,0);
// CASE: close-mapper-null
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},closeToken={},got,closes=0,s=sequence([7]),out={};function C(){return out;}s.it.return=null;try{Array.from.call(C,s.source,function(){throw token;});}catch(e){got=e;}assert.sameValue(got,token);assert.sameValue(closes,0);
// CASE: close-mapper-noncallable
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},closeToken={},got,closes=0,s=sequence([7]),out={};function C(){return out;}s.it.return=1;try{Array.from.call(C,s.source,function(){throw token;});}catch(e){got=e;}assert.sameValue(got,token);assert.sameValue(closes,0);
// CASE: close-mapper-getter-throw
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},closeToken={},got,closes=0,s=sequence([7]),out={};function C(){return out;}Object.defineProperty(s.it,"return",{get:function(){closes++;throw closeToken;}});try{Array.from.call(C,s.source,function(){throw token;});}catch(e){got=e;}assert.sameValue(got,token);assert.sameValue(closes,1);
// CASE: close-mapper-call-throw
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},closeToken={},got,closes=0,s=sequence([7]),out={};function C(){return out;}s.it.return=function(){closes++;throw closeToken;};try{Array.from.call(C,s.source,function(){throw token;});}catch(e){got=e;}assert.sameValue(got,token);assert.sameValue(closes,1);
// CASE: close-mapper-primitive
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},closeToken={},got,closes=0,s=sequence([7]),out={};function C(){return out;}s.it.return=function(){closes++;return 2;};try{Array.from.call(C,s.source,function(){throw token;});}catch(e){got=e;}assert.sameValue(got,token);assert.sameValue(closes,1);
// CASE: close-mapper-object
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},closeToken={},got,closes=0,s=sequence([7]),out={};function C(){return out;}s.it.return=function(){closes++;assert.sameValue(this,s.it);assert.sameValue(arguments.length,0);return {};};try{Array.from.call(C,s.source,function(){throw token;});}catch(e){got=e;}assert.sameValue(got,token);assert.sameValue(closes,1);
// CASE: close-definition-missing
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},closeToken={},got,closes=0,s=sequence([7]),out={};function C(){return out;}Object.preventExtensions(out);assert.throws(TypeError,function(){Array.from.call(C,s.source);});assert.sameValue(closes,0);
// CASE: close-definition-undefined
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},closeToken={},got,closes=0,s=sequence([7]),out={};function C(){return out;}s.it.return=undefined;Object.preventExtensions(out);assert.throws(TypeError,function(){Array.from.call(C,s.source);});assert.sameValue(closes,0);
// CASE: close-definition-null
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},closeToken={},got,closes=0,s=sequence([7]),out={};function C(){return out;}s.it.return=null;Object.preventExtensions(out);assert.throws(TypeError,function(){Array.from.call(C,s.source);});assert.sameValue(closes,0);
// CASE: close-definition-noncallable
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},closeToken={},got,closes=0,s=sequence([7]),out={};function C(){return out;}s.it.return=1;Object.preventExtensions(out);assert.throws(TypeError,function(){Array.from.call(C,s.source);});assert.sameValue(closes,0);
// CASE: close-definition-getter-throw
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},closeToken={},got,closes=0,s=sequence([7]),out={};function C(){return out;}Object.defineProperty(s.it,"return",{get:function(){closes++;throw closeToken;}});Object.preventExtensions(out);assert.throws(TypeError,function(){Array.from.call(C,s.source);});assert.sameValue(closes,1);
// CASE: close-definition-call-throw
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},closeToken={},got,closes=0,s=sequence([7]),out={};function C(){return out;}s.it.return=function(){closes++;throw closeToken;};Object.preventExtensions(out);assert.throws(TypeError,function(){Array.from.call(C,s.source);});assert.sameValue(closes,1);
// CASE: close-definition-primitive
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},closeToken={},got,closes=0,s=sequence([7]),out={};function C(){return out;}s.it.return=function(){closes++;return 2;};Object.preventExtensions(out);assert.throws(TypeError,function(){Array.from.call(C,s.source);});assert.sameValue(closes,1);
// CASE: close-definition-object
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},closeToken={},got,closes=0,s=sequence([7]),out={};function C(){return out;}s.it.return=function(){closes++;assert.sameValue(this,s.it);assert.sameValue(arguments.length,0);return {};};Object.preventExtensions(out);assert.throws(TypeError,function(){Array.from.call(C,s.source);});assert.sameValue(closes,1);
// CASE: mapper-mutates-live-return-before-throw
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},got,n=0,s=sequence([1]);s.it.return=function(){throw "old";};try{Array.from(s.source,function(){s.it.return=function(){n++;return {};};throw token;});}catch(e){got=e;}assert.sameValue(got,token);assert.sameValue(n,1);
// CASE: close-preserves-output-prefix-and-stops-next
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},got,n=0,closes=0,out={},s=sequence([]);function C(){return out;}s.it.next=function(){return {value:++n};};s.it.return=function(){closes++;assert.sameValue(out[0],2);assert.sameValue(Object.prototype.hasOwnProperty.call(out,"1"),false);return {};};try{Array.from.call(C,s.source,function(v,k){if(k===1)throw token;return v*2;});}catch(e){got=e;}assert.sameValue(got,token);assert.sameValue(n,2);assert.sameValue(closes,1);
// CASE: sparse-array-becomes-dense-undefined
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=Array.from(new Array(2));sameList(a,[undefined,undefined]);assert.sameValue(Object.prototype.hasOwnProperty.call(a,"0"),true);assert.sameValue(Object.prototype.hasOwnProperty.call(a,"1"),true);
// CASE: array-custom-iterator-override
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=[1,2],s=sequence([9]);a[Symbol.iterator]=s.source[Symbol.iterator];sameList(Array.from(a),[9]);
// CASE: array-null-iterator-uses-arraylike
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=[1,2];a[Symbol.iterator]=null;sameList(Array.from(a),[1,2]);
// CASE: string-empty
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
sameList(Array.from(""),[]);
// CASE: string-ascii
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
sameList(Array.from("ab"),["a","b"]);
// CASE: string-supplementary
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
sameList(Array.from("A\uD83D\uDE00B"),["A","\ud83d\ude00","B"]);
// CASE: string-high
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
sameList(Array.from("\uD800x"),["\ud800","x"]);
// CASE: string-low
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
sameList(Array.from("x\uDC00"),["x","\udc00"]);
// CASE: string-high-high-low
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
sameList(Array.from("\uD800\uD801\uDC00"),["\ud800","\ud801\udc00"]);
// CASE: string-combining
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
sameList(Array.from("e\u0301\u00e9"),["e","\u0301","\u00e9"]);
// CASE: disabled-string-iterator-uses-codeunits
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
String.prototype[Symbol.iterator]=null;var a=Array.from("A\uD83D\uDE00B");sameList(a,["A","\uD83D","\uDE00","B"]);
// CASE: mapped-arguments-mutation-live
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var f=Function("a","b","return Array.from(arguments,function(v,k){if(k===0)b=9;return v;});");sameList(f(1,2),[1,9]);
// CASE: strict-arguments-are-unmapped
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
function f(a,b){"use strict";return Array.from(arguments,function(v,k){if(k===0)b=9;return v;});}sameList(f(1,2),[1,2]);
// CASE: arguments-disabled-iterator-arraylike
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
function f(){arguments[Symbol.iterator]=undefined;return Array.from(arguments);}sameList(f(2,3),[2,3]);
// CASE: primitive-arraylike-empty-number
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
sameList(Array.from(7),[]);
// CASE: primitive-arraylike-empty-boolean
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
sameList(Array.from(true),[]);
// CASE: primitive-arraylike-empty-symbol
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
sameList(Array.from(Symbol("x")),[]);
// CASE: huge-length-custom-ctor-first-u32plus
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},got,o={get 0(){throw "index";},length:4294967296};function C(n){assert.sameValue(n,4294967296);throw token;}try{Array.from.call(C,o);}catch(e){got=e;}assert.sameValue(got,token);
// CASE: huge-length-native-array-range-u32plus
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var reads=0,o={length:4294967296,get 0(){reads++;throw "index";}};assert.throws(RangeError,function(){Array.from(o);});assert.sameValue(reads,0);
// CASE: huge-length-custom-ctor-first-safe-max
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},got,o={get 0(){throw "index";},length:9007199254740991};function C(n){assert.sameValue(n,9007199254740991);throw token;}try{Array.from.call(C,o);}catch(e){got=e;}assert.sameValue(got,token);
// CASE: huge-length-native-array-range-safe-max
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var reads=0,o={length:9007199254740991,get 0(){reads++;throw "index";}};assert.throws(RangeError,function(){Array.from(o);});assert.sameValue(reads,0);
// CASE: huge-length-custom-ctor-first-above-safe
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},got,o={get 0(){throw "index";},length:9007199254740992};function C(n){assert.sameValue(n,9007199254740991);throw token;}try{Array.from.call(C,o);}catch(e){got=e;}assert.sameValue(got,token);
// CASE: huge-length-native-array-range-above-safe
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var reads=0,o={length:9007199254740992,get 0(){reads++;throw "index";}};assert.throws(RangeError,function(){Array.from(o);});assert.sameValue(reads,0);
// CASE: huge-length-custom-ctor-first-infinity
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},got,o={get 0(){throw "index";},length:Infinity};function C(n){assert.sameValue(n,9007199254740991);throw token;}try{Array.from.call(C,o);}catch(e){got=e;}assert.sameValue(got,token);
// CASE: huge-length-native-array-range-infinity
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var reads=0,o={length:Infinity,get 0(){reads++;throw "index";}};assert.throws(RangeError,function(){Array.from(o);});assert.sameValue(reads,0);
// CASE: huge-custom-range-first-getter-before-resource
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},got;function C(){return {};}try{Array.from.call(C,{length:Infinity,get 0(){throw token;}});}catch(e){got=e;}assert.sameValue(got,token);
// CASE: huge-custom-range-first-mapper-before-resource
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var token={},got;function C(){return {};}try{Array.from.call(C,{0:7,length:Infinity},function(v,k){assert.sameValue(v,7);assert.sameValue(k,0);throw token;});}catch(e){got=e;}assert.sameValue(got,token);
// CASE: metadata
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var from=Array.from;verifyProperty(Array,"from",{value:from,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(from,"name",{value:"from",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(from,"length",{value:1,writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.prototype.hasOwnProperty.call(from,"prototype"),false);assert.throws(TypeError,function(){new from({length:0});});
// CASE: saved-alias-call-apply-bind
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var from=Array.from;Array.from=3;sameList(from.call(null,{0:1,length:1}),[1]);sameList(from.apply(null,[{0:2,length:1}]),[2]);sameList(from.bind(null)({0:3,length:1}),[3]);
// CASE: reflect-apply
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
sameList(Reflect.apply(Array.from,null,[{0:5,length:1}]),[5]);
// CASE: prerequisite-generator
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
function* g(){yield 2;yield 4;}sameList(Array.from(g()),[2,4]);
// CASE: prerequisite-class-subclass
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
class C extends Array{}var a=C.from([2,4]);assert.sameValue(a instanceof C,true);sameList(a,[2,4]);
// CASE: prerequisite-Set
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
sameList(Array.from(new Set([2,2,4])),[2,4]);
// CASE: prerequisite-Map
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var a=Array.from(new Map([["a",2],["b",4]]));sameList(a[0],["a",2]);sameList(a[1],["b",4]);
// CASE: prerequisite-TypedArray
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
sameList(Array.from(new Uint8Array([2,4])),[2,4]);
// CASE: prerequisite-Proxy-definition-order
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var log=[],target={},proxy=new Proxy(target,{defineProperty:function(o,k,d){log.push(k+":"+d.value);return Reflect.defineProperty(o,k,d);}});function C(){return proxy;}sameList(Array.from.call(C,sequence([7]).source),[7]);assert.sameValue(log[0],"0:7");assert.sameValue(log[1],"length:1");
// CASE: prerequisite-BigInt-length-rejected
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
assert.throws(TypeError,function(){Array.from({length:BigInt(1)});});
// CASE: budget-infinite-iterator
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var s=sequence([]);s.it.next=function(){return {value:1,done:false};};Array.from(s.source);
// CASE: budget-infinite-mapper
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
Array.from(sequence([1]).source,function(){while(true){}});
// CASE: budget-infinite-close
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var s=sequence([1]);s.it.return=function(){while(true){}};Array.from(s.source,function(){throw 1;});
// CASE: budget-recursive-mapper
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
function f(){return Array.from(sequence([1]).source,f);}f();
// CASE: budget-retained-results
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
var kept=[];while(true)kept.push(Array.from({0:1,length:1}));
// CASE: budget-huge-custom-arraylike
assert.sameValue(typeof Array.from,'function','Array.from availability');
var $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);
var $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};
var $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);
function sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}
function sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}
function traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}
function C(){return {};}Array.from.call(C,{length:Infinity});
