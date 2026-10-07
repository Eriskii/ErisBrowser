// Independently authored represented clone cases: one body per fresh realm.
// Each body accepts the harness's explicit Boolean mode argument but ignores it.
function cloneCheck(value, message) { if (!value) throw new Error(message); }
function cloneTypeError(body, message) {
  let caught = false;
  try { body(); } catch (error) { caught = error instanceof TypeError; }
  cloneCheck(caught, message);
}
function cloneUnits(value, expected) {
  cloneCheck(value.length === expected.length, 'literal unit count');
  for (let i = 0; i < expected.length; i++) cloneCheck(value.charCodeAt(i) === expected[i], 'literal UTF16 unit');
}
function cloneKeys(actual, expected) {
  cloneCheck(actual.length === expected.length, 'literal key count');
  for (let i = 0; i < expected.length; i++) cloneCheck(actual[i] === expected[i], 'literal key order');
}
function clonePrerequisite() {
  if (typeof Node !== 'function' || typeof Text !== 'function' || typeof DocumentFragment !== 'function')
    throw new TypeError('represented constructors');
  const descriptor = Object.getOwnPropertyDescriptor(Node.prototype, 'cloneNode');
  if (!descriptor || typeof descriptor.value !== 'function') throw new TypeError('ordinary clone method');
  const method = descriptor.value, source = new Text('guard'), copy = method.call(source);
  if (copy === source || copy.nodeType !== 3 || copy.data !== 'guard' || copy.parentNode !== null || source.data !== 'guard')
    throw new TypeError('working fresh genuine clone');
  return method;
}
const nodeCloneCases = {
  metadata_and_cached_method: function() {
    const method = clonePrerequisite(), descriptor = Object.getOwnPropertyDescriptor(Node.prototype, 'cloneNode');
    cloneCheck(descriptor.value === method && descriptor.writable === true && descriptor.enumerable === true && descriptor.configurable === true, 'ordinary operation descriptor');
    cloneCheck(descriptor.get === undefined && descriptor.set === undefined, 'data descriptor');
    const name = Object.getOwnPropertyDescriptor(method, 'name'), length = Object.getOwnPropertyDescriptor(method, 'length');
    cloneCheck(name.value === 'cloneNode' && !name.writable && !name.enumerable && name.configurable, 'name descriptor');
    cloneCheck(length.value === 0 && !length.writable && !length.enumerable && length.configurable, 'optional Boolean arity');
    cloneKeys(Reflect.ownKeys(method), ['length', 'name']);
    cloneCheck(Object.getPrototypeOf(method) === Function.prototype, 'function inheritance');
    for (const node of [document, document.createElement('div'), new DocumentFragment(), new Text(''), new Comment('')])
      cloneCheck(node.cloneNode === method && Object.getOwnPropertyDescriptor(node, 'cloneNode') === undefined, 'shared inherited method');
    if (typeof Reflect.construct !== 'function') throw new TypeError('Reflect.construct prerequisite');
    const made = Reflect.construct(Text, ['guard']);
    if (made.data !== 'guard' || method.call(made).data !== 'guard') throw new TypeError('working positive construction');
    cloneTypeError(function() { Reflect.construct(method, []); }, 'method is not constructible');
    return true;
  },
  represented_complete_key_order: function() {
    clonePrerequisite();
    const strings = ['isConnected','nodeValue','textContent','getRootNode','hasChildNodes','normalize','cloneNode','isEqualNode','isSameNode','compareDocumentPosition','contains',
      'ELEMENT_NODE','ATTRIBUTE_NODE','TEXT_NODE','CDATA_SECTION_NODE','ENTITY_REFERENCE_NODE','ENTITY_NODE',
      'PROCESSING_INSTRUCTION_NODE','COMMENT_NODE','DOCUMENT_NODE','DOCUMENT_TYPE_NODE','DOCUMENT_FRAGMENT_NODE','NOTATION_NODE',
      'DOCUMENT_POSITION_DISCONNECTED','DOCUMENT_POSITION_PRECEDING','DOCUMENT_POSITION_FOLLOWING','DOCUMENT_POSITION_CONTAINS',
      'DOCUMENT_POSITION_CONTAINED_BY','DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC','constructor'];
    cloneKeys(Object.getOwnPropertyNames(Node.prototype), strings);
    const keys = Reflect.ownKeys(Node.prototype);
    cloneCheck(strings.length === 30 && keys.length === 31 && keys[30] === Symbol.toStringTag, '31 represented prototype keys');
    for (let i = 0; i < 30; i++) cloneCheck(keys[i] === strings[i], 'complete prototype order');
    cloneKeys(Object.keys(Node.prototype), strings.slice(0, 29));
    cloneKeys(Reflect.ownKeys(Node), ['length','name','prototype','ELEMENT_NODE','ATTRIBUTE_NODE','TEXT_NODE','CDATA_SECTION_NODE',
      'ENTITY_REFERENCE_NODE','ENTITY_NODE','PROCESSING_INSTRUCTION_NODE','COMMENT_NODE','DOCUMENT_NODE','DOCUMENT_TYPE_NODE',
      'DOCUMENT_FRAGMENT_NODE','NOTATION_NODE','DOCUMENT_POSITION_DISCONNECTED','DOCUMENT_POSITION_PRECEDING',
      'DOCUMENT_POSITION_FOLLOWING','DOCUMENT_POSITION_CONTAINS','DOCUMENT_POSITION_CONTAINED_BY','DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC']);
    return true;
  },
  authentic_receiver_without_public_property_reads: function() {
    const method = clonePrerequisite(), fake = Object.create(Node.prototype), poison = {}; let reads = 0;
    for (const key of ['nodeType','nodeName','parentNode','childNodes','data']) Object.defineProperty(fake, key, {get: function() { reads++; throw new Error('public fake field'); }});
    for (const key of ['valueOf','toString',Symbol.toPrimitive]) Object.defineProperty(poison, key, {get: function() { reads++; throw new Error('Boolean hook'); }});
    for (const receiver of [null,undefined,window,Node.prototype,fake,{},false,1,'text',Symbol('node')])
      cloneTypeError(function() { method.call(receiver, poison); }, 'authentic receiver first');
    cloneCheck(reads === 0 && method.call(new Text('ok'), poison).data === 'ok', 'no public brand or Boolean conversion hooks');
    return true;
  },
  optional_boolean_defaults_and_truthy_objects: function() {
    const method = clonePrerequisite(), source = new DocumentFragment(), text = new Text('child'); source.append(text);
    const omitted = method.call(source);
    cloneCheck(omitted !== source && omitted.nodeType === 11 && omitted.childNodes.length === 0, 'missing Boolean defaults false');
    for (const value of [undefined,null,false,0,-0,0/0,'']) {
      const copy = method.call(source, value);
      cloneCheck(copy !== source && copy.childNodes.length === 0 && copy.parentNode === null, 'falsy shallow copy');
    }
    let reads = 0; const poison = {};
    for (const key of ['valueOf','toString',Symbol.toPrimitive]) Object.defineProperty(poison, key, {get: function() { reads++; throw new Error('conversion'); }});
    for (const value of [true,1,-1,'false',[],function(){},Symbol('deep'),poison]) {
      const copy = method.call(source, value);
      cloneCheck(copy !== source && copy.childNodes.length === 1 && copy.firstChild !== text && copy.firstChild.data === 'child' && copy.firstChild.parentNode === copy, 'truthy deep copy');
    }
    cloneCheck(reads === 0 && source.childNodes.length === 1 && source.firstChild === text && text.parentNode === source, 'source and poisoned objects unchanged');
    return true;
  },
  argument_expressions_finish_before_copy_and_throw: function() {
    const method = clonePrerequisite(), source = new DocumentFragment(), child = new Text('before'); source.append(child); let trace = '';
    function receiver() { trace += 'R'; return source; }
    function deep() { trace += 'A'; child.data = 'first'; return true; }
    function extra() { trace += 'E'; child.data = 'final'; return {toString:function(){throw new Error('ignored');}}; }
    const copy = method.call(receiver(), deep(), extra());
    cloneCheck(trace === 'RAE' && copy.firstChild.data === 'final' && child.data === 'final' && copy.firstChild !== child, 'all expressions precede fresh copy');
    const marker = {}; let caught;
    try { method.call({}, (trace += 'M', child.data = 'retained', true), (function(){trace += 'T'; throw marker;})()); }
    catch (error) { caught = error; }
    cloneCheck(caught === marker && trace === 'RAEMT' && child.data === 'retained' && copy.firstChild.data === 'final', 'expression throw precedes brand and keeps earlier effects');
    return true;
  },
  exact_text_comment_data_and_independent_mutation: function() {
    const method = clonePrerequisite();
    const sources = [new Text(''),new Text('A\uD83D\uDE80B'),new Text('\uD800\uDC00\uD800'),new Comment('\uDC00\r\n\u0000'),new Text('e\u0301')];
    const units = [[],[65,55357,56960,66],[55296,56320,55296],[56320,13,10,0],[101,769]];
    for (let i = 0; i < sources.length; i++) {
      const source = sources[i], a = method.call(source), b = method.call(source,true);
      cloneCheck(a !== source && b !== source && a !== b && a.nodeType === source.nodeType && b.nodeType === source.nodeType, 'fresh exact leaf kind');
      cloneCheck(a.parentNode === null && b.parentNode === null && a.childNodes.length === 0 && b.childNodes.length === 0, 'detached childless leaves');
      cloneUnits(source.data,units[i]);cloneUnits(a.data,units[i]);cloneUnits(b.data,units[i]);
      a.data='changed';cloneUnits(source.data,units[i]);cloneUnits(b.data,units[i]);
    }
    return true;
  },
  processing_instruction_mutated_data_is_not_revalidated: function() {
    const method = clonePrerequisite();
    if (typeof ProcessingInstruction !== 'function' || typeof document.createProcessingInstruction !== 'function') throw new TypeError('PI prerequisites');
    const source = document.createProcessingInstruction('Build','start'); source.data='?>\uD800\u0000';
    cloneUnits(source.data,[63,62,55296,0]);
    const copy = method.call(source,true);
    cloneCheck(copy !== source && copy.nodeType === 7 && copy.target === 'Build' && copy.nodeName === 'Build' && copy.parentNode === null, 'fresh PI target and kind');
    cloneUnits(copy.data,[63,62,55296,0]);copy.data='other';
    cloneUnits(source.data,[63,62,55296,0]);cloneCheck(source.target === 'Build', 'source PI retained');
    return true;
  },
  shallow_element_fields_without_children: function() {
    const method = clonePrerequisite(), source = document.createElement('div'), child = new Text('old');
    source.setAttribute('id','copy-id');source.setAttribute('data-empty','');source.setAttribute('title','\u00A0literal');source.append(child);
    const copy = method.call(source,false);
    cloneCheck(copy !== source && copy.nodeType === 1 && copy.nodeName === 'DIV' && copy.namespaceURI === 'http://www.w3.org/1999/xhtml' && copy.parentNode === null, 'fresh represented element');
    cloneCheck(copy.childNodes.length === 0 && copy.getAttribute('id') === 'copy-id' && copy.getAttribute('data-empty') === '' && copy.getAttribute('missing') === null && copy.getAttribute('title') === '\u00A0literal', 'exact scalar attributes without children');
    copy.setAttribute('title','changed');copy.removeAttribute('data-empty');source.setAttribute('id','source-id');
    cloneCheck(source.getAttribute('title') === '\u00A0literal' && source.getAttribute('data-empty') === '' && copy.getAttribute('id') === 'copy-id' && source.firstChild === child && child.parentNode === source, 'independent attributes and retained child');
    return true;
  },
  deep_current_child_order_and_text_segmentation: function() {
    const method = clonePrerequisite(), source = document.createElement('section'), branch = document.createElement('b');
    const a = new Text('A'), empty = new Text(''), b = new Text('B'), comment = new Comment('mark'), nested = new Text('C');
    branch.append(nested);source.append(a,empty,b,comment,branch);source.append(a);
    const copy = method.call(source,true), children = copy.childNodes;
    cloneCheck(copy !== source && children.length === 5 && copy.parentNode === null, 'fresh root and five current children');
    cloneCheck(children[0].nodeType === 3 && children[0].data === '' && children[1].nodeType === 3 && children[1].data === 'B' && children[2].nodeType === 8 && children[2].data === 'mark' && children[3].nodeName === 'B' && children[4].data === 'A', 'literal order preserves empty segmentation');
    const originals = [empty,b,comment,branch,a];
    for (let i = 0; i < 5; i++) cloneCheck(children[i] !== originals[i] && children[i].parentNode === copy && source.childNodes[i] === originals[i] && originals[i].parentNode === source, 'fresh reciprocal children retain source');
    cloneCheck(children[3].childNodes.length === 1 && children[3].firstChild !== nested && children[3].firstChild.data === 'C' && children[3].firstChild.parentNode === children[3], 'nested fresh descendant');
    children[3].firstChild.data='new';cloneCheck(nested.data === 'C' && source.textContent === 'BCA' && copy.textContent === 'BnewA', 'independent deep data');
    return true;
  },
  foreign_namespaces_and_attribute_annotations: function() {
    const method = clonePrerequisite(), box = document.createElement('div');
    box.innerHTML='<svg viewBox="0 0 3 3"><linearGradient xlink:href="same"></linearGradient></svg><math><mi mathvariant="bold">x</mi></math>';
    const svg = box.firstChild, math = box.childNodes[1], gradient = svg.firstChild;
    if (svg.namespaceURI !== 'http://www.w3.org/2000/svg' || gradient.nodeName !== 'linearGradient' || math.namespaceURI !== 'http://www.w3.org/1998/Math/MathML') throw new TypeError('foreign parsed prerequisites');
    const copy = method.call(box,true), cs = copy.firstChild, cm = copy.childNodes[1], cg = cs.firstChild;
    cloneCheck(cs !== svg && cg !== gradient && cm !== math && cs.namespaceURI === 'http://www.w3.org/2000/svg' && cg.nodeName === 'linearGradient' && cs.getAttribute('viewBox') === '0 0 3 3' && cg.getAttribute('xlink:href') === 'same', 'SVG fields copied exactly');
    cloneCheck(cm.namespaceURI === 'http://www.w3.org/1998/Math/MathML' && cm.firstChild.nodeName === 'mi' && cm.firstChild.getAttribute('mathvariant') === 'bold' && cm.firstChild.firstChild.data === 'x', 'MathML fields and data');
    const equal = Node.prototype.isEqualNode, negativeBox = document.createElement('div');negativeBox.innerHTML='<svg><linearGradient></linearGradient></svg>';
    const negative = negativeBox.firstChild.firstChild;negative.setAttribute('xlink:href','same');
    if (typeof equal !== 'function' || equal.call(gradient,negative) !== false || negative.getAttribute('xlink:href') !== 'same') throw new TypeError('namespace annotation negative prerequisite');
    cloneCheck(equal.call(gradient,cg) === true && equal.call(cg,negative) === false, 'copied namespace annotation distinguished from null namespace');
    cg.setAttribute('xlink:href','changed');cloneCheck(gradient.getAttribute('xlink:href') === 'same', 'source foreign attribute retained');
    return true;
  },
  fragment_copy_detachment_and_later_splice: function() {
    const method = clonePrerequisite(), source = new DocumentFragment(), a = new Text('A'), b = new Comment('B');source.append(a,b);
    const shallow = method.call(source,false), deep = method.call(source,true), ca = deep.firstChild, cb = deep.childNodes[1];
    cloneCheck(shallow !== source && deep !== source && shallow !== deep && shallow.nodeType === 11 && deep.nodeType === 11 && shallow.childNodes.length === 0 && deep.childNodes.length === 2, 'separate fragment results');
    cloneCheck(ca !== a && cb !== b && ca.data === 'A' && cb.data === 'B' && ca.parentNode === deep && cb.parentNode === deep && deep.parentNode === null, 'fresh fragment children');
    const destination = document.createElement('div');destination.append(deep);
    cloneCheck(deep.childNodes.length === 0 && deep.parentNode === null && destination.childNodes[0] === ca && destination.childNodes[1] === cb && ca.parentNode === destination && cb.parentNode === destination, 'copy children splice by identity');
    cloneCheck(source.childNodes[0] === a && source.childNodes[1] === b && a.parentNode === source && b.parentNode === source, 'source fragment unchanged');
    document.body.append(destination);const detached = method.call(destination,true);
    cloneCheck(destination.isConnected === true && detached.isConnected === false && detached.parentNode === null && detached.firstChild !== ca && detached.firstChild.data === 'A', 'connected source yields detached copy');
    return true;
  },
  shallow_template_has_fresh_empty_content: function() {
    const method = clonePrerequisite(), source = document.createElement('template'), ordinary = new Text('ordinary'), inside = new Text('inside');
    if (!source.content || source.content.nodeType !== 11) throw new TypeError('template content');
    source.content.append(inside);source.append(ordinary);source.setAttribute('data-template','yes');
    const a = method.call(source), b = method.call(source,false);
    cloneCheck(a !== source && b !== source && a !== b && a.nodeName === 'TEMPLATE' && a.getAttribute('data-template') === 'yes', 'fresh shallow template hosts');
    cloneCheck(a.content !== source.content && b.content !== source.content && a.content !== b.content && a.content.nodeType === 11 && b.content.nodeType === 11, 'fresh content even when shallow');
    cloneCheck(a.childNodes.length === 0 && b.childNodes.length === 0 && a.content.childNodes.length === 0 && b.content.childNodes.length === 0 && a.content.parentNode === null && a.content.getRootNode() === a.content, 'separate empty content root');
    cloneCheck(source.firstChild === ordinary && source.content.firstChild === inside && ordinary.parentNode === source && inside.parentNode === source.content, 'source host and content unchanged');
    return true;
  },
  deep_nested_template_copies_both_distinct_graphs: function() {
    const method = clonePrerequisite(), outer = document.createElement('template'), inner = document.createElement('template');
    if (!outer.content || !inner.content) throw new TypeError('nested template content');
    const outerOrd = new Text('outer ordinary'), innerOrd = new Comment('inner ordinary'), exact = new Text('A\uD800'), tail = new Text('tail');
    inner.append(innerOrd);inner.content.append(exact);outer.content.append(inner,tail);outer.append(outerOrd);
    const copy = method.call(outer,true), ci = copy.content.firstChild;
    cloneCheck(copy !== outer && copy.content !== outer.content && copy.childNodes.length === 1 && copy.firstChild !== outerOrd && copy.firstChild.data === 'outer ordinary', 'outer ordinary graph copied');
    cloneCheck(copy.content.childNodes.length === 2 && ci !== inner && ci.nodeName === 'TEMPLATE' && ci.parentNode === copy.content && copy.content.childNodes[1] !== tail && copy.content.childNodes[1].data === 'tail', 'content child order');
    cloneCheck(ci.content !== inner.content && ci.content !== copy.content && ci.childNodes.length === 1 && ci.firstChild !== innerOrd && ci.firstChild.nodeType === 8 && ci.firstChild.data === 'inner ordinary', 'nested ordinary graph and fresh content');
    cloneCheck(ci.content.childNodes.length === 1 && ci.content.firstChild !== exact && ci.content.firstChild.parentNode === ci.content && ci.content.parentNode === null && ci.content.getRootNode() === ci.content, 'nested content reciprocal ordinary links');
    cloneUnits(ci.content.firstChild.data,[65,55296]);ci.content.firstChild.data='changed';cloneUnits(exact.data,[65,55296]);
    cloneCheck(outer.firstChild === outerOrd && outer.content.firstChild === inner && inner.firstChild === innerOrd && inner.content.firstChild === exact, 'all original graph identities retained');
    return true;
  },
  cloned_template_content_is_an_independent_fragment: function() {
    const method = clonePrerequisite(), host = document.createElement('template'), inside = new Text('\uDC00');
    if (!host.content) throw new TypeError('template content');host.content.append(inside);
    document.body.append(host);const copy = method.call(host.content,true), child = copy.firstChild;
    cloneCheck(copy !== host.content && copy.nodeType === 11 && copy.parentNode === null && copy.getRootNode() === copy && copy.isConnected === false, 'copied hosted fragment is an ordinary detached root');
    cloneCheck(child !== inside && child.parentNode === copy && child.getRootNode() === copy && host.content.firstChild === inside && inside.parentNode === host.content, 'source and copy stay separate');
    cloneUnits(child.data,[56320]);
    const destination = document.createElement('div');document.body.append(destination);destination.append(copy);
    cloneCheck(copy.childNodes.length === 0 && copy.isConnected === false && destination.firstChild === child && child.isConnected === true && host.content.firstChild === inside && inside.isConnected === false, 'copied fragment splices without changing hosted source');
    return true;
  },
  detached_details_and_base_copies_preserve_source_state: function() {
    const method = clonePrerequisite(), holder = document.createElement('div'), source = document.createElement('details'), summary = document.createElement('summary');
    summary.append(new Text('Summary'));source.append(new Text('before'),summary);source.setAttribute('name','clone-group');source.setAttribute('open','');holder.append(source);document.body.append(holder);
    if (source.getAttribute('open') !== '' || source.childNodes[1] !== summary) throw new TypeError('open connected details prerequisite');
    const copy = method.call(source,true);
    cloneCheck(source.getAttribute('open') === '' && copy.getAttribute('open') === '' && copy.getAttribute('name') === 'clone-group' && copy.parentNode === null, 'detached clone does not close source group');
    cloneCheck(copy.childNodes.length === 2 && copy.childNodes[1] !== summary && copy.childNodes[1].nodeName === 'SUMMARY' && copy.childNodes[1].firstChild.data === 'Summary' && source.childNodes[1] === summary, 'fresh summary and retained source identity');
    holder.append(copy);
    cloneCheck(source.getAttribute('open') === '' && copy.getAttribute('open') === null, 'insertion closes new same-name member');
    copy.setAttribute('open','');cloneCheck(source.getAttribute('open') === null && copy.getAttribute('open') === '', 'later copied member participates in its new group');
    const first = document.createElement('base');first.setAttribute('href','https://clone-source.example/a/');document.body.append(first);
    if (document.baseURI !== 'https://clone-source.example/a/') throw new TypeError('connected first base prerequisite');
    const copiedBase = method.call(first,false);
    cloneCheck(copiedBase !== first && copiedBase.parentNode === null && copiedBase.getAttribute('href') === 'https://clone-source.example/a/' && document.baseURI === 'https://clone-source.example/a/', 'detached base copy preserves active selection');
    copiedBase.setAttribute('href','https://clone-copy.example/b/');
    cloneCheck(first.getAttribute('href') === 'https://clone-source.example/a/' && document.baseURI === 'https://clone-source.example/a/', 'detached copied href does not change original base');
    document.body.append(copiedBase);cloneCheck(document.baseURI === 'https://clone-source.example/a/', 'later copied base does not replace first');
    first.remove();cloneCheck(document.baseURI === 'https://clone-copy.example/b/' && first.getAttribute('href') === 'https://clone-source.example/a/', 'explicit removal selects copied base and retains detached original');
    return true;
  },
  clone_omits_own_state_listeners_and_prototype_override: function() {
    const method = clonePrerequisite(), source = document.createElement('button'), token = Symbol('private');let calls = 0, handlerCalls = 0;
    source.extra = {value:1};source[token] = 'source';source.setAttribute('data-kept','yes');source.append(new Text('button'));
    if (typeof source.addEventListener !== 'function' || typeof source.dispatchEvent !== 'function' || typeof Event !== 'function') throw new TypeError('event prerequisites');
    source.addEventListener('clone-probe',function(){calls++;});source.onclick=function(){handlerCalls++;};
    source.dispatchEvent(new Event('clone-probe'));source.dispatchEvent(new Event('click'));
    if (calls !== 1 || handlerCalls !== 1) throw new TypeError('working source listeners and assigned handler');
    const copy = method.call(source,true);
    cloneCheck(copy.extra === undefined && copy[token] === undefined && copy.getAttribute('data-kept') === 'yes', 'own bags omitted but attributes copied');
    copy.dispatchEvent(new Event('clone-probe'));copy.dispatchEvent(new Event('click'));
    cloneCheck(calls === 1 && handlerCalls === 1, 'listeners and assigned event handler not copied');
    source.dispatchEvent(new Event('clone-probe'));source.dispatchEvent(new Event('click'));
    cloneCheck(calls === 2 && handlerCalls === 2, 'source listeners retained');
    if (typeof Reflect.construct !== 'function') throw new TypeError('alternative construction prerequisite');
    const alternative = {marker:'source prototype'};function Alternate(){} Alternate.prototype=alternative;
    const text = Reflect.construct(Text,['exact'],Alternate);
    if (Object.getPrototypeOf(text) !== alternative) throw new TypeError('working alternative prototype');
    text.extra='not copied';const fresh = method.call(text);
    cloneCheck(fresh !== text && Object.getPrototypeOf(fresh) === Text.prototype && fresh.data === 'exact' && fresh.extra === undefined && fresh.marker === undefined, 'new intrinsic interface without source override');
    return true;
  },
  saved_method_shadows_replacement_and_no_resurrection: function() {
    const method = clonePrerequisite(), source = new Text('saved'), descriptor = Object.getOwnPropertyDescriptor(Node.prototype,'cloneNode');
    source.cloneNode=function(){return 'own';};cloneCheck(source.cloneNode() === 'own' && method.call(source).data === 'saved', 'own shadow and saved method');delete source.cloneNode;
    try {
      Node.prototype.cloneNode=function(){return 'replacement';};cloneCheck(source.cloneNode() === 'replacement' && method.call(source).data === 'saved', 'ordinary prototype replacement');
      cloneCheck(delete Node.prototype.cloneNode, 'configurable deletion');
      cloneCheck(source.cloneNode === undefined && document.createElement('div').cloneNode === undefined && document.cloneNode === undefined, 'legacy and virtual routes do not resurrect');
      const copy = method.call(source,true);cloneCheck(copy !== source && copy.data === 'saved' && copy.cloneNode === undefined, 'saved native remains callable after deletion');
    } finally { Object.defineProperty(Node.prototype,'cloneNode',descriptor); }
    cloneCheck(source.cloneNode === method && method.call(source).data === 'saved', 'restored ordinary method');
    return true;
  }
};
