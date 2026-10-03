# Node predicates

This increment adds ordinary `Node.prototype.hasChildNodes`, `isSameNode` and
`contains` methods for the represented nodes. They read current internal node
identity, children and parent links. They do not change the Document or decode
character data. All 32 new strict/sloppy modes pass, with 16 healthy controls;
historical full-inventory failures remain recorded separately.

## Ordinary methods and conversion

| Method | Length | Result |
| --- | --- | --- |
| `hasChildNodes()` | 0 | Whether the receiver has any ordinary children, including empty Text or Comment nodes. |
| `isSameNode(other)` | 1 | Whether the nullable Node argument identifies the receiver. |
| `contains(other)` | 1 | Whether the nullable Node argument is the receiver or an ordinary descendant. |

All three prototype properties are writable, enumerable and configurable data
properties. Each realm has one cached native function identity per method, with
ordinary `name` and `length` descriptors and Function prototype inheritance.
They are not constructors. Saved functions continue to work after a property is
shadowed, replaced or deleted. Deletion does not expose a hidden fallback.

Invocation checks the authentic receiver before required argument presence and
nullable interface conversion. `isSameNode()` and `contains()` throw TypeError
when the first argument is absent. A present `null` or `undefined` converts to
nullable null and returns false. Any other non-node value throws TypeError;
inheriting from `Node.prototype` does not manufacture a node brand. An authentic
node constructed with an alternate prototype retains its brand.

No authored `toString`, `valueOf`, `Symbol.toPrimitive`, `parentNode`, `childNodes`
or prototype getter participates in these native checks. Extra arguments are
ignored without conversion. JavaScript still evaluates all argument expressions
before invocation, including ignored extras. A move or removal performed by an
argument expression is therefore visible to the method's current tree read.

The authentic Document singleton and its public parent alias compare as the
same node. `isSameNode` compares identities, regardless of equal payloads or
position. Detached nodes contain themselves; unrelated detached trees do not
contain each other. These operations do not introduce a cross-document identity
scheme for reusing a Runtime with unrelated Rust Documents.

## Ordinary tree boundaries and local validation

`hasChildNodes` tests the receiver's direct child vector. A template's separate
content fragment is not an ordinary child. `contains` starts with its argument,
checks inclusive identity, and follows only ordinary parent links. It does not
follow a template fragment's host or enter descendants through a host
association. Explicitly calling the same methods on the content fragment reads
that separate tree.

The path validates each reached node's local shape and each followed parent's
container shape. It scans that parent's child IDs to require exactly one
reciprocal membership for the current node. Unrelated siblings' payloads and
subtrees are not read. No whole-arena scan, traversal vector, visited-ID map or
hidden cache is introduced.

A match at the snapshot-admitted boundary of 256 ordinary edges is accepted;
this can require 257 node visits. Self-parent links refuse directly. Cycles and
longer paths that do not first reach a result refuse within the bound. Reached
invalid IDs, missing or duplicate membership, noncontainer parents, childful
leaves, parented Fragments and misplaced Documents produce TypeError. This is
an internal malformed-host-data policy, not an additional standard hierarchy
exception or a complete graph validator. A self or ancestor result does not
inspect links above that result; null needs no traversal, and identity alone
does not validate unrelated graph state.

## Work and allocation admission

Each method pays receiver admission before inspecting its internal ID. Required
methods separately pay presence and nullable conversion checks. `contains`
pays each local visit and parent check, then prepays the entire selected
parent's child-ID scan. Its work is proportional to reached ancestors plus
scanned sibling IDs. It is not claimed to be independent of sibling count.
A wide valid sibling vector can exhaust the existing author work allowance.

The bodies allocate no owned traversal or payload storage. They return booleans
and leave all Document fields unchanged on success or refusal. The existing
native dispatch still allocates and charges its temporary invocation-name
copy, and generic VM paths retain their own admission. Inherited diagnostic
String construction and physical allocator failure remain outside any claim
of complete allocation accounting or recoverable memory exhaustion.

Admitted work remains spent even if a later check refuses; an early duplicate
may retain the full prepaid sibling-scan charge. Resource refusal follows the
existing terminal entry behavior and does not run an ordinary JavaScript
`catch` or `finally`. The predicates themselves cannot roll back earlier author
argument effects. Numeric limits, cumulative heap accounting, execution reset
sites, process authority and ERWA transport remain unchanged.

## Metadata order and constant publication

The represented Node operations appear in IDL order: `hasChildNodes`,
`normalize`, `isSameNode`, `contains`. At this checkpoint, the pristine prototype has 26 own
keys: `nodeValue`, `textContent`, those four operations, the eighteen constants,
`constructor`, then `Symbol.toStringTag` in `Reflect.ownKeys`. This is the
current represented subset, not complete Node interface coverage. The original
constants, existing accessor/native identities and constructor flags remain
unchanged.

Before constant installation the prototype has seven entries, including its
tag; the constructor has three. Exact owner and physical-order guards use
these separate shapes and reserved order capacities of 26 and 21. Adding the
constants produces a 25-entry prototype map before its later constructor
property, and a 21-entry constructor map.

The pinned Rust 1.88 and 1.98 sorted map builders need four allocated tree nodes
for the 25-entry map: the initial leaf, root and second leaf at item 12, and a
third leaf at item 24. The final right-border repair moves four pairs without
allocating. The 21-entry constructor still needs three nodes. Admission covers
both prototype overflows, all typed nodes, complete map buffers, full UTF-16 key
comparison costs and the 48-entry sort scratch per map even for sorted input.
Old map/order charges remain cumulative.

All work, heap charges and fallible vector reservations finish before either old
map is consumed. Actual key ordering is checked before collecting each map;
both completed maps stay local until both succeed. A quota refusal occurs
before consumption. A later private invariant failure discards the incomplete
initializer instead of publishing a usable Runtime. Standard BTree/sort
allocation retains its inherited infallible process-memory boundary.

Measured raw bootstrap leaves **8,807 work units** and charges **1,755,715 bytes**,
with **685 objects/capacity**, 321 native entries and 25 legacy prototype entries.
Relative to Normalize, the three new metadata bags and adjusted constant build
consume 1,392 additional work units and 11,214 charged bytes. These observations
match the previously frozen ledger predictions. Numeric quotas and reset sites
are unchanged.

## Historical inventories and validation status

The frozen Node-constant fixture and frozen Normalize fixture remain byte-exact.
The older Node inventory already has two failing case modes and four unhealthy
reflected-order controls after Normalize; those six historical observations
remain unchanged. The new predicates also change two Normalize full-inventory
case modes and four Normalize reflected-order controls. Their original
expectations remain attached to the raw candidate outcomes. A matching
negative exception is insufficient when its same-mode positive partner fails.
The frozen classifier accepts exactly these additional six inventory changes;
it does not relabel their failures or claim that all historical controls are
healthy. No other established observation or input changes.

Private historical inventory wrappers run in disposable realms. The old Node
wrapper saves and removes all four later operations; the old Normalize wrapper
saves and removes the three predicates. Each invokes its unchanged historical
oracle, then restores descriptor/value identities. Restoring a property cannot
restore its creation order, so these realms are discarded. Separate pristine
realms run the new independently authored literal 26-key inventory. External
historical observations are never wrapped or relabeled as passes.

The new source contains sixteen independent bodies, each run in fresh strict
and sloppy realms, and twelve private runtime groups. Private witnesses
cover exact and one-short work/heap admission, every cut of an ancestor walk,
receiver/arity order, nullish arguments, all represented leaves and Document
aliases, the accepted 256-edge boundary, malformed local links and cycles,
unrelated-arena independence, terminal wide-sibling refusal, saved-call cleanup,
and metadata isolation between two live realms. Existing constant tests retain
their identity/order and refusal assertions with the seven/three input shapes.

The first all-target compilation and all **30 new groups** passed: 28 runtime
groups and the direct Page and confined-worker integrations. Successful focused
commands contain **149 observations covering 145 distinct groups**. Including
the original stale-snapshot attempt, the retained focused history contains 175
pass observations and two failed descriptive assertions. The initial bootstrap
filter recorded 26 passes and those two failures; measured results justified
three numeric-literal updates in two private snapshot files.

The compiler's unused-`mut` warning in the literal reference Page setup was
corrected without changing its behavior. The first strict native Clippy run
then found three unnecessary cloned argument slices in a private test. Those
became `std::slice::from_ref` calls with the same arguments and assertions.
Both strict native and presenter Clippy, formatting and the release build pass.
Original sources, warnings and failed attempts remain retained; production and
both historical/new frozen JavaScript fixtures are unchanged by these
corrections. Rust 1.88 passes **1,805 default tests** and Rust 1.98 passes
**1,928 native Vulkan tests**, including ignored confinement checks.

The first native full-suite attempt recorded 1,616 passes and one failure in
`date_host::tests::host_run_capture_silent_blocked_helper_obeys_cancel_and_deadline`:
the helper's silent-read marker was not observed before capture returned under
the existing one-second deadline. The failed assertion precedes the action and
result assertions, so the actual cancel/timeout action and returned error are
not recorded. The unchanged isolated test passed, followed by the complete
native suite with `--test-threads=4`. The test's deadline and assertions were unchanged.
Scheduling sensitivity is an inference from those observations, not a proven
cause; the original failed run remains retained.

The two browser integrations retain eighteen explicit IDs across load and one
real click. The click moves a subtree to the other visible column and detaches
a separate retained branch without creating nodes. Saved predicates check both
phases, own shadows and ordinary template children versus separate content.
The title changes from `Predicates ready` to `Predicates done`; independent
literal HTML references check complete canvas equality, green/blue pixels,
visible text position and status text. The worker path also checks the decoded
snapshot and confines execution through the existing worker boundary.

The complete release comparison retains **4,236 established case records and
460 controls**. Of these, 4,234 cases and 456 controls are unchanged. The only
new losses are the two Normalize inventory modes, now ordinary `Error`
exceptions, and four Normalize order controls, now `TypeError` and unhealthy.
Established cases matching their declared expectations fall from 4,148 to 4,146;
healthy controls fall from 456 to 452. The older Node two-case/four-control
failures remain unchanged. Raw old Node and Normalize profile jobs both retain
exit 1, as does the aggregate replay job. The separate classification receipt
identifies the exact inventory changes without changing raw results or control
health.

All **32 new predicate modes pass**, up from zero on the Normalize release.
All **16 new controls are healthy**, up from four. Each negative control also
requires its same-mode positive partner to meet its expectation. No input drift
or other established result change occurred. These selected records are not a
platform-wide conformance rate, and no host speedup is claimed.

The [summary](evidence/node-predicates.json) and
[archive](evidence/node-predicates.tar.gz) bind the sources, concrete fee ledger,
commands, logs, original failures, source corrections, raw before/after reports,
classifier and independent reviews. The release adapter is 3,756,760 bytes,
SHA-256 `1d0a4eaa3a900f759b574de1454c91ae5be7570e0a7c3be51a9db169162dc07f`,
built from final source manifest
`a4ae87eeeacd46964f1f8b193f45412bf2e11a9bcca1f10358aae741336cf9ef`.
The preceding [Normalize CI receipt](evidence/node-normalize-ci.json) records
all nine jobs passing for `618fa359`; it is not a predicate-checkpoint CI result.

These methods do not add shadow-tree/composed ancestry, cross-document adoption,
CDATA/XMLDocument construction, or missing Node members. Existing unrelated DOM
and allocation limitations remain.

A later [root lookup checkpoint](node-root.md) adds `getRootNode` before
`hasChildNodes` and expands the represented prototype to 27 own keys. The
26-key inventory and its original outcomes here remain historical evidence.

Normative references: [Node](https://dom.spec.whatwg.org/#interface-node),
[hasChildNodes](https://dom.spec.whatwg.org/#dom-node-haschildnodes),
[isSameNode](https://dom.spec.whatwg.org/#dom-node-issamenode),
[contains](https://dom.spec.whatwg.org/#dom-node-contains),
[interface conversion](https://webidl.spec.whatwg.org/#es-interface), and
[operation bindings](https://webidl.spec.whatwg.org/#es-operations).
