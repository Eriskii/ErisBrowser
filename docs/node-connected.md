# Node.isConnected

`Node.prototype.isConnected` is an ordinary readonly accessor for the represented
node kinds. It reports whether the current ordinary root is the canonical
Document. A connected template's content fragment remains a separate,
disconnected tree; ordinary children appended directly to that template are
connected. Visibility, names, character data and public property shadows do not
determine the result. Actual ShadowRoot and independent document ownership remain
unimplemented.

The getter checks its receiver's native identity. Its name is `get isConnected`,
its length is zero, and it cannot be constructed. The property is enumerable and
configurable, with no setter. Ordinary own properties and prototype replacement
or deletion work normally; saved getter calls retain their behavior. Extra
arguments are ignored after their expressions have been evaluated.

The implementation shares the bounded ordinary-root traversal with
[`getRootNode`](node-root.md). The existing options conversion, callback effects,
traversal charges and errors of that method are preserved. The connection getter
does not read options, call authored getters, or allocate traversal storage.
Generic invocation, native-name copies and error construction retain their
separate costs and limitations.

For `D` parent edges and parent child-vector lengths `S_i`, the getter body costs
`34 + 25D + 4ΣS_i` work units. Each visited parent must contain exactly one
reciprocal child link. The existing 256-edge limit and local shape checks remain;
wide parent scans can exhaust the work allowance at shallow depth. This is not a
whole-graph validation or a general browser performance result. No quotas,
execution resets, worker authority or transport formats change.

The [DOM definition](https://dom.spec.whatwg.org/#dom-node-isconnected) uses the
shadow-including root; this implementation currently represents ordinary trees.
[Web IDL attribute rules](https://webidl.spec.whatwg.org/#es-attributes) determine
the ordinary getter-only descriptor.

## Metadata and measured admission

The pristine prototype now has 29 keys: `isConnected`, `nodeValue`,
`textContent`, `getRootNode`, `hasChildNodes`, `normalize`, `isEqualNode`,
`isSameNode`, `contains`, the eighteen constants, `constructor`, then the tag
symbol. The constructor retains 21 keys. The constant batch starts with ten
prototype entries, produces 28 map entries and reserves 29 order slots.
The same four prototype tree nodes and three constructor nodes are admitted.

Measured raw initialization retains 7,292 work units, charges 1,765,502 bytes and
has 688 objects/capacity, 321 native registry entries and 25 legacy prototypes.
That is 524 additional work units and 3,275 charged bytes relative to equality.
The scoped native route adds 51 work units and 55 bytes beyond the getter body;
generic invocation costs remain additional. A unary 256-edge getter body uses
7,458 work units. These accounting figures are not wall-clock benchmarks.

## Validation and retained failures

Rust 1.88 passes 1,903 default tests; Rust 1.98 passes 2,026 native Vulkan tests.
Both strict Clippy checks, formatting and the release build pass. Thirty new
groups comprise sixteen independent strict/sloppy Runtime cases, twelve private
boundary groups and two browser witnesses. Initial compilation and all thirty
new groups passed on their first attempt.

The Page and confined-worker witnesses use a real click to move an existing
branch through a detached fragment, template content, ordinary template children
and back into the body. Both saved and ordinary getter reads check each state.
All 21 captured identities and the node count remain stable. Literal full-canvas,
glyph, title, status and green-to-blue checks verify the visible result; the
getter itself performs no mutation.

The focused checks contain 250 successful observations across 243 distinct test
names. A first constant-batch regression recorded eleven passes and six failures
at one shared test-setup assertion that still expected nine preconstant entries.
Correcting that one literal to ten produced seventeen passes. Three separate
bootstrap snapshot literals were updated after successful measurement. All
attempts and before images remain retained; production code did not change after
initial compilation. Across all focused attempts there are 261 passing
observations and those six original failures.

The release passes all 32 new case modes and 16 paired controls, compared with
zero cases and four healthy controls in the published predecessor. Of 4,332
historical case rows and 508 controls, 4,330 and 504 remain byte-exact. The only
new losses are the old equality inventory's two Error cases and four unhealthy
TypeError controls. Twenty-four older inventory failures remain unchanged.
The new 29-key expectations and their paired controls pass. No unhealthy row is
reclassified as a success; five historical commands and their aggregate retain
exit one.

## Evidence and limits

The [summary](evidence/node-connected.json) and
[archive](evidence/node-connected.tar.gz) retain exact sources, held independent
expectations, both test corrections, complete logs, before/after release rows and
an independent data-only result audit. Earlier holds and their additive audit or
review-metadata corrections remain intact. The
[preceding equality CI record](evidence/node-equality-ci.json) has nine successful
jobs and is separate from this checkpoint's local validation.

The first independent result audit stopped at a stale preparation-manifest path
after verifying source and test results. A reviewed reader-only correction binds
the actual retained preparation records; the second audit passes with 369 bound
inputs. The failed report and original reader remain in the evidence archive.

The release adapter SHA-256 is
`d9f1df99e5cf879bb8e8c07423b047a5da89f3de8e4ec5a327967bf5a9738715`.
The final twenty-path source manifest is
`d07f434be9dd87d0adf1c21a614fe77014311d83315ae0912dc3f931944620df`.
Full web compatibility, production security and the Chromium performance target
remain unfinished.
