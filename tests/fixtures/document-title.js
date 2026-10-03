// Independent ordinary-success oracles: one named body per fresh realm/mode.
// https://html.spec.whatwg.org/multipage/dom.html#document.title
// https://dom.spec.whatwg.org/#concept-child-text-content
// https://infra.spec.whatwg.org/#strip-and-collapse-ascii-whitespace
// https://webidl.spec.whatwg.org/#es-attributes
function titleCheck(value, message) { if (!value) throw new Error(message); }
function titleUnits(value, expected) {
  titleCheck(value.length === expected.length, 'UTF16 length');
  for (let i = 0; i < expected.length; i++)
    titleCheck(value.charCodeAt(i) === expected[i], 'UTF16 unit ' + i);
}
function titleHead() {
  const head = document.head, body = document.body;
  titleCheck(head && body && typeof head.appendChild === 'function', 'HTML setup');
  head.textContent = ''; body.textContent = '';
  return head;
}
function titleElement(text) {
  const node = document.createElement('title');
  if (text !== undefined) node.appendChild(new Text(text));
  return node;
}
function titleDescriptor() {
  const d = Object.getOwnPropertyDescriptor(Document.prototype, 'title');
  titleCheck(d && typeof d.get === 'function' && typeof d.set === 'function', 'title accessor prerequisite');
  return d;
}
function titleDetachRoot() {
  const root = document.documentElement, append = document.append;
  titleCheck(root && typeof append === 'function', 'root/append prerequisite');
  titleCheck(append.call(document) === undefined, 'working Document append');
  let error;
  try { append.call(document, root, new Text('x')); } catch (e) { error = e; }
  titleCheck(error && error.name === 'HierarchyRequestError' && error.code === 3, 'root assembly rejection');
  titleCheck(document.documentElement === null && root.parentNode.nodeType === 11, 'root moved before rejection');
  return root;
}
function titleForeign(markup, namespace, name) {
  const box = document.createElement('div'); box.innerHTML = markup;
  const node = box.firstChild;
  titleCheck(node && node.namespaceURI === namespace && node.nodeName === name, 'namespace parsing prerequisite');
  return node;
}
var documentTitleCases = {
  direct_text_ascii_whitespace: function () {
    const head = titleHead(), title = titleElement();
    const nested = document.createElement('b'); nested.appendChild(new Text('excluded'));
    title.append(new Text('\t A\r'), new Comment('comment'), nested, new Text('\n\f B  '));
    head.appendChild(title);
    titleUnits(document.title, [65,32,66]);
    titleUnits(title.firstChild.data, [9,32,65,13]);
    titleUnits(title.lastChild.data, [10,12,32,66,32,32]);
    titleCheck(nested.firstChild.data === 'excluded', 'getter leaves descendants');
    title.firstChild.data = ' \t'; title.lastChild.data = '\r\n\f ';
    titleUnits(document.title, []);
    return true;
  },
  exact_units_and_non_ascii_spaces: function () {
    const head = titleHead(), title = titleElement();
    const nested = document.createElement('span'); nested.appendChild(new Text('ignored'));
    const high = new Text('\t\uD800'), low = new Text('\uDC00\u00A0\u000B\uD800 ');
    title.append(high, new Comment('excluded'), nested, low); head.appendChild(title);
    titleUnits(document.title, [55296,56320,160,11,55296]);
    titleUnits(JSON.parse(JSON.stringify(document.title)), [55296,56320,160,11,55296]);
    titleUnits(high.data, [9,55296]); titleUnits(low.data, [56320,160,11,55296,32]);
    titleCheck(title.childNodes.length === 4 && title.childNodes[0] === high && title.childNodes[3] === low, 'read does not rewrite nodes');
    return true;
  },
  setter_exact_fresh_text_and_retention: function () {
    const head = titleHead(), title = titleElement(), branch = document.createElement('b');
    const old = new Text('old\uD800'), note = new Comment('\uDC00');
    branch.append(old, note); title.appendChild(branch); head.appendChild(title);
    document.title = ' A\uD800\tB ';
    const first = title.firstChild;
    titleCheck(first.nodeType === 3 && title.childNodes.length === 1 && first.parentNode === title, 'fresh Text');
    titleUnits(first.data, [32,65,55296,9,66,32]); titleUnits(document.title, [65,55296,32,66]);
    titleCheck(branch.parentNode === null && old.parentNode === branch && note.parentNode === branch, 'only direct child detached');
    titleUnits(old.data, [111,108,100,55296]); titleUnits(note.data, [56320]);
    document.title = ' A\uD800\tB ';
    const second = title.firstChild;
    titleCheck(second !== first && first.parentNode === null, 'equal assignment still replaces');
    titleUnits(first.data, [32,65,55296,9,66,32]);
    document.title = '';
    titleCheck(title.firstChild === null && second.parentNode === null && title.parentNode === head, 'empty leaves title without Text');
    return true;
  },
  nullish_symbol_and_abrupt_conversion: function () {
    const head = titleHead(), title = titleElement('old'); head.appendChild(title);
    document.title = null; titleUnits(title.firstChild.data, [110,117,108,108]);
    document.title = undefined; titleUnits(title.firstChild.data, [117,110,100,101,102,105,110,101,100]);
    const d = titleDescriptor(); d.set.call(document);
    titleUnits(d.get.call(document), [117,110,100,101,102,105,110,101,100]);
    const kept = title.firstChild; let error;
    titleCheck(typeof Symbol === 'function', 'Symbol prerequisite');
    try { document.title = Symbol('x'); } catch (e) { error = e; }
    titleCheck(error instanceof TypeError && title.firstChild === kept, 'Symbol throws before replacement');
    const marker = {}; let hits = 0, caught;
    try { document.title = {toString: function () { hits++; kept.data = 'prefix'; throw marker; }}; }
    catch (e) { caught = e; }
    titleCheck(caught === marker && hits === 1 && title.firstChild === kept && kept.data === 'prefix', 'abrupt author prefix');
    return true;
  },
  callback_selects_current_title: function () {
    const head = titleHead(), old = titleElement('old'), next = titleElement('new');
    head.appendChild(old); let hints = '', fallback = 0;
    const value = {valueOf: function () { fallback++; return 'wrong'; }};
    value[Symbol.toPrimitive] = function (hint) {
      hints += hint; head.removeChild(old); head.appendChild(next); return ' \uDC00 after\t';
    };
    document.title = value;
    titleCheck(hints === 'string' && fallback === 0, 'one string-hint conversion');
    titleCheck(old.parentNode === null && old.firstChild.data === 'old', 'stale target untouched');
    titleUnits(next.firstChild.data, [32,56320,32,97,102,116,101,114,9]);
    titleUnits(document.title, [56320,32,97,102,116,101,114]);
    return true;
  },
  connected_order_and_template_trees: function () {
    titleHead();
    const detached = titleElement('detached'), template = document.createElement('template');
    titleCheck(template.content && template.content.nodeType === 11, 'template content prerequisite');
    const hidden = titleElement('inert'), ordinary = titleElement('ordinary'), later = titleElement('later');
    template.content.appendChild(hidden); template.appendChild(ordinary);
    document.body.append(template, later);
    titleCheck(document.title === 'ordinary', 'ordinary template children participate');
    template.removeChild(ordinary);
    titleCheck(document.title === 'later', 'detached/content titles excluded');
    template.appendChild(ordinary);
    titleCheck(document.title === 'ordinary', 'connected order restored');
    document.body.appendChild(template);
    titleCheck(document.title === 'later', 'moving subtree changes first title');
    titleCheck(detached.parentNode === null && hidden.parentNode === template.content, 'other trees retained');
    return true;
  },
  missing_title_empty_creation_and_missing_head: function () {
    const head = titleHead(), marker = new Comment('head marker'); head.appendChild(marker);
    document.title = '';
    const title = head.lastChild;
    titleCheck(title !== marker && title.nodeName === 'TITLE' && title.namespaceURI === 'http://www.w3.org/1999/xhtml', 'new HTML title');
    titleCheck(head.firstChild === marker && title.firstChild === null, 'append empty title at end');
    const root = document.documentElement; root.removeChild(head);
    titleCheck(document.head === null, 'head detached');
    const bodyTitle = titleElement('body'); document.body.appendChild(bodyTitle);
    document.title = 'live'; titleCheck(bodyTitle.firstChild.data === 'live', 'existing title without head');
    document.body.removeChild(bodyTitle); let calls = 0;
    document.title = {toString: function () { calls++; return 'ignored'; }};
    titleCheck(calls === 1 && document.title === '' && document.body.firstChild === null, 'missing title and head ignores converted input');
    return true;
  },
  html_namespace_non_html_root: function () {
    titleHead(); const root = document.createElement('main'), title = titleElement('old');
    root.appendChild(title); let conversions = 0;
    document.title = {toString: function () { conversions++; titleDetachRoot(); document.append(root); return 'new'; }};
    titleCheck(document.documentElement === root && document.head === null, 'non-html HTML root');
    titleCheck(conversions === 1 && title.firstChild.data === 'new' && document.title === 'new', 'select current root after conversion');
    root.removeChild(title);
    const fakeHead = document.createElement('head'); root.appendChild(fakeHead); let calls = 0;
    document.title = {toString: function () { calls++; return 'ignored'; }};
    titleCheck(calls === 1 && document.head === null && fakeHead.firstChild === null && document.title === '', 'descendant head is not document head');
    return true;
  },
  no_root_and_foreign_root_convert_before_ignore: function () {
    titleHead();
    const foreign = titleForeign('<math></math>', 'http://www.w3.org/1998/Math/MathML', 'math');
    const title = titleElement(' foreign title '); foreign.appendChild(title);
    titleDetachRoot(); let trace = '';
    document.title = {toString: function () { trace += 'N'; return '\uD800'; }};
    titleCheck(trace === 'N' && document.documentElement === null && document.title === '', 'no root still converts');
    document.append(foreign);
    titleCheck(document.title === 'foreign title', 'getter uses HTML title under foreign root');
    document.title = {toString: function () { trace += 'F'; return 'ignored'; }};
    titleCheck(trace === 'NF' && title.firstChild.data === ' foreign title ', 'foreign setter ignores only after conversion');
    const marker = {}; let caught;
    try { document.title = {toString: function () { throw marker; }}; } catch (e) { caught = e; }
    titleCheck(caught === marker && document.title === 'foreign title', 'ignored branch propagates conversion throw');
    return true;
  },
  svg_root_first_direct_svg_title: function () {
    titleHead();
    const svg = titleForeign('<svg><g><title>nested</title></g><title> first </title><title>second</title></svg>', 'http://www.w3.org/2000/svg', 'svg');
    const group = svg.childNodes[0], first = svg.childNodes[1], second = svg.childNodes[2];
    const htmlTitle = titleElement('wrong namespace');
    svg.textContent = ''; svg.append(htmlTitle, group, first, second);
    titleDetachRoot(); document.append(svg);
    titleCheck(document.title === 'first', 'first direct SVG title selected');
    const old = first.firstChild; document.title = ' \uD800\tX ';
    titleUnits(first.firstChild.data, [32,55296,9,88,32]); titleUnits(document.title, [55296,32,88]);
    titleCheck(old.parentNode === null && group.firstChild.firstChild.data === 'nested', 'old text detached and nested title untouched');
    titleCheck(htmlTitle.firstChild.data === 'wrong namespace' && second.firstChild.data === 'second', 'other namespace/second titles untouched');
    return true;
  },
  svg_missing_title_creates_first_child: function () {
    titleHead();
    const svg = titleForeign('<svg><g><title>nested</title></g><desc>description</desc></svg>', 'http://www.w3.org/2000/svg', 'svg');
    const group = svg.firstChild, desc = svg.lastChild, htmlTitle = titleElement('HTML');
    svg.appendChild(htmlTitle); titleDetachRoot(); document.append(svg);
    titleCheck(document.title === '', 'no applicable direct SVG title');
    document.title = '';
    const created = svg.firstChild;
    titleCheck(created.namespaceURI === 'http://www.w3.org/2000/svg' && created.nodeName === 'title', 'new SVG title');
    titleCheck(created.firstChild === null && created.parentNode === svg, 'empty title has no Text');
    titleCheck(svg.childNodes[1] === group && svg.childNodes[2] === desc && svg.childNodes[3] === htmlTitle, 'insert before previous first child');
    document.title = '\uDC00'; titleUnits(created.firstChild.data, [56320]); titleUnits(document.title, [56320]);
    titleCheck(svg.firstChild === created && svg.childNodes.length === 4, 'reuse created title');
    return true;
  },
  ordinary_descriptors_shadows_deletion_and_brands: function () {
    const head = titleHead(), title = titleElement('initial'); head.appendChild(title);
    const d = titleDescriptor();
    titleCheck(d.enumerable && d.configurable && !('value' in d), 'ordinary prototype accessor');
    titleCheck(d.get.name === 'get title' && d.get.length === 0 && d.set.name === 'set title' && d.set.length === 1, 'accessor metadata');
    titleCheck(Object.getOwnPropertyDescriptor(document, 'title') === undefined, 'no own virtual descriptor');
    titleCheck(d.set.call(document, 'saved') === undefined && d.get.call(document) === 'saved', 'saved functions work');
    let calls = 0, error;
    const fake = Object.create(Document.prototype), value = {toString: function () { calls++; return 'wrong'; }};
    try { d.set.call(fake, value); } catch (e) { error = e; }
    titleCheck(error instanceof TypeError && calls === 0, 'setter brand before coercion');
    error = undefined; try { d.get.call(title); } catch (e) { error = e; }
    titleCheck(error instanceof TypeError, 'getter authentic Document brand');
    Object.defineProperty(document, 'title', {value:'own', writable:true, configurable:true});
    document.title = 'changed own';
    titleCheck(document.title === 'changed own' && d.get.call(document) === 'saved', 'own shadow does not mutate title');
    delete document.title;
    let receiver, written;
    try {
      Object.defineProperty(Document.prototype, 'title', {get:function(){receiver=this;return 'custom';},set:function(v){receiver=this;written=v;},configurable:true});
      titleCheck(document.title === 'custom' && receiver === document, 'replacement getter receiver');
      document.title = value; titleCheck(receiver === document && written === value && calls === 0, 'replacement setter receives original value');
      delete Document.prototype.title;
      titleCheck(document.title === undefined, 'deleted accessor does not resurrect');
      document.title = 'expando';
      titleCheck(document.title === 'expando' && d.get.call(document) === 'saved', 'post-delete ordinary write');
      delete document.title;
      titleCheck(document.title === undefined, 'own deletion still no fallback');
      d.set.call(document, 'still saved'); titleCheck(d.get.call(document) === 'still saved', 'saved intrinsic survives deletion');
    } finally { Object.defineProperty(Document.prototype, 'title', d); }
    titleCheck(document.title === 'still saved', 'restored descriptor');
    return true;
  }
};
