# ArrayBuffer checkpoint

The runtime implements nonshared fixed and resizable `ArrayBuffer` objects,
the `byteLength`, `maxByteLength`, `resizable` and `detached` getters, `resize`,
same-realm species-aware `slice`, `transfer` and `transferToFixedLength`.
Backing storage has a private brand; inherited properties, prototype changes
and ordinary lookalikes do not acquire it. Buffers retain ordinary own-property
and integrity behavior. Freezing a buffer object does not freeze its internal
storage. `ArrayBuffer.isView` returns false for the currently supported value
set without invoking author hooks: no genuine typed-array or DataView type is
installed yet.

Constructor length and options conversions precede prototype lookup. Resize
rejects fixed buffers before converting its argument; a detached resizable
buffer still performs that conversion before detached rejection. Transfer
converts an explicit new length before checking detachment, uses the saved
intrinsic constructor without consulting source constructor/species, and
detaches only after successful allocation and copying. Earlier callback effects
survive later refusal. Resizable transfer retains the original maximum;
`transferToFixedLength` removes resizability and may exceed that old maximum
within the storage limits.

Slice captures bounds from the initial length, constructs and validates its
species result, then rechecks source state. A callback may shrink the source,
reducing the copied prefix without reducing the requested result length.
An existing larger result keeps its untouched suffix. Results must be genuine,
attached, distinct buffers of sufficient length, even for an empty slice.
These rules follow the primary [ArrayBuffer algorithms](https://tc39.es/ecma262/multipage/structured-data.html#sec-arraybuffer-objects),
[ArrayBufferCopyAndDetach](https://tc39.es/ecma262/multipage/structured-data.html#sec-arraybuffercopyanddetach)
and [SpeciesConstructor](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-speciesconstructor).

The independently reviewed [source](array-buffer.js), [matrix](array-buffer-matrix.json)
and [preparation record](array-buffer-local-preparation.json) preserve **52 sources
/ 104 strict and sloppy modes**: 90 ordinary semantic modes, ten prerequisite
modes and four explicit terminal-policy modes. Sixteen
[positive/wrong feature pairs](array-buffer-preflights.json) add 64 controls;
twelve unchanged common controls make **180 total rows**. Each source first
guards ArrayBuffer availability and a genuine instance. Negative method cases
establish successful behavior first. A wrong feature control verifies only
after its same-mode positive partner succeeds. Original Test262 helpers and
all source/mode/expectation fingerprints are unchanged.

The [before report](array-buffer-initial.json) retains **102 failed and two
unsupported modes**, no raw fixture passes, and twelve verified common controls.
At the original ArrayBuffer checkpoint, the [candidate report](array-buffer-final.json) records **88 passed, eight failed,
two unsupported and six resource stops**: **92 of 104 expectations and all 76
controls verify**. Both modes of `getter-method-flags-and-nonconstructability`
unexpectedly reach the instruction limit; their ordinary-success expectations
were unmet at that checkpoint. Four other resource outcomes match the frozen recursive-conversion
and repeated-allocation policy cases. They are terminal engine limits, not
ECMAScript exceptions. The eight failures require Uint8Array, DataView,
SharedArrayBuffer or Proxy. The unchanged conservative `$262` source check
rejects the two foreign-realm modes before adapter execution. All ten
prerequisite expectations remain ordinary success. The
[complete comparison](array-buffer-comparison.json) retains **88 raw pass gains,
zero losses** and every changed observation, with no retries or oracle changes.

The later [for-in length-bucket follow-up](for-in-length-buckets.md) replays these
same sources, helpers and expectations. Both ordinary metadata modes now pass:
**90 passed, eight failed, two unsupported and four resource outcomes**, with
**94 of 104 expectations and all 76 controls verified**. The four expected
terminal-resource outcomes and ten prerequisite gaps are unchanged. The original
reports above remain intact.

Supported buffer-only JavaScript and [Page/worker fixtures](array-buffer-integration-fixture.json)
check metadata, conversion order, callback effects, detachment and ordinary
properties; they do not inspect backing bytes. The retained TypedArray/DataView
prerequisite sources do attempt genuine byte reads. Separately, **24 private
Rust groups** check independent [seeded byte expectations](array-buffer-backing-oracle.json),
zero initialization, shrink/regrow zeroing, copy independence, untouched species
suffixes, both backing-record orders and callbacks that grow the record table.
Measured one-short work/heap tests preserve prior author effects while refusing
the pending allocation or copy. Deterministic quota tests do not simulate every
host allocator failure.

All [resource limits](array-buffer-limits.json) remain unchanged. Byte allocation,
zeroing, copying, record lookup/growth and relevant callback work consume shared
budgets. Shrink, detach and replacement do not refund cumulative charges.
An initial byte length above the existing absolute **8 MiB** capacity ceiling
produces catchable RangeError at byte-block creation, after prototype lookup and
ordinary object creation, before nonexistent allocation/zeroing charges.
The separate late resizable-maximum feasibility check follows initial block
creation; merely storing a maximum does not allocate that many bytes.
These explicit [host capacity policies](array-buffer-capacity-policy.json) are
distinct from normative ToIndex/range errors and remaining cumulative work/heap
shortages, which stay terminal. The [capacity oracle](array-buffer-capacity-oracle.json)
and private tests cover ordering and transfer atomicity; they do not imply that
every request below the absolute ceiling fits the remaining budget.

The [complete pinned upstream profile](test262-array-buffer.md) retains all
**221 sources / 442 modes** and records **262 passed, 50 failed and 130 excluded**,
with **all 160 controls verified and no resources**. The failures are 48
Uint8Array prerequisite modes and two untagged BigInt parse failures; exclusions
are 112 declared-feature and eighteen host-hook source-check modes. Its separate
[known-state gate](test262-array-buffer-baseline-validation.json) preserves those
failures and exclusions. This resource-free formal run does not erase the two
unexpected local work stops.

The **41 older profiles / 18,121 modes / 4,092 controls** retain exactly four
pass gains, in Array.from and Object.seal ArrayBuffer cases, with no other case
or control changes. Seven older local suites retain **all 1,378 fixture modes
and 316 controls unchanged**. The catalog now has **42 profiles / 18,563 modes /
4,252 controls** and **34 checked known-state gates**; only the two affected old
baselines were strengthened. No corpus, expectation, metadata rule or quota
was relaxed to obtain these results.

The [validation record](array-buffer-validation.json) binds Rust **1.88 and 1.98**,
strict Clippy, **1,269 default / 1,280 Vulkan-feature tests**, **274 Python groups**,
and **57 CPU pixel references per release configuration**. Loading and clicking
the fixture pass directly and through the actual confined worker. The retained
[Date-host follow-up](array-buffer-date-host-followup.json) records an initial
feature-test failure, a traced run with a different failure and the successful
full untraced rerun; the first attempts are not discarded. The first ArrayBuffer
candidate's Clippy-only range-assertion correction is also retained.
One mutation-smoke run covers **15,000 cases**, with zero panics/invariant failures
and seventeen bounded paint stops. Byte-identical HTML adapters and 68 unchanged
inputs justify [reusing](array-buffer-html-reuse.json) the preceding **3,868 matched
/ two mismatched / six unsupported** report; no fresh HTML run is claimed.
All seven [remote CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36959177431)
passed for published commit `9a7b8dc`.

Shared buffers, genuine views, Proxy, foreign realms, host detachment hooks and
immutable-buffer proposals remain separate gaps. Scoped backing-store accounting
does not establish general property/JSON accounting, total process-memory bounds,
allocator recovery or browser security. Browser painting remains on the CPU with
the optional upload presenter. The separate Vulkan probe's source-alpha rectangle
and nearest-image evidence is unchanged; this checkpoint adds no GPU execution,
browser GPU rasterization, performance comparison or full-compatibility claim.
