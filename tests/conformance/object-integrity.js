// CASE: seal-intrinsic-metadata-and-not-a-constructor
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var d=Object.getOwnPropertyDescriptor(Object,name);
assert.sameValue(d.value,m);assert.sameValue(d.writable,true);assert.sameValue(d.enumerable,false);assert.sameValue(d.configurable,true);
var n=Object.getOwnPropertyDescriptor(m,'name'),l=Object.getOwnPropertyDescriptor(m,'length');
assert.sameValue(n.value,name);assert.sameValue(n.writable,false);assert.sameValue(n.enumerable,false);assert.sameValue(n.configurable,true);
assert.sameValue(l.value,1);assert.sameValue(l.writable,false);assert.sameValue(l.enumerable,false);assert.sameValue(l.configurable,true);
assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);
assert.throws(TypeError,function(){new m({});});
assert.notSameValue(Object.seal,Object.freeze);assert.notSameValue(Object.isSealed,Object.isFrozen);
// CASE: freeze-intrinsic-metadata-and-not-a-constructor
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var d=Object.getOwnPropertyDescriptor(Object,name);
assert.sameValue(d.value,m);assert.sameValue(d.writable,true);assert.sameValue(d.enumerable,false);assert.sameValue(d.configurable,true);
var n=Object.getOwnPropertyDescriptor(m,'name'),l=Object.getOwnPropertyDescriptor(m,'length');
assert.sameValue(n.value,name);assert.sameValue(n.writable,false);assert.sameValue(n.enumerable,false);assert.sameValue(n.configurable,true);
assert.sameValue(l.value,1);assert.sameValue(l.writable,false);assert.sameValue(l.enumerable,false);assert.sameValue(l.configurable,true);
assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);
assert.throws(TypeError,function(){new m({});});
assert.notSameValue(Object.seal,Object.freeze);assert.notSameValue(Object.isSealed,Object.isFrozen);
// CASE: isSealed-intrinsic-metadata-and-not-a-constructor
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isSealed",m=Object[name],isQuery=true,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var d=Object.getOwnPropertyDescriptor(Object,name);
assert.sameValue(d.value,m);assert.sameValue(d.writable,true);assert.sameValue(d.enumerable,false);assert.sameValue(d.configurable,true);
var n=Object.getOwnPropertyDescriptor(m,'name'),l=Object.getOwnPropertyDescriptor(m,'length');
assert.sameValue(n.value,name);assert.sameValue(n.writable,false);assert.sameValue(n.enumerable,false);assert.sameValue(n.configurable,true);
assert.sameValue(l.value,1);assert.sameValue(l.writable,false);assert.sameValue(l.enumerable,false);assert.sameValue(l.configurable,true);
assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);
assert.throws(TypeError,function(){new m({});});
assert.notSameValue(Object.seal,Object.freeze);assert.notSameValue(Object.isSealed,Object.isFrozen);
// CASE: isFrozen-intrinsic-metadata-and-not-a-constructor
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isFrozen",m=Object[name],isQuery=true,freezing=false,frozenQuery=true,strictMode=(function(){return this;})()===undefined;
var d=Object.getOwnPropertyDescriptor(Object,name);
assert.sameValue(d.value,m);assert.sameValue(d.writable,true);assert.sameValue(d.enumerable,false);assert.sameValue(d.configurable,true);
var n=Object.getOwnPropertyDescriptor(m,'name'),l=Object.getOwnPropertyDescriptor(m,'length');
assert.sameValue(n.value,name);assert.sameValue(n.writable,false);assert.sameValue(n.enumerable,false);assert.sameValue(n.configurable,true);
assert.sameValue(l.value,1);assert.sameValue(l.writable,false);assert.sameValue(l.enumerable,false);assert.sameValue(l.configurable,true);
assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);
assert.throws(TypeError,function(){new m({});});
assert.notSameValue(Object.seal,Object.freeze);assert.notSameValue(Object.isSealed,Object.isFrozen);
// CASE: seal-all-supported-primitives-and-omitted-argument
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var s=Symbol('primitive'),values=[undefined,null,false,true,0,-0,1,-2,NaN,Infinity,-Infinity,'','abc','\ud800',s];
for(var i=0;i<values.length;i++){assert.sameValue(m(values[i]),isQuery?true:values[i]);}
assert.sameValue(m(),isQuery?true:undefined);
// CASE: freeze-all-supported-primitives-and-omitted-argument
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var s=Symbol('primitive'),values=[undefined,null,false,true,0,-0,1,-2,NaN,Infinity,-Infinity,'','abc','\ud800',s];
for(var i=0;i<values.length;i++){assert.sameValue(m(values[i]),isQuery?true:values[i]);}
assert.sameValue(m(),isQuery?true:undefined);
// CASE: isSealed-all-supported-primitives-and-omitted-argument
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isSealed",m=Object[name],isQuery=true,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var s=Symbol('primitive'),values=[undefined,null,false,true,0,-0,1,-2,NaN,Infinity,-Infinity,'','abc','\ud800',s];
for(var i=0;i<values.length;i++){assert.sameValue(m(values[i]),isQuery?true:values[i]);}
assert.sameValue(m(),isQuery?true:undefined);
// CASE: isFrozen-all-supported-primitives-and-omitted-argument
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isFrozen",m=Object[name],isQuery=true,freezing=false,frozenQuery=true,strictMode=(function(){return this;})()===undefined;
var s=Symbol('primitive'),values=[undefined,null,false,true,0,-0,1,-2,NaN,Infinity,-Infinity,'','abc','\ud800',s];
for(var i=0;i<values.length;i++){assert.sameValue(m(values[i]),isQuery?true:values[i]);}
assert.sameValue(m(),isQuery?true:undefined);
// CASE: seal-ignored-receivers-call-apply-bind
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var receivers=[undefined,null,0,false,'receiver',Symbol('receiver'),{}];
for(var i=0;i<receivers.length;i++){var o={x:1};assert.sameValue(m.call(receivers[i],o),isQuery?false:o);}
var a={},b={};assert.sameValue(m.apply(null,[a]),isQuery?false:a);assert.sameValue(m.bind(7,b)(),isQuery?false:b);
// CASE: freeze-ignored-receivers-call-apply-bind
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var receivers=[undefined,null,0,false,'receiver',Symbol('receiver'),{}];
for(var i=0;i<receivers.length;i++){var o={x:1};assert.sameValue(m.call(receivers[i],o),isQuery?false:o);}
var a={},b={};assert.sameValue(m.apply(null,[a]),isQuery?false:a);assert.sameValue(m.bind(7,b)(),isQuery?false:b);
// CASE: isSealed-ignored-receivers-call-apply-bind
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isSealed",m=Object[name],isQuery=true,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var receivers=[undefined,null,0,false,'receiver',Symbol('receiver'),{}];
for(var i=0;i<receivers.length;i++){var o={x:1};assert.sameValue(m.call(receivers[i],o),isQuery?false:o);}
var a={},b={};assert.sameValue(m.apply(null,[a]),isQuery?false:a);assert.sameValue(m.bind(7,b)(),isQuery?false:b);
// CASE: isFrozen-ignored-receivers-call-apply-bind
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isFrozen",m=Object[name],isQuery=true,freezing=false,frozenQuery=true,strictMode=(function(){return this;})()===undefined;
var receivers=[undefined,null,0,false,'receiver',Symbol('receiver'),{}];
for(var i=0;i<receivers.length;i++){var o={x:1};assert.sameValue(m.call(receivers[i],o),isQuery?false:o);}
var a={},b={};assert.sameValue(m.apply(null,[a]),isQuery?false:a);assert.sameValue(m.bind(7,b)(),isQuery?false:b);
// CASE: seal-method-lookup-and-argument-evaluation-order
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var log='',o={x:1},holder={get method(){log+='M';return m;}},extra={get valueOf(){throw 'unused';},get toString(){throw 'unused';}};
function first(){log+='A';return o;}function second(){log+='B';o.x=2;return extra;}
assert.sameValue(holder.method(first(),second()),isQuery?false:o);assert.sameValue(log,'MAB');assert.sameValue(o.x,2);
assert.sameValue(Object.isExtensible(o),isQuery);
// CASE: freeze-method-lookup-and-argument-evaluation-order
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var log='',o={x:1},holder={get method(){log+='M';return m;}},extra={get valueOf(){throw 'unused';},get toString(){throw 'unused';}};
function first(){log+='A';return o;}function second(){log+='B';o.x=2;return extra;}
assert.sameValue(holder.method(first(),second()),isQuery?false:o);assert.sameValue(log,'MAB');assert.sameValue(o.x,2);
assert.sameValue(Object.isExtensible(o),isQuery);
// CASE: isSealed-method-lookup-and-argument-evaluation-order
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isSealed",m=Object[name],isQuery=true,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var log='',o={x:1},holder={get method(){log+='M';return m;}},extra={get valueOf(){throw 'unused';},get toString(){throw 'unused';}};
function first(){log+='A';return o;}function second(){log+='B';o.x=2;return extra;}
assert.sameValue(holder.method(first(),second()),isQuery?false:o);assert.sameValue(log,'MAB');assert.sameValue(o.x,2);
assert.sameValue(Object.isExtensible(o),isQuery);
// CASE: isFrozen-method-lookup-and-argument-evaluation-order
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isFrozen",m=Object[name],isQuery=true,freezing=false,frozenQuery=true,strictMode=(function(){return this;})()===undefined;
var log='',o={x:1},holder={get method(){log+='M';return m;}},extra={get valueOf(){throw 'unused';},get toString(){throw 'unused';}};
function first(){log+='A';return o;}function second(){log+='B';o.x=2;return extra;}
assert.sameValue(holder.method(first(),second()),isQuery?false:o);assert.sameValue(log,'MAB');assert.sameValue(o.x,2);
assert.sameValue(Object.isExtensible(o),isQuery);
// CASE: seal-abrupt-argument-prevents-operation
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var reason={},caught,log='',o={x:1};function first(){log+='A';return o;}function extra(){log+='B';throw reason;}
try{m(first(),extra());}catch(e){caught=e;}
assert.sameValue(caught,reason);assert.sameValue(log,'AB');assert.sameValue(Object.isExtensible(o),true);
assert.sameValue(Object.getOwnPropertyDescriptor(o,'x').configurable,true);
// CASE: freeze-abrupt-argument-prevents-operation
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var reason={},caught,log='',o={x:1};function first(){log+='A';return o;}function extra(){log+='B';throw reason;}
try{m(first(),extra());}catch(e){caught=e;}
assert.sameValue(caught,reason);assert.sameValue(log,'AB');assert.sameValue(Object.isExtensible(o),true);
assert.sameValue(Object.getOwnPropertyDescriptor(o,'x').configurable,true);
// CASE: isSealed-abrupt-argument-prevents-operation
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isSealed",m=Object[name],isQuery=true,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var reason={},caught,log='',o={x:1};function first(){log+='A';return o;}function extra(){log+='B';throw reason;}
try{m(first(),extra());}catch(e){caught=e;}
assert.sameValue(caught,reason);assert.sameValue(log,'AB');assert.sameValue(Object.isExtensible(o),true);
assert.sameValue(Object.getOwnPropertyDescriptor(o,'x').configurable,true);
// CASE: isFrozen-abrupt-argument-prevents-operation
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isFrozen",m=Object[name],isQuery=true,freezing=false,frozenQuery=true,strictMode=(function(){return this;})()===undefined;
var reason={},caught,log='',o={x:1};function first(){log+='A';return o;}function extra(){log+='B';throw reason;}
try{m(first(),extra());}catch(e){caught=e;}
assert.sameValue(caught,reason);assert.sameValue(log,'AB');assert.sameValue(Object.isExtensible(o),true);
assert.sameValue(Object.getOwnPropertyDescriptor(o,'x').configurable,true);
// CASE: seal-internal-descriptors-never-call-author-hooks
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var reads=0,o={};function poison(){reads++;throw 'unexpected author call';}
Object.defineProperty(o,'valueOf',{get:poison,configurable:true});Object.defineProperty(o,'toString',{get:poison,configurable:true});
Object.defineProperty(o,'constructor',{get:poison,configurable:true});Object.defineProperty(o,'length',{get:poison,configurable:true});
Object.defineProperty(o,Symbol.toPrimitive,{get:poison,configurable:true});
assert.sameValue(m(o),isQuery?false:o);assert.sameValue(reads,0);
if(isQuery){Object.preventExtensions(o);assert.sameValue(m(o),false);assert.sameValue(reads,0);}
// CASE: freeze-internal-descriptors-never-call-author-hooks
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var reads=0,o={};function poison(){reads++;throw 'unexpected author call';}
Object.defineProperty(o,'valueOf',{get:poison,configurable:true});Object.defineProperty(o,'toString',{get:poison,configurable:true});
Object.defineProperty(o,'constructor',{get:poison,configurable:true});Object.defineProperty(o,'length',{get:poison,configurable:true});
Object.defineProperty(o,Symbol.toPrimitive,{get:poison,configurable:true});
assert.sameValue(m(o),isQuery?false:o);assert.sameValue(reads,0);
if(isQuery){Object.preventExtensions(o);assert.sameValue(m(o),false);assert.sameValue(reads,0);}
// CASE: isSealed-internal-descriptors-never-call-author-hooks
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isSealed",m=Object[name],isQuery=true,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var reads=0,o={};function poison(){reads++;throw 'unexpected author call';}
Object.defineProperty(o,'valueOf',{get:poison,configurable:true});Object.defineProperty(o,'toString',{get:poison,configurable:true});
Object.defineProperty(o,'constructor',{get:poison,configurable:true});Object.defineProperty(o,'length',{get:poison,configurable:true});
Object.defineProperty(o,Symbol.toPrimitive,{get:poison,configurable:true});
assert.sameValue(m(o),isQuery?false:o);assert.sameValue(reads,0);
if(isQuery){Object.preventExtensions(o);assert.sameValue(m(o),false);assert.sameValue(reads,0);}
// CASE: isFrozen-internal-descriptors-never-call-author-hooks
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isFrozen",m=Object[name],isQuery=true,freezing=false,frozenQuery=true,strictMode=(function(){return this;})()===undefined;
var reads=0,o={};function poison(){reads++;throw 'unexpected author call';}
Object.defineProperty(o,'valueOf',{get:poison,configurable:true});Object.defineProperty(o,'toString',{get:poison,configurable:true});
Object.defineProperty(o,'constructor',{get:poison,configurable:true});Object.defineProperty(o,'length',{get:poison,configurable:true});
Object.defineProperty(o,Symbol.toPrimitive,{get:poison,configurable:true});
assert.sameValue(m(o),isQuery?false:o);assert.sameValue(reads,0);
if(isQuery){Object.preventExtensions(o);assert.sameValue(m(o),false);assert.sameValue(reads,0);}
// CASE: seal-saved-alias-after-property-replacement-and-deletion
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
Object[name]=null;var a={};assert.sameValue(m(a),isQuery?false:a);
delete Object[name];var b={};assert.sameValue(m(b),isQuery?false:b);
Object.defineProperty(m,'name',{value:'renamed'});assert.sameValue(m(7),isQuery?true:7);
// CASE: freeze-saved-alias-after-property-replacement-and-deletion
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
Object[name]=null;var a={};assert.sameValue(m(a),isQuery?false:a);
delete Object[name];var b={};assert.sameValue(m(b),isQuery?false:b);
Object.defineProperty(m,'name',{value:'renamed'});assert.sameValue(m(7),isQuery?true:7);
// CASE: isSealed-saved-alias-after-property-replacement-and-deletion
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isSealed",m=Object[name],isQuery=true,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
Object[name]=null;var a={};assert.sameValue(m(a),isQuery?false:a);
delete Object[name];var b={};assert.sameValue(m(b),isQuery?false:b);
Object.defineProperty(m,'name',{value:'renamed'});assert.sameValue(m(7),isQuery?true:7);
// CASE: isFrozen-saved-alias-after-property-replacement-and-deletion
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isFrozen",m=Object[name],isQuery=true,freezing=false,frozenQuery=true,strictMode=(function(){return this;})()===undefined;
Object[name]=null;var a={};assert.sameValue(m(a),isQuery?false:a);
delete Object[name];var b={};assert.sameValue(m(b),isQuery?false:b);
Object.defineProperty(m,'name',{value:'renamed'});assert.sameValue(m(7),isQuery?true:7);
// CASE: seal-primitive-input-does-not-read-wrapper-hooks
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var calls=0;function poison(){calls++;throw 'coercion';}
Object.defineProperty(Number.prototype,'valueOf',{get:poison,configurable:true});
Object.defineProperty(String.prototype,'toString',{get:poison,configurable:true});
Object.defineProperty(Boolean.prototype,Symbol.toPrimitive,{get:poison,configurable:true});
assert.sameValue(m(-0),isQuery?true:-0);assert.sameValue(m('x'),isQuery?true:'x');assert.sameValue(m(false),isQuery?true:false);assert.sameValue(calls,0);
// CASE: freeze-primitive-input-does-not-read-wrapper-hooks
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var calls=0;function poison(){calls++;throw 'coercion';}
Object.defineProperty(Number.prototype,'valueOf',{get:poison,configurable:true});
Object.defineProperty(String.prototype,'toString',{get:poison,configurable:true});
Object.defineProperty(Boolean.prototype,Symbol.toPrimitive,{get:poison,configurable:true});
assert.sameValue(m(-0),isQuery?true:-0);assert.sameValue(m('x'),isQuery?true:'x');assert.sameValue(m(false),isQuery?true:false);assert.sameValue(calls,0);
// CASE: isSealed-primitive-input-does-not-read-wrapper-hooks
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isSealed",m=Object[name],isQuery=true,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var calls=0;function poison(){calls++;throw 'coercion';}
Object.defineProperty(Number.prototype,'valueOf',{get:poison,configurable:true});
Object.defineProperty(String.prototype,'toString',{get:poison,configurable:true});
Object.defineProperty(Boolean.prototype,Symbol.toPrimitive,{get:poison,configurable:true});
assert.sameValue(m(-0),isQuery?true:-0);assert.sameValue(m('x'),isQuery?true:'x');assert.sameValue(m(false),isQuery?true:false);assert.sameValue(calls,0);
// CASE: isFrozen-primitive-input-does-not-read-wrapper-hooks
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isFrozen",m=Object[name],isQuery=true,freezing=false,frozenQuery=true,strictMode=(function(){return this;})()===undefined;
var calls=0;function poison(){calls++;throw 'coercion';}
Object.defineProperty(Number.prototype,'valueOf',{get:poison,configurable:true});
Object.defineProperty(String.prototype,'toString',{get:poison,configurable:true});
Object.defineProperty(Boolean.prototype,Symbol.toPrimitive,{get:poison,configurable:true});
assert.sameValue(m(-0),isQuery?true:-0);assert.sameValue(m('x'),isQuery?true:'x');assert.sameValue(m(false),isQuery?true:false);assert.sameValue(calls,0);
// CASE: seal-data-descriptor-flags-and-values
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var o={visible:1},child={};Object.defineProperty(o,'hidden',{value:child,writable:false,enumerable:false,configurable:true});
Object.defineProperty(o,'fixed',{value:NaN,writable:true,enumerable:false,configurable:false});
assert.sameValue(m(o),o);assert.sameValue(Object.isExtensible(o),false);
var a=Object.getOwnPropertyDescriptor(o,'visible'),b=Object.getOwnPropertyDescriptor(o,'hidden'),c=Object.getOwnPropertyDescriptor(o,'fixed');
assert.sameValue(a.value,1);assert.sameValue(a.writable,!freezing);assert.sameValue(a.enumerable,true);assert.sameValue(a.configurable,false);
assert.sameValue(b.value,child);assert.sameValue(b.writable,false);assert.sameValue(b.enumerable,false);assert.sameValue(b.configurable,false);
assert.sameValue(c.value,NaN);assert.sameValue(c.writable,!freezing);assert.sameValue(c.enumerable,false);assert.sameValue(c.configurable,false);
assert.sameValue(Object.isSealed(o),true);assert.sameValue(Object.isFrozen(o),freezing);
// CASE: freeze-data-descriptor-flags-and-values
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var o={visible:1},child={};Object.defineProperty(o,'hidden',{value:child,writable:false,enumerable:false,configurable:true});
Object.defineProperty(o,'fixed',{value:NaN,writable:true,enumerable:false,configurable:false});
assert.sameValue(m(o),o);assert.sameValue(Object.isExtensible(o),false);
var a=Object.getOwnPropertyDescriptor(o,'visible'),b=Object.getOwnPropertyDescriptor(o,'hidden'),c=Object.getOwnPropertyDescriptor(o,'fixed');
assert.sameValue(a.value,1);assert.sameValue(a.writable,!freezing);assert.sameValue(a.enumerable,true);assert.sameValue(a.configurable,false);
assert.sameValue(b.value,child);assert.sameValue(b.writable,false);assert.sameValue(b.enumerable,false);assert.sameValue(b.configurable,false);
assert.sameValue(c.value,NaN);assert.sameValue(c.writable,!freezing);assert.sameValue(c.enumerable,false);assert.sameValue(c.configurable,false);
assert.sameValue(Object.isSealed(o),true);assert.sameValue(Object.isFrozen(o),freezing);
// CASE: seal-idempotent-return-identity-and-existing-restrictions
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var o={x:1};Object.defineProperty(o,'r',{value:-0,writable:false,configurable:false});Object.preventExtensions(o);
assert.sameValue(m(o),o);assert.sameValue(m(o),o);assert.sameValue(m(o),o);
assert.sameValue(Object.getOwnPropertyDescriptor(o,'r').value,-0);assert.sameValue(Object.getOwnPropertyDescriptor(o,'r').writable,false);
assert.sameValue(Object.isSealed(o),true);assert.sameValue(Object.isFrozen(o),freezing);
// CASE: freeze-idempotent-return-identity-and-existing-restrictions
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var o={x:1};Object.defineProperty(o,'r',{value:-0,writable:false,configurable:false});Object.preventExtensions(o);
assert.sameValue(m(o),o);assert.sameValue(m(o),o);assert.sameValue(m(o),o);
assert.sameValue(Object.getOwnPropertyDescriptor(o,'r').value,-0);assert.sameValue(Object.getOwnPropertyDescriptor(o,'r').writable,false);
assert.sameValue(Object.isSealed(o),true);assert.sameValue(Object.isFrozen(o),freezing);
// CASE: seal-writes-to-existing-data-follow-caller-mode
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var o={x:1};m(o);var caught=false;try{o.x=2;}catch(e){assert.sameValue(e instanceof TypeError,true);caught=true;}
assert.sameValue(caught,freezing&&strictMode);assert.sameValue(o.x,freezing?1:2);
function strictWrite(){'use strict';o.x=3;}
if(freezing){assert.throws(TypeError,strictWrite);}else{strictWrite();assert.sameValue(o.x,3);}
// CASE: freeze-writes-to-existing-data-follow-caller-mode
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var o={x:1};m(o);var caught=false;try{o.x=2;}catch(e){assert.sameValue(e instanceof TypeError,true);caught=true;}
assert.sameValue(caught,freezing&&strictMode);assert.sameValue(o.x,freezing?1:2);
function strictWrite(){'use strict';o.x=3;}
if(freezing){assert.throws(TypeError,strictWrite);}else{strictWrite();assert.sameValue(o.x,3);}
// CASE: seal-new-properties-refused-without-changing-value
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var o={},s=Symbol('new');m(o);var caught=false;try{o.x=1;}catch(e){assert.sameValue(e instanceof TypeError,true);caught=true;}
assert.sameValue(caught,strictMode);assert.sameValue(Object.getOwnPropertyDescriptor(o,'x'),undefined);
assert.throws(TypeError,function(){'use strict';o[s]=2;});assert.sameValue(Object.getOwnPropertyDescriptor(o,s),undefined);
assert.throws(TypeError,function(){Object.defineProperty(o,'y',{value:3});});assert.sameValue(Object.isExtensible(o),false);
// CASE: freeze-new-properties-refused-without-changing-value
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var o={},s=Symbol('new');m(o);var caught=false;try{o.x=1;}catch(e){assert.sameValue(e instanceof TypeError,true);caught=true;}
assert.sameValue(caught,strictMode);assert.sameValue(Object.getOwnPropertyDescriptor(o,'x'),undefined);
assert.throws(TypeError,function(){'use strict';o[s]=2;});assert.sameValue(Object.getOwnPropertyDescriptor(o,s),undefined);
assert.throws(TypeError,function(){Object.defineProperty(o,'y',{value:3});});assert.sameValue(Object.isExtensible(o),false);
// CASE: seal-deletion-follows-caller-mode-without-getter
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var calls=0,o={};Object.defineProperty(o,'x',{get:function(){calls++;throw 'getter';},configurable:true});m(o);
var caught=false,result;try{result=delete o.x;}catch(e){assert.sameValue(e instanceof TypeError,true);caught=true;}
assert.sameValue(caught,strictMode);if(!strictMode)assert.sameValue(result,false);
assert.throws(TypeError,function(){'use strict';delete o.x;});assert.sameValue(calls,0);
assert.sameValue(delete o.absent,true);assert.notSameValue(Object.getOwnPropertyDescriptor(o,'x'),undefined);
// CASE: freeze-deletion-follows-caller-mode-without-getter
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var calls=0,o={};Object.defineProperty(o,'x',{get:function(){calls++;throw 'getter';},configurable:true});m(o);
var caught=false,result;try{result=delete o.x;}catch(e){assert.sameValue(e instanceof TypeError,true);caught=true;}
assert.sameValue(caught,strictMode);if(!strictMode)assert.sameValue(result,false);
assert.throws(TypeError,function(){'use strict';delete o.x;});assert.sameValue(calls,0);
assert.sameValue(delete o.absent,true);assert.notSameValue(Object.getOwnPropertyDescriptor(o,'x'),undefined);
// CASE: seal-accessor-identity-and-setter-survive
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var calls=0,stored=0,o={},getter=function(){calls++;return stored;},setter=function(v){assert.sameValue(this,o);stored=v;};
Object.defineProperty(o,'x',{get:getter,set:setter,enumerable:true,configurable:true});
assert.sameValue(m(o),o);assert.sameValue(calls,0);assert.sameValue(Object.isFrozen(o),true);assert.sameValue(calls,0);
var d=Object.getOwnPropertyDescriptor(o,'x');assert.sameValue(d.get,getter);assert.sameValue(d.set,setter);assert.sameValue(d.configurable,false);assert.sameValue(d.enumerable,true);
assert.sameValue(Object.getOwnPropertyDescriptor(d,'value'),undefined);assert.sameValue(Object.getOwnPropertyDescriptor(d,'writable'),undefined);
o.x=9;assert.sameValue(stored,9);assert.sameValue(o.x,9);assert.sameValue(calls,1);
Object.defineProperty(o,'x',{get:getter,set:setter});assert.throws(TypeError,function(){Object.defineProperty(o,'x',{get:function(){}});});
// CASE: freeze-accessor-identity-and-setter-survive
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var calls=0,stored=0,o={},getter=function(){calls++;return stored;},setter=function(v){assert.sameValue(this,o);stored=v;};
Object.defineProperty(o,'x',{get:getter,set:setter,enumerable:true,configurable:true});
assert.sameValue(m(o),o);assert.sameValue(calls,0);assert.sameValue(Object.isFrozen(o),true);assert.sameValue(calls,0);
var d=Object.getOwnPropertyDescriptor(o,'x');assert.sameValue(d.get,getter);assert.sameValue(d.set,setter);assert.sameValue(d.configurable,false);assert.sameValue(d.enumerable,true);
assert.sameValue(Object.getOwnPropertyDescriptor(d,'value'),undefined);assert.sameValue(Object.getOwnPropertyDescriptor(d,'writable'),undefined);
o.x=9;assert.sameValue(stored,9);assert.sameValue(o.x,9);assert.sameValue(calls,1);
Object.defineProperty(o,'x',{get:getter,set:setter});assert.throws(TypeError,function(){Object.defineProperty(o,'x',{get:function(){}});});
// CASE: seal-undefined-accessors-remain-accessors
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var o={};Object.defineProperty(o,'x',{get:undefined,set:undefined,enumerable:false,configurable:true});m(o);
var d=Object.getOwnPropertyDescriptor(o,'x');assert.sameValue(d.get,undefined);assert.sameValue(d.set,undefined);assert.sameValue(d.enumerable,false);assert.sameValue(d.configurable,false);
assert.sameValue(Object.getOwnPropertyDescriptor(d,'writable'),undefined);assert.sameValue(Object.isFrozen(o),true);
assert.throws(TypeError,function(){'use strict';o.x=1;});assert.sameValue(o.x,undefined);
// CASE: freeze-undefined-accessors-remain-accessors
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var o={};Object.defineProperty(o,'x',{get:undefined,set:undefined,enumerable:false,configurable:true});m(o);
var d=Object.getOwnPropertyDescriptor(o,'x');assert.sameValue(d.get,undefined);assert.sameValue(d.set,undefined);assert.sameValue(d.enumerable,false);assert.sameValue(d.configurable,false);
assert.sameValue(Object.getOwnPropertyDescriptor(d,'writable'),undefined);assert.sameValue(Object.isFrozen(o),true);
assert.throws(TypeError,function(){'use strict';o.x=1;});assert.sameValue(o.x,undefined);
// CASE: seal-descriptor-redefinition-compatible-and-rejected
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var o={x:1};m(o);Object.defineProperty(o,'x',{value:1});
assert.throws(TypeError,function(){Object.defineProperty(o,'x',{configurable:true});});
assert.throws(TypeError,function(){Object.defineProperty(o,'x',{enumerable:false});});
assert.throws(TypeError,function(){Object.defineProperty(o,'x',{get:function(){return 1;}});});
if(freezing){assert.throws(TypeError,function(){Object.defineProperty(o,'x',{value:2});});assert.throws(TypeError,function(){Object.defineProperty(o,'x',{writable:true});});}
else{Object.defineProperty(o,'x',{value:2,writable:false});assert.sameValue(o.x,2);assert.sameValue(Object.isFrozen(o),true);}
// CASE: freeze-descriptor-redefinition-compatible-and-rejected
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var o={x:1};m(o);Object.defineProperty(o,'x',{value:1});
assert.throws(TypeError,function(){Object.defineProperty(o,'x',{configurable:true});});
assert.throws(TypeError,function(){Object.defineProperty(o,'x',{enumerable:false});});
assert.throws(TypeError,function(){Object.defineProperty(o,'x',{get:function(){return 1;}});});
if(freezing){assert.throws(TypeError,function(){Object.defineProperty(o,'x',{value:2});});assert.throws(TypeError,function(){Object.defineProperty(o,'x',{writable:true});});}
else{Object.defineProperty(o,'x',{value:2,writable:false});assert.sameValue(o.x,2);assert.sameValue(Object.isFrozen(o),true);}
// CASE: seal-symbols-including-nonenumerable-and-same-description
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var a=Symbol('same'),b=Symbol('same'),o={};o[a]=1;Object.defineProperty(o,b,{value:2,writable:true,enumerable:false,configurable:true});m(o);
var da=Object.getOwnPropertyDescriptor(o,a),db=Object.getOwnPropertyDescriptor(o,b);
assert.sameValue(da.value,1);assert.sameValue(db.value,2);assert.sameValue(da.enumerable,true);assert.sameValue(db.enumerable,false);
assert.sameValue(da.configurable,false);assert.sameValue(db.configurable,false);assert.sameValue(da.writable,!freezing);assert.sameValue(db.writable,!freezing);
assert.sameValue(Object.getOwnPropertySymbols(o).length,2);assert.sameValue(Object.isSealed(o),true);assert.sameValue(Object.isFrozen(o),freezing);
// CASE: freeze-symbols-including-nonenumerable-and-same-description
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var a=Symbol('same'),b=Symbol('same'),o={};o[a]=1;Object.defineProperty(o,b,{value:2,writable:true,enumerable:false,configurable:true});m(o);
var da=Object.getOwnPropertyDescriptor(o,a),db=Object.getOwnPropertyDescriptor(o,b);
assert.sameValue(da.value,1);assert.sameValue(db.value,2);assert.sameValue(da.enumerable,true);assert.sameValue(db.enumerable,false);
assert.sameValue(da.configurable,false);assert.sameValue(db.configurable,false);assert.sameValue(da.writable,!freezing);assert.sameValue(db.writable,!freezing);
assert.sameValue(Object.getOwnPropertySymbols(o).length,2);assert.sameValue(Object.isSealed(o),true);assert.sameValue(Object.isFrozen(o),freezing);
// CASE: seal-symbol-accessor-with-no-author-invocation
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var k=Symbol('key'),o={},reads=0,stored=0,g=function(){reads++;return stored;},s=function(v){stored=v;};
Object.defineProperty(o,k,{get:g,set:s,configurable:true});m(o);assert.sameValue(reads,0);assert.sameValue(Object.isFrozen(o),true);
var d=Object.getOwnPropertyDescriptor(o,k);assert.sameValue(d.get,g);assert.sameValue(d.set,s);assert.sameValue(d.configurable,false);o[k]=7;assert.sameValue(stored,7);assert.sameValue(reads,0);
// CASE: freeze-symbol-accessor-with-no-author-invocation
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var k=Symbol('key'),o={},reads=0,stored=0,g=function(){reads++;return stored;},s=function(v){stored=v;};
Object.defineProperty(o,k,{get:g,set:s,configurable:true});m(o);assert.sameValue(reads,0);assert.sameValue(Object.isFrozen(o),true);
var d=Object.getOwnPropertyDescriptor(o,k);assert.sameValue(d.get,g);assert.sameValue(d.set,s);assert.sameValue(d.configurable,false);o[k]=7;assert.sameValue(stored,7);assert.sameValue(reads,0);
// CASE: seal-own-only-shallow-and-inherited-setter
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var child={x:1},calls=0,p={inherited:2,set accepted(v){assert.sameValue(this,o);calls+=v;}},o=Object.create(p);o.child=child;m(o);
child.x=3;child.newValue=4;p.inherited=5;o.accepted=7;
assert.sameValue(o.child,child);assert.sameValue(child.x,3);assert.sameValue(child.newValue,4);assert.sameValue(o.inherited,5);assert.sameValue(calls,7);
assert.sameValue(Object.isExtensible(child),true);assert.sameValue(Object.isExtensible(p),true);assert.sameValue(Object.getOwnPropertyDescriptor(o,'accepted'),undefined);
// CASE: freeze-own-only-shallow-and-inherited-setter
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var child={x:1},calls=0,p={inherited:2,set accepted(v){assert.sameValue(this,o);calls+=v;}},o=Object.create(p);o.child=child;m(o);
child.x=3;child.newValue=4;p.inherited=5;o.accepted=7;
assert.sameValue(o.child,child);assert.sameValue(child.x,3);assert.sameValue(child.newValue,4);assert.sameValue(o.inherited,5);assert.sameValue(calls,7);
assert.sameValue(Object.isExtensible(child),true);assert.sameValue(Object.isExtensible(p),true);assert.sameValue(Object.getOwnPropertyDescriptor(o,'accepted'),undefined);
// CASE: seal-null-prototype-and-exact-utf16-keys
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var o=Object.create(null);o['\ud800']=1;o['\udfff']=2;o['\u0000']=3;o['__proto__']=4;m(o);
assert.sameValue(Object.getPrototypeOf(o),null);assert.sameValue(Object.getOwnPropertyNames(o).length,4);
assert.sameValue(Object.getOwnPropertyDescriptor(o,'\ud800').value,1);assert.sameValue(Object.getOwnPropertyDescriptor(o,'\udfff').value,2);
assert.sameValue(Object.getOwnPropertyDescriptor(o,'\u0000').value,3);assert.sameValue(Object.getOwnPropertyDescriptor(o,'__proto__').value,4);
assert.sameValue(Object.isSealed(o),true);assert.sameValue(Object.isFrozen(o),freezing);
// CASE: freeze-null-prototype-and-exact-utf16-keys
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var o=Object.create(null);o['\ud800']=1;o['\udfff']=2;o['\u0000']=3;o['__proto__']=4;m(o);
assert.sameValue(Object.getPrototypeOf(o),null);assert.sameValue(Object.getOwnPropertyNames(o).length,4);
assert.sameValue(Object.getOwnPropertyDescriptor(o,'\ud800').value,1);assert.sameValue(Object.getOwnPropertyDescriptor(o,'\udfff').value,2);
assert.sameValue(Object.getOwnPropertyDescriptor(o,'\u0000').value,3);assert.sameValue(Object.getOwnPropertyDescriptor(o,'__proto__').value,4);
assert.sameValue(Object.isSealed(o),true);assert.sameValue(Object.isFrozen(o),freezing);
// CASE: seal-prototype-lock-retains-same-prototype
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var p={},o=Object.create(p);m(o);assert.sameValue(Object.setPrototypeOf(o,p),o);
assert.throws(TypeError,function(){Object.setPrototypeOf(o,{});});assert.throws(TypeError,function(){Object.setPrototypeOf(o,null);});
assert.sameValue(Object.getPrototypeOf(o),p);p.x=7;assert.sameValue(o.x,7);
// CASE: freeze-prototype-lock-retains-same-prototype
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var p={},o=Object.create(p);m(o);assert.sameValue(Object.setPrototypeOf(o,p),o);
assert.throws(TypeError,function(){Object.setPrototypeOf(o,{});});assert.throws(TypeError,function(){Object.setPrototypeOf(o,null);});
assert.sameValue(Object.getPrototypeOf(o),p);p.x=7;assert.sameValue(o.x,7);
// CASE: seal-ordinary-self-cycle-is-shallow
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var o={};o.self=o;m(o);assert.sameValue(o.self,o);assert.sameValue(Object.isSealed(o),true);assert.sameValue(Object.isFrozen(o),freezing);
// CASE: freeze-ordinary-self-cycle-is-shallow
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var o={};o.self=o;m(o);assert.sameValue(o.self,o);assert.sameValue(Object.isSealed(o),true);assert.sameValue(Object.isFrozen(o),freezing);
// CASE: seal-dense-array-descriptors-and-length
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var a=[1,undefined,3];a.extra=4;assert.sameValue(m(a),a);assert.sameValue(a.length,3);assert.sameValue(a[0],1);assert.sameValue(a[1],undefined);
for(var i=0;i<3;i++){var d=Object.getOwnPropertyDescriptor(a,''+i);assert.sameValue(d.configurable,false);assert.sameValue(d.writable,!freezing);assert.sameValue(d.enumerable,true);}
var l=Object.getOwnPropertyDescriptor(a,'length');assert.sameValue(l.value,3);assert.sameValue(l.enumerable,false);assert.sameValue(l.configurable,false);assert.sameValue(l.writable,!freezing);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'extra').writable,!freezing);assert.sameValue(Object.isSealed(a),true);assert.sameValue(Object.isFrozen(a),freezing);
// CASE: freeze-dense-array-descriptors-and-length
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var a=[1,undefined,3];a.extra=4;assert.sameValue(m(a),a);assert.sameValue(a.length,3);assert.sameValue(a[0],1);assert.sameValue(a[1],undefined);
for(var i=0;i<3;i++){var d=Object.getOwnPropertyDescriptor(a,''+i);assert.sameValue(d.configurable,false);assert.sameValue(d.writable,!freezing);assert.sameValue(d.enumerable,true);}
var l=Object.getOwnPropertyDescriptor(a,'length');assert.sameValue(l.value,3);assert.sameValue(l.enumerable,false);assert.sameValue(l.configurable,false);assert.sameValue(l.writable,!freezing);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'extra').writable,!freezing);assert.sameValue(Object.isSealed(a),true);assert.sameValue(Object.isFrozen(a),freezing);
// CASE: seal-sparse-max-length-array-does-not-create-holes
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var a=new Array(4294967295),s=Symbol('tail');a[0]=1;a[4294967294]=2;a['4294967295']=3;a['01']=4;a[s]=5;m(a);
assert.sameValue(a.length,4294967295);assert.sameValue(Object.getOwnPropertyDescriptor(a,'1'),undefined);assert.sameValue(Object.getOwnPropertyDescriptor(a,'4294967293'),undefined);
assert.sameValue(Object.getOwnPropertyNames(a).length,5);assert.sameValue(Object.getOwnPropertySymbols(a).length,1);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'4294967294').value,2);assert.sameValue(Object.getOwnPropertyDescriptor(a,'4294967294').writable,!freezing);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'4294967295').value,3);assert.sameValue(Object.getOwnPropertyDescriptor(a,'01').configurable,false);
assert.sameValue(Object.getOwnPropertyDescriptor(a,s).configurable,false);assert.sameValue(Object.isSealed(a),true);assert.sameValue(Object.isFrozen(a),freezing);
// CASE: freeze-sparse-max-length-array-does-not-create-holes
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var a=new Array(4294967295),s=Symbol('tail');a[0]=1;a[4294967294]=2;a['4294967295']=3;a['01']=4;a[s]=5;m(a);
assert.sameValue(a.length,4294967295);assert.sameValue(Object.getOwnPropertyDescriptor(a,'1'),undefined);assert.sameValue(Object.getOwnPropertyDescriptor(a,'4294967293'),undefined);
assert.sameValue(Object.getOwnPropertyNames(a).length,5);assert.sameValue(Object.getOwnPropertySymbols(a).length,1);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'4294967294').value,2);assert.sameValue(Object.getOwnPropertyDescriptor(a,'4294967294').writable,!freezing);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'4294967295').value,3);assert.sameValue(Object.getOwnPropertyDescriptor(a,'01').configurable,false);
assert.sameValue(Object.getOwnPropertyDescriptor(a,s).configurable,false);assert.sameValue(Object.isSealed(a),true);assert.sameValue(Object.isFrozen(a),freezing);
// CASE: seal-array-holes-prototypes-and-failed-fill
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var p={1:8},a=[1,,3];Object.setPrototypeOf(a,p);m(a);assert.sameValue(a[1],8);assert.sameValue(Object.getOwnPropertyDescriptor(a,'1'),undefined);
assert.throws(TypeError,function(){'use strict';a[1]=9;});assert.sameValue(a[1],8);assert.sameValue(Object.getOwnPropertyDescriptor(a,'1'),undefined);
p[1]=10;assert.sameValue(a[1],10);assert.sameValue(Object.getOwnPropertyDescriptor(p,'1').configurable,true);
// CASE: freeze-array-holes-prototypes-and-failed-fill
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var p={1:8},a=[1,,3];Object.setPrototypeOf(a,p);m(a);assert.sameValue(a[1],8);assert.sameValue(Object.getOwnPropertyDescriptor(a,'1'),undefined);
assert.throws(TypeError,function(){'use strict';a[1]=9;});assert.sameValue(a[1],8);assert.sameValue(Object.getOwnPropertyDescriptor(a,'1'),undefined);
p[1]=10;assert.sameValue(a[1],10);assert.sameValue(Object.getOwnPropertyDescriptor(p,'1').configurable,true);
// CASE: seal-indexed-array-accessor-survives
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var a=[1],reads=0,stored=2,g=function(){reads++;return stored;},s=function(v){assert.sameValue(this,a);stored=v;};
Object.defineProperty(a,'0',{get:g,set:s,enumerable:false,configurable:true});m(a);assert.sameValue(reads,0);
var d=Object.getOwnPropertyDescriptor(a,'0');assert.sameValue(d.get,g);assert.sameValue(d.set,s);assert.sameValue(d.configurable,false);assert.sameValue(d.enumerable,false);assert.sameValue(Object.getOwnPropertyDescriptor(d,'writable'),undefined);
a[0]=7;assert.sameValue(stored,7);assert.sameValue(reads,0);assert.sameValue(a[0],7);assert.sameValue(reads,1);
assert.sameValue(Object.isFrozen(a),freezing);
// CASE: freeze-indexed-array-accessor-survives
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var a=[1],reads=0,stored=2,g=function(){reads++;return stored;},s=function(v){assert.sameValue(this,a);stored=v;};
Object.defineProperty(a,'0',{get:g,set:s,enumerable:false,configurable:true});m(a);assert.sameValue(reads,0);
var d=Object.getOwnPropertyDescriptor(a,'0');assert.sameValue(d.get,g);assert.sameValue(d.set,s);assert.sameValue(d.configurable,false);assert.sameValue(d.enumerable,false);assert.sameValue(Object.getOwnPropertyDescriptor(d,'writable'),undefined);
a[0]=7;assert.sameValue(stored,7);assert.sameValue(reads,0);assert.sameValue(a[0],7);assert.sameValue(reads,1);
assert.sameValue(Object.isFrozen(a),freezing);
// CASE: seal-readonly-array-length-remains-readonly
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var a=[1];Object.defineProperty(a,'length',{writable:false});m(a);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'length').writable,false);assert.sameValue(Object.isFrozen(a),freezing);
assert.throws(TypeError,function(){'use strict';a.length=1;});assert.sameValue(a.length,1);
// CASE: freeze-readonly-array-length-remains-readonly
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var a=[1];Object.defineProperty(a,'length',{writable:false});m(a);
assert.sameValue(Object.getOwnPropertyDescriptor(a,'length').writable,false);assert.sameValue(Object.isFrozen(a),freezing);
assert.throws(TypeError,function(){'use strict';a.length=1;});assert.sameValue(a.length,1);
// CASE: seal-string-wrapper-intrinsic-properties-and-extra
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var s=new String('A\ud800');s.extra=7;m(s);assert.sameValue(s.valueOf(),'A\ud800');assert.sameValue(s.length,2);
for(var i=0;i<2;i++){var d=Object.getOwnPropertyDescriptor(s,''+i);assert.sameValue(d.value,i===0?'A':'\ud800');assert.sameValue(d.writable,false);assert.sameValue(d.enumerable,true);assert.sameValue(d.configurable,false);}
var l=Object.getOwnPropertyDescriptor(s,'length');assert.sameValue(l.value,2);assert.sameValue(l.writable,false);assert.sameValue(l.enumerable,false);assert.sameValue(l.configurable,false);
assert.sameValue(Object.getOwnPropertyDescriptor(s,'extra').writable,!freezing);assert.sameValue(Object.isSealed(s),true);assert.sameValue(Object.isFrozen(s),freezing);
// CASE: freeze-string-wrapper-intrinsic-properties-and-extra
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var s=new String('A\ud800');s.extra=7;m(s);assert.sameValue(s.valueOf(),'A\ud800');assert.sameValue(s.length,2);
for(var i=0;i<2;i++){var d=Object.getOwnPropertyDescriptor(s,''+i);assert.sameValue(d.value,i===0?'A':'\ud800');assert.sameValue(d.writable,false);assert.sameValue(d.enumerable,true);assert.sameValue(d.configurable,false);}
var l=Object.getOwnPropertyDescriptor(s,'length');assert.sameValue(l.value,2);assert.sameValue(l.writable,false);assert.sameValue(l.enumerable,false);assert.sameValue(l.configurable,false);
assert.sameValue(Object.getOwnPropertyDescriptor(s,'extra').writable,!freezing);assert.sameValue(Object.isSealed(s),true);assert.sameValue(Object.isFrozen(s),freezing);
// CASE: seal-string-wrapper-with-no-extra-is-frozen-when-sealed
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var values=['','xy','\ud800'];for(var i=0;i<values.length;i++){var o=new String(values[i]);assert.sameValue(m(o),o);assert.sameValue(Object.isFrozen(o),true);assert.sameValue(Object.isSealed(o),true);assert.sameValue(o.valueOf(),values[i]);}
// CASE: freeze-string-wrapper-with-no-extra-is-frozen-when-sealed
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var values=['','xy','\ud800'];for(var i=0;i<values.length;i++){var o=new String(values[i]);assert.sameValue(m(o),o);assert.sameValue(Object.isFrozen(o),true);assert.sameValue(Object.isSealed(o),true);assert.sameValue(o.valueOf(),values[i]);}
// CASE: seal-number-boolean-symbol-wrappers-remain-usable
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var sym=Symbol('wrapped'),values=[new Number(-0),new Boolean(false),Object(sym)];
for(var i=0;i<values.length;i++){var o=values[i];m(o);assert.sameValue(Object.isFrozen(o),true);assert.sameValue(Object.isSealed(o),true);}
assert.sameValue(values[0].valueOf(),-0);assert.sameValue(values[1].valueOf(),false);assert.sameValue(values[2].valueOf(),sym);
// CASE: freeze-number-boolean-symbol-wrappers-remain-usable
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var sym=Symbol('wrapped'),values=[new Number(-0),new Boolean(false),Object(sym)];
for(var i=0;i<values.length;i++){var o=values[i];m(o);assert.sameValue(Object.isFrozen(o),true);assert.sameValue(Object.isSealed(o),true);}
assert.sameValue(values[0].valueOf(),-0);assert.sameValue(values[1].valueOf(),false);assert.sameValue(values[2].valueOf(),sym);
// CASE: seal-ordinary-function-call-and-prototype-remain-live
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
function f(v){return v+1;}f.extra=3;var p=f.prototype;m(f);assert.sameValue(f(4),5);assert.sameValue(f.prototype,p);
assert.sameValue(Object.getOwnPropertyDescriptor(f,'extra').writable,!freezing);assert.sameValue(Object.getOwnPropertyDescriptor(f,'length').configurable,false);
p.member=9;assert.sameValue(f.prototype.member,9);assert.sameValue(Object.isExtensible(p),true);assert.sameValue(Object.isSealed(f),true);assert.sameValue(Object.isFrozen(f),freezing);
// CASE: freeze-ordinary-function-call-and-prototype-remain-live
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
function f(v){return v+1;}f.extra=3;var p=f.prototype;m(f);assert.sameValue(f(4),5);assert.sameValue(f.prototype,p);
assert.sameValue(Object.getOwnPropertyDescriptor(f,'extra').writable,!freezing);assert.sameValue(Object.getOwnPropertyDescriptor(f,'length').configurable,false);
p.member=9;assert.sameValue(f.prototype.member,9);assert.sameValue(Object.isExtensible(p),true);assert.sameValue(Object.isSealed(f),true);assert.sameValue(Object.isFrozen(f),freezing);
// CASE: seal-frozen-native-method-still-calls
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var callable=Number.isFinite;m(callable);assert.sameValue(callable(1),true);assert.sameValue(callable('1'),false);assert.sameValue(Object.isFrozen(callable),true);
// CASE: freeze-frozen-native-method-still-calls
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var callable=Number.isFinite;m(callable);assert.sameValue(callable(1),true);assert.sameValue(callable('1'),false);assert.sameValue(Object.isFrozen(callable),true);
// CASE: isSealed-data-descriptor-truth-table
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isSealed",m=Object[name],isQuery=true,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
for(var ext=0;ext<2;ext++){for(var config=0;config<2;config++){for(var writable=0;writable<2;writable++){for(var enumerable=0;enumerable<2;enumerable++){
var o={};Object.defineProperty(o,'x',{value:1,writable:!!writable,enumerable:!!enumerable,configurable:!!config});if(!ext)Object.preventExtensions(o);
assert.sameValue(m(o),!ext&&!config&&(!frozenQuery||!writable));
var d=Object.getOwnPropertyDescriptor(o,'x');assert.sameValue(d.writable,!!writable);assert.sameValue(d.enumerable,!!enumerable);assert.sameValue(d.configurable,!!config);assert.sameValue(Object.isExtensible(o),!!ext);
}}}}
// CASE: isFrozen-data-descriptor-truth-table
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isFrozen",m=Object[name],isQuery=true,freezing=false,frozenQuery=true,strictMode=(function(){return this;})()===undefined;
for(var ext=0;ext<2;ext++){for(var config=0;config<2;config++){for(var writable=0;writable<2;writable++){for(var enumerable=0;enumerable<2;enumerable++){
var o={};Object.defineProperty(o,'x',{value:1,writable:!!writable,enumerable:!!enumerable,configurable:!!config});if(!ext)Object.preventExtensions(o);
assert.sameValue(m(o),!ext&&!config&&(!frozenQuery||!writable));
var d=Object.getOwnPropertyDescriptor(o,'x');assert.sameValue(d.writable,!!writable);assert.sameValue(d.enumerable,!!enumerable);assert.sameValue(d.configurable,!!config);assert.sameValue(Object.isExtensible(o),!!ext);
}}}}
// CASE: isSealed-accessor-truth-table-without-invocation
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isSealed",m=Object[name],isQuery=true,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var calls=0;function poison(){calls++;throw 'getter';}
for(var ext=0;ext<2;ext++){for(var config=0;config<2;config++){for(var enumerable=0;enumerable<2;enumerable++){
var o={};Object.defineProperty(o,'x',{get:poison,set:poison,enumerable:!!enumerable,configurable:!!config});if(!ext)Object.preventExtensions(o);
assert.sameValue(m(o),!ext&&!config);assert.sameValue(calls,0);
}}}
// CASE: isFrozen-accessor-truth-table-without-invocation
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isFrozen",m=Object[name],isQuery=true,freezing=false,frozenQuery=true,strictMode=(function(){return this;})()===undefined;
var calls=0;function poison(){calls++;throw 'getter';}
for(var ext=0;ext<2;ext++){for(var config=0;config<2;config++){for(var enumerable=0;enumerable<2;enumerable++){
var o={};Object.defineProperty(o,'x',{get:poison,set:poison,enumerable:!!enumerable,configurable:!!config});if(!ext)Object.preventExtensions(o);
assert.sameValue(m(o),!ext&&!config);assert.sameValue(calls,0);
}}}
// CASE: isSealed-empty-and-inherited-properties-do-not-count
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isSealed",m=Object[name],isQuery=true,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var calls=0,p={x:1,get inherited(){calls++;throw 'inherited';}},o=Object.create(p);
assert.sameValue(m(o),false);Object.preventExtensions(o);assert.sameValue(m(o),true);assert.sameValue(calls,0);assert.sameValue(Object.isExtensible(p),true);
assert.sameValue(m(Object.preventExtensions(Object.create(null))),true);
// CASE: isFrozen-empty-and-inherited-properties-do-not-count
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isFrozen",m=Object[name],isQuery=true,freezing=false,frozenQuery=true,strictMode=(function(){return this;})()===undefined;
var calls=0,p={x:1,get inherited(){calls++;throw 'inherited';}},o=Object.create(p);
assert.sameValue(m(o),false);Object.preventExtensions(o);assert.sameValue(m(o),true);assert.sameValue(calls,0);assert.sameValue(Object.isExtensible(p),true);
assert.sameValue(m(Object.preventExtensions(Object.create(null))),true);
// CASE: isSealed-all-properties-not-just-first-or-enumerable
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isSealed",m=Object[name],isQuery=true,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var o={};Object.defineProperty(o,'first',{value:1});Object.defineProperty(o,'hidden',{value:2,writable:true,configurable:true,enumerable:false});Object.preventExtensions(o);
assert.sameValue(m(o),false);Object.defineProperty(o,'hidden',{configurable:false});assert.sameValue(m(o),!frozenQuery);
Object.defineProperty(o,'hidden',{writable:false});assert.sameValue(m(o),true);
// CASE: isFrozen-all-properties-not-just-first-or-enumerable
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isFrozen",m=Object[name],isQuery=true,freezing=false,frozenQuery=true,strictMode=(function(){return this;})()===undefined;
var o={};Object.defineProperty(o,'first',{value:1});Object.defineProperty(o,'hidden',{value:2,writable:true,configurable:true,enumerable:false});Object.preventExtensions(o);
assert.sameValue(m(o),false);Object.defineProperty(o,'hidden',{configurable:false});assert.sameValue(m(o),!frozenQuery);
Object.defineProperty(o,'hidden',{writable:false});assert.sameValue(m(o),true);
// CASE: isSealed-symbol-only-state-is-observed
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isSealed",m=Object[name],isQuery=true,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var k=Symbol('hidden'),o={};Object.defineProperty(o,k,{value:1,writable:true,configurable:true});Object.preventExtensions(o);
assert.sameValue(m(o),false);Object.defineProperty(o,k,{configurable:false});assert.sameValue(m(o),!frozenQuery);Object.defineProperty(o,k,{writable:false});assert.sameValue(m(o),true);
// CASE: isFrozen-symbol-only-state-is-observed
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isFrozen",m=Object[name],isQuery=true,freezing=false,frozenQuery=true,strictMode=(function(){return this;})()===undefined;
var k=Symbol('hidden'),o={};Object.defineProperty(o,k,{value:1,writable:true,configurable:true});Object.preventExtensions(o);
assert.sameValue(m(o),false);Object.defineProperty(o,k,{configurable:false});assert.sameValue(m(o),!frozenQuery);Object.defineProperty(o,k,{writable:false});assert.sameValue(m(o),true);
// CASE: isSealed-sparse-array-length-counts-as-data-property
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isSealed",m=Object[name],isQuery=true,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var a=new Array(4294967295);Object.preventExtensions(a);assert.sameValue(m(a),!frozenQuery);
Object.defineProperty(a,'length',{writable:false});assert.sameValue(m(a),true);assert.sameValue(Object.getOwnPropertyDescriptor(a,'0'),undefined);assert.sameValue(a.length,4294967295);
// CASE: isFrozen-sparse-array-length-counts-as-data-property
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isFrozen",m=Object[name],isQuery=true,freezing=false,frozenQuery=true,strictMode=(function(){return this;})()===undefined;
var a=new Array(4294967295);Object.preventExtensions(a);assert.sameValue(m(a),!frozenQuery);
Object.defineProperty(a,'length',{writable:false});assert.sameValue(m(a),true);assert.sameValue(Object.getOwnPropertyDescriptor(a,'0'),undefined);assert.sameValue(a.length,4294967295);
// CASE: isSealed-array-element-flags-are-independent-from-length
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isSealed",m=Object[name],isQuery=true,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var a=[1];Object.defineProperty(a,'length',{writable:false});Object.preventExtensions(a);assert.sameValue(m(a),false);
Object.defineProperty(a,'0',{configurable:false});assert.sameValue(m(a),!frozenQuery);Object.defineProperty(a,'0',{writable:false});assert.sameValue(m(a),true);
// CASE: isFrozen-array-element-flags-are-independent-from-length
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isFrozen",m=Object[name],isQuery=true,freezing=false,frozenQuery=true,strictMode=(function(){return this;})()===undefined;
var a=[1];Object.defineProperty(a,'length',{writable:false});Object.preventExtensions(a);assert.sameValue(m(a),false);
Object.defineProperty(a,'0',{configurable:false});assert.sameValue(m(a),!frozenQuery);Object.defineProperty(a,'0',{writable:false});assert.sameValue(m(a),true);
// CASE: isSealed-string-wrapper-empty-and-nonempty
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isSealed",m=Object[name],isQuery=true,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var values=['','ab','\ud800'];for(var i=0;i<values.length;i++){var o=new String(values[i]);assert.sameValue(m(o),false);Object.preventExtensions(o);assert.sameValue(m(o),true);}
// CASE: isFrozen-string-wrapper-empty-and-nonempty
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isFrozen",m=Object[name],isQuery=true,freezing=false,frozenQuery=true,strictMode=(function(){return this;})()===undefined;
var values=['','ab','\ud800'];for(var i=0;i<values.length;i++){var o=new String(values[i]);assert.sameValue(m(o),false);Object.preventExtensions(o);assert.sameValue(m(o),true);}
// CASE: seal-sealed-array-length-can-grow-and-shrink-holes
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var a=new Array(5);a[1]=7;m(a);a.length=9;assert.sameValue(a.length,9);a.length=2;assert.sameValue(a.length,2);assert.sameValue(a[1],7);assert.sameValue(Object.isSealed(a),true);assert.sameValue(Object.isFrozen(a),false);
var caught=false;try{a.length=0;}catch(e){assert.sameValue(e instanceof TypeError,true);caught=true;}assert.sameValue(caught,strictMode);assert.sameValue(a.length,2);assert.sameValue(a[1],7);
Object.defineProperty(a,'length',{writable:false});assert.sameValue(Object.isFrozen(a),false);Object.defineProperty(a,'1',{writable:false});assert.sameValue(Object.isFrozen(a),true);
// CASE: freeze-freeze-locks-empty-array-length
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var a=[];m(a);assert.sameValue(Object.isFrozen(a),true);assert.sameValue(Object.getOwnPropertyDescriptor(a,'length').writable,false);
assert.throws(TypeError,function(){'use strict';a.length=0;});assert.throws(TypeError,function(){a.push(1);});assert.sameValue(a.length,0);
// CASE: freeze-frozen-data-redefinition-uses-samevalue
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var child={},o={zero:-0,nan:NaN,child:child};m(o);
Object.defineProperty(o,'zero',{value:-0});Object.defineProperty(o,'nan',{value:NaN});Object.defineProperty(o,'child',{value:child});
assert.throws(TypeError,function(){Object.defineProperty(o,'zero',{value:0});});assert.throws(TypeError,function(){Object.defineProperty(o,'child',{value:{}});});
assert.sameValue(o.zero,-0);assert.sameValue(o.nan,NaN);assert.sameValue(o.child,child);
// CASE: seal-sealing-a-frozen-object-does-not-thaw
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var o=Object.freeze({x:1});assert.sameValue(m(o),o);assert.sameValue(Object.isFrozen(o),true);assert.sameValue(Object.getOwnPropertyDescriptor(o,'x').writable,false);
// CASE: freeze-freezing-a-sealed-object-tightens-writability
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var o=Object.seal({x:1});assert.sameValue(Object.isFrozen(o),false);assert.sameValue(m(o),o);assert.sameValue(Object.isFrozen(o),true);assert.sameValue(Object.getOwnPropertyDescriptor(o,'x').writable,false);
// CASE: seal-mapped-arguments-capture-current-value-and-aliasing
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var fn=Function('a','applyIntegrity','frozen','a=2;var args=arguments;applyIntegrity(args);assert.sameValue(args[0],2);a=3;assert.sameValue(args[0],frozen?2:3);args[0]=4;assert.sameValue(a,frozen?3:4);return [a,args[0],Object.getOwnPropertyDescriptor(args,"0").writable];');
var result=fn(1,m,freezing);assert.sameValue(result[0],freezing?3:4);assert.sameValue(result[1],freezing?2:4);assert.sameValue(result[2],!freezing);
// CASE: freeze-mapped-arguments-capture-current-value-and-aliasing
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var fn=Function('a','applyIntegrity','frozen','a=2;var args=arguments;applyIntegrity(args);assert.sameValue(args[0],2);a=3;assert.sameValue(args[0],frozen?2:3);args[0]=4;assert.sameValue(a,frozen?3:4);return [a,args[0],Object.getOwnPropertyDescriptor(args,"0").writable];');
var result=fn(1,m,freezing);assert.sameValue(result[0],freezing?3:4);assert.sameValue(result[1],freezing?2:4);assert.sameValue(result[2],!freezing);
// CASE: seal-sealed-mapping-survives-then-explicit-readonly-severs
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var fn=Function('a','applyIntegrity','var args=arguments;applyIntegrity(args);a=5;assert.sameValue(args[0],5);Object.defineProperty(args,"0",{writable:false});a=6;assert.sameValue(args[0],5);return a;');
assert.sameValue(fn(1,m),6);
// CASE: seal-strict-arguments-restricted-accessor-not-invoked
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
function f(a){'use strict';var args=arguments,d=Object.getOwnPropertyDescriptor(args,'callee');assert.sameValue(m(args),args);assert.sameValue(Object.isSealed(args),true);assert.sameValue(Object.isFrozen(args),freezing);var after=Object.getOwnPropertyDescriptor(args,'callee');assert.sameValue(after.get,d.get);assert.sameValue(after.set,d.set);assert.sameValue(after.configurable,false);assert.throws(TypeError,function(){return args.callee;});return args[0];}
assert.sameValue(f(7),7);
// CASE: freeze-strict-arguments-restricted-accessor-not-invoked
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
function f(a){'use strict';var args=arguments,d=Object.getOwnPropertyDescriptor(args,'callee');assert.sameValue(m(args),args);assert.sameValue(Object.isSealed(args),true);assert.sameValue(Object.isFrozen(args),freezing);var after=Object.getOwnPropertyDescriptor(args,'callee');assert.sameValue(after.get,d.get);assert.sameValue(after.set,d.set);assert.sameValue(after.configurable,false);assert.throws(TypeError,function(){return args.callee;});return args[0];}
assert.sameValue(f(7),7);
// CASE: seal-regexp-global-lastindex-respects-data-lock
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var r=/a/g;r.lastIndex=1;assert.sameValue(m(r),r);var d=Object.getOwnPropertyDescriptor(r,'lastIndex');
assert.sameValue(d.value,1);assert.sameValue(d.writable,!freezing);assert.sameValue(d.enumerable,false);assert.sameValue(d.configurable,false);
if(freezing){assert.throws(TypeError,function(){r.exec('ba');});assert.sameValue(r.lastIndex,1);}else{var match=r.exec('ba');assert.sameValue(match[0],'a');assert.sameValue(match.index,1);assert.sameValue(r.lastIndex,2);}
// CASE: freeze-regexp-global-lastindex-respects-data-lock
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var r=/a/g;r.lastIndex=1;assert.sameValue(m(r),r);var d=Object.getOwnPropertyDescriptor(r,'lastIndex');
assert.sameValue(d.value,1);assert.sameValue(d.writable,!freezing);assert.sameValue(d.enumerable,false);assert.sameValue(d.configurable,false);
if(freezing){assert.throws(TypeError,function(){r.exec('ba');});assert.sameValue(r.lastIndex,1);}else{var match=r.exec('ba');assert.sameValue(match[0],'a');assert.sameValue(match.index,1);assert.sameValue(r.lastIndex,2);}
// CASE: seal-regexp-nonglobal-exec-does-not-need-lastindex-write
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var r=/a/;r.lastIndex=7;m(r);assert.sameValue(r.exec('a')[0],'a');assert.sameValue(r.lastIndex,7);assert.sameValue(Object.isSealed(r),true);assert.sameValue(Object.isFrozen(r),freezing);
// CASE: freeze-regexp-nonglobal-exec-does-not-need-lastindex-write
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var r=/a/;r.lastIndex=7;m(r);assert.sameValue(r.exec('a')[0],'a');assert.sameValue(r.lastIndex,7);assert.sameValue(Object.isSealed(r),true);assert.sameValue(Object.isFrozen(r),freezing);
// CASE: isSealed-regexp-lastindex-counts-in-integrity-query
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isSealed",m=Object[name],isQuery=true,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
var r=/a/g;Object.preventExtensions(r);assert.sameValue(m(r),!frozenQuery);Object.defineProperty(r,'lastIndex',{writable:false});assert.sameValue(m(r),true);
// CASE: isFrozen-regexp-lastindex-counts-in-integrity-query
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="isFrozen",m=Object[name],isQuery=true,freezing=false,frozenQuery=true,strictMode=(function(){return this;})()===undefined;
var r=/a/g;Object.preventExtensions(r);assert.sameValue(m(r),!frozenQuery);Object.defineProperty(r,'lastIndex',{writable:false});assert.sameValue(m(r),true);
// CASE: seal-ordinary-constructor-still-constructs
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="seal",m=Object[name],isQuery=false,freezing=false,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
function F(value){this.x=value;}var prototype=F.prototype;m(F);prototype.y=9;var o=new F(4);assert.sameValue(o.x,4);assert.sameValue(o.y,9);assert.sameValue(Object.getPrototypeOf(o),prototype);assert.sameValue(Object.isExtensible(o),true);
// CASE: freeze-ordinary-constructor-still-constructs
assert.sameValue(typeof Object.seal,'function','seal prerequisite');
assert.sameValue(typeof Object.freeze,'function','freeze prerequisite');
assert.sameValue(typeof Object.isSealed,'function','isSealed prerequisite');
assert.sameValue(typeof Object.isFrozen,'function','isFrozen prerequisite');
var guardSealed={x:1},guardFrozen={x:1};
assert.sameValue(Object.seal(guardSealed),guardSealed);assert.sameValue(Object.isSealed(guardSealed),true);assert.sameValue(Object.isFrozen(guardSealed),false);
assert.sameValue(Object.freeze(guardFrozen),guardFrozen);assert.sameValue(Object.isFrozen(guardFrozen),true);assert.sameValue(Object.isSealed(guardFrozen),true);
assert.sameValue(Object.isSealed({}),false);assert.sameValue(Object.isFrozen({}),false);
var name="freeze",m=Object[name],isQuery=false,freezing=true,frozenQuery=false,strictMode=(function(){return this;})()===undefined;
function F(value){this.x=value;}var prototype=F.prototype;m(F);prototype.y=9;var o=new F(4);assert.sameValue(o.x,4);assert.sameValue(o.y,9);assert.sameValue(Object.getPrototypeOf(o),prototype);assert.sameValue(Object.isExtensible(o),true);
