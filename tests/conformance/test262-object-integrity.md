# Pinned Test262 Object integrity inventory

The [DataView follow-up](data-view.md) closes both `seal-dataview.js` modes:
this unchanged profile now records **398 passed, 38 failed and 38 unsupported**,
with all 224 controls verified. The ArrayBuffer and Date checkpoints previously
recorded 396 and 394 passes respectively. Earlier measurements and prerequisite descriptions below
are retained as historical evidence; the Date comparison is at the end.

This profile retains the four complete direct directories below at Test262
revision `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`. The
[manifest](../upstream/test262-object-integrity/manifest.json) records every
unchanged source and helper hash, plus nine complete Git-tree proof documents
that bind the inventory to the pinned commit. Selection does not depend on
anticipated passes or available prerequisites.

| Directory | Sources | Sloppy modes | Strict modes | Total modes |
| --- | ---: | ---: | ---: | ---: |
| [Object/seal](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/Object/seal) | 94 | 93 | 93 | 186 |
| [Object/freeze](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/Object/freeze) | 53 | 52 | 52 | 104 |
| [Object/isSealed](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/Object/isSealed) | 33 | 33 | 33 | 66 |
| [Object/isFrozen](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/Object/isFrozen) | 59 | 59 | 59 | 118 |
| Total | **239** | **237** | **237** | **474** |

Two sources declare `noStrict` and two declare `onlyStrict`. There are no
frontmatter negative tests, fixtures, nested directories or non-JavaScript
entries in these selections. The 157,276 source bytes remain unchanged.
Upstream `assert.js`, `sta.js`, `propertyHelper.js`, `compareArray.js`,
`isConstructor.js` and `resizableArrayBufferUtils.js` are retained unchanged,
alongside [LICENSE](../upstream/test262-object-integrity/LICENSE) and
[INTERPRETING.md](../upstream/test262-object-integrity/INTERPRETING.md).
Manifest SHA-256:
`602023dfe820d591c4a8fc8d83a9e4680012f4b15083f855f84da427539c988d`.

The primary algorithms are
[Object.seal](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.seal),
[Object.freeze](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.freeze),
[Object.isSealed](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.issealed)
and [Object.isFrozen](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.isfrozen),
using [SetIntegrityLevel](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-setintegritylevel)
and [TestIntegrityLevel](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-testintegritylevel).
[Implementation scope and independent local fixtures](object-integrity.md)
are recorded separately.

## Results and retained dependencies

The before adapter comes from published commit
`ff48c9391af1bfd9775b77ccf83047a5abf7856c`, SHA-256
`c5cb23f3fdb36a848937fdb2a8853987a91b2ea874dfec2a15b525b94eec04b4`.
The final release uses adapter SHA-256
`7f9db9b98e06e2554e05df85a64a98e4ccd1fea8a97d894e8dad67e8500d7341`
and source-input digest
`f0a3ff4e2e3135a2102de21177f7edee1bd3cfde7a01dcecb253daad67e7532e`.
The [complete evidence](object-integrity.json) retains observations,
comparisons, fingerprints and provenance.

| Outcome | Before implementation | Final release |
| --- | ---: | ---: |
| Passed | 0 | 378 |
| Failed | 440 | 58 |
| Unsupported by metadata | 26 | 26 |
| Unsupported parser syntax | 8 | 8 |
| Unsupported host integrity queries | 0 | 4 |
| Resource limit, timeout or adapter error | 0 | 0 |

The final release adds **378 passes**; four former failures now explicitly
report unsupported host integrity, and the other **92 complete observations**
are unchanged. All 474 source, mode and ordered-harness fingerprints and the
feature policy are preserved. The before run has no incidental raw passes,
and its positive method controls fail, so it is not a healthy implementation
baseline.

All **58 remaining failures** are runtime `ReferenceError` observations from
missing prerequisites: Date accounts for 16 modes, untagged Proxy uses for six,
typed-array constructors for 22, ArrayBuffer/DataView for four,
Map/Set/WeakMap/WeakSet for eight and Promise for two. These sources remain
executed failures. They are not converted into metadata exclusions or treated
as integrity-method defects.

The profile uses exactly the existing `CONSTRUCTION_FEATURES | REGEXP_FEATURES`
policy. Its **26 metadata exclusions** retain Proxy (16 modes),
resizable-arraybuffer (two), AggregateError (two), FinalizationRegistry (two),
SharedArrayBuffer (two) and WeakRef (two). Eight metadata-admitted modes reach
unsupported async-arrow, async-function or generator syntax during parsing.
The four newly reached host refusals are both modes of
`Object/isFrozen/15.2.3.12-3-1.js` and
`Object/isSealed/15.2.3.11-4-1.js`, which query the global object. None is filtered
from the inventory.

## Harness integrity and replay

All **224 controls** verify on the final release: 32 unchanged core checks
plus twelve positive/deliberately wrong pairs for each method in both modes.
Every method-specific source proves callability and successful invocation
before expected-error assertions. Coverage includes primitives, descriptor
flags, accessor preservation, arrays and holes, symbols, inherited properties,
shallow behavior, mapped/unmapped arguments and native metadata. Metadata
helpers use `restore: true`. The wrong partner changes only the final assertion
and must produce the unchanged harness's `Test262Error`; unrelated intrinsic
exceptions do not verify a negative control. Before implementation only 128
controls verify, and all 96 new positive controls fail their availability
guard, so the overall preflight fails.

The [shared runner contract](test262.md) retains fresh runtimes, unchanged
harnesses, bounded execution and separate failure/unsupported/resource rows.
Offline Python tests verify complete Git proofs, reject omissions and
same-count replacements, reproduce the corpus and compare every older
contract. All **33 prior profiles / 14,304 modes / 2,740 controls** retain their
source, policy, exclusion and generated-control fingerprints. Eight older
array-descriptor modes also gain passes: both modes of
`Object/defineProperty/15.2.3.6-4-604.js`, `15.2.3.6-4-605.js`,
`15.2.3.6-4-607.js` and `15.2.3.6-4-608.js`. The other **14,296 older complete
observations** and all older controls are identical. No previous pass is lost.

```sh
python3 tools/import_test262.py --profile object-integrity
python3 tools/test262_conformance.py --profile object-integrity --baseline tests/conformance/test262-object-integrity-current.json
```

The final replay has no resource, timeout, adapter or harness errors and
satisfies all controls. Its [recorded baseline](test262-object-integrity-current.json)
passes the CLI gate; the descriptor baseline also retains its eight new passes.
The combined inventory now has **34 profiles / 14,778 modes / 2,964 verified
controls** and **27 healthy regression gates**. The runner's healthy status
requires valid controls and bounded execution, not every corpus case passing:
all 58 failures and 38 unsupported observations remain in the baseline and full
report. Neither this profile nor the local fixtures establish complete
ECMAScript or host-object conformance.

### Final Date comparison

Final frozen adapter `ab40a96f2cc1908359186e7c0648cecddc5ad8f8b2adf8a60de0e0d2c9a8f220`
records **394 passed, 42 failed, 38 unsupported**: **16 new passes and zero lost passes**
against the published before release. Complete observations and controls match
the first frozen candidate; all original source/helper/mode/policy identities
remain unchanged. The existing known-state baseline was strengthened to protect these new passes.
The [complete comparison](test262-date-profiles-comparison.json) and
[baseline receipt](test262-date-baseline-recording.json) retain exact hashes.
Original before and first-candidate reports remain preserved in the
[Date evidence](date.json). No script quota was increased.
