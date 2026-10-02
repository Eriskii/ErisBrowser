# Array concat checkpoint

`Array.prototype.concat` creates its same-realm species result before inspecting
the receiver's spreadability. Arrays consult their live constructor and
`Symbol.species`; generic non-Array receivers skip those hooks. The constructor
receives one zero argument. Reassigning the global Array binding does not replace
the intrinsic fallback. The method processes the boxed receiver followed by its
arguments. Primitive arguments remain individual values. An object's
`Symbol.isConcatSpreadable` uses Boolean conversion; only undefined falls back
to its Array brand. Truthy object flags do not invoke conversion hooks.

Each spread item captures its length when reached and visits indices in
ascending order through separate live HasProperty and Get operations. Present
values become own writable, enumerable, configurable data properties, bypassing
inherited setters. Holes advance the destination without deleting preexisting
result keys. The final strict length Set can invoke a setter or shrink an
existing Array. An ordinary result retains keys beyond its assigned length.
Earlier definitions and author effects survive later failures.

Species results may be ordinary objects, Arrays, mapped arguments or an input
itself. If `a = [1]`, `b = [2,3]` and a's species returns b, `a.concat(b)` produces
that same b as `[1,1,1]`: earlier writes change the later input's live values.
No iterator acquisition or closing occurs. Explicitly spread boxed Strings
expose UTF-16 code units; primitive String arguments remain single values.
Other Array methods' species behavior is unchanged.

The scope follows the primary [concat and IsConcatSpreadable algorithms](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.concat),
[ArraySpeciesCreate](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-arrayspeciescreate)
and [ArraySetLength](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-arraysetlength).
Lengths and counters retain safe-integer semantics through 2^53−1. Overflow
follows the item's length conversion and precedes its indexed reads. Concat
starts its output index at zero and visits every logical hole: a public >2^32
hole walk reaches the existing work limit before the normative final Array
length RangeError. That local expectation describes engine policy. Private
seeded-counter tests cover the late u32 boundary; a separate public partial-shrink
case checks strict final Array length refusal and prior descending deletions.

The [frozen fixture](array-concat-fixture.json), [source](array-concat.js) and
[matrix](array-concat-matrix.json) retain **45 sources / 90 sloppy and strict
modes**. Every semantic source and feature control first proves successful Array
and generic-object calls. Fourteen [positive/wrong feature pairs](array-concat-preflights.json)
in both modes give **56 controls**, alongside **twelve unchanged common controls**.
A wrong assertion verifies only when its same-mode positive partner succeeds.
The original Test262 assertion helpers and all 158 source/mode/expectation
identities are unchanged. The [local preparation record](array-concat-local-preparation.json)
binds the independent reviews, helper hashes, runner and both executions.

The [before report](array-concat-initial.json), from published commit
`dffca3d69c39de5c58e55838918c0b704b275d5d`, records **88 availability failures and
two cross-realm host-hook exclusions**. No fixture or feature control verifies;
the twelve common controls do. The [final report](array-concat-final.json)
retains **80 passed, four failed, two unsupported and four expected resource
stops**: **84 of 90 expectations and all 68 controls verify**. Proxy, Uint8Array
and cross-realm sources keep their six unmet ordinary-success expectations.
The retained runner's conservative `$262` check excludes cross-realm sources
before adapter execution. Recursive species and the >2^32 hole walk account
for the four expected terminal resources; these are not ECMAScript exceptions.
The [complete comparison](array-concat-local-comparison.json) preserves every
changed observation, all 80 raw pass gains and zero lost passes.

The [complete pinned upstream profile](test262-array-concat.md) preserves all
**69 sources / 137 modes**, original helpers/legal bytes and authenticated
[manifest proofs](../upstream/test262-array-concat/manifest.json). It records
**113 passed, four failed, eighteen unsupported and two resource outcomes**,
with **all 96 controls verified**. The failures require Uint8Array; sixteen
metadata exclusions remain fixed, and two untagged class modes remain admitted
and unsupported. Both modes of an ordinary 4,000-hole test exhaust work. Fourteen
incidental before TypeError passes remain visible; 99 modes gain passes and none
lose one. Resource remains a bad-run status under the unchanged runner, so this
profile receives **no baseline or CI gate**.

All **40 historical profiles / 17,984 cases / 3,996 controls** remain exactly
unchanged; all **33 established gates** pass. Six selected older local suites
preserve **1,288 observations and 248 controls**. The combined inventory has
**41 profiles / 18,121 modes / 4,092 controls**, still with 33 known-state gates.
No unsupported source, resource outcome, quota or metadata rule was removed to
obtain these results.

The [validation](array-concat-validation.json), [integration](array-concat-integration-validation.json)
and [combined evidence](array-concat.json) retain exact source/binary identities,
logs and reviews. Rust **1.88 and 1.98** each pass strict all-target Clippy and
**1,243 default / 1,254 Vulkan-feature tests**; all **thirteen private concat
groups** and **264 Python tests** pass. Both release configurations preserve all
**57 CPU pixel references**. Loading and clicking the page fixture pass directly
and through the actual confined worker. Byte-identical HTML adapters and 68
unchanged inputs justify reusing the preceding **3,868 matched / two mismatched /
six unsupported** report; no fresh HTML run is claimed. One mutation-smoke run
covers **15,000 cases**, with zero caught panics or invariant failures and
seventeen bounded paint stops. All seven [remote CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36570342566)
passed for published commit `f65c666`.

The method borrows arguments and keeps constant-size loop state rather than
reserving its logical range. Each reached item and index, including holes,
consumes shared work. Reused scoped helpers charge tree searches, borrowed mapped
names, numeric keys, storage growth and Array shrink before the corresponding
work. Private tests cover lookup and heap cutpoints, prior effects, cycles,
recursion, cumulative budgets, lazy host refusal and cleanup. All [quotas](array-concat-limits.json)
remain unchanged. Terminal limits and reached unsupported errors preserve prior
effects, bypass author catch/finally and unwind internal frames.

These scoped charges do not establish general constructor/callback/property/JSON
accounting, allocator recovery or total process-memory bounds. Proxy, typed-array,
cross-realm and broader host/exotic semantics remain incomplete. The optional
Vulkan upload presenter and standalone opaque rectangle/image probe are unchanged.
This checkpoint makes no new GPU, performance, full-compatibility or
production-security claim.
