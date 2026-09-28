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

This selection is broader than default-parameter syntax and deliberately keeps
unrelated function tests. It is not full Test262, Web Platform Tests, or web
compatibility coverage. The vendored data retains its
[upstream license](../upstream/test262-functions/LICENSE).
