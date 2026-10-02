// Script-level checks of represented HTML, SVG and MathML prototype identity.
(function () {
  function check(value, label) { if (!value) throw new Error(label); }
  function identity(node, C) {
    check(Object.getPrototypeOf(node) === C.prototype, 'immediate ' + C.name);
    check(node.constructor === C && node instanceof C, 'constructor ' + C.name);
    check(C.prototype.isPrototypeOf(node), 'membership ' + C.name);
    check(node instanceof Element && node instanceof Node, 'core chain ' + C.name);
    check(Object.prototype.toString.call(node) === '[object ' + C.name + ']', 'tag ' + C.name);
    var copy = node.cloneNode(false);
    check(copy !== node && Object.getPrototypeOf(copy) === C.prototype, 'clone ' + C.name);
  }
  var html = [
    ['div', HTMLDivElement], ['section', HTMLElement], ['h3', HTMLHeadingElement],
    ['audio', HTMLAudioElement], ['td', HTMLTableCellElement],
    ['unknownwidget', HTMLUnknownElement], ['x-widget', HTMLElement],
    ['annotation-xml', HTMLUnknownElement]
  ];
  for (var i = 0; i < html.length; i++) identity(document.createElement(html[i][0]), html[i][1]);
  check(Object.getPrototypeOf(HTMLAudioElement.prototype) === HTMLMediaElement.prototype,
        'media intermediate prototype');
  check(Object.getPrototypeOf(HTMLAudioElement) === HTMLMediaElement,
        'media interface object inheritance');
  var holder = document.createElement('div');
  holder.innerHTML = '<svg id="vector"><linearGradient id="gradient"></linearGradient>' +
    '<rect id="shape"></rect><unknownsvg id="unknown"></unknownsvg>' +
    '<foreignObject><div id="html"></div></foreignObject></svg>' +
    '<math id="math"><mrow id="row"></mrow></math>';
  var foreign = [
    ['vector', SVGSVGElement], ['gradient', SVGLinearGradientElement],
    ['shape', SVGRectElement], ['unknown', SVGElement], ['html', HTMLDivElement],
    ['math', MathMLElement], ['row', MathMLElement]
  ];
  for (var j = 0; j < foreign.length; j++)
    identity(holder.querySelector('#' + foreign[j][0]), foreign[j][1]);
  var shape = holder.querySelector('#shape');
  check(shape instanceof SVGGeometryElement && shape instanceof SVGGraphicsElement &&
        shape instanceof SVGElement && !(shape instanceof HTMLElement), 'SVG intermediate chain');
  check(!(holder.querySelector('#row') instanceof SVGElement) &&
        !(holder.querySelector('#row') instanceof HTMLElement), 'MathML namespace branch');
  return true;
})()
