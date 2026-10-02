# Own properties on DOM objects

Scripts can attach ordinary string and symbol properties to Document, elements,
text nodes and document fragments. Values retain their JavaScript identity and
UTF-16 keys; assigning an ordinary property does not convert its value to a DOM
string. Each object has separate, lazy storage, retained when a node is detached
and omitted when it is cloned.

Own data and accessor descriptors support reflection, flags, strict assignment,
deletion and SameValue checks for nonconfigurable properties. Accessors receive
the actual DOM object as `this`. An own property whose value is `undefined` still
shadows the native member. For example, replacing one element's `querySelector`
leaves sibling elements unchanged; deleting the replacement restores the same
saved native function.

Existing native attribute names are classified by the receiver's node kind,
namespace and element type. `input.value` reaches its native setter, while
`div.value`, `fragment.id` and `text.onclick` can hold ordinary values. Explicit
own descriptors take priority over native fields. Unsupported native setters
remain explicit failures, including style/class-list forwarding and outerHTML.
This does not complete the semantics or inventory of HTML, SVG or MathML fields.

Node own-key enumeration orders numeric keys, other strings and symbols, and
for-in checks live descriptors. Full Document enumeration remains unsupported:
its unforgeable `location` and legacy named-property machinery are not yet
implemented. The engine does not substitute an incomplete own-key list for those
semantics. Individual ordinary properties and symbol reflection are supported;
`document.location` remains explicitly unsupported.

Storage uses the existing per-runtime host identity map. Reads and missing
deletions create no bag. Work and storage admission precede publication, and
descriptor conversion completes before the operation examines live state.
Failed admission cannot publish the new descriptor or host-map entry. These
checks do not establish complete fallible allocation coverage or production
security.

Complete DOM interface members and construction, complete Document reflection,
host integrity operations, JSON serialization of host properties and broader
generic Array behavior remain unfinished. At that checkpoint, the coarse EventTarget/Object
prototype chain could still intercept a virtual native member through an inherited
property. The subsequent [interface prototype work](dom-prototypes.md) supplies
represented node chains and moves nine ParentNode operations onto their defining
prototypes; remaining virtual members still need migration.

Nineteen focused groups cover identity, descriptor flags, live callbacks, key
ordering, receiver classification and exact/one-short resource admission. A
six-case HTML fixture checks literal pixels and result text on load and click,
both directly and through the confined renderer. Rust 1.88 passes 1,424 default
tests; Rust 1.98 passes 1,547 tests with native Vulkan enabled, including ignored
confinement tests. Strict native Clippy and formatting pass.

The unchanged DOM identity corpus remains at 54/58 verified expectations with
all 44 controls healthy. Its two replacement modes now reach the missing
Document enumeration operation; they are still unmet ordinary-success cases,
not new passes or metadata exclusions. All 78 Object.is local case records and
60 controls remain unchanged. The [raw results](evidence/dom-own-properties.json)
retain initial failures and subsequent corrections.

The complete existing descriptor profile records 3,556 passes and 18 exclusions,
gaining both modes of the HTML form getter-retention test without regressions.
The symbol profile retains 184 passes, four failures and 54 exclusions. Existing
baseline inventories and expectations are unchanged.

Normative references: [Web IDL platform objects](https://webidl.spec.whatwg.org/#platform-object),
[Document's legacy behavior](https://html.spec.whatwg.org/multipage/dom.html#the-document-object)
and [ordinary property operations](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-ordinary-object-internal-methods-and-internal-slots).
