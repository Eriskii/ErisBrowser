# Pinned Test262 numeric parsing inventory

The profile retains every direct JavaScript file in the complete parseInt and
parseFloat directories at Test262 revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`:
[parseInt](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/parseInt),
[parseFloat](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/parseFloat).

| Directory | Sources | Modes | Source bytes |
| --- | ---: | ---: | ---: |
| parseInt | 55 | 110 | 60,421 |
| parseFloat | 54 | 108 | 49,920 |
| Total | **109** | **218** | **110,341** |

Both directories are flat; all sources require both modes and have no execution
flags or negative frontmatter. Unchanged helpers are assert.js, sta.js,
propertyHelper.js, isConstructor.js and decimalToHexString.js. Sources, helpers,
LICENSE and INTERPRETING.md total **156,979 bytes**. Pinned API inventories and
Git blob hashes are verified before import. The
[manifest](../upstream/test262-numeric-parsing/manifest.json) SHA-256 is
`f8959cc2e43d94b5b0c671105f87d3fc831fa6359065566791707a6f9b400c6a`.

The unchanged core-only policy retains **38 metadata-unsupported modes**:
30 numeric-separator-literal, four u180e and four Reflect.construct. The two
Reflect sources also declare arrow-function. Original sources and prerequisites
are not shortened or reclassified to improve results.

The profile adds 48 controls to the 32 core controls: six positive/wrong pairs
per function, in both modes, for **80 preflights**. Groups cover prefix grammar,
string/radix hook order, abrupt identity, metadata, argument evaluation and
alias mutation. Every variant first proves successful object conversion and
identity with the Number alias. Wrong partners differ only at the final
assertion and require Test262Error. Descriptor helpers restore properties.

Six Python groups check pinned imports/helpers, admission, assertion identity,
paired setup, corrupted/incomplete source inventories and earlier contracts.
All twelve prior profiles retain **4,527 case and 840 preflight fingerprints**,
manifests, fixtures and feature sets. Replay against the runner from `cf28fc4`
confirms equality; canonical digest:
`ad0c739d73c9593801c2f8524fc70ef679a58372f7da1b69a26c39f31b7fdd97`.

| Outcome | Before cf28fc4 | Parsing conversion and aliases |
| --- | ---: | ---: |
| Passed | 142 | **164** |
| Failed | 20 | **0** |
| Unsupported | 48 | **46** |
| Harness error | 4 | **4** |
| Resource | 4 | **4** |
| Timeout or adapter error | 0 | **0** |
| Verified preflights | 56/80 | **80/80** |

The [initial report](test262-numeric-parsing-initial.json) preserves all
observations. Initial positive controls fail their conversion/alias guard;
wrong controls failing there do not demonstrate semantics. The
[candidate report](test262-numeric-parsing-latest.json) retains every case and
all prior passes, adding **22 passes without losses**. Twenty gains resolve
ordinary failures. Two resolve parseInt's own length-descriptor check,
`S15.1.2.2_A9.6.js`, previously unsupported before native metadata existed.
The remaining 46 unsupported modes comprise 38 metadata exclusions and eight
runtime global-reflection outcomes.

Four unchanged harness errors occur while parsing decimalToHexString.js, which
uses unsupported unsigned-right-shift assignment (`>>>=`). Both modes of
parseFloat/S15.1.2.3_A6.js and parseInt/S15.1.2.2_A8.js therefore never execute
their test bodies. These remain harness errors, not expected test failures or
passes. Four unchanged instruction-limit outcomes occur in both modes of
parseInt/S15.1.2.2_A7.2_T1.js and S15.1.2.2_A7.3_T1.js, whose nested loops
exercise complete radix matrices.

An actual baseline-recording invocation returns one and creates **no baseline**.
Its repeated observations exactly match the candidate. The profile remains a
local measurement; it is not an all-pass conformance or healthy regression gate.
The existing [Number inventory comparison](test262-number-statics-numeric-parsing.json)
separately gains four alias modes, reaching **248 passed / 92 unsupported** with
104 verified preflights. Actual recording and gating preserve those gains in
the existing Number CI baseline. Every outcome in the other eleven profiles
is unchanged, including prior resource limits.

A separate root-authored matrix generates 1,050 signed strings, covering radices
2–36 and widths 1–10. Python integer arithmetic supplies exact expected values,
all below 2^53. Direct global calls and coercing Number aliases in both modes
pass **4,200 assertions in 132 bounded batches**. The frozen before binary fails
all batches at the alias/conversion guard; a deliberately wrong candidate
control throws Test262Error. This supplemental matrix does not replace, shorten
or change the upstream loops or their resource outcomes. Input SHA-256:
`0b09024529f5d2cf8488e9741faccee6ef103e2a56f15ba10c844a1a89574073`.
It is a local sampled check, not an independent-agent or arbitrary-magnitude
rounding proof.

```sh
python3 tools/import_test262.py --profile numeric-parsing
python3 tools/test262_conformance.py --profile numeric-parsing
```

| Artifact | SHA-256 |
| --- | --- |
| Before adapter | `b3b0964931661f63d570d6b3bbb52061a9054fe00e4ec8a1025f624326c95d12` |
| Candidate adapter | `528693af5794bc16e18b956ffe781cbab91b335615c82ea773eca6fe2b13c126` |
| Candidate source input | `4e98b1e45cd79f3b964d213cfc6c2bbfd012977d7243e9a1e9fe69bfd144a58c` |
| Initial report | `24d30b521a510b25b83895e02b2fbbebba2d416a752f6206c5d80c8edd64a77a` |
| Candidate report | `6b140c49b8d86675ea8c9cdce3d95d97e41a337eca7163c9e8527ea4a5631e8c` |
| Number comparison | `661abf24bd9e0dc8f92a44f490eafce13048fc6c798e0070e10bb7fafeaf22ac` |
| Updated Number baseline | `8b4837fb66a9ced2a9e1910830bc5a8aa67d81d52b6766764c2aceddef7b8717` |

The [implementation scope](numeric-parsing.md) records the ordinary conversion,
metadata, resource accounting and remaining unsupported infrastructure.
