// Independently authored literal witnesses; one named body per fresh realm.
// The driver prepends the strict directive to this file for strict mode.
function tvCheck(ok, label) { if (!ok) throw new Error(label); }
function tvThrows(fn, kind) {
  var caught = false;
  try { fn(); } catch (e) { if (!(e instanceof kind)) throw e; caught = true; }
  tvCheck(caught, 'required intrinsic error');
}
function tvAbrupt(fn, reason) {
  var caught = false;
  try { fn(); } catch (e) { tvCheck(e === reason, 'authored abrupt identity'); caught = true; }
  tvCheck(caught, 'authored throw reached');
}
function tvKinds() {
  return [Int8Array,Uint8Array,Uint8ClampedArray,Int16Array,Uint16Array,
    Int32Array,Uint32Array,Float16Array,Float32Array,Float64Array];
}
function tvGuard() {
  var p = Object.getPrototypeOf(Uint8Array.prototype);
  if (typeof p.subarray !== 'function' || typeof p.join !== 'function' ||
      !Object.hasOwn(p,'toString') || p.toString !== Array.prototype.toString)
    throw new TypeError('TypedArray view/string methods prerequisite');
  var a = new Uint8Array([3,5]), b = p.subarray.call(a,1);
  if (!ArrayBuffer.isView(a) || b === a || b.buffer !== a.buffer || b.length !== 1 ||
      b[0] !== 5 || p.join.call(a,':') !== '3:5' || p.toString.call(a) !== '3,5')
    throw new TypeError('working view/string prerequisite');
  return p;
}
var typedArrayViewCases = {
  metadata_alias_and_saved_identity: function () {
    var p = tvGuard(), names = ['subarray','join','toString'], lengths = [2,1,0];
    function C() { this.marker = 17; }
    tvCheck(Reflect.construct(C,[]).marker === 17, 'real construct prerequisite');
    for (var i = 0; i < names.length; i++) {
      var f = p[names[i]], d = Object.getOwnPropertyDescriptor(p,names[i]);
      tvCheck(d.value === f && d.writable && !d.enumerable && d.configurable, 'method flags');
      tvCheck(f.name === names[i] && f.length === lengths[i], 'literal metadata');
      tvCheck(Object.getPrototypeOf(f) === Function.prototype && !Object.hasOwn(f,'prototype'), 'builtin shape');
      tvThrows(function () { Reflect.construct(f,[]); },TypeError);
    }
    var sub = p.subarray, join = p.join, str = p.toString, a = new Uint8Array([4,6]);
    tvCheck(delete p.subarray && delete p.join && delete p.toString, 'configurable methods');
    tvCheck(!Object.hasOwn(p,'subarray') && !Object.hasOwn(p,'join') && !Object.hasOwn(p,'toString'), 'actual deletion');
    tvCheck(sub.call(a,1)[0] === 6 && join.call(a,'/') === '4/6', 'saved native identity');
    a.join = join;
    tvCheck(str.call(a) === '4,6' && str === Array.prototype.toString, 'saved exact alias');
    return true;
  },
  all_ten_kinds_share_bytes_and_preserve_kind: function () {
    tvGuard();
    var kinds = tvKinds(), widths = [1,1,1,2,2,4,4,2,4,8];
    for (var i = 0; i < kinds.length; i++) {
      var K = kinds[i], a = new K([1,2,3,4]); a.extra = 51;
      var b = a.subarray(1,3);
      tvCheck(a.length === 4 && a[1] === 2 && ArrayBuffer.isView(a), 'genuine source');
      tvCheck(b !== a && b.buffer === a.buffer && b.byteOffset === widths[i] && b.length === 2, 'shared slice geometry');
      tvCheck(Object.getPrototypeOf(b) === K.prototype && b[0] === 2 && b[1] === 3, 'same-kind contents');
      tvCheck(!Object.hasOwn(b,'extra'), 'expandos not copied');
      b[0] = 9; a[2] = 11;
      tvCheck(a[1] === 9 && b[1] === 11 && a[0] === 1 && a[3] === 4, 'bidirectional shared storage');
    }
    return true;
  },
  literal_clamped_indices_and_empty_offsets: function () {
    tvGuard();
    var a = new Uint8Array([10,20,30,40,50]);
    var starts = [-Infinity,NaN,-0,1.9,-1.9,Infinity,-99,4];
    var ends = [NaN,undefined,undefined,4.9,undefined,undefined,99,1];
    var offsets = [0,0,0,1,4,5,0,4], lengths = [0,5,5,3,1,0,5,0];
    for (var i = 0; i < starts.length; i++) {
      var b = a.subarray(starts[i],ends[i]);
      tvCheck(b.buffer === a.buffer && b.byteOffset === offsets[i] && b.length === lengths[i], 'literal clamp result');
    }
    tvCheck(a.subarray().length === 5 && a.subarray(1,null).length === 0, 'omission and null');
    return true;
  },
  invalid_receiver_precedes_argument_hooks: function () {
    var p = tvGuard(), calls = 0;
    var arg = {valueOf:function () { calls++; return 0; }};
    var bad = [undefined,null,7,'x',{},[],new DataView(new ArrayBuffer(2))];
    tvCheck(bad[6].getUint8(0) === 0, 'genuine DataView prerequisite');
    for (var i = 0; i < bad.length; i++) {
      var receiver = bad[i];
      tvThrows(function () { p.subarray.call(receiver,arg,arg); },TypeError);
    }
    tvCheck(calls === 0, 'brand rejection before ToInteger');
    var fake = Object.create(Uint8Array.prototype);
    tvThrows(function () { p.subarray.call(fake,arg); },TypeError);
    tvCheck(calls === 0, 'prototype does not grant private brand');
    return true;
  },
  callback_order_and_species_argument_values: function () {
    tvGuard();
    var a = new Uint8Array([2,4,6,8,10]), trace = '', owner = {};
    var begin = {valueOf:function () { tvCheck(this === begin,'begin receiver'); trace += 'B'; return 1; }};
    var end = {valueOf:function () { tvCheck(this === end,'end receiver'); trace += 'E'; return 4; }};
    function Species(buffer,offset,length) {
      trace += 'N';
      tvCheck(new.target === Species && arguments.length === 3, 'species construct and arity');
      tvCheck(buffer === a.buffer && offset === 1 && length === 3, 'literal species arguments');
      return new Uint8Array(buffer,offset,length);
    }
    Object.defineProperty(a,'constructor',{get:function () { trace += 'C'; return owner; }});
    Object.defineProperty(owner,Symbol.species,{get:function () { trace += 'S'; return Species; }});
    var result = a.subarray(begin,end);
    tvCheck(trace === 'BECSN' && result[0] === 4 && result[2] === 8, 'ordered callbacks and result');
    return true;
  },
  abrupt_conversion_and_species_identity: function () {
    tvGuard();
    var a = new Uint8Array(3), reason = {}, trace = '', owner = {};
    Object.defineProperty(a,'constructor',{configurable:true,get:function () { trace += 'C'; return owner; }});
    Object.defineProperty(owner,Symbol.species,{configurable:true,get:function () { trace += 'S'; throw reason; }});
    tvAbrupt(function () { a.subarray({valueOf:function () { trace += 'B'; throw reason; }},{valueOf:function () { trace += 'E'; return 2; }}); },reason);
    tvCheck(trace === 'B','begin abrupt excludes later callbacks');
    trace = '';
    tvAbrupt(function () { a.subarray(0,{valueOf:function () { trace += 'E'; throw reason; }}); },reason);
    tvCheck(trace === 'E','end abrupt excludes species');
    trace = '';
    tvAbrupt(function () { a.subarray(0,1); },reason);
    tvCheck(trace === 'CS','species getter abrupt identity');
    Object.defineProperty(owner,Symbol.species,{value:function () { trace += 'N'; throw reason; }});
    trace = '';
    tvAbrupt(function () { a.subarray(0,1); },reason);
    tvCheck(trace === 'CN','constructor abrupt identity');
    tvThrows(function () { a.subarray(Symbol('start')); },TypeError);
    return true;
  },
  intrinsic_default_survives_global_replacement: function () {
    tvGuard();
    var K = Uint8Array, a = new K([13,17,19]);
    Uint8Array = function () { throw new Error('global constructor must not be used'); };
    a.constructor = undefined;
    var one = a.subarray(1);
    tvCheck(Object.getPrototypeOf(one) === K.prototype && one[0] === 17, 'private default constructor');
    a.constructor = {}; a.constructor[Symbol.species] = null;
    var two = a.subarray(2);
    tvCheck(two.buffer === a.buffer && two[0] === 19, 'null species defaults');
    a.constructor = null;
    tvThrows(function () { a.subarray(0); },TypeError);
    a.constructor = {}; a.constructor[Symbol.species] = {};
    tvThrows(function () { a.subarray(0); },TypeError);
    return true;
  },
  species_accepts_short_different_kind_and_same_object: function () {
    tvGuard();
    var a = new Uint8Array([3,5,7,9]), owner = {}, chosen = new Float64Array([2.5]);
    a.constructor = owner;
    owner[Symbol.species] = function () { return chosen; };
    tvCheck(a.subarray(0,4) === chosen && chosen[0] === 2.5, 'short different Number-kind result');
    chosen = new Uint16Array(0);
    tvCheck(a.subarray(0,4) === chosen, 'zero-length custom result');
    chosen = a;
    tvCheck(a.subarray(1,2) === a && a.length === 4, 'source itself is legal result');
    chosen = {};
    tvThrows(function () { a.subarray(0); },TypeError);
    return true;
  },
  species_result_bounds_are_refreshed_after_constructor: function () {
    tvGuard();
    var a = new Uint8Array(2), owner = {}, b = new ArrayBuffer(4,{maxByteLength:8}), result = new Uint8Array(b,2,2);
    tvCheck(result.length === 2 && b.resizable, 'live result prerequisite');
    a.constructor = owner;
    owner[Symbol.species] = function () { b.resize(1); return result; };
    tvThrows(function () { a.subarray(0); },TypeError);
    tvCheck(result.length === 0, 'actual result became out of bounds');
    owner[Symbol.species] = function () { b.resize(4); return result; };
    tvCheck(a.subarray(0) === result && result.length === 2, 'revived result is accepted');
    owner[Symbol.species] = function () { b.transfer(); return result; };
    tvThrows(function () { a.subarray(0); },TypeError);
    tvCheck(b.detached, 'detached result rejected');
    return true;
  },
  tracking_and_fixed_species_arity: function () {
    tvGuard();
    var b = new ArrayBuffer(6,{maxByteLength:10}), tracking = new Uint8Array(b,2), fixed = new Uint8Array(b,2,4);
    var trace = '', owner = {};
    owner[Symbol.species] = function (buffer,offset,length) {
      tvCheck(buffer === b, 'original buffer passed');
      trace += arguments.length + ':' + offset + ':' + length + ';';
      return new Uint8Array(0);
    };
    tracking.constructor = owner; fixed.constructor = owner;
    tracking.subarray(1); tracking.subarray(1,undefined); fixed.subarray(1);
    tracking.subarray(1,{valueOf:function () { trace += 'E;'; return undefined; }});
    tvCheck(trace === '2:3:undefined;2:3:undefined;3:3:3;E;3:3:0;', 'literal two/three argument schedule');
    return true;
  },
  tracking_subviews_follow_growth_and_fixed_subviews_do_not: function () {
    tvGuard();
    var b = new ArrayBuffer(8,{maxByteLength:12}), a = new Uint8Array(b,2);
    a[2] = 31;
    var tracking = a.subarray(2), fixed = a.subarray(2,4);
    tvCheck(tracking.byteOffset === 4 && tracking.length === 4 && fixed.length === 2, 'initial view geometry');
    b.resize(12);
    tvCheck(tracking.length === 8 && fixed.length === 2 && tracking[0] === 31, 'growth tracks omitted end only');
    b.resize(3);
    tvCheck(tracking.length === 0 && fixed.length === 0, 'both views become out of bounds');
    b.resize(8);
    tvCheck(tracking.length === 4 && fixed.length === 2 && tracking[0] === 0 && fixed[0] === 0, 'revived zeroed storage');
    return true;
  },
  initially_out_of_bounds_sources_keep_stored_offset: function () {
    tvGuard();
    var b = new ArrayBuffer(8,{maxByteLength:8}), tracking = new Uint8Array(b,2), fixed = new Uint8Array(b,2,3), calls = 0;
    var begin = {valueOf:function () { calls++; b.resize(8); return 1; }};
    b.resize(0);
    tvCheck(tracking.length === 0 && tracking.byteOffset === 0, 'public out-of-bounds getters');
    var one = tracking.subarray(begin);
    tvCheck(one.byteOffset === 2 && one.length === 6, 'stored offset and captured zero length');
    b.resize(0);
    var two = fixed.subarray(begin);
    tvCheck(two.byteOffset === 2 && two.length === 0 && calls === 2, 'fixed source captured zero length');
    return true;
  },
  captured_source_length_and_fresh_default_bounds: function () {
    tvGuard();
    var b = new ArrayBuffer(4,{maxByteLength:8}), a = new Uint8Array(b);
    var grown = a.subarray({valueOf:function () { b.resize(8); return 1; }},6);
    tvCheck(grown.byteOffset === 1 && grown.length === 3, 'end clamps against original four');
    b.resize(4);
    tvThrows(function () { a.subarray({valueOf:function () { b.resize(2); return 0; }},4); },RangeError);
    tvCheck(b.byteLength === 2, 'completed callback retained');
    var valid = new Float32Array([1.5]);
    a.constructor = {}; a.constructor[Symbol.species] = function () { b.transfer(); return valid; };
    tvCheck(a.subarray(0,1) === valid && b.detached, 'valid species result survives source detachment');
    return true;
  },
  detached_source_can_reach_custom_species: function () {
    tvGuard();
    var buffer = new ArrayBuffer(4), a = new Uint8Array(buffer,2), chosen = new Uint16Array([29]), trace = '';
    a.constructor = {};
    a.constructor[Symbol.species] = function (b,offset,length) {
      trace += 'N';
      tvCheck(b === buffer && offset === 2 && length === 0 && arguments.length === 3, 'detached source stored arguments');
      return chosen;
    };
    buffer.transfer();
    tvCheck(buffer.detached && a.length === 0, 'actual detach prerequisite');
    var result = a.subarray({valueOf:function () { trace += 'B'; return 2; }},3);
    tvCheck(result === chosen && trace === 'BN', 'subarray does not eagerly validate source bounds');
    return true;
  },
  bound_species_is_constructed_with_original_target: function () {
    tvGuard();
    var a = new Uint8Array([11,13,17]), calls = 0;
    function Species(buffer,offset,length) {
      calls++; tvCheck(new.target === Species && arguments.length === 3, 'bound construct target');
      return new Uint8Array(buffer,offset,length);
    }
    var bound = Species.bind(null);
    a.constructor = {}; a.constructor[Symbol.species] = bound;
    var result = a.subarray(1,3);
    tvCheck(calls === 1 && result.buffer === a.buffer && result[0] === 13 && result[1] === 17, 'bound species result');
    return true;
  },
  join_all_kinds_and_literal_number_strings: function () {
    tvGuard();
    var kinds = tvKinds();
    for (var i = 0; i < kinds.length; i++) {
      var a = new kinds[i]([1,2,3]);
      tvCheck(a.join() === '1,2,3' && a.join('/') === '1/2/3' && a.join('') === '123', 'all ten join kinds');
    }
    var values = new Float64Array([NaN,-0,Infinity,-Infinity,1e21,1000000000000000.25]);
    tvCheck(values.join('|') === 'NaN|0|Infinity|-Infinity|1e+21|1000000000000000.2', 'literal Number formatting');
    return true;
  },
  join_validates_receiver_before_separator: function () {
    var p = tvGuard(), calls = 0, separator = {toString:function () { calls++; return '|'; }};
    var b = new ArrayBuffer(4,{maxByteLength:8}), fixed = new Uint8Array(b,2,2);
    tvCheck(fixed.length === 2, 'initial valid view'); b.resize(1);
    tvThrows(function () { p.join.call(fixed,separator); },TypeError);
    var d = new ArrayBuffer(1), detached = new Uint8Array(d); d.transfer();
    tvCheck(d.detached, 'actual detachment');
    tvThrows(function () { p.join.call(detached,separator); },TypeError);
    tvThrows(function () { p.join.call({length:2,0:1},separator); },TypeError);
    tvThrows(function () { p.join.call(undefined,separator); },TypeError);
    tvCheck(calls === 0, 'no separator callbacks for invalid initial receiver');
    return true;
  },
  join_separator_coercion_once_even_when_empty: function () {
    tvGuard();
    var a = new Uint8Array([4,6]), empty = new Uint8Array(0), count = 0, sep = {}, reason = {};
    sep[Symbol.toPrimitive] = function (hint) { tvCheck(this === sep && hint === 'string','separator receiver/hint'); count++; return '~'; };
    sep.valueOf = function () { throw new Error('unexpected valueOf'); };
    tvCheck(a.join(sep) === '4~6' && empty.join(sep) === '' && count === 2, 'once for each call including empty');
    tvCheck(a.join(undefined) === '4,6' && a.join(null) === '4null6', 'default and explicit null');
    tvThrows(function () { empty.join(Symbol('separator')); },TypeError);
    tvAbrupt(function () { a.join({toString:function () { throw reason; }}); },reason);
    return true;
  },
  join_captures_length_but_refreshes_each_element: function () {
    tvGuard();
    var b = new ArrayBuffer(4,{maxByteLength:8}), a = new Uint8Array(b);
    a[0] = 7; a[1] = 8; a[2] = 9; a[3] = 10;
    tvCheck(a.join({toString:function () { b.resize(2); return '|'; }}) === '7|8||', 'tracking shrink leaves empty fields');
    b.resize(4); var fixed = new Uint8Array(b,0,4);
    tvCheck(fixed.join({toString:function () { b.resize(2); return '|'; }}) === '|||', 'fixed-view OOB leaves all fields empty');
    a[0] = 11; a[1] = 12;
    tvCheck(a.join({toString:function () { b.resize(4); a[0] = 21; a[2] = 99; return ':'; }}) === '21:12', 'growth does not extend captured loop');
    b.resize(3);
    tvCheck(a.join({toString:function () { b.transfer(); return '-'; }}) === '--' && b.detached, 'mid-call detach produces empty fields');
    return true;
  },
  join_ignores_shadow_length_and_invalid_index_prototypes: function () {
    var p = tvGuard(), b = new ArrayBuffer(3,{maxByteLength:3}), a = new Uint8Array(b), calls = 0;
    a[0] = 4; a[1] = 5; a[2] = 6;
    Object.defineProperty(a,'length',{get:function () { calls++; throw new Error('own length'); }});
    Object.defineProperty(p,'2',{get:function () { calls++; throw new Error('inherited numeric index'); },configurable:true});
    tvCheck(p.join.call(a,{toString:function () { b.resize(1); return ','; }}) === '4,,', 'internal length and integer-indexed absence');
    tvCheck(calls === 0, 'poison accessors not reached');
    return true;
  },
  join_retains_exact_utf16_separator_units: function () {
    tvGuard();
    var result = new Uint8Array([1,2,3]).join('\ud800X\udfff');
    var expected = [49,55296,88,57343,50,55296,88,57343,51];
    tvCheck(result.length === 9, 'literal UTF16 length');
    for (var i = 0; i < expected.length; i++) tvCheck(result.charCodeAt(i) === expected[i], 'literal UTF16 unit');
    tvCheck(result === '1\ud800X\udfff2\ud800X\udfff3', 'literal lone-surrogate preservation');
    return true;
  },
  to_string_alias_is_generic_and_returns_join_result: function () {
    var p = tvGuard(), saved = p.toString, token = {}, seen = '', object = {};
    Object.defineProperty(object,'join',{get:function () {
      tvCheck(this === object,'join getter receiver'); seen += 'G';
      return function () { tvCheck(this === object && arguments.length === 0,'join call receiver/arity'); seen += 'C'; return token; };
    }});
    Array.prototype.toString = null;
    tvCheck(saved.call(object) === token && seen === 'GC', 'saved alias does not coerce custom join result');
    tvCheck(p.toString === saved, 'original shared function identity retained');
    var reason = {};
    Object.defineProperty(object,'other',{value:1});
    var abrupt = {}; Object.defineProperty(abrupt,'join',{get:function () { throw reason; }});
    tvAbrupt(function () { saved.call(abrupt); },reason);
    return true;
  },
  to_string_fallback_and_detached_custom_join: function () {
    var p = tvGuard(), saved = p.toString, object = {}, trace = '';
    Object.defineProperty(object,'join',{get:function () { trace += 'J'; return 17; }});
    Object.defineProperty(object,Symbol.toStringTag,{get:function () { trace += 'T'; return 'Card'; }});
    Object.prototype.toString = function () { return 'wrong mutable property'; };
    tvCheck(saved.call(object) === '[object Card]' && trace === 'JT', 'intrinsic fallback and tag order');
    var buffer = new ArrayBuffer(2), a = new Uint8Array(buffer); buffer.transfer();
    tvCheck(buffer.detached, 'actual detached prerequisite');
    tvThrows(function () { saved.call(a); },TypeError);
    a.join = function () { tvCheck(this === a && arguments.length === 0,'custom detached receiver'); return 'detached custom'; };
    tvCheck(saved.call(a) === 'detached custom', 'alias has no early TypedArray brand/bounds validation');
    tvThrows(function () { saved.call(null); },TypeError);
    tvThrows(function () { saved.call(undefined); },TypeError);
    return true;
  }
};
