// Preparation only. Each named body is an independent fresh-realm success case.
// No body has been parsed or executed. Literal expectations are not observations.
function textOpCheck(value, message) {
  if (!value) throw new Error(message);
}
function textOpUnits(value, expected) {
  textOpCheck(typeof value === 'string' && value.length === expected.length, 'unit length');
  for (var i = 0; i < expected.length; i++) {
    textOpCheck(value.charCodeAt(i) === expected[i], 'literal unit ' + i);
  }
}
function textOpPrerequisite() {
  if (typeof Text !== 'function' || typeof DocumentFragment !== 'function') {
    throw new TypeError('prerequisite: Text and DocumentFragment constructors');
  }
  var split = Object.getOwnPropertyDescriptor(Text.prototype, 'splitText');
  var whole = Object.getOwnPropertyDescriptor(Text.prototype, 'wholeText');
  if (!split || typeof split.value !== 'function' || !whole || typeof whole.get !== 'function') {
    throw new TypeError('prerequisite: authentic Text operation descriptors');
  }
  // Prove the exact saved functions work before testing errors/forged receivers.
  var probe = new Text('xy');
  var tail = split.value.call(probe, 1);
  if (probe.data !== 'x' || tail.data !== 'y' || whole.get.call(probe) !== 'x') {
    throw new TypeError('prerequisite: successful saved split and whole getter');
  }
  return { split: split.value, whole: whole.get, splitDescriptor: split, wholeDescriptor: whole };
}
function textOpTypeError(action) {
  var seen = false;
  try { action(); } catch (error) { seen = error instanceof TypeError; }
  textOpCheck(seen, 'expected TypeError');
}
function textOpIndexError(action) {
  var seen = false;
  try { action(); } catch (error) { seen = error.name === 'IndexSizeError' && error.code === 1; }
  textOpCheck(seen, 'expected IndexSizeError with legacy code 1');
}
var textOperationCases = {
  metadata: function () {
    var p = textOpPrerequisite();
    textOpCheck(p.splitDescriptor.writable === true && p.splitDescriptor.enumerable === true &&
                p.splitDescriptor.configurable === true, 'method flags');
    textOpCheck(p.wholeDescriptor.set === undefined && p.wholeDescriptor.enumerable === true &&
                p.wholeDescriptor.configurable === true, 'readonly getter flags');
    textOpCheck(p.split.name === 'splitText' && p.split.length === 1, 'method metadata');
    textOpCheck(p.whole.name === 'get wholeText' && p.whole.length === 0, 'getter metadata');
    textOpCheck(Object.getOwnPropertyDescriptor(CharacterData.prototype, 'splitText') === undefined &&
                Object.getOwnPropertyDescriptor(CharacterData.prototype, 'wholeText') === undefined,
                'Text owns the two members');
    var text = new Text('a');
    textOpCheck(Object.getOwnPropertyDescriptor(text, 'splitText') === undefined &&
                Object.getOwnPropertyDescriptor(text, 'wholeText') === undefined, 'inherited members');
    var keys = Object.getOwnPropertyNames(Text.prototype);
    var wholeAt = -1, splitAt = -1, constructorAt = -1;
    for (var i = 0; i < keys.length; i++) {
      if (keys[i] === 'wholeText') wholeAt = i;
      if (keys[i] === 'splitText') splitAt = i;
      if (keys[i] === 'constructor') constructorAt = i;
    }
    textOpCheck(wholeAt >= 0 && wholeAt < splitAt && splitAt < constructorAt,
                'Web IDL attributes before operations before constructor');
    return true;
  },
  detached_surrogate_boundary: function () {
    var p = textOpPrerequisite();
    var text = new Text('A\uD83D\uDE80B');
    var tail = p.split.call(text, 2);
    textOpCheck(tail !== text && text.parentNode === null && tail.parentNode === null, 'fresh detached tail');
    textOpUnits(text.data, [65, 55357]);
    textOpUnits(tail.data, [56960, 66]);
    textOpUnits(p.whole.call(text), [65, 55357]);
    textOpUnits(p.whole.call(tail), [56960, 66]);
    textOpCheck(Object.getPrototypeOf(tail) === Text.prototype && tail.nodeType === 3, 'ordinary new Text');
    return true;
  },
  attached_surrogate_boundary_and_identity: function () {
    var p = textOpPrerequisite();
    var box = new DocumentFragment();
    var left = new Text('L'), text = new Text('A\uD83D\uDE80B'), right = new Text('R');
    box.append(left, text, right);
    var tail = p.split.call(text, 2);
    var children = box.childNodes;
    textOpCheck(children.length === 4 && children[0] === left && children[1] === text &&
                children[2] === tail && children[3] === right && tail.parentNode === box, 'insert exactly after receiver');
    textOpUnits(text.data, [65, 55357]);
    textOpUnits(tail.data, [56960, 66]);
    for (var i = 0; i < children.length; i++) {
      textOpUnits(p.whole.call(children[i]), [76, 65, 55357, 56960, 66, 82]);
    }
    textOpUnits(box.textContent, [76, 65, 55357, 56960, 66, 82]);
    return true;
  },
  zero_end_and_empty_make_distinct_nodes: function () {
    var p = textOpPrerequisite();
    var box = new DocumentFragment(), text = new Text('ab');
    box.append(text);
    var all = p.split.call(text, 0);
    var end = p.split.call(all, 2);
    var empty = p.split.call(end, 0);
    var children = box.childNodes;
    textOpCheck(children.length === 4 && children[0] === text && children[1] === all &&
                children[2] === end && children[3] === empty, 'empty tails inserted, no coalescing');
    textOpUnits(text.data, []); textOpUnits(all.data, [97, 98]);
    textOpUnits(end.data, []); textOpUnits(empty.data, []);
    textOpCheck(end !== empty && all !== text, 'NewObject even at endpoints');
    textOpUnits(p.whole.call(empty), [97, 98]);
    return true;
  },
  whole_text_barriers_and_detached_runs: function () {
    var p = textOpPrerequisite();
    var box = new DocumentFragment();
    var a = new Text('a'), empty = new Text(''), b = new Text('b'), c = new Text('c');
    var d = new Text('d'), e = new Text('e'), comment = new Comment('ignored');
    var pi = new ProcessingInstruction('ok', 'ignored'), element = document.createElement('span');
    element.append(new Text('inside'));
    box.append(a, empty, b, comment, c, pi, d, element, e);
    textOpUnits(p.whole.call(a), [97, 98]); textOpUnits(p.whole.call(empty), [97, 98]);
    textOpUnits(p.whole.call(b), [97, 98]); textOpUnits(p.whole.call(c), [99]);
    textOpUnits(p.whole.call(d), [100]); textOpUnits(p.whole.call(e), [101]);
    textOpUnits(p.whole.call(element.firstChild), [105, 110, 115, 105, 100, 101]);
    textOpCheck(box.parentNode === null && box.childNodes.length === 9, 'detached tree unchanged');
    var isolated = new Text('\uD800');
    textOpUnits(p.whole.call(isolated), [55296]);
    return true;
  },
  conversion_uses_fresh_data_and_parent: function () {
    var p = textOpPrerequisite();
    var old = new DocumentFragment(), destination = new DocumentFragment(), text = new Text('a');
    var before = new Text('['), after = new Text(']'), calls = 0;
    old.append(text); destination.append(before, after);
    var tail = p.split.call(text, {valueOf: function () {
      calls++; text.data = 'WXYZ'; destination.append(text); return 1;
    }});
    textOpCheck(calls === 1 && old.childNodes.length === 0, 'conversion completed once');
    var children = destination.childNodes;
    textOpCheck(children.length === 4 && children[0] === before && children[1] === after &&
                children[2] === text && children[3] === tail, 'fresh destination');
    textOpUnits(text.data, [87]); textOpUnits(tail.data, [88, 89, 90]);
    textOpUnits(p.whole.call(text), [91, 93, 87, 88, 89, 90]);
    return true;
  },
  shrink_during_conversion_retains_callback_prefix: function () {
    var p = textOpPrerequisite();
    var box = new DocumentFragment(), text = new Text('long'), calls = 0;
    box.append(text);
    textOpIndexError(function () {
      p.split.call(text, {valueOf: function () { calls++; text.data = 'q'; return 2; }});
    });
    textOpCheck(calls === 1 && box.childNodes.length === 1 && box.firstChild === text, 'no outer insertion');
    textOpUnits(text.data, [113]);
    return true;
  },
  numeric_wrapping_required_argument_and_symbol: function () {
    var p = textOpPrerequisite();
    var offsets = [undefined, null, NaN, Infinity, -Infinity, -0, 4294967296, 4294967297, 1.9, -4294967295];
    var left = [[], [], [], [], [], [], [], [97], [97], [97]];
    var right = [[97,98], [97,98], [97,98], [97,98], [97,98], [97,98], [97,98], [98], [98], [98]];
    for (var i = 0; i < offsets.length; i++) {
      var text = new Text('ab'), tail = p.split.call(text, offsets[i]);
      textOpUnits(text.data, left[i]); textOpUnits(tail.data, right[i]);
    }
    var unchanged = new Text('ab');
    textOpTypeError(function () { p.split.call(unchanged); });
    textOpTypeError(function () { p.split.call(unchanged, Symbol('offset')); });
    textOpIndexError(function () { p.split.call(unchanged, -1); });
    textOpIndexError(function () { p.split.call(unchanged, 3); });
    textOpUnits(unchanged.data, [97, 98]);
    return true;
  },
  authentic_brand_precedes_offset_conversion: function () {
    var p = textOpPrerequisite(), calls = 0;
    var offset = {valueOf: function () { calls++; return 0; }};
    var receivers = [Object.create(Text.prototype), new Comment('x'),
                     new ProcessingInstruction('ok', 'x'), document.createElement('div'), {}, null];
    for (var i = 0; i < receivers.length; i++) {
      var receiver = receivers[i];
      textOpTypeError(function () { p.split.call(receiver, offset); });
      textOpTypeError(function () { p.whole.call(receiver); });
    }
    textOpCheck(calls === 0, 'invalid receiver was not coerced');
    return true;
  },
  throwing_conversion_preserves_exact_identity_and_effects: function () {
    var p = textOpPrerequisite(), box = new DocumentFragment(), text = new Text('old');
    var marker = {sentinel: true}, calls = 0, seen;
    box.append(text);
    try {
      p.split.call(text, {valueOf: function () {
        calls++; text.data = '\uD800x'; throw marker;
      }, toString: function () { throw new Error('must not reach fallback'); }});
    } catch (error) { seen = error; }
    textOpCheck(seen === marker && calls === 1 && box.childNodes.length === 1 && box.firstChild === text,
                'original exception and no outer mutation');
    textOpUnits(text.data, [55296, 120]);
    return true;
  },
  saved_functions_survive_property_replacement_and_deletion: function () {
    var p = textOpPrerequisite(), text = new Text('ab');
    try {
      Object.defineProperty(Text.prototype, 'wholeText', {value: 'shadow', writable: true,
                            enumerable: true, configurable: true});
      Object.defineProperty(Text.prototype, 'splitText', {value: function () { return 17; },
                            writable: true, enumerable: true, configurable: true});
      textOpCheck(text.wholeText === 'shadow' && text.splitText(0) === 17, 'replacement visible');
      textOpUnits(p.whole.call(text), [97, 98]);
      var tail = p.split.call(text, 1);
      textOpUnits(text.data, [97]); textOpUnits(tail.data, [98]);
      delete Text.prototype.wholeText; delete Text.prototype.splitText;
      textOpCheck(text.wholeText === undefined && text.splitText === undefined, 'no fallback resurrection');
      textOpUnits(p.whole.call(tail), [98]);
    } finally {
      Object.defineProperty(Text.prototype, 'wholeText', p.wholeDescriptor);
      Object.defineProperty(Text.prototype, 'splitText', p.splitDescriptor);
    }
    return true;
  },
  internal_slots_ignore_authored_public_getters: function () {
    var p = textOpPrerequisite(), box = new DocumentFragment(), text = new Text('ab'), right = new Text('!');
    var getData = Object.getOwnPropertyDescriptor(CharacterData.prototype, 'data').get;
    var calls = 0;
    function trap() { calls++; throw new Error('public getter must not run'); }
    box.append(text, right);
    Object.defineProperty(text, 'data', {get: trap, configurable: true});
    Object.defineProperty(text, 'length', {get: trap, configurable: true});
    Object.defineProperty(text, 'parentNode', {get: trap, configurable: true});
    Object.defineProperty(right, 'data', {get: trap, configurable: true});
    textOpUnits(p.whole.call(text), [97, 98, 33]);
    var tail = p.split.call(text, 1), children = box.childNodes;
    textOpCheck(calls === 0 && children.length === 3 && children[0] === text &&
                children[1] === tail && children[2] === right, 'internal structure used');
    textOpUnits(getData.call(text), [97]); textOpUnits(tail.data, [98]);
    return true;
  },
  result_uses_default_text_prototype: function () {
    var p = textOpPrerequisite(), alternate = Object.create(null), getterCalls = 0;
    function Target() {}
    var bound = Target.bind(null);
    Object.defineProperty(bound, 'prototype', {get: function () { getterCalls++; return alternate; }});
    var text = Reflect.construct(Text, ['ab'], bound);
    textOpCheck(Object.getPrototypeOf(text) === alternate && getterCalls === 1, 'alternate prototype prerequisite');
    var tail = p.split.call(text, 1);
    textOpCheck(Object.getPrototypeOf(text) === alternate && Object.getPrototypeOf(tail) === Text.prototype &&
                getterCalls === 1 && tail !== text, 'ordinary fresh Text, no newTarget repeat');
    var getData = Object.getOwnPropertyDescriptor(CharacterData.prototype, 'data').get;
    textOpUnits(getData.call(text), [97]); textOpUnits(tail.data, [98]);
    return true;
  },
  whole_text_exact_pair_join_without_mutation: function () {
    var p = textOpPrerequisite(), box = new DocumentFragment();
    var high = new Text('\uD83D'), empty = new Text(''), low = new Text('\uDE80'), lone = new Text('\uD800');
    box.append(high, empty, low, lone);
    textOpUnits(p.whole.call(low), [55357, 56960, 55296]);
    textOpUnits(high.data, [55357]); textOpUnits(empty.data, []);
    textOpUnits(low.data, [56960]); textOpUnits(lone.data, [55296]);
    var children = box.childNodes;
    textOpCheck(children.length === 4 && children[0] === high && children[1] === empty &&
                children[2] === low && children[3] === lone, 'getter neither merges nor repairs stored nodes');
    return true;
  }
};
