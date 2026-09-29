# Pinned Test262 Array.prototype.splice inventory

This profile retains every recursive JavaScript descendant of
[`Array/prototype/splice`](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/Array/prototype/splice)
at Test262 revision `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`:
**81 sources / 162 modes**, one complete directory, **112,801 source bytes**.
Every source runs in both sloppy and strict mode. There are no metadata
negatives, async/module flags, nested source directories or fixture files.
The [manifest](../upstream/test262-array-splice/manifest.json) preserves source,
metadata, helper and Git-object identities. Eight retained proof documents
bind the complete selection to the pinned root tree and splice subtree
`79383dccad0703220657c1592f4e5688f51f711c`.

The six unchanged helpers are `assert.js`, `sta.js`, `compareArray.js`,
`propertyHelper.js`, `isConstructor.js` and `proxyTrapsHelper.js`; original
[LICENSE](../upstream/test262-array-splice/LICENSE) and
[INTERPRETING.md](../upstream/test262-array-splice/INTERPRETING.md) are retained.
Selection does not remove sources because of anticipated failures.
The policy is exactly the previously frozen `ARRAY_FROM_FEATURES` set.
It admits **138 modes** and retains **24 metadata exclusions**:
12 exponentiation, eight Proxy and four cross-realm modes. One exponentiation
source also uses untagged Proxy and Reflect.get; admitting its syntax alone
would not establish its other prerequisites.

| Observation | Published before | Splice candidate |
| --- | ---: | ---: |
| Passed | 12 | 138 |
| Failed | 126 | 0 |
| Metadata excluded | 24 | 24 |
| Resource / timeout / adapter error | 0 | 0 |
| Verified controls | 32 / 96 | 96 / 96 |

The [initial report](test262-array-splice-initial.json) and
[final report](test262-array-splice-final.json) are exact copies of the frozen
execution outputs. **126 modes gain a pass**, with no lost pass; all twelve
old raw passes and all 24 exclusion records are unchanged. The old passes are
six exception-only sources in both modes: two readonly-length tests,
`create-ctor-non-object.js`, `create-species-non-ctor.js`, and the two
`target-array-*` descriptor-refusal tests. Calling the absent method already
produced their expected TypeError. Those raw passes did not establish splice
support, and the before feature preflight was unhealthy.

The **96 controls** comprise 32 unchanged common controls plus sixteen
independently authored positive/deliberately-wrong pairs in both modes.
Each new control first completes Array and generic-object splice calls.
Wrong partners require the unchanged harness's Test262Error and a successful
same-mode positive partner. Sources, modes, helpers, expectations and policy
were frozen before either run and remain identical.

The [local and runtime scope](array-splice.md) separately retains all
**48 local sources / 96 modes**: 86 ordinary passes, four expected terminal
resource stops, four missing-Proxy/Uint8Array failures and two source-gated
cross-realm cases. Thus **90/96 local expectations** verify, alongside all
**76 local controls**. These resources are engine-policy controls, not
ECMAScript errors or additional semantic passes. Neither this profile nor the
implementation claims Proxy, typed-array or cross-realm support.

Before adapter SHA-256: `908f215d69c86911d7629892ff6f65ed275dc3236075840e55076bcce42c270d`, preserved from published
`605725f6df10be6a346de15c220fcf7e3e6f6aed` (the preceding graphics change did not
alter the browser runtime). Candidate adapter SHA-256: `f9ffdf6172908c875308ee40fe5783d3b673bbdccf7685a61c608b331348f721`.
Exact candidate production input: `835dcf31a48ea8acc2a1074a833ea812ed4517397029ad4d5f46276d110723db`.

| Binding | SHA-256 |
| --- | --- |
| Initial report | `dd6afd381b7ed591485cae204fea84130cbf1e71001ea65fcd039f4342d32c2e` |
| Final report | `9d6db75b098d1713b54691ec29bd5dd1c0d844e5e7ea762cc9c0fffad549155a` |
| Current baseline | `f101d6ecf1d515dfb4f1e37a8cab6b96818a46f09aff2039ddd4dae26e06eec1` |
| Corpus manifest | `104cb0d7553df8c317eaad5e5d561fdc86e6dad2cf337f93d947818a0ea35026` |
| Runner policy | `3ffb7cd01ba9148d4ef61806a0d476c682ebbb9f9dce37190188a963006a73da` |
| Formal source/control freeze | `2632c143deda9fdbabfb971ae95ccf60e6dea9cd443effb2eba1db292ce7d38c` |
| Complete before/candidate comparison | `20300f903017d9e6c76f7548f2b8a02bf53d1dfafded03df10e9e91db5dbe404` |

The [current baseline](test262-array-splice-current.json) is the runner's exact
compact projection of the final report. It stores all 162 case identities and
statuses; full control observations remain in the final report. Admission
requires all controls to verify and no resource, timeout or adapter errors.
Its in-memory gate has zero regressions and zero unrecorded improvements.
This is a healthy regression gate with 24 explicit exclusions, not full
Test262 conformance. Running without a baseline returns nonzero while those
excluded modes remain.

All 39 historical profiles were replayed before baseline changes:
**17,822 cases / 3,900 controls**, zero lost passes or other observation changes,
and all **32 existing gates** passed. The only ten gains are eight
`array-altered-during-loop.js` modes in the four find directories and two
Array.from `elements-deleted-after.js` modes. Only those two healthy current
baselines are strengthened; their historical reports and the other thirty
baselines stay unchanged. The seven observation-only profiles receive no
baseline. No quota, corpus, feature policy or external expectation changed.

```sh
python3 tools/import_test262.py --profile array-splice
python3 tools/test262_conformance.py --profile array-splice --baseline tests/conformance/test262-array-splice-current.json
```
