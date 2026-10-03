# Node.isEqualNode

This increment adds structural equality for represented nodes. The default and
native Vulkan suites, independent release expectations, Page and confined-worker
witnesses pass. Historical prototype-inventory differences remain explicit below.

The ordinary `Node.prototype.isEqualNode(other)` method compares internal node
fields and ordered ordinary children. Equal nodes can have different identities,
parents, connection states, own JavaScript properties and rendering state.
Neither cloning nor serialization is used to decide equality.

## Receiver and argument rules

The receiver must be an authentic represented Node. Its brand is checked before
the required nullable Node argument. Omitting the argument throws TypeError;
explicit `null` or `undefined` returns false. Other non-Nodes throw without
calling `toString`, `valueOf` or `Symbol.toPrimitive`. Inheriting from
`Node.prototype` does not create a Node brand. Authentic nodes produced through
the supported alternate-constructor-prototype path retain their brand.

Argument expressions, including extra arguments, finish before native
invocation. Their mutations and thrown values retain their normal order. Once
invoked, the comparison body uses internal data without authored getters,
conversion hooks or callbacks. Authored shadows of `nodeType`, `childNodes`,
data fields or the method itself do not change a saved native call's operands.

## Equality within the represented model

Different kinds or child counts are unequal. Matching kinds compare these fields
before traversing children:

| Kind | Equality fields |
| --- | --- |
| Document and DocumentFragment | Ordered ordinary children; document URL/mode and fragment host are excluded. |
| Text and Comment | Exact retained UTF-16 data, with the two kinds remaining distinct. |
| ProcessingInstruction | Case-sensitive scalar target and exact retained data, without rerunning creation validators. |
| DocumentType | Name, public ID and system ID; absent optional IDs compare as empty strings, and the internal quirks flag is excluded. |
| Element | Represented namespace and local tag, unordered attributes with their actual namespace annotations and values, then ordered ordinary children. |

Canonical Scalar/Units storage permits borrowed data comparison without an
intermediate JavaScript string or replacement projection. Unpaired UTF-16 units
remain significant, and Unicode normalization is not performed. Two adjacent
Text nodes and a single Text with their combined data are different structures.

Element attribute comparison relies on the current fixed qualified-name roster:
its namespace/local-name pairs are unambiguous in this storage model. Sorted
borrowed map walks can therefore compare unordered attributes while checking
their namespace annotations. Equal spellings and values with different actual
namespaces remain unequal. Literal colons do not create invented element
prefixes. This does not add general XML prefix storage, arbitrary namespace
aliases or new attribute APIs.

Template elements compare their ordinary children, excluding hosted content.
Passing `template.content` explicitly compares that fragment's ordinary
children, irrespective of its host association. Parent links and connection
state are not equality fields, though reached descendant links are checked for
local consistency. No ancestor walk above the operands is needed.

## Ordinary metadata and historical inventories

The property is writable, enumerable and configurable. Its cached
function has name `isEqualNode`, length one, ordinary Function inheritance and
no construction behavior. Saved calls survive own shadows and
prototype property replacement or deletion, without a hidden fallback.

The new represented prototype inventory has 28 keys: `nodeValue`, `textContent`,
`getRootNode`, `hasChildNodes`, `normalize`, `isEqualNode`, `isSameNode`, `contains`,
the eighteen Node constants, `constructor`, then `Symbol.toStringTag`. The
constructor's 21-key inventory stays unchanged. These are literal implemented
inventories, not the entire standard Node interface.

The constant batch consequently starts with nine prototype entries and three
constructor entries. Before the later constructor property, the prototype map
has 27 entries and its order buffer reserves 28 slots. The pinned sorted build
retains four prototype tree nodes and three constructor tree nodes, with
admission preceding consumption of either old bag. Existing `getRootNode`
dispatch remains the first length-eleven comparison; only its unmatched branch
pays for the additional `isEqualNode` comparison.

Adding the member invalidates two frozen getRootNode inventory case modes and
four reflected-order controls. Their sources and expectations remain unchanged:
ordinary Error case losses and TypeError/unhealthy controls remain visible.
Eighteen older Node-constant, Normalize and predicate inventory observations
remain unhealthy and byte-exact.
The new pristine 28-key case and positive/wrong control pair are separate.

Private historical wrappers temporarily delete the later configurable
members, call an unchanged older oracle, and restore the saved descriptors.
Those realms are discarded because reinsertion changes creation order. External
replays retain their original sources and realms without those wrappers; no
historical failure is relabeled as healthy.

## Reached work and storage admission

After authentic conversion and local header checks, a top-level same-ID pair
returns true. That shortcut does not validate its subtree or attribute contents.
For distinct operands, a kind, count or field mismatch returns false without
auditing unvisited data. Matched leaves and empty containers need no owned
traversal buffers. Equal-length field comparison prepays its full selected field
even if the actual comparison finds an earlier difference.

Nonempty matching containers use an iterative paired depth-first walk, selecting
one child pair at a time. Each side has separate reached-ID bits stored in sparse
64-ID pages, plus a frame vector for active depth. IDs repeated within one side
refuse; overlap between the two operand trees is allowed. The traversal admits the
256-edge endpoint, rejects deeper reached edges and checks actual descendant
backlinks. A descendant identity match does not bypass these checks.

Work is paid before field comparisons, borrowed attribute iterator creation and
advancement, side-map alignment, namespace checks, child selection, reached-map
searches and updates. New map pages admit typed blocks and insertion/cleanup
work; frame growth admits allocation and relocation before a fallible reserve.
Existing capacity is reused without pretending a new allocation occurred.
There is no comparison payload copy, persistent cache or whole-arena bitmap.
Scratch depends on reached pages and active depth, so widely spaced node IDs
can require more page entries than a dense layout with the same shape.

Reached invalid host data refuses with TypeError; work, arithmetic and storage
refusals are Resource. The same-ID and early-mismatch paths are deliberately
local policies, not graph-validation certificates. Consumed work and cumulative
allocation remain spent when scratch is dropped. Standard BTree allocation
retains its infallible process-OOM boundary; generic VM, native dispatch and
diagnostic allocation limits remain additional. The comparison body does not
mutate the DOM, but terminal Resource refusal cannot undo completed argument
effects and does not run ordinary JavaScript `catch` or `finally`.

The existing 100,000-work and 8 MiB cumulative runtime limits remain unchanged.
A structurally admitted tree can still exceed them. There is no promise that
every DOM at the depth, node or retained-byte limit can be compared in one turn.
Measured raw bootstrap leaves 7,816 work units and charges 1,762,227 bytes, with
687 objects and capacity, 321 native entries and 25 legacy prototype entries.
The delta is 502 work units, 3,256 charged bytes and one object, matching the
held forecast. The tested contiguous and alternating 256-edge branches consume
65,681 and 70,989 body work units and 13,416 charged bytes each. Both complete
under the fixed allowance; exact and one-short retries preserve DOM state.
The sparse layout refuses under the same work limit. Its full 577,499-work
schedule remains an arithmetic forecast, not a measured successful call.

These are logical accounting figures, not elapsed-time or physical-memory
measurements. Invocation and construction charges remain additional. DOM storage,
ERWA/EWB1 transport, rendering and reset sites are unchanged.

## Frozen expectations and validation

Sixteen independent fresh-realm bodies define 32 strict/sloppy expectations.
Four positive/wrong pairs define 16 controls. A negative tail is healthy only
when its same-mode positive partner succeeds. The published getRootNode release
baseline matches zero new cases and has four healthy controls; all 32 case
failures are ordinary runtime TypeError from missing prerequisites, with no
binding drift. The candidate passes all 32 cases and 16 controls.

Of 4,300 established case records, 4,298 remain byte-exact; of 492 controls,
488 remain byte-exact. Only the six getRootNode inventory observations above
change. Established healthy counts fall from 4,208 to 4,206 cases and from
480 to 476 controls. All four historical inventory profile commands and the
aggregate command retain exit one. The classifier identifies the anticipated
differences without changing raw health. The pristine 28-key expectations pass.

The authored Page and confined-worker witness compares a live branch with an
independently constructed detached peer. A real click changes and restores exact
Comment data and an attribute, then changes visible Text on each side in turn.
Each intermediate inequality and restored equality is asserted. Twenty-nine IDs
and the total node count stay unchanged; no click nodes are created. Template
ordinary children and contents are checked separately. Titles are `Equality ready` and `Equality done`.
An independent scalar page supplies literal green/blue markers, text and canvas
expectations. Rendering changes come from explicit data mutations, not from the
equality operation. Both browser paths pass the full-canvas, glyph, position,
identity and status checks.

Rust 1.88 passes 1,873 default tests and Rust 1.98 passes 1,996 native Vulkan
tests, including ignored tests. Both full suites use four test threads. Formatting,
both strict Clippy checks and the release build pass. The 36 new groups comprise
16 independent Runtime groups, 18 private groups and two browser groups. Focused
checks retain 219 passing observations covering 213 unique names.

Initial compilation and all new groups passed. The initial bootstrap filter
retains 28 passes and two stale descriptive snapshot failures; exactly three
literals in two files changed after measurement. No production source correction
was needed after initial compilation. Pre-hold test setup corrections, a data-only
inventory-name correction and a final-review timing correction remain in the
evidence with their original records. None changes a frozen JS expectation.

The [summary](evidence/node-equality.json), [archive](evidence/node-equality.tar.gz)
and [validation record](VALIDATION.md) bind the final source, attempts and complete
raw comparisons. The predecessor [CI receipt](evidence/node-root-ci.json) records
all nine jobs passing for `51907b7`; it is distinct from this local validation.
No speedup or platform-wide conformance rate is claimed.
CDATA, Attr, ShadowRoot, broader XML representation and cross-document adoption
remain outside the represented scope. Proxy, BigInt, broader Node members,
legacy string writers and unrelated existing DOM/allocator gaps remain.

Primary references: [isEqualNode](https://dom.spec.whatwg.org/#dom-node-isequalnode),
[node equality](https://dom.spec.whatwg.org/#concept-node-equals),
[Node IDL](https://dom.spec.whatwg.org/#interface-node),
[operation bindings](https://webidl.spec.whatwg.org/#es-operations), and
[nullable conversion](https://webidl.spec.whatwg.org/#es-nullable-type).

## Later connection checkpoint

[Node.isConnected](node-connected.md) subsequently expands the pristine prototype
to 29 keys. This chapter's frozen 28-key case loses both modes and its four
reflected-order controls become unhealthy; the raw Error/TypeError observations
remain recorded. Private historical wrappers temporarily remove the later getter,
restore its exact descriptor and discard the realm. The current connection
inventory is checked separately. Equality production code and its reached fees
are unchanged.
