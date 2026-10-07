# Node.cloneNode

`Node.prototype.cloneNode` is an ordinary method for the represented
non-Document node kinds. It creates detached copies with fresh node identities,
preserves exact stored character data and element fields, and optionally copies
ordinary descendants. Document cloning remains explicitly unsupported.

The cached nonconstructible method has name `cloneNode`, length zero and an
ordinary writable/enumerable/configurable descriptor. Its receiver must be an
authentic node. The optional argument uses Boolean conversion, defaults to false,
and invokes no authored conversion hooks. Argument expressions finish before the
method runs, including ignored additional arguments. Their mutations and thrown
values retain their ordinary effects. Saved methods, own shadows, prototype
replacement and deletion use ordinary property behavior; removing the method
does not expose the former virtual route.

Copies own their payloads and have fresh default interface prototypes. Runtime
expandos, assigned handlers, registered listeners and per-node prototype overrides
are not copied. Text, Comment and processing-instruction data retain unmatched
UTF-16 units. Processing-instruction data that was mutated to contain `?>` copies
as stored. Stored tag, attribute and namespace annotations, and DocumentType
name, optional identifiers and quirks flag, are copied directly. The copier does
not recreate data through parser factories or silently truncate a stored field.

An HTML template always receives a fresh content fragment, including a shallow
copy. Deep cloning processes the content hook before ordinary children, with
separate child vectors and fresh reciprocal host/content references. Cloning a
hosted fragment directly produces a hostless fragment. Template contents retain
separate ordinary roots; independent inert owner Documents remain unfinished.

The [DOM cloning algorithm](https://dom.spec.whatwg.org/#concept-node-clone)
specifies copying and insertion order. The HTML
[template hook](https://html.spec.whatwg.org/multipage/scripting.html#the-template-element)
and [details rules](https://html.spec.whatwg.org/multipage/interactive-elements.html#the-details-element)
define the represented additional effects. Copied open details queue fresh toggle
tasks. Later copied members of the same name group close themselves; coalescing
can retain a closed-to-closed task. Template-content groups are separate. Existing
groups, tasks, active dispatch and summary caches remain unchanged. Detached base
copies contribute their href storage without changing the connected base URL.

## Publication and limits

The checked copier stages the complete selected graph and its metadata before
publication. It admits payload copies, traversal, typed temporary storage,
metadata insertion, cleanup, logical nodes and actual arena growth. A final
fallible arena reservation precedes an admitted commit without further budget
checks. An ordinary helper or native-body Result refusal leaves the document,
source payloads, arena capacity and existing metadata unchanged; consumed budget
is not refunded. The inherited BTree allocator can still abort on process OOM.

This boundary covers the checked copier and native wrapper. Generic VM result
handling or subsequent authored work can exhaust after a successful copy, and
completed argument effects are retained. Whole-evaluation rollback is not claimed.

The root starts at depth zero. Ordinary edges and template-to-content edges each
add one; depth 256 is permitted and an attempted 257th edge refuses. Selected
links, kinds, duplicate visits and template pairs are checked. Shallow copying
does not traverse unselected descendants, and cloning a subtree does not audit
its former ancestry. A leaf with spare arena capacity needs no whole-document
scratch or metadata scan. Real growth of a full node arena is charged separately.
Existing node, retained-byte, work and heap limits remain in force; an allowed
depth does not imply every shape at that depth fits those budgets.

The represented prototype now has 31 keys, placing `cloneNode` after `normalize`
and before `isEqualNode`. The constructor retains 21 keys. Metadata initialization
removes the legacy callable and installs the ordinary method. Numeric quotas,
script/event resets, worker authority and transport formats remain unchanged.

## Remaining platform work

Independent Document identity and ownership, inert template owner Documents,
full ordered Attr/prefix/namespace storage, ShadowRoot, CDATA, mutation observers,
custom-element reactions, form dirty/value/checkedness cloning and script
already-started state remain absent or incomplete. Scalar attribute storage is
copied exactly as represented; this does not make every legacy attribute producer
an exact UTF-16 producer. This increment does not establish complete DOM cloning,
full web compatibility, production security or the requested Chromium performance
threshold.

## Object.hasOwn

The clone browser witness also uses `Object.hasOwn` to check that a copied node has no source expando. This increment adds the missing ordinary static method while retaining that witness unchanged. Its cached function has name `hasOwn`, length 2, `Function.prototype`, and no constructor behavior. The property on `Object` is writable and configurable, and is not enumerable. Saved references continue working after replacement or deletion.

The method converts its first argument to an object before converting its key with the string hint. A null or undefined target throws before any key-conversion hook runs; omitted arguments are undefined. Key conversion can change the target, and lookup observes those changes. The static call receiver is ignored. Presence checks do not invoke a property's getter or walk its prototype chain. This follows the [ECMAScript algorithm](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.hasown).

Supported reflection includes ordinary objects, function metadata, Symbol identity, array holes and own index descriptors, exact UTF-16 string indices, DOM expandos, and Window global properties. Lexical bindings are not Window properties. Primitive wrappers and observable key conversion share the existing work and allocation budgets; ordinary presence lookup does not copy a property's value. Resource refusal retains completed conversion effects and is not whole-expression rollback.

Existing host reflection limits remain explicit: Document's unimplemented `location` descriptor, Window event-handler descriptors, and unsupported string-property host surfaces still refuse. This addition does not implement Proxy objects or expand those host contracts. Its independent cases and focused tests are recorded separately from the clone groups; final validation outcomes belong in the evidence summary.


## Validation

Rust 1.88 passes **2,004 default tests** and Rust 1.98 passes **2,127 native Vulkan
tests**, including confinement tests. Formatting, both strict Clippy
configurations and release construction pass. The increment adds **49 clone
groups and 17 Object.hasOwn groups**. Fourteen successful focused commands contain
420 observations across 407 unique test names.

Both browser paths load the same fixture and dispatch one click. They retain all
20 original captured identities and add exactly ten copied nodes. The original
graph remains unchanged; copy labels, exact data, title and status match literal
expectations. Full-canvas comparisons and nonempty glyph checks pass in both Page
and the confined worker.

The release passes **34/34 clone modes and 16/16 controls**, from the predecessor's
0/34 and 4/16. A separate ownership replay passes **16/16 modes and 8/8 controls**,
from 0/16 and 4/8. Across the 4,396 established cases and 540 controls, 4,394 case
rows and 536 control rows remain byte-exact. Adding an ordinary prototype member
invalidates two previous complete-inventory expectations and four corresponding
controls; their raw failures remain recorded. All 36 earlier unhealthy inventory
rows stay unchanged. Seven historical replay jobs retain exit one; the exact
comparison and classification pass.

Initial failures remain in the evidence: a test binding required mutability; the
page exposed missing Object.hasOwn; and an old aggregate legacy-method count
needed to reflect removal of virtual cloneNode. The page's diagnostic rerun also
failed before the missing API was implemented. A Vulkan Clippy attempt stopped
before compilation on broken temporary Cargo cache links; checksum-verified
locked archives restored those entries. These are not first-attempt clean runs.

Final raw bootstrap measures 4,709 remaining work units, 1,782,581 charged bytes,
690 objects and capacity, 320 native registry entries and 25 legacy prototypes.
The ownership ledger's original conditional sum omitted 20 work units in its
addition; the corrected delta is 1,310 work and 7,517 bytes. The original forecast
and additive correction are retained. Numeric quotas and execution resets did
not change.

Release SHA-256 is
`e77a1efcd0b08a2ac570d65e0d0a4579aa55b51f02a4c99188b6bbbf19e7310d`.
The final 31-path source manifest is
`76b82bdfc67a7bc28f364be0caa9cf13c9f8105a707e873fb95db389df018196`.
The [summary](evidence/node-clone.json) and [archive](evidence/node-clone.tar.gz)
retain source, frozen fixtures, release observations, failed attempts and exact
corrections. These results cover the stated cases and do not establish full web
compatibility, production security or the requested Chromium performance threshold.

The first complete independent result audit passes with 686 bound inputs and no
findings. It reconstructs the raw outcomes, source corrections and all recorded
commands, including failed attempts, and verifies unchanged input hashes.
