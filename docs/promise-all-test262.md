# Promise.all upstream results

The complete immediate Test262 `test/built-ins/Promise/all` directory at revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd` contains 98 sources and 196 required
sloppy/strict modes. Eris records **170 passed, 18 failed and 8 unsupported**.
All 90 harness controls pass. No selected mode is removed from the denominator.
This is a bounded selection, not full Promise or ECMAScript conformance.

The [manifest](../tests/upstream/test262-promise-all/manifest.json) authenticates
every source, six unchanged harness helpers and two legal/instruction files
against the pinned Git root. Seven retained proof documents establish complete
directory coverage and blob identities. The selection includes 57 asynchronous
sources (114 modes) and 41 synchronous sources (82 modes). Import and verification
retain the existing file, proof, byte and runtime limits.

| Observation | Modes |
| --- | ---: |
| Passed | 170 |
| Host failure: final unhandled rejection | 14 |
| Parse SyntaxError: call-spread syntax | 4 |
| Unsupported class feature declared in metadata | 4 |
| Unsupported class syntax encountered by the parser | 4 |
| Total | 196 |

The fourteen host failures come from seven synchronous sources:
`capability-resolve-throws-no-close`, `invoke-resolve-error-close`,
`invoke-resolve-get-once-multiple-calls`, `invoke-then-error-close`,
`invoke-then-get-error-close`, `iter-next-val-err-no-close` and
`iter-step-err-no-close`. Each reaches audited quiescence with one unhandled
rejection. They retain the separate `host-failure`/`unhandled-rejection`
observation; they are not relabeled as ECMAScript assertion failures or passes.

The two `invoke-resolve-on-*-every-iteration-of-promise.js` sources reach
unsupported call-spread syntax in `boundPromiseResolve(...args)`, which currently
produces a parse `SyntaxError`. All four modes remain failed. Four class-bearing
sources remain unsupported: two are excluded by their declared feature metadata
and two reach the parser. The latter were not filtered through a source-text
heuristic; their unchanged source bytes and observations remain available.

The profile uses the existing optional `test262-host` adapter, authenticated
`$DONE` completion and final-unhandled-rejection veto. Its supported feature set
adds `Symbol.iterator` to the older Promise profile. Eight new literal preflight
variants run in both modes, after the existing 74 controls. Positive/mismatch
pairs cover method metadata, ordered asynchronous values, rejection identity and
generic iterator/once behavior. Each new result depends on all 37 common controls
in its mode; mismatches also depend on their corresponding positive. A missing,
duplicate, malformed or unverified control blocks all 196 cases and prevents
baseline recording.

Eleven new Python test groups cover source authentication, metadata retention,
exact identities, preflight dependencies, failure blocking and preservation of
the old profile. All 543 Python tests pass. The first prospective oracle had
incorrectly inserted a null `timeout` field; an unchanged-parent control caught
the mismatch before production import. Source derivation corrected the oracle
to preserve absent metadata, without changing the parser or fingerprint rule.
The original failure, correction and review remain in the evidence.

The existing 232-source, 458-mode Promise-methods profile was replayed with the
same release adapter: **420 passed, 30 unsupported and 8 failed**, with all 74
controls passing, zero regressions and zero newly passing modes. Its source,
case and policy identities remain unchanged. This checkpoint changes Python
tools, corpus, baseline and CI; it makes no Rust runtime changes. The release
adapter was built from the published `Promise.all` implementation at `5dc150e2`.
No new browser, GPU-device, production-security or Chromium performance claim
follows from this run.

To run the regression gate:

```sh
cargo build --locked --release --features test262-host --bin eris-js
python3 tools/test262_conformance.py --binary target/release/eris-js \
  --profile promise-all \
  --baseline tests/conformance/test262-promise-all-current.json \
  --output artifacts/test262-promise-all.json
```

The [baseline](../tests/conformance/test262-promise-all-current.json) preserves
every status and case identity. A successful baseline gate means the harness is
healthy and no recorded pass regressed; it does not mean all 196 modes passed.
The [evidence record](evidence/promise-all-test262/summary.json) includes complete
reports, frozen controls, source bindings, original correction history and logs.
