// Independently authored ordinary-success fixture; run unchanged in either mode.
// Sources and deliberately separate feature gaps are in core-fixture-notes.md.
(function () {
  function check(value, message) { if (!value) throw new Error(message); }
  function throwsTypeError(action) {
    var error; try { action(); } catch (caught) { error = caught; }
    check(error instanceof TypeError, 'expected TypeError');
  }
  function data(object, key, value, writable, enumerable, configurable) {
    var label = typeof key === 'symbol' ? 'symbol' : key;
    var d = Object.getOwnPropertyDescriptor(object, key);
    check(d !== undefined && d.value === value, 'data property value: ' + label);
    check(d.writable === writable && d.enumerable === enumerable && d.configurable === configurable,
          'data property flags: ' + label);
    check(d.get === undefined && d.set === undefined, 'data property kind: ' + label);
    return d;
  }
  function enumerated(object, wanted) {
    var count = 0;
    for (var key in object) if (key === wanted) count++;
    return count;
  }
  function parentMethodsEnumerate(object) {
    var found = [0, 0, 0];
    for (var key in object) {
      if (key === 'querySelector') found[0]++;
      else if (key === 'querySelectorAll') found[1]++;
      else if (key === 'append') found[2]++;
    }
    check(found.join(',') === '1,1,1', 'all inherited ParentNode operations enumerate once');
  }
  function implementsChain(object, constructors) {
    for (var i = 0; i < constructors.length; i++) {
      var C = constructors[i];
      check(object instanceof C, 'instanceof ' + C.name);
      check(C.prototype.isPrototypeOf(object), 'isPrototypeOf ' + C.name);
    }
  }

  var interfaces = [
    [Node, 'Node', EventTarget], [Element, 'Element', Node],
    [Document, 'Document', Node], [DocumentFragment, 'DocumentFragment', Node],
    [CharacterData, 'CharacterData', Node], [Text, 'Text', CharacterData],
    [Comment, 'Comment', CharacterData]
  ];
  for (var i = 0; i < interfaces.length; i++) {
    var C = interfaces[i][0], name = interfaces[i][1], parent = interfaces[i][2];
    check(typeof C === 'function', 'interface function: ' + name);
    check(Object.getPrototypeOf(C) === parent, 'interface-object inheritance: ' + name);
    check(Object.getPrototypeOf(C.prototype) === parent.prototype, 'prototype inheritance: ' + name);
    data(C, 'prototype', C.prototype, false, false, false);
    data(C, 'name', name, false, false, true);
    data(C, 'length', 0, false, false, true);
    data(C.prototype, 'constructor', C, true, false, true);
    data(C.prototype, Symbol.toStringTag, name, false, false, true);
    check(Object.prototype.toString.call(C.prototype) === '[object ' + name + ']', 'interface tag');
  }
  var constants = [
    ['ELEMENT_NODE', 1], ['ATTRIBUTE_NODE', 2], ['TEXT_NODE', 3], ['CDATA_SECTION_NODE', 4],
    ['ENTITY_REFERENCE_NODE', 5], ['ENTITY_NODE', 6], ['PROCESSING_INSTRUCTION_NODE', 7],
    ['COMMENT_NODE', 8], ['DOCUMENT_NODE', 9], ['DOCUMENT_TYPE_NODE', 10],
    ['DOCUMENT_FRAGMENT_NODE', 11], ['NOTATION_NODE', 12],
    ['DOCUMENT_POSITION_DISCONNECTED', 1], ['DOCUMENT_POSITION_PRECEDING', 2],
    ['DOCUMENT_POSITION_FOLLOWING', 4], ['DOCUMENT_POSITION_CONTAINS', 8],
    ['DOCUMENT_POSITION_CONTAINED_BY', 16], ['DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC', 32]
  ];
  for (var k = 0; k < constants.length; k++) {
    data(Node, constants[k][0], constants[k][1], false, true, false);
    data(Node.prototype, constants[k][0], constants[k][1], false, true, false);
  }
  check(HTMLDocument === Document, 'HTMLDocument aliases Document');
  check(Object.prototype.toString.call(document) === '[object Document]', 'document inherits its interface tag');
  check(Object.getPrototypeOf(document) === Document.prototype, 'document immediate prototype');
  var element = document.createElement('section'), child = document.createElement('i');
  var fragment = document.createDocumentFragment(), fragmentChild = document.createElement('i');
  element.appendChild(child); fragment.appendChild(fragmentChild);
  var text = document.createTextNode('factory'), comment = new Comment('comment');
  check(Object.getPrototypeOf(fragment) === DocumentFragment.prototype, 'factory fragment prototype');
  check(Object.getPrototypeOf(text) === Text.prototype, 'factory text prototype');
  check(Object.getPrototypeOf(comment) === Comment.prototype, 'comment prototype');
  // HTML elements can have more-derived HTML prototypes; do not flatten them.
  implementsChain(element, [Element, Node, EventTarget]);
  implementsChain(document, [Document, Node, EventTarget]);
  implementsChain(fragment, [DocumentFragment, Node, EventTarget]);
  implementsChain(text, [Text, CharacterData, Node, EventTarget]);
  implementsChain(comment, [Comment, CharacterData, Node, EventTarget]);
  check(!(text instanceof Comment) && !(comment instanceof Text), 'distinct CharacterData branches');
  check(!(fragment instanceof Element) && !(document instanceof Element), 'distinct Node branches');

  var owners = [Document.prototype, Element.prototype, DocumentFragment.prototype];
  var targets = [document, element, fragment];
  var selectors = ['html', 'i', 'i'], answers = [document.documentElement, child, fragmentChild];
  var methods = ['querySelector', 'querySelectorAll', 'append'], saved = [];
  function invoke(method, key, target, selector, answer) {
    if (key === 'append') check(method.call(target) === undefined, 'empty native append');
    else if (key === 'querySelector') check(method.call(target, selector) === answer, 'native query');
    else {
      var list = method.call(target, selector);
      check(list.length === 1 && list[0] === answer, 'native query all');
    }
  }
  function operation(owner, target, key, selector, answer, enumerableReceiver) {
    var method = target[key];
    check(typeof method === 'function', 'saved native operation');
    var original = data(owner, key, method, true, true, true);
    check(Object.getOwnPropertyDescriptor(target, key) === undefined, 'operation is inherited');
    check(Object.getOwnPropertyDescriptor(Node.prototype, key) === undefined, 'mixin is not on Node');
    data(method, 'name', key, false, false, true);
    data(method, 'length', key === 'append' ? 0 : 1, false, false, true);
    for (var j = 0; j < saved.length; j++) check(saved[j] !== method, 'distinct nine function identities');
    saved.push(method);
    invoke(method, key, target, selector, answer);

    var forged = Object.create(owner), conversions = 0;
    check(owner.isPrototypeOf(forged) && forged instanceof owner.constructor, 'forged prototype membership');
    throwsTypeError(function () {
      method.call(forged, {toString: function () { conversions++; return 'i'; }});
    });
    check(conversions === 0, 'interface brand precedes argument conversion');

    var marker = {}, replacement = function () {
      check(this === target, 'replacement uses actual receiver'); return marker;
    };
    try {
      owner[key] = replacement;
      check(target[key] === replacement && target[key]() === marker, 'prototype replacement visible');
      check(Object.getOwnPropertyDescriptor(target, key) === undefined, 'no instance copy');
      check(delete owner[key], 'delete configurable prototype operation');
      check(target[key] === undefined && Object.getOwnPropertyDescriptor(owner, key) === undefined,
            'deleted operation must not resurrect through host fallback');
      if (enumerableReceiver !== undefined)
        check(enumerated(enumerableReceiver, key) === 0, 'deleted operation not enumerated');
      invoke(method, key, target, selector, answer);
    } finally {
      Object.defineProperty(owner, key, original);
    }
    check(target[key] === method, 'native restored explicitly');
    if (enumerableReceiver !== undefined)
      check(enumerated(enumerableReceiver, key) === 1, 'restored operation enumerated');
  }
  for (var ownerIndex = 0; ownerIndex < owners.length; ownerIndex++) {
    // Enumerating a plain inheritor tests Document.prototype without requiring
    // the separately unsupported complete own-key list of the Document host.
    var enumerableReceiver = ownerIndex === 0 ? Object.create(owners[0]) : targets[ownerIndex];
    var unscopables = owners[ownerIndex][Symbol.unscopables];
    data(owners[ownerIndex], Symbol.unscopables, unscopables, false, false, true);
    check(Object.getPrototypeOf(unscopables) === null, 'null-prototype unscopables');
    data(unscopables, 'append', true, true, true, true);
    parentMethodsEnumerate(enumerableReceiver);
    for (var methodIndex = 0; methodIndex < methods.length; methodIndex++) {
      operation(owners[ownerIndex], targets[ownerIndex], methods[methodIndex],
                selectors[ownerIndex], answers[ownerIndex],
                ownerIndex === 1 && methodIndex === 0 ? enumerableReceiver : undefined);
    }
  }
  check(saved.length === 9, 'all nine ParentNode operations checked');

  var fresh = new DocumentFragment(), madeText = new Text('text'), madeComment = new Comment('note');
  check(Object.getPrototypeOf(fresh) === DocumentFragment.prototype && fresh.nodeType === 11,
        'genuine new fragment');
  check(Object.getPrototypeOf(madeText) === Text.prototype && madeText.nodeType === 3,
        'genuine new text');
  check(Object.getPrototypeOf(madeComment) === Comment.prototype && madeComment.nodeType === 8,
        'genuine new comment');
  check(madeText.textContent === 'text' && madeComment.textContent === 'note', 'constructor data');
  fresh.append(madeText, madeComment);
  check(fresh.childNodes.length === 2 && fresh.childNodes[0] === madeText && fresh.childNodes[1] === madeComment,
        'constructed nodes participate in native tree operations');
  check(fresh.textContent === 'text', 'comment excluded from descendant text');
  check(new Text().textContent === '' && new Text(undefined).textContent === '' &&
        new Text(null).textContent === 'null', 'Text optional DOMString defaults');
  check(new Comment().textContent === '' && new Comment(undefined).textContent === '' &&
        new Comment(null).textContent === 'null', 'Comment optional DOMString defaults');
  var ignored = 0;
  new DocumentFragment({toString: function () { ignored++; throw new Error('ignored'); }});
  check(ignored === 0, 'fragment ignores extra arguments');
  var noConversion = {toString: function () { ignored++; return 'x'; }};
  for (var j = 0; j < 3; j++) {
    var illegal = [Node, Element, CharacterData][j];
    throwsTypeError(function () { illegal(noConversion); });
    throwsTypeError(function () { new illegal(noConversion); });
    var constructible = [DocumentFragment, Text, Comment][j];
    throwsTypeError(function () { constructible(noConversion); });
  }
  check(ignored === 0, 'call/illegal construction rejects before conversion');

  function customConstruction(C, type) {
    var trace = '', prototype = Object.create(C.prototype);
    var NewTarget = (function () {}).bind(null);
    Object.defineProperty(NewTarget, 'prototype', {
      get: function () { trace += 'P'; return prototype; }, configurable: true
    });
    var input = {toString: function () { trace += 'S'; return 'data'; }};
    var value = Reflect.construct(C, [input], NewTarget);
    check(trace === (type === 11 ? 'P' : 'SP'), 'conversion precedes newTarget prototype');
    check(Object.getPrototypeOf(value) === prototype && value instanceof C, 'custom prototype retained');
    check(value.nodeType === type && value.textContent === (type === 11 ? '' : 'data'), 'native construction survives override');
    trace = '';
    prototype = 1;
    value = Reflect.construct(C, [], NewTarget);
    check(trace === 'P' && Object.getPrototypeOf(value) === C.prototype, 'primitive prototype uses same-realm default');
    if (type !== 11) {
      var marker = {}, caught;
      trace = '';
      try {
        Reflect.construct(C, [{toString: function () { trace += 'S'; throw marker; }}], NewTarget);
      } catch (error) { caught = error; }
      check(caught === marker && trace === 'S', 'abrupt conversion prevents prototype access');
      trace = ''; caught = undefined;
      Object.defineProperty(NewTarget, 'prototype', {
        get: function () { trace += 'P'; throw marker; }, configurable: true
      });
      try { Reflect.construct(C, [input], NewTarget); } catch (error) { caught = error; }
      check(caught === marker && trace === 'SP', 'prototype failure follows conversion and preserves identity');
    }
  }
  customConstruction(DocumentFragment, 11);
  customConstruction(Text, 3);
  customConstruction(Comment, 8);
  return true;
})();
