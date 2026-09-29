# Pinned Test262 relational comparison inventory

The [constructor-policy follow-up](constructor-policy.md) admits Reflect call/construct
and `new.target` metadata. Current results: **300 passed, 64 unsupported**.
The inventory and source bytes are unchanged. Measurements and policy descriptions
below retain the history of earlier checkpoints.

This profile retains every direct JavaScript file in four complete, flat
[expression directories](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/language/expressions)
at revision `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`:

| Directory | Sources | Source bytes |
| --- | ---: | ---: |
| less-than | 45 | 58,363 |
| greater-than | 49 | 60,601 |
| less-than-or-equal | 47 | 58,700 |
| greater-than-or-equal | 43 | 56,476 |
| Total | **184** | **234,140** |

There are **364 mode cases**: 184 sloppy and 180 strict. Four sources are
noStrict; none have negative metadata. Original LICENSE, INTERPRETING.md,
sources and four unchanged helpers (assert, sta, propertyHelper and compareArray)
total **279,836 bytes**. The latter two helpers support core assertion controls.
The [manifest](../upstream/test262-relational/manifest.json) SHA-256 is
`34497c818a4de422d2628afe1270525fe6c5467bbb43126c1796324a5bc8f138`.

The unchanged core feature policy retains **56 metadata-unsupported modes**:
52 require BigInt and four require BigInt plus Symbol. Sources, expectations
and metadata are identical before and after implementation.

The 32 core assertion controls are joined by six positive/wrong pairs for each
operator in both modes: **128 preflights**. Each added control starts with a
boxed-string comparison. Pairs check type selection, live conversion order,
abrupt identity, UTF-16 ordering, fallback hooks and left-associative chains.
Wrong partners change only the final assertion and require actual Test262Error;
parser failures cannot satisfy a wrong-answer control.

Six Python groups check inventory/helper integrity, feature admission, guarded
pairs, assertion identity, corrupt/incomplete imports and prior contracts.
All seventeen earlier profiles retain **6,104 case and 1,344 preflight
fingerprints**, manifests, fixtures and feature sets. Replay against the actual
runner at `bf0fa0d` confirms equality; canonical digest:
`fea8c5972fb80b8e3e87e7195dbc38005cf50251d80428f9ec6e7a6b192a87ed`.

| Outcome | Before bf0fa0d | Ordinary relational conversion |
| --- | ---: | ---: |
| Passed | 292 | **300** |
| Failed | 8 | **0** |
| Unsupported | 64 | **64** |
| Resource, harness error, timeout or adapter error | 0 | **0** |
| Verified preflights | 80/128 | **128/128** |

The [initial report](test262-relational-initial.json) preserves every observation,
including failed positive controls. The
[candidate report](test262-relational-latest.json) adds **eight passes without
losses**, covering both modes of each operator's `A3.2_T1.2` boxed-string source.
Alongside the 56 metadata modes, eight `A1` variants reach unsupported dynamic
eval. These original cases remain in the inventory.

Actual baseline recording and a subsequent gate return zero and reproduce every
candidate observation. The [current baseline](test262-relational-current.json)
retains all 64 unsupported results while protecting passing cases. A plain run
returns one; the CI gate does not imply full relational conformance.

All observations and policies in the seventeen earlier profiles are identical.
Existing resource stops in URI, numeric parsing, identifiers, functions and sort
remain explicit; none are converted into healthy baselines. HTML remains at
3,868 matches, two mismatches and six unsupported modes.

```sh
python3 tools/import_test262.py --profile relational
python3 tools/test262_conformance.py --profile relational
python3 tools/test262_conformance.py --profile relational --baseline tests/conformance/test262-relational-current.json
```

| Artifact | SHA-256 |
| --- | --- |
| Before adapter | `1c7c0853356cc5282833d3ab0ed5deede421cbab3022668e1ba3cb5bcb7f7d06` |
| Candidate adapter | `a1697ac1b5fa8c70b6c8dc2990bc473bcd5680e892b6f9cfbdc8fc5788740157` |
| Candidate source input | `6838e6c870859dfd799bfe7204775ad81c61e4bd27ef2c8ca7f996ad429f4e88` |
| Initial report | `b059a49ae0cbbcee94e9ed64aefbcd148fcc7c7ef97c7510eb833124d7b084a7` |
| Candidate report | `b1ada742caa5c3bfa3aa2f5cacf33a45b4dcdbd50655892fdf1254cb467dea4e` |
| Current baseline | `e86260961b586de3423fbd9bab9bed25559745c314803d1107ad02a1b58078d1` |

The [implementation scope](relational.md) describes ordinary conversion,
UTF-16 selection, resource accounting and remaining exotic-value gaps.
