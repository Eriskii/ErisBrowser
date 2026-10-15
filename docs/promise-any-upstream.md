# Promise.any upstream results

The complete pinned Test262 `Promise/any` directory contains **94 sources and
188 sloppy/strict modes**: 130 authenticated asynchronous modes and 58 checked
synchronous modes. It records **172 passed, 14 unsupported and two failed
modes**, with all **102 controls verified**. Every selected mode remains in the
denominator. This is a bounded selection, not full ECMAScript or Promise
conformance.

The [corpus manifest](../tests/upstream/test262-promise-any/manifest.json)
binds all 94 source blobs, six unchanged upstream helpers, two
legal/instruction files and seven complete proof documents. Including the
manifest, the corpus has 110 files at revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`.

Both modes of `returns-promise.js` encounter the unchanged host's
final-unhandled-rejection veto after `Promise.any([])`; this is a host-policy
failure, not a reported body assertion or algorithm failure. Fourteen modes
remain unsupported: six declare classes, four declare destructured bindings,
and four untagged modes encounter class syntax in the parser.

The selected host route uses ERJS2 requests and requires ERJR3 responses. Only
`promise-any` gains the eighth host intrinsic identity, `AggregateError`. The
ordinary `aggregate-error` profile retains its ERJR2-only capability; default
and older host routes retain seven identities. Explicit profile context reaches
decoding, validation, classification and preflight verification. Diagnostic
names remain separate from private identity, and accepting an AggregateError
observation still classifies an ordinary positive case as failed. Wire bytes,
completion/audit rules and final-unhandled-rejection policy do not change.
These checks rely on trusted adapter output; they do not authenticate a
compromised adapter merely because its frame has a valid schema.

The controls have 42 expected passed, 58 expected failed and two expected
resource outcomes; all 102 satisfy their frozen expectations. The gate retains
the original 74 common rows and adds 20 semantic pair rows plus eight identity
rows. It checks method metadata/nonconstruction, first fulfillment, ordered and
empty errors, reusable fulfillment callbacks and once-only rejection. A custom
constructor's reject throws the genuine empty aggregate twice, exercising its
actual escape and outer retry. Separate renamed and fake constructor/prototype
objects distinguish diagnostics from identity. Handled asynchronous controls
assert errors directly; an unhandled-rejection host record does not identify
its rejection value's class.

The gate rebuilds canonical source/helper/mode/metadata fingerprints and exact
prerequisite references, validates raw evidence and recomputes classification.
A supplied verified flag or an unrelated exception cannot satisfy it. Failed
same-mode positives invalidate dependent guards. An unhealthy gate blocks the
selection and baseline recording.

The focused run also passes all 32 groups after the gate correction. All **624
Python tests pass**, including the 32 new focused groups. The parent run passed
592 tests. The first focused candidate run passed 31 groups and failed one
forged-harness guard. An empty-harness check confined to the new semantic
controls corrects that gate omission; frozen literals, oracles and test
expectations stay unchanged.

The measured Any baseline and replay preserve the exact 188 case and 102
control records. The five older profiles retain their complete 1,100 case and
400 control arrays, including failed and unsupported outcomes, with no policy
or baseline changes. The
[baseline](../tests/conformance/test262-promise-any-current.json) retains the
measured status and identity projection. Successful baseline recording/replay
means the gate is healthy and outcomes agree; it does not mean every case
passes.

The published [runtime checkpoint](promise-any.md) at `90c653d813fe` supplies
both Rust 1.88 release adapters unchanged. Any and the older Promise profiles
use its `test262-host` binary; the ordinary AggregateError replay uses its
ordinary binary. This tooling checkpoint performs no new Rust build or full
Rust-suite validation and changes no runtime limit or production Rust input.

The host CI matrix adds this baseline gate to its Rust 1.88/stable jobs.
Reproduce with the existing host build and retained baseline:

```sh
cargo +1.88.0 build --locked --release --features test262-host --bin eris-js
python3 tools/test262_conformance.py --binary target/release/eris-js \
  --profile promise-any \
  --baseline tests/conformance/test262-promise-any-current.json \
  --output artifacts/test262-promise-any-gate.json
```

No success result for this checkpoint's remote CI is claimed here. Classes,
destructured bindings, async functions and multiple realms remain unfinished;
source feature admission is not a claim of complete feature support. These
results establish no production security, process-memory bound or CPU/GPU/
Chromium-relative performance result. The [evidence
manifest](evidence/promise-any-upstream/manifest.json) and [source/check
archive](evidence/promise-any-upstream/sources-and-checks.tar.gz) retain the
sources, controls, observations and validation receipts.
