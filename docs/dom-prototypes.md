# DOM interface prototypes

Represented DOM nodes use their interface's initial prototype chain. For example,
a parsed `div` inherits from `HTMLDivElement.prototype`, `HTMLElement.prototype`,
`Element.prototype`, `Node.prototype` and `EventTarget.prototype`. Text and
Comment inherit through CharacterData. The mapping uses the node's stored
namespace and local name; SVG and MathML do not use the HTML element table.
These identities do not imply that every member of those interfaces exists.

Interface objects have corresponding inheritance, `name`, `length`, `prototype`
and prototype `constructor` properties. Node constants, interface string tags
and the applicable unscopables objects are installed as descriptors.
`HTMLDocument` aliases `Document`; it does not introduce another prototype.
Supported property lookup, `Object.getPrototypeOf`, `isPrototypeOf`, `instanceof`
and for-in traversal use the same document-aware chain.

The existing nine `querySelector`, `querySelectorAll` and `append` functions now
live on Document, Element and DocumentFragment prototypes. Replacing a prototype
member affects its instances; deleting it does not synthesize another native
function. A previously saved function remains callable. Method receiver checks
still use actual node identity and kind: inheriting from an interface prototype
does not create a native node or satisfy a native receiver check.

`new DocumentFragment()`, `new Text(data)` and `new Comment(data)` create real
nodes in the active document. Text and Comment convert their arguments before
reading `newTarget.prototype`. After author callbacks finish, construction
checks current document capacity and admits retained storage before publishing
the node. An alternate object prototype is stored separately from ordinary own
properties. Omitted or undefined data defaults to the empty string; null converts
to `"null"`. Lone UTF-16 surrogates remain an explicit unsupported operation
because the document storage cannot preserve them.

The [ProcessingInstruction follow-up](processing-instruction.md) adds its
constructor, Document factory and CharacterData data/length accessors.
Independent Document construction remains explicitly unsupported.
Independent document ownership, custom-element registration and successful
HTML constructor behavior are unfinished. Attr, CDATASection, XMLDocument and
ShadowRoot are not represented by the current node model. Most native members
and attributes still require migration onto interface prototypes. Complete
Document own-key reflection, named properties and host prototype mutation remain
unfinished as well.

The installer eagerly creates ordinary mutable interface metadata and stores 154
interface records by static index, including the existing EventTarget. Constructor
handles retain their private property-bag IDs; this cache does not change native
function identity. Static node mapping returns the same index directly. New
global bindings use one admitted sorted merge; creation order remains independent
of map order. At the prototype checkpoint, raw bootstrap left 11,133 of 100,000
work units and charged 1,746,193 cumulative bytes, with 663 objects. The
processing-instruction members bring these measurements to 9,511 remaining
units, 1,761,677 bytes and 668 objects. Successful initialization grants
the existing author allowance; it never resets the cumulative heap ledger.

The comprehensive original JavaScript fixture still exceeds the fixed author
work allowance in both modes. Its ordinary-success expectation remains unmet;
smaller independently executable cases provide separate behavior coverage.
The metadata installer and constructor paths retain existing limits. These
scoped checks do not establish full fallible-allocation coverage, production
security or browser performance parity.

The frozen DOM method suite now verifies 56/58 expectations with all 44 controls
healthy. The two newly passing modes cover real interface prototypes; complete
Document enumeration remains unmet. Object.is retains 72/78 expectations and
60 healthy controls. The Symbol, array-descriptor and Reflect-construction
profiles retain all compared case/control observations. See the
[validation](VALIDATION.md#represented-dom-interface-prototypes) and
[evidence record](evidence/dom-prototypes.json).

Normative references: [Web IDL interface objects](https://webidl.spec.whatwg.org/#es-interface-object),
[interface prototype objects](https://webidl.spec.whatwg.org/#es-interface-prototype-object),
[DOM interfaces](https://dom.spec.whatwg.org/),
[HTML element interfaces](https://html.spec.whatwg.org/multipage/indices.html#elements-3)
and [SVG element interfaces](https://svgwg.org/svg2-draft/types.html#SVGDOMElements).
