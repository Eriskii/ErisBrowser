# Pinned Test262 logical-assignment inventory

The [constructor-policy follow-up](constructor-policy.md) admits Reflect call/construct
and `new.target` metadata. Current results: **78 passed, 6 failed, 48 unsupported**.
The inventory and source bytes are unchanged. Measurements and policy descriptions
below retain the history of earlier checkpoints.

Current update: [Symbol primitive conversion](symbols.md) adds six passing
modes. The complete inventory now has **78 passed / 6 failed / 48 unsupported**,
with all 104 controls verified. Class syntax accounts for the remaining failures.
The passing-case baseline is updated; the original measurements below remain
historical evidence.

The profile retains every direct JavaScript file in
[language/expressions/logical-assignment](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/language/expressions/logical-assignment)
at revision `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`: **78 sources, 86,430
source bytes and 132 mode cases**. The directory is flat. Modes comprise 54
sloppy and 78 strict variants, with 24 onlyStrict sources. There are 18 negative
parse/SyntaxError variants. Original LICENSE, INTERPRETING.md, sources and four
unchanged helpers (assert, sta, propertyHelper and compareArray) total
**132,126 bytes**. The latter two helpers support the core assertion controls.

The [manifest](../upstream/test262-logical-assignment/manifest.json) SHA-256 is
`a83f71b8a6b6a1523951d73a4711f024cf77ad3bf90e6698bc6224e7bfabf1a3`.
The profile enables logical-assignment-operators alongside the unchanged core
features, both before and after implementation. **48 modes remain unsupported
by metadata**: 42 private-field variants and six BigInt variants. Other profiles'
feature sets are unchanged. No sources, metadata or expectations are rewritten.

The 32 core controls are joined by six positive/wrong pairs for each operator
in both modes: **104 preflights**. Each added control first performs a successful
canonical assignment. Pairs cover result identity, skipped references, taken
references, abrupt effects, readonly bindings and function names. Wrong partners
change only the final assertion and require an actual Test262Error. A parser
error cannot satisfy them.

Six Python groups check full inventory/helpers, feature admission, guarded
pairs, assertion identity, damaged/incomplete import rejection and prior
contracts. Replay against the runner at `789165b` verifies all fifteen previous
profiles retain **5,626 case and 1,112 preflight fingerprints**, plus manifests,
fixtures and feature sets. Canonical digest:
`95e61080fc9b6ba38004a7d1fea980d1d6a219acf7915b52b102a86c9dbece3e`.

| Outcome | Before 789165b | Logical assignment |
| --- | ---: | ---: |
| Passed | 18 | **72** |
| Failed | 66 | **12** |
| Unsupported | 48 | **48** |
| Resource, harness error, timeout or adapter error | 0 | **0** |
| Verified preflights | 32/104 | **104/104** |

The [initial report](test262-logical-assignment-initial.json) preserves parser
failures and the failed guarded controls. The
[candidate report](test262-logical-assignment-latest.json) adds **54 passes with
no losses**. Eighteen already-passing negative cases now report invalid-target
or strict-binding diagnostics instead of a generic expected-expression error.
The twelve remaining failures move past operator recognition: six class-name
modes report invalid class syntax, and six general operator modes reach missing
Symbol and report ReferenceError. These tests do not declare those prerequisites
in feature metadata; their failures remain failures. All 84 diagnostic/status
changes are retained in the full comparison.

Actual baseline recording and a subsequent CLI gate both return zero and
reproduce every observation. The
[current baseline](test262-logical-assignment-current.json) retains all twelve
failures and 48 unsupported outcomes while protecting passes. A plain run
returns one. The gate does not imply complete logical-assignment conformance.
Every case observation and preflight result in the fifteen earlier profiles is
identical. Existing resource stops in other profiles remain nonpassing.

```sh
python3 tools/import_test262.py --profile logical-assignment
python3 tools/test262_conformance.py --profile logical-assignment
python3 tools/test262_conformance.py --profile logical-assignment --baseline tests/conformance/test262-logical-assignment-current.json
```

| Artifact | SHA-256 |
| --- | --- |
| Before adapter | `c9e47a8245eaa6e14c60f1ae0874213bdef2b00a6dacb4648c845155fdb136a3` |
| Candidate adapter | `e9d33d19a55217d159ab6e337e6f4af21a69651a7325b762247505f7c3b5cdf4` |
| Candidate source input | `8c452cdea5acad7f74b0af769ec0686ed0f6eeda2ec67f47464a0230537dfbed` |
| Initial report | `a86289aeb508dbaa660712a1a45c7f9ffa1ead80d7c190eed3f1e62ad20418c6` |
| Candidate report | `b5433fa11261b61f4d4c5fe7218d3cccd3d1b90847d289fc64a002236efe27cb` |
| Current baseline | `ad49bf074436d4b37a00ad41064f45f477e8aad81ae15fc4df3eadcb14e7742a` |

The [implementation scope](logical-assignment.md) records reference ordering,
short-circuit effects, function-name inference and bounded execution.
