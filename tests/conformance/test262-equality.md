# Pinned Test262 equality inventory

The [constructor-policy follow-up](constructor-policy.md) admits Reflect call/construct
and `new.target` metadata. Current results: **190 passed, 96 unsupported**.
The inventory and source bytes are unchanged. Measurements and policy descriptions
below retain the history of earlier checkpoints.

This profile retains every direct JavaScript file in four complete, flat
[expression directories](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/language/expressions)
at revision `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`:

| Directory | Sources | Source bytes |
| --- | ---: | ---: |
| equals | 47 | 65,289 |
| does-not-equals | 38 | 55,431 |
| strict-equals | 30 | 43,835 |
| strict-does-not-equals | 30 | 43,595 |
| Total | **145** | **208,150** |

There are **286 mode cases**: 145 sloppy and 141 strict. Four sources are
noStrict; none have negative metadata. Original LICENSE, INTERPRETING.md,
sources and four unchanged helpers (assert, sta, propertyHelper and compareArray)
total **253,846 bytes**. The latter two helpers support core assertion controls.
The [manifest](../upstream/test262-equality/manifest.json) SHA-256 is
`b1d07bdfaee72e164608d7ed3351ab70d05931ffda34f174f6c3f99bf12a0046`.

The unchanged core feature policy retains **80 metadata-unsupported modes**:
56 require BigInt, eight BigInt plus Symbol, 12 Symbol.toPrimitive and four Symbol.
Sources, expectations and metadata are identical before and after implementation.

The 32 core assertion controls are joined by six positive/wrong pairs for each
operator in both modes: **128 preflights**. Every added control begins with a
boxed-number loose-equality guard, including strict-operator controls. Pairs
cover the type lattice, live hooks, exact abrupt identity, skipped conversions,
UTF-16, ordinary identity, numeric boundaries and evaluation order.
Wrong partners change only the final assertion and require actual Test262Error;
parser failure cannot satisfy a wrong-answer control.

Six Python groups check inventory/helper integrity, feature admission, guarded
pairs, assertion identity, corrupt/incomplete imports and prior contracts.
All eighteen earlier profiles retain **6,468 case and 1,472 preflight
fingerprints**, manifests, fixtures and feature sets. Replay against the actual
runner at `9adf86a` confirms equality; canonical digest:
`bd5b82e4d44252d4a1e287117c32805b2cb6b810162f6645c3ebcf405a1d2a42`.

| Outcome | Before 9adf86a | Ordinary equality conversion |
| --- | ---: | ---: |
| Passed | 146 | **190** |
| Failed | 44 | **0** |
| Unsupported | 96 | **96** |
| Resource, harness error, timeout or adapter error | 0 | **0** |
| Verified preflights | 80/128 | **128/128** |

The [initial report](test262-equality-initial.json) preserves every observation,
including failed positive controls. The
[candidate report](test262-equality-latest.json) adds **44 passes without losses**:
20 equals and 24 does-not-equals modes. Object-to-primitive, boxed-value and
Boolean/nullish cases now pass. Strict-operator observations remain unchanged.
Alongside the 80 metadata modes, 16 variants reach unsupported dynamic eval.
These original cases remain in the inventory.

Actual baseline recording and a subsequent gate return zero and reproduce every
candidate observation. The [current baseline](test262-equality-current.json)
retains all 96 unsupported results while protecting passing cases. A plain run
returns one; the CI gate does not imply full equality conformance.

All observations and policies in the eighteen earlier profiles are identical.
Existing resource stops in URI, numeric parsing, identifiers, functions and sort
remain explicit; none are converted into healthy baselines. HTML remains at
3,868 matches, two mismatches and six unsupported modes.

```sh
python3 tools/import_test262.py --profile equality
python3 tools/test262_conformance.py --profile equality
python3 tools/test262_conformance.py --profile equality --baseline tests/conformance/test262-equality-current.json
```

| Artifact | SHA-256 |
| --- | --- |
| Before adapter | `a1697ac1b5fa8c70b6c8dc2990bc473bcd5680e892b6f9cfbdc8fc5788740157` |
| Candidate adapter | `5663f74effc73282db269e77aeede8ece73795c324713a4646a2e4c73df0dfb8` |
| Candidate source input | `816e4c1ee47a2096de279c92b7ac0b7b55ddf14283ad0590ee92d1141e70f2f4` |
| Initial report | `2f296df558c95025428f06e1d3bd788609e91bd1edc59003a3fcc0d8890dda24` |
| Candidate report | `d70a92a2866b90d9ea5de98604d169757de2b3a081b04cd1735693aeeee626af` |
| Current baseline | `91e65f1532487a2b2d552e4c95f118e07249f23abc1d4dd65de56fb9103f262c` |

The [implementation scope](equality.md) describes ordinary coercion, bounded
string work, skipped hooks and remaining exotic-value gaps.
