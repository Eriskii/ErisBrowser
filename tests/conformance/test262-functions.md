# Pinned Test262 function selection

This separate profile retains every direct `.js` file from four directories
at the same pinned Test262 revision as the existing selections:
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`.

| Directory beneath `test/language/` | Files |
| --- | ---: |
| `expressions/function` | 69 |
| `statements/function` | 256 |
| `expressions/arrow-function` | 55 |
| `expressions/object/method-definition` | 283 |

All **663 source files** and five unchanged harness files are present. Metadata
requires **1,131 variants**: 616 sloppy and 515 strict. There are no standalone
fixture files; 179 source tests produce 313 parse-negative variants, all
requiring intrinsic `SyntaxError`. Async/generator methods, rest-parameter
variants and other tests outside this profile's admitted features remain in the
inventory and denominator.

The importer verifies pinned Git blob identities and records paths, byte sizes
and SHA-256 values. The runner verifies complete directory inventories, hashes,
source/harness/mode identities and path safety before execution. Its metadata
reader now accepts common top-level YAML indentation; original source bytes,
including the indented description in `13.2-30-s.js`, are not rewritten.
The manifest SHA-256 is
`15281ebe6e6c980c547f113563df7d2b5d346ac0db5c9c4aca75dbb6fc60b330`.

```sh
cargo build --locked --release --bin eris-js
python3 tools/test262_conformance.py --profile functions
```

This profile permits execution of declared default parameters, object methods,
computed property names and trailing function commas, in addition to the base
String/JSON feature policy. Permission to execute a test is not a support claim:
the interpreter's actual result determines its outcome. Other declared features,
async completion, modules, host hooks and unavailable execution modes remain
explicitly unsupported. Existing profile policies and baselines are unchanged.
The [runner contract](test262.md) describes isolated processes, bounds,
unchanged assertions, negative error identity and outcome categories.

## Identifier checkpoint

The [identifier comparison](test262-functions-identifiers.json) records **509
passed / 620 unsupported / two resource stops**, with all 48 preflights verified.
Both modes of `S13_A7_T1.js`, `S14_A5_T1.js` and `S14_A5_T2.js` newly pass after
Unicode escape decoding; no earlier pass is lost. All 1,131 identities and the
existing policy match the [sort checkpoint](test262-functions-sort.json).
There are no remaining ordinary failures in this selection, but both unchanged
32-level nested-function tests still exceed the parser limit. No healthy
functions baseline is recorded. The final adapter SHA-256 is
`b7f77cbbc11f14297b7b34716668a10c0522be62562a33a07756dcc89f17d12f`.

## Initial measurement

The [complete initial report](test262-functions-initial.json) records a frozen
`eris-js` build from engine checkpoint `3491721`, before default-parameter work.
Its SHA-256 is
`15902a07b1ef9c866942c9418264a3fee8e5d045ef64ccef8b60a83794f75edc`.
All 32 existing assertion/strict-semantics preflights pass.

| Outcome | Variants |
| --- | ---: |
| Passed | 350 |
| Unsupported | 718 |
| Failed | 61 |
| Resource stop | 2 |

Both resource outcomes are `statements/function/S13.2.1_A1_T1.js`, whose 32
nested immediately invoked functions exceed the current parser nesting budget.
The runner correctly refuses to record a healthy baseline. This report is an
initial measurement, not an accepted regression baseline or a passing CI gate.
Its failure inventory is retained without modifying tests or relaxing budgets.

## Default-parameter checkpoint

The [updated complete report](test262-functions-latest.json) records
**490 passed / 620 unsupported / 19 failed / two resource stops**. Comparing
identical source/harness/mode fingerprints and the same execution policy gives
**140 newly passing variants and no lost passes**. The two 32-nested-function
resource stops remain unchanged, so this still is not a healthy baseline.

There are now **48 passing preflights**: the original 32 plus 16 positive and
deliberately incorrect default-value, TDZ, arguments and scope checks across
both modes. These execute the unchanged upstream assertion helpers. Deliberate
mismatches must throw `Test262Error`; disabling assertions cannot pass the
preflight. No upstream file or previous profile policy changed.

The release adapter SHA-256 is
`77003eefd7a91dccb2b418e0d59bc234091edbfb1a7bb6a217de63cd1ece8b69`.
Its source-input digest is
`e6d4d857c922db8942abaec7cf230454b3487313f1121dbc6e7babbe0385bcd6`.
The [implementation scope](default-parameters.md) records remaining syntax and
host limits, including the next unchanged WPT harness blocker.

## Prototype-membership checkpoint

The [later complete report](test262-functions-prototypes.json) records
**500 passed / 620 unsupported / nine failed / two resource stops** after
implementing `Object.prototype.isPrototypeOf`. Ten variants from five existing
files now pass: function-valued constructor prototypes, primitive constructor
prototype fallback and Function.prototype ancestry. The comparison preserves
all source/harness/mode identities and the same execution policy, with no lost
passes. All 48 existing preflights still pass.

The unchanged two resource stops still prevent a healthy functions baseline.
The nine remaining failures include missing Array sorting, identifier escapes
and a host `self` binding defect; none was filtered or relabeled to improve
these counts. The separate [method inventory](test262-is-prototype-of.md) adds
its own complete-directory measurement and assertion preflight.

## Window.self checkpoint

The [complete Window.self report](test262-functions-self.json) records
**501 passed / 620 unsupported / eight failed / two resource stops**. Correct
replacement of the host `self` accessor allows the strict variant of
`statements/function/13.2-30-s.js` to run its existing bound-function assertions.
Its sloppy variant already passed because the test did not check receiver
identity; separate runtime regressions verify replacement in both modes.

All 1,131 source/harness/mode identities, the execution policy and 48 verified
preflights match the prototype checkpoint. One variant newly passes and none
lose a pass. The eight remaining failures concern Array sorting and escaped
identifiers; the two parser stops remain. No test source, assertion helper,
metadata or admission policy was changed, and no healthy functions baseline
was recorded. [The implementation scope](window-self.md) includes remaining
Window and global-property limitations.

Release adapter SHA-256:
`073e46ca2273fb55e458432d2b17c4edac6b6f6110f61a83d79b3eaeb2bc6d8b`.
Source-input digest:
`5e4cfb6d251433ae56cc1900fd3380dc19e8b759de5bc2a162a45ac3efddcab2`.

## Array sorting checkpoint

The [complete sorting report](test262-functions-sort.json) records
**503 passed / 620 unsupported / six failed / two resource stops**. Both modes
of `statements/function/S13.2.1_A5_T1.js` now pass: its unchanged comparator
closure reaches the custom Array.prototype.sort implementation. All 1,131
source/harness/mode identities, the execution policy and 48 verified preflights
match the previous report. There are two gained passes and no lost passes.
The intervening global-value checkpoint retained the prior 501-pass result.

The six remaining failures exercise identifier escapes; both original nested
function parser stops remain. No healthy functions baseline is recorded.
The [sort implementation](array-sort.md) and separate
[complete method inventory](test262-array-sort.md) document ordinary receiver
support and the independent descriptor, reduce and resource limitations.

Release adapter SHA-256:
`c039eb6be9360342f9cec2fd8da08cd0efc4245a04370bca0c114e1ff812825c`.
Source-input digest:
`dce62370c4dbfa806975d8c207bc38ad955997f89d653357848bc2e1b36a148c`.

This selection is broader than default-parameter syntax and deliberately keeps
unrelated function tests. It is not full Test262, Web Platform Tests, or web
compatibility coverage. The vendored data retains its
[upstream license](../upstream/test262-functions/LICENSE).
