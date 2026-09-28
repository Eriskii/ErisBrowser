# Pinned Test262 coercing global predicate inventory

The profile retains every JavaScript file in the complete isFinite and isNaN
directories at Test262 revision `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`:
[isFinite](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/isFinite),
[isNaN](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/isNaN).

| Directory | Sources | Modes | Source bytes |
| --- | ---: | ---: | ---: |
| isFinite | 15 | 30 | 11,859 |
| isNaN | 15 | 30 | 11,417 |
| Total | **30** | **60** | **23,276** |

Both directories are flat. All sources run in both modes; none has negative
frontmatter or execution flags. Exact upstream bytes, including five unchanged
helpers (assert, sta, propertyHelper, isConstructor and nans), LICENSE and
INTERPRETING.md, total **69,881 bytes**. Revision-pinned API inventories and Git
blob hashes are checked before import. The [manifest](../upstream/test262-numeric-conversion/manifest.json)
hash is `de7cf1098618ad0fdb7786d4ab4f6c648bfb6fe2543981fe21ca6e0cba50928f`.

The unchanged core-only feature policy retains **32 metadata-unsupported modes**:
24 Symbol.toPrimitive, four Symbol and four Reflect.construct. Two Symbol-tagged
source files also declare arrow-function. Four additional runtime observations
report unsupported reflection on the global function bindings. No prerequisites
are removed or sources shortened to improve the count.

The runner adds 48 controls to the 32 core controls, for **80 preflights**.
Six paired Number groups in both modes cover primitive distinctions, live hook
order, abrupt identity, noncallable fallback, boxing/aliases and argument order.
Three paired groups for each global predicate cover coercion, abrupt order and
metadata/aliases. Every added variant first proves successful object conversion.
Wrong partners differ only in the final assertion and require Test262Error;
TypeError, ReferenceError and SyntaxError do not satisfy them. Descriptor helpers
restore properties before subsequent reads.

Six Python groups check corpus/helpers, admission, assertion identity, guarded
pairs, corrupt/incomplete imports and earlier contracts. All eleven prior
profiles retain **4,467 case and 760 preflight fingerprints**, plus their source
inventories, fixtures and feature sets. A replay against the actual runner from
`a2b22a1` confirms equality; canonical digest:
`33eaed6126587bc54161f7eb8113e53a0ec6eccae1bd7a6186ace5f4edf57033`.

| Outcome | Before a2b22a1 | Ordinary conversion |
| --- | ---: | ---: |
| Passed | 16 | **24** |
| Failed | 8 | **0** |
| Unsupported | 36 | **36** |
| Resource, timeout or adapter error | 0 | **0** |
| Verified preflights | 56/80 | **80/80** |

The [initial report](test262-numeric-conversion-initial.json) preserves all
failures. Its 24 failed conversion guards prevent baseline recording; wrong
controls failing at those guards do not prove conversion semantics. The
[candidate report](test262-numeric-conversion-latest.json) preserves every case
identity and all 16 earlier passes, adding eight with no losses. The improved
sources are `return-abrupt-from-tonumber-number.js` and `tonumber-operations.js`
in each directory and both modes.

Actual baseline recording and a subsequent CLI gate return zero with identical
observations. The [baseline](test262-numeric-conversion-current.json) retains
all 36 unsupported modes. A plain nonbaseline run still returns one. CI runs
the complete baseline gate; passing this gate is not full conformance.

The existing Number inventory separately gains **ten passes** from ordinary
constructor conversion: **244 passed, four failed, 92 unsupported**, with all
104 preflights verified. Its [complete comparison](test262-number-statics-numeric-conversion.json)
preserves earlier evidence; the current Number baseline now protects the ten
gains. Both modes of Number's parseInt.js and parseFloat.js remain failures
because parsing aliases are missing. Every outcome in the other ten profiles
is unchanged, including function, sort and identifier resource stops.

```sh
python3 tools/import_test262.py --profile numeric-conversion
python3 tools/test262_conformance.py --profile numeric-conversion
python3 tools/test262_conformance.py --profile numeric-conversion --baseline tests/conformance/test262-numeric-conversion-current.json
```

| Artifact | SHA-256 |
| --- | --- |
| Before adapter | `f6ee32dffda7d56c3197cac56586cb9442134f141af1ed69528baf15d8e23675` |
| Candidate adapter | `b3b0964931661f63d570d6b3bbb52061a9054fe00e4ec8a1025f624326c95d12` |
| Candidate source input | `41577621af344b5f9301064d69abbf7a65dcdf4f46de8a5d1524e9e03734004d` |
| Initial report | `dfafb5b1003d057ff80c78bf44a0462ca4be974b7225ead417763413877b283e` |
| Candidate report | `620625bdd6528b8732961c2f6d90dab9602cda512d6861c649013225199fc589` |
| Current baseline | `8fec4cd70dfea5f1c1f49e91a2fb491be0eb70cc2af18ed30afb92cda2797a7b` |
| Number comparison | `d09587b9bd1f8922d0b2797d5ddcb3592505d4c74e9067c9e8f818f5970d2c78` |
| Number baseline at cf28fc4 | `03c6fad5ceb5f119df2dc5db1d432bece74bba5b6cadc6aead3300c75882a427` |

The [implementation scope](numeric-conversion.md) and
[validation record](../../docs/VALIDATION.md) distinguish ordinary conversion
from missing Symbol, BigInt, host and constructor infrastructure.

The subsequent [parsing increment](test262-numeric-parsing.md) resolves the four
Number alias failures. The Number baseline hash above refers to
[checkpoint cf28fc4](https://github.com/Eriskii/ErisBrowser/blob/cf28fc4eba40b5a22f8ea1632e33825a3c751fc1/tests/conformance/test262-number-statics-current.json);
the current baseline and separate comparison are linked from the parsing report.
This profile's 24 passes and 36 unsupported outcomes remain unchanged.
