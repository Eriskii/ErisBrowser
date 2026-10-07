// Source-only independent TypedArray foundation expectations.
// One named body per fresh realm; prepend strict directive for strict mode.
function taCheck(ok, label) { if (!ok) throw new Error(label); }
function taTypeError(fn, label) {
  var seen = false;
  try { fn(); } catch (e) { if (e instanceof TypeError) seen = true; else throw e; }
  taCheck(seen, label);
}
function taRangeError(fn, label) {
  var seen = false;
  try { fn(); } catch (e) { if (e instanceof RangeError) seen = true; else throw e; }
  taCheck(seen, label);
}
function taGuard() {
  if (typeof Uint8Array !== 'function' || typeof Float16Array !== 'function')
    throw new TypeError('Number TypedArray prerequisites');
  var a = new Uint8Array(1); a[0] = 23;
  if (a[0] !== 23 || a.length !== 1 || a.byteLength !== 1 || !ArrayBuffer.isView(a))
    throw new TypeError('authentic indexed view prerequisite');
  return a;
}
var typedArrayFoundationCases = {
  all_ten_number_kinds: function () {
    taGuard();
    var cs = [Int8Array, Uint8Array, Uint8ClampedArray, Int16Array, Uint16Array,
      Int32Array, Uint32Array, Float16Array, Float32Array, Float64Array];
    var widths = [1,1,1,2,2,4,4,2,4,8];
    var expected = [1,1,255,257,257,257,257,257.75,257.75,257.75];
    for (var i = 0; i < 10; i++) {
      var C = cs[i], a = new C(1); a[0] = 257.75;
      taCheck(a[0] === expected[i] && a.length === 1 && a.byteLength === widths[i], 'ten literal conversions');
      taCheck(C.BYTES_PER_ELEMENT === widths[i] && C.prototype.BYTES_PER_ELEMENT === widths[i], 'element widths');
      taCheck(ArrayBuffer.isView(a) && !Array.isArray(a), 'view rather than Array');
    }
    return true;
  },
  clamp_and_float16_ties: function () {
    taGuard();
    var c = new Uint8ClampedArray([-1,0.5,1.5,2.5,254.5,255.5,NaN,Infinity]);
    var ce = [0,0,2,2,254,255,0,255];
    for (var i = 0; i < 8; i++) taCheck(c[i] === ce[i], 'Uint8Clamp tie or saturation');
    var f = new Float16Array([1.00048828125,1.0004883110523224,1.0004882514476776,
      1.00146484375,0.0000000298023223876953125,0.000000059604644775390625,65520,-0]);
    var fe = [1,1.0009765625,1,1.001953125,0,0.000000059604644775390625,Infinity,-0];
    for (var j = 0; j < 8; j++) taCheck(Object.is(f[j], fe[j]), 'direct binary64 to Float16 rounding');
    return true;
  },
  same_kind_preserves_nan_payload: function () {
    taGuard();
    var b = new ArrayBuffer(8), d = new DataView(b);
    d.setUint32(0, 305419896, true); d.setUint32(4, 2146959361, true);
    var a = new Float64Array(b), c = new Float64Array(a), cd = new DataView(c.buffer);
    taCheck(a[0] !== a[0] && c[0] !== c[0], 'NaN prerequisite');
    taCheck(c.buffer !== b && cd.getUint32(0,true) === 305419896 && cd.getUint32(4,true) === 2146959361, 'raw same-kind NaN copy');
    d.setUint32(0,0,true); taCheck(cd.getUint32(0,true) === 305419896, 'independent copied backing');
    return true;
  },
  primitive_lengths_and_call_refusal: function () {
    taGuard();
    taCheck(new Uint16Array().length === 0 && new Uint16Array(undefined).length === 0, 'empty branches');
    taCheck(new Uint16Array(null).length === 0 && new Uint16Array(true).length === 1, 'primitive ToIndex');
    taCheck(new Uint16Array('2').length === 2 && new Uint16Array(2.9).length === 2, 'string length is not iterable');
    taRangeError(function () { new Uint16Array(-1); }, 'negative length');
    taRangeError(function () { new Uint16Array(Infinity); }, 'infinite length');
    taTypeError(function () { Uint16Array(1); }, 'new required');
    return true;
  },
  iterable_collected_before_conversion: function () {
    taGuard(); var log = '', src = {}, i = 0;
    var first = {valueOf:function () { log += 'a'; return 7; }};
    var second = {valueOf:function () { log += 'b'; return 9; }};
    Object.defineProperty(src, Symbol.iterator, {get:function () { log += 'g'; return function () {
      log += 'i'; return {next:function () { log += 'n'; i++; return i === 1 ? {value:first,done:false} : i === 2 ? {value:second,done:false} : {done:true}; }};
    }; }});
    var a = new Uint8Array(src);
    taCheck(log === 'ginnnab' && a[0] === 7 && a[1] === 9, 'exhaust then convert');
    return true;
  },
  arraylike_interleaves_get_and_convert: function () {
    taGuard(); var log = '', src = {};
    Object.defineProperty(src, Symbol.iterator, {get:function () { log += 'i'; return null; }});
    Object.defineProperty(src, 'length', {get:function () { log += 'l'; return 2; }});
    Object.defineProperty(src, '0', {get:function () { log += '0'; return {valueOf:function () { log += 'a'; return 7; }}; }});
    Object.defineProperty(src, '1', {get:function () { log += '1'; return {valueOf:function () { log += 'b'; return 9; }}; }});
    src.valueOf = function () { throw new Error('object must not become primitive length'); };
    var a = new Uint8Array(src);
    taCheck(log === 'il0a1b' && a[0] === 7 && a[1] === 9, 'array-like order');
    return true;
  },
  typed_source_ignores_authored_lookups: function () {
    taGuard(); var a = new Uint16Array([257,258]), calls = 0;
    function poison() { calls++; throw new Error('typed source lookup'); }
    Object.defineProperty(a, Symbol.iterator, {get:poison});
    Object.defineProperty(a, 'length', {get:poison});
    Object.defineProperty(a, 'constructor', {get:poison});
    var copy = new Uint8Array(a);
    taCheck(calls === 0 && copy.length === 2 && copy[0] === 1 && copy[1] === 2 && copy.buffer !== a.buffer, 'authentic different-kind copy');
    return true;
  },
  buffer_offsets_fixed_and_tracking: function () {
    taGuard(); var b = new ArrayBuffer(7,{maxByteLength:12});
    var track = new Uint16Array(b,2), fixed = new Uint16Array(b,2,2), empty = new Uint16Array(b,6,0);
    taCheck(track.length === 2 && fixed.length === 2 && empty.length === 0 && empty.byteOffset === 6, 'odd RAB suffix floors');
    fixed[0] = 258; var d = new DataView(b);
    taCheck(track[0] === 258 && d.getUint8(2) === 2 && d.getUint8(3) === 1, 'declared little-endian alias');
    b.resize(10); taCheck(track.length === 4 && fixed.length === 2 && empty.byteOffset === 6, 'tracking versus explicit length');
    taRangeError(function () { new Uint16Array(b,1); }, 'unaligned offset');
    taRangeError(function () { new Uint16Array(new ArrayBuffer(7)); }, 'unaligned fixed buffer length');
    return true;
  },
  shrink_oob_regrowth: function () {
    taGuard(); var b = new ArrayBuffer(8,{maxByteLength:12}), fixed = new Uint16Array(b,2,2), track = new Uint16Array(b,2);
    fixed[0] = 258; fixed[1] = 772; b.resize(4);
    taCheck(fixed.length === 0 && fixed.byteLength === 0 && fixed.byteOffset === 0 && fixed.buffer === b && fixed[0] === undefined, 'fixed out of bounds');
    taCheck(track.length === 1 && track[0] === 258 && !('0' in fixed), 'tracking remains valid');
    b.resize(8); taCheck(fixed.length === 2 && fixed.byteOffset === 2 && fixed[0] === 258 && fixed[1] === 0 && track.length === 3, 'restored bounds and zero regrowth');
    return true;
  },
  transfer_detaches_retained_views: function () {
    taGuard(); var b = new ArrayBuffer(4,{maxByteLength:8}), a = new Uint8Array(b); a[0] = 37;
    var before = a.values(); taCheck(before.next().value === 37, 'genuine values method before detachment');
    var next = b.transfer(6), c = new Uint8Array(next);
    taCheck(b.detached && a.buffer === b && ArrayBuffer.isView(a), 'detached identity remains view');
    taCheck(a.length === 0 && a.byteLength === 0 && a.byteOffset === 0 && a[0] === undefined && !Object.hasOwn(a,'0'), 'detached indices absent');
    taCheck(c.length === 6 && c[0] === 37 && c[4] === 0 && next.resizable, 'transferred bytes and growth');
    taTypeError(function () { new Uint8Array(b); }, 'detached buffer constructor');
    taTypeError(function () { a.values(); }, 'detached iterator constructor');
    return true;
  },
  alternative_prototype_and_conversion_order: function () {
    taGuard(); var proto = {}, log = '', b = new ArrayBuffer(8,{maxByteLength:16});
    function Alternate() {} Alternate.prototype = proto;
    var a = Reflect.construct(Uint16Array,[b,{valueOf:function () { log += 'o'; return 2; }},{valueOf:function () { log += 'l'; b.resize(10); return 3; }}],Alternate);
    var shared = Object.getPrototypeOf(Uint16Array.prototype), lengthGet = Object.getOwnPropertyDescriptor(shared,'length').get;
    taCheck(Object.getPrototypeOf(a) === proto && lengthGet.call(a) === 3 && a[0] === 0 && log === 'ol', 'alternate prototype with live offset and length conversion');
    taTypeError(function () { lengthGet.call(proto); }, 'unbranded prototype');
    return true;
  },
  recursive_collection_keeps_records_searchable: function () {
    taGuard(); var inner, src = {}, step = 0;
    src[Symbol.iterator] = function () { inner = new Uint16Array([513]); return {next:function () { step++; return step === 1 ? {value:7,done:false} : {done:true}; }}; };
    var outer = new Uint8Array(src);
    taCheck(outer[0] === 7 && outer.length === 1 && inner[0] === 513 && inner.length === 1, 'outer and callback-created records');
    taCheck(outer.buffer !== inner.buffer && ArrayBuffer.isView(inner) && ArrayBuffer.isView(outer), 'both authentic identities searchable');
    inner[0] = 1027; outer[0] = 9; taCheck(inner[0] === 1027 && outer[0] === 9, 'later independent stores');
    return true;
  },
  canonical_numeric_keys_stop_prototype: function () {
    taGuard(); var a = new Uint8Array([7]), proto = {}, hits = 0;
    var invalid = ['-0','NaN','Infinity','-Infinity','1.5','1','9007199254740992','1e+21','1e-7'];
    for (var i=0;i<invalid.length;i++) Object.defineProperty(proto,invalid[i],{get:function () { hits++; return 99; },configurable:true});
    proto['01'] = 11; proto['1.0'] = 12; proto['+0'] = 13;
    Object.setPrototypeOf(a,proto);
    for (var j=0;j<invalid.length;j++) taCheck(a[invalid[j]] === undefined && !(invalid[j] in a), 'invalid canonical key stops');
    taCheck(hits === 0 && a['01'] === 11 && a['1.0'] === 12 && a['+0'] === 13 && a[-0] === 7, 'ordinary noncanonical keys and numeric negative zero');
    return true;
  },
  indexed_set_converts_before_live_bounds: function () {
    taGuard(); var b = new ArrayBuffer(0,{maxByteLength:4}), a = new Uint8Array(b), calls = 0;
    a[0] = {valueOf:function () { calls++; b.resize(2); return 257; }};
    taCheck(calls === 1 && a[0] === 1 && a.length === 2, 'out-of-bounds becomes in-bounds during conversion');
    a[0] = {valueOf:function () { calls++; b.resize(0); return 9; }};
    taCheck(calls === 2 && a[0] === undefined && a.length === 0, 'in-bounds becomes out-of-bounds');
    a['-0'] = {valueOf:function () { calls++; return 9; }};
    taCheck(calls === 3 && !Object.hasOwn(a,'-0'), 'invalid same receiver still converts');
    return true;
  },
  distinct_receiver_no_numeric_conversion: function () {
    taGuard(); var a = new Uint8Array([7]), child = Object.create(a), calls = 0;
    var value = {valueOf:function () { calls++; return 9; }};
    child[0] = value; child[1] = value; child['-0'] = value;
    taCheck(calls === 0 && child[0] === value && a[0] === 7, 'ordinary receiver valid index property');
    taCheck(Object.hasOwn(child,'0') && !Object.hasOwn(child,'1') && !Object.hasOwn(child,'-0'), 'invalid distinct receiver does not create');
    return true;
  },
  indexed_descriptor_and_delete: function (strictMode) {
    taGuard(); var a = new Uint8Array([7]), d = Object.getOwnPropertyDescriptor(a,'0');
    taCheck(d.value === 7 && d.writable && d.enumerable && d.configurable, 'index descriptor flags');
    var caught = false, result;
    try { result = delete a[0]; } catch (e) { if (e instanceof TypeError) caught = true; else throw e; }
    taCheck(strictMode ? caught : !caught && result === false, 'strict/sloppy valid deletion');
    taCheck(a[0] === 7 && delete a[9] && delete a['-0'], 'invalid deletion succeeds');
    return true;
  },
  define_property_rejections_do_not_convert: function () {
    taGuard(); var a = new Uint8Array([7]), calls = 0, v = {valueOf:function () { calls++; return 257; }};
    Object.defineProperty(a,'0',{value:v,writable:true,enumerable:true,configurable:true});
    taCheck(a[0] === 1 && calls === 1, 'accepted compatible descriptor');
    taTypeError(function () { Object.defineProperty(a,'0',{value:v,configurable:false}); }, 'cannot make index nonconfigurable');
    taTypeError(function () { Object.defineProperty(a,'0',{value:v,writable:false}); }, 'cannot make index readonly');
    taTypeError(function () { Object.defineProperty(a,'1',{value:v}); }, 'invalid index rejected');
    taTypeError(function () { Object.defineProperty(a,'0',{get:function () { return 9; }}); }, 'index accessor rejected');
    taCheck(calls === 1 && a[0] === 1, 'rejected definitions do not coerce value');
    return true;
  },
  own_keys_enumeration_and_generic_consumers: function () {
    taGuard(); var a = new Uint8Array([7,9]), sym = Symbol('extra'); a.z = 11; a['01'] = 13; a[sym] = 17;
    var keys = Reflect.ownKeys(a);
    taCheck(keys.length === 5 && keys[0] === '0' && keys[1] === '1' && keys[2] === 'z' && keys[3] === '01' && keys[4] === sym, 'indices before ordinary strings before symbol');
    taCheck(Object.keys(a).join(',') === '0,1,z,01' && Object.values(a).join(',') === '7,9,11,13', 'own enumerable walkers');
    var list = ''; for (var k in a) list += k + ',';
    taCheck(list === '0,1,z,01,' && JSON.stringify(a) === '{"0":7,"1":9,"z":11,"01":13}', 'for-in and JSON');
    taCheck(Array.prototype.map.call(a,function (v) { return v+1; }).join(',') === '8,10', 'generic Array indexed reads');
    return true;
  },
  integrity_resizable_and_fixed: function () {
    taGuard(); var b = new ArrayBuffer(4,{maxByteLength:8}), fixed = new Uint8Array(b,0,2), tracking = new Uint8Array(b);
    var ordinary = {}; Object.preventExtensions(ordinary); taCheck(!Object.isExtensible(ordinary), 'ordinary preventExtensions prerequisite');
    var empty = new Uint8Array(0); taCheck(Object.freeze(empty) === empty && Object.isFrozen(empty), 'empty fixed freeze prerequisite');
    taTypeError(function () { Object.preventExtensions(fixed); }, 'fixed view of RAB cannot prevent extensions');
    taTypeError(function () { Object.preventExtensions(tracking); }, 'tracking RAB cannot prevent extensions');
    taCheck(Object.isExtensible(fixed) && Object.isExtensible(tracking), 'RAB refusal retains extensibility');
    var plain = new Uint8Array([7]); taTypeError(function () { Object.freeze(plain); }, 'nonempty fixed freeze fails');
    taCheck(!Object.isExtensible(plain) && !Object.isFrozen(plain) && plain[0] === 7, 'freeze partial preventExtensions effect');
    return true;
  },
  getter_brand_tag_and_isview: function () {
    var a = taGuard(), shared = Object.getPrototypeOf(Uint8Array.prototype), names = ['buffer','byteLength','byteOffset','length'];
    for (var i=0;i<4;i++) {
      var d = Object.getOwnPropertyDescriptor(shared,names[i]);
      taCheck(typeof d.get === 'function' && d.set === undefined && !d.enumerable && d.configurable && d.get.length === 0, 'getter metadata');
      taCheck(d.get.call(a) === (i===0 ? a.buffer : i===2 ? 0 : 1), 'saved genuine getter');
      taTypeError(function () { d.get.call(Object.create(Uint8Array.prototype)); }, 'forged getter brand');
    }
    var tag = Object.getOwnPropertyDescriptor(shared,Symbol.toStringTag).get;
    taCheck(tag.call(a) === 'Uint8Array' && tag.call({}) === undefined && Object.prototype.toString.call(a) === '[object Uint8Array]', 'tag exceptional brand convention');
    taCheck(ArrayBuffer.isView(new DataView(new ArrayBuffer(0))) && !ArrayBuffer.isView(a.buffer) && !ArrayBuffer.isView(Object.create(Uint8Array.prototype)), 'isView authentic variants');
    return true;
  },
  constructor_metadata_and_saved_global: function () {
    taGuard(); var C = Float16Array, shared = Object.getPrototypeOf(C), p = Object.getPrototypeOf(C.prototype);
    taCheck(C.name === 'Float16Array' && C.length === 3 && shared.name === 'TypedArray' && shared.length === 0, 'constructor names and lengths');
    taCheck(shared.prototype === p && p.constructor === shared && C.prototype.constructor === C, 'intrinsic hierarchy');
    var species = Object.getOwnPropertyDescriptor(shared,Symbol.species), marker = {};
    taCheck(species.get.call(marker) === marker && !species.enumerable && species.configurable && species.set === undefined, 'generic species');
    var bp = Object.getOwnPropertyDescriptor(C,'BYTES_PER_ELEMENT');
    taCheck(bp.value === 2 && !bp.writable && !bp.enumerable && !bp.configurable, 'constant descriptor');
    taTypeError(function () { new shared(); }, 'abstract constructor refuses');
    var globalDescriptor = Object.getOwnPropertyDescriptor(globalThis,'Float16Array');
    taCheck(globalDescriptor.value === C && globalDescriptor.writable && !globalDescriptor.enumerable && globalDescriptor.configurable, 'global descriptor');
    delete globalThis.Float16Array;
    taCheck(typeof globalThis.Float16Array === 'undefined' && new C([1.5])[0] === 1.5, 'saved intrinsic after deletion');
    return true;
  },
  iterators_alias_live_length_and_sticky_done: function () {
    taGuard(); var shared = Object.getPrototypeOf(Uint8Array.prototype), b = new ArrayBuffer(2,{maxByteLength:4}), a = new Uint8Array(b);
    a[0]=7; a[1]=9; taCheck(shared[Symbol.iterator] === shared.values && shared.keys.length === 0 && shared.entries.length === 0, 'iterator metadata and alias');
    Object.defineProperty(a,'length',{value:99});
    var values = a.values(), keys = a.keys(), entries = a.entries();
    taCheck(values.next().value === 7 && keys.next().value === 0 && entries.next().value.join(',') === '0,7', 'three iterator kinds');
    b.resize(3); a[2]=11;
    taCheck(values.next().value === 9 && values.next().value === 11 && values.next().done === true, 'live tracking length ignores own length');
    b.resize(4); a[3]=13; taCheck(values.next().done === true, 'completion remains sticky after growth');
    return true;
  },
  iterator_oob_detached_and_array_borrow: function () {
    taGuard(); var b = new ArrayBuffer(4,{maxByteLength:8}), a = new Uint8Array(b,0,4); a[0]=7;
    var it = a.values(); taCheck(it.next().value === 7, 'genuine iterator prerequisite');
    b.resize(2); taTypeError(function () { it.next(); }, 'fixed iterator OOB next');
    b.resize(4); taCheck(it.next().value === 0, 'iterator index not advanced by abrupt OOB');
    var borrowed = Array.prototype.values.call(a); Object.defineProperty(a,'length',{value:0});
    taCheck(borrowed.next().value === 7, 'Array iterator also uses authentic view length');
    b.transfer(); taTypeError(function () { borrowed.next(); }, 'Array iterator detached next');
    taTypeError(function () { Uint8Array.prototype.values.call({length:1,0:7}); }, 'typed iterator refuses generic receiver');
    return true;
  },
  exhausted_iterable_conversion_abrupt_no_close: function () {
    taGuard(); var log = '', src = {}, i = 0, token = {};
    src[Symbol.iterator] = function () { return {next:function () { log += 'n'; i++; return i===1 ? {done:false,value:{valueOf:function () { log += 'v'; throw token; }}} : {done:true}; },return:function () { log += 'r'; return {}; }}; };
    var caught; try { new Uint8Array(src); } catch (e) { caught=e; }
    taCheck(caught === token && log === 'nnv', 'no iterator close after exhausted list conversion');
    return true;
  },
  canonical_decimal_roundtrip_and_key_conversion: function () {
    taGuard(); var a = new Uint8Array([7]), once = 0, key = {};
    key[Symbol.toPrimitive] = function (hint) { once++; taCheck(hint === 'string','key hint'); return '0'; };
    taCheck(a[key] === 7 && once === 1, 'single ToPropertyKey');
    var ordinary = ['01','1.0','+0','9007199254740993','1000000000000000000000','0.0000001','1e-6'];
    for (var i=0;i<ordinary.length;i++) { a[ordinary[i]] = i+20; taCheck(a[ordinary[i]] === i+20 && Object.hasOwn(a,ordinary[i]), 'noncanonical roundtrip ordinary key'); }
    var invalid = ['9007199254740992','1e+21','1e-7','0.000001'];
    for (var j=0;j<invalid.length;j++) { a[invalid[j]]=99; taCheck(a[invalid[j]] === undefined && !Object.hasOwn(a,invalid[j]), 'canonical invalid numeric key'); }
    return true;
  },
  newtarget_getter_and_final_buffer_witness: function () {
    taGuard();
    function Base() {} var Alternate = Base.bind(null), marker = {}, log = '';
    var b = new ArrayBuffer(8,{maxByteLength:12});
    Object.defineProperty(Alternate,'prototype',{get:function () { log += 'p'; return marker; }});
    var a = Reflect.construct(Uint16Array,[b,{valueOf:function () { log += 'o'; return 2; }},{valueOf:function () { log += 'l'; b.resize(6); return 2; }}],Alternate);
    taCheck(log === 'pol' && Object.getPrototypeOf(a) === marker && a[0] === 0, 'prototype before buffer conversions');
    log = '';
    taTypeError(function () { Reflect.construct(Uint16Array,[Symbol('length')],Alternate); }, 'primitive conversion fails before allocation');
    taCheck(log === '', 'no prototype read after primitive failure');
    log = '';
    taTypeError(function () { Reflect.construct(Uint16Array,[b,{valueOf:function () { log += 'o'; return 0; }},{valueOf:function () { log += 'l'; b.transfer(); return 1; }}],Alternate); }, 'fresh detached witness after explicit length');
    taCheck(log === 'pol' && b.detached, 'callback effects survive constructor refusal');
    return true;
  },
  floating_alias_bytes_and_signed_zero: function () {
    taGuard(); var a = new Float32Array([1.5,-0,Infinity]), d = new DataView(a.buffer);
    taCheck(d.getUint32(0,true) === 1069547520 && d.getUint32(4,true) === 2147483648 && d.getUint32(8,true) === 2139095040, 'Float32 literal bytes');
    var f = new Float64Array([-0,-Infinity,NaN]);
    taCheck(Object.is(f[0],-0) && f[1] === -Infinity && f[2] !== f[2], 'Float64 exceptional numbers');
    return true;
  }
};
