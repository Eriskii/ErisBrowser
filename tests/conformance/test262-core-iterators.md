# Pinned Test262 core iterator inventory

This profile retains **76 unchanged sources / 144 modes** at Test262 revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`. It includes six complete recursive
subtrees under `test/built-ins`:

| Selected subtree | Sources |
| --- | ---: |
| `Array/prototype/values` | 12 |
| `Array/prototype/keys` | 12 |
| `Array/prototype/entries` | 12 |
| `String/prototype/Symbol.iterator` | 6 |
| `ArrayIteratorPrototype` | 27 |
| `StringIteratorPrototype` | 7 |
| Total | **76** |

The [manifest](../upstream/test262-core-iterators/manifest.json) preserves all
87,300 source bytes across nine directories, 76 sloppy and 68 strict modes,
helpers, legal files and fifteen original Git API proof documents. The proof
follows the pinned commit/root to each selected recursive subtree and
reconstructs all descendant directory and blob identities. No failing source
or declared prerequisite is removed.

Admission was frozen before execution as exactly
`CONSTRUCTION_FEATURES | REGEXP_FEATURES | {'Array.prototype.values'}`.
All **38 metadata exclusions** remain unchanged: 18 resizable-arraybuffer and
20 TypedArray modes. All 106 admitted modes execute. Unchanged helpers are
`assert.js`, `sta.js`, `compareArray.js`, `propertyHelper.js`,
`isConstructor.js`, `testTypedArray.js`, `detachArrayBuffer.js` and
`resizableArrayBufferUtils.js`.

## Observations and supported boundary

The [initial report](test262-core-iterators-initial.json) and
[final report](test262-core-iterators-final.json) are exact copies of the
independent reports, retaining every case, exclusion and raw control result.

| Outcome | Before | Final |
| --- | ---: | ---: |
| Raw passed | 8 | 106 |
| Failed | 98 | 0 |
| Metadata unsupported | 38 | 38 |
| Resource, timeout, adapter or harness error | 0 | 0 |
| Verified controls | 32 / 132 | 132 / 132 |

The eight before passes are both modes of four sources:
`entries/return-abrupt-from-this.js`, `keys/return-abrupt-from-this.js`,
`values/this-val-non-obj-coercible.js`, and
`String/prototype/Symbol.iterator/this-val-non-obj-coercible.js`.
Each expects TypeError from calling the method on null or undefined. A missing
method produces an incidental TypeError before the intended receiver check.
The before report's successful iterator guards fail, so those raw outcomes
alone establish no iterator support. The final report adds **98 passes**,
retains all eight prior raw passes and keeps every metadata exclusion.

The implemented scope covers Array values/keys/entries, String iteration over
exact UTF-16 units, the corresponding private-branded `next` methods,
intrinsic aliases/metadata and the shared iterator self method. Array next
uses a saved index with live length and indexed reads; completion permanently
releases its retained source. String iteration combines a valid surrogate
pair and otherwise yields the individual code unit. These behaviors follow
ECMA-262's [Array iterator algorithms](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array-iterator-objects)
and [String iterator algorithms](https://tc39.es/ecma262/multipage/text-processing.html#sec-string-iterator-objects).

The profile does not establish TypedArray/resizable-buffer iteration, generic
Window/DOM/CSSOM property traversal, generator or async iterator support,
iterator helpers, or unrestricted resource capacity. Ordinary supported
property traversal, private state and result allocations use the shared
work/heap/recursion limits. The separate [for-of profile](test262-for-of.md)
retains two missing-Proxy failures and all nine prior incidental parse-negative
passes that become unsupported; they are not hidden by this profile's results.

## Controls, identity and replay

There are **132 controls**: 32 unchanged core controls plus 25 new
positive/deliberately-wrong pairs in both modes. New controls check method
availability and a successful iterator call before error assertions. They
exercise live arrays, holes, keys/entries, length and getter reentrancy,
completion, UTF-16, coercion, private brands and intrinsic metadata. Wrong
partners must produce the unchanged harness's Test262Error and their separate
positive partner must verify. All 100 new controls fail verification before
implementation; all 132 final controls verify. The two new profiles together
verify **212 controls** without changing assertions or expectations.

Before adapter:
`aae61052e935bebf797f05a412d78b1d677b80ff66cd9369d0fbafe332cb8d22`,
from commit `b8a2a988c3da50bd40be34c0cf34cf6c13c00f6e`, input digest
`7a1bcda0e3f121542761245cfc32830edce9959373c93690741506bd96c0ffea`.
Final candidate 2 adapter:
`66bf753b598273161e6cc88f5004d83b4d5d1f88e2919e96db2414ccc2cc1316`;
source-input digest:
`cd50007f53a7b9431c104be33996295c2e4da96440963637f3eef20a02aedfa8`.

| Bound record | SHA-256 |
| --- | --- |
| Initial report | `debdc7eb13c72d8f2986be931eeb4ea41d2f056c22599f731f2f87193fc272ab` |
| Final report | `a7f0c805c28e9650cac52d1193ed5fa1484d97491cfd5696dac415f979966723` |
| Corpus manifest | `44bcb91d37ca69d53440c40f58b12dce05ec979ae64b15fd8c0a575f79230e09` |
| Runner policy | `739e4c14931cd9bba8a6cefcd8dd67ba5d373a5f185022365cb90449e4a1baec` |
| Frozen source/mode/helper case records | `55ba7e3c26cb8735097bc4ce86a8f547ebb9e6e6fc7b46efda843fcd5e8d3915` |
| Frozen control records | `572c4e27caf6536afae88d892018220e9b16426b91f863ae908176e5fb6d01a4` |
| Frozen proof-file records | `58720ad694fbcfc25a4b0d26a9599eaa616344af60bcc9115630306b373a0386` |
| Formal tooling/corpus freeze | `fa3f377113cbae8c01ca5a34db69dc27c9bf9dde2b27afd1be7d865d7cfcb5e0` |
| Shared source freeze | `95645966c407170525a7060e82ae73b0743a6ef20266f1edbab554ac54e4c37e` |
| Before execution ledger | `088582ca325450fcbf11685592f9e012d7c1e1cb702f947814906e14332b2fcc` |
| Final execution ledger | `7b250408602435d390317776ce36cf1a406e5db3382f40d7361989e962a183b1` |

The case/control/proof digests use sorted-key, compact JSON from their frozen
contract fields. Both runs verified the entire shared 879-file ledger,
binary/tool hashes and exact mode/control fingerprints before and after.
Execution used three-second per-case limits, four workers, bounded output and
a 600-second outer profile deadline. There were no retries or normalized
failures. The [for-of provenance](test262-for-of.md) records the shared freeze
and candidate history.

The [current regression baseline](test262-core-iterators-current.json) uses
the runner's exact `current` shape, derived from the final report only after
all controls verified and no `BAD_RUN_STATUSES` remained. `check_baseline`
against that candidate reports zero regressions and zero unrecorded
improvements. It retains all 38 unsupported modes and does not claim full
ECMAScript or host-object compatibility.

```sh
python3 tools/import_test262.py --profile core-iterators
python3 tools/test262_conformance.py --profile core-iterators --baseline tests/conformance/test262-core-iterators-current.json
```

Publication copied existing report bytes and checked the baseline in memory;
it did not execute another adapter. A run without a baseline returns nonzero
for the retained unsupported outcomes.
