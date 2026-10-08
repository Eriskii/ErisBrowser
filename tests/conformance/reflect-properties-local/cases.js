// Independently authored source proposal. One named body per fresh realm.
// Strict mode prepends its directive to this whole file plus invocation.
function rpCheck(ok, label) { if (!ok) throw new Error(label); }
function rpTypeError(fn) {
  var caught = false;
  try { fn(); } catch (e) { if (!(e instanceof TypeError)) throw e; caught = true; }
  rpCheck(caught, 'intrinsic TypeError required');
}
function rpGuard() {
  var names = ['deleteProperty','get','getOwnPropertyDescriptor','getPrototypeOf',
    'has','isExtensible','preventExtensions','set','setPrototypeOf'];
  for (var i = 0; i < names.length; i++) {
    if (typeof Reflect[names[i]] !== 'function') throw new TypeError('Reflect prerequisite');
  }
  var p = {}, o = Object.create(p);
  if (!Reflect.set(o,'x',17) || Reflect.get(o,'x') !== 17 || !Reflect.has(o,'x') ||
      Reflect.getOwnPropertyDescriptor(o,'x').value !== 17 || Reflect.getPrototypeOf(o) !== p ||
      !Reflect.deleteProperty(o,'x') || Reflect.has(o,'x') || !Reflect.isExtensible(o) ||
      !Reflect.setPrototypeOf(o,null) || Reflect.getPrototypeOf(o) !== null ||
      !Reflect.preventExtensions(o) || Reflect.isExtensible(o)) throw new TypeError('working Reflect prerequisite');
}
function rpView() {
  var a = new Uint8Array(1); a[0] = 17;
  if (a[0] !== 17 || a.length !== 1 || !ArrayBuffer.isView(a)) throw new TypeError('authentic view prerequisite');
  return a;
}
var reflectPropertyCases = {
  metadata_and_nonconstruction: function () {
    rpGuard();
    var names = ['deleteProperty','get','getOwnPropertyDescriptor','getPrototypeOf',
      'has','isExtensible','preventExtensions','set','setPrototypeOf'];
    var lengths = [2,2,2,1,2,1,1,3,2];
    function Guard() { this.marker = 41; }
    rpCheck(Reflect.construct(Guard,[]).marker === 41, 'construct prerequisite');
    for (var i = 0; i < names.length; i++) {
      var f = Reflect[names[i]], d = Object.getOwnPropertyDescriptor(Reflect,names[i]);
      rpCheck(d.value === f && d.writable && !d.enumerable && d.configurable, 'method descriptor');
      rpCheck(f.name === names[i] && f.length === lengths[i], 'literal names and lengths');
      rpCheck(Object.getPrototypeOf(f) === Function.prototype && !Object.hasOwn(f,'prototype'), 'ordinary builtin shape');
      rpTypeError(function () { Reflect.construct(f,[]); });
    }
    return true;
  },
  target_validation_precedes_key_conversion: function () {
    rpGuard();
    var reads = 0, key = {};
    Object.defineProperty(key,Symbol.toPrimitive,{get:function () { reads++; throw new Error('key touched'); }});
    var bad = [undefined,null,false,0,'x',Symbol('primitive')];
    var keyed = ['get','set','has','deleteProperty','getOwnPropertyDescriptor'];
    for (var i = 0; i < keyed.length; i++) {
      for (var j = 0; j < bad.length; j++) {
        var f = Reflect[keyed[i]], target = bad[j];
        rpTypeError(function () { f(target,key,9); });
      }
    }
    var plain = ['getPrototypeOf','isExtensible','preventExtensions','setPrototypeOf'];
    for (var k = 0; k < plain.length; k++) {
      for (var n = 0; n < bad.length; n++) {
        var g = Reflect[plain[k]], value = bad[n];
        rpTypeError(function () { g(value,null); });
      }
    }
    rpCheck(reads === 0, 'invalid target cannot inspect key');
    return true;
  },
  get_uses_original_receiver_and_preserves_abrupt_identity: function () {
    rpGuard();
    var p = {}, o = Object.create(p), other = {}, seen = [];
    Object.defineProperty(p,'x',{get:function () { 'use strict'; seen.push(this); return this; }});
    rpCheck(Reflect.get(o,'x') === o && Reflect.get(o,'x',other) === other, 'target and distinct receiver');
    rpCheck(Reflect.get(o,'x',undefined) === undefined && Reflect.get(o,'x',null) === null, 'explicit nullish receiver');
    rpCheck(Reflect.get(o,'x',7) === 7 && seen.length === 5, 'primitive receiver without boxing');
    var reason = new TypeError('authored'), caught;
    Object.defineProperty(o,'boom',{get:function () { throw reason; }});
    try { Reflect.get(o,'boom'); } catch (e) { caught = e; }
    rpCheck(caught === reason, 'getter abrupt identity escapes');
    return true;
  },
  key_conversion_once_precedes_fresh_property_walk: function () {
    rpGuard();
    var first = {x:1}, second = {x:29}, o = Object.create(first), trace = '', key = {};
    key[Symbol.toPrimitive] = function (hint) {
      rpCheck(this === key && hint === 'string','key hook receiver/hint');
      trace += 'K'; Object.setPrototypeOf(o,second); return 'x';
    };
    rpCheck(Reflect.get(o,key) === 29 && trace === 'K', 'fresh prototype after key callback');
    var sym = Symbol('exact'), symbolKey = {toString:function () { trace += 'S'; return sym; }};
    o[sym] = 31;
    rpCheck(Reflect.get(o,symbolKey) === 31 && trace === 'KS', 'ordinary string hint may return Symbol');
    var reason = {}, abrupt = {}; abrupt[Symbol.toPrimitive] = function () { throw reason; };
    var caught; try { Reflect.has(o,abrupt); } catch (e) { caught = e; }
    rpCheck(caught === reason, 'key abrupt identity escapes');
    return true;
  },
  presence_and_descriptors_do_not_call_getters: function () {
    rpGuard();
    var reads = 0, p = {}, o = Object.create(p), sym = Symbol('key'), lone = '\uD800';
    function getter() { reads++; return 3; }
    function setter(v) { reads += v; }
    Object.defineProperty(p,'inherited',{get:getter});
    Object.defineProperty(o,lone,{get:getter,set:setter,enumerable:true,configurable:true});
    o[sym] = 11;
    rpCheck(Reflect.has(o,'inherited') && Reflect.getOwnPropertyDescriptor(o,'inherited') === undefined, 'has traverses, descriptor does not');
    var a = Reflect.getOwnPropertyDescriptor(o,lone), b = Reflect.getOwnPropertyDescriptor(o,lone);
    rpCheck(a !== b && a.get === getter && a.set === setter && a.enumerable && a.configurable, 'fresh accessor descriptor');
    rpCheck(!Object.hasOwn(a,'value') && !Object.hasOwn(a,'writable'), 'accessor fields only');
    a.get = null;
    rpCheck(Reflect.getOwnPropertyDescriptor(o,lone).get === getter && reads === 0, 'descriptor copy not live');
    var d = Reflect.getOwnPropertyDescriptor(o,sym);
    rpCheck(d.value === 11 && d.writable && d.enumerable && d.configurable && !Reflect.has(o,'absent'), 'symbol/data flags');
    return true;
  },
  deletion_returns_boolean_and_preserves_array_length: function () {
    rpGuard();
    var p = {}, o = Object.create(p), sym = Symbol('key');
    Object.defineProperty(p,'inherited',{value:2,configurable:false});
    Object.defineProperty(o,'fixed',{value:3,configurable:false});
    o[sym] = 5;
    rpCheck(Reflect.deleteProperty(o,'inherited') && Reflect.has(o,'inherited'), 'only own delete');
    rpCheck(!Reflect.deleteProperty(o,'fixed') && o.fixed === 3, 'nonconfigurable Boolean refusal');
    rpCheck(Reflect.deleteProperty(o,sym) && !Reflect.has(o,sym) && Reflect.deleteProperty(o,'absent'), 'symbol and absent deletion');
    var a = [7,8];
    rpCheck(Reflect.deleteProperty(a,'0') && a.length === 2 && !(0 in a) && a[1] === 8, 'array hole no length shrink');
    rpCheck(!Reflect.deleteProperty(Object('a'),'0'), 'boxed string index nonconfigurable');
    return true;
  },
  ordinary_set_receiver_descriptor_distinctions: function () {
    rpGuard();
    var target = {x:1}, receiver = {}, calls = 0, value = {};
    rpCheck(Reflect.set(target,'x',value,receiver) && receiver.x === value && target.x === 1, 'distinct Receiver stores value without coercion');
    Object.defineProperty(receiver,'x',{get:function () { return 7; },set:function () { calls++; },configurable:true});
    rpCheck(!Reflect.set(target,'x',9,receiver) && calls === 0 && receiver.x === 7, 'Receiver own accessor is refusal, not setter invocation');
    Object.defineProperty(receiver,'x',{value:8,writable:false});
    rpCheck(!Reflect.set(target,'x',9,receiver) && receiver.x === 8, 'readonly Receiver');
    rpCheck(!Reflect.set(target,'x',9,undefined) && !Reflect.set(target,'x',9,3), 'explicit primitive Receiver');
    rpCheck(Reflect.set(target,'x',12) && target.x === 12, 'omitted Receiver defaults target');
    var seen, passed;
    Object.defineProperty(target,'accessor',{set:function (v) { 'use strict'; seen = this; passed = v; }});
    rpCheck(Reflect.set(target,'accessor',value,undefined) && seen === undefined && passed === value, 'target setter gets undefined Receiver');
    rpCheck(Reflect.set(target,'accessor',value,4) && seen === 4, 'target setter gets primitive Receiver');
    return true;
  },
  set_expression_key_and_callback_order: function () {
    rpGuard();
    var trace = '', target = {}, receiver = {}, payload = {valueOf:function () { throw new Error('no ordinary coercion'); }};
    Object.defineProperty(target,'x',{set:function (v) { trace += 'S'; rpCheck(this === receiver && v === payload,'setter Receiver/value'); }});
    var key = {}; key[Symbol.toPrimitive] = function (hint) { trace += 'P'; rpCheck(hint === 'string','hint'); return 'x'; };
    function t() { trace += 'T'; return target; }
    function k() { trace += 'K'; return key; }
    function v() { trace += 'V'; return payload; }
    function r() { trace += 'R'; return receiver; }
    function extra() { trace += 'E'; }
    rpCheck(Reflect.set(t(),k(),v(),r(),extra()) && trace === 'TKVREPS','expressions before builtin conversion');
    var reason = new TypeError('setter'), caught;
    Object.defineProperty(target,'boom',{set:function () { throw reason; }});
    try { Reflect.set(target,'boom',1); } catch (e) { caught = e; }
    rpCheck(caught === reason,'callback TypeError cannot become false');
    return true;
  },
  typed_array_target_and_receiver_are_distinct_algorithms: function () {
    rpGuard(); var a = rpView(), calls = 0, rhs = {valueOf:function () { calls++; return 9; }};
    var receiver = {};
    rpCheck(Reflect.get(a,'0',receiver) === 17,'Get reads target storage');
    rpCheck(Reflect.set(a,'0',rhs,receiver) && receiver[0] === rhs && a[0] === 17 && calls === 0,'valid target distinct ordinary receiver');
    rpCheck(Reflect.set(a,'1',rhs,{}) && calls === 0,'invalid target distinct receiver succeeds without conversion');
    var empty = new Uint8Array(0), ordinary = {'0':1};
    rpCheck(!Reflect.set(ordinary,'0',rhs,empty) && calls === 0,'ordinary target plus invalid typed Receiver uses DefineOwnProperty');
    rpCheck(!Reflect.set(a,'0',rhs,empty) && calls === 0,'valid typed target plus short typed Receiver refuses');
    var other = new Uint8Array(1);
    rpCheck(Reflect.set(a,'0',rhs,other) && calls === 1 && other[0] === 9 && a[0] === 17,'valid typed Receiver converts once');
    rpCheck(Reflect.set(a,'-0',rhs) && calls === 2 && !Object.hasOwn(a,'-0'),'same invalid Receiver still converts');
    return true;
  },
  typed_array_callbacks_refresh_bounds: function () {
    rpGuard(); rpView();
    var buffer = new ArrayBuffer(0,{maxByteLength:4}), a = new Uint8Array(buffer), calls = 0;
    var rhs = {valueOf:function () { calls++; buffer.resize(1); return 23; }};
    rpCheck(Reflect.set(a,'0',rhs) && a[0] === 23 && calls === 1,'RHS grows formerly invalid index');
    rhs.valueOf = function () { calls++; buffer.resize(0); return 42; };
    rpCheck(Reflect.set(a,'0',rhs) && calls === 2 && !Reflect.has(a,'0'),'RHS shrinks before fresh validity');
    var operations = ['get','has','getOwnPropertyDescriptor','deleteProperty'];
    var expected = [undefined,false,undefined,true];
    for (var i = 0; i < operations.length; i++) {
      buffer.resize(2); a[1] = 19;
      var hits = 0, key = {toString:function () { hits++; buffer.resize(1); return '1'; }};
      rpCheck(Reflect[operations[i]](a,key) === expected[i] && hits === 1,'key callback fresh witness');
    }
    var detached = new Uint8Array(new ArrayBuffer(1)), key2 = {toString:function () { detached.buffer.transfer(); return '0'; }};
    rpCheck(Reflect.getOwnPropertyDescriptor(detached,key2) === undefined && !Reflect.has(detached,'0'),'detach during key conversion');
    return true;
  },
  typed_array_descriptors_and_prototype_poison: function () {
    rpGuard(); var a = rpView(), d = Reflect.getOwnPropertyDescriptor(a,'0'), reads = 0;
    rpCheck(d.value === 17 && d.writable && d.enumerable && d.configurable,'numeric descriptor literal');
    rpCheck(!Reflect.deleteProperty(a,'0') && a[0] === 17,'configurable descriptor still integer-indexed delete refusal');
    var p = Object.create(Object.getPrototypeOf(a));
    Object.defineProperty(p,'-0',{get:function () { reads++; throw new Error('must stop'); }});
    Object.setPrototypeOf(a,p);
    rpCheck(!Reflect.has(a,'-0') && Reflect.get(a,'-0') === undefined && reads === 0,'invalid canonical key stops prototype walk');
    Object.defineProperty(p,'01',{get:function () { reads++; return 33; }});
    rpCheck(Reflect.has(a,'01') && reads === 0 && Reflect.get(a,'01') === 33 && reads === 1,'ordinary key traverses with presence avoiding getter');
    return true;
  },
  extensibility_boolean_results_and_resizable_views: function () {
    rpGuard(); var o = {}, a = rpView();
    rpCheck(Reflect.isExtensible(o) && Reflect.preventExtensions(o) && !Reflect.isExtensible(o),'ordinary prevention');
    rpCheck(Reflect.preventExtensions(o) && !Reflect.set(o,'new',1),'repeat prevention and new property refusal');
    rpCheck(Reflect.preventExtensions(a) && !Reflect.isExtensible(a),'fixed-buffer view prevention');
    rpCheck(Reflect.set(a,'0',22) && a[0] === 22 && !Reflect.set(a,'extra',1),'existing element remains writable');
    var b = new ArrayBuffer(0,{maxByteLength:4}), tracking = new Uint8Array(b), fixed = new Uint8Array(b,0,0);
    rpCheck(!Reflect.preventExtensions(tracking) && Reflect.isExtensible(tracking),'tracking resizable refusal');
    rpCheck(!Reflect.preventExtensions(fixed) && Reflect.isExtensible(fixed),'fixed-length resizable refusal');
    rpTypeError(function () { Object.preventExtensions(tracking); });
    return true;
  },
  prototypes_validate_and_return_boolean_refusals: function () {
    rpGuard(); var p = {}, q = {}, o = Object.create(p);
    rpCheck(Reflect.getPrototypeOf(o) === p && Reflect.setPrototypeOf(o,q) && Reflect.getPrototypeOf(o) === q,'ordinary prototype mutation');
    var child = Object.create(o);
    rpCheck(!Reflect.setPrototypeOf(o,child) && Reflect.getPrototypeOf(o) === q,'cycle returns false');
    rpCheck(Reflect.preventExtensions(o) && Reflect.setPrototypeOf(o,q) && !Reflect.setPrototypeOf(o,p),'same prototype on nonextensible succeeds');
    var bad = [undefined,0,false,'x',Symbol('p')];
    for (var i = 0; i < bad.length; i++) { var value = bad[i]; rpTypeError(function () { Reflect.setPrototypeOf({},value); }); }
    rpCheck(Reflect.setPrototypeOf(q,null) && Reflect.getPrototypeOf(q) === null,'null prototype');
    rpCheck(Reflect.getPrototypeOf([]) === Array.prototype && Reflect.getPrototypeOf(function () {}) === Function.prototype,'array/function brands');
    return true;
  },
  saved_methods_survive_own_replacement_and_deletion: function () {
    rpGuard(); var get = Reflect.get, set = Reflect.set, has = Reflect.has, del = Reflect.deleteProperty;
    var o = {x:11};
    Reflect.get = 3;
    rpCheck(Reflect.get === 3 && Reflect.apply(get,null,[o,'x']) === 11,'saved getter independent this');
    rpCheck(del(Reflect,'set') && Reflect.set === undefined && set(o,'x',19) && get(o,'x') === 19,'saved setter survives deletion');
    var sym = Symbol('x');
    rpCheck(set(o,sym,29) && has(o,sym) && get(o,sym) === 29 && del(o,sym) && !has(o,sym),'saved methods Symbol identity');
    return true;
  }
};
