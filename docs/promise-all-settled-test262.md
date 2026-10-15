# Promise.allSettled upstream results

The complete immediate Test262 `test/built-ins/Promise/allSettled` directory at
revision `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd` contains 104 sources and
208 required sloppy/strict modes. Eris records **188 passed, 12 failed and
8 unsupported**. All 94 harness controls verify their expected outcomes.
Every selected mode remains in the denominator. This is a bounded selection,
not full Promise or ECMAScript conformance.

The [manifest](../tests/upstream/test262-promise-all-settled/manifest.json)
binds every original source, six unchanged harness helpers and two
legal/instruction files to the pinned Git root. Seven complete nonrecursive
proof documents establish the directory ancestry and raw blob identities.
There are 57 asynchronous sources (114 modes) and 47 synchronous sources
(94 modes). No source is rewritten or removed because of its feature tags,
syntax or eventual result. File, proof, byte and runtime limits remain unchanged.

| Observation | Modes |
| --- | ---: |
| Passed | 188 |
| Host failure: final unhandled rejection | 12 |
| Unsupported class feature declared in metadata | 4 |
| Unsupported class syntax encountered by the parser | 4 |
| Total | 208 |

The passed rows comprise 112 asynchronous and 76 synchronous modes. Two of the
114 asynchronous modes are class-feature exclusions; the result does not claim
that every asynchronous mode passed.

The twelve host failures come from six synchronous sources, each in both modes:
`capability-resolve-throws-no-close.js`, `invoke-resolve-error-close.js`,
`invoke-then-error-close.js`, `invoke-then-get-error-close.js`,
`iter-next-val-err-no-close.js` and `iter-step-err-no-close.js`. Each raw record
reports `host-failure` / `unhandled-rejection`, audited quiescence and exactly
one unhandled rejection, with no completion, body-abrupt or terminal flags.
These are final host-policy vetoes, not observed upstream assertion exceptions.
Source inspection shows discarded aggregate promises on the relevant abrupt
paths; the reports do not serialize their rejection values or establish an
additional exact rejection-object identity result.

The tagged sources `ctx-ctor.js` and
`invoke-resolve-on-promises-every-iteration-of-custom.js` account for four
metadata exclusions. The untagged
`resolve-throws-iterator-return-is-not-callable.js` and
`resolve-throws-iterator-return-null-or-undefined.js` reach the adapter and
produce four actual parser refusals: `'class' is not supported`. The former
also contains `0n`, but class refusal is observed first. These results establish
no separate BigInt execution or refusal outcome. All original bytes and modes
remain in the manifest, reports and baseline.

The profile reuses the optional `test262-host` adapter and its authenticated
`$DONE` completion, sticky error observations and final audit. It adds only
`Promise.allSettled` to its own supported feature set. The three older Promise
profiles keep their existing feature policies, source identities and baselines.
No Rust, resource-cap, transport or host-policy change belongs to this checkpoint.
The release adapter comes from the published
[Promise.allSettled implementation](promise-all-settled.md) at `486bca305a08`.

The gate requires 94 verified control rows: the unchanged 74 common controls,
then five positive/mismatch pairs in both modes. The new pairs check method
metadata and nonconstruction; both branches of `checkSettledPromises`; mixed
settlement order, payloads and own descriptors; paired callback gates and the
last callback's generic resolve return value; and ordinary abrupt iterator
closing with the unfinished-iteration sentinel retained. The helper pair also
requires exact assertion failures for wrong fulfilled values and rejected
reasons; its poisoned variant replaces the helper with a no-op.

Each new row requires all 37 common controls in its mode. The settlement and
shared-gate pairs additionally require the same-mode healthy settled helper;
each mismatch requires its matching positive. Missing, duplicate, extra,
wrong-mode or unverified controls block the upstream cases and baseline
recording. A deliberate mismatch is healthy only when its expected exact
synchronous assertion or authenticated asynchronous completion error is verified.
The pinned `promiseHelper.js` remains unchanged; this profile exercises its
already-defined `checkSettledPromises` path as well as the common sequence helper.

All 13 new Python test groups pass. The complete Python suite passes
569 groups: the prior 556 names plus the exact 13 additions, with no skipped or
renamed groups. On the published parent with only the frozen corpus and tests
imported, two metadata/preservation groups pass and eleven new-profile groups
produce 35 error rows. Those are group/subtest counts, not 35 independent tests.
The errors arise at missing profile symbols or dispatch; one old generic
preflight reaches a profile-specific mock and raises `KeyError('success')`.
Deeper refusal assertions are not claimed as reached on that parent.
No prospective source, control, assertion or expected outcome was changed after
these observations.

The 13 frozen profile groups cover complete source/proof/helper authentication,
metadata and mode identities, locally rehashed mutations, missing or unbound
inputs, exact control dependencies, unavailable host capability, malformed
completion evidence, blocked reports and old-profile preservation. Sixteen old
test files add the new profile to 20 existing exclusion sites, preserving their
historical table counts and hash expectations. The new preservation group
separately binds all 56 prior profile maps and feature sets.

The first unbaselined measurement exits 1 because it retains nonpassing cases,
while all controls are healthy. The normal `--record-baseline` command then
records the measured statuses, and the `--baseline` replay exits 0 with exactly
the same case and control arrays. Recording and replay preserve all twelve
failures and eight unsupported modes; there are no resource, timeout, blocked
or adapter-error rows. The baseline is the report's exact status/identity
projection, not a manually edited outcome table.

All executions reuse the published API release adapter byte-for-byte:
4,432,600 bytes, SHA-256
`b653280c13235292194de1491cf134e3523e6b10e21c0eb73f9f19d6301d872a`.
Rust execution inputs and build manifests are unchanged. This tooling/corpus
checkpoint does not claim a local Rust rebuild or Rust-suite rerun.

The preservation comparisons cover the prior 842 Promise case identities and
254 control rows across the methods, all and race profiles. Their reports retain
all prior statuses. New-profile replay compares all 208 case records and all
94 control records, rather than only aggregate counts.

To run the regression gate:

```sh
cargo build --locked --release --features test262-host --bin eris-js
python3 tools/test262_conformance.py --binary target/release/eris-js \
  --profile promise-all-settled \
  --baseline tests/conformance/test262-promise-all-settled-current.json \
  --output artifacts/test262-promise-all-settled-gate.json
```

CI is configured with an adjacent `promise-all-settled` gate after the existing
Promise gates, using their release host build. No remote CI result for this
profile checkpoint is claimed here.

The [baseline](../tests/conformance/test262-promise-all-settled-current.json)
preserves every mode's status and identity. A successful regression gate means
the controls are healthy and no recorded pass regressed; it does not establish
that every mode passed. The
[evidence record](evidence/promise-all-settled-test262/summary.json) binds the
raw reports, immutable source and binary identities, frozen controls and tests,
parent observations and any retained corrections. Earlier checkpoint chapters
and their measured figures remain historical records.
