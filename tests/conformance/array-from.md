# Array.from checkpoint

Eris now implements generic synchronous `Array.from` over iterable and array-like
inputs. Mapper validation precedes iterator lookup; iterable construction receives
zero arguments and array-like construction receives the captured length. Iteration
caches `next`, reads live result properties, maps with value/index arguments and
creates own indexed data properties. Completion performs a strict length write.
Custom constructors, bound functions, returned objects, aliased inputs/results,
mapped arguments and UTF-16 String iteration remain observable through these paths.

Mapping and output-definition throws close the iterator. Acquisition, stepping,
value reads and the final length write do not. An ordinary pending throw keeps its
identity over ordinary closing failures. Terminal resource/unsupported outcomes
skip author cleanup and preserve earlier effects while restoring internal counters.
This is the engine's bounded-runtime policy, not an ECMAScript exception claim.

The [frozen local fixture](array-from.js) contains 201 sources and 402 sloppy/strict
modes. Its [expectations](array-from-fixture.json) were fixed before implementation
or engine execution. Every source starts with successful array-like and custom
iterable availability checks. Sixteen positive/wrong control pairs run in both
modes; twelve additional controls verify the unchanged Test262 harness. A wrong
assertion only verifies when its same-mode positive partner succeeds.

The [published-before observations](array-from-initial.json), from commit
`cbb970d0e0b117dd50a3861336afd59ac19f9aaa`, retain 398 failures and four parse
unsupported outcomes. None of the 64 feature controls verify; all twelve common
harness controls do. The [final observations](array-from-final.json) retain
**376 passed, ten failed, four unsupported and twelve expected resource stops**.
Thus **388 of 402 expectations verify**, including all twelve resource expectations,
and **all 76 controls verify**. The fourteen unmet ordinary-success expectations
are both modes of seven prerequisites: generators, classes, Set, Map, typed arrays,
Proxy and BigInt. Their expectations and denominator remain unchanged. Missing
BigInt produces a retained harness failure because that source expects TypeError.

The [complete pinned upstream profile](test262-array-from.md) contains 47 sources
and 90 modes, with original helper/legal bytes and root-linked Git proofs. The
final release has **82 passed, four failed and four metadata exclusions**, and
**all 96 controls verify**. It gains 74 raw passes and loses none. Eight earlier
TypeError coincidences remain visible in the before report. The remaining failures
require `splice` or `ArrayBuffer`; generator and cross-realm exclusions are unchanged.
Its known-state baseline preserves those gaps and is not an all-pass conformance gate.

All **38 historical profiles, 17,732 complete case observations and 3,804 controls**
remain identical. All 31 existing regression gates pass and their files remain
unchanged. With the new profile there are 39 profiles, 17,822 modes, 3,900 verified
controls and 32 known-state gates. Seven older profiles remain observation-only.

The [validation record](array-from-integration-validation.json) binds the source
archive, eight release binaries, logs and independent reviews. Rust 1.88 and 1.98
each pass strict all-target Clippy and **1,187 default / 1,198 Vulkan-feature tests**,
with zero failures or ignored tests. All 244 Python tests pass. Both release variants
preserve all 57 CPU pixel references. The page fixture checks six visible outcomes
before and after a click, directly and through the real confined renderer. Both
HTML adapter binaries are byte-identical to the published-before binaries, so no
new HTML replay is assigned. One deterministic mutation-smoke run exercises
15,000 cases without caught panics or invariant failures; seventeen bounded paint
stops remain within the checked invariants.

Seventeen new private groups check ordering, descriptors, callback identity,
actual work/heap cut points, cycles, lazy host boundaries, repeated calls and
terminal cleanup. All 513 script groups pass. An initial private-test compilation
failure used equality on a descriptor enum without PartialEq; only the test
assertions changed, and the failed log is retained. Independent review identified
long-name deduplication work when shrinking a constructor-returned Array. The
new path prepays actual retained-key, dense-storage, deletion and scratch costs
before the shared length operation. Existing [script limits](array-from-limits.json)
are unchanged; logical input length never reserves a proportional output buffer.

The related [shared enumeration accounting gap](array-from-enumeration-followup.json)
remains open for other public callers and is recorded in the security docket.
Host-only prototype/output paths remain explicitly unsupported when reached.
Array.fromAsync and wider language/platform prerequisites remain incomplete.
No GPU execution, Chromium comparison, full web compatibility or security proof
is assigned to this checkpoint. See the [combined evidence](array-from.json).
