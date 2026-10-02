// Independent ordinary-success cases. Run one named function in a fresh realm.
// Original nodeDataCases are preserved separately without edits.
// https://dom.spec.whatwg.org/#dom-node-nodevalue
// https://dom.spec.whatwg.org/#dom-node-textcontent
// https://webidl.spec.whatwg.org/#es-nullable-type
function exactNodeCheck(value, message) { if (!value) throw new Error(message); }
function exactNodeUnits(value, expected) {
  exactNodeCheck(value.length === expected.length, 'UTF16 length');
  for (let i = 0; i < expected.length; i++)
    exactNodeCheck(value.charCodeAt(i) === expected[i], 'UTF16 unit ' + i);
}
function exactNodeDescriptor(name) {
  const d = Object.getOwnPropertyDescriptor(Node.prototype, name);
  exactNodeCheck(d && typeof d.get === 'function' && typeof d.set === 'function', 'accessor prerequisite');
  return d;
}
var exactNodeDataCases = {
  character_data_exact_identity: function () {
    const names = ['nodeValue', 'textContent'];
    for (let i = 0; i < names.length; i++) {
      const d = exactNodeDescriptor(names[i]), box = document.createElement('div');
      const text = new Text('old'), comment = new Comment('old');
      const pi = new ProcessingInstruction('probe', 'old');
      box.append(text, comment, pi);
      const nodes = [text, comment, pi];
      for (let k = 0; k < nodes.length; k++) {
        const node = nodes[k]; d.set.call(node, 'A\uD800B\uDC00');
        exactNodeUnits(d.get.call(node), [65,55296,66,56320]);
        exactNodeUnits(node.data, [65,55296,66,56320]);
        exactNodeCheck(node.parentNode === box && box.childNodes[k] === node, 'same CharacterData identity');
      }
      d.set.call(pi, '?>\uD800');
      exactNodeUnits(pi.data, [63,62,55296]);
      exactNodeCheck(pi.target === 'probe', 'PI target retained');
    }
    return true;
  },
  cross_node_pair_and_text_only_aggregation: function () {
    exactNodeDescriptor('textContent');
    const box = document.createElement('div'), nested = document.createElement('span');
    const high = new Text('A\uD800'), low = new Text('\uDC00B');
    nested.append(low, new ProcessingInstruction('probe', 'excluded'));
    const template = document.createElement('template');
    template.content.append(new Text('inert'));
    box.append(high, new Comment('excluded'), nested, template, new Text('\uD800'));
    exactNodeUnits(box.textContent, [65,55296,56320,66,55296]);
    exactNodeUnits(high.data, [65,55296]); exactNodeUnits(low.data, [56320,66]);
    exactNodeCheck(box.nodeValue === null && nested.nodeValue === null, 'container nodeValue');
    exactNodeUnits(JSON.parse(JSON.stringify(box.textContent)), [65,55296,56320,66,55296]);
    const copy = box.cloneNode(true);
    exactNodeUnits(copy.textContent, [65,55296,56320,66,55296]);
    exactNodeCheck(copy !== box && copy.firstChild !== high, 'independent clone');
    return true;
  },
  equal_container_write_still_creates_new_text: function () {
    exactNodeDescriptor('textContent');
    const box = document.createElement('div'), branch = document.createElement('span');
    const old = new Text('old\uD800'), comment = new Comment('\uDC00');
    branch.append(old, comment); box.append(branch);
    box.textContent = 'A\uD800B\uDC00'; const first = box.firstChild;
    exactNodeCheck(first.nodeType === 3 && first.parentNode === box, 'new Text');
    exactNodeCheck(branch.parentNode === null && old.parentNode === branch && comment.parentNode === branch, 'only direct child detached');
    exactNodeUnits(old.data, [111,108,100,55296]); exactNodeUnits(comment.data, [56320]);
    box.textContent = 'A\uD800B\uDC00'; const second = box.firstChild;
    exactNodeCheck(second !== first && first.parentNode === null && box.childNodes.length === 1, 'equal value new identity');
    exactNodeUnits(first.data, [65,55296,66,56320]); exactNodeUnits(second.data, [65,55296,66,56320]);
    box.textContent = null;
    exactNodeCheck(box.firstChild === null && second.parentNode === null, 'empty clears without Text');
    exactNodeUnits(second.data, [65,55296,66,56320]);
    return true;
  },
  fragment_exact_replacement_and_nullable_conversion: function () {
    const d = exactNodeDescriptor('textContent'), f = new DocumentFragment();
    f.append(new Text('old')); d.set.call(f, '\uDC00\uD800'); const old = f.firstChild;
    exactNodeUnits(f.textContent, [56320,55296]);
    d.set.call(f, undefined);
    exactNodeCheck(f.firstChild === null && old.parentNode === null, 'raw undefined empty');
    exactNodeUnits(old.data, [56320,55296]);
    d.set.call(f, {toString: function () {return undefined;}});
    exactNodeCheck(f.textContent === 'undefined', 'converted undefined string');
    d.set.call(f, {toString: function () {return null;}});
    exactNodeCheck(f.textContent === 'null', 'converted null string');
    d.set.call(f); exactNodeCheck(f.firstChild === null, 'missing argument empty');
    return true;
  },
  ignored_kind_conversion_and_extra_arguments: function () {
    const value = exactNodeDescriptor('nodeValue'), content = exactNodeDescriptor('textContent');
    const box = document.createElement('div'); box.textContent = '\uD800';
    let trace = '';
    const input = {toString: function () {trace += 'C'; return '\uDC00';}};
    const extra = {toString: function () {trace += 'X'; throw new Error('extra argument used');}};
    value.set.call(box, input, extra); content.set.call(document, input, extra);
    exactNodeCheck(trace === 'CC' && document.textContent === null, 'ignored kinds still convert only supplied value');
    exactNodeUnits(box.textContent, [55296]);
    const text = new Text('old'); value.set.call(text, '\uDC00', extra);
    exactNodeUnits(text.data, [56320]); exactNodeCheck(trace === 'CC', 'extra ignored for real write');
    let error; try {value.set.call(box, Symbol('value'));} catch (caught) {error = caught;}
    exactNodeCheck(error instanceof TypeError, 'ignored kind Symbol conversion');
    exactNodeUnits(box.textContent, [55296]);
    return true;
  },
  exact_callback_children_and_abrupt_prefix: function () {
    const d = exactNodeDescriptor('textContent'), box = document.createElement('div');
    const branch = document.createElement('span'), leaf = new Text('\uD800');
    branch.append(leaf); box.append(branch); const added = new Text('\uDC00');
    let trace = '';
    d.set.call(box, {toString: function () {trace += 'C'; box.append(added); return '\uD800\uDC00';}});
    exactNodeCheck(trace === 'C' && branch.parentNode === null && added.parentNode === null, 'fresh children replaced');
    exactNodeCheck(leaf.parentNode === branch, 'detached descendant link');
    exactNodeUnits(leaf.data, [55296]); exactNodeUnits(added.data, [56320]);
    exactNodeUnits(box.textContent, [55296,56320]);
    const kept = box.firstChild, token = {}; let observed;
    try {d.set.call(box, {toString: function () {kept.nodeValue = '\uDC00'; box.append(branch); throw token;}});}
    catch (error) {observed = error;}
    exactNodeCheck(observed === token && box.firstChild === kept && branch.parentNode === box, 'callback prefix survives abrupt conversion');
    exactNodeUnits(box.textContent, [56320,55296]);
    return true;
  },
  saved_intrinsics_ignore_own_shadows: function () {
    const v = exactNodeDescriptor('nodeValue'), t = exactNodeDescriptor('textContent');
    const node = new Text('\uD800');
    Object.defineProperty(node, 'nodeValue', {value: 'shadow', configurable: true});
    Object.defineProperty(node, 'textContent', {value: undefined, configurable: true});
    exactNodeCheck(node.nodeValue === 'shadow' && node.textContent === undefined, 'own shadow');
    v.set.call(node, '\uDC00'); exactNodeUnits(t.get.call(node), [56320]);
    exactNodeCheck(node.nodeValue === 'shadow', 'native write keeps shadow');
    delete node.nodeValue; delete node.textContent;
    exactNodeUnits(node.nodeValue, [56320]); exactNodeUnits(node.textContent, [56320]);
    return true;
  },
  base_selection_follows_replaced_children: function () {
    exactNodeDescriptor('textContent');
    const box = document.createElement('div'), first = document.createElement('base');
    const second = document.createElement('base');
    first.setAttribute('href', 'https://first.example/a/');
    second.setAttribute('href', 'https://second.example/b/');
    box.append(first); document.body.append(box, second);
    exactNodeCheck(document.baseURI === 'https://first.example/a/', 'first connected base');
    first.textContent = 'ignored child';
    exactNodeCheck(document.baseURI === 'https://first.example/a/', 'base itself stays connected');
    box.textContent = 'replacement';
    exactNodeCheck(first.parentNode === null && first.getAttribute('href') === 'https://first.example/a/', 'base retained detached');
    exactNodeCheck(document.baseURI === 'https://second.example/b/', 'next connected base selected');
    second.remove(); exactNodeCheck(document.baseURI === document.URL, 'document URL fallback');
    return true;
  },
  details_groups_separate_after_detachment: function () {
    exactNodeDescriptor('textContent');
    const box = document.createElement('div'), a = document.createElement('details');
    const peer = document.createElement('details'), summary = document.createElement('summary');
    const leaf = new Text('retained\uD800'); summary.append(leaf); a.append(summary);
    a.setAttribute('name', 'node-data-group'); peer.setAttribute('name', 'node-data-group');
    a.setAttribute('open', ''); box.append(a); document.body.append(box, peer);
    exactNodeCheck(a.hasAttribute('open') && !peer.hasAttribute('open'), 'initial group');
    box.textContent = 'replacement'; peer.setAttribute('open', '');
    exactNodeCheck(a.parentNode === null && summary.parentNode === a && leaf.parentNode === summary, 'detached details subtree');
    exactNodeCheck(a.hasAttribute('open') && peer.hasAttribute('open'), 'separate roots have independent groups');
    exactNodeUnits(leaf.data, [114,101,116,97,105,110,101,100,55296]);
    peer.removeAttribute('open'); peer.setAttribute('open', '');
    exactNodeCheck(a.hasAttribute('open'), 'connected group does not close detached details');
    return true;
  },
  template_exact_direct_children_are_separate: function () {
    exactNodeDescriptor('textContent'); const template = document.createElement('template');
    const hidden = new Text('\uD800'); template.content.append(hidden);
    template.textContent = '\uDC00'; const direct = template.firstChild;
    exactNodeUnits(template.textContent, [56320]); exactNodeUnits(template.content.textContent, [55296]);
    template.textContent = '';
    exactNodeCheck(direct.parentNode === null && hidden.parentNode === template.content, 'two independent child lists');
    exactNodeUnits(hidden.data, [55296]); exactNodeUnits(direct.data, [56320]);
    template.content.textContent = '\uD800\uDC00';
    exactNodeCheck(hidden.parentNode === null && template.firstChild === null, 'fragment-only replacement');
    exactNodeUnits(template.content.textContent, [55296,56320]);
    return true;
  }
};
