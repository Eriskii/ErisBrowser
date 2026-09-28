# Pinned Test262 reduce and reduceRight selection

This isolated profile imports both complete direct directories at Test262
revision `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`:

| Directory | Sources | Sloppy | Strict | Variants | Source bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| [Array/prototype/reduce](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/Array/prototype/reduce) | 260 | 260 | 257 | 517 | 176,889 |
| [Array/prototype/reduceRight](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/Array/prototype/reduceRight) | 260 | 260 | 257 | 517 | 187,722 |
| Total | **520** | **520** | **514** | **1,034** | **364,611** |

There are no nested directories, fixture files, modules, async tests or
frontmatter negative tests at this pin. Each directory has three `noStrict`
sources and otherwise uses both modes. The largest source is 3,808 bytes.
Every source is checked against the pinned API Git blob and retained unchanged;
no selection is based on anticipated passing behavior.

The [manifest](../upstream/test262-array-reduce/manifest.json) retains all source
paths, sizes and hashes plus the seven unchanged harnesses: `assert.js`,
`sta.js`, `compareArray.js`, `propertyHelper.js`, `isConstructor.js`,
`resizableArrayBufferUtils.js` and `testTypedArray.js`. Upstream
[LICENSE](../upstream/test262-array-reduce/LICENSE) and `INTERPRETING.md` are also
retained. Aggregate imported content is **431,206 bytes**.

Manifest SHA-256:
`77ee716188d99fdba64adbe151ef4efd915fbaa6345b2b0ec8fe3550efb5ec24`.
Canonical sorted directory-inventory SHA-256:
`b7200a203682b4c2cd82695de080d24e7bfdc98f36113930bd56d719850cc60e`.

```sh
python3 tools/import_test262.py --profile array-reduce
python3 tools/test262_conformance.py --profile array-reduce
python3 tools/test262_conformance.py --profile array-reduce \
  --baseline tests/conformance/test262-array-reduce-current.json
python3 -m unittest discover -s tools -p 'test_test262_conformance.py'
```

## Unchanged admission and retained prerequisites

Admission uses the existing **core feature set only**: `arrow-function`,
`String.fromCodePoint`, `well-formed-json-stringify` and `for-in-order`.
Per directory, five sources / ten modes declare unavailable `Reflect.construct`
or `resizable-arraybuffer` features; one resizable source also declares
`TypedArray`. These **20 metadata-unsupported modes** remain in the inventory.
The remaining **1,014 modes** are admitted for actual execution, not presumed
passing. Proxy, Symbol, typed arrays, resizable buffers and Reflect support are
not added by this profile.

Four complete sources / eight modes use `Date`. The unchanged
`reduceRight/length-near-integer-limit.js` uses `Number.MAX_SAFE_INTEGER` and
examines descending indices near the safe-integer limit. The later Number
checkpoint below resolves that constant prerequisite. These sources have no
new exclusion; missing globals and actual execution outcomes remain visible.
Array indexed/length descriptor definitions also retain their existing
unsupported outcome; ordinary-object descriptor tests are executed. Sources
are neither rewritten nor shortened to remove prerequisites or resource work.

All nine previous corpora, policies and preflights remain unchanged. A pinned
contract comparison preserves **3,093 previous case fingerprints** and **528
previous preflight fingerprints**, plus manifest hashes, fixtures and feature
sets. Its canonical SHA-256 is
`00bc0a2ee53cc28fd1eda61af5f8cc98cf83c910a1d65680987ef914b0a47177`.
The new profile's standard three-second runner policy SHA-256 is
`81b61b3419f5a73997b1f29d352177e4847797cf9c8d8fcc353a52c0acb6be2e`.
The [shared runner contract](test262.md) applies.

## Assertion and method-availability preflights

The profile keeps **32 unchanged core preflights** and adds twelve
positive/deliberately wrong pairs for each method in both execution modes:
**96 new variants, 128 total**. Each pair shares its complete setup and changes
only its final assertion. A wrong result must report the unchanged assertion
harness's `Test262Error`; TypeError, ReferenceError or SyntaxError does not
verify the wrong partner.

Every new variant first checks that its selected method is callable and that a
successful reduction returns 3. These checks occur outside and before any
`assert.throws` callback. This prevents missing `.reduce`, `.reduceRight` or
`.call` behavior from masquerading as correct error validation. Both positive
and deliberately wrong variants must verify before the profile is healthy.

The twelve paired groups cover:

- Directional callback order and reduction with and without an initial value.
- Omitted versus explicit undefined initial values, empty receivers and exact
  object accumulator identity.
- Holes versus present undefined, including all-hole rejection without an
  initial accumulator.
- Length getter/conversion before callback validation and no indexed read for
  a noncallable callback.
- Inherited, nonenumerable indexed properties and accumulator search.
- Live getter changes to future presence and values.
- Captured length despite callback mutation of the length and extra indices.
- Exact indexed-getter/callback abrupt identity and preserved earlier effects.
- Four callback arguments, receiver identity, strict/sloppy this behavior and
  bound callback receivers.
- Generic boxed strings and stable boxed receiver identity.
- Saved aliases through call/apply/bind after method replacement/deletion.
- Method/name/length descriptors, prototype identity and nonconstructability.

Descriptor preflights use unchanged `propertyHelper.js` with `restore:true`
before subsequent metadata reads. No assertion helper or upstream source is
modified. These checks follow the current primary algorithms for
[reduce](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.reduce)
and [reduceRight](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.reduceright).

Six focused Python groups check complete inventories/modes/helpers, pinned
source digests, isolated policy and retained dependencies, failed assertion
identity, guarded paired setup, metadata restoration, corrupted/incomplete
imports and preservation of every previous profile contract. The focused
module has **57 passing tests** at this checkpoint.

## Later Number static builtin checkpoint

The [complete later comparison](test262-array-reduce-number-statics.json)
preserves all 1,034 case and 128 preflight identities. Number.MAX_SAFE_INTEGER
enables `reduceRight/length-near-integer-limit.js` in both modes: **848 passed /
16 failed / 170 unsupported**, with no lost passes, resources, timeouts or adapter
errors. All 128 preflights verify. The remaining failures still concern Date and
Math/JSON tags. Baseline recording and its gate both pass with identical results;
the [current baseline](test262-array-reduce-current.json) now protects the two gains.

Candidate adapter SHA-256:
`f6ee32dffda7d56c3197cac56586cb9442134f141af1ed69528baf15d8e23675`.
Current baseline SHA-256:
`0eeff1f9bdccb1a035ed2d5e888390352f1f9a2704511a4f979085a26ed79c76`.
The original reduction measurements below remain historical evidence.

## Frozen initial measurement

The [complete initial report](test262-array-reduce-initial.json) records the
preserved adapter from checkpoint `9d5add8528d6b4636b8a9f474dfb7cd4dc10549f`.

| Outcome | reduce | reduceRight | Total |
| --- | ---: | ---: | ---: |
| Passed | 42 | 42 | **84** |
| Failed | 391 | 389 | **780** |
| Unsupported | 84 | 86 | **170** |
| Resource, timeout or adapter error | 0 | 0 | **0** |

**80/128 preflights individually verify**: all 32 core checks and the 48 wrong
partners. All 48 guarded positive variants fail method availability. The wrong
partners encounter `Test262Error` at that same availability assertion, so their
individual verification is not evidence of implemented reduction semantics.
An actual `--record-baseline` request returned 1 and created no baseline.
**No healthy initial baseline exists.**

The 84 raw upstream passes remain recorded unchanged. Missing-method lookup or
call TypeError can satisfy unchanged exception expectations, so these results
must be read alongside the failed positive availability controls. The profile
does not remove such cases or turn their existing passes into future gains.

All initial failures and unsupported observations are retained in the full
report, grouped by exact identity/message and source/mode IDs:

| Variants | Status / actual runtime observation |
| ---: | --- |
| 476 | Failed TypeError: `cannot access property of null or undefined` |
| 214 | Failed TypeError: `value is not callable` |
| 74 | Failed Test262Error: `uncaught exception: [object Object]` |
| 8 | Failed TypeError: `cannot convert null or undefined to object` |
| 8 | Failed ReferenceError: `'Date' is not defined` |
| 150 | Runtime UnsupportedFeature: `array indexed/length descriptor mutation is not implemented` |
| 20 | Declared-feature unsupported: Reflect.construct or resizable/typed-array metadata |

The eight Date failures are a separate prerequisite from missing reduction
methods. Other independent prerequisites remain visible in the complete source
inventory even when an earlier missing-method failure prevents reaching them.

Initial adapter SHA-256:
`b7f77cbbc11f14297b7b34716668a10c0522be62562a33a07756dcc89f17d12f`.
Source-input digest:
`aa1d611fc199bdff6eab451307c2166ca0fe71ab3b8c39dd1bf0182ca5431cb7`.
Sanitized full initial-report SHA-256:
`3ef353618b171e3376ca7185c51d2dec429d66ba0988d812d923d8bf389ac491`.

A candidate must preserve all **1,034 source/harness/mode identities** and
**128 preflight identities**, along with manifest and policy. Every failure,
unsupported test and resource stop remains recorded. Failed preflights,
resource termination, timeouts or adapter errors prevent a healthy baseline;
this selection does not claim full Test262 or ECMAScript conformance.

## Frozen candidate and regression baseline

The [complete candidate report](test262-array-reduce-latest.json) preserves
every outcome and comparison with the initial adapter. All **1,034
source/harness/mode fingerprints**, **128 preflight fingerprints**, manifest
and policy are unchanged.

| Outcome | Initial | Candidate |
| --- | ---: | ---: |
| Passed | 84 | **846** |
| Failed | 780 | **18** |
| Unsupported | 170 | **170** |
| Resource, timeout or adapter error | 0 | **0** |
| Verified preflights | 80/128 | **128/128** |

The candidate retains all **84 initial raw passes** and adds **762 passing
variants**, with zero prior-pass losses. Every one of the **170 unsupported
observations** is unchanged: 20 declared-feature exclusions and 150 actual
Array indexed/length descriptor limitations. Directional totals are:

| Method | Passed | Failed | Unsupported |
| --- | ---: | ---: | ---: |
| reduce | 425 | 8 | 84 |
| reduceRight | 421 | 10 | 86 |

All 48 new positive controls now succeed, and their 48 wrong partners report
Test262Error after the successful shared setup. These controls replace the
initial availability failure as evidence for the selected method semantics;
they do not erase or reinterpret any initial corpus observation.

The 18 remaining failed variants retain these exact diagnostics and sources.
Every source below runs in both sloppy and strict modes; paths are relative to
`test/built-ins/Array/prototype/`:

| Variants | Sources | Actual observation and prerequisite |
| ---: | --- | --- |
| 4 | `reduce/15.4.4.21-1-10.js`, `reduceRight/15.4.4.22-1-10.js` | Runtime Test262Error, `uncaught exception: [object Object]`; callback expects Math's `[object Math]` tag |
| 4 | `reduce/15.4.4.21-1-13.js`, `reduceRight/15.4.4.22-1-13.js` | Runtime Test262Error, `uncaught exception: [object Object]`; callback expects JSON's `[object JSON]` tag |
| 8 | `reduce/15.4.4.21-1-11.js`, `reduce/15.4.4.21-9-c-ii-31.js`, `reduceRight/15.4.4.22-1-11.js`, `reduceRight/15.4.4.22-9-c-ii-31.js` | Runtime ReferenceError, `'Date' is not defined` |
| 2 | `reduceRight/length-near-integer-limit.js` | Runtime Test262Error, `uncaught exception: [object Object]`; missing `Number.MAX_SAFE_INTEGER` prevents the intended length setup |

Independent bounded controls confirm that `Object.prototype.toString.call`
currently returns `[object Object]` for both Math and JSON, and that
`Number.MAX_SAFE_INTEGER` is undefined. Their exact sources and observations
are retained in the candidate report's triage section. These are existing
prerequisite gaps, not evidence that reduction traverses those ordinary
receivers incorrectly. No constants, object tags or unsupported descriptors
were added to improve this profile's count.

The runner's existing measurement-health conditions permit a regression
baseline: every preflight verifies, there are no resource stops, timeouts or
adapter errors, and no earlier pass is lost. An actual recording command
returned 0 and wrote [the original reduction baseline](https://github.com/Eriskii/ErisBrowser/blob/58c027ac3a203b3295429dbaf6a10e7d5f31c68c/tests/conformance/test262-array-reduce-current.json).
A subsequent actual `--baseline` invocation returned 0 with **zero regressions
and zero new passes**; its case/preflight observations exactly match both the
first candidate run and baseline-recording run. The baseline records **all
1,034 statuses**, including the 18 failures and 170 unsupported variants. It
is a regression checkpoint, not an all-passing conformance claim. A plain
nonbaseline invocation still returns 1 for this incomplete selection.

Candidate adapter SHA-256:
`2b0e59686ff8fb8db409c8a54b8f92df592f00668fcd9ff00da2de0b3585705a`.
Source-input digest:
`be187dd3a00025c8461dc08013ca49696231b350d5b4490ea8166c15de83a79f`.
Runtime source SHA-256:
`745171c379a51aca6b10b1f5904f8e95c29dde675b1d2d39d09f9c68041b7983`.
Sanitized full candidate-report SHA-256:
`108cb3b7315e7199e7db678687157bbd7d569740bd5961f63f297519c4d751c7`.
Original reduction baseline SHA-256:
`6d57a0d72dc750c9ec90d6e6392111df96af3612ec3765a2be1b9406a7876108`.
