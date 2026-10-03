// Independent ordinary-tree position cases; one body per fresh realm/mode.
// https://dom.spec.whatwg.org/#dom-node-comparedocumentposition
// https://webidl.spec.whatwg.org/#es-interface
function positionCheck(value, message) { if (!value) throw new Error(message); }
function positionTypeError(body, message) {
  let caught = false;
  try { body(); } catch (error) { caught = error instanceof TypeError; }
  positionCheck(caught, message);
}
function positionKeys(actual, expected) {
  positionCheck(actual.length === expected.length, 'key count');
  for (let i = 0; i < expected.length; i++) positionCheck(actual[i] === expected[i], 'literal key order');
}
function positionDisconnected(compare, a, b) {
  const ab = compare.call(a,b), ba = compare.call(b,a);
  positionCheck((ab === 35 && ba === 37) || (ab === 37 && ba === 35), 'permitted disconnected masks and reversal');
  positionCheck(compare.call(a,b) === ab && compare.call(b,a) === ba, 'repeat stability');
  return ab;
}
function positionPrerequisite() {
  if (typeof Node !== 'function' || typeof Text !== 'function' || typeof DocumentFragment !== 'function')
    throw new TypeError('represented constructors');
  const descriptor = Object.getOwnPropertyDescriptor(Node.prototype,'compareDocumentPosition');
  if (!descriptor || typeof descriptor.value !== 'function') throw new TypeError('position method');
  const compare = descriptor.value, a = new Text('a'), b = new Text('b'), fragment = new DocumentFragment();
  fragment.append(a,b);
  if (compare.call(a,a) !== 0 || compare.call(a,b) !== 4 || compare.call(b,a) !== 2 ||
      compare.call(fragment,a) !== 20 || compare.call(a,fragment) !== 10 ||
      !document.body || compare.call(document,document.body) !== 20 || compare.call(document.body,document) !== 10)
    throw new TypeError('working genuine identity, order and ancestor comparisons');
  return compare;
}
const nodePositionCases = {
  metadata_and_cached_method: function() {
    const compare = positionPrerequisite(), descriptor = Object.getOwnPropertyDescriptor(Node.prototype,'compareDocumentPosition');
    positionCheck(descriptor.value === compare && descriptor.writable === true && descriptor.enumerable === true && descriptor.configurable === true, 'ordinary method descriptor');
    positionCheck(descriptor.get === undefined && descriptor.set === undefined, 'data property');
    const name = Object.getOwnPropertyDescriptor(compare,'name'), length = Object.getOwnPropertyDescriptor(compare,'length');
    positionCheck(name.value === 'compareDocumentPosition' && !name.writable && !name.enumerable && name.configurable, 'method name');
    positionCheck(length.value === 1 && !length.writable && !length.enumerable && length.configurable, 'required arity');
    positionKeys(Reflect.ownKeys(compare), ['length','name']);
    positionCheck(Object.getPrototypeOf(compare) === Function.prototype, 'Function inheritance');
    const leaf = new Text('guard'); let seen = 0;
    for (const key in leaf) if (key === 'compareDocumentPosition') seen++;
    positionCheck(seen === 1 && leaf.compareDocumentPosition === compare && Object.getOwnPropertyDescriptor(leaf,'compareDocumentPosition') === undefined, 'inherited cached identity');
    if (typeof Reflect.construct !== 'function') throw new TypeError('Reflect.construct prerequisite');
    const made = Reflect.construct(Text,['guard']);
    if (made.data !== 'guard' || compare.call(made,made) !== 0) throw new TypeError('working genuine constructor');
    positionTypeError(function() { Reflect.construct(compare,[leaf]); }, 'method nonconstructible');
    return true;
  },
  represented_complete_key_order: function() {
    positionPrerequisite();
    const strings = ['isConnected','nodeValue','textContent','getRootNode','hasChildNodes','normalize','isEqualNode','isSameNode','compareDocumentPosition','contains',
      'ELEMENT_NODE','ATTRIBUTE_NODE','TEXT_NODE','CDATA_SECTION_NODE','ENTITY_REFERENCE_NODE','ENTITY_NODE',
      'PROCESSING_INSTRUCTION_NODE','COMMENT_NODE','DOCUMENT_NODE','DOCUMENT_TYPE_NODE','DOCUMENT_FRAGMENT_NODE','NOTATION_NODE',
      'DOCUMENT_POSITION_DISCONNECTED','DOCUMENT_POSITION_PRECEDING','DOCUMENT_POSITION_FOLLOWING','DOCUMENT_POSITION_CONTAINS',
      'DOCUMENT_POSITION_CONTAINED_BY','DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC','constructor'];
    positionKeys(Object.getOwnPropertyNames(Node.prototype), strings);
    const keys = Reflect.ownKeys(Node.prototype);
    positionCheck(strings.length === 29 && keys.length === 30 && keys[29] === Symbol.toStringTag, 'conditional30 represented keys');
    for (let i = 0; i < 29; i++) positionCheck(keys[i] === strings[i], 'prototype string order');
    positionKeys(Object.keys(Node.prototype), strings.slice(0,28));
    positionKeys(Reflect.ownKeys(Node), ['length','name','prototype','ELEMENT_NODE','ATTRIBUTE_NODE','TEXT_NODE','CDATA_SECTION_NODE',
      'ENTITY_REFERENCE_NODE','ENTITY_NODE','PROCESSING_INSTRUCTION_NODE','COMMENT_NODE','DOCUMENT_NODE','DOCUMENT_TYPE_NODE',
      'DOCUMENT_FRAGMENT_NODE','NOTATION_NODE','DOCUMENT_POSITION_DISCONNECTED','DOCUMENT_POSITION_PRECEDING',
      'DOCUMENT_POSITION_FOLLOWING','DOCUMENT_POSITION_CONTAINS','DOCUMENT_POSITION_CONTAINED_BY','DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC']);
    const names = ['DOCUMENT_POSITION_DISCONNECTED','DOCUMENT_POSITION_PRECEDING','DOCUMENT_POSITION_FOLLOWING','DOCUMENT_POSITION_CONTAINS','DOCUMENT_POSITION_CONTAINED_BY','DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC'];
    const values = [1,2,4,8,16,32];
    for (let i = 0; i < 6; i++) positionCheck(Node[names[i]] === values[i] && Node.prototype[names[i]] === values[i], 'literal existing constants');
    return true;
  },
  authentic_receiver_brand_without_public_properties: function() {
    const compare = positionPrerequisite(), other = new Text('other'), fake = Object.create(Node.prototype); let reads = 0;
    for (const key of ['nodeType','parentNode','ownerDocument','getRootNode','childNodes','toString',Symbol.toPrimitive])
      Object.defineProperty(fake,key,{get:function() { reads++; throw new Error('forged property'); }});
    for (const receiver of [null,undefined,window,Node.prototype,fake,{},0,'',Symbol('receiver')])
      positionTypeError(function() { compare.call(receiver,other); }, 'authentic receiver required');
    positionTypeError(function() { compare(other); }, 'unbound method receiver');
    positionCheck(reads === 0 && compare.call(other,other) === 0, 'no duck typing or receiver coercion');
    return true;
  },
  required_nonnullable_argument_without_coercion: function() {
    const compare = positionPrerequisite(), node = new Text('x'), forged = Object.create(Node.prototype); let reads = 0;
    const poison = {};
    for (const key of ['toString','valueOf',Symbol.toPrimitive,'nodeType','parentNode'])
      Object.defineProperty(poison,key,{get:function() { reads++; throw new Error('argument hook'); }});
    if (typeof node.contains !== 'function' || typeof node.isSameNode !== 'function' || typeof node.isEqualNode !== 'function' ||
        node.contains(node) !== true || node.isSameNode(node) !== true || node.isEqualNode(node) !== true)
      throw new TypeError('working previous nullable predicates');
    positionCheck(node.contains(null) === false && node.isSameNode(undefined) === false && node.isEqualNode(null) === false, 'previous nullable inputs remain valid');
    positionTypeError(function() { compare.call(node); }, 'missing required argument');
    for (const other of [null,undefined,false,0,'node',Symbol('other'),{},forged,poison,Node.prototype])
      positionTypeError(function() { compare.call(node,other); }, 'nonnullable authentic Node argument');
    positionCheck(reads === 0 && compare.call(node,node) === 0, 'no interface-argument conversion hooks');
    return true;
  },
  identity_document_alias_and_structural_equality: function() {
    const compare = positionPrerequisite(), root = document.documentElement, a = new Text('equal'), b = new Text('equal'), fragment = new DocumentFragment();
    if (!root || root.parentNode !== document || typeof a.isEqualNode !== 'function' || a.isEqualNode(b) !== true)
      throw new TypeError('Document alias and structural equality prerequisites');
    positionCheck(compare.call(document,root.parentNode) === 0 && compare.call(root.parentNode,document) === 0, 'canonical Document identity');
    for (const node of [document,document.body,root,a,b,fragment]) positionCheck(compare.call(node,node) === 0, 'same authentic node');
    const disconnected = positionDisconnected(compare,a,b);
    b.data = 'different';
    positionCheck(compare.call(a,b) === disconnected && a.data === 'equal' && b.data === 'different', 'equality and payload do not collapse identity');
    return true;
  },
  ancestor_masks_at_different_depths: function() {
    const compare = positionPrerequisite(), outer = document.createElement('div'), middle = document.createElement('span'), inner = document.createElement('b'), leaf = new Text('leaf'), fragment = new DocumentFragment();
    inner.append(leaf); middle.append(inner); outer.append(middle); fragment.append(outer);
    for (const ancestor of [fragment,outer,middle,inner]) {
      positionCheck(compare.call(ancestor,leaf) === 20, 'other descendant includes following and contained-by');
      positionCheck(compare.call(leaf,ancestor) === 10, 'other ancestor includes preceding and contains');
    }
    positionCheck(compare.call(outer,inner) === 20 && compare.call(inner,outer) === 10, 'unequal depth alignment');
    document.body.append(fragment);
    positionCheck(compare.call(document,leaf) === 20 && compare.call(leaf,document) === 10, 'Document path after splice');
    positionCheck(fragment.childNodes.length === 0 && outer.parentNode === document.body && leaf.parentNode === inner, 'same branch retained');
    return true;
  },
  preorder_uses_child_positions_not_creation_order: function() {
    const compare = positionPrerequisite(), parent = document.createElement('div'), a = document.createElement('a'), b = document.createElement('b'), c = document.createElement('i');
    const at = new Text('a'), bt = new Comment('b'), ct = new Text('c'); a.append(at); b.append(bt); c.append(ct);
    parent.append(c,a,b);
    positionCheck(parent.childNodes[0] === c && parent.childNodes[1] === a && parent.childNodes[2] === b, 'literal reversed creation order');
    positionCheck(compare.call(c,a) === 4 && compare.call(a,c) === 2, 'sibling order');
    positionCheck(compare.call(ct,at) === 4 && compare.call(bt,ct) === 2, 'different branch leaf order');
    positionCheck(compare.call(ct,a) === 4 && compare.call(b,at) === 2, 'preorder across branch depth');
    positionCheck(compare.call(c,ct) === 20 && compare.call(ct,c) === 10, 'ancestor differs from plain branch order');
    return true;
  },
  moving_existing_siblings_changes_current_order: function() {
    const compare = positionPrerequisite(), parent = document.createElement('div'), a = new Text('a'), b = new Text('b'), c = new Text('c'); parent.append(a,b,c);
    positionCheck(compare.call(a,b) === 4 && compare.call(c,a) === 2, 'initial order');
    parent.append(a);
    positionCheck(parent.childNodes[0] === b && parent.childNodes[1] === c && parent.childNodes[2] === a, 'same nodes reordered');
    positionCheck(compare.call(a,b) === 2 && compare.call(c,a) === 4, 'fresh sibling positions');
    parent.append(b);
    positionCheck(compare.call(a,b) === 4 && compare.call(b,c) === 2, 'later move updates order again');
    const parked = new DocumentFragment(); parked.append(parent);
    positionCheck(compare.call(c,b) === 4 && compare.call(parent,b) === 20 && a.data === 'a' && b.data === 'b' && c.data === 'c', 'detaching common ancestor preserves internal order and payload');
    return true;
  },
  disconnected_masks_reversal_and_stable_existing_nodes: function() {
    const compare = positionPrerequisite(), left = document.createElement('div'), right = document.createElement('div'), alternative = new DocumentFragment();
    const a = new Text('a'), b = new Text('b'); left.append(a); right.append(b);
    const before = positionDisconnected(compare,a,b), roots = positionDisconnected(compare,left,right);
    const noise = new Text('unrelated');
    positionCheck(compare.call(a,b) === before && compare.call(left,right) === roots && compare.call(noise,noise) === 0, 'unrelated allocation does not alter masks');
    alternative.append(a);
    positionCheck(compare.call(a,b) === before && positionDisconnected(compare,a,b) === before, 'same operands remain disconnected after move');
    left.append(a);
    positionCheck(compare.call(a,b) === before && a.parentNode === left && b.parentNode === right, 'repeat stability after restoring parent');
    positionDisconnected(compare,document,a);
    positionDisconnected(compare,new DocumentFragment(),new Text('standalone'));
    return true;
  },
  fragments_splice_children_but_keep_detached_identity: function() {
    const compare = positionPrerequisite(), fragment = new DocumentFragment(), a = new Text('a'), b = new Comment('b'); fragment.append(a,b);
    positionCheck(compare.call(fragment,a) === 20 && compare.call(a,fragment) === 10 && compare.call(a,b) === 4, 'fragment ordinary tree');
    positionDisconnected(compare,fragment,document);
    document.body.append(fragment);
    positionCheck(fragment.childNodes.length === 0 && a.parentNode === document.body && b.parentNode === document.body, 'fragment itself remains detached');
    positionDisconnected(compare,fragment,a);
    positionCheck(compare.call(a,b) === 4 && compare.call(document,b) === 20 && compare.call(b,document) === 10, 'spliced children have fresh Document ancestry');
    fragment.append(b);
    positionDisconnected(compare,a,b);
    positionCheck(compare.call(fragment,b) === 20 && b.data === 'b', 'detached original child retained');
    return true;
  },
  template_content_and_ordinary_children_have_separate_roots: function() {
    const compare = positionPrerequisite(), template = document.createElement('template');
    if (!template.content || template.content.nodeType !== 11) throw new TypeError('actual template content');
    const ordinary = new Text('ordinary'), inside = new Text('inside'), tail = new Text('tail'); template.append(ordinary); template.content.append(inside,tail); document.body.append(template);
    positionCheck(compare.call(template,ordinary) === 20 && compare.call(document,ordinary) === 20, 'ordinary template child');
    positionDisconnected(compare,template,template.content);
    positionDisconnected(compare,ordinary,inside);
    positionCheck(compare.call(template.content,inside) === 20 && compare.call(inside,tail) === 4 && compare.call(tail,inside) === 2, 'content has its own ordered tree');
    template.append(inside);
    positionCheck(compare.call(template,inside) === 20 && compare.call(ordinary,inside) === 4 && template.content.firstChild === tail, 'move out of content changes only ordinary links');
    positionDisconnected(compare,inside,tail);
    positionCheck(inside.data === 'inside' && tail.data === 'tail', 'content move retains data');
    return true;
  },
  namespace_visibility_and_exact_data_do_not_decide_position: function() {
    const compare = positionPrerequisite(), box = document.createElement('div'); box.innerHTML = '<svg><g></g></svg><math><mi></mi></math>';
    const svg = box.firstChild, math = box.childNodes[1];
    if (svg.namespaceURI !== 'http://www.w3.org/2000/svg' || math.namespaceURI !== 'http://www.w3.org/1998/Math/MathML' || typeof ProcessingInstruction !== 'function')
      throw new TypeError('represented namespace and PI prerequisites');
    const text = new Text('A\uD800'), comment = new Comment('\uDC00'), pi = new ProcessingInstruction('probe','data'); box.append(text,comment,pi); box.setAttribute('style','display:none');
    positionCheck(compare.call(svg.firstChild,math.firstChild) === 4 && compare.call(pi,text) === 2 && compare.call(comment,pi) === 4, 'ordinary kind and namespace order');
    text.data = '\uDC00'; comment.data = 'different'; pi.data = '?>'; box.setAttribute('data-note','changed');
    positionCheck(compare.call(svg.firstChild,math.firstChild) === 4 && compare.call(pi,text) === 2 && compare.call(comment,pi) === 4, 'payload and attributes do not affect order');
    positionCheck(text.data.length === 1 && text.data.charCodeAt(0) === 56320 && pi.data === '?>', 'exact data mutations retained');
    return true;
  },
  private_links_ignore_public_fields_and_extra_values: function() {
    const compare = positionPrerequisite(), parent = document.createElement('div'), a = new Text('a'), b = new Text('b'), extra = {}; parent.append(a,b); let reads = 0;
    for (const node of [a,b]) for (const key of ['parentNode','ownerDocument','nodeType','getRootNode','isConnected'])
      Object.defineProperty(node,key,{get:function() { reads++; throw new Error('public field'); },configurable:true});
    Object.defineProperty(parent,'childNodes',{get:function() { reads++; throw new Error('public children'); },configurable:true});
    for (const key of ['composed','toString','valueOf',Symbol.toPrimitive])
      Object.defineProperty(extra,key,{get:function() { reads++; throw new Error('extra conversion'); }});
    positionCheck(compare.call(a,b,extra,null,undefined,Symbol('ignored')) === 4 && compare.call(parent,a,extra) === 20 && reads === 0, 'internal links and ignored extra values');
    parent.append(a);
    positionCheck(compare.call(a,b,extra) === 2 && reads === 0, 'fresh internal order after move');
    return true;
  },
  argument_expressions_finish_before_read_and_throw_prefix_survives: function() {
    const compare = positionPrerequisite(), parent = document.createElement('div'), a = new Text('a'), b = new Text('b'), marker = {}; parent.append(a,b); let trace = '';
    function receiver() { trace += 'R'; parent.append(a); return a; }
    function other() { trace += 'A'; parent.append(b); return b; }
    positionCheck(compare.call(receiver(),other()) === 4 && trace === 'RA', 'receiver then other expressions set final order');
    function extra() { trace += 'E'; parent.append(a); return {ignored:true}; }
    positionCheck(compare.call(a,b,extra()) === 2 && trace === 'RAE', 'ignored extra expression runs before native body');
    function throwing() { trace += 'T'; parent.append(b); throw marker; }
    let caught = false;
    try { compare.call(a,b,throwing()); } catch (error) { caught = error === marker; }
    positionCheck(caught && trace === 'RAET' && parent.childNodes[0] === a && parent.childNodes[1] === b, 'throw identity and completed move prefix');
    positionCheck(compare.call(a,b) === 4, 'saved native remains usable');
    return true;
  },
  own_shadows_prototype_replacement_deletion_and_saved_calls: function() {
    const compare = positionPrerequisite(), descriptor = Object.getOwnPropertyDescriptor(Node.prototype,'compareDocumentPosition');
    const parent = document.createElement('div'), a = new Text('a'), b = new Text('b'); parent.append(a,b);
    a.compareDocumentPosition = function() { return 99; };
    positionCheck(a.compareDocumentPosition(b) === 99 && compare.call(a,b) === 4, 'own ordinary function shadow');
    delete a.compareDocumentPosition;
    positionCheck(a.compareDocumentPosition === compare, 'delete reveals cached method');
    Object.defineProperty(Node.prototype,'compareDocumentPosition',{value:function() { return 98; },writable:true,enumerable:true,configurable:true});
    positionCheck(a.compareDocumentPosition(b) === 98 && compare.call(a,b) === 4, 'prototype replacement');
    delete Node.prototype.compareDocumentPosition;
    positionCheck(typeof a.compareDocumentPosition === 'undefined' && compare.call(a,b) === 4, 'no fallback resurrection');
    a.compareDocumentPosition = function() { return 97; };
    positionCheck(a.compareDocumentPosition(b) === 97, 'ordinary expando after prototype deletion');
    delete a.compareDocumentPosition; Object.defineProperty(Node.prototype,'compareDocumentPosition',descriptor);
    positionCheck(a.compareDocumentPosition === compare && a.compareDocumentPosition(b) === 4, 'saved exact descriptor restored');
    return true;
  },
  authentic_alternate_prototype_retains_node_brand: function() {
    const compare = positionPrerequisite();
    if (typeof Reflect.construct !== 'function') throw new TypeError('Reflect.construct prerequisite');
    const guard = Reflect.construct(Text,['guard']);
    if (guard.data !== 'guard' || compare.call(guard,guard) !== 0) throw new TypeError('working genuine constructor');
    const prototype = Object.create(null), target = function() {}.bind(null); let gets = 0;
    Object.defineProperty(target,'prototype',{get:function() { gets++; return prototype; }});
    const a = Reflect.construct(Text,['A\uD800'],target), b = new Text('b'), fragment = new DocumentFragment();
    const data = Object.getOwnPropertyDescriptor(CharacterData.prototype,'data').get;
    fragment.append(a,b);
    positionCheck(gets === 1 && Object.getPrototypeOf(a) === prototype && fragment.firstChild === a, 'authentic alternative prototype identity');
    positionCheck(compare.call(a,a) === 0 && compare.call(a,b) === 4 && compare.call(b,a) === 2 && compare.call(fragment,a) === 20, 'brand does not depend on JS prototype chain');
    const value = data.call(a);
    positionCheck(value.length === 2 && value.charCodeAt(1) === 55296, 'saved accessor sees exact original payload');
    positionTypeError(function() { compare.call(Object.create(prototype),a); }, 'lookalike remains forged');
    return true;
  }
};
