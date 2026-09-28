# Pinned Test262 global-value selection

This separate profile retains every direct `.js` file in four Test262 built-in
directories at revision `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`:

| Directory under `test/built-ins` | Sources | Sloppy | Strict | Variants |
| --- | ---: | ---: | ---: | ---: |
| `global` | 29 | 27 | 29 | 56 |
| `undefined` | 8 | 7 | 5 | 12 |
| `NaN` | 6 | 6 | 4 | 10 |
| `Infinity` | 6 | 6 | 4 | 10 |
| Total | **49** | **46** | **42** | **88** |

These directories contain no nested directories or fixture files at this pin.
All selected tests have positive metadata. There is no `built-ins/globalThis`
directory: its two tests are `global/global-object.js` and
`global/property-descriptor.js`. Retaining the complete `global` directory also
retains tests of other global functions, constructors, enumeration and eval;
this is not a selection filtered to tests that exercise only four properties.

The corpus includes unchanged `assert.js`, `sta.js`, `propertyHelper.js`,
`LICENSE` and `INTERPRETING.md`. Test sources total 34,804 bytes. Each test was
verified against its pinned Git blob, and the runner verifies every retained
source, harness, metadata and mode fingerprint. No test or harness is shortened,
rewritten or removed to accommodate a missing API. The imported data retains its
[upstream license](../upstream/test262-global-values/LICENSE).

Manifest SHA-256:
`c1b627a7249d3ab460d5be88b54f34ead98c375c160bee9e72379aef213894a4`.

```sh
python3 tools/import_test262.py --profile global-values
python3 tools/test262_conformance.py --profile global-values
python3 -m unittest discover -s tools -p 'test_test262_conformance.py'
```

## Execution policy and assertions

The isolated policy adds only `globalThis` to the existing core feature set:
`String.fromCodePoint`, `arrow-function`, `for-in-order`, and
`well-formed-json-stringify`. Every earlier profile's feature policy, corpus and
preflight definitions remain unchanged. Only two selected files declare a
feature (`globalThis`, four variants); all 88 variants are admitted for actual
execution. Admission does not claim all their operations are implemented.

The execution-policy SHA-256, with the normal three-second per-process deadline,
is `f096c9be039b6e5a005d3997e38d8da419399e8c6e8d3a23d2162d20c1763d03`.
The [shared runner contract](test262.md) applies: bounded fresh processes,
unchanged assertion helpers, intrinsic error identities, stable adapter hashes,
and complete source/harness/mode comparisons.

The profile keeps **32 core preflights** and adds **32 global-value preflights**,
with eight positive/deliberately incorrect pairs in both execution modes:

- Initial values and global-this identity.
- The data descriptor flags and values of undefined, NaN and Infinity.
- The initial writable, nonenumerable, configurable globalThis descriptor.
- Ignored sloppy writes versus strict TypeError, without value coercion.
- Failed sloppy deletion versus strict TypeError for immutable value properties.
- Replacing globalThis while top-level and explicitly supplied realm this remain stable.
- Deleting and recreating globalThis, including its newly created data-property flags.
- Lexically shadowing globalThis without changing the Window property or realm this.

The expectations follow [ECMAScript global value properties](https://tc39.es/ecma262/multipage/global-object.html#sec-value-properties-of-the-global-object)
and [GlobalDeclarationInstantiation](https://tc39.es/ecma262/multipage/ecmascript-language-scripts-and-modules.html#sec-globaldeclarationinstantiation).
A deliberately incorrect check must raise the unchanged harness's Test262Error;
an unrelated TypeError, missing binding or parse error does not satisfy it.
All **64 preflights** must verify before a healthy regression baseline can be
recorded.

New positive preflights use descriptor reads, assignments, deletion and unchanged
assertions. They do not require host own-key enumeration, Window hasOwnProperty
or propertyIsEnumerable. The ordinary-object propertyHelper checks from the core
preflight remain intact. Corpus tests requesting propertyHelper still execute
the entire unchanged helper: its actual enumeration/reflection/write/delete
operations are not replaced by descriptor-only checks.

Four focused Python test groups check exact filenames/counts/modes, unchanged
harness bytes, isolated admission policy, old-preflight preservation, detection
of disabled assertions and wrong exception types, and importer rejection of a
corrupted pinned blob in every directory or an incomplete directory inventory.
The complete focused module contains **40 passing tests** at import time.
A separate identity comparison confirmed all **2,229 mode fingerprints** and
**296 preflight fingerprints** across the six earlier profiles remain unchanged,
along with their manifest hashes and feature policies.

## Frozen initial measurement

The [complete initial report](test262-global-values-initial.json) measures the
preserved adapter from checkpoint `a9f4530ff3f68c77831a30af919fd606f9f588b1`,
before the global-value correction. It records every observation rather than
only a count or list of passes.

| Outcome | Variants |
| --- | ---: |
| Passed | 32 |
| Failed | 16 |
| Unsupported | 40 |
| Resource stop, timeout or adapter error | 0 |

All **32 core preflights** and **12 of the 32 new preflights** verify, for
**44/64** overall. Incorrect descriptor flags, protected globalThis writes and
deletion, and rejected lexical shadowing prevent the new positive checks from
verifying. Some deliberately wrong assertions also complete on the old behavior;
that is a preflight failure, not a success. A requested initial baseline write
was refused, and **no healthy initial baseline exists**.

Six failed variants are the direct descriptor checks for undefined, NaN and
Infinity. The remaining ten fail on missing Date or URI-function bindings. The
40 unsupported observations comprise 18 dynamic-eval variants, 14 host
own-property-enumeration variants, and eight host own-property-reflection
variants. These dependencies are often unlabelled in older source metadata;
no synthetic exclusions are added to hide them. The four propertyHelper sources
remain eight actual unsupported observations.

All failing sources run in both modes. Exact failure groups are:

| Sources under `test/built-ins` | Variants | Error type / intrinsic identity | Actual message |
| --- | ---: | --- | --- |
| `undefined/15.1.1.3-0.js`, `NaN/15.1.1.1-0.js`, `Infinity/15.1.1.2-0.js` | 6 | Test262Error / empty | `uncaught exception: [object Object]` |
| `global/S10.2.3_A1.1_T2.js`, `global/S10.2.3_A1.2_T2.js` | 4 | ReferenceError / ReferenceError | `'decodeURI' is not defined` |
| `global/S10.2.3_A1.1_T3.js`, `global/S10.2.3_A1.2_T3.js`, `global/global-object.js` | 6 | ReferenceError / ReferenceError | `'Date' is not defined` |

Test262Error is the authored harness exception, so its intrinsic-identity field
is empty. These descriptor failures differ from the intrinsic ReferenceErrors
caused by missing unrelated globals. No missing binding is synthesized to make
those presence tests pass.

Initial adapter SHA-256:
`073e46ca2273fb55e458432d2b17c4edac6b6f6110f61a83d79b3eaeb2bc6d8b`.
Source-input digest:
`5e4cfb6d251433ae56cc1900fd3380dc19e8b759de5bc2a162a45ac3efddcab2`.
Sanitized full initial-report SHA-256:
`55ea2a7102ad2376a856d8e110b63163237dd64da4523aedb5c944d7906654c4`.

## Implemented-value checkpoint

The [complete updated report](test262-global-values-latest.json) records:

| Outcome | Initial | Updated |
| --- | ---: | ---: |
| Passed | 32 | **38** |
| Failed | 16 | **10** |
| Unsupported | 40 | **40** |
| Resource stop, timeout or adapter error | 0 | **0** |
| Verified preflights | 44/64 | **64/64** |

Six variants newly pass: both modes of `undefined/15.1.1.3-0.js`,
`NaN/15.1.1.1-0.js` and `Infinity/15.1.1.2-0.js`. All **32 earlier passes** remain.
The ten missing-Date/URI failures and all 40 unsupported variants retain their
statuses and observations. No case, harness, metadata, mode, feature policy or
preflight identity changed: the comparison verified all **88 case fingerprints**
and **64 preflight fingerprints**, plus source hashes, manifest and policy.

After those checks, the runner recorded the
[checkpoint regression baseline](https://github.com/Eriskii/ErisBrowser/blob/dbe89907138bc8331b865a97da16d40042f39b31/tests/conformance/test262-global-values-current.json). A separate invocation
of the actual baseline CLI gate exited 0 with zero regressions and zero further
improvements. The non-baseline run exits 1 because ten failures and forty
unsupported variants remain. The runner's healthy-baseline criterion requires
all preflights to verify and excludes resource, timeout and adapter errors;
it does not require every retained upstream test to pass.

```sh
python3 tools/test262_conformance.py --profile global-values --baseline tests/conformance/test262-global-values-current.json
```

Measured adapter SHA-256:
`3bcf5b560c662c70568c0748c6646f85fe056710231bd339b0317b67ec3c7663`.
Source-input digest:
`7ab85433d856ca68c585e4a02c3f52a96dcb3ae90f72414b8e8282b1f6415975`.
Runtime source SHA-256:
`c85a3408efb0e177f6493e12a0e7d294d943f7af57f837cb1e19cd24ba70f214`.
Sanitized latest-report SHA-256:
`d7f49559aca55c998a9f36090f02d347d09cc4632d5c4a22fc02b04b301a39e0`.
Checkpoint baseline SHA-256:
`868158b8a79305e12fc1a06f73f645bd40597c099fbef4197b8fa8cc41e0fcdf`.

The full latest report includes both measurements' provenance, every outcome,
the six exact gains and actual baseline-gate verification. The
[implementation scope](global-values.md) describes the supported descriptor,
assignment, deletion and declaration paths and their limits. This is a bounded
regression checkpoint; general global-object/Window semantics and full Test262
conformance remain incomplete.

## Subsequent URI globals

The [URI increment](test262-uri.md) adds both modes of the two URI-global sources,
producing **42 passed / six missing-Date failures / 40 unsupported**, with all
64 controls. The [full comparison](test262-global-values-uri.json) records four
gains without any other observation changes. Actual recording and gate checks
protect them in the [current baseline](test262-global-values-current.json); the
checkpoint link above retains earlier evidence.
