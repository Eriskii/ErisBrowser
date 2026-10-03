# Node data accessors

`Node.prototype.nodeValue` and `textContent` are ordinary mutable accessors over
the represented DOM node kinds. They preserve exact UTF-16 data, including
unpaired units, through the existing [canonical storage](dom-strings.md) and
[data builder](dom-production.md).

| Receiver | `nodeValue` | `textContent` |
| --- | --- | --- |
| Document or Doctype | Gets null; setter ignores the converted value | Gets null; setter ignores the converted value |
| Text, Comment or ProcessingInstruction | Gets or replaces exact node data | Gets or replaces exact node data |
| Element or DocumentFragment | Gets null; setter ignores the converted value | Gets descendant Text data; setter replaces direct children with zero or one fresh Text |

Container reads concatenate Text data in tree order. Comments and PI data do not
contribute; template content is a separate hosted fragment. A high unit in one
Text and a low unit in another remain adjacent in the returned string even when
an excluded Comment lies between the nodes. No display-replacement projection
mediates these reads. Existing traversal, string-output and allocation limits
still apply.

## Conversion and ordinary properties

Saved getters and setters require an authentic represented node. A forged object
that inherits `Node.prototype` fails before argument conversion. Both setters
use nullable DOMString conversion: raw null, undefined or an omitted setter
argument becomes empty. An object whose conversion returns null or undefined
instead produces the strings `"null"` or `"undefined"`. This differs from the
CharacterData `data` setter's undefined-to-`"undefined"` behavior.

Ignored receiver kinds still perform conversion and propagate a thrown value or
Symbol conversion error. Once conversion succeeds, they return without building
DOM data or requiring spare node/storage capacity. Extra arguments are ignored.
Real writes inspect current node data or children after conversion, so callback
effects precede the outer operation. A throwing callback retains its own effects
and prevents the outer write.

The two Node prototype properties are enumerable and configurable. Their
getters/setters have the ordinary names `get nodeValue`, `set nodeValue`,
`get textContent`, `set textContent`, with lengths zero/one. They are separate
nonconstructable functions with mutable metadata. Own properties can shadow
them; saved intrinsic functions still operate on the authentic node. Replacing
or deleting the prototype descriptor changes normal lookup, and deletion does
not resurrect the former virtual `textContent` implementation. Other virtual
Node/Element attributes have not been migrated by this change.

## Replacement and retained state

CharacterData replacement preserves node identity, parent links and PI target.
It does not repeat PI creation-only validation, so later data can contain `?>`.
Nonempty Element/Fragment assignment creates a fresh Text, including when the
assigned value equals the previous text. Empty assignment creates no Text.

Only former direct children are detached. Their descendants keep their parent
links, exact data and node identities. Detached nodes remain in the document's
arena and retained-byte ledger; their storage is not credited toward the new
Text. Repeated equal assignments can therefore consume retained capacity. Empty
replacement can work at the node cap when the removal's work and scratch costs
fit; it does not demand a spare node slot. Nonempty replacement also checks the
existing host-inclusive depth limit before publishing a new leaf.

The container path validates current children and prepares its replacement
payload, final child vector and affected base/details work before detachment.
It resolves an affected base URL against the future tree, preserves an unaffected
selected base, and prepares details root/name bookkeeping and summary-cache
invalidation. Typed arena growth is reserved last. Commit performs no author
callback, fallible reservation or new Resource decision: it detaches the old
children, applies the prepared metadata changes and publishes the optional Text.
A terminal refusal leaves the outer tree/base/details change unpublished;
earlier conversion effects remain.

## Work, storage and limits

The DOM helper consumes the runtime's actual remaining work and cumulative
allocated-byte counters. Both absolute counters are copied back before success
or failure propagates. Failed preparation does not restore spent work or refund
temporary allocation charges, and the helper grants no new allowance. Existing
100,000-step author and bootstrap limits, the cumulative 8 MiB script budget,
100,000-node cap, 32 MiB retained DOM byte bound and script/event reset behavior remain
unchanged.

The implementation charges its reached child/ancestor scans, explicit scratch
growth, names, conservative map operations, final child vector, node admission
and actual arena growth. It stages affected base/details handling only when
needed; ordinary replacements do not clone the document or all metadata maps.

This is bounded Resource refusal, not universal allocation recovery. B-tree
allocation and the existing URL/encoding/IDNA machinery still allocate
infallibly. The inherited URL work allowance and retained-result charge do not
provide exact accounting for every temporary allocation inside those libraries.
Conservative logical heap charges are not a process-wide allocator meter, and
process OOM is not converted into a recoverable DOM exception.

## Focused validation and remaining work

The first focused runs pass 29 runtime groups, nine DOM container groups, two
direct Page tests and two confined-worker tests: 42 added groups. The original
eight JS cases and ten separately authored exact-unit cases pass in both modes,
36 ordinary-success invocations in fresh runtimes. The original sources and
expectations are unchanged. The legacy lossy textContent assertions in the UTF-16 boundary and
String.concat tests are retained in history and explicitly migrated to exact-unit
expectations. The first full default run exposed the latter assertion; its
failure log is retained, and the correction changes no production code.

The browser witnesses check fresh Text identity versus retained CharacterData
identity, detached exact data through worker snapshots, literal replacement-glyph
rendering, changed style/text after layout, title and clean textarea data, base
selection and details-group/summary state. These are checks of the named effects;
they do not establish complete synchronous children-changed reactions or the
dirty textarea model.

Measured raw bootstrap retains 6,640 work units, charges 1,798,334 bytes, and
contains 677 objects with capacity 677, 321 native registry entries and 25 legacy
prototype entries. Four accessor metadata bags and the shifted Node property
insertion work are charged; the author-entry budget rule is unchanged.

All 1,619 default tests on Rust 1.88 and 1,742 Vulkan-feature tests on
Rust 1.98 pass, including ignored confinement tests. Strict all-target Clippy
passes for the native feature on Rust 1.98 and the presenter on Rust 1.88;
formatting passes. The final release preserves every complete observation across
4,088 established case modes and 372 controls. CharacterData remains 34/34 and
PI remains 38/40; the two PI host-prototype mutation gaps are unchanged.

The original 16 Node modes advance from zero to 16 passing expectations, and the
separate exact-unit inventory advances from zero to 20. All sources and ordinary
success expectations are unchanged. Eight generic control modes pass on both
releases. Eight feature-dependent controls become healthy once the accessors
exist, giving 16/16 controls on the final release; they are not counted as
ordinary case gains. The [summary](evidence/node-data.json) and
[verified archive](evidence/node-data.tar.gz) retain complete sources, manifests,
commands, initial failures, before/after observations and reviews. The
[preceding producer checkpoint CI](evidence/dom-production-ci.json) passed all
nine jobs.

Attr/CDATA and independent Document ownership, MutationObservers, live
Range/NodeIterator updates, custom-element reactions, shadow-slot work and
complete native Node members remain incomplete. A later
[exact append increment](append-domstrings.md) preserves separate string
arguments and failure prefixes without changing the Node checkpoint above.
The later [Document.title increment](document-title.md) migrates title writes
and normalized reads. Legacy `innerText` and textarea setters, attribute writes, HTML parser input, exact
nonscalar HTML serialization and unit-aware native editing remain separate
boundaries. Replacing a title/style/textarea element's children through
`textContent` does not make those other write APIs exact. This is not full Node
or DOMString conformance, production security certification or a GPU performance
claim.

Normative references: [nodeValue](https://dom.spec.whatwg.org/#dom-node-nodevalue),
[textContent](https://dom.spec.whatwg.org/#dom-node-textcontent),
[string replace all](https://dom.spec.whatwg.org/#string-replace-all),
[nullable conversion](https://webidl.spec.whatwg.org/#es-nullable-type) and
[attribute bindings](https://webidl.spec.whatwg.org/#es-attributes).
