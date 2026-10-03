# Node.getRootNode

This increment adds an ordinary `Node.prototype.getRootNode(options)` method for
represented nodes. It returns the highest ordinary ancestor after options
conversion has finished. A connected tree returns the canonical public Document;
a detached node or fragment returns the retained Node value at its own root.
The default and native Vulkan suites, independent release expectations, Page
and confined-worker witnesses pass. Historical inventory differences are retained
explicitly below.

## Options and callback order

The method authenticates its receiver before converting options. Omitted options,
`undefined` and `null` select the default without reading a property. Any other
primitive throws TypeError. Object options perform exactly one ordinary inherited
Get of `composed`, including nonenumerable or inherited accessors, with the
original options object as `this`. An undefined member selects false; other
members use noncoercive boolean conversion. A truthy object does not invoke its
`toString`, `valueOf` or `Symbol.toPrimitive`.

Argument expressions, including ignored extra arguments, run before native
invocation. An options getter can move the receiver or an ancestor, detach a
subtree, call other supported methods or throw. The root walk keeps only the
authentic receiver ID across that callback and then reads fresh internal links.
Thrown values and completed callback effects are preserved. It does not consult
authored `parentNode`, `childNodes` or `getRootNode` shadows while walking.

No ShadowRoot kind is represented. Both composed values therefore follow the
same ordinary parent chain. In particular, a template content fragment remains
a separate root even with `composed: true`; its host association is not a parent
link. Ordinary children added directly to the template remain in the template's
ordinary tree. This increment does not add shadow-including ancestry or
cross-document adoption and identity.

## Ordinary metadata

The prototype property is writable, enumerable and configurable. Its cached
native function has name `getRootNode`, length zero, ordinary Function inheritance
and no construction behavior. Saved calls continue to work after own shadows,
prototype property replacement or deletion; deletion does not reveal a hidden fallback.
Inheriting from `Node.prototype` does not manufacture a brand. Authentic nodes
created through the supported alternate-constructor-prototype path keep theirs.

At this checkpoint, `Reflect.ownKeys(Node.prototype)` has
27 keys: `nodeValue`, `textContent`, `getRootNode`, `hasChildNodes`, `normalize`,
`isSameNode`, `contains`, the eighteen Node constants, `constructor`, then
`Symbol.toStringTag`. The constructor's 21-key inventory remains unchanged.
This is a literal inventory of implemented members, not complete Node coverage.

The constant batch guards eight existing prototype entries and three constructor
entries. Its prototype map has 26 entries before the later constructor property,
and its order buffer has 27 slots. The pinned sorted map construction still needs
four allocated tree nodes for that map and three for the constructor map. It
retains all comparison, scratch, map and owner charges, and admits both maps
before consuming either original. Standard BTree/sort allocation retains its
inherited infallible process-memory boundary.

## Bounded reads and admission

The native walk pays each reached node and parent check, then prepays the whole
selected parent's child-ID scan to require exactly one reciprocal membership.
Work depends on ancestor depth and scanned sibling IDs. Unrelated sibling
payloads and subtrees are not traversed, and no arena-wide bitmap or cached root
is created. The accepted 256-edge endpoint requires up to 257 node visits.

Reached invalid IDs, malformed local shapes, missing or duplicate reciprocal
membership, self-parent links and overlong paths refuse explicitly. Bounded
cycles cannot return a fabricated root. This is a local policy for malformed
Rust host state, not complete graph validation; unreached state is not audited.
Canonical Document conversion additionally requires the actual Document arena
root. A sufficiently wide valid sibling list can exhaust the author work budget.

Traversal allocates no owned storage. Object options separately allocate an
exact paid UTF-16 `composed` key Vec and Rc, followed by the existing ordinary
lookup and any callback charges. Nullish defaults avoid that key allocation.
Native dispatch's temporary name copy, generic VM work and inherited diagnostic
String allocation remain separate. No whole-call allocation, physical memory
recovery or host performance guarantee follows from allocation-free traversal.

All admitted work and cumulative allocation charges remain spent on refusal.
Callbacks share the current budget without resets. Terminal Resource refusal
does not run ordinary JavaScript `catch` or `finally`, and cannot undo callback
effects already completed. Numeric quotas, reset sites, authority and ERWA/EWB1
transport remain unchanged.

## Frozen expectations and validation

Sixteen independent fresh-realm bodies define 32 strict/sloppy expectations.
Four positive/wrong pairs define 16 controls; a wrong-tail exception is healthy
only when its same-mode positive partner succeeds. The published predicate
release baseline has zero matching new cases and four healthy controls. All
32 baseline case failures are ordinary runtime TypeError from unavailable
prerequisites, with no binding drift. The candidate passes all 32 cases and 16 controls.

Adding the member intentionally changes two frozen predicate full-inventory
case modes and four reflected-order controls. Those original sources and raw
observations remain visible: ordinary Error case losses and TypeError/unhealthy
control outcomes. The twelve older Node-constant and Normalize inventory
observations remain unhealthy and byte-exact. Every other established observation
is unchanged: 4,266 of 4,268 cases and 472 of 476 controls are byte-exact. New
pristine-realm 27-key expectations and paired controls are separate from these
historical checks. No historical failure is relabeled as a pass.

Private historical wrappers save and delete the later members before invoking
the unchanged earlier oracle, restore its descriptors afterward and discard the
realm because reinsertion cannot restore property creation order. External
replays retain the original realms and expectations without those wrappers.

The authored Page and confined-worker witness has load and one real-click phase.
Three inherited options getters temporarily move a receiver, move its ancestor,
then detach another retained branch; each returned root must reflect the fresh
tree. Twenty explicit IDs and the total node count stay unchanged across the
click. Ordinary template children, actual content and an unhosted fragment remain
distinct. Titles are `Roots ready` and `Roots done`; a separately authored scalar
reference checks the complete canvas, green/blue markers, text position and
nonempty glyph bands. Both browser paths pass these checks. No additional click nodes are created.

Rust 1.88 passes 1,837 default tests and Rust 1.98 passes 1,960 native Vulkan
tests, including ignored tests. Both full suites use four test threads. Formatting,
both strict Clippy checks and the release build pass. The 32 new groups comprise
16 independent Runtime groups, 14 direct Runtime groups and two browser groups.
Focused checks retain 182 passing observations covering 177 unique test names.
The initial bootstrap filter additionally retains 27 passes and two stale
snapshot failures; exactly three descriptive literals in two files were corrected
after measurement. No production change was needed after initial compilation.

Measured raw bootstrap leaves 8,318 work units and charges 1,758,971 bytes, with
686 objects and capacity, 321 native entries and 25 legacy prototype entries.
The delta is 489 work units and 3,256 charged bytes. These are logical accounting
measurements, not elapsed-time or physical-memory measurements.

The [summary](evidence/node-root.json), [archive](evidence/node-root.tar.gz) and
[validation record](VALIDATION.md) retain source holds, all attempts, release
bindings and complete comparisons. The preceding corrective commit's
[CI receipt](evidence/probe-fix-ci.json) records nine passing jobs; it is distinct
from this increment's local validation. No speedup or platform-wide conformance
rate is claimed.
Missing ShadowRoot, Proxy, BigInt, broader Node members and unrelated existing
DOM/allocator limitations remain outside this increment.

The subsequent [equality checkpoint](node-equality.md) adds `isEqualNode` and
expands the represented prototype to 28 keys. The 27-key inventory and its
original results above remain historical evidence. This root checkpoint's
[CI receipt](evidence/node-root-ci.json) records all nine jobs passing for
`51907b7`; it is separate from equality validation.

Primary references: [getRootNode](https://dom.spec.whatwg.org/#dom-node-getrootnode),
[Node IDL](https://dom.spec.whatwg.org/#interface-node),
[tree root](https://dom.spec.whatwg.org/#concept-tree-root),
[dictionary conversion](https://webidl.spec.whatwg.org/#es-dictionary),
[boolean conversion](https://webidl.spec.whatwg.org/#es-boolean), and
[operation bindings](https://webidl.spec.whatwg.org/#es-operations).

## Shared walk for connection state

The later [Node.isConnected checkpoint](node-connected.md) extracts the existing
1,771-byte paid ordinary-root loop into a shared internal helper. The loop and
options conversion remain byte-exact. `getRootNode` retains its receiver check,
options Get, fresh link reads, errors, canonical return values and costs. The new
getter uses that helper without options conversion and pays separately for its
Boolean result; template hosts remain outside ordinary ancestry.
