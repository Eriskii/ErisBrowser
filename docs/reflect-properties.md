# Reflect property operations

Eris implements nine additional Reflect methods: `has`, `get`, `set`,
`deleteProperty`, `getOwnPropertyDescriptor`, `getPrototypeOf`, `setPrototypeOf`,
`isExtensible` and `preventExtensions`. They cover represented ordinary objects,
Arrays, functions, mapped arguments, boxed strings and Number TypedArrays.
Existing Reflect operations retain their earlier behavior and corpus.

Targets must be objects, and target validation precedes observable property-key
conversion. `get` and `set` preserve the original receiver through prototype
lookup and accessor calls. Omitting a receiver differs from supplying
`undefined` or a primitive. An ordinary writable target property combined with
a receiver's own accessor returns `false` without calling that receiver's
setter. An accessor found on the target instead receives the supplied receiver.
Exceptions from author callbacks retain their identity and are not converted
into Boolean refusal.

Descriptors retain their flags, mapped arguments retain parameter aliases, and
boxed-string indices expose individual UTF-16 units. Array length reduction
can return `false` after the deletions required before a nonconfigurable element
is encountered. Prototype changes reject cycles, and the intrinsic
`Object.prototype` remains immutable even if the global `Object` name changes.
An unchanged prototype remains an allowed no-op.

TypedArray behavior uses the existing live buffer and index machinery. A write
with the same TypedArray receiver converts its value before checking the
current bounds. A conversion callback can resize or detach the buffer. Distinct
target/receiver paths retain their different conversion and property-definition
rules. Invalid canonical numeric indices do not fall through to inherited
properties. Resizable views can refuse `preventExtensions` with `false`.

Represented DOM expandos and installed prototype descriptors are supported.
Unresolved legacy DOM string properties explicitly report `Unsupported`; the
implementation does not infer absence or create a shadow property for those
unknown host behaviors. Style, class-list and console string reflection,
unimplemented host prototype/extensibility operations, Proxy and foreign
realms remain outside this increment. These boundaries are narrower than full
platform-object reflection. The selected language behavior follows the
[ECMAScript Reflect algorithms](https://tc39.es/ecma262/multipage/reflection.html#sec-reflect-object).

## Resource accounting

Property searches, prototype walks, callback entry, descriptor results and
metadata storage retain explicit logical resource charges. Refusal preserves
completed author callbacks and any partial mutation required by the operation.
Private tests check exact and one-short admissions, unpaid mutation boundaries,
and cleanup after saved-native calls. These local checks do not establish
general security or performance guarantees.

The prerequisite ArrayBuffer metadata installer uses a bounded, prepaid
construction path while preserving its public identities and property order.
It reduces startup charges by 3,489 work units and 5,711 bytes. The separate
Reflect installer consumes 3,278 work units, 25,704 bytes at its installation
sites and 1,944 bytes for the additional object arena slots.

The measured complete-bootstrap diagnostic is **496 work units remaining,
1,872,268 charged bytes, and 730 objects with capacity 730**. The native registry
has 320 entries and the legacy prototype table has 25. The 496 figure is the
remaining initialization allowance, not the amount of work performed. Public
execution limits are unchanged; the normal author-work allowance is assigned
only after initialization succeeds. Descriptive startup snapshots were updated
from this measurement, with their original bytes retained.

## Recorded validation

The [local report](../tests/conformance/reflect-properties-local-current.json)
passes all **28 case modes and 20 paired-control modes**. Its 14 independently
authored bodies run in sloppy and strict mode and must return `true`. Wrong
controls require intrinsic `Error` identity and a successful positive partner
in the same mode; an absent method's `TypeError` cannot count as healthy coverage.
The unchanged TypedArray local corpus still passes 54 cases and 32 controls.

The [upstream report](../tests/conformance/test262-reflect-properties-report.json)
records **200 passed modes, 16 explicitly unsupported Proxy modes, and all 52
verified harness controls**. Its complete selected population is 108 original
files, each in two modes, at Test262 revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`. It includes every file in the nine
method directories and the three direct Reflect object tests, with their
original helpers. The [preparation record](reflect-properties-test262.md)
documents this selection and the retained before reports. The published
predecessor had 46 passed, 154 failed and 16 unsupported modes, but only 36 of
52 healthy controls; those apparent passes did not establish method support.
The final [condensed baseline](../tests/conformance/test262-reflect-properties-current.json)
is separate from both complete reports.

All **15 Reflect private tests** and **seven ArrayBuffer installer tests** pass
on Rust 1.88 and 1.98. The Rust 1.88 suite with `vulkan-raster` and
`--include-ignored` passes **2,227 tests**, including the Page and confined-worker
witnesses. The first full-suite attempt is retained: 1,893 library tests passed
and two confinement tests failed when the outer execution sandbox denied their
TCP socket preflight. The identical source passed after the authorized rerun
outside that outer sandbox; the failed attempt is not represented as a product
fix. Strict Clippy passes on both Rust versions, and the final formatting check
passes. All 276 Test262 protocol tests pass; the preparation evidence retains
the initial three registration-test failures and their narrow profile-exclusion
corrections.

Across **37 existing profiles, all 16,136 mode observations remain exactly
equal to the published predecessor**, including the existing 52 failures and
3,488 unsupported observations. There are no new gains in this historical
comparison. Earlier ArrayBuffer and object-integrity gains belong to the
TypedArray foundation checkpoint.

The separate [TypedArray report after Reflect](../tests/conformance/test262-typedarray-after-reflect.json)
retains the complete 1,238-mode population and all 64 healthy controls:

| Outcome | TypedArray foundation | After Reflect |
| --- | ---: | ---: |
| Passed | 572 | 588 |
| Failed | 86 | 44 |
| Unsupported | 488 | 488 |
| Resource limit | 92 | 118 |

Sixteen former failures now pass. Another 26 advance beyond an absent Reflect
method and reach the unchanged work limit; these are resource stops, not passes.
Two `HasProperty/inherited-property` modes remain failed with changed diagnostics:
they now reach an assertion requiring the still-absent `subarray` property.
That dependency is identified by reading the retained test source. Thus 42
statuses and 44 complete raw rows change. All 572 previous passed rows and all
92 previous resource rows remain exact, with no prior-pass regression. Other
TypedArray API and dependency gaps remain visible in the report.

The [browser fixture](../tests/conformance/reflect-properties-browser/reflect-properties.html)
retains saved Reflect methods and a live resizable view across a real hit/click.
Both Page and confined-worker tests check all nine methods, receiver refusal,
resize callbacks, literal displayed results and the entire canvas against
script-free references. Seven captured node identities and the complete
parent/child graph remain stable while the marker changes from green to blue.

CI adds a hash-bound local runner gate and the upstream Reflect baseline gate.
Existing Rust discovery includes the private and browser groups, including
ignored confinement tests, and Python discovery includes the protocol tests.
The preparation commit `8570143` passed all nine jobs in
[run 37708412764](https://github.com/Eriskii/ErisBrowser/actions/runs/37708412764).
That is preparation CI; it is not a claim that the later implementation commit
has completed CI.

The [evidence summary](evidence/reflect-properties.json) and
[retained evidence archive](evidence/reflect-properties.tar.gz) bind the source,
adapter, reports, reviews and failure history. The validated adapter SHA-256 is
`c21074ad37ed121547e6386dd529c67c6c438a6a0b2a6583d453118c49ef78e8`;
it is the candidate adopted unchanged as the final executable. This selected
coverage does not establish full ECMAScript or web compatibility, production
security, or a Chromium performance result.
