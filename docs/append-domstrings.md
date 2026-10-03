# Exact string arguments in append

`Document.append`, `Element.append` and `DocumentFragment.append` preserve exact
UTF-16 string arguments, including unpaired surrogates. Each string becomes its
own Text node. A high unit in one argument and a low unit in the next remain two
distinct nodes; an exact descendant-text read can join their units into a pair
without rewriting either payload.

This extends [checked insertion](document-append.md) with the existing
[canonical data builder](dom-production.md). Metadata, defining-interface
identity, receiver checks and the hierarchy-checking routine are unchanged.

## Conversion, materialization and insertion

The defining-interface receiver check precedes argument conversion. Genuine
Node arguments retain their identities without string coercion. Every other
argument undergoes DOMString conversion in order. All conversions finish before
append creates any Text or moves any argument node; later callbacks can therefore
change the tree and consume capacity before fresh admission.

The converted strings then materialize separately, in argument order. One
string containing a valid pair can use scalar UTF-8 storage; separate unmatched
arguments each keep exact UTF-16 storage. An empty string still creates an empty
Text and needs a node slot. Explicit undefined and null become the strings
`"undefined"` and `"null"`. With no arguments there is no Text: the existing
empty-fragment path runs and leaves the destination's children unchanged.

One materialized argument goes directly to checked insertion. Zero or multiple
arguments use a temporary fragment; multiple arguments move into it in their
original order, including duplicates, before the final destination check.

Failure preserves the completed prefix:

- A throwing conversion retains earlier author effects but creates no outer
  Text nodes and moves no outer argument nodes.
- A later work/storage refusal during materialization can leave earlier Texts
  detached. A fragment-creation refusal occurs after all its Texts materialize.
- A later assembly or final hierarchy error retains completed fragment moves.
  There is no whole-call rollback.

Document rejects Text children, including empty Text. For example,
`document.append(document.documentElement, '\uD800')` first moves the root and
new exact Text into a fragment, then throws `HierarchyRequestError` at final
Document insertion. Author code can restore the same root; the exact Text stays
in the retained fragment. The unmatched unit itself is valid DOMString data.
Catchable conversion/hierarchy exceptions remain distinct from terminal
implementation resource limits.

## Admission and accounting

The argument vector retains immutable JavaScript strings instead of allocating
replacement-character UTF-8 copies. After conversion, the shared builder pays
for full classification and emits one chosen payload: UTF-8 for a scalar
string, or two bytes per unit when an unpaired unit occurs. The Units path also
pays for its separate canonicality check. Fresh per-node and document-ledger
admission uses that chosen byte count before output allocation.

Each Text retains its logical node charge. Payload emission finishes and
publication work is prepaid before the final fallible node-arena reservation;
only then does the owned creator move its buffer into the new node.
Temporary-fragment creation uses the same typed arena
growth preflight at its existing position in the operation. Actual growth pays
for moving the current arena and the required buffer using the actual Node
header size. Spare-capacity cases perform no relocation. Admissions remain
per node, so a bulk reservation cannot erase the observable earlier prefix.

The 100,000-step author allowance, cumulative estimated 8 MiB script allocation
budget, existing string/per-node limits, 100,000-node cap and 32 MiB retained DOM
payload cap are unchanged. Detached Texts and fragments remain in the arena;
their data and temporary script allocation charges are not refunded. Two older
private tests now reserve test-owned headroom to isolate their original
node-cap/move assertions; separate tests exercise real arena-growth refusal.

Raw insertion still uses the existing child-vector, B-tree, disclosure-group and
base-URL machinery. Its work/storage preflights remain, but its infallible
allocations and URL/encoding/IDNA temporary allocation limits are inherited.
This increment does not turn those operations into a fully fallible transaction
or an exact process-wide allocator meter. The parser's insertion path is
unchanged.

## Validation

The frozen nine-path change passes 1,637 default tests on Rust 1.88 and 1,760
Vulkan-feature tests on Rust 1.98, with no failures or ignored tests. Strict
all-target Clippy passes on Rust 1.98 with Vulkan rasterization and Rust 1.88
with the presenter; formatting passes. All compile/test commands passed on their
first attempts, with no production or assertion correction after compilation.

Eighteen test groups were added: seven independent cases, nine admission/prefix
groups and two Page/worker tests. Focused validation passed 78 unique tests
across 81 executions. Raw bootstrap is unchanged: 6,640 remaining work units,
1,798,334 charged bytes, 677 objects/capacity, 321 native entries and 25 legacy
prototype entries. Budgets, reset points and metadata are unchanged.

The final release preserves every complete result across 4,124 established case
modes and 388 controls. The fourteen new ordinary-success modes advance from
four to fourteen passing expectations, with ten gains and no losses or other
changes. Twelve control modes pass on both releases; the four exact-payload
controls become healthy, giving sixteen of sixteen on the candidate. All sources
and ordinary success expectations remain frozen; controls are counted separately.

The [summary](evidence/append-domstrings.json) and
[verified archive](evidence/append-domstrings.tar.gz) retain the original
preparation drafts, approved capacity setup corrections, source manifests,
commands/logs, release fingerprint, complete before/after records and reviews.
The initial unexecuted literal draft gained a successful saved Document.append
prerequisite check before its corrected bytes were frozen; both versions remain
in preparation history.

The preceding [Node checkpoint CI](evidence/node-data-ci.json) completed all
nine jobs successfully. That receipt identifies the published Node commit and
does not report append implementation results.

The browser witness checks literal units and separate Text IDs, retained old
data, cross-node aggregate joining, a Document hierarchy-failure fragment and
restoration of the same root. Both load and click compare the full canvas with
independently literal scalar replacement-glyph references. The reference keeps
the node boundaries and chosen payload byte totals. Worker snapshots must
preserve the same exact live and detached data through unchanged `ERWA` transport.

Earlier [producer](dom-production.md), [Node accessor](node-data.md) and
[checked-append](document-append.md) reports remain records of their own
checkpoints. This increment does not implement `prepend`, `replaceChildren`,
exact attribute or remaining `innerText`/textarea writes, the dirty
textarea model, complete mutation/custom-element reactions, cross-document
adoption or full DOM compatibility. A later [Document.title increment](document-title.md)
adds ordinary exact title accessors and normalized reads; it does not alter the
append checkpoint measurements above.

Normative references: [converting nodes into a node](https://dom.spec.whatwg.org/#converting-nodes-into-a-node),
[ParentNode.append](https://dom.spec.whatwg.org/#dom-parentnode-append),
[pre-insertion validity](https://dom.spec.whatwg.org/#concept-node-ensure-pre-insertion-validity)
and [Web IDL operation conversion](https://webidl.spec.whatwg.org/#es-overloads).
