// Independent small case; run in a fresh Runtime/Document, in either mode.
// Same normative slice as core-fixture.js; that aggregate remains unchanged.
(function () {
    function check(value, message) { if (!value) throw new Error(message); }
    function data(object, key, value, writable, enumerable, configurable) {
        var d = Object.getOwnPropertyDescriptor(object, key);
        check(d !== undefined && d.value === value && d.writable === writable &&
              d.enumerable === enumerable && d.configurable === configurable &&
              d.get === undefined && d.set === undefined, 'interface descriptor');
    }
    var rows = [[Node, 'Node', EventTarget], [Element, 'Element', Node],
                [Document, 'Document', Node], [DocumentFragment, 'DocumentFragment', Node],
                [CharacterData, 'CharacterData', Node], [Text, 'Text', CharacterData],
                [Comment, 'Comment', CharacterData]];
    for (var i = 0; i < rows.length; i++) {
        var C = rows[i][0], name = rows[i][1], parent = rows[i][2];
        check(typeof C === 'function' && Object.getPrototypeOf(C) === parent,
              'interface object inheritance');
        check(Object.getPrototypeOf(C.prototype) === parent.prototype, 'prototype inheritance');
        data(C, 'prototype', C.prototype, false, false, false);
        data(C, 'name', name, false, false, true);
        data(C, 'length', 0, false, false, true);
        data(C.prototype, 'constructor', C, true, false, true);
        data(C.prototype, Symbol.toStringTag, name, false, false, true);
        check(Object.prototype.toString.call(C.prototype) === '[object ' + name + ']',
              'prototype interface tag');
    }
    check(HTMLDocument === Document, 'HTMLDocument is an alias');
    check(Object.getPrototypeOf(document) === Document.prototype &&
          Object.prototype.toString.call(document) === '[object Document]', 'document identity');
    var fragment = document.createDocumentFragment(), text = document.createTextNode('x');
    var comment = new Comment('x'), element = document.createElement('section');
    check(Object.getPrototypeOf(fragment) === DocumentFragment.prototype &&
          Object.getPrototypeOf(text) === Text.prototype &&
          Object.getPrototypeOf(comment) === Comment.prototype, 'genuine node prototypes');
    check(document instanceof Document && Document.prototype.isPrototypeOf(document),
          'Document membership');
    check(fragment instanceof DocumentFragment && fragment instanceof Node &&
          Node.prototype.isPrototypeOf(fragment), 'fragment membership');
    check(text instanceof Text && text instanceof CharacterData && text instanceof EventTarget &&
          CharacterData.prototype.isPrototypeOf(text), 'Text membership');
    check(comment instanceof Comment && comment instanceof CharacterData && comment instanceof Node,
          'Comment membership');
    check(element instanceof Element && element instanceof Node && element instanceof EventTarget &&
          Element.prototype.isPrototypeOf(element), 'derived HTML membership');
    check(!(text instanceof Comment) && !(comment instanceof Text) &&
          !(fragment instanceof Element) && !(document instanceof Element), 'separate branches');
    return true;
})();
