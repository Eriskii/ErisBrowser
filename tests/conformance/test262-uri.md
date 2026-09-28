# Pinned Test262 URI inventory

This profile retains every direct JavaScript source in four builtin directories
at Test262 revision `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`:
[encodeURI](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/encodeURI)
(31), [encodeURIComponent](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/encodeURIComponent)
(31), [decodeURI](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/decodeURI)
(55), and [decodeURIComponent](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/decodeURIComponent)
(56). These total **173 sources / 231,287 source bytes / 346 modes**, equally
split between strict and sloppy execution. No source has flags or negative
metadata. Complete Unicode loops remain unchanged.

Original LICENSE, INTERPRETING.md, sources and six unchanged helpers (assert,
sta, propertyHelper, compareArray, isConstructor and decimalToHexString) total
**278,168 bytes**. The
[manifest](../upstream/test262-uri/manifest.json) SHA-256 is
`bf48d1dddac882a9609fe78aea7521e7477c1cef2ba6036a74c48513ad28ab8f`.
The unchanged core feature policy excludes eight Reflect.construct modes.
Sources, expectations, helpers and admission are identical before and after.

The 32 core controls are joined by six positive/wrong pairs for each function
in both modes: **128 preflights**. Each added control first calls its builtin.
Pairs cover Unicode, reserved sets, conversion, abrupt completion, malformed
input and metadata. Wrong partners alter only the final assertion and require
actual Test262Error. A missing builtin or parser error cannot satisfy them.
Six Python groups check inventory/helpers, admission, guarded pairs, assertion
identity, damaged/incomplete imports and old profile contracts.

Replay against the actual runner at `dbe8990` retains all sixteen previous
profiles' **5,758 case and 1,216 preflight fingerprints**, manifests, fixtures
and feature sets. Canonical digest:
`c2db14dd09f5e37be3d75237cb6cefcb6294921ce460793062c864d5e101fa69`.

| Outcome | Before dbe8990 | URI functions |
| --- | ---: | ---: |
| Passed | 0 | **208** |
| Failed | 260 | **0** |
| Unsupported | 34 | **34** |
| Runtime instruction-limit stops | 52 | **104** |
| Harness error, timeout or adapter error | 0 | **0** |
| Verified preflights | 32/128 | **128/128** |

The [initial report](test262-uri-initial.json) and
[candidate report](test262-uri-latest.json) retain all observations. The 208
new passes lose none. All 52 old instruction-limit stops persist unchanged;
52 former failures now reach the shared instruction limit in complete loops.
Those transitions are not counted as passes. Remaining unsupported modes are
ten labeled-statement variants, sixteen host-reflection variants and eight
Reflect.construct prerequisites. No upstream loop or resource ceiling changed.

An actual baseline-recording attempt returns one and reproduces every candidate
observation. The 104 resources prevent a healthy URI baseline; no current URI
baseline or URI CI regression gate is installed. A plain run also returns one.
The [implementation record](uri.md) documents supplemental independent-codec
oracle checks without substituting them for these full upstream loops.

Every observation in fifteen earlier profiles remains identical. The
[global-values comparison](test262-global-values-uri.json) adds four passes for
URI bindings, producing **42 passed / six failed / 40 unsupported** with all
64 controls. The failures require Date. Actual recording and gate verification
protect those four gains in the
[updated global baseline](test262-global-values-current.json).

```sh
python3 tools/import_test262.py --profile uri
python3 tools/test262_conformance.py --profile uri
```

| Artifact | SHA-256 |
| --- | --- |
| Before adapter | `e9d33d19a55217d159ab6e337e6f4af21a69651a7325b762247505f7c3b5cdf4` |
| Candidate adapter | `1c7c0853356cc5282833d3ab0ed5deede421cbab3022668e1ba3cb5bcb7f7d06` |
| Candidate source input | `9b4cf0cd976c9fe947cbbe8addf0bfc281dc5ec5dcd19a8c7026a7835bdbfe50` |
| Initial report | `d76eadd66d781a17bc044f3eeb2be4cd1e032f68150fd7f6540ae481f4bdb97f` |
| Candidate report | `ce06f9bc7163d0c309d741e5853289ddfb0aac16a7cce25c1dd3ac621173b59a` |
| Global-values comparison | `b9b534ee1d5493a976637b6a7eab02a57e32e16a968408f1de754537abc5533a` |
| Updated global baseline | `ea3dd8c07ab8a76f22e02b4de9ce01035547ec06c37ef4f08569d922425196ee` |

## Subsequent ordinary labels

The [labeled control-flow increment](test262-labels.md) adds two decodeURI passes
and allows eight formerly unsupported variants to reach their full-loop
instruction limit. The [complete comparison](test262-uri-labels.json) records
**210 passed / 24 unsupported / 112 resources**, with all 128 controls and no lost
passes. All 104 earlier stops remain. No healthy URI baseline is recorded; the
historical tables above retain the original URI checkpoint evidence.

## Window reflection follow-up

The later [Window reflection checkpoint](window-reflection.md) adds 16 global URI function reflection passes: **226 passed / eight unsupported / 112 runtime resources**, with all 128 controls verified. Resource observations and every other outcome remain unchanged; there is still no healthy baseline.
