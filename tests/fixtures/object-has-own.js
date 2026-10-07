// Independent literal expectations. Run each named body in a fresh realm.
// These additional Object.hasOwn groups do not alter the held clone fixture.
var objectHasOwnGlobalProbe = 17;
let objectHasOwnLexicalProbe = 19;
function objectHasOwnGlobalFunctionProbe() { return 23; }
function ownCheck(value, message) { if (!value) throw new Error(message); }
function ownPrerequisite() {
  const method = Object.hasOwn;
  if (typeof method !== 'function') throw new TypeError('Object.hasOwn callable prerequisite');
  const parent = {inherited: 1}, target = Object.create(parent);
  target.own = undefined;
  if (method(target, 'own') !== true || method(target, 'inherited') !== false || method(target, 'missing') !== false)
    throw new TypeError('working own and inherited distinction prerequisite');
  return method;
}
var objectHasOwnCases = {
  ordinary_metadata_saved_method_and_ignored_this: function() {
    const method = ownPrerequisite(), outer = Object.getOwnPropertyDescriptor(Object, 'hasOwn');
    const name = Object.getOwnPropertyDescriptor(method, 'name'), length = Object.getOwnPropertyDescriptor(method, 'length');
    ownCheck(outer.value === method && outer.writable && !outer.enumerable && outer.configurable, 'ordinary static descriptor');
    ownCheck(name.value === 'hasOwn' && !name.writable && !name.enumerable && name.configurable, 'method name descriptor');
    ownCheck(length.value === 2 && !length.writable && !length.enumerable && length.configurable, 'method length descriptor');
    ownCheck(Object.getPrototypeOf(method) === Function.prototype && Object.getOwnPropertyDescriptor(method, 'prototype') === undefined, 'nonconstructor metadata');
    ownCheck(method.call(null, {x: 1}, 'x') === true && method.call(undefined, {}, 'x') === false, 'static receiver ignored');
    if (typeof Reflect.construct !== 'function') throw new TypeError('Reflect.construct prerequisite');
    function Guard() { this.ok = 1; }
    if (Reflect.construct(Guard, []).ok !== 1) throw new TypeError('working construction prerequisite');
    let caught = false;
    try { Reflect.construct(method, []); } catch (error) { caught = error instanceof TypeError; }
    ownCheck(caught, 'Object.hasOwn is not constructable');
    try {
      Object.hasOwn = function() { return 'replacement'; };
      ownCheck(Object.hasOwn() === 'replacement' && method({x: 1}, 'x') === true, 'saved function after replacement');
      ownCheck(delete Object.hasOwn, 'configurable deletion');
      ownCheck(Object.hasOwn === undefined && method({x: 1}, 'x') === true, 'no resurrection and saved call');
    } finally { Object.defineProperty(Object, 'hasOwn', outer); }
    ownCheck(Object.hasOwn === method, 'restoration');
    return true;
  },
  target_before_key_conversion_and_fresh_lookup: function() {
    const method = ownPrerequisite(), token = {}, target = {taken: 1};
    let trace = '', expressions = 0;
    const key = { [Symbol.toPrimitive]: function(hint) { trace += hint; throw token; } };
    for (const receiver of [null, undefined]) {
      let caught = false;
      try { method(receiver, key); } catch (error) { caught = error instanceof TypeError; }
      ownCheck(caught && trace === '', 'nullish target refuses before key hook');
    }
    let caught = false;
    try { method(); } catch (error) { caught = error instanceof TypeError; }
    ownCheck(caught, 'missing target is undefined');
    caught = false;
    try { method(null, (expressions += 1, key), expressions += 1); } catch (error) { caught = error instanceof TypeError; }
    ownCheck(caught && expressions === 2 && trace === '', 'argument expressions finish before conversion');
    caught = false;
    try { method(target, key); } catch (error) { caught = error === token; }
    ownCheck(caught && trace === 'string', 'key string hint once and thrown identity');
    trace = '';
    const add = { [Symbol.toPrimitive]: function(hint) { trace += hint; target.made = undefined; return 'made'; } };
    ownCheck(method(target, add) === true && trace === 'string', 'lookup observes key callback addition');
    const remove = { toString: function() { delete target.taken; return 'taken'; }, valueOf: function() { throw token; } };
    ownCheck(method(target, remove) === false, 'lookup observes key callback deletion without fallback');
    trace = '';
    const fallback = { toString: function() { trace += 's'; return {}; }, valueOf: function() { trace += 'v'; return 'made'; } };
    ownCheck(method(target, fallback) === true && trace === 'sv', 'ordinary string-hint fallback order');
    ownCheck(method({undefined: 1}) === true, 'missing key converts to undefined string');
    return true;
  },
  descriptor_presence_without_getters_or_target_methods: function() {
    const method = ownPrerequisite(), token = {};
    let reads = 0;
    const parent = {inherited: 1};
    Object.defineProperty(parent, 'inheritedGetter', {get: function() { reads += 1; throw token; }, configurable: true});
    const target = Object.create(parent);
    Object.defineProperty(target, 'getter', {get: function() { reads += 1; throw token; }, configurable: true});
    Object.defineProperty(target, 'setter', {set: function() { reads += 1; throw token; }, configurable: true});
    Object.defineProperty(target, 'hidden', {value: undefined});
    Object.defineProperty(target, 'hasOwnProperty', {get: function() { reads += 1; throw token; }, configurable: true});
    target.toString = function() { reads += 1; throw token; };
    target.valueOf = function() { reads += 1; throw token; };
    target[Symbol.toPrimitive] = function() { reads += 1; throw token; };
    ownCheck(method(target, 'getter') === true && method(target, 'setter') === true && method(target, 'hidden') === true, 'own accessor and nonenumerable presence');
    ownCheck(method(target, 'hasOwnProperty') === true && method(target, 'inherited') === false && method(target, 'inheritedGetter') === false && reads === 0, 'no getter target conversion or prototype lookup');
    const bare = Object.create(null); bare.x = null;
    ownCheck(method(bare, 'x') === true && method(bare, 'hasOwnProperty') === false, 'null prototype and null own value');
    return true;
  },
  primitive_wrappers_and_exact_string_indices: function() {
    const method = ownPrerequisite(), text = 'A\uD83D\uDE80\uD800';
    ownCheck(method(text, 'length') === true, 'primitive string length is own');
    for (const index of [0, 1, 2, 3]) ownCheck(method(text, index) === true, 'each UTF16 index is own');
    ownCheck(method(text, 4) === false && method(text, '-0') === false && method(text, '01') === false && method(text, '1.0') === false, 'nonindices and out of range absent');
    ownCheck(method(text, -0) === true && method('', 0) === false && method('', 'length') === true, 'numeric minus zero and empty string');
    ownCheck(method(Object(text), '3') === true && method(Object(text), '4') === false, 'boxed exact string');
    for (const value of [false, true, 0, NaN, Symbol('primitive')]) {
      ownCheck(method(value, 'valueOf') === false && method(value, 'toString') === false && method(value, 'length') === false, 'wrapper does not own inherited names');
    }
    ownCheck(method(text, 'constructor') === false, 'string constructor is inherited');
    return true;
  },
  array_holes_accessors_and_noncanonical_names: function() {
    const method = ownPrerequisite(), array = [undefined, , 3];
    let reads = 0;
    const parent = Object.create(Array.prototype);
    Object.defineProperty(parent, '1', {get: function() { reads += 1; throw new Error('inherited getter'); }, configurable: true});
    Object.setPrototypeOf(array, parent);
    ownCheck(method(array, 'length') === true && method(array, 0) === true && method(array, 1) === false && method(array, 2) === true && method(array, 3) === false, 'array length present undefined and inherited hole');
    Object.defineProperty(array, '1', {get: function() { reads += 1; throw new Error('own getter'); }, configurable: true});
    ownCheck(method(array, 1) === true && reads === 0, 'own index descriptor without evaluation');
    delete array[0]; array['01'] = 'named';
    ownCheck(method(array, 0) === false && method(array, '01') === true && method(array, '1.0') === false && reads === 0, 'deleted hole and distinct named key');
    return true;
  },
  symbols_exact_keys_and_function_metadata: function() {
    const method = ownPrerequisite(), one = Symbol('same'), two = Symbol('same'), target = {};
    target[one] = undefined; target['\uD800'] = 1; target['A\u0000B'] = 2;
    ownCheck(method(target, one) === true && method(target, two) === false && method(target, Object(one)) === true, 'Symbol identity including boxed key conversion');
    ownCheck(method(target, '\uD800') === true && method(target, '\uFFFD') === false && method(target, 'A\u0000B') === true && method(target, 'A') === false, 'exact nonscalar and embedded NUL keys');
    function ordinary(a) { return a; }
    ownCheck(method(ordinary, 'name') === true && method(ordinary, 'length') === true && method(ordinary, 'prototype') === true && method(ordinary, 'call') === false, 'ordinary function own metadata');
    ownCheck(method(method, 'name') === true && method(method, 'length') === true && method(method, 'prototype') === false && method(method, 'call') === false, 'native function own metadata');
    ordinary[one] = 9; ownCheck(method(ordinary, one) === true && method(ordinary, two) === false, 'function Symbol expando');
    return true;
  },
  dom_expandos_are_separate_from_inherited_members: function() {
    const method = ownPrerequisite(), text = new Text('exact'), element = document.createElement('div'), symbol = Symbol('node');
    let reads = 0;
    ownCheck(method(text, 'data') === false && method(text, 'nodeValue') === false && method(text, 'cloneNode') === false, 'ordinary Node members are inherited');
    Object.defineProperty(text, 'note', {get: function() { reads += 1; throw new Error('DOM own getter'); }, configurable: true});
    text[symbol] = 7; element.id = 'attribute';
    ownCheck(method(text, 'note') === true && method(text, symbol) === true && reads === 0, 'DOM own accessor and Symbol without reads');
    ownCheck(method(element, 'id') === false && method(element, 'missing') === false, 'reflected attribute and absent name are not own JS descriptors');
    document.objectHasOwnNote = undefined;
    ownCheck(method(document, 'objectHasOwnNote') === true && method(document, 'title') === false, 'Document expando versus prototype member');
    delete text.note; delete text[symbol];
    ownCheck(method(text, 'note') === false && method(text, symbol) === false && reads === 0, 'DOM deletions are current');
    return true;
  },
  window_global_properties_are_not_lexical_bindings: function() {
    const method = ownPrerequisite(), symbol = Symbol('window');
    let reads = 0;
    ownCheck(method(window, 'objectHasOwnGlobalProbe') === true && method(window, 'objectHasOwnGlobalFunctionProbe') === true, 'global var and function properties');
    ownCheck(objectHasOwnLexicalProbe === 19 && method(window, 'objectHasOwnLexicalProbe') === false, 'lexical binding is not an own Window property');
    Object.defineProperty(window, 'objectHasOwnAccessorProbe', {get: function() { reads += 1; throw new Error('Window getter'); }, configurable: true});
    Object.defineProperty(window, '\uD800window', {value: 1, configurable: true});
    window[symbol] = 3;
    ownCheck(method(window, 'objectHasOwnAccessorProbe') === true && method(window, '\uD800window') === true && method(window, '\uFFFDwindow') === false && method(window, symbol) === true && reads === 0, 'Window descriptor Symbol and exact string routes');
    delete window.objectHasOwnAccessorProbe; delete window['\uD800window']; delete window[symbol];
    ownCheck(method(window, 'objectHasOwnAccessorProbe') === false && method(window, symbol) === false && reads === 0, 'current deleted Window properties');
    return true;
  }
};
