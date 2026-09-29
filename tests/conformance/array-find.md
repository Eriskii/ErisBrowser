# Array find methods

`Array.prototype.find`, `findIndex`, `findLast` and `findLastIndex` now use live
property reads on supported ECMAScript receivers. Length is converted once
before callback validation; even empty input requires a callable predicate.
Ascending or descending traversal visits every index in that captured range,
including holes and deleted entries. Inherited properties and getters remain
observable at the time of each visit. The callback receives the read value,
index and original coerced object, with the supplied `thisArg` and ordinary
strict/sloppy call semantics. Truthy results stop traversal; value-returning
methods return the value saved before the callback, even if it replaces or
deletes that property. No match returns `undefined` or index `-1`.

These semantics follow
[find](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.find),
[findIndex](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.findindex),
[findLast](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.findlast),
[findLastIndex](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.findlastindex)
and [FindViaPredicate](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-findviapredicate).
The methods stream their logical indices without materializing an index list
or result array. Callback truthiness uses `ToBoolean` without author conversion
hooks. Constructor, species and iterator hooks are not consulted.

## Independent coverage

The frozen [fixture](array-find.js) contains 63 semantic groups across the four
methods: 252 source cases and **504 strict/sloppy modes**. Every source first
checks all four methods and makes successful value, index and empty-input
calls. Twelve separate paired controls check SameValue, error identity and
descriptor assertions. All **504 final release variants pass** and all **12 controls
verify**. The [evidence record](array-find.json) retains every source, mode,
helper, expectation and runner fingerprint. Fixture SHA-256:
`fdf171bc4d2b5d2bcf795a2fbb691db93a2405f3de8fac6ea9ebe69343e1b974`;
frozen matrix SHA-256:
`8d32016f2d666559d560fcc5fe7f71f8c23aa874454581721c0bc8ff26b808fd`.

Before adapter `7f9db9b9…`, from published commit `9bb0684…`, records 504 runtime
Test262Error failures and verifies the twelve independent controls. These are
guarded missing-method observations, not 504 distinct algorithm defects. Its
generic thrown-object messages do not independently identify assertion sites.
All 44 published release files were hash-verified; the original CI-in-progress
record and later confirmation of six successful jobs remain separately recorded.

Cases include generic and boxed receivers, UTF-16 string units, sparse arrays,
lookup order, inherited accessors, callback mutation, abrupt identity and saved
value identity. Large logical lengths return or throw on their first visit,
including maximum-safe-integer generic lengths and maximum-u32 sparse arrays.
They do not require scanning the full range or allocating absent elements.
Unchanged Test262 `assert.js`, `sta.js` and `propertyHelper.js` at revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd` provide the assertions.

The separate [complete upstream profile](test262-array-find.md) retains
**94 sources / 180 modes**. Its final release records **140 passed, eight
failed and 32 metadata unsupported**, with all **288 controls verified**. It
gains 116 passes; the other 64 complete records remain identical. Eight failures
now reach missing `splice` in the first callback of four unchanged upstream
sources. Their generic TypeError records alone do not reveal that changed
execution path. Every older profile observation is unchanged, and the new
recorded baseline passes its CLI gate while retaining all failures and
unsupported cases.

## Resource boundaries and remaining scope

Every logical visit and prototype edge consumes shared work. Decimal index
keys, property scratch and the three callback arguments are precharged before
allocation or invocation. The shared property walker now charges comparisons
against the relevant retained property, parameter and hole trees. A mapped
argument read borrows its retained binding name and charges its actual length
and environment lookup bound before retrieving the value. It avoids a fresh
long-name copy. Values are owned before invoking getters or predicates; those
callbacks do not run while retaining mutable property-storage borrows.

The first release candidate caused work-limit regressions in both modes of
an older lastIndexOf source. Index formatting charged sixteen digits even for
short keys. It now precharges one unit plus the actual decimal digit count and
fills precisely that many stack slots, retaining the 64-byte allocation charge
and all stronger property-lookup charges. Both modes pass again. The evidence
preserves the initial binary, source and full observations alongside the
corrected result; no quota or corpus contract changed.

Work and allocation remain cumulative through recursive getters, callbacks and
repeated calls. Earlier author side effects remain visible when a later quota
check fails. Resource exhaustion remains uncatchable and preserves the existing
terminal cleanup behavior. The 100,000-step and 8 MiB cumulative heap limits,
32 logical calls, 96 weighted stack units and 96-edge prototype bound are
unchanged. A huge logical length is supported for early return; a full scan
still consumes those budgets.

All thirteen private test groups pass, covering the frozen semantic fixture,
exact refusal, prior getter effects, callback storage, mapped names, comparison
work, strings, host boundaries and cleanup. Together with 62 older array groups,
all **75 focused groups** pass. One initial native-callback test setup looked up
Boolean incorrectly; the corrected test retrieves the installed intrinsic.
Its earlier failure is retained, and no production code or frozen semantic
fixture changed to fit that setup.

Rust 1.88 and 1.95 pass strict all-target Clippy and **1,084 default / 1,095
Vulkan-feature tests**, none ignored. All **207 Python tests** and both releases' **57 CPU pixel references** pass;
HTML observations are unchanged. The final
**15,000-case** deterministic mutation smoke reports no caught panic or invariant
failure. Source, binary and log hashes are in the evidence. Bounded independent
source review found no concrete defect; this is not an allocation-containment
proof.

Proxy, typed arrays, BigInt, cross-realm behavior, general host receivers,
async predicates and full `with`/unscopables behavior remain outside this slice.
A supported ordinary object with all needed own properties need not touch its
host prototype; an actual lookup reaching an unsupported host is refused.
This is not full ECMAScript or web compatibility, a security certification or a
Chromium performance comparison. Vulkan rendering scope is unchanged.
