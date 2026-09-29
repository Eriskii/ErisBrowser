# Pinned Test262 for-of inventory

This profile retains the complete recursive
[`test/language/statements/for-of` subtree](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/language/statements/for-of)
at revision `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`: **751 unchanged
sources / 1,442 modes**, comprising 182 direct sources and 569 under `dstr`.
The modes are 729 sloppy, 711 strict and two module cases. The
[manifest](../upstream/test262-for-of/manifest.json) retains all 1,146,879
source bytes, helpers, legal files and seven original Git API proof documents.
The proof follows the pinned commit/root to the recursive subtree and
reconstructs its complete descendant directory set and blob identities.
Sources are not selected by expected success.

Admission was frozen before either adapter ran as exactly
`CONSTRUCTION_FEATURES | REGEXP_FEATURES | {'for-of', 'let', 'const'}`.
All **1,253 metadata exclusions** remain unchanged; 189 modes are admitted.
Excluded features include destructuring, generators, async iteration, modules,
TypedArray, Map, Set, Proxy and other declared unavailable prerequisites.
Untagged prerequisites stay admitted. The unchanged helpers are `assert.js`,
`sta.js`, `compareArray.js`, `propertyHelper.js`, `asyncHelpers.js`,
`doneprintHandle.js` and `resizableArrayBufferUtils.js`.

## Observations and retained gaps

The [initial report](test262-for-of-initial.json) and
[final report](test262-for-of-final.json) are exact copies of the independently
executed reports, including every case, exclusion and raw assertion-control
observation.

| Outcome | Before | Final |
| --- | ---: | ---: |
| Raw passed | 49 | 137 |
| Failed | 99 | 2 |
| Metadata unsupported | 1,253 | 1,253 |
| Runtime/parser unsupported | 41 | 50 |
| Resource, timeout, adapter or harness error | 0 | 0 |
| Verified controls | 32 / 80 | 80 / 80 |

All 49 before passes are expected parse-SyntaxError observations from a parser
without for-of support. The successful protocol controls fail in that report,
so those raw negative passes do not establish implementation conformance.
The final report has **97 failed-to-passed gains and nine passed-to-unsupported
changes**. All nine are retained explicitly:

| Source within `statements/for-of` | Modes lost from raw passed | Final diagnostic |
| --- | --- | --- |
| `decl-fun.js` | sloppy | Legacy conditional function declarations unsupported |
| `labelled-fn-stmt-const.js` | sloppy | Same legacy declaration boundary |
| `labelled-fn-stmt-let.js` | sloppy | Same legacy declaration boundary |
| `labelled-fn-stmt-lhs.js` | sloppy | Same legacy declaration boundary |
| `labelled-fn-stmt-var.js` | sloppy | Same legacy declaration boundary |
| `head-lhs-invalid-asnmt-ptrn-ary.js` | sloppy, strict | Destructuring for-of targets unsupported |
| `head-lhs-invalid-asnmt-ptrn-obj.js` | sloppy, strict | Destructuring for-of targets unsupported |

These sources expect SyntaxError. Their final unsupported classifications are
a remaining early-error limitation, not passes. Complete validation of
unavailable grammar is not claimed. The 50 final runtime/parser unsupported
modes comprise 12 dynamic-eval, 29 destructuring/rest-binding, five legacy
function-declaration and four destructuring-target observations.

Both final failures are `iterator-next-result-type.js`, in sloppy and strict
modes. Its untagged `Proxy` dependency produces the recorded ReferenceError
`'Proxy' is not defined`. The source and admission policy remain unchanged;
the failures are neither filtered nor reclassified.

The supported iteration and completion behavior is specified by ECMA-262's
[for-in/for-of statement algorithms](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-for-in-and-for-of-statements)
and [IteratorClose](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-iteratorclose).
This bounded implementation does not establish destructuring, generators,
async iteration, dynamic eval, module or generic host-iterator compatibility.
Shared compiler, callback, work, heap and nesting limits remain in force.

## Controls, identity and replay

The profile has 32 unchanged core controls and **48 frozen new controls**:
12 positive/deliberately-wrong pairs in both modes. Each new source first
requires a successful custom iterator protocol. A wrong partner must produce
the unchanged harness's Test262Error and its separately executed positive
partner must verify. Incidental parse errors, missing methods or matching
errors in two failed prerequisites cannot establish health. All 80 final
controls verify; together with the [core iterator profile](test262-core-iterators.md),
all **212 controls** verify. No assertions or expectations were rewritten.

Before adapter: `aae61052e935bebf797f05a412d78b1d677b80ff66cd9369d0fbafe332cb8d22`,
from browser commit `b8a2a988c3da50bd40be34c0cf34cf6c13c00f6e`, production-input
digest `7a1bcda0e3f121542761245cfc32830edce9959373c93690741506bd96c0ffea`.
Final candidate 2 adapter:
`66bf753b598273161e6cc88f5004d83b4d5d1f88e2919e96db2414ccc2cc1316`;
source-input digest:
`cd50007f53a7b9431c104be33996295c2e4da96440963637f3eef20a02aedfa8`.
Candidate 1 stopped in validation before a release/profile execution; its
unrelated Date pipe test failure is not silently replaced by this record.

| Bound record | SHA-256 |
| --- | --- |
| Initial report | `da71b637a3f8ec236f0140a91b9ed725d3403b7f8195e651203131dca756e331` |
| Final report | `9f425f02cde9f56975c0e70809fc4c7318cdfe31e2d7ef882040ef3d76326ea7` |
| Corpus manifest | `c1eea71c09bc3fdc4028c28f90916e20bf7e8d0fb9c1cb834b1ce3dd545c7496` |
| Runner policy | `a8ca04723e940f044d57036a0935292efe7a8e53916e6dd71d7c0aabc2f468f4` |
| Frozen source/mode/helper case records | `fe0f7abc3a405c9f517541e0a0759011fc4aebc3b5273c57dd47726b86a07aac` |
| Frozen control records | `5a6cc8173141a00d5754a11710c5cdabc358263ac5b6117d1aea396df66b0f53` |
| Frozen proof-file records | `ea06675729fc248a35d63dd543fb4da923b2a977ed99a7a7439d12647dc9cf7f` |
| Formal tooling/corpus freeze | `fa3f377113cbae8c01ca5a34db69dc27c9bf9dde2b27afd1be7d865d7cfcb5e0` |
| Shared source freeze | `95645966c407170525a7060e82ae73b0743a6ef20266f1edbab554ac54e4c37e` |
| Before execution ledger | `088582ca325450fcbf11685592f9e012d7c1e1cb702f947814906e14332b2fcc` |
| Final execution ledger | `7b250408602435d390317776ce36cf1a406e5db3382f40d7361989e962a183b1` |

The case/control/proof record digests use sorted-key, compact JSON from the
frozen contracts. The shared 879-file ledger was verified before and after
execution, as were the binary, importer, runner and exact mode/control
fingerprints. Cases used the unchanged runner's three-second deadline and
bounded output capture, with four workers; each profile had a 600-second
outer deadline. No run was retried or failure normalized.

The [current regression baseline](test262-for-of-current.json) is derived
from the final report using the runner's exact `current` shape. All preflights
verify and no `BAD_RUN_STATUSES` are present. `check_baseline` against that
candidate reports zero regressions and zero unrecorded improvements. The
baseline preserves both failures, all unsupported modes and the nine raw
before-pass losses; it is not a full-conformance claim.

```sh
python3 tools/import_test262.py --profile for-of
python3 tools/test262_conformance.py --profile for-of --baseline tests/conformance/test262-for-of-current.json
```

These are replay commands; publication copied the existing reports and checked
the baseline in memory without another adapter execution. Running without a
baseline returns nonzero for the retained failed/unsupported outcomes.
