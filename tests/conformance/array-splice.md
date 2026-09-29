# Array splice checkpoint

Eris implements generic `Array.prototype.splice` with captured length, ordered
argument conversion and live property operations. Deleted positions use separate
HasProperty and Get operations, preserving holes and inherited values. Present
values become own data properties on the deleted result, bypassing inherited
setters. Its strict length write precedes the shift, tail-deletion and item-write
phases; earlier result definitions can already change an aliased source.
Left shifts ascend,
right shifts descend, and excess tail properties are deleted from the highest
index downward. Supplied items are written in order before the final strict
source length write. Earlier effects survive a later exception.

The implementation follows the primary [splice algorithm](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.splice)
and same-realm [ArraySpeciesCreate](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-arrayspeciescreate).
Arrays consult their live constructor and `Symbol.species`; non-Array generic
receivers skip these hooks. Ordinary, bound and supported native constructors
receive one length argument. Their result may be the source itself, another
Array, mapped arguments or another supported object. Getters and setters keep
the original receiver, and no arena borrow crosses author execution. Reassigning
the global Array binding does not replace the intrinsic fallback. The existing
map/filter/slice species behavior is unchanged.

Generic lengths and indices use safe integers through 2^53−1. A default deleted
Array exceeding 2^32−1 is rejected before allocation. Actual Array growth can
write the ordinary key `4294967295` before the final length write throws
RangeError. Sparse logical lengths do not allocate their absent elements.
Splice does not acquire iterators or invoke IteratorClose.

The [frozen local expectations](array-splice-fixture.json) retain 48 sources and
96 sloppy/strict modes. Every semantic source first proves successful ordinary
Array and generic-object calls. Sixteen positive/wrong feature pairs in both
modes give 64 controls, alongside twelve unchanged harness controls. A wrong
assertion verifies only when its same-mode positive partner succeeds.

The [published-before report](array-splice-initial.json), from commit
`605725f6df10be6a346de15c220fcf7e3e6f6aed`, records 94 availability failures and
two cross-realm host-hook unsupported outcomes. No local expectation or feature
control verifies; the twelve common controls do. The [final report](array-splice-final.json)
retains **86 passed, four failed, two unsupported and four expected resource
stops**: **90 of 96 expectations and all 76 controls verify**. The six unmet
ordinary-success expectations are both modes of Proxy, typed-array and
cross-realm prerequisites. Their expectations and denominator remain unchanged.
The four resource expectations concern the engine's terminal-budget policy,
not standard ECMAScript exceptions.

The [complete pinned upstream profile](test262-array-splice.md) retains all
81 sources and 162 modes, unchanged harness/legal bytes and root-linked Git
proofs. It records **138 passed and 24 metadata exclusions**, with **all 96
controls verified**. Its unchanged policy excludes exponentiation, Proxy and
cross-realm metadata; no source is selected by its result. The before report's
twelve incidental TypeError passes remain recorded, while unsuccessful method
guards prevent them from establishing implementation health. The
[known-state baseline](test262-array-splice-current.json) preserves the complete
selection and exclusions.

Across all **39 historical profiles, 17,822 modes and 3,900 controls**, exactly
ten modes gain passes: eight find-family callback cases and two Array.from
live-iteration cases. No prior pass is lost; every other complete observation
and every control remains identical. All 32 established gates pass. The new splice
baseline and strengthened find/Array.from baselines bring the configured catalog
to **33 known-state gates**. All 33 final baseline projections check against the
completed reports with zero regressions or improvements. These offline checks
performed no engine rerun and do not claim remote CI execution. Five selected
older local suites preserve another **1,192 case observations and 172 controls**.
The new profile brings the inventory to 40 profiles, 17,984 modes and 3,996 controls.

The [integration validation](array-splice-integration-validation.json) binds the
source archive, binaries, logs and reviews. Rust **1.88 and 1.98** each pass strict
all-target Clippy and **1,228 default / 1,239 Vulkan-feature tests**, with no failed
or ignored tests. All 18 new private groups pass. The tooling checks comprise
212 Test262 Python tests on the exact imported bytes and 42 remaining Python
tests. Both release configurations pass **57 CPU pixel references**. The page
fixture checks six outcomes during loading and clicking, plus result text,
directly and through the real confined renderer. HTML retains **3,868 matched,
two mismatched and six unsupported modes**, without baseline regressions. One
mutation-smoke run checks **15,000 generated cases**, with zero caught panics or
invariant failures and seventeen bounded paint stops.

The method borrows its supplied items and uses constant-size loop state. Each
reached iteration consumes the shared work ledger. Scoped lookup charges derive
from B=6 tree bounds, including string/symbol/native trees, parameter maps,
hole sets and borrowed mapped-name environment searches. Dense and creation-order
vector growth prepays movement and the full new buffer; short numeric keys bound
deletion comparisons without rescanning entire retained names. Writable Array
shrink reuses charged own-key enumeration and adds its remaining deletion and
order-retention costs. Private tests cover heap/work cutpoints, prior mutation
prefixes, vector relocation, mapped names, cycles, sparse storage and cleanup.
All [script quotas](array-splice-limits.json) remain unchanged.

Resource and reached unsupported errors preserve completed effects, bypass
author catch/finally and unwind internal call/frame state. These scoped logical
charges do not settle general constructor/callback/property/JSON accounting,
allocator fallibility or total process memory. Proxy, typed-array, cross-realm
species and general host-object behavior remain incomplete. This checkpoint
adds no GPU execution or performance comparison and establishes neither full
web compatibility nor production security. See the [combined evidence](array-splice.json).
