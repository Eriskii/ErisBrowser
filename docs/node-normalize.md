# Exact descendant Text normalization

`Node.prototype.normalize()` removes empty descendant Text nodes and merges
each adjacent Text run into its first nonempty node. It returns `undefined` and
creates no nodes. Surviving and detached nodes keep their identities, own
properties and prototypes. Removed nodes retain their original data and remain
in the document arena; their parent becomes null.

This is tree normalization, with exact UTF-16 concatenation. It does not perform
Unicode normalization or collapse whitespace. A high surrogate at the end of
one Text and a low surrogate at the start of the next can form a pair in the
survivor. Unmatched units remain exact. Empty Comments and processing
instructions remain in place and stop a Text run, as do elements. Descendants
inside an intervening element are visited separately in tree order.

## Receivers and ordinary properties

The method is named `normalize`, has length zero and is not a constructor. Its
Node prototype property is writable, enumerable and configurable. Saved calls,
own shadows, prototype replacement and deletion use ordinary property lookup;
deletion does not reveal a hidden native fallback. Forged objects cannot acquire
the authentic Node brand by changing their prototype.

Extra arguments are ignored without coercion. JavaScript still evaluates the
argument expressions before the call. The operation reads internal nodes and
data, bypassing authored `data`, `childNodes` or `normalize` properties, and runs
no author callbacks during traversal and mutation.

| Receiver | Represented behavior |
| --- | --- |
| Document, Element or DocumentFragment | Visit ordinary descendants and normalize each sibling Text run. |
| Text, Comment, processing instruction or doctype | Return `undefined`; the receiver itself is not a descendant. |
| A template Element | Visit its ordinary children, without entering its separate content fragment. |
| A template's content fragment | Visit that fragment's descendants, without following its host association. |

An empty Text receiver is therefore not removed, even when attached beside
other Texts. Detached containers work within their own subtree. Normalizing a
Document does not enter template content through the host. The internal path
also accepts coherent Document-parent Text from the existing Rust host/snapshot
interfaces and preserves the snapshot-admitted 256-edge depth boundary.

## Resource admission and retained effects

Empty descendants are removed as they are reached. For a nonempty run, the
runtime validates and counts all its members, plans and emits one exact
canonical payload, then replaces the survivor's data. Tail detachment is a
separate admitted stage after that replacement. The removal helper validates
the current local links, prepays every detach and suffix shift, then commits
without further allocation or quota checks.

A terminal Resource refusal can leave earlier empty removals and completed
runs in place. It can also leave a merged survivor beside its still-attached
tails, temporarily duplicating those units in a descendant read. A refusal
during final traversal bookkeeping can occur after all tree changes are
complete. There is no rollback. Resource termination retains these effects
and follows the existing terminal-entry behavior; it is not an ordinary
JavaScript exception handled by `catch` or `finally`.

Detached tails release neither arena slots nor their retained payload bytes.
Replacement admission credits only the survivor's old stored payload. Canonical
representation changes can increase the total: a one-byte `A` followed by a
two-byte unmatched high surrogate starts with three stored bytes and ends with
four in the merged Units survivor plus the retained two-byte tail. A split
surrogate pair starts with two bytes in each node and ends with a four-byte
scalar survivor plus the retained two-byte tail.

Even a nonempty singleton uses the full paid canonical copy/replacement path.
It keeps the same identity and units but still needs work and allocation
headroom. Normalization requires no new node slot, so a full arena can succeed
if the other budgets admit the operation. Empty containers and leaf receivers
allocate no traversal scratch.

Traversal uses a bounded depth-first cursor and a per-call map of reached IDs.
Their storage and work, including map comparisons, possible growth and cleanup,
are admitted before use. This avoids scanning unrelated arena entries or
allocating a bitmap for the entire document. Exact run emission uses a borrowed
stream and the existing DOM data builder, without an intermediate JavaScript
string or its smaller length limit. Both success and failure preserve the DOM
helper's absolute spent counters. Scratch and replaced payloads do not refund
the runtime's cumulative allocation charge.

One removed range compacts its suffix once. Separate empty removals and runs
can repeatedly shift the same later siblings; each reached shift is paid. The
complete normalizer is not claimed to run in linear time. Quota admission does
not make the standard BTree allocator fallible or establish recovery from
physical process-memory exhaustion. Public Rust callers can still corrupt
structures outside these checked paths; the local removal helper relies on the
runtime's immediately preceding uniqueness proof.

The 100,000-unit bootstrap and author work limits, cumulative 8 MiB script heap,
100,000-node arena, per-data and retained-DOM limits, execution reset sites and
ERWA transport stay unchanged. Text-only removals do not change selected base
or summary elements, details-group membership or frozen base URLs. Existing
summary NodeId caches remain valid when child indices shift.

## Reflection and initialization

The new operation adds one cached function bag and one ordinary Node prototype
member. The constant initializer now admits the distinct four-entry prototype
and three-entry constructor inputs before adding the same eighteen constants.
Its guards, sorted map construction and order storage account for these actual
shapes. Constructor properties and constant values do not change.

The current represented prototype inventory has 23 own keys: `nodeValue`,
`textContent`, `normalize`, the eighteen constants, `constructor`, then
`Symbol.toStringTag` in `Reflect.ownKeys`. This is the implemented subset, not a
claim of a complete Node interface.

The frozen earlier Node-constant oracle requires the former 22-key inventory.
Adding the method changes two historical case modes and four reflected-order
control modes in the final release comparison. Their original sources,
expectations and full observations remain retained. A negative control whose same-mode
positive partner fails is unhealthy even if its exception matches. The new
inventory has separately authored cases and controls; historical losses are
not relabeled as new passes.

One private historical test uses a fresh disposable realm, removes only the
new operation, and invokes the unchanged old body. Its `finally` restores the
descriptor and value, but cannot restore property creation order. A separate
fresh-realm assertion checks the genuine new inventory. Neither wrapper changes
the external historical expectations.

Measured raw bootstrap leaves **10,199 work units** and charges **1,744,501
bytes**, with **682 objects/capacity**, 321 native entries and 25 legacy prototype
entries. Relative to the preceding Text checkpoint, the new method and adjusted
constant staging consume 437 additional work units and 3,230 charged bytes,
including one new function bag. Numeric quotas and reset sites are unchanged.

## Browser witness and validation

The Page and confined-worker witness loads once and dispatches one
real click. Eight five-Text runs cover visible scalar and nonscalar content,
title, style, a clean textarea, template content, ordinary template children and
a hidden details element. The click retains eight survivors and detaches 32
existing nodes without creating any. Every original ID and payload remains
checked. Ordinary template normalization is checked before the content fragment
is normalized explicitly.

The title changes from `Normalize ready` to `Normalize done`. Separate literal
green/blue pixels and a clean textarea Text draw command check live layout.
A details summary moves from child index five to one with the same NodeId; the
direct Page witness primes that cache before the click. A decoded worker
snapshot's cache check makes no claim about the original worker's cache.

Before merging, the visible surrogate halves use two independent literal
replacement-glyph nodes in the scalar pixel reference. After merging, the
survivor paints the literal `A🚀B`. Both phases require whole-canvas equality
and nonempty glyph bands. Per-node display projection means the
rendered glyphs can change on normalization even though the exact concatenated
units do not. This is not a cross-node shaping equivalence claim.

Rust 1.88 passes **1,775 default tests** and Rust 1.98 passes **1,898 native
Vulkan tests**, including ignored confinement checks. Formatting and strict
native and presenter Clippy pass. There are **42 new groups**: eleven DOM,
twenty-nine runtime (including sixteen independent bodies in both modes), and
two browser integrations. Successful focused commands contain 120 observations
covering 117 distinct groups.

The first all-target compile and all forty new DOM/runtime groups passed. The
initial bootstrap filter recorded 25 passes and two stale descriptive assertion
failures. Measurement justified three literal updates in two private test
files; the original assertions and failures remain retained. The first full
default run then recorded 1,535 passes and one old partial-inventory assertion
failure. That test now explicitly requires `normalize` at index two and the
first constant at index three; its name records the added operation, and all
other descriptor, function identity and realm-isolation checks remain unchanged.
The original test and failed run are retained. The sixteen initially held
source and fixture paths stay byte-identical through these
test-only corrections.

The release build passes. Its complete comparison retains **4,204 established
case records and 444 controls**: 4,202 cases and 440 controls are unchanged.
The two historical full-inventory modes now fail with `Error`, and four old
reflected-order controls produce `TypeError` and remain unhealthy. The raw Node
profile and aggregate replay job retain their nonzero exits. A separately frozen
classifier accepts exactly these six inventory differences; it does not change
the raw observations or waive control health. No other established record changes
or input drift occurred.

All **32 new strict/sloppy Normalize modes pass**, up from zero on the preceding
Text release, and all **16 controls are healthy**, up from four. Positive/wrong
controls require both the expected observation and a healthy same-mode positive
partner. These selected records are not a platform-wide conformance rate.

The [summary](evidence/node-normalize.json) and
[archive](evidence/node-normalize.tar.gz) bind the held sources, fee ledgers,
commands, logs, before/after reports, original failing assertions, corrected tests
and independent review records. The preceding
[Text checkpoint CI receipt](evidence/text-operations-ci.json) records all nine
GitHub jobs passing for `9825a53`; those are preceding-checkpoint results, not CI
results for Normalize. No timing improvement or Chromium comparison is claimed.

This increment does not add CDATASection or XMLDocument construction/parsing,
live Range boundary updates, MutationObserver records, slotting or
custom-element reactions. The standard uses *exclusive Text* for normalization:
future CDATA nodes would be barriers, unlike their participation in `wholeText`.
The current Node model has no CDATA kind. Live layout/title/style refresh is
not a claim of complete synchronous DOM reactions. Dirty textarea behavior,
legacy exact writers and full DOM/interface coverage remain unfinished.

Normative references: [Node.normalize](https://dom.spec.whatwg.org/#dom-node-normalize),
[exclusive Text](https://dom.spec.whatwg.org/#exclusive-text-node),
[replace data](https://dom.spec.whatwg.org/#concept-cd-replace), and
[interface prototype operations](https://webidl.spec.whatwg.org/#es-operations).
