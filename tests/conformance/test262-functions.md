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
requiring intrinsic `SyntaxError`. Async/generator methods, rest parameters and
other unimplemented features remain in the inventory and denominator.

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

This selection is broader than default-parameter syntax and deliberately keeps
unrelated function tests. It is not full Test262, Web Platform Tests, or web
compatibility coverage. The vendored data retains its
[upstream license](../upstream/test262-functions/LICENSE).
