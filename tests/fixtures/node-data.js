// Independent ordinary-success candidates. Load definitions, then invoke exactly
// one nodeDataCases[name]() in a fresh realm, separately in sloppy/strict mode.
// No execution has occurred. Doctype and terminal quota tests need Rust setup.
// https://dom.spec.whatwg.org/#dom-node-nodevalue
// https://dom.spec.whatwg.org/#dom-node-textcontent
// https://webidl.spec.whatwg.org/#es-nullable-type
// https://webidl.spec.whatwg.org/#es-attributes
function nodeDataEqual(actual, expected) {
  if (actual !== expected) throw new Error('Node data expectation');
}
function nodeDataDescriptor(name) {
  const d = Object.getOwnPropertyDescriptor(Node.prototype, name);
  if (!d || typeof d.get !== 'function' || typeof d.set !== 'function') {
    throw new Error('Node data accessor prerequisite');
  }
  return d;
}
var nodeDataCases = {
  descriptors_and_represented_getters: function () {
    const names = ['nodeValue', 'textContent'];
    for (let i = 0; i < names.length; i++) {
      const name = names[i], d = nodeDataDescriptor(name);
      nodeDataEqual(d.enumerable, true); nodeDataEqual(d.configurable, true);
      nodeDataEqual(d.get.name, 'get ' + name); nodeDataEqual(d.get.length, 0);
      nodeDataEqual(d.set.name, 'set ' + name); nodeDataEqual(d.set.length, 1);
      nodeDataEqual(Object.getOwnPropertyDescriptor(document, name), undefined);
      nodeDataEqual(d.get.call(document), null);
      const t = new Text('A\uD83C\uDF3F'), c = new Comment('comment');
      const p = new ProcessingInstruction('probe', 'instruction');
      nodeDataEqual(d.get.call(t), 'A\uD83C\uDF3F');
      nodeDataEqual(d.get.call(c), 'comment'); nodeDataEqual(d.get.call(p), 'instruction');
    }
    const e = document.createElement('div'), nested = document.createElement('span');
    nested.append(new Text('B')); e.append(new Text('A'), new Comment('ignored'), nested);
    e.append(new ProcessingInstruction('probe', 'ignored'));
    const f = new DocumentFragment(); f.append(e, new Text('C'));
    nodeDataEqual(e.nodeValue, null); nodeDataEqual(f.nodeValue, null);
    nodeDataEqual(e.textContent, 'AB'); nodeDataEqual(f.textContent, 'ABC');
    return true;
  },
  nullable_setter_defaults: function () {
    const names = ['nodeValue', 'textContent'];
    for (let i = 0; i < names.length; i++) {
      const d = nodeDataDescriptor(names[i]);
      const t = new Text('kept');
      nodeDataEqual(d.set.call(t, null), undefined); nodeDataEqual(t.data, '');
      t.data = 'again'; d.set.call(t, undefined); nodeDataEqual(t.data, '');
      t.data = 'again'; d.set.call(t); nodeDataEqual(t.data, '');
      d.set.call(t, {toString: function () {return 'undefined';}});
      nodeDataEqual(t.data, 'undefined');
      d.set.call(t, {toString: function () {return null;}});
      nodeDataEqual(t.data, 'null');
      const c = new Comment('before'), p = new ProcessingInstruction('probe', 'before');
      d.set.call(c, null); d.set.call(p, '?>');
      nodeDataEqual(c.data, ''); nodeDataEqual(p.data, '?>'); nodeDataEqual(p.target, 'probe');
    }
    const e = document.createElement('div'); e.append(new Text('old'));
    e.textContent = undefined; nodeDataEqual(e.firstChild, null);
    return true;
  },
  ignored_kinds_still_convert: function () {
    const value = nodeDataDescriptor('nodeValue'), content = nodeDataDescriptor('textContent');
    const e = document.createElement('div'), f = new DocumentFragment();
    e.append(new Text('kept')); f.append(new Text('also kept'));
    let trace = '';
    const input = {toString: function () {trace += 'C'; return '\uD800';}};
    value.set.call(e, input); value.set.call(f, input);
    value.set.call(document, input); content.set.call(document, input);
    nodeDataEqual(trace, 'CCCC'); nodeDataEqual(e.textContent, 'kept');
    nodeDataEqual(f.textContent, 'also kept'); nodeDataEqual(document.textContent, null);
    const marker = {}; let observed = false;
    try {content.set.call(document, {toString: function () {trace += 'T'; throw marker;}});}
    catch (error) {observed = error === marker;}
    nodeDataEqual(observed, true); nodeDataEqual(trace, 'CCCCT');
    let symbolError = false;
    try {value.set.call(e, Symbol('ignored'));} catch (error) {symbolError = error instanceof TypeError;}
    nodeDataEqual(symbolError, true); nodeDataEqual(e.textContent, 'kept');
    return true;
  },
  authentic_brand_before_conversion: function () {
    const names = ['nodeValue', 'textContent']; let converted = 0;
    const input = {toString: function () {converted++; return 'changed';}};
    const forged = Object.create(Node.prototype);
    for (let i = 0; i < names.length; i++) {
      const d = nodeDataDescriptor(names[i]); let getError = false, setError = false;
      try {d.get.call(forged);} catch (error) {getError = error instanceof TypeError;}
      try {d.set.call(forged, input);} catch (error) {setError = error instanceof TypeError;}
      nodeDataEqual(getError, true); nodeDataEqual(setError, true); nodeDataEqual(converted, 0);
      const real = new Text('kept'); d.set.call(real, 'changed');
      nodeDataEqual(d.get.call(real), 'changed');
    }
    return true;
  },
  character_data_uses_current_state: function () {
    const names = ['nodeValue', 'textContent'];
    for (let i = 0; i < names.length; i++) {
      const d = nodeDataDescriptor(names[i]), t = new Text('old');
      let trace = '';
      const value = {toString: function () {trace += 'C'; t.data = 'callback-data'; return 'final';}};
      d.set.call(t, value); nodeDataEqual(trace, 'C'); nodeDataEqual(t.data, 'final');
      const marker = {}; let observed = false;
      try {d.set.call(t, {toString: function () {t.data = 'retained'; throw marker;}});}
      catch (error) {observed = error === marker;}
      nodeDataEqual(observed, true); nodeDataEqual(t.data, 'retained');
    }
    return true;
  },
  container_replacement_uses_current_children: function () {
    const d = nodeDataDescriptor('textContent'), e = document.createElement('div');
    const branch = document.createElement('span'), old = new Text('old');
    branch.append(old); e.append(branch);
    const added = new Text('callback'); let converted = 0;
    d.set.call(e, {toString: function () {converted++; e.append(added); return 'final';}});
    nodeDataEqual(converted, 1); nodeDataEqual(e.textContent, 'final');
    nodeDataEqual(branch.parentNode, null); nodeDataEqual(old.parentNode, branch);
    nodeDataEqual(old.data, 'old'); nodeDataEqual(added.parentNode, null);
    nodeDataEqual(added.data, 'callback');
    const previous = e.firstChild; d.set.call(e, 'final');
    nodeDataEqual(e.firstChild === previous, false); nodeDataEqual(previous.parentNode, null);
    nodeDataEqual(previous.data, 'final');
    const marker = {}, retained = new Text('retained'); let observed = false;
    try {d.set.call(e, {toString: function () {e.append(retained); throw marker;}});}
    catch (error) {observed = error === marker;}
    nodeDataEqual(observed, true); nodeDataEqual(retained.parentNode, e);
    nodeDataEqual(e.textContent, 'finalretained');
    const f = new DocumentFragment(); f.append(e); d.set.call(f, 'replacement');
    nodeDataEqual(e.parentNode, null); nodeDataEqual(f.firstChild.data, 'replacement');
    d.set.call(f, ''); nodeDataEqual(f.firstChild, null);
    return true;
  },
  ordinary_shadow_and_prototype_deletion: function () {
    const names = ['nodeValue', 'textContent'];
    for (let i = 0; i < names.length; i++) {
      const name = names[i], d = nodeDataDescriptor(name), t = new Text('native');
      Object.defineProperty(t, name, {value: 'own', writable: true, configurable: true});
      nodeDataEqual(t[name], 'own'); nodeDataEqual(d.get.call(t), 'native');
      delete t[name]; nodeDataEqual(t[name], 'native');
      let receiver, supplied;
      Object.defineProperty(Node.prototype, name, {get: function () {return 'replacement';},
        set: function (v) {receiver = this; supplied = v;}, enumerable: true, configurable: true});
      const input = {}; t[name] = input;
      nodeDataEqual(receiver, t); nodeDataEqual(supplied, input); nodeDataEqual(t.data, 'native');
      nodeDataEqual(t[name], 'replacement');
      delete Node.prototype[name]; nodeDataEqual(t[name], undefined);
      nodeDataEqual(document[name], undefined);
      t[name] = 'expando'; nodeDataEqual(t[name], 'expando'); nodeDataEqual(t.data, 'native');
      delete t[name]; d.set.call(t, 'saved'); nodeDataEqual(t.data, 'saved');
      Object.defineProperty(Node.prototype, name, d); nodeDataEqual(t[name], 'saved');
    }
    return true;
  },
  template_content_is_separate: function () {
    nodeDataDescriptor('textContent');
    const template = document.createElement('template');
    template.content.append(new Text('hidden'));
    nodeDataEqual(template.textContent, '');
    template.textContent = 'direct';
    nodeDataEqual(template.firstChild.data, 'direct'); nodeDataEqual(template.textContent, 'direct');
    nodeDataEqual(template.content.textContent, 'hidden');
    template.textContent = null; nodeDataEqual(template.firstChild, null);
    nodeDataEqual(template.content.textContent, 'hidden');
    return true;
  }
};
