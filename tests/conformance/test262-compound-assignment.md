# Pinned Test262 compound-assignment inventory

The [constructor-policy follow-up](constructor-policy.md) admits Reflect call/construct
and `new.target` metadata. Current results: **617 passed, 169 unsupported**.
The inventory and source bytes are unchanged. Measurements and policy descriptions
below retain the history of earlier checkpoints.

The [general Window binding follow-up](window-global-bindings.md) raises this
profile to **617 passed / 169 unsupported**. Eleven strict captured-global writes
now reach their expected ReferenceError after a getter deletes the property.
Source inventory and policy are unchanged.

This profile retains every direct JavaScript file in
[language/expressions/compound-assignment](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/language/expressions/compound-assignment)
at revision `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`: **454 sources, 453,728
source bytes and 786 mode cases**. The directory is flat. Modes comprise 398
sloppy and 388 strict variants; 66 sources specify noStrict and 56 onlyStrict.
There are 45 negative parse/SyntaxError variants. Arithmetic assignments and
private-field prerequisites are retained alongside the six new bitwise forms.

Original LICENSE, INTERPRETING.md, sources and four unchanged helpers (assert,
sta, propertyHelper and compareArray) total **499,424 bytes**. The latter two
helpers support core assertion controls even though selected tests do not
request them. The first importer preparation omitted those control dependencies
and the runner refused before measurement; import was corrected using the same
pinned helper bytes. No controls, source files or expectations were removed.
The [manifest](../upstream/test262-compound-assignment/manifest.json) SHA-256 is
`edc2b04b5e9f810e1757a327f5d2a32c240f41d61e8a6d2a89cb35c3ec14f1d0`.

The unchanged core-only policy retains **96 metadata-unsupported modes** for
class-fields-private; eight of these also require exponentiation. Sources and
metadata are identical between before and candidate measurements.

The 32 core assertion controls are joined by four positive/wrong pairs for each
of six operators, in both modes: **128 preflights**. Each added control first
executes a successful canonical assignment. Pairs check returned/written values,
single reference and conversion order, abrupt effects and strict/sloppy write
failure. Wrong partners differ only at the final assertion and require
Test262Error. A parser SyntaxError cannot satisfy a negative control.

Six Python groups check inventory/helpers, admission, guarded pairs, actual
assertion identity, corrupted/incomplete imports and earlier contracts. All
thirteen earlier profiles retain **4,745 case and 920 preflight fingerprints**,
plus manifests, fixtures and feature sets. Replay against the actual runner at
`479e6b7` confirms equality; canonical digest:
`ae448f013345bdb5bc769ef0fad714bd9f5ee289345ca89fd096d48988cb3d34`.

| Outcome | Before 479e6b7 | Six bitwise assignments |
| --- | ---: | ---: |
| Passed | 290 | **596** |
| Failed | 328 | **10** |
| Unsupported | 168 | **180** |
| Resource, harness error, timeout or adapter error | 0 | **0** |
| Verified preflights | 32/128 | **128/128** |

The [initial report](test262-compound-assignment-initial.json) preserves the
unsupported-operator parser failures. All 96 new controls fail initially, so
that snapshot cannot record a healthy baseline. The
[candidate report](test262-compound-assignment-latest.json) retains all 290
passes and adds **306 with no losses**. Twelve former failures now reach an
explicit unsupported operation: six dynamic eval and six host property
definitions. Twenty-four already-passing negative syntax cases now report the
proper invalid-target or strict eval/arguments assignment error instead of a
generic expression error. Every changed diagnostic remains in the comparison.

The ten remaining failures are both modes of five existing += sources:
S11.13.2_A4.4_T1.4.js and S11.13.2_A4.4_T2.6.js through T2.9.js. They exercise
boxed/string addition conversion, which this increment does not change. All
report runtime Test262Error. Unsupported results comprise 96 metadata modes,
29 runtime eval cases, 44 parse-time with cases and 11 host property-definition
cases. These prerequisites remain full original tests.

Actual baseline recording and a subsequent CLI gate return zero and reproduce
the complete candidate observations. The
[checkpoint baseline](https://github.com/Eriskii/ErisBrowser/blob/09fe914310df83f57e17c4af8e99f9378732fc82/tests/conformance/test262-compound-assignment-current.json) retains all ten
failures and 180 unsupported outcomes while protecting passes. A plain run
still returns one; the CI gate is not a claim of all-pass conformance.

Every observation in twelve previous profiles is identical. In the separate
[parsing comparison](test262-numeric-parsing-compound-assignment.json), the
unchanged decimalToHexString.js helper now loads. Its four dependent modes
reach their complete Unicode loops and stop at the existing instruction limit:
**164 passed / 46 unsupported / eight resources**, with 80 verified controls.
There are no new parsing passes, and resources still prevent a healthy parsing
baseline. The four earlier loop limits and all earlier evidence are retained.

```sh
python3 tools/import_test262.py --profile compound-assignment
python3 tools/test262_conformance.py --profile compound-assignment
python3 tools/test262_conformance.py --profile compound-assignment --baseline tests/conformance/test262-compound-assignment-current.json
```

| Artifact | SHA-256 |
| --- | --- |
| Before adapter | `528693af5794bc16e18b956ffe781cbab91b335615c82ea773eca6fe2b13c126` |
| Candidate adapter | `92710e2646cc80ba2d2839837941e5c6fe62242f6cad772396074bb880805d78` |
| Candidate source input | `100da85dd37e4aee42d21f6b83afa7ff6459a21e87042cd4b9274a4a9e6f6931` |
| Initial report | `611dd86904c245f46d8830f492849495302b24bceb25c07d675c25c2b4b3f68a` |
| Candidate report | `ff7fe1bb8ca0f645e6f0feefa6c9371d827fb4a29fc78306d38a8547329fecd7` |
| Checkpoint baseline | `75f029ea4ce4364b96fbba24f06cad70baec3cb8f5d71ca490b79e58ac419a81` |
| Parsing comparison | `bffcd9ab4898076622c1002fc11c3a6523eb1185981c27f44468ca7fb6520950` |

The [implementation scope](compound-assignment.md) describes reference semantics
and distinguishes the new Number operators from remaining language gaps.

## Subsequent ordinary addition conversion

The [addition increment](test262-addition.md) resolves all ten boxed/string +=
failures, producing **606 passed / 180 unsupported** with all 128 controls.
The [full comparison](test262-compound-assignment-addition.json) retains every
case and records the ten gains. The
[current baseline](test262-compound-assignment-current.json) protects them; the
checkpoint baseline linked above preserves the earlier evidence.
