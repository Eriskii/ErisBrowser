// Independent literals; invoke one nodeConstantCases body per fresh realm/mode.
// Node's represented prototype inventory is intentionally narrower than full DOM.
// https://dom.spec.whatwg.org/#interface-node
// https://webidl.spec.whatwg.org/#js-constants
function nodeConstantCheck(value, message) {
  if (!value) throw new Error(message);
}
function nodeConstantKeys(actual, expected) {
  nodeConstantCheck(actual.length === expected.length, 'complete key count');
  for (let i = 0; i < expected.length; i++)
    nodeConstantCheck(actual[i] === expected[i], 'key order ' + i);
}
const nodeConstantRows = [
  ['ELEMENT_NODE', 1], ['ATTRIBUTE_NODE', 2], ['TEXT_NODE', 3],
  ['CDATA_SECTION_NODE', 4], ['ENTITY_REFERENCE_NODE', 5], ['ENTITY_NODE', 6],
  ['PROCESSING_INSTRUCTION_NODE', 7], ['COMMENT_NODE', 8], ['DOCUMENT_NODE', 9],
  ['DOCUMENT_TYPE_NODE', 10], ['DOCUMENT_FRAGMENT_NODE', 11], ['NOTATION_NODE', 12],
  ['DOCUMENT_POSITION_DISCONNECTED', 1], ['DOCUMENT_POSITION_PRECEDING', 2],
  ['DOCUMENT_POSITION_FOLLOWING', 4], ['DOCUMENT_POSITION_CONTAINS', 8],
  ['DOCUMENT_POSITION_CONTAINED_BY', 16], ['DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC', 32]
];
function nodeConstantDescriptors(owner) {
  nodeConstantCheck(nodeConstantRows.length === 18, 'literal roster');
  for (let i = 0; i < 18; i++) {
    const row = nodeConstantRows[i], d = Object.getOwnPropertyDescriptor(owner, row[0]);
    nodeConstantCheck(d !== undefined, 'own constant ' + row[0]);
    nodeConstantCheck(typeof d.value === 'number' && d.value === row[1], 'literal constant value');
    nodeConstantCheck(d.writable === false && d.enumerable === true && d.configurable === false,
      'constant descriptor flags');
    nodeConstantCheck(d.get === undefined && d.set === undefined, 'constant is data');
    nodeConstantCheck(owner[row[0]] === row[1], 'ordinary constant lookup');
  }
}
const nodeConstantCases = {
  constructor_constant_descriptors: function () {
    nodeConstantCheck(typeof Node === 'function' && Node !== Node.prototype, 'Node interface');
    nodeConstantDescriptors(Node);
    return true;
  },
  prototype_constant_descriptors: function () {
    nodeConstantCheck(typeof Node === 'function' && Node.prototype.constructor === Node, 'Node prototype');
    nodeConstantDescriptors(Node.prototype);
    return true;
  },
  constructor_complete_key_order: function () {
    const expected = ['length', 'name', 'prototype',
      'ELEMENT_NODE', 'ATTRIBUTE_NODE', 'TEXT_NODE', 'CDATA_SECTION_NODE',
      'ENTITY_REFERENCE_NODE', 'ENTITY_NODE', 'PROCESSING_INSTRUCTION_NODE', 'COMMENT_NODE',
      'DOCUMENT_NODE', 'DOCUMENT_TYPE_NODE', 'DOCUMENT_FRAGMENT_NODE', 'NOTATION_NODE',
      'DOCUMENT_POSITION_DISCONNECTED', 'DOCUMENT_POSITION_PRECEDING',
      'DOCUMENT_POSITION_FOLLOWING', 'DOCUMENT_POSITION_CONTAINS',
      'DOCUMENT_POSITION_CONTAINED_BY', 'DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC'];
    nodeConstantKeys(Object.getOwnPropertyNames(Node), expected);
    nodeConstantKeys(Reflect.ownKeys(Node), expected);
    nodeConstantKeys(Object.keys(Node), [
      'ELEMENT_NODE', 'ATTRIBUTE_NODE', 'TEXT_NODE', 'CDATA_SECTION_NODE',
      'ENTITY_REFERENCE_NODE', 'ENTITY_NODE', 'PROCESSING_INSTRUCTION_NODE', 'COMMENT_NODE',
      'DOCUMENT_NODE', 'DOCUMENT_TYPE_NODE', 'DOCUMENT_FRAGMENT_NODE', 'NOTATION_NODE',
      'DOCUMENT_POSITION_DISCONNECTED', 'DOCUMENT_POSITION_PRECEDING',
      'DOCUMENT_POSITION_FOLLOWING', 'DOCUMENT_POSITION_CONTAINS',
      'DOCUMENT_POSITION_CONTAINED_BY', 'DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC']);
    return true;
  },
  prototype_complete_key_order: function () {
    // Complete current represented inventory, not the complete standard Node API.
    const strings = ['nodeValue', 'textContent',
      'ELEMENT_NODE', 'ATTRIBUTE_NODE', 'TEXT_NODE', 'CDATA_SECTION_NODE',
      'ENTITY_REFERENCE_NODE', 'ENTITY_NODE', 'PROCESSING_INSTRUCTION_NODE', 'COMMENT_NODE',
      'DOCUMENT_NODE', 'DOCUMENT_TYPE_NODE', 'DOCUMENT_FRAGMENT_NODE', 'NOTATION_NODE',
      'DOCUMENT_POSITION_DISCONNECTED', 'DOCUMENT_POSITION_PRECEDING',
      'DOCUMENT_POSITION_FOLLOWING', 'DOCUMENT_POSITION_CONTAINS',
      'DOCUMENT_POSITION_CONTAINED_BY', 'DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC', 'constructor'];
    nodeConstantKeys(Object.getOwnPropertyNames(Node.prototype), strings);
    const all = Reflect.ownKeys(Node.prototype);
    nodeConstantCheck(all.length === 22, 'all prototype keys');
    for (let i = 0; i < 21; i++) nodeConstantCheck(all[i] === strings[i], 'prototype string order');
    nodeConstantCheck(all[21] === Symbol.toStringTag, 'symbol after strings');
    nodeConstantKeys(Object.keys(Node.prototype), ['nodeValue', 'textContent',
      'ELEMENT_NODE', 'ATTRIBUTE_NODE', 'TEXT_NODE', 'CDATA_SECTION_NODE',
      'ENTITY_REFERENCE_NODE', 'ENTITY_NODE', 'PROCESSING_INSTRUCTION_NODE', 'COMMENT_NODE',
      'DOCUMENT_NODE', 'DOCUMENT_TYPE_NODE', 'DOCUMENT_FRAGMENT_NODE', 'NOTATION_NODE',
      'DOCUMENT_POSITION_DISCONNECTED', 'DOCUMENT_POSITION_PRECEDING',
      'DOCUMENT_POSITION_FOLLOWING', 'DOCUMENT_POSITION_CONTAINS',
      'DOCUMENT_POSITION_CONTAINED_BY', 'DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC']);
    return true;
  },
  saved_nonconstant_metadata_and_accessors: function () {
    const proto = Node.prototype;
    nodeConstantCheck(Object.getPrototypeOf(Node) === EventTarget, 'constructor parent');
    nodeConstantCheck(Object.getPrototypeOf(proto) === EventTarget.prototype, 'prototype parent');
    const length = Object.getOwnPropertyDescriptor(Node, 'length');
    const name = Object.getOwnPropertyDescriptor(Node, 'name');
    const prototype = Object.getOwnPropertyDescriptor(Node, 'prototype');
    const constructor = Object.getOwnPropertyDescriptor(proto, 'constructor');
    const tag = Object.getOwnPropertyDescriptor(proto, Symbol.toStringTag);
    nodeConstantCheck(length.value === 0 && !length.writable && !length.enumerable && length.configurable, 'length');
    nodeConstantCheck(name.value === 'Node' && !name.writable && !name.enumerable && name.configurable, 'name');
    nodeConstantCheck(prototype.value === proto && !prototype.writable && !prototype.enumerable && !prototype.configurable, 'prototype');
    nodeConstantCheck(constructor.value === Node && constructor.writable && !constructor.enumerable && constructor.configurable, 'constructor');
    nodeConstantCheck(tag.value === 'Node' && !tag.writable && !tag.enumerable && tag.configurable, 'tag');
    const nodeValue = Object.getOwnPropertyDescriptor(proto, 'nodeValue');
    const textContent = Object.getOwnPropertyDescriptor(proto, 'textContent');
    nodeConstantCheck(typeof nodeValue.get === 'function' && typeof nodeValue.set === 'function' &&
      typeof textContent.get === 'function' && typeof textContent.set === 'function', 'saved accessors');
    nodeConstantCheck(nodeValue.enumerable && nodeValue.configurable &&
      textContent.enumerable && textContent.configurable, 'accessor flags');
    nodeConstantCheck(nodeValue.get !== nodeValue.set && nodeValue.get !== textContent.get, 'separate function identities');
    const text = new Text('before');
    nodeConstantCheck(nodeValue.get.call(text) === 'before', 'saved getter works');
    textContent.set.call(text, 'after');
    nodeConstantCheck(nodeValue.get.call(text) === 'after' && textContent.get.call(text) === 'after', 'saved setter works');
    nodeConstantCheck(Object.getOwnPropertyDescriptor(proto, 'nodeValue').get === nodeValue.get &&
      Object.getOwnPropertyDescriptor(proto, 'textContent').set === textContent.set, 'stable saved identities');
    return true;
  },
  separate_mutable_constructor_and_prototype_bags: function () {
    const proto = Node.prototype, key = '__nodeConstantsOwnBagProbe', symbol = Symbol('own-bag');
    const a = {}, b = {};
    nodeConstantCheck(Object.getOwnPropertyDescriptor(Node, key) === undefined &&
      Object.getOwnPropertyDescriptor(proto, key) === undefined, 'fresh own string slots');
    try {
      Object.defineProperty(Node, key, {value: a, writable: true, enumerable: true, configurable: true});
      nodeConstantCheck(Object.getOwnPropertyDescriptor(proto, key) === undefined, 'constructor addition isolated');
      Object.defineProperty(proto, key, {value: b, writable: true, enumerable: false, configurable: true});
      nodeConstantCheck(Node[key] === a && proto[key] === b, 'separate string values');
      Node[key] = 17;
      nodeConstantCheck(Node[key] === 17 && proto[key] === b, 'constructor string write isolated');
      Object.defineProperty(Node, symbol, {value: a, writable: true, configurable: true});
      nodeConstantCheck(Object.getOwnPropertyDescriptor(proto, symbol) === undefined, 'constructor symbol isolated');
      Object.defineProperty(proto, symbol, {value: b, writable: true, configurable: true});
      proto[symbol] = 29;
      nodeConstantCheck(Node[symbol] === a && proto[symbol] === 29, 'prototype symbol write isolated');
      nodeConstantCheck(delete Node[key], 'delete constructor string');
      nodeConstantCheck(Object.getOwnPropertyDescriptor(Node, key) === undefined && proto[key] === b, 'delete string isolation');
      nodeConstantCheck(delete proto[symbol], 'delete prototype symbol');
      nodeConstantCheck(Object.getOwnPropertyDescriptor(proto, symbol) === undefined && Node[symbol] === a, 'delete symbol isolation');
      nodeConstantCheck(Node.prototype === proto && proto.constructor === Node &&
        Node.ELEMENT_NODE === 1 && proto.DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC === 32, 'metadata survives mutations');
    } finally {
      delete Node[key]; delete proto[key]; delete Node[symbol]; delete proto[symbol];
    }
    return true;
  },
  realm_markers_are_initially_absent: function () {
    nodeConstantCheck(Object.getOwnPropertyDescriptor(Node, '__nodeConstantsRealmProbe') === undefined,
      'fresh constructor realm marker');
    nodeConstantCheck(Object.getOwnPropertyDescriptor(Node.prototype, '__nodeConstantsRealmProbe') === undefined,
      'fresh prototype realm marker');
    return true;
  }
};

// Separate PUBLIC Runtime host protocol, not standalone fresh-realm test cases.
// Keep A and B alive concurrently: A.seed_a, B.clean case, B.seed_b,
// A.verify_a, B.verify_b. No JavaScript API creates a second realm here.
function nodeConstantSeedRealm(a, b) {
  nodeConstantCases.realm_markers_are_initially_absent();
  Object.defineProperty(Node, '__nodeConstantsRealmProbe', {value: a, writable: true, configurable: true});
  Object.defineProperty(Node.prototype, '__nodeConstantsRealmProbe', {value: b, writable: true, configurable: true});
  return true;
}
function nodeConstantVerifyRealm(a, b) {
  nodeConstantCheck(Object.getOwnPropertyDescriptor(Node, '__nodeConstantsRealmProbe').value === a,
    'constructor realm retained');
  nodeConstantCheck(Object.getOwnPropertyDescriptor(Node.prototype, '__nodeConstantsRealmProbe').value === b,
    'prototype realm retained');
  nodeConstantCheck(Node.ELEMENT_NODE === 1 && Node.prototype.DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC === 32,
    'constants retained with realm marker');
  return true;
}
const nodeConstantRealmSteps = {
  seed_a: function () { return nodeConstantSeedRealm(41, 42); },
  seed_b: function () { return nodeConstantSeedRealm(71, 72); },
  verify_a: function () { return nodeConstantVerifyRealm(41, 42); },
  verify_b: function () { return nodeConstantVerifyRealm(71, 72); }
};
