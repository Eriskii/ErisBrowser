# Complete pinned Promise.try selection

The `promise-try` profile retains the complete 15-source Test262 directory at
revision `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd` and all 30 strict/sloppy modes.
It records **24 passed and six unsupported modes**, with all **94 controls
verified**. This is a bounded upstream selection, separate from the earlier
[Promise.try runtime checkpoint](promise-try.md).

The six unsupported rows are both modes of `avoids-wrap-for-subclass.js`,
`ctx-ctor-for-error.js` and `ctx-ctor.js`. Their declared `class` feature is
excluded before adapter execution. The original bodies remain unchanged in the
corpus and all six rows remain in the report. There are no negative-metadata
sources or substituted fixtures. Of the 30 modes, 22 have synchronous policy
and eight require authenticated async completion. Six async modes pass and two
are excluded for `class`; 18 synchronous modes pass and four are excluded.

The [corpus manifest](../tests/upstream/test262-promise-try/manifest.json) binds
the complete 33-file package: 15 source bodies, eight harness files, two legal
files, seven retained proof documents and the manifest. Complete root-linked
Git trees and blob IDs authenticate the selection and helper bytes. Seven
helpers come from the original source closure; `promiseHelper.js` is added for
the existing harness guards. The unchanged `compareArray.js` is a compatibility
shim: executable global and assertion comparison routines come from `assert.js`.
Equal and unequal array controls exercise those routines; the async pair also
exercises `asyncTest` and `assert.throwsAsync` from `asyncHelpers.js`.

The profile uses the existing `test262-host` adapter with ERJS2 requests and
ERJR3 responses. Synchronous cases use checked-sync completion; async cases
require authenticated `$DONE`. The existing final-quiescent unhandled-rejection
veto remains in force. The seven intrinsic error identities remain unchanged;
this profile adds no AggregateError identity capability, protocol bytes or
runtime behavior. Its supported feature tags are `Reflect.construct` and
`promise-try`.

The canonical gate preserves all 74 common controls and adds five positive/
mismatch pairs in both modes: method metadata, callback ordering and arguments,
genuine Promise identity, ordinary rejection and async helper behavior. Each
mismatch changes one final assertion or expected value. The gate checks exact
source/helper fingerprints, complete raw observations and same-mode prerequisite
records, then recomputes the existing Promise classification. Missing, duplicate,
extra, forged or unhealthy records block all 30 source modes and prevent writing
either a new or an existing baseline. These are checks on trusted adapter
evidence, not a new browser security boundary.

The complete Python suite passes **683 tests**, retaining all 652 parent groups
and adding 31. All 31 pass on the first candidate. On the published parent,
six of those groups pass and 25 do not; unittest's 60 failure and 46 error
observations include subtests. Import and loader tests reproduce the exact
package and reject source, helper, proof and selection mutations, including
manifest rewrites with self-consistent hashes. Protocol and gate tests cover
completion evidence, prerequisite failures and baseline refusal. No candidate
source, test or expectation correction was needed.

The initial healthy run records the [new baseline](../tests/conformance/test262-promise-try-current.json).
Recording and the subsequent baseline check both exit zero. The complete 30
case and 94 control arrays match exactly, with zero regressions or improvements.
All seven older profiles also preserve their complete **1,300 case and 546
control arrays**, policies, corpus identities and reused binary identities,
including retained failed and unsupported outcomes. The 60 earlier profile
policies and their baselines remain unchanged.

A root report checker initially compared the empty `fixture_files` list with a
numeric count and stopped before producing its receipt. Correcting that
comparison changed no candidate source, test, oracle or report; both actual
profile runs were already complete with healthy gates. The original checker
and correction remain in the evidence.

This checkpoint reuses the exact Rust 1.88 ordinary and host release adapters
from the published runtime. Rust/Cargo/build inputs and runtime limits are
unchanged; no new Rust suite, Clippy or build is claimed. The 1,820-input selected
inventory has a separate tracked-source review and is not a complete build
graph. All 11 jobs of prior CI run `38817823918` passed for published parent
`15db31c7`; the configured baseline step is not a new remote-success result.
Classes, async functions and multiple realms remain unfinished, and these
measurements establish no full-conformance, production-security or performance
claim. The runtime chapter and earlier validation body remain byte-exact.

With a host-enabled adapter already built, run the recorded gate from the
repository root:

```sh
python3 tools/test262_conformance.py --binary target/release/eris-js --profile promise-try --baseline tests/conformance/test262-promise-try-current.json --output artifacts/test262-promise-try-gate.json
```

The [evidence manifest](evidence/promise-try-upstream/manifest.json) and
[source/check archive](evidence/promise-try-upstream/sources-and-checks.tar.gz)
retain the frozen contracts, authenticated corpus, parent observations and
complete validation records.
