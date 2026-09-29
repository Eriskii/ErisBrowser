# Pinned Test262 Array find inventory

This profile retains four complete direct directories at Test262 revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`. The
[manifest](../upstream/test262-array-find/manifest.json) records the unchanged
sources, metadata and helper hashes. Ten complete Git-tree proof documents
bind all selected filenames and source blobs to the pinned commit. The separate
preparation also retains a root-linked harness-tree proof. Selection does not
filter sources by expected success or missing dependencies.

| Directory | Sources | Sloppy modes | Strict modes | Total modes |
| --- | ---: | ---: | ---: | ---: |
| [Array/prototype/find](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/Array/prototype/find) | 23 | 22 | 22 | 44 |
| [Array/prototype/findIndex](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/Array/prototype/findIndex) | 23 | 22 | 22 | 44 |
| [Array/prototype/findLast](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/Array/prototype/findLast) | 24 | 23 | 23 | 46 |
| [Array/prototype/findLastIndex](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/Array/prototype/findLastIndex) | 24 | 23 | 23 | 46 |
| Total | **94** | **90** | **90** | **180** |

Each directory has one `noStrict` and one `onlyStrict` source. There are no
frontmatter negatives, fixtures, nested directories or non-JavaScript entries
in this selection. All **111,737 source bytes** are retained. Unchanged helpers
are `assert.js`, `sta.js`, `compareArray.js`, `propertyHelper.js`,
`isConstructor.js`, `testTypedArray.js` and `resizableArrayBufferUtils.js`, with
[LICENSE](../upstream/test262-array-find/LICENSE) and
[INTERPRETING.md](../upstream/test262-array-find/INTERPRETING.md).
Manifest SHA-256:
`09fba3be727b98e9fcdc54343abd7725f1392244ff1bdec1c0afc3e30b5c0eba`.
The ordered source/mode/helper identity digest is
`fcfbb3348fd6f25f52219d46886e4e58b5c8e654f8bffbb963f4ef5e1a82ff24`.

The policy is exactly `ARRAY_PREDICATE_FEATURES | {'array-find-from-last'}`,
approved before either adapter run. It preserves **32 metadata exclusions**:
eight modes declare both TypedArray and resizable-arraybuffer; another 24
declare resizable-arraybuffer alone. All 76 modes declaring
`array-find-from-last` retain that metadata. No prerequisite inspection changes
admission. The [implementation and local fixture](array-find.md) cover the
supported receiver boundary separately.

## Final results and dependencies

The before release is published commit
`9bb0684af279f2f886b3b7aeef32eb4bbf11c055`, adapter SHA-256
`7f9db9b98e06e2554e05df85a64a98e4ccd1fea8a97d894e8dad67e8500d7341`.
Its formal run exactly matches all 180 frozen diagnostic records. The corrected
final adapter SHA-256 is
`68a36c3e0d2641c63c90d72a5596e503fa19ad30e2e751888113f4c28b58d977`,
with source-input digest
`aaaa136be0837ebe81c7c9edeb3a73270de2ae4188e3efc9a73a1483f042eeb5`.
The [complete evidence](array-find.json) binds final observations and retained
initial-candidate records to their respective binaries and sources.

| Outcome | Before implementation | Final release |
| --- | ---: | ---: |
| Passed | 24 | 140 |
| Failed | 124 | 8 |
| Metadata unsupported | 32 | 32 |
| Resource, timeout, adapter or harness error | 0 | 0 |

The final release adds **116 passes** and preserves **64 other complete records**,
including all former passes. The 24 before passes are incidental TypeErrors
from `predicate-is-not-callable-throws.js`,
`return-abrupt-from-this-length-as-symbol.js` and `return-abrupt-from-this.js`
in all four directories and both modes. An absent method already throws on
these paths; their raw outcomes alone do not establish correct behavior.

All eight remaining failures are both modes of
`array-altered-during-loop.js` in the four selected directories. Each source's
first callback calls missing `Array.prototype.splice`. The recorded runtime
TypeError, `value is not callable`, is identical before and after this change.
The callback's newly reached missing dependency follows from source inspection
and the implemented call path; the diagnostic text alone does not identify the
call site. All eight remain executed failures, with original sources and
expected outcomes intact.

The first release candidate regressed both modes of
`Array/prototype/lastIndexOf/15.4.4.15-3-16.js` to work-limit stops. Stronger
property lookup charges combined with a fixed sixteen-digit formatting charge
over 2,574 keys exhausted the existing budget. Decimal key construction now
precharges one unit plus the actual one-to-sixteen digit count, then fills
exactly that many stack slots. Its 64-byte allocation charge, stronger lookup
charges and all quotas remain unchanged. Both modes pass again. The initial
binary, source, full reports and exact regressions are retained in the evidence;
no source, policy, expectation or preflight was changed to recover them.

## Harness integrity and replay

The profile has **288 controls**: 32 unchanged core controls plus sixteen
positive/deliberately wrong pairs per method in both modes. Each new source
checks method availability and successful true/false predicate calls before
any expected-error assertion. The final release verifies every control. Before
implementation all 128 new positive controls fail their availability guard;
160 controls verify, but the whole preflight fails. No healthy before baseline
is claimed.

Pairs check holes and inherited values, both traversal directions, callback
arguments and receivers, captured length and saved values, live mutations and
getters, Boolean short-circuiting, abrupt identity, generic strings, unused
constructor/species access and intrinsic metadata. Metadata helpers restore
each descriptor. Wrong partners change only the final expected value and must
produce the unchanged harness's Test262Error; an unrelated TypeError does not
verify the mismatch control. A bounded independent static review found no
setup or expectation defect.

All **34 prior profiles / 14,778 modes / 2,964 controls** retain exactly their
source, ordered-helper, mode, metadata, policy, exclusion and generated-control
contracts. Every older complete case observation and assertion-control result
is also preserved by the corrected release; no previous pass is lost. Six new
Python test groups cover complete proof replay, proof and
blob corruption, offline import reproduction, guarded pairs, false-positive
controls and the complete historical contract digest. The full **207-test
Python suite** passes. The [shared runner](test262.md) preserves bounded fresh
adapter execution and separate failure/unsupported/resource observations.

```sh
python3 tools/import_test262.py --profile array-find
python3 tools/test262_conformance.py --profile array-find --baseline tests/conformance/test262-array-find-current.json
```

The [recorded baseline](test262-array-find-current.json) passes the CLI gate.
Its healthy status requires verified controls and no resource, timeout, adapter
or harness errors; it preserves all eight failures and 32 unsupported modes.
Running without a baseline still returns nonzero for those retained outcomes.
The combined inventory now has **35 profiles / 14,958 modes / 3,252 verified
controls** and **28 healthy regression gates**. This does not establish full
ECMAScript or host-object conformance.
