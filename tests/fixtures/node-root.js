// Independent ordinary-tree getRootNode cases. Invoke one named body per fresh realm.
// DOM: https://dom.spec.whatwg.org/#dom-node-getrootnode
// IDL: https://webidl.spec.whatwg.org/#es-dictionary and #es-operations
function rootCheck(value, message) { if (!value) throw new Error(message); }
function rootTypeError(body, message) {
  let caught = false;
  try { body(); } catch (error) { caught = error instanceof TypeError; }
  rootCheck(caught, message);
}
function rootKeys(actual, expected) {
  rootCheck(actual.length === expected.length, 'key count');
  for (let i = 0; i < expected.length; i++) rootCheck(actual[i] === expected[i], 'literal key order');
}
function rootPrerequisite() {
  if (typeof Node !== 'function' || typeof Text !== 'function' || typeof DocumentFragment !== 'function')
    throw new TypeError('represented Node constructors');
  const method = Node.prototype.getRootNode;
  if (typeof method !== 'function') throw new TypeError('getRootNode callable');
  const leaf = new Text('guard'), fragment = new DocumentFragment();
  if (method.call(leaf) !== leaf || method.call(fragment) !== fragment)
    throw new TypeError('genuine detached roots');
  fragment.append(leaf);
  if (method.call(leaf) !== fragment || method.call(leaf, {composed: true}) !== fragment)
    throw new TypeError('genuine fragment roots');
  return method;
}
const nodeRootCases = {
  metadata_and_inherited_identity: function() {
    const method = rootPrerequisite(), descriptor = Object.getOwnPropertyDescriptor(Node.prototype, 'getRootNode');
    if (typeof Reflect.construct !== 'function') throw new TypeError('Reflect.construct prerequisite');
    const constructed = Reflect.construct(Text, ['guard']);
    if (constructed.data !== 'guard' || method.call(constructed) !== constructed)
      throw new TypeError('working Reflect.construct Text prerequisite');
    rootCheck(descriptor.value === method && descriptor.writable && descriptor.enumerable && descriptor.configurable, 'ordinary operation flags');
    rootCheck(descriptor.get === undefined && descriptor.set === undefined, 'data descriptor');
    const name = Object.getOwnPropertyDescriptor(method, 'name'), length = Object.getOwnPropertyDescriptor(method, 'length');
    rootCheck(name.value === 'getRootNode' && !name.writable && !name.enumerable && name.configurable, 'name flags');
    rootCheck(length.value === 0 && !length.writable && !length.enumerable && length.configurable, 'optional dictionary arity');
    rootKeys(Reflect.ownKeys(method), ['length', 'name']);
    rootCheck(Object.getPrototypeOf(method) === Function.prototype, 'function prototype');
    for (const node of [document, document.createElement('div'), new DocumentFragment(), new Text(''), new Comment('')]) {
      rootCheck(node.getRootNode === method && Object.getOwnPropertyDescriptor(node, 'getRootNode') === undefined, 'inherited shared identity');
    }
    rootTypeError(function() { Reflect.construct(method, []); }, 'method is not constructible');
    return true;
  },
  represented_complete_key_order: function() {
    rootPrerequisite();
    const strings = ['nodeValue','textContent','getRootNode','hasChildNodes','normalize','isSameNode','contains',
      'ELEMENT_NODE','ATTRIBUTE_NODE','TEXT_NODE','CDATA_SECTION_NODE','ENTITY_REFERENCE_NODE','ENTITY_NODE',
      'PROCESSING_INSTRUCTION_NODE','COMMENT_NODE','DOCUMENT_NODE','DOCUMENT_TYPE_NODE','DOCUMENT_FRAGMENT_NODE','NOTATION_NODE',
      'DOCUMENT_POSITION_DISCONNECTED','DOCUMENT_POSITION_PRECEDING','DOCUMENT_POSITION_FOLLOWING','DOCUMENT_POSITION_CONTAINS',
      'DOCUMENT_POSITION_CONTAINED_BY','DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC','constructor'];
    rootKeys(Object.getOwnPropertyNames(Node.prototype), strings);
    const keys = Reflect.ownKeys(Node.prototype);
    rootCheck(keys.length === 27 && strings.length === 26 && keys[26] === Symbol.toStringTag, '27 represented keys');
    for (let i = 0; i < 26; i++) rootCheck(keys[i] === strings[i], 'complete prototype order');
    const enumerable = strings.slice(0, 25);
    rootKeys(Object.keys(Node.prototype), enumerable);
    const constructor = ['length','name','prototype','ELEMENT_NODE','ATTRIBUTE_NODE','TEXT_NODE','CDATA_SECTION_NODE',
      'ENTITY_REFERENCE_NODE','ENTITY_NODE','PROCESSING_INSTRUCTION_NODE','COMMENT_NODE','DOCUMENT_NODE','DOCUMENT_TYPE_NODE',
      'DOCUMENT_FRAGMENT_NODE','NOTATION_NODE','DOCUMENT_POSITION_DISCONNECTED','DOCUMENT_POSITION_PRECEDING',
      'DOCUMENT_POSITION_FOLLOWING','DOCUMENT_POSITION_CONTAINS','DOCUMENT_POSITION_CONTAINED_BY','DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC'];
    rootKeys(Reflect.ownKeys(Node), constructor);
    return true;
  },
  connected_detached_and_document_aliases: function() {
    const method = rootPrerequisite(), html = document.documentElement, body = document.body;
    if (!html || !body) throw new TypeError('parsed HTML tree');
    const outer = document.createElement('div'), inner = document.createElement('span'), leaf = new Text('same');
    inner.append(leaf); outer.append(inner);
    rootCheck(method.call(leaf) === outer && method.call(outer) === outer, 'detached element root');
    body.append(outer);
    rootCheck(method.call(leaf) === document && method.call(inner, {composed: true}) === document, 'connected roots');
    rootCheck(method.call(document) === document && method.call(html.parentNode) === document && html.parentNode === document, 'canonical Document value');
    const parked = new DocumentFragment(); parked.append(outer);
    rootCheck(method.call(leaf) === parked && method.call(outer) === parked && method.call(parked) === parked, 'move to fragment');
    parked.textContent = '';
    rootCheck(outer.parentNode === null && inner.parentNode === outer && leaf.parentNode === inner, 'retained detached links');
    rootCheck(method.call(leaf, {composed: true}) === outer && leaf.data === 'same', 'fresh retained root');
    return true;
  },
  leaf_roots_preserve_exact_data_and_identity: function() {
    const method = rootPrerequisite();
    if (typeof Comment !== 'function' || typeof document.createProcessingInstruction !== 'function') throw new TypeError('represented leaf constructors');
    const a = new Text('\uD800x'), same = new Text('\uD800x'), comment = new Comment('\uDC00'), pi = document.createProcessingInstruction('ok', '?');
    for (const node of [a, same, comment, pi]) rootCheck(method.call(node) === node && method.call(node, {composed: true}) === node, 'each detached leaf is its own root');
    rootCheck(method.call(a) !== method.call(same), 'root identity is not payload equality');
    rootCheck(a.data.length === 2 && a.data.charCodeAt(0) === 55296 && a.data.charCodeAt(1) === 120, 'retained high surrogate');
    rootCheck(comment.data.length === 1 && comment.data.charCodeAt(0) === 56320 && pi.target === 'ok' && pi.data === '?', 'other leaf payloads');
    return true;
  },
  template_content_never_crosses_host: function() {
    const method = rootPrerequisite(), host = document.createElement('div'), template = document.createElement('template'), content = template.content;
    if (!content || content.nodeType !== 11) throw new TypeError('genuine template content');
    const inside = new Text('inside'), ordinary = new Text('ordinary'), nested = document.createElement('template'), deep = new Text('deep');
    content.append(inside, nested); nested.content.append(deep); template.append(ordinary); host.append(template);
    for (const options of [undefined, {composed: false}, {composed: true}]) {
      rootCheck(method.call(content, options) === content && method.call(inside, options) === content, 'content root');
      rootCheck(method.call(nested, options) === content && method.call(deep, options) === nested.content, 'nested independent content');
      rootCheck(method.call(ordinary, options) === host && method.call(template, options) === host, 'ordinary template children');
    }
    document.body.append(host);
    rootCheck(method.call(ordinary, {composed: true}) === document && method.call(inside, {composed: true}) === content, 'connected host does not join content');
    return true;
  },
  default_options_do_not_read_object_prototype: function() {
    const method = rootPrerequisite(), leaf = new Text('x'), previous = Object.getOwnPropertyDescriptor(Object.prototype, 'composed');
    let reads = 0, receiver;
    Object.defineProperty(Object.prototype, 'composed', {get: function() { reads++; receiver = this; return true; }, configurable: true});
    try {
      rootCheck(method.call(leaf) === leaf && method.call(leaf, undefined) === leaf && method.call(leaf, null) === leaf, 'default dictionary roots');
      rootCheck(reads === 0, 'defaults do not synthesize a JS object Get');
      const explicit = {};
      rootCheck(method.call(leaf, explicit) === leaf && reads === 1 && receiver === explicit, 'explicit object uses inherited Get');
    } finally {
      if (previous === undefined) delete Object.prototype.composed;
      else Object.defineProperty(Object.prototype, 'composed', previous);
    }
    return true;
  },
  inherited_non_enumerable_member_get_once: function() {
    const method = rootPrerequisite(), leaf = new Text('x'), proto = {}, options = Object.create(proto);
    let reads = 0, receiver, unrelated = 0;
    Object.defineProperty(proto, 'composed', {get: function() { reads++; receiver = this; return undefined; }, enumerable: false, configurable: true});
    for (const key of ['unknown', 'toString', 'valueOf', Symbol.iterator]) Object.defineProperty(options, key, {get: function() { unrelated++; throw new Error('unrelated member'); }, enumerable: true});
    rootCheck(method.call(leaf, options) === leaf, 'undefined member defaults false');
    rootCheck(reads === 1 && receiver === options && unrelated === 0, 'one inherited Get with original receiver and no enumeration');
    return true;
  },
  object_options_and_boolean_conversion_do_not_coerce: function() {
    const method = rootPrerequisite(), leaf = new Text('x'); let reads = 0, coercions = 0;
    const truthy = {};
    for (const key of [Symbol.toPrimitive, 'valueOf', 'toString']) Object.defineProperty(truthy, key, {get: function() { coercions++; throw new Error('boolean coercion hook'); }});
    const array = [], callable = function() {}, host = new Text('options');
    for (const options of [array, callable, host]) {
      Object.defineProperty(options, 'composed', {get: function() { reads++; rootCheck(this === options, 'dictionary receiver'); return truthy; }, configurable: true});
      rootCheck(method.call(leaf, options) === leaf, 'represented object dictionary');
    }
    for (const value of [false, true, 0, NaN, '', 'yes', null, undefined, Symbol('truth'), truthy]) rootCheck(method.call(leaf, {composed: value}) === leaf, 'ordinary roots for both boolean values');
    rootCheck(reads === 3 && coercions === 0, 'ToBoolean never requests primitive hooks');
    return true;
  },
  non_null_primitive_options_throw: function() {
    const method = rootPrerequisite(), leaf = new Text('x');
    rootCheck(method.call(leaf, null) === leaf && method.call(leaf, undefined) === leaf, 'allowed nullish options');
    for (const value of [false, true, 0, -0, 1, NaN, '', 'x', Symbol('options')]) {
      rootTypeError(function() { method.call(leaf, value); }, 'dictionary rejects non-null primitive');
    }
    return true;
  },
  receiver_brand_precedes_dictionary_get: function() {
    const method = rootPrerequisite(); let reads = 0, fakeReads = 0;
    const options = {get composed() { reads++; throw new Error('must not read options'); }};
    const fake = Object.create(Node.prototype);
    Object.defineProperty(fake, 'nodeType', {get: function() { fakeReads++; return 3; }});
    for (const value of [null, undefined, globalThis, Node.prototype, fake, {}, 0, 'x', true]) {
      rootTypeError(function() { method.call(value, options); }, 'authentic receiver before conversion');
    }
    rootCheck(reads === 0 && fakeReads === 0, 'brand uses internal identity');
    return true;
  },
  callback_moves_receiver_and_ancestor_before_root_read: function() {
    const method = rootPrerequisite(), left = new DocumentFragment(), right = new DocumentFragment(), leaf = new Text('x');
    left.append(leaf); let reads = 0;
    const first = {get composed() { reads++; right.append(leaf); return false; }};
    rootCheck(method.call(leaf, first) === right && reads === 1 && leaf.parentNode === right, 'receiver moved during conversion');
    const branch = document.createElement('span'); branch.append(leaf); left.append(branch);
    const second = {get composed() { reads++; right.append(branch); return true; }};
    rootCheck(method.call(leaf, second) === right && reads === 2 && branch.parentNode === right, 'ancestor moved during conversion');
    const third = {get composed() { right.textContent = ''; return false; }};
    rootCheck(method.call(leaf, third) === branch && branch.parentNode === null && leaf.parentNode === branch, 'ancestor detached during conversion');
    return true;
  },
  getter_abrupt_identity_retains_completed_move: function() {
    const method = rootPrerequisite(), left = new DocumentFragment(), right = new DocumentFragment(), leaf = new Text('x'), marker = {};
    left.append(leaf); let reads = 0, caught = false;
    const options = {get composed() { reads++; right.append(leaf); throw marker; }};
    try { method.call(leaf, options); } catch (error) { caught = error === marker; }
    rootCheck(caught && reads === 1 && leaf.parentNode === right && left.childNodes.length === 0, 'abrupt identity and move retained');
    rootCheck(method.call(leaf) === right, 'fresh call sees retained mutation');
    return true;
  },
  argument_expressions_and_ignored_extra_conversion: function() {
    const method = rootPrerequisite(), left = new DocumentFragment(), right = new DocumentFragment(), leaf = new Text('x');
    left.append(leaf); let trace = '', coercions = 0;
    const poison = {toString: function() { coercions++; throw new Error('extra conversion'); }, valueOf: function() { coercions++; throw new Error('extra conversion'); }};
    function options() { trace += 'A'; return {get composed() { trace += 'C'; return false; }}; }
    function extra() { trace += 'B'; right.append(leaf); return poison; }
    rootCheck(method.call(leaf, options(), extra()) === right && trace === 'ABC' && coercions === 0, 'argument expressions then dictionary Get');
    const marker = {}; let caught = false;
    function fail() { trace += 'X'; throw marker; }
    try { method.call({}, fail()); } catch (error) { caught = error === marker; }
    rootCheck(caught && trace === 'ABCX', 'argument throw precedes native brand');
    return true;
  },
  saved_method_shadow_replacement_and_deletion: function() {
    const method = rootPrerequisite(), leaf = new Text('x'), descriptor = Object.getOwnPropertyDescriptor(Node.prototype, 'getRootNode');
    const replacement = function() { return 'authored'; };
    leaf.getRootNode = replacement;
    rootCheck(leaf.getRootNode() === 'authored' && method.call(leaf) === leaf, 'own shadow does not replace saved native');
    delete leaf.getRootNode;
    try {
      Node.prototype.getRootNode = replacement;
      rootCheck(leaf.getRootNode === replacement && leaf.getRootNode() === 'authored' && method.call(leaf) === leaf, 'ordinary prototype replacement');
      delete Node.prototype.getRootNode;
      rootCheck(leaf.getRootNode === undefined && Object.getOwnPropertyDescriptor(Node.prototype, 'getRootNode') === undefined, 'no fallback resurrection');
      rootCheck(method.call(leaf) === leaf, 'saved method after deletion');
    } finally { Object.defineProperty(Node.prototype, 'getRootNode', descriptor); }
    rootCheck(leaf.getRootNode === method, 'restored descriptor');
    return true;
  },
  internal_tree_ignores_public_properties_and_nested_call: function() {
    const method = rootPrerequisite(), box = new DocumentFragment(), leaf = new Text('x'); box.append(leaf); let poison = 0, reads = 0;
    for (const node of [box, leaf]) for (const key of ['parentNode', 'childNodes', 'host', 'shadowRoot', 'nodeType']) {
      Object.defineProperty(node, key, {get: function() { poison++; throw new Error('public tree getter'); }, configurable: true});
    }
    const options = {get composed() { reads++; rootCheck(method.call(leaf) === box, 'nested default call'); return true; }};
    rootCheck(method.call(leaf, options) === box && method.call(box) === box && reads === 1 && poison === 0, 'internal links after one callback');
    return true;
  },
  authentic_alternate_prototype_keeps_node_brand: function() {
    const method = rootPrerequisite(), target = (function() {}).bind(null), prototype = Object.create(null);
    if (typeof Reflect.construct !== 'function') throw new TypeError('Reflect.construct prerequisite');
    Object.defineProperty(target, 'prototype', {value: prototype});
    const leaf = Reflect.construct(Text, ['kept'], target), box = new DocumentFragment();
    rootCheck(Object.getPrototypeOf(leaf) === prototype && leaf.getRootNode === undefined, 'genuine alternate prototype');
    rootCheck(method.call(leaf) === leaf, 'detached authentic brand');
    box.append(leaf); rootCheck(method.call(leaf, {composed: true}) === box, 'linked authentic brand');
    rootTypeError(function() { method.call(prototype); }, 'prototype object is not a Node');
    return true;
  }
};
