# Pinned Test262 labeled control-flow inventory

The [constructor-policy follow-up](constructor-policy.md) admits Reflect call/construct
and `new.target` metadata. Current results: **100 passed, 25 unsupported**.
The inventory and source bytes are unchanged. Measurements and policy descriptions
below retain the history of earlier checkpoints.

This profile retains every direct JavaScript file in three complete, flat
[statement directories](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/language/statements)
at revision `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`:

| Directory | Sources | Source bytes |
| --- | ---: | ---: |
| labeled | 24 | 12,428 |
| break | 20 | 14,845 |
| continue | 24 | 14,408 |
| Total | **68** | **41,681** |

There are **125 mode cases**: 62 sloppy, 61 strict and two modules. Flags retain
five noStrict, four onlyStrict and two module sources. There are 68 negative
mode cases, all expecting a parse-phase SyntaxError. Original LICENSE,
INTERPRETING.md, sources and five unchanged helpers (assert, sta, propertyHelper,
compareArray and tcoHelper) total **87,810 bytes**. Property/array helpers support
core controls; tcoHelper remains present even though tail calls are unsupported.
The [manifest](../upstream/test262-labels/manifest.json) SHA-256 is
`32889cc4ab3de15132243dae4981418ba559cd10294df9222329070b8f74649b`.

The unchanged core policy retains **17 metadata-unsupported modes**: eight class
static blocks, two async functions, two async iteration, two generators, two
modules and one tail-call optimization case. Sources and expectations are never
rewritten to fit implementation behavior.

The 32 core assertion controls are joined by twelve positive/wrong pairs in
both modes: **80 preflights**. Each added control starts with a labeled break.
Pairs cover blocks, nested loop kinds, alias chains, switches, finally behavior,
return/throw identity, hoisting, decoded names and function scopes. The sloppy
Unicode pair also guards labeled `let` with newline ASI. Wrong partners change
only the final assertion and require actual Test262Error; parse failures cannot
satisfy them.

Six Python groups verify inventory/helper integrity, feature admission, guarded
pairs, assertion identity, corrupt/incomplete imports and prior contracts.
All nineteen earlier profiles retain **6,754 case and 1,600 preflight
fingerprints**, manifests, fixtures and feature sets. Replay against the actual
runner at `52b373f` confirms equality; canonical digest:
`586bc156a2b40f9d5f4f3d07bcfe35e642d5b807970be341935ee6f9eb1a2ab2`.

| Outcome | Before 52b373f | Ordinary labeled control flow |
| --- | ---: | ---: |
| Passed | 30 | **100** |
| Failed | 0 | **0** |
| Unsupported | 95 | **25** |
| Resource, harness error, timeout or adapter error | 0 | **0** |
| Verified preflights | 32/80 | **80/80** |

The [initial report](test262-labels-initial.json) preserves every observation.
The [candidate report](test262-labels-latest.json) adds **70 passes without losses**.
It retains all 78 changes: 70 gains, four former label-unsupported modes now
reaching unsupported dynamic eval, and four passing negative modes whose
SyntaxError diagnostic now identifies an unknown target. The 25 unsupported
outcomes comprise the 17 metadata modes and eight dynamic-eval cases. Fifty-two
negative cases pass; all other negative outcomes remain explicit.

A preliminary candidate had 95 passes, one failure and 29 unsupported modes.
The unchanged `let-identifier-with-newline.js` exposed a statement-context ASI
bug. Correcting that grammar also handles the block/array variants and rejects
class declarations in statement position, producing five further passes.
Preliminary observations and validation remain in local
`artifacts/test262-labels-preliminary-*.json` and `artifacts/*labels-preliminary*`.
The final guard set was also executed against the frozen before adapter.

Actual baseline recording and a subsequent gate return zero and reproduce every
candidate observation. The [current baseline](test262-labels-current.json)
retains all 25 unsupported outcomes while protecting passes. A plain run returns
one; this is not an all-pass ECMAScript conformance claim.

Eighteen previous profiles preserve every observation. The
[URI comparison](test262-uri-labels.json) retains the complete inventory and adds
two passes for decodeURI/S15.1.3.1_A2.2_T1.js. Eight other formerly label-unsupported
modes now reach the existing instruction limit. URI therefore has **210 passed /
24 unsupported / 112 resources**, with all 128 controls. All 104 prior stops and
other observations persist. No healthy URI baseline is recorded; no quota is
raised. HTML remains at 3,868 matches, two mismatches and six unsupported modes.

```sh
python3 tools/import_test262.py --profile labels
python3 tools/test262_conformance.py --profile labels
python3 tools/test262_conformance.py --profile labels --baseline tests/conformance/test262-labels-current.json
```

| Artifact | SHA-256 |
| --- | --- |
| Before adapter | `5663f74effc73282db269e77aeede8ece73795c324713a4646a2e4c73df0dfb8` |
| Candidate adapter | `9e643c885523b75a1e1a991bb3e7c7a9c84601b6adacb1243932f379716c2cc0` |
| Candidate source input | `e34329d60bba0a821277b13b537bf0021285d2d2add3e554160f51e48e684c61` |
| Initial report | `dc7d19d5fe65b64307be10795b643a122223437378382008939f68a272177dc6` |
| Candidate report | `84501434aefa0a8f0e8c177cd4d850bd1c5b27b90e46c4f4f93f5aa870101329` |
| Current baseline | `5a84e2f694ccb009e7a7e21d501df73c346368df454d43d9618965a02753c22f` |
| URI comparison | `eb7f76b625fa2fe581fcc289f3b77729cc102e43c9a60171f7ca58ee894613db` |

The [implementation scope](labels.md) describes jump propagation, parser-stack
corrections, bounded target resolution and remaining completion-value gaps.
