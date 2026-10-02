# DataView: independent tests before implementation

This checkpoint adds authenticated upstream tests and independent local byte and
ordering oracles. It does **not** add DataView runtime support. The browser goal
remains incomplete; these results make no security or Chromium performance claim.

The intended implementation covers a nonshared DataView constructor, `buffer`,
`byteOffset`, `byteLength`, authentic `ArrayBuffer.isView`, and nine Number getter
and setter pairs: Int8, Uint8, Int16, Uint16, Int32, Uint32, Float16, Float32 and
Float64. Tests cover fixed and tracking views, resize and transfer, conversion
and prototype callback ordering, byte order and direct binary64-to-binary16
rounding. BigInt codecs, shared buffers, typed arrays, Proxy and foreign realms
remain prerequisites to complete compatibility.

The complete pinned Test262 DataView subtree is retained: **561 sources, 1,122
modes, 27 directories and seven unchanged helpers**. Original Git proofs link it
to revision `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`. The new profile admits 706
modes and records 416 metadata or host-hook exclusions. No old profile policy,
helper, expectation, resource limit or baseline was changed.

`Float16Array` is the pinned corpus's umbrella feature tag for its DataView
Float16 method tests. Admission here does not claim a Float16Array constructor or
Math.f16round implementation. Six BigInt-method sources lack the BigInt tag and
remain admitted: four resize tests and two nonconstructability tests. An absent
method can incidentally satisfy the latter; their raw outcomes cannot establish
BigInt support.

The [formal before report](test262-data-view-before.json) uses preserved adapter
`650c0860141ec576ddbad348af53fc125b6399d9087f8ae9c9b20992aa5975e7`, built from the
runtime published in commit `0668e83832df232e46cc427aec18d6f28333e61c`. It records
**706 failed and 416 unsupported modes**. All 32 common harness controls verify;
none of the 200 DataView-specific controls verify. Those controls comprise 50
positive/wrong pairs in both modes, with successful byte operations required
before negative assertions. Their unhealthy baseline is retained as observed.

The [independent local fixture](data-view-local.js) contains 69 sources in both
modes: 124 scoped ordinary-completion cases, ten missing-prerequisite cases with
ordinary-completion expectations, and four explicit engine resource-policy
cases. The [paired controls](data-view-local-preflights.json) add 80 feature rows;
the runner also retains its 12 common controls. The [codec literals](data-view-codec-vectors.tsv)
contain 100 independent encode/decode rows and 19 decode-only rows. They include
42 previously frozen Float16 inputs checked by an exact rational reference.
Public NaN tests require valid classification and stable encoding; they do not
require an arbitrary payload or sign.

The [local before report](data-view-local-before.json) records **136 failed and
two unsupported case modes**, with zero fixture expectations verified. All 12
common controls verify; none of the 80 feature controls verify. Both baseline
process groups were cleaned up before their leaders were reaped, and all input
bindings remained unchanged. The 136 local failures comprise 134 runtime guard
failures and two BigInt parse failures; the two unsupported rows require host
hooks. The 706 formal failures comprise 658 runtime ReferenceErrors, 40 runtime
Test262Errors and eight BigInt parse failures. Neither run has a raw pass or
resource observation.

All 367 Python tooling tests pass. Static comparisons preserve all 42 earlier
profiles' 18,563 mode contracts and 4,252 control contracts byte for byte. No new
regression baseline was written. This preparation did not rerun Rust or GPU tests.
The parent commit's [nine-job CI receipt](../../docs/evidence/for-in-length-buckets-ci.json)
is retained separately.

Review corrected two negative-test guards that could accept a missing method as
the expected TypeError, and one authored Float16 byte literal. Initial Python
preparation failures involved a mechanically renamed helper path and incomplete
expectations for existing Float16 metadata exclusions. Original drafts and failed
logs are retained. No runtime observation was used to change an oracle.

The [preparation evidence inventory](../../docs/evidence/data-view-preparation/index.json)
binds inputs, reviews, drivers and complete before observations. Its compressed
archive contains source and records, not executable builds. The prior adapter's
source inventory and publication audit remain in the
[for-in checkpoint](for-in-length-buckets.md).

To reproduce the formal selection with an available Eris adapter:

```sh
python3 tools/test262_conformance.py --profile data-view --binary /path/to/eris-js --output /tmp/data-view.json
python3 -m unittest discover -s tools -p 'test_test262_data_view.py'
```

The runner retains every case, exclusion and failure. A nonzero exit is expected
for this before binary. The standard-derived scope follows the
[DataView algorithms](https://tc39.es/ecma262/multipage/structured-data.html#sec-dataview-objects).
