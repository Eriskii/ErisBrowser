// Independent ordinary-tree connection cases, one named body per fresh realm.
// https://dom.spec.whatwg.org/#dom-node-isconnected
// https://webidl.spec.whatwg.org/#es-attributes
function connectedCheck(value, message) { if (!value) throw new Error(message); }
function connectedTypeError(body, message) {
  let caught = false;
  try { body(); } catch (error) { caught = error instanceof TypeError; }
  connectedCheck(caught, message);
}
function connectedKeys(actual, expected) {
  connectedCheck(actual.length === expected.length, 'key count');
  for (let i = 0; i < expected.length; i++) connectedCheck(actual[i] === expected[i], 'literal key order');
}
function connectedPrerequisite() {
  if (typeof Node !== 'function' || typeof Text !== 'function' || typeof DocumentFragment !== 'function')
    throw new TypeError('represented constructors');
  const descriptor = Object.getOwnPropertyDescriptor(Node.prototype, 'isConnected');
  if (!descriptor || typeof descriptor.get !== 'function') throw new TypeError('isConnected getter');
  const get = descriptor.get, detached = new Text('guard');
  if (get.call(document) !== true || !document.body || get.call(document.body) !== true ||
      get.call(detached) !== false || get.call(new DocumentFragment()) !== false)
    throw new TypeError('working authentic connection and disconnection');
  return get;
}
const nodeConnectedCases = {
  metadata_and_cached_getter: function() {
    const get = connectedPrerequisite(), descriptor = Object.getOwnPropertyDescriptor(Node.prototype, 'isConnected');
    connectedCheck(descriptor.get === get && descriptor.set === undefined && descriptor.enumerable === true && descriptor.configurable === true, 'readonly ordinary accessor');
    connectedCheck(descriptor.value === undefined && descriptor.writable === undefined, 'accessor descriptor');
    const name = Object.getOwnPropertyDescriptor(get, 'name'), length = Object.getOwnPropertyDescriptor(get, 'length');
    connectedCheck(name.value === 'get isConnected' && !name.writable && !name.enumerable && name.configurable, 'getter name');
    connectedCheck(length.value === 0 && !length.writable && !length.enumerable && length.configurable, 'getter arity');
    connectedKeys(Reflect.ownKeys(get), ['length', 'name']);
    connectedCheck(Object.getPrototypeOf(get) === Function.prototype, 'Function inheritance');
    const node = new Text('x'); let seen = 0;
    for (const key in node) if (key === 'isConnected') seen++;
    connectedCheck(seen === 1 && node.isConnected === false && Object.getOwnPropertyDescriptor(node, 'isConnected') === undefined, 'inherited enumerable accessor');
    connectedCheck(Object.getOwnPropertyDescriptor(Node.prototype, 'isConnected').get === get, 'cached identity');
    if (typeof Reflect.construct !== 'function') throw new TypeError('Reflect.construct prerequisite');
    const made = Reflect.construct(Text, ['guard']);
    if (made.data !== 'guard' || get.call(made) !== false) throw new TypeError('working genuine constructor');
    connectedTypeError(function() { Reflect.construct(get, []); }, 'nonconstructible getter');
    return true;
  },
  represented_complete_key_order: function() {
    connectedPrerequisite();
    const strings = ['isConnected','nodeValue','textContent','getRootNode','hasChildNodes','normalize','isEqualNode','isSameNode','contains',
      'ELEMENT_NODE','ATTRIBUTE_NODE','TEXT_NODE','CDATA_SECTION_NODE','ENTITY_REFERENCE_NODE','ENTITY_NODE',
      'PROCESSING_INSTRUCTION_NODE','COMMENT_NODE','DOCUMENT_NODE','DOCUMENT_TYPE_NODE','DOCUMENT_FRAGMENT_NODE','NOTATION_NODE',
      'DOCUMENT_POSITION_DISCONNECTED','DOCUMENT_POSITION_PRECEDING','DOCUMENT_POSITION_FOLLOWING','DOCUMENT_POSITION_CONTAINS',
      'DOCUMENT_POSITION_CONTAINED_BY','DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC','constructor'];
    connectedKeys(Object.getOwnPropertyNames(Node.prototype), strings);
    const keys = Reflect.ownKeys(Node.prototype);
    connectedCheck(strings.length === 28 && keys.length === 29 && keys[28] === Symbol.toStringTag, 'conditional29 represented keys');
    for (let i = 0; i < 28; i++) connectedCheck(keys[i] === strings[i], 'prototype creation order');
    connectedKeys(Object.keys(Node.prototype), strings.slice(0,27));
    connectedKeys(Reflect.ownKeys(Node), ['length','name','prototype','ELEMENT_NODE','ATTRIBUTE_NODE','TEXT_NODE','CDATA_SECTION_NODE',
      'ENTITY_REFERENCE_NODE','ENTITY_NODE','PROCESSING_INSTRUCTION_NODE','COMMENT_NODE','DOCUMENT_NODE','DOCUMENT_TYPE_NODE',
      'DOCUMENT_FRAGMENT_NODE','NOTATION_NODE','DOCUMENT_POSITION_DISCONNECTED','DOCUMENT_POSITION_PRECEDING',
      'DOCUMENT_POSITION_FOLLOWING','DOCUMENT_POSITION_CONTAINS','DOCUMENT_POSITION_CONTAINED_BY','DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC']);
    return true;
  },
  authentic_receiver_brand_without_properties: function() {
    const get = connectedPrerequisite(), fake = Object.create(Node.prototype); let reads = 0;
    for (const key of ['parentNode','nodeType','ownerDocument','getRootNode'])
      Object.defineProperty(fake, key, {get: function() { reads++; throw new Error('forged property'); }});
    for (const receiver of [null, undefined, window, Node.prototype, fake, {}, 0, '', Symbol('node')])
      connectedTypeError(function() { get.call(receiver); }, 'authentic receiver');
    connectedTypeError(function() { get(); }, 'missing authentic receiver');
    connectedCheck(reads === 0 && get.call(document) === true, 'no forged field reads');
    return true;
  },
  document_alias_and_connected_character_kinds: function() {
    const get = connectedPrerequisite(), root = document.documentElement;
    if (!root || root.parentNode !== document) throw new TypeError('canonical Document alias');
    const box = document.createElement('div'), text = new Text('A\uD800'), comment = new Comment('kept');
    if (typeof ProcessingInstruction !== 'function') throw new TypeError('PI prerequisite');
    const pi = new ProcessingInstruction('probe','data'); box.append(text,comment,pi); document.body.append(box);
    for (const node of [document,root.parentNode,root,document.body,box,text,comment,pi])
      connectedCheck(get.call(node) === true && node.isConnected === true, 'connected represented kind');
    const parked = new DocumentFragment(); parked.append(box);
    for (const node of [box,text,comment,pi,parked]) connectedCheck(get.call(node) === false, 'detached retained kinds');
    connectedCheck(text.data.length === 2 && text.data.charCodeAt(1) === 55296 && text.parentNode === box, 'exact payload retained');
    return true;
  },
  detached_leaves_branches_and_fragment_splicing: function() {
    const get = connectedPrerequisite(), box = document.createElement('div'), child = new Text('x'), fragment = new DocumentFragment();
    box.append(child); fragment.append(box);
    for (const node of [new Text(''),new Comment(''),box,child,fragment]) connectedCheck(get.call(node) === false, 'ordinary detached roots');
    document.body.append(fragment);
    connectedCheck(get.call(box) === true && get.call(child) === true && get.call(fragment) === false, 'append splices children, not fragment');
    connectedCheck(fragment.childNodes.length === 0 && box.parentNode === document.body && box.firstChild === child, 'retained identities after splice');
    return true;
  },
  current_connection_after_repeated_subtree_moves: function() {
    const get = connectedPrerequisite(), branch = document.createElement('div'), nested = document.createElement('span'), leaf = new Text('leaf');
    const parked = new DocumentFragment(), detached = document.createElement('section'); nested.append(leaf); branch.append(nested);
    connectedCheck(get.call(leaf) === false, 'initial detached branch');
    document.body.append(branch); connectedCheck(get.call(leaf) === true, 'connected ancestor');
    parked.append(branch); connectedCheck(get.call(leaf) === false, 'fragment ancestor');
    detached.append(branch); connectedCheck(get.call(leaf) === false, 'detached element ancestor');
    document.body.append(detached); connectedCheck(get.call(leaf) === true && get.call(branch) === true, 'same identities reconnected');
    parked.append(detached); connectedCheck(get.call(leaf) === false && get.call(branch) === false, 'same identities disconnected again');
    connectedCheck(branch.firstChild === nested && nested.firstChild === leaf && leaf.data === 'leaf', 'no replacement or payload change');
    return true;
  },
  template_content_is_separate_from_ordinary_children: function() {
    const get = connectedPrerequisite(), template = document.createElement('template');
    if (!template.content || template.content.nodeType !== 11) throw new TypeError('actual template content');
    const ordinary = new Text('ordinary'), inside = new Text('inside'); template.append(ordinary); template.content.append(inside); document.body.append(template);
    connectedCheck(get.call(template) === true && get.call(ordinary) === true, 'ordinary template tree');
    connectedCheck(get.call(template.content) === false && get.call(inside) === false, 'host association is not connection');
    template.append(inside); connectedCheck(get.call(inside) === true && template.content.childNodes.length === 0, 'move out of content');
    template.content.append(ordinary); connectedCheck(get.call(ordinary) === false && get.call(inside) === true, 'move into content');
    const parked = new DocumentFragment(); parked.append(template);
    connectedCheck(get.call(template) === false && get.call(inside) === false && get.call(ordinary) === false, 'both trees detached independently');
    return true;
  },
  namespace_visibility_and_empty_data_do_not_decide: function() {
    const get = connectedPrerequisite(), box = document.createElement('div');
    box.innerHTML='<svg><g></g></svg><math><mi></mi></math>';
    const svg = box.firstChild, math = box.childNodes[1], g = svg.firstChild, mi = math.firstChild, empty = new Text('');
    if (svg.namespaceURI !== 'http://www.w3.org/2000/svg' || math.namespaceURI !== 'http://www.w3.org/1998/Math/MathML') throw new TypeError('foreign parse prerequisites');
    box.setAttribute('style','display:none'); box.append(empty); document.body.append(box);
    for (const node of [box,svg,g,math,mi,empty]) connectedCheck(get.call(node) === true, 'hidden foreign or empty connected node');
    const parked = new DocumentFragment(); parked.append(box);
    for (const node of [box,svg,g,math,mi,empty]) connectedCheck(get.call(node) === false, 'same hidden detached nodes');
    return true;
  },
  receiver_and_extra_argument_effects_precede_read: function() {
    const get = connectedPrerequisite(), branch = document.createElement('div'), leaf = new Text('x'), parked = new DocumentFragment();
    branch.append(leaf); let trace = '';
    function receiver() { trace += 'R'; document.body.append(branch); return leaf; }
    function extra() { trace += 'A'; parked.append(branch); return {ignored:true}; }
    connectedCheck(get.call(receiver(),extra()) === false && trace === 'RA', 'extra argument can disconnect before getter');
    function connect() { trace += 'C'; document.body.append(branch); return 0; }
    connectedCheck(get.call(leaf,connect()) === true && trace === 'RAC', 'fresh current links after argument effects');
    connectedCheck(leaf.parentNode === branch && branch.parentNode === document.body, 'retained identity');
    return true;
  },
  throwing_argument_preserves_completed_prefix: function() {
    const get = connectedPrerequisite(), branch = document.createElement('div'), leaf = new Text('x'), parked = new DocumentFragment(), marker = {};
    branch.append(leaf); document.body.append(branch); let trace = '', caught = false;
    function first() { trace += 'A'; parked.append(branch); return 1; }
    function throwing() { trace += 'B'; throw marker; }
    try { get.call(leaf,first(),throwing()); } catch (error) { caught = error === marker; }
    connectedCheck(caught && trace === 'AB' && branch.parentNode === parked && get.call(leaf) === false, 'expression throw identity and mutation prefix');
    document.body.append(branch); connectedCheck(get.call(leaf) === true, 'saved getter still usable');
    return true;
  },
  ignored_extra_values_and_internal_field_shadows: function() {
    const get = connectedPrerequisite(), box = document.createElement('div'), node = new Text('x'), extra = {}; let reads = 0;
    box.append(node); document.body.append(box);
    for (const key of ['composed','toString','valueOf',Symbol.toPrimitive])
      Object.defineProperty(extra, key, {get: function() { reads++; throw new Error('extra conversion'); }});
    for (const key of ['parentNode','ownerDocument','getRootNode','nodeType'])
      Object.defineProperty(node, key, {get: function() { reads++; throw new Error('public node field'); }, configurable:true});
    Object.defineProperty(box, 'childNodes', {get: function() { reads++; throw new Error('public children'); }, configurable:true});
    connectedCheck(get.call(node,extra,null,undefined,Symbol('ignored')) === true && reads === 0, 'no options conversion or authored tree lookup');
    const parked = new DocumentFragment(); parked.append(box);
    connectedCheck(get.call(node,extra) === false && reads === 0, 'private current links after move');
    return true;
  },
  readonly_assignment_follows_current_language_mode: function(strict) {
    const get = connectedPrerequisite(), node = new Text('x'); let calls = 0, caught = false;
    const value = {toString:function() { calls++; return 'converted'; }};
    // Literal true/false is supplied from the frozen strict/sloppy mode metadata.
    if (typeof strict !== 'boolean') throw new TypeError('explicit source-mode literal required');
    try { node.isConnected = value; } catch (error) { if (!(error instanceof TypeError)) throw error; caught = true; }
    connectedCheck(caught === strict && calls === 0 && node.isConnected === false && get.call(node) === false, 'sloppy ignore or strict TypeError');
    connectedCheck(Object.getOwnPropertyDescriptor(node,'isConnected') === undefined, 'readonly assignment creates no own property');
    return true;
  },
  strict_readonly_assignment_keeps_rhs_effects: function() {
    const get = connectedPrerequisite(), node = new Text('x'); let calls = 0, trace = '';
    const value = {toString:function() { calls++; return 'converted'; }};
    function rhs() { trace += 'R'; document.body.append(node); return value; }
    connectedTypeError(function() { 'use strict'; node.isConnected = rhs(); }, 'strict missing setter');
    connectedCheck(trace === 'R' && calls === 0 && get.call(node) === true && node.isConnected === true, 'RHS runs once without conversion, getter uses new root');
    connectedCheck(Object.getOwnPropertyDescriptor(node,'isConnected') === undefined, 'strict refusal does not publish own property');
    return true;
  },
  own_data_and_accessor_shadows_delete_normally: function() {
    const get = connectedPrerequisite(), node = new Text('x'); let reads = 0;
    Object.defineProperty(node,'isConnected',{value:'shadow',writable:true,enumerable:true,configurable:true});
    connectedCheck(node.isConnected === 'shadow' && get.call(node) === false, 'own data shadow');
    node.isConnected = 42; connectedCheck(node.isConnected === 42 && get.call(node) === false, 'ordinary own writable value');
    connectedCheck(delete node.isConnected && node.isConnected === false, 'delete restores inherited getter');
    Object.defineProperty(node,'isConnected',{get:function() { reads++; return 'accessor'; },configurable:true});
    connectedCheck(node.isConnected === 'accessor' && reads === 1 && get.call(node) === false, 'own accessor is separate');
    document.body.append(node); connectedCheck(get.call(node) === true && reads === 1, 'saved getter ignores own accessor');
    connectedCheck(delete node.isConnected && node.isConnected === true, 'delete restores fresh native read');
    return true;
  },
  prototype_replacement_deletion_and_saved_getter: function() {
    const get = connectedPrerequisite(), descriptor = Object.getOwnPropertyDescriptor(Node.prototype,'isConnected'), node = new Text('x');
    Object.defineProperty(Node.prototype,'isConnected',{value:'replacement',writable:true,enumerable:true,configurable:true});
    connectedCheck(node.isConnected === 'replacement' && get.call(node) === false, 'prototype property replacement');
    connectedCheck(delete Node.prototype.isConnected && node.isConnected === undefined, 'no virtual getter resurrection');
    node.isConnected = 'expando'; connectedCheck(node.isConnected === 'expando' && get.call(node) === false, 'no virtual readonly interception after deletion');
    connectedCheck(delete node.isConnected, 'remove ordinary expando');
    Object.defineProperty(Node.prototype,'isConnected',descriptor); document.body.append(node);
    connectedCheck(node.isConnected === true && Object.getOwnPropertyDescriptor(Node.prototype,'isConnected').get === get, 'restore exact saved getter');
    return true;
  },
  authentic_alternate_prototype_retains_connection_brand: function() {
    const get = connectedPrerequisite();
    if (typeof Reflect.construct !== 'function') throw new TypeError('Reflect.construct prerequisite');
    const guard = Reflect.construct(Text,['guard']);
    if (guard.data !== 'guard' || get.call(guard) !== false) throw new TypeError('working Text construction');
    function NewTarget() {} const target = NewTarget.bind(null), prototype = Object.create(null); let reads = 0;
    Object.defineProperty(target,'prototype',{get:function() { reads++; return prototype; },configurable:true});
    const node = Reflect.construct(Text,['kept'],target);
    connectedCheck(reads === 1 && Object.getPrototypeOf(node) === prototype && node.isConnected === undefined, 'actual alternate prototype');
    connectedCheck(get.call(node) === false, 'authentic detached brand'); document.body.append(node);
    connectedCheck(document.body.lastChild === node && get.call(node) === true, 'same authentic identity connected');
    const parked = new DocumentFragment(); parked.append(node);
    connectedCheck(parked.firstChild === node && get.call(node) === false, 'same authentic identity disconnected');
    return true;
  }
};
