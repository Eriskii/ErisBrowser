# Node.compareDocumentPosition

`Node.prototype.compareDocumentPosition` is an ordinary method for the represented
node kinds. It compares current ordinary parent/child relationships while keeping
template content separate. The implementation does not add Attr, ShadowRoot or
independent document ownership.

The method compares the argument with its receiver in represented ordinary
trees. The result is `0` for the same node, `2` or `4` for an earlier or later
branch, `10` for an ancestor and `20` for a descendant. Document aliases share
one canonical identity. A template's actual content fragment has a separate
ordinary root from its host; direct ordinary children of the template participate
in its own tree. Character data, element names, attributes, namespaces,
visibility and structural equality do not determine position.

Disconnected trees return `35` or `37`: DISCONNECTED and
IMPLEMENTATION_SPECIFIC together with one direction bit. The direction
uses the original canonical operand IDs: a smaller argument ID gives `35`, a
larger argument ID gives `37`. The IDs remain stable in the current append-only
arena, including retained detached nodes. This is a local implementation policy,
not pointer ordering, a cross-document ownership policy or a global tree order.
Portable independent cases accept either allowed direction, require opposite
reversal, and require stable results for unchanged disconnected operands. Private
tests separately verify the chosen ID policy.

The cached nonconstructible method has name `compareDocumentPosition`, length
one, and an ordinary writable/enumerable/configurable descriptor. Its receiver
must be authentic. The required argument is **nonnullable**: omission, null,
undefined, primitives and forged nodes throw TypeError. Receiver authentication
precedes argument conversion; interface conversion calls no authored coercion
hook. Public `parentNode`, `childNodes`, `nodeType` or prototype lookalikes do not
replace internal identity and links. Saved calls, own shadows and prototype
property replacement, deletion and restoration follow ordinary lookup, without
fallback resurrection.

Receiver and argument expressions, including ignored extra arguments, finish
before invocation. Their completed mutations and thrown values remain observable.
The native body then uses one immutable Document borrow, calls no authored
callback and performs no DOM mutation. Resource refusal can retain those earlier
argument effects; it does not roll them back or refund work.

The [DOM algorithm](https://dom.spec.whatwg.org/#dom-node-comparedocumentposition)
and [ordinary tree order](https://dom.spec.whatwg.org/#concept-tree-order) define
the masks and relationships. [Web IDL interface conversion](https://webidl.spec.whatwg.org/#es-interface)
and [regular operation rules](https://webidl.spec.whatwg.org/#es-operations)
define authenticity, required arguments and method behavior. Actual Attr nodes
and their special comparison branches, ShadowRoot, CDATA and independent
cross-document ownership remain unrepresented. This is not the complete Node
interface.

## Traversal and charged work

Distinct operands undergo two complete ordinary root/depth walks before roots
are compared. Each reached parent must be a valid container and contain exactly
one reciprocal selected-child link. The endpoint at 256 edges is admitted; a
257th edge refuses. Reached local shape and canonical Document identity are
checked. A same-ID result performs those local checks but does not walk unvisited
ancestors or audit unrelated graph state.

Equal roots proceed through paid depth alignment. Divergent branches then ascend
in pairs to their common parent and scan that parent's entire child vector for
the two selected positions. The immutable borrow keeps the earlier edge checks
valid during alignment. No ancestor vector, visited bitmap, arena-global scan,
payload copy or template-host traversal is introduced. The comparison body owns
no traversal or payload allocation. Generic invocation, native-name copying,
metadata installation and error construction retain their separate allocation
charges and inherited allocator limitations.

The implementation charges the following work for successful body paths. Let `D` be the sum of the two operand depths, `S` the sum of
all parent-vector lengths reached by both root walks, and `A` the original depth
difference. For divergent branches, `J` counts paired ascent rounds including
the final common-parent round, and `L` is that parent's child-vector length.

| Result path | Body work |
| --- | ---: |
| Same ID after local checks | `66` |
| Disconnected roots | `118 + 29D + 4S` |
| Ancestor or descendant | `130 + 29D + 4S + 10A` |
| Divergent branches | `143 + 29D + 4S + 10A + 20J + 6L` |

Each complete walk costs `24 + 29D_i + 4ΣS_i`. The selected native route
adds `60` work units and `64` charged bytes beyond the body; generic VM lookup,
invocation and diagnostics remain additional. Wide parent vectors can exhaust
the fixed work allowance at shallow depth. Admitting the depth endpoint is not
an unconditional promise that every 256-edge shape fits the allowance.
These are logical charges, not machine instruction counts, physical memory
measurements or wall-clock timings.

No quota, success-only author grant, script/event reset, worker authority or
transport format changes. Existing root, connection, equality and other predicate
bodies and successful fees remain unchanged; their regression filters pass.
Whole-graph malformed-state detection and comprehensive allocator recovery are
not claimed.

## Metadata and measured initialization

The new method is placed between `isSameNode` and `contains`. The represented
pristine prototype has 30 keys: `isConnected`, `nodeValue`, `textContent`,
`getRootNode`, `hasChildNodes`, `normalize`, `isEqualNode`, `isSameNode`,
`compareDocumentPosition`, `contains`, the eighteen constants, `constructor`,
then `Symbol.toStringTag`. The constructor retains 21 keys.

The constant batch starts with eleven prototype entries, builds a
29-entry map and reserves thirty order slots for the later constructor property.
The constructor stays at three initial entries, 21 map entries and 21 order
slots. The pinned admission still uses four prototype tree nodes and three
constructor nodes. Guards, retained descriptors and native handles stay explicit;
adding the method does not waive admission or relax the owner shape.

Measured raw initialization retains 6,599 work units, charges 1,768,914 logical
bytes and has 689 objects/capacity, with 321 native registry entries and 25 legacy
prototypes. Relative to connected, that is 693 additional work units, 3,412
charged bytes and one additional object, matching the source-derived ledger.
The typed storage admission uses the actual represented types; these observed
sizes are not ABI promises, physical heap measurements, performance evidence or
security results. Numeric quotas and reset boundaries are unchanged.

## Validation and retained history

Rust 1.88 passes **1,938 default tests** and Rust 1.98 passes **2,061 native
Vulkan tests**, including ignored confinement tests. Both strict Clippy checks,
formatting and the release adapter build pass. All sixteen validation commands
pass on their first attempt. The 35 new groups comprise sixteen independent
Runtime bodies exercised in both language modes, seventeen private boundary
groups, and Page/confined-worker browser witnesses.

Nine focused commands contain 286 passing observations across 278 unique test
names, with no failing attempt. The exact/one-short private checks cover complete
walks, alignment, final sibling scan/result, 256-edge endpoints, overdepth,
selected malformed links, saved Native invocation and terminal resource refusal.
They preserve body DOM state and distinguish completed argument effects.
Two live runtimes retain separate mutable method metadata. These checks do not
validate every malformed host graph or establish general browser performance.

The Page and confined-worker witnesses load once and dispatch one real click.
The same branch moves through a detached fragment, template content, ordinary
template children and back to the visible pair; its visible order changes from
Alpha/Beta to Beta/Alpha. All 22 captured node identities and the arena length
remain stable, with no click-created nodes. Saved and ordinary method calls,
reciprocal links, exact text, fixed base URL, title/status, two colored samples
and nonempty glyphs pass. Independent script-free literal trees specify each
whole canvas. Explicit mutations, not comparison itself, cause the visible change.

The release passes **32/32 new case modes and 16/16 paired controls**, from a
published connected baseline of zero matching case modes and four healthy common
controls. All 32 old missing-feature cases and twelve feature-control modes
reported TypeError before implementation. Genuine successful prerequisites
precede negative catches, and a wrong-tail control is healthy only with its
successful same-mode positive partner.

Across nineteen established profiles, **4,362 of 4,364 case rows** and **520 of
524 controls** remain byte-exact. The only new losses are the old connected
complete-inventory's two case modes and four controls. The cases retain
Error/Error failures; the controls fail their old prerequisite with
TypeError/TypeError and false health. All thirty older unhealthy inventory rows
remain unchanged. Established healthy totals move from 4,268/504 to 4,266/500.
No frozen source or raw health is rewritten. The new 30-key inventory and its
positive/wrong pair pass independently. All six historical inventory job exits
and the aggregate retain exit one; the comparison and classifier both exit zero.

Initial compilation and all 35 new groups passed before three descriptive
bootstrap literals in two test files were updated from the measured diagnostic.
No failing bootstrap run occurred, and production source did not change after
initial compilation. The nineteen-path initial hold and twenty-one-path final
hold retain both before images and the exact three-literal correction. Source
formatting, original fixtures, independent expectations and review history remain
retained.

The first complete independent data-only audit clears with **378 bound inputs**,
no findings and unchanged pre/post bindings. Its source preparation retained a
focused-summary schema correction and the original reader hold before execution;
no complete audit failed. The audit reconstructs complete raw rows, every literal
test status, command exits, source changes, current oracle fingerprints and paired
control health. It does not rerun an engine or replace the browser checks.

## Evidence and remaining limits

The [summary](evidence/node-position.json) and [archive](evidence/node-position.tar.gz)
retain held source, frozen independent inputs, complete validation and replay
logs, exact corrections, classifier policy and audit provenance. Existing older
records remain historical. The release adapter SHA-256 is
`f0b249ba7a726e943685315c95bb55b5ba8ca5e3add50051e1c6553bbcbb7007`;
the final twenty-one-path source manifest is
`c6d4423dadf42a2812b69717fa86bb4ddae0543c9dd1b1caa092151b6c8f5377`.

Actual Attr ordering, ShadowRoot, CDATA, independent document ownership, remaining
DOM/JS interfaces and inherited allocator boundaries remain limits. Full web
compatibility, production security, comprehensive verification and the requested
Chromium performance threshold remain unfinished.
