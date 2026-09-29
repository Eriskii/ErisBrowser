# Pinned Test262 identifier and whitespace selection

The [constructor-policy follow-up](constructor-policy.md) admits Reflect call/construct
and `new.target` metadata. Current results: **509 passed, 154 unsupported, 6 resource**.
The inventory and source bytes are unchanged. Measurements and policy descriptions
below retain the history of earlier checkpoints.

This separate profile retains every direct JavaScript source in two complete
[Test262 language directories](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/language)
at revision `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`:

| Directory | Sources | Sloppy | Strict | Variants |
| --- | ---: | ---: | ---: | ---: |
| `identifiers` | 268 | 267 | 268 | 535 |
| `white-space` | 67 | 67 | 67 | 134 |
| Total | **335** | **334** | **335** | **669** |

There are no nested directories, fixture files, modules or asynchronous tests
at this pin. Only `identifiers/val-yield-strict.js` has `onlyStrict`.
The selection includes **122 parse-SyntaxError negative sources** (243 modes)
and **213 positive sources** (426 modes), without reducing either group to
anticipated passes. Every source, size and hash is retained in the
[manifest](../upstream/test262-identifiers/manifest.json).

Sources total **2,725,828 bytes**; the largest complete generated Unicode test
is **125,267 bytes**. No source requests an additional include. The import
retains unchanged `assert.js`, `sta.js`, `propertyHelper.js` and `compareArray.js`
for the existing assertion contract, together with upstream
[LICENSE](../upstream/test262-identifiers/LICENSE) and `INTERPRETING.md`.
Aggregate imported bytes are **2,771,524**. Each test was checked against the
pinned API Git blob; the runner verifies every source/harness/mode fingerprint.
No test or harness is shortened, rewritten, or removed to fit implementation
or resource limits.

Manifest SHA-256:
`b93b9e76f5231a3d342d95fe0a8cf47ca79c2a3101f2a78e4d5bb564c4d06ea8`.

```sh
python3 tools/import_test262.py --profile identifiers
python3 tools/test262_conformance.py --profile identifiers
python3 -m unittest discover -s tools -p 'test_test262_conformance.py'
```

## Isolated execution policy

The policy is the existing core feature set plus **`u180e` only**. It does not
invent an `identifiers` metadata feature. Core contains `String.fromCodePoint`,
`arrow-function`, `for-in-order`, and `well-formed-json-stringify`.

Sixty sources declare both `class` and `class-fields-private`, retaining
**120 metadata-unsupported modes**. The numeric-separator source retains two
more unsupported modes. The remaining **547 modes** are admitted for execution,
not presumed passing. Two whitespace sources declare u180e; their actual
execution remains visible. Dynamic eval in legacy whitespace tests is also
executed and reported through the existing runtime limitation rather than a
new source exclusion. General class/private syntax, numeric separators, Proxy,
Reflect and eval are not added by this profile.

All eight earlier profiles, policies, corpora and preflight definitions remain
unchanged, including array-sort's retained resource outcomes. An independent
comparison preserved **2,424 prior case fingerprints** and **440 prior preflight
fingerprints**, plus manifest hashes, fixtures and feature policies.
The standard three-second process policy has SHA-256:
`bdbeba5b53e7ef315e7f06e78fe8c66247cd7dd9ae2545ac5ec25749b94d95d5`.
The [shared runner contract](test262.md) applies.

## Positive controls and negative syntax checks

The profile keeps **32 unchanged core preflights**, adds eight
positive/deliberately incorrect pairs in both execution modes (**32 variants**),
and adds twelve explicit syntax-rejection cases in both modes (**24 variants**).
All **88 preflights** must verify before a healthy baseline can be recorded.

The positive pairs cover:

- Raw/escaped name aliasing, both escape forms and long leading-zero escapes.
- Other_ID_Start and ID-versus-XID distinctions.
- Combining marks, continuation digits and connector punctuation.
- U+200C/U+200D join controls and supplementary identifier characters.
- Distinct precomposed and combining spellings, without normalization.
- Escaped reserved property names, without converting them into keyword terminals.
- BOM/Unicode whitespace and literal keyword controls.

Each deliberately wrong partner changes only the final expected value after
the same setup. It must raise the unchanged assertion harness's Test262Error;
SyntaxError from an old lexer or another unrelated error does not satisfy it.

The syntax checks cover escaped reserved bindings, escaped grammar/literal/this
terminals, initial combining marks/digits, lone and adjacent surrogate escapes,
empty/out-of-range braced escapes, numeric adjacency and raw U+0085. They use
parse-negative metadata and require the actual intrinsic SyntaxError identity
in the parse phase, without eval or runtime source rewriting.

**A syntax result alone does not verify a lexer.** Every new syntax preflight
also requires a separately executed positive lexer control in the same mode.
The full report retains the raw negative observation and explicitly embeds its
prerequisite name, mode, source/case fingerprint, expected outcome, verification
and actual result. The overall verification is false if either side fails.
The old profiles' report shapes and fingerprints are unchanged. Upstream corpus
negative outcomes themselves are never rewritten by this preflight rule.

These checks follow [ECMAScript identifier names](https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html#sec-names-and-keywords),
[identifier early errors](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-identifiers-static-semantics-early-errors)
and [whitespace](https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html#sec-white-space).
The independent [Unicode data checkpoint](unicode-identifiers-data.md) records
the exact official 18.0.0 properties and exhaustive scalar validation; this
Test262 pin remains unchanged and is not replaced by generated local tests.

Six focused Python groups cover exact directory inventory hashes and counts,
all modes and negative metadata, unchanged source/helper bytes, isolated
metadata policy, runtime eval retention, old core checks, wrong-error detection,
raw Unicode test characters, same-mode prerequisite identity/result preservation,
and corrupted/incomplete importer rejection in both directories. The focused
Python module has **51 passing tests** at this checkpoint.

## Frozen initial measurement

The [complete initial report](test262-identifiers-initial.json) measures the
preserved adapter from checkpoint `eee65867ad88de6417ce24052641b43ac593abef`.

| Outcome | Positive modes | Negative modes | All modes |
| --- | ---: | ---: | ---: |
| Passed | 140 | 237 | **377** |
| Failed | 134 | 4 | **138** |
| Unsupported | 152 | 2 | **154** |
| Resource, timeout or adapter error | 0 | 0 | **0** |

All **32 core preflights** verify; all **56 new preflights** remain unverified,
for **32/88** overall. The 16 new positive variants reject valid source, and
the 16 wrong partners also encounter lexer SyntaxError instead of the expected
Test262Error. Of 24 syntax checks, 22 have raw expected-SyntaxError passes, but
remain unverified because their separately executed positive lexer control
fails. The two raw U+0085 checks fail directly because the old lexer accepts it
as whitespace. These raw observations and failed prerequisites are retained.
An actual initial `--record-baseline` request was refused; **no healthy initial
baseline exists**.

The initial corpus failures consist of:

| Variants | Actual observation |
| ---: | --- |
| 98 | Parse SyntaxError on backslash in valid escaped-name sources |
| 4 | Parse SyntaxError on U+2118 in valid Other_ID_Start sources |
| 30 | Parse SyntaxError on valid continuation marks/join controls |
| 2 | Parse SyntaxError on the BOM in a positive whitespace source |
| 4 | Parse completes for negative `vertical-tilde-start.js` and `vertical-tilde-continue.js`, both modes |

All SyntaxError observations carry intrinsic identity `SyntaxError`; the
unexpected parse-completion observations have no error identity. The full
report groups every exact diagnostic string with its source/mode IDs. The
154 unsupported variants comprise **122 declared-feature exclusions** and
**32 actual runtime UnsupportedFeature observations**, message
`dynamic eval is not implemented`.

Of the 237 retained negative corpus passes, **162** currently arise from blanket
backslash rejection. They remain passes under unchanged upstream negative
metadata, but do not imply correctly decoded identifier semantics. This is why
the separate positive-control prerequisite is required for measurement health.
No such raw pass is removed or converted into an implementation gain.

Initial adapter SHA-256:
`c039eb6be9360342f9cec2fd8da08cd0efc4245a04370bca0c114e1ff812825c`.
Source-input digest:
`dce62370c4dbfa806975d8c207bc38ad955997f89d653357848bc2e1b36a148c`.
Sanitized full initial-report SHA-256:
`2c9eff3a16b7fbc8df599fee5aac0e63e97c7a9dd4353446b38370db9749968b`.

A later candidate must preserve every one of the **669 case identities** and
**88 preflight identities**, including prerequisite identity/mode mapping,
manifest and policy. All failures, unsupported tests and resource outcomes must
remain visible. Any resource stop, timeout, adapter error or failed preflight
prevents a healthy baseline or passing conformance gate. The first candidate observation is retained separately below.

## Preliminary implementation measurement

The [complete preliminary report](test262-identifiers-preliminary.json) preserves
an initial candidate before a review of possible duplicate identifier-lookup
work. It is not a healthy baseline and is not presented as a no-regression result.

| Outcome | Initial | Preliminary candidate |
| --- | ---: | ---: |
| Passed | 377 | **483** |
| Failed | 138 | **0** |
| Unsupported | 154 | **154** |
| Resource stop | 0 | **32** |
| Timeout or adapter error | 0 | **0** |
| Verified preflights | 32/88 | **88/88** |

All 669 source/harness/mode fingerprints, 88 preflight fingerprints and their
prerequisite mappings, manifest and policy are unchanged. There are **122 newly
passing variants**, **361 retained passes**, and **16 prior passes lost to a
resource stop**. Another 16 formerly failing escaped-source variants also stop
on resources. All 154 unsupported observations are unchanged.

The 32 resource outcomes are both modes of raw and escaped
`start-unicode-5.2.0`, `8.0.0`, `9.0.0`, `10.0.0`, `13.0.0`, `15.0.0`, `16.0.0`
and `17.0.0` sources. Their actual observation is parse-phase ResourceLimit,
message `regular expression work limit exceeded`; this is the shared compile
work diagnostic. The raw-source variants supplied the 16 earlier passes.
Every exact lost case and transition is listed in the report. Sources and
budgets remain unchanged; no large-file exclusion is added.

An actual `--record-baseline` invocation returned 1 and created no baseline,
with identical case and preflight observations to the initial candidate run.
All preflights passing does not override the resource-health requirement.
A later measured optimization must preserve these preliminary observations
and the same complete inventory rather than replacing this evidence silently.

Preliminary adapter SHA-256:
`2e0f03085c4608c642e864d5b1bf63d784119eb3a37836e691c979740fe3615a`.
Source-input digest:
`947c129d5b223bd9c1736abf4d10a964e73584ef6bc75b411cda7a075cdb84eb`.
Runtime source SHA-256:
`c3bb12d6f8fef3e30b75572b3b030c40d18842083b6fc128bc9f1fc8f28425dc`.
Sanitized preliminary-report SHA-256:
`0d3796149d1240066ae71c598ca5a4aac73080b3faa3aa2688cca63f65e9be28`.

## Optimized final measurement

The [complete final report](test262-identifiers-latest.json) retains every
observation and compares it with both reports above. Exact Unicode membership
now uses two fixed packed-table reads; the lexer avoids its duplicate raw
first-character lookup, the parser borrows its source, and invalid-token
diagnostics use precharged shared ownership. These reduce actual lookup, copy
and token-storage costs. Runtime quotas, source files, harnesses, policies and
preflight definitions are unchanged.

| Outcome | Initial | Preliminary | Final |
| --- | ---: | ---: | ---: |
| Passed | 377 | 483 | **507** |
| Failed | 138 | 0 | **0** |
| Unsupported | 154 | 154 | **154** |
| Resource stop | 0 | 32 | **8** |
| Timeout or adapter error | 0 | 0 | **0** |
| Verified preflights | 32/88 | 88/88 | **88/88** |

All **669 case identities**, **88 preflight identities**, positive prerequisite
identity/mode mappings, manifest and policy match both preserved measurements.
All **377 initial passes** are retained, with **130 newly passing variants**
and no prior-pass loss. All **483 preliminary passes** are retained, with
**24 additional passes**. Every one of the **154 unsupported observations** is
unchanged, including its actual runtime diagnostic where execution occurred.

The following table explicitly accounts for all 16 raw-source pass losses in
the preliminary result. Paths have the prefix `test/language/identifiers/`;
each cell records both independently executed modes.

| Source | Initial sloppy / strict | Preliminary sloppy / strict | Final sloppy / strict |
| --- | --- | --- | --- |
| `start-unicode-5.2.0.js` | passed / passed | resource / resource | passed / passed |
| `start-unicode-8.0.0.js` | passed / passed | resource / resource | passed / passed |
| `start-unicode-9.0.0.js` | passed / passed | resource / resource | passed / passed |
| `start-unicode-10.0.0.js` | passed / passed | resource / resource | passed / passed |
| `start-unicode-13.0.0.js` | passed / passed | resource / resource | passed / passed |
| `start-unicode-15.0.0.js` | passed / passed | resource / resource | passed / passed |
| `start-unicode-16.0.0.js` | passed / passed | resource / resource | passed / passed |
| `start-unicode-17.0.0.js` | passed / passed | resource / resource | passed / passed |

Eight additional escaped-source modes recover from the preliminary resource
stop: `start-unicode-{13.0.0,15.0.0,16.0.0,17.0.0}-escaped.js`, both modes.
The remaining **eight resource outcomes** are both modes of
`start-unicode-{5.2.0,8.0.0,9.0.0,10.0.0}-escaped.js`. Each reports parse-phase
`ResourceLimit`, message `regular expression work limit exceeded`, from the
shared compile work budget. Their complete sources remain in the inventory.
Final positive-mode outcomes are 266 passed, 152 unsupported and eight resource;
negative modes are 241 passed and two unsupported.

An actual final `--record-baseline` invocation returned 1 and created no
baseline. **The eight retained resource stops prevent a healthy baseline or
passing conformance gate**, despite all preflights verifying. Neither earlier
report was rewritten or replaced, and this selection does not claim full
Test262 or ECMAScript conformance.

Final adapter SHA-256:
`b7f77cbbc11f14297b7b34716668a10c0522be62562a33a07756dcc89f17d12f`.
Source-input digest:
`aa1d611fc199bdff6eab451307c2166ca0fe71ab3b8c39dd1bf0182ca5431cb7`.
Runtime source SHA-256:
`8bf71b8cb985e48dfb89a40db2c001d22d570cebcb69bd6648667581203a625b`.
Packed Unicode module SHA-256:
`49187f73fcbae9d685e7848c6a412315c247a7fbfd4f3919bd5c08f8a6a7b552`.
Sanitized full final-report SHA-256:
`ca6d5282f979ef7308263f90494efe5019bfeb6f1379e0cd49f065073a4922f5`.

## Flat executable ownership and paged-token checkpoint

The subsequent [complete report](test262-identifiers-flat-code.json) preserves
all 669 case identities, policy and 88 preflight observations. All preflights
verify. Both modes of `start-unicode-5.2.0-escaped.js` now pass, changing the
counts to **509 passed / 154 unsupported / six compile resource stops**. Every
other observation is identical to the previous report. Paged tokens and bounded
flat lowering remove actual prefix relocation and redundant lowering work without
raising limits or changing the source. The profile remains a nonpassing
observation with no healthy baseline. See [implementation and evidence](flat-code.md).
