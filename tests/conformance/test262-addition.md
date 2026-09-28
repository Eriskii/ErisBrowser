# Pinned Test262 addition inventory

This profile retains every direct JavaScript file in
[language/expressions/addition](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/language/expressions/addition)
at revision `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`: **48 sources, 107,338
source bytes and 95 mode cases**. The directory is flat. There are 48 sloppy
and 47 strict modes, with one noStrict source and no negative metadata.

Original LICENSE, INTERPRETING.md, sources and four unchanged helpers (assert,
sta, propertyHelper and compareArray) total **153,034 bytes**. The latter two
helpers support core assertion controls. The
[manifest](../upstream/test262-addition/manifest.json) SHA-256 is
`56088f55075e4d44da6915ca41383604182a8a72aaef8230af61369513cbdc0c`.

The unchanged core feature policy retains **26 metadata-unsupported modes**
for Symbol.toPrimitive, Symbol, BigInt and computed-property-names. Sources,
expectations and metadata are identical before and after implementation.

The 32 core assertion controls are joined by eight positive/wrong pairs in both
modes: **64 preflights**. Each added control first executes object-to-string
addition. Pairs check primitive and boxed values, evaluation order, live hooks,
abrupt completion, UTF-16 strings and compound reference/abrupt behavior.
Wrong partners change only the final assertion and require Test262Error;
parser failure cannot satisfy a wrong-answer control.

Six Python groups check inventory/helper integrity, feature admission, guarded
pairs, actual assertion identity, corrupted/incomplete imports and prior
contracts. All fourteen earlier profiles retain **5,531 case and 1,048 preflight
fingerprints**, manifests, fixtures and feature sets. Replay against the actual
runner at `09fe914` confirms equality; canonical digest:
`d63bb1ae7775ccd7b9fbde2fcd1bb6773e31a528f1f1eaf7bf85226ec3e23ff7`.

| Outcome | Before 09fe914 | Ordinary addition |
| --- | ---: | ---: |
| Passed | 51 | **65** |
| Failed | 16 | **2** |
| Unsupported | 28 | **28** |
| Resource, harness error, timeout or adapter error | 0 | **0** |
| Verified preflights | 48/64 | **64/64** |

The [initial report](test262-addition-initial.json) preserves all observations,
including failed positive controls. The
[candidate report](test262-addition-latest.json) adds **14 passes with no losses**.
The only remaining failures are both modes of S11.6.1_A2.2_T2.js, which require
unimplemented Date and report ReferenceError. Alongside the 26 metadata modes,
two S11.6.1_A1.js variants reach unsupported dynamic eval. These original cases
remain in the inventory; they are not rewritten or hidden.

Actual baseline recording and a subsequent gate return zero and reproduce
every candidate observation. The
[current baseline](test262-addition-current.json) retains both failures and all
28 unsupported results while protecting passing cases. A plain run returns one;
the CI gate does not imply all-pass addition conformance.

All observations in thirteen earlier profiles are identical. The
[compound-assignment comparison](test262-compound-assignment-addition.json)
adds ten passes, covering both modes of five boxed/string += sources. Its
inventory now has **606 passed / 180 unsupported**, with all 128 controls.
The [updated baseline](test262-compound-assignment-current.json) protects the ten
gains. All previous case/policy fingerprints and passing outcomes are retained.
Existing resource stops in parsing, identifiers, functions and sort remain;
none are converted into healthy baselines.

```sh
python3 tools/import_test262.py --profile addition
python3 tools/test262_conformance.py --profile addition
python3 tools/test262_conformance.py --profile addition --baseline tests/conformance/test262-addition-current.json
```

| Artifact | SHA-256 |
| --- | --- |
| Before adapter | `92710e2646cc80ba2d2839837941e5c6fe62242f6cad772396074bb880805d78` |
| Candidate adapter | `c9e47a8245eaa6e14c60f1ae0874213bdef2b00a6dacb4648c845155fdb136a3` |
| Candidate source input | `dc2e14d6e60eb413384ed88b7518a3207b5a79f1052f18be6549731a637d25a6` |
| Initial report | `dc4cceace7b09ab3cddb617543bd5dffc4be2a6e3170fd688a328e1faf2f5b9e` |
| Candidate report | `a815f3e2faf52e9dbb6ac4c0bb3f1eb70ddf201db2b2e2472a5a23998335d8bc` |
| Addition baseline | `d2770943fb2be1285decb499952fd908ca8021a49e321f5be26dab71a45dfd57` |
| Compound comparison | `f2140357b459bb23c0749e4c190a516d19843360cd981077337c8cdbcc848292` |
| Updated compound baseline | `11cd52f4c5416495f0653d4444f798e70beecd6afd2de328980f7025dddb5213` |

The [implementation scope](addition.md) describes ordinary conversion and bounded
string allocation, including the remaining exotic-value gaps.
