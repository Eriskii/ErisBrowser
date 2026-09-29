# Object integrity and frozen fixtures

`Object.seal`, `Object.freeze`, `Object.isSealed` and `Object.isFrozen` now
operate on supported ECMAScript objects, including ordinary objects, arrays,
functions, boxed strings, arguments objects and RegExp instances. Sealing
prevents extensions and makes own properties nonconfigurable; freezing also
makes own data properties nonwritable. Accessor functions remain intact, and
neither operation recursively freezes children or prototypes. Queries inspect
descriptor flags without invoking getters. Non-object inputs are returned
unchanged by the mutators and produce `true` from the queries.

The [complete pinned upstream profile](test262-object-integrity.md) preserves
all 239 sources and 474 modes. The final release records 378 passed, 58
missing-prerequisite failures and 38 unsupported modes, with all 224 controls
verified. Eight older descriptor modes also pass; no previous pass is lost.

## Local fixtures

These independently authored fixtures cover **52 semantic groups** across
`Object.seal`, `Object.freeze`, `Object.isSealed` and `Object.isFrozen`:
**114 source cases / 228 strict and sloppy variants**. Each `// CASE:` block in
[object-integrity.js](object-integrity.js) runs in a fresh runtime with unchanged
pinned Test262 assertion helpers. Twelve separate positive and deliberately
wrong controls check SameValue, intrinsic error identity and property flags.

Sources and expectations were frozen before execution. The published before
release at commit `ff48c9391af1bfd9775b77ccf83047a5abf7856c` records **228 failed
semantic variants**, all runtime `Test262Error` observations at method-availability
guards. All **12 assertion controls** verify. Every semantic case first checks
all four methods and successful seal/freeze/query behavior, before any expected
error assertion. Missing methods therefore cannot produce false passes. These
observations do not validate code beyond the guards. The final release
passes all **228 unchanged semantic variants**, and all **12 assertion
controls** still verify. All 240 semantic/control fingerprints are retained;
these local fixtures are separate from the upstream regression baseline.

The [machine-readable evidence](object-integrity.json) retains the before and
after observations, frozen matrix, exact sources/modes, expectations, helper
hashes and preparation/release provenance. Historical preparation records retain the
then-pending CI status. The later publication checkpoint separately confirms
[CI run 36521631349](https://github.com/Eriskii/ErisBrowser/actions/runs/36521631349)
completed all six jobs successfully; no historical record was rewritten.

The before adapter SHA-256 is
`c5cb23f3fdb36a848937fdb2a8853987a91b2ea874dfec2a15b525b94eec04b4`.
The unchanged fixture SHA-256 is
`9f2b3cd66ab6c834ee9439860e27b34d8c2d541ae6b8c56652744a32f0a27f8b`;
the frozen matrix SHA-256 is
`cd92a13c71da59a8354348d79274217f2ca680b7fc72c2e69b913bf97a3b4ff8`.
The final adapter SHA-256 is
`7f9db9b98e06e2554e05df85a64a98e4ccd1fea8a97d894e8dad67e8500d7341`;
the final source-input digest is
`f0a3ff4e2e3135a2102de21177f7edee1bd3cfde7a01dcecb253daad67e7532e`.

Expectations follow the primary algorithms for
[seal](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.seal),
[freeze](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.freeze),
[isSealed](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.issealed),
[isFrozen](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.isfrozen),
[SetIntegrityLevel](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-setintegritylevel)
and [TestIntegrityLevel](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-testintegritylevel).
Coverage includes primitive identity, descriptors without getter invocation,
symbols and UTF-16 keys, shallow child/prototype behavior, strict/sloppy writes
and deletion, aliases and metadata. Arrays cover sparse u32 lengths, holes,
inherited indices, accessors and the distinction between sealing and readonly
length. Boxed strings retain virtual UTF-16 indices; function and constructor
behavior remains usable. Sloppy mapped arguments retain aliases when sealed
and snapshot/detach them when frozen. Dynamic Function bodies preserve that
sloppy context under either outer mode. RegExp cases distinguish global
`lastIndex` updates from nonglobal execution.

Replay must preserve all 240 semantic/control source, mode and expectation
fingerprints. Original semantic report labels remain `object-integrity.js` and
control labels remain `harness-controls`; the published source path is
`tests/conformance/object-integrity.js`. This display-path mapping does not
change hashed inputs. Case bodies include their original trailing newlines.
Strictness remains an adapter mode, not a source rewrite.

## Bounds and remaining scope

Integrity operations use a dedicated snapshot of own keys. Scanning, key
materialization, vector storage and sorting are charged before allocation
against the caller's existing work and cumulative heap allowances. Array
logical gaps do not create keys. Queries read only configurable/writable flags;
they do not fetch mapped argument values or allocate virtual boxed-string
character values. Mutators skip compatible flags that are already unchanged,
avoiding redundant descriptor records for virtual string indices.

Changed properties use the shared definition path one at a time. Freezing
mapped arguments snapshots their current values and detaches the affected
aliases; sealing retains the mappings. Binding names are borrowed, and work
charges include their actual length. Nonextensibility is applied before the
key snapshot, so it remains visible if later allocation or work fails.
Earlier successful property changes also remain visible after a later
failure. The operation is not an atomic rollback transaction. Quotas and
uncatchable resource-error behavior are unchanged.

The bounded runner uses three seconds per mode, 512 MiB address space,
30 CPU seconds, disabled core files, bounded output and a 45-second outer
deadline. Proxy, Window/Document/DOM hosts, typed arrays, BigInt and cross-realm
behavior are outside this inventory. The implementation explicitly rejects
unsupported host integrity operations, including Window/global-object queries;
it does not infer integrity from partial host reflection. Resource thresholds
and partial-failure oracles are covered by **15 private test groups**. They
check primitive and extensible-query fast paths, snapshot refusal after
nonextensibility, ordered partial array/argument updates, actual mapped-name
work, virtual-string flags, sparse logical lengths, host refusal before
sidecar mutation and terminal shared-budget cleanup.

Rust **1.88 and 1.95** pass strict all-target Clippy and **1,071 default / 1,082
Vulkan-feature tests**, with none ignored. All **201 Python tests**, both
release configurations' **57 CPU pixel references**, unchanged HTML observations
and **15,000 mutation cases** pass. The combined upstream comparison retains
**34 profiles / 14,778 modes / 2,964 verified controls** and **27 healthy
regression gates**. Remaining failed and unsupported observations stay visible.
These observations do not establish a native/GPU result, production security
or a Chromium performance comparison.
