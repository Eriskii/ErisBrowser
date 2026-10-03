// Independent represented-tree equality cases. One named body per fresh realm.
// DOM https://dom.spec.whatwg.org/#concept-node-equals
// IDL https://webidl.spec.whatwg.org/#es-interface and #es-nullable-type
function equalCheck(value, message) { if (!value) throw new Error(message); }
function equalTypeError(body, message) {
  let caught = false;
  try { body(); } catch (error) { caught = error instanceof TypeError; }
  equalCheck(caught, message);
}
function equalUnits(value, expected) {
  equalCheck(value.length === expected.length, 'unit count');
  for (let i = 0; i < expected.length; i++) equalCheck(value.charCodeAt(i) === expected[i], 'literal unit');
}
function equalKeys(actual, expected) {
  equalCheck(actual.length === expected.length, 'key count');
  for (let i = 0; i < expected.length; i++) equalCheck(actual[i] === expected[i], 'literal key order');
}
function equalPrerequisite() {
  if (typeof Node !== 'function' || typeof Text !== 'function' || typeof DocumentFragment !== 'function')
    throw new TypeError('represented constructors');
  const method = Node.prototype.isEqualNode;
  if (typeof method !== 'function') throw new TypeError('isEqualNode callable');
  const a = new Text('guard'), b = new Text('guard'), different = new Text('different');
  if (a === b || method.call(a, a) !== true || method.call(a, b) !== true ||
      method.call(a, different) !== false || method.call(a, new DocumentFragment()) !== false)
    throw new TypeError('working genuine equality and inequality');
  return method;
}
const nodeEqualityCases = {
  metadata_and_inherited_identity: function() {
    const method = equalPrerequisite(), descriptor = Object.getOwnPropertyDescriptor(Node.prototype, 'isEqualNode');
    equalCheck(descriptor.value === method && descriptor.writable === true && descriptor.enumerable === true && descriptor.configurable === true, 'ordinary operation');
    equalCheck(descriptor.get === undefined && descriptor.set === undefined, 'data descriptor');
    const name = Object.getOwnPropertyDescriptor(method, 'name'), length = Object.getOwnPropertyDescriptor(method, 'length');
    equalCheck(name.value === 'isEqualNode' && !name.writable && !name.enumerable && name.configurable, 'name descriptor');
    equalCheck(length.value === 1 && !length.writable && !length.enumerable && length.configurable, 'required nullable arity');
    equalKeys(Reflect.ownKeys(method), ['length', 'name']);
    equalCheck(Object.getPrototypeOf(method) === Function.prototype, 'Function inheritance');
    for (const node of [document, document.createElement('div'), new DocumentFragment(), new Text(''), new Comment('')])
      equalCheck(node.isEqualNode === method && Object.getOwnPropertyDescriptor(node, 'isEqualNode') === undefined, 'shared inherited function');
    if (typeof Reflect.construct !== 'function') throw new TypeError('Reflect.construct prerequisite');
    const made = Reflect.construct(Text, ['guard']);
    if (made.data !== 'guard' || method.call(made, new Text('guard')) !== true) throw new TypeError('working Text construction');
    equalTypeError(function() { Reflect.construct(method, [made]); }, 'nonconstructible method');
    return true;
  },
  represented_complete_key_order: function() {
    equalPrerequisite();
    const strings = ['nodeValue','textContent','getRootNode','hasChildNodes','normalize','isEqualNode','isSameNode','contains',
      'ELEMENT_NODE','ATTRIBUTE_NODE','TEXT_NODE','CDATA_SECTION_NODE','ENTITY_REFERENCE_NODE','ENTITY_NODE',
      'PROCESSING_INSTRUCTION_NODE','COMMENT_NODE','DOCUMENT_NODE','DOCUMENT_TYPE_NODE','DOCUMENT_FRAGMENT_NODE','NOTATION_NODE',
      'DOCUMENT_POSITION_DISCONNECTED','DOCUMENT_POSITION_PRECEDING','DOCUMENT_POSITION_FOLLOWING','DOCUMENT_POSITION_CONTAINS',
      'DOCUMENT_POSITION_CONTAINED_BY','DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC','constructor'];
    equalKeys(Object.getOwnPropertyNames(Node.prototype), strings);
    const keys = Reflect.ownKeys(Node.prototype);
    equalCheck(keys.length === 28 && strings.length === 27 && keys[27] === Symbol.toStringTag, '28 represented keys');
    for (let i = 0; i < 27; i++) equalCheck(keys[i] === strings[i], 'prototype order');
    equalKeys(Object.keys(Node.prototype), strings.slice(0, 26));
    equalKeys(Reflect.ownKeys(Node), ['length','name','prototype','ELEMENT_NODE','ATTRIBUTE_NODE','TEXT_NODE','CDATA_SECTION_NODE',
      'ENTITY_REFERENCE_NODE','ENTITY_NODE','PROCESSING_INSTRUCTION_NODE','COMMENT_NODE','DOCUMENT_NODE','DOCUMENT_TYPE_NODE',
      'DOCUMENT_FRAGMENT_NODE','NOTATION_NODE','DOCUMENT_POSITION_DISCONNECTED','DOCUMENT_POSITION_PRECEDING',
      'DOCUMENT_POSITION_FOLLOWING','DOCUMENT_POSITION_CONTAINS','DOCUMENT_POSITION_CONTAINED_BY','DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC']);
    return true;
  },
  required_nullable_argument_without_coercion: function() {
    const method = equalPrerequisite(), node = new Text('x'); let reads = 0;
    equalCheck(method.call(node, null) === false && method.call(node, undefined) === false, 'explicit nullish is null');
    equalTypeError(function() { method.call(node); }, 'omitted required argument');
    const fake = {};
    for (const key of ['valueOf', 'toString', Symbol.toPrimitive]) Object.defineProperty(fake, key, {get: function() { reads++; throw new Error('coercion'); }});
    for (const value of [false, 0, '', Symbol('node'), fake]) equalTypeError(function() { method.call(node, value); }, 'authentic nullable Node conversion');
    equalCheck(reads === 0 && method.call(node, new Text('x')) === true, 'no conversion hooks and route remains usable');
    return true;
  },
  authentic_receiver_and_argument_brands: function() {
    const method = equalPrerequisite(), node = new Text('x'), fake = Object.create(Node.prototype); let reads = 0;
    for (const key of ['nodeType','data','parentNode']) Object.defineProperty(fake, key, {get: function() { reads++; throw new Error('fake property'); }});
    for (const value of [null, undefined, window, Node.prototype, fake, {}, 1, 'x']) {
      equalTypeError(function() { method.call(value, node); }, 'receiver brand');
      if (value !== null && value !== undefined) equalTypeError(function() { method.call(node, value); }, 'argument brand');
    }
    equalCheck(reads === 0 && method.call(node, node) === true, 'brands use authentic state');
    return true;
  },
  exact_character_data_and_distinct_kinds: function() {
    const method = equalPrerequisite(), a = new Text('A\uD800\uDC00\uD800'), b = new Text('A\uD800\uDC00\uD800');
    equalUnits(a.data, [65,55296,56320,55296]);
    equalCheck(a !== b && method.call(a,b) === true && method.call(b,a) === true, 'same exact data different identity');
    equalCheck(method.call(a,new Text('A\uD800\uDC00\uFFFD')) === false, 'unmatched surrogate is not replacement glyph');
    equalCheck(method.call(new Text('\u00E9'),new Text('e\u0301')) === false, 'no Unicode normalization');
    const c = new Comment('\uDC00'), d = new Comment('\uDC00');
    equalUnits(c.data,[56320]);
    equalCheck(method.call(c,d) === true && method.call(c,new Text('\uDC00')) === false, 'Comment identity and kind');
    equalCheck(method.call(new Text(''),new Comment('')) === false && method.call(new Text(''),new Text('')) === true, 'empty kinds');
    equalUnits(a.data,[65,55296,56320,55296]);
    return true;
  },
  processing_instruction_target_and_mutated_data: function() {
    const method = equalPrerequisite();
    if (typeof ProcessingInstruction !== 'function' || typeof document.createProcessingInstruction !== 'function') throw new TypeError('PI creation');
    const a = new ProcessingInstruction('Build','\uD800'), b = document.createProcessingInstruction('Build','\uD800');
    equalCheck(a.target === 'Build' && b.target === 'Build' && method.call(a,b) === true, 'PI constructor and factory equality');
    equalCheck(method.call(a,new ProcessingInstruction('build','\uD800')) === false, 'target case matters');
    equalCheck(method.call(a,new Comment('\uD800')) === false, 'PI is not Comment');
    a.data='?>\uDC00';b.data='?>\uDC00';
    equalUnits(a.data,[63,62,56320]);
    equalCheck(method.call(a,b) === true && a.target === 'Build' && b.target === 'Build', 'mutation data is compared without creation validation');
    b.data='?>';equalCheck(method.call(a,b) === false, 'data difference');
    return true;
  },
  element_namespaces_local_names_and_case: function() {
    const method = equalPrerequisite(), box = document.createElement('div');
    box.innerHTML='<svg><a></a><linearGradient></linearGradient></svg><math><mi></mi></math>';
    const svg=box.firstChild, math=box.childNodes[1], svgA=svg.firstChild, gradient=svg.childNodes[1];
    if (svg.namespaceURI !== 'http://www.w3.org/2000/svg' || math.namespaceURI !== 'http://www.w3.org/1998/Math/MathML' || gradient.nodeName !== 'linearGradient') throw new TypeError('parsed foreign prerequisites');
    const htmlA=document.createElement('a'), htmlSvg=document.createElement('svg');
    equalCheck(method.call(svgA,htmlA) === false && method.call(svg,htmlSvg) === false, 'namespace difference');
    equalCheck(method.call(document.createElement('DIV'),document.createElement('div')) === true, 'HTML creation lowercases local name');
    equalCheck(method.call(document.createElement('div'),document.createElement('span')) === false, 'local name difference');
    const peer=document.createElement('div');peer.innerHTML='<svg><a></a><linearGradient></linearGradient></svg><math><mi></mi></math>';
    equalCheck(method.call(box,peer) === true, 'mixed namespace descendants');
    peer.childNodes[1].firstChild.setAttribute('mathvariant','bold');
    equalCheck(method.call(box,peer) === false, 'foreign descendant attributes');
    return true;
  },
  unordered_attributes_and_exact_scalar_values: function() {
    const method=equalPrerequisite(), a=document.createElement('div'), b=document.createElement('div');
    a.setAttribute('title','\u00A0x');a.setAttribute('data-x','');
    b.setAttribute('data-x','');b.setAttribute('title','\u00A0x');
    equalCheck(method.call(a,b) === true && method.call(b,a) === true, 'attribute insertion order ignored');
    b.setAttribute('title',' x');equalCheck(method.call(a,b) === false, 'NBSP differs from ASCII space');
    b.setAttribute('title','\u00A0x');b.removeAttribute('data-x');
    equalCheck(method.call(a,b) === false, 'absent differs from empty attribute');
    b.setAttribute('data-x','');equalCheck(method.call(a,b) === true, 'restored attribute equality');
    a.extra={only:'a'};b.extra=42;a.onclick=function(){};
    equalCheck(method.call(a,b) === true, 'own bags and event listeners are not attributes');
    return true;
  },
  foreign_attribute_namespace_side_map_matters: function() {
    const method=equalPrerequisite(), box=document.createElement('div');
    box.innerHTML='<svg xlink:href="same"></svg><svg></svg>';
    const a=box.firstChild,b=box.childNodes[1];
    if (a.namespaceURI !== 'http://www.w3.org/2000/svg' || b.namespaceURI !== a.namespaceURI || typeof b.setAttribute !== 'function') throw new TypeError('SVG attribute setup');
    b.setAttribute('xlink:href','same');
    equalCheck(a.getAttribute('xlink:href') === 'same' && b.getAttribute('xlink:href') === 'same', 'same qualified key and value');
    equalCheck(method.call(a,b) === false && method.call(b,a) === false, 'foreign parsed namespace versus null namespace');
    if (typeof a.cloneNode !== 'function') throw new TypeError('clone prerequisite');
    const copy=a.cloneNode(false);
    equalCheck(copy !== a && copy.namespaceURI === a.namespaceURI && method.call(a,copy) === true, 'clone preserves namespace metadata');
    copy.setAttribute('xlink:href','changed');equalCheck(method.call(a,copy) === false, 'value differs within matching namespace');
    return true;
  },
  child_order_and_text_segmentation_are_structural: function() {
    const method=equalPrerequisite(), a=new DocumentFragment(),b=new DocumentFragment();
    const first=new Text('A'),second=new Text('B');a.append(first,second);b.append(new Text('AB'));
    equalCheck(a.textContent === 'AB' && b.textContent === 'AB' && method.call(a,b) === false, 'same aggregate different segmentation');
    b.textContent='';b.append(new Text('A'),new Text('B'));
    equalCheck(method.call(a,b) === true, 'matching segmentation');
    a.append(first);equalCheck(a.childNodes[0] === second && method.call(a,b) === false, 'child order matters');
    a.append(second);equalCheck(method.call(a,b) === true, 'order restored');
    b.append(new Comment(''));equalCheck(method.call(a,b) === false, 'empty Comment still a child');
    return true;
  },
  template_content_and_host_are_not_ordinary_children: function() {
    const method=equalPrerequisite(), a=document.createElement('template'),b=document.createElement('template');
    if (!a.content || !b.content || a.content.nodeType !== 11) throw new TypeError('template contents');
    a.content.append(new Text('A'));b.content.append(new Text('B'));
    equalCheck(method.call(a,b) === true && method.call(a.content,b.content) === false, 'content excluded from element equality');
    const loose=new DocumentFragment();loose.append(new Text('A'));
    equalCheck(method.call(a.content,loose) === true && method.call(loose,a.content) === true, 'fragment host excluded');
    a.append(new Text('ordinary'));equalCheck(method.call(a,b) === false, 'ordinary template child included');
    b.append(new Text('ordinary'));equalCheck(method.call(a,b) === true, 'matching ordinary children despite different content');
    return true;
  },
  connection_parent_and_document_identity_are_separate: function() {
    const method=equalPrerequisite(), a=document.createElement('div'),b=document.createElement('div');
    const childA=new Text('x'),childB=new Text('x');a.append(childA);b.append(childB);
    document.body.append(a);
    equalCheck(a !== b && method.call(a,b) === true && method.call(childA,childB) === true, 'connected versus detached equality');
    const parked=new DocumentFragment();parked.append(a);parked.textContent='';
    equalCheck(a.parentNode === null && childA.parentNode === a && method.call(a,b) === true, 'retained detached subtree');
    equalCheck(method.call(document,document) === true && method.call(document,document.documentElement.parentNode) === true, 'canonical Document alias');
    equalCheck(method.call(document,new DocumentFragment()) === false, 'Document differs from fragment');
    return true;
  },
  live_changes_and_argument_expression_order: function() {
    const method=equalPrerequisite(), a=new Text('x'),b=new Text('x');let trace='';
    const ignored={toString:function(){throw new Error('ignored coercion');}};
    function argument(){trace+='A';b.data='x';return b;}
    function extra(){trace+='B';b.data='y';return ignored;}
    equalCheck(method.call(a,argument(),extra()) === false && trace === 'AB', 'all argument expressions precede equality');
    b.data='x';equalCheck(method.call(a,b) === true, 'fresh data after restoring');
    const marker={};let caught;
    try { method.call({},(a.data='z',trace+='C',b),(function(){trace+='D';throw marker;})()); }
    catch(error){caught=error;}
    equalCheck(caught === marker && trace === 'ABCD' && a.data === 'z' && b.data === 'x', 'argument throw precedes brand and retains completed mutation');
    equalCheck(method.call(a,b) === false, 'next comparison reads current data');
    return true;
  },
  saved_method_shadow_replacement_and_deletion: function() {
    const method=equalPrerequisite(), a=new Text('x'),b=new Text('x'),saved=Object.getOwnPropertyDescriptor(Node.prototype,'isEqualNode');
    a.isEqualNode=function(){return 'own';};
    equalCheck(a.isEqualNode(b) === 'own' && method.call(a,b) === true, 'own shadow and saved method');
    delete a.isEqualNode;
    try {
      Node.prototype.isEqualNode=function(){return 'replacement';};
      equalCheck(a.isEqualNode(b) === 'replacement' && method.call(a,b) === true, 'prototype replacement');
      equalCheck(delete Node.prototype.isEqualNode, 'configurable method deletion');
      equalCheck(a.isEqualNode === undefined && b.isEqualNode === undefined && method.call(a,b) === true, 'no fallback resurrection');
    } finally { Object.defineProperty(Node.prototype,'isEqualNode',saved); }
    equalCheck(a.isEqualNode === method && b.isEqualNode === method, 'restored descriptor identity');
    return true;
  },
  internal_fields_ignore_public_shadows: function() {
    const method=equalPrerequisite(), a=document.createElement('div'),b=document.createElement('div'),ta=new Text('x'),tb=new Text('x');
    a.append(ta);b.append(tb);let reads=0;
    for(const node of [a,b,ta,tb]) for(const key of ['nodeType','nodeName','namespaceURI','prefix','localName','attributes','childNodes','parentNode','data','nodeValue','textContent'])
      Object.defineProperty(node,key,{configurable:true,get:function(){reads++;throw new Error('public getter');}});
    equalCheck(method.call(a,b) === true && method.call(ta,tb) === true && method.call(a,a) === true, 'internal values and ordered children');
    equalCheck(reads === 0, 'no authored property Get');
    return true;
  },
  authentic_alternate_prototype_does_not_change_equality: function() {
    const method=equalPrerequisite();
    if(typeof Reflect.construct !== 'function')throw new TypeError('Reflect.construct prerequisite');
    const guard=Reflect.construct(Text,['guard']);
    if(guard.data !== 'guard'||method.call(guard,new Text('guard')) !== true)throw new TypeError('working construction');
    const Alternate=(function(){}).bind(null),prototype=Object.create(null);
    Object.defineProperty(Alternate,'prototype',{value:prototype,configurable:true});
    const altered=Reflect.construct(Text,['\uD800'],Alternate),ordinary=new Text('\uD800');
    equalCheck(Object.getPrototypeOf(altered) === prototype && altered !== ordinary, 'genuine alternate prototype');
    equalCheck(method.call(altered,ordinary) === true && method.call(ordinary,altered) === true, 'implemented interface independent of JS prototype');
    const fake=Object.create(prototype);
    equalTypeError(function(){method.call(fake,ordinary);},'forged receiver');
    equalTypeError(function(){method.call(ordinary,fake);},'forged argument');
    return true;
  }
};
