# Pinned Test262 Number static builtin inventory

This profile retains every direct JavaScript file in the Number root and eight
complete constant/predicate directories at Test262 revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`.
[Upstream root](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/Number).

| Directory relative to Number | Sources | Variants | Source bytes |
| --- | ---: | ---: | ---: |
| Direct root | 120 | 240 | 97,498 |
| MAX_VALUE | 4 | 8 | 2,496 |
| MIN_VALUE | 4 | 8 | 2,580 |
| NEGATIVE_INFINITY | 4 | 8 | 1,902 |
| POSITIVE_INFINITY | 4 | 8 | 1,901 |
| isFinite | 8 | 16 | 6,024 |
| isInteger | 9 | 18 | 6,231 |
| isNaN | 7 | 14 | 5,227 |
| isSafeInteger | 10 | 20 | 7,336 |
| Total | **170** | **340** | **131,195** |

All sources require both strict and sloppy modes, with no negative frontmatter.
The eight selected subdirectories have no nested entries. The root is imported
directly, including constructor and parseInt/parseFloat tests unrelated to the
implemented slice; other Number subdirectories are not recursively imported.
EPSILON, MAX_SAFE_INTEGER, MIN_SAFE_INTEGER and NaN tests live directly in root.
No source is selected or shortened based on anticipated passing behavior.

The [manifest](../upstream/test262-number-statics/manifest.json) records exact
source sizes and hashes. Files were checked against revision-pinned Git blobs;
original bytes, license and INTERPRETING.md are preserved. The four unchanged
harnesses are assert.js, sta.js, propertyHelper.js and isConstructor.js. Total
imported content is **177,192 bytes**. Manifest SHA-256:
`3c5c2503458843bfc64eaf546ee08b6e693714f561df42330d9001a2ec8216cd`.

Admission uses the unchanged core feature set only. **90 metadata-unsupported
variants** retain numeric separators, u180e, Symbol, BigInt, Reflect,
exponentiation or cross-realm prerequisites. In particular, mixed ordinary/Symbol
argument tests remain complete excluded files, rather than being rewritten to
remove Symbol assertions. Another two variants reach unsupported host property
reflection during Number's global descriptor check. All **92 unsupported
observations** remain unchanged between the initial and candidate measurements.

## Assertion controls and unchanged contracts

The profile keeps 32 core preflights and adds 18 positive/wrong assertion pairs
in both modes: **104 preflights total**. Three groups per predicate cover
classification, noncoercion/receivers and metadata/aliases/nonconstructability.
Six constant groups cover special values, finite extrema, epsilon, safe limits,
descriptors and strict/sloppy immutability. Every method-dependent negative check
first proves callable availability and a successful canonical call. Wrong
partners differ only in their final assertion and must report Test262Error.
Descriptor helpers use `restore:true` before later reads.

Six Python groups check pinned inventory/helpers, admission, actual assertion
identity, guarded pairs, corrupted/incomplete imports and prior contracts.
All ten earlier profiles retain **4,127 case and 656 preflight fingerprints**,
plus manifests, fixtures and feature sets. A replay against the runner from
checkpoint `58c027a` confirms identical contracts; their canonical digest is
`9a15cbffcdbf5d6de826c9d56453e38787867f2e8effb3386fc1d08033042f8f`.
The new profile's policy digest is
`81b61b3419f5a73997b1f29d352177e4847797cf9c8d8fcc353a52c0acb6be2e`.

## Complete measurements

| Outcome | Initial checkpoint 58c027a | Number implementation |
| --- | ---: | ---: |
| Passed | 158 | **234** |
| Failed | 90 | **14** |
| Unsupported | 92 | **92** |
| Resource, timeout or adapter error | 0 | **0** |
| Verified preflights | 70/104 | **104/104** |

The [full initial report](test262-number-statics-initial.json) preserves every
observation. Its failed availability controls prevent a healthy baseline;
individual wrong-partner successes at an availability assertion are not evidence
of method semantics. The [full candidate report](test262-number-statics-latest.json)
retains all case/preflight identities and all 158 earlier passes, adding **76
passes with no losses**. Every failure and unsupported case remains recorded.

The 14 remaining failures are both modes of seven direct-root sources:

- `S8.12.8_A3.js`, `S8.12.8_A4.js`, `S9.1_A1_T1.js`, `S9.3_A5_T1.js` and
  `return-abrupt-tonumber-value.js`: incomplete Number constructor conversion
  and abrupt-completion behavior.
- `parseFloat.js` and `parseInt.js`: missing Number parsing aliases.

All 14 report runtime Test262Error, `uncaught exception: [object Object]`.
The full reports group exact diagnostics and case IDs for every initial and
remaining nonpass. The [implemented scope](number-statics.md) remains separate
from these retained gaps.

An actual baseline-recording invocation and a subsequent CLI gate both return
0, preserving exactly the candidate's cases, preflights and observations.
The [baseline at this checkpoint](https://github.com/Eriskii/ErisBrowser/blob/a2b22a1f2c298ce70346552d64b96d82c15cbb37/tests/conformance/test262-number-statics-current.json)
contains all 340 statuses, including the 14 failures and 92 unsupported cases;
it preserves passes without claiming all-pass conformance. A plain nonbaseline invocation
continues to return 1. CI runs the baseline gate.

```sh
python3 tools/import_test262.py --profile number-statics
python3 tools/test262_conformance.py --profile number-statics
python3 tools/test262_conformance.py --profile number-statics --baseline tests/conformance/test262-number-statics-current.json
```

| Artifact | SHA-256 |
| --- | --- |
| Before adapter | `2b0e59686ff8fb8db409c8a54b8f92df592f00668fcd9ff00da2de0b3585705a` |
| Candidate adapter | `f6ee32dffda7d56c3197cac56586cb9442134f141af1ed69528baf15d8e23675` |
| Candidate source input | `f9f8fbdd7b40a1a276b7eb493113639e9bd9478913571e7543e4141b8ed324a5` |
| Initial report | `64bf8b71ec67dd8d38fc4281d7c8c5ddae3ecd5f73bbd4eaf33633a9b06eb21d` |
| Candidate report | `53a04f8c09aca71d2c5875e179108cba2a2e999e9cc314eaec49359f0428bb96` |
| Original baseline at a2b22a1 | `153ed888ebf9af8394517d3078a6db35c3ff6c553a2f84d919ba25e93ac3c5e3` |

Number.MAX_SAFE_INTEGER also enables the unchanged near-limit reduceRight
source in both modes. The [complete separate comparison](test262-array-reduce-number-statics.json)
retains 1,034 variants and 128 preflights, with **848 passes, 16 failures and
170 unsupported cases**. Its regression baseline now preserves those two gains.

## Subsequent ordinary conversion

The [numeric conversion increment](test262-numeric-conversion.md) resolves all
ten constructor-conversion failures: **244 passed, four failed, 92 unsupported**.
The [full comparison](test262-number-statics-numeric-conversion.json) retains
all 340 case identities and 104 verified preflights with no lost passes.
The [baseline at cf28fc4](https://github.com/Eriskii/ErisBrowser/blob/cf28fc4eba40b5a22f8ea1632e33825a3c751fc1/tests/conformance/test262-number-statics-current.json)
protects those gains; its SHA-256 is
`03c6fad5ceb5f119df2dc5db1d432bece74bba5b6cadc6aead3300c75882a427`.
Both modes of parseFloat.js and parseInt.js remain failed. Original reports and
the original baseline linked above remain historical evidence.

## Subsequent parsing aliases

The [numeric parsing increment](test262-numeric-parsing.md) resolves the four
remaining alias failures: **248 passed / 92 unsupported**, with all 104 controls
verified. The [complete comparison](test262-number-statics-numeric-parsing.json)
retains every case and adds four passes without losses. The
[current baseline](test262-number-statics-current.json) protects these gains;
SHA-256 `8b4837fb66a9ced2a9e1910830bc5a8aa67d81d52b6766764c2aceddef7b8717`.
Earlier reports and linked checkpoint baselines remain historical evidence.
