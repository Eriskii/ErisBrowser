# Promise.race upstream results

The complete immediate Test262 `test/built-ins/Promise/race` directory at revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd` contains 94 sources and 188 required
sloppy/strict modes. Eris records **170 passed, 10 failed and 8 unsupported**.
All 90 harness controls verify their expected outcomes. Every selected mode stays
in the denominator; this is a bounded selection, not full Promise or ECMAScript
conformance.

The [manifest](../tests/upstream/test262-promise-race/manifest.json) binds all
94 unchanged sources, seven harness helpers and the two legal/instruction files
to the pinned Git root. Seven retained proof documents establish complete
coverage and blob identities. The selection contains 63 asynchronous sources
(126 modes) and 31 synchronous sources (62 modes). File, proof, byte and runtime
limits remain unchanged.

| Observation | Modes |
| --- | ---: |
| Passed | 170 |
| Host failure: final unhandled rejection | 10 |
| Unsupported class feature declared in metadata | 4 |
| Unsupported class syntax encountered by the parser | 4 |
| Total | 188 |

The ten host failures come from five synchronous sources:
`invoke-resolve-error-close.js`, `invoke-then-error-close.js`,
`invoke-then-get-error-close.js`, `iter-next-val-err-no-close.js` and
`iter-step-err-no-close.js`. Each mode reaches audited quiescence with one
unhandled rejection. These retain the separate `host-failure` /
`unhandled-rejection` observation; they are not relabeled as ECMAScript assertion
failures or passes.

Two class-tagged sources account for four unsupported modes. Two further sources
lack that feature declaration and reach the parser, which reports unsupported
class syntax. Their original bytes and observations remain in the inventory;
no source-text filter removes them.

The profile reuses the existing optional `test262-host` adapter, authenticated
`$DONE` completion and final-unhandled-rejection veto. Its supported feature set
matches the Promise.all profile. Four positive/mismatch pairs run in both modes
after the unchanged 74 common controls. They check descriptors and
nonconstruction, the first winner and comparison helpers, an empty race through
a finite callback checkpoint, rejection identity, and generic shared callbacks
with repeated calls and return values. Each row requires all 37 common controls
in its mode; a mismatch also requires its corresponding positive. Missing,
duplicate, malformed or unverified controls block all 188 cases and prevent
baseline recording. Deliberate mismatch failures are healthy only when their
expected error or authenticated completion observation is verified.

A source review caught a defect in the first API-control draft before execution:
the pinned property helper deletes configurable properties unless restoration
is requested. The corrected draft adds `{ restore: true }` to six calls across
the two API sources. The other six sources remain exact, and both versions and
the review are retained. The requested `compareArray.js` is an unchanged
compatibility shim; `assert.js` supplies the actual comparison functions.

All **556 Python tests pass**: the exact prior 543 names plus 13 new profile
groups. The new groups cover authentication, metadata and mode identities,
control dependencies, host capability, blocked reports and old-profile
preservation. On the parent, two groups passed and eleven reported errors
across 36 error records. Those errors exposed the absent profile or dispatch;
one `KeyError('success')` came from the old generic preflight reaching the
race-only mock lookup. No frozen expectation changed after that baseline.

The run uses the immutable release host adapter from the published
[Promise.race implementation](promise-race.md) at `4819802e`.
No Rust, runtime cap, transport or host-policy change is part of this checkpoint.
The race baseline replay reproduces all 188 case records and 90 control records
exactly. The older profiles also retain all 654 case records and 164 control
records exactly, with zero regressions or newly passing modes. Their prior
statuses remain visible in their own reports.

To run the regression gate:

```sh
cargo build --locked --release --features test262-host --bin eris-js
python3 tools/test262_conformance.py --binary target/release/eris-js \
  --profile promise-race \
  --baseline tests/conformance/test262-promise-race-current.json \
  --output artifacts/test262-promise-race.json
```

CI is configured to build the host adapter and run this gate beside the existing
Promise.all gate. The [baseline](../tests/conformance/test262-promise-race-current.json)
preserves every status and case identity. A successful gate means the controls
are healthy and no recorded pass regressed; it does not mean all 188 modes passed.
The [evidence record](evidence/promise-race-test262/summary.json) binds complete
reports, source and binary identities, frozen controls, correction history and
validation logs. These results add no device-GPU, security or Chromium-relative
performance claim.
