# Authenticated Promise Test262 host

The optional `test262-host` feature lets `eris-js` run the pinned
`promise-methods-async` and [`promise-all`](promise-all-test262.md) profiles with checked synchronous completion and
an authenticated `$DONE` path. Promise jobs use Eris's interpreter and existing
microtask FIFO. This adds a test adapter contract; it does not add timers,
async functions, combinators or the full browser event loop.

The profile retains **232 unchanged source files and 458 required modes** at
Test262 revision `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`. It includes 222
asynchronous modes from 113 files and 236 synchronous modes from 119 files.
The eight complete immediate directories are:

| Directory under `test/built-ins` | Sources |
| --- | ---: |
| `Promise` | 58 |
| `Promise/Symbol.species` | 5 |
| `Promise/prototype` | 6 |
| `Promise/prototype/catch` | 14 |
| `Promise/prototype/finally` | 29 |
| `Promise/prototype/then` | 75 |
| `Promise/reject` | 15 |
| `Promise/resolve` | 30 |

[The manifest](../tests/upstream/test262-promise-methods-async/manifest.json)
links each selected directory to the pinned Git root and retains eight
auxiliary blobs, including the original assertion helpers, completion helper,
interpretation instructions and license. This is not the complete Promise
tree. Class and cross-realm cases remain in the inventory and result denominator;
the supported-feature policy reports them as unsupported instead of deleting
their sources. Failing and resource-limited cases also remain visible.

The adapter checks framing, harness order and the exact 625-byte
[`doneprintHandle.js`](../tests/upstream/test262-promise-methods-async/harness/doneprintHandle.js)
before capturing host state or creating a Runtime. It parses the outer test
body first, so a parse failure cannot execute the harness. Harness sources run
unchanged in their required order. Immediately after the completion helper,
the Runtime captures the original `$DONE` function and installs a private
observer that delegates the same receiver and arguments to it. The helper's
truthiness rule is preserved, including success for falsy arguments. Saved
genuine aliases remain usable after public property replacement.

An asynchronous pass requires exactly one authenticated completion call, one
success print from its delegated helper, no failure print, normal body/harness
completion and healthy final state. A JavaScript string or public `print` call
cannot establish completion authority. Duplicate or premature completion,
ordinary queued exceptions, delegate errors and fatal resource failures veto
provisional success. Actual JavaScript errors retain their own type and phase;
completion-policy failures carry a separate host reason. A dynamic `Function`
parse error reached during body evaluation has runtime phase even though its
diagnostic contains a source offset.

This profile deliberately requires authenticated `$DONE`. The pinned
[Test262 interpretation instructions](../tests/upstream/test262-promise-methods-async/INTERPRETING.md)
also permit asynchronous tests that call `print` directly. Those print-only
contracts are outside this adapter's selected scope; the restriction is not a
general Test262 rule.

After the body and its queued jobs finish, the adapter separately requires an
empty, unsuspended FIFO and a funded scan of genuine Promise records. A rejected
record still marked unhandled vetoes the result. A rejection handled during
the same drain is judged by its final state. This final-unhandled policy applies
to synchronous rows too, without requiring `$DONE` there. It is a host policy,
not implementation of browser `unhandledrejection`/`rejectionhandled` events.
Pending Promises alone do not make the FIFO nonquiescent. There is no later
waiting period or timer scheduler.

All admission, metadata, delegation and final scanning use existing work and
heap limits. Host setup is staged: a refusal can retain charged, unexposed
metadata allocations but cannot publish a partial session. Setup normally
costs 405 work units with spare object slots; binding costs 49 and arming 16.
A completion entry costs 24 before ordinary callback work. Finalization costs
24 plus three units per Promise record and gets no new allowance or retry.
These are resource-policy charges, not performance measurements.

The feature is off by default. `ERJS2` requests and `ERJR3` evidence are confined
to these profiles; disabled builds reject the new request before Runtime creation.
Existing `ERJS1`/`ERJR2` requests, old profile policies and baseline hashes retain
their previous behavior, including when the binary was built with the feature.
The new response has six text fields and eight evidence fields in a 12-byte
tail. Missing or contradictory evidence is a nonpass, with no legacy fallback.

To run the selected corpus:

```sh
cargo build --locked --release --features test262-host --bin eris-js
python3 tools/test262_conformance.py --binary target/release/eris-js \
  --profile promise-methods-async \
  --baseline tests/conformance/test262-promise-methods-async-current.json \
  --output artifacts/test262-promise-methods-async.json
```

The checked-in baseline preserves the observed results, including failures.
Baseline checks bind the corpus, mode inventory, per-case source/harness
and runner policy, and reject loss of any previous pass. They do not require
all retained cases to pass.

The runner requires all **74 preflight variants** before executing upstream
cases. Seventeen Runtime groups cover identity, delegation, lifecycle and
quota boundaries; eleven adapter groups cover authentication, bounded framing,
phase/evidence consistency and disabled-feature dispatch. CI builds the feature
on Rust 1.88 and stable and requires both exact test rosters before the full
suite. Corpus import has separate inventory and proof controls.

The recorded selection verifies all 74 controls and reports **420 passed,
30 unsupported and eight failed modes**. Two failures reference the missing
`Proxy` global. Six are final-unhandled host-policy vetoes in three sources:
`rxn-handler-fulfilled-next-abrupt.js`, `rxn-handler-rejected-next-abrupt.js`
and `arg-uniq-ctor.js`, each in both modes. Four of those six have successful
authenticated `$DONE` observations. These vetoes are distinct from upstream
assertion failures. All eight failures remain in the denominator and baseline;
a successful regression gate does not mean the entire selection passed.

Final full suites pass 2,887 default tests and 2,914 host-enabled tests on
Rust 1.88, plus 3,241 tests with both `vulkan-raster` and `test262-host` on Rust
1.99. Each suite includes 89 confined-worker tests, with none ignored. Strict
Clippy checks pass with and without the host on both compilers; formatting and
all 532 Python tooling tests pass. The CI discovery guard verifies all 28 new
Rust groups. The existing synchronous 652-mode selection also preserves its
568 passes and 84 unsupported modes through the feature-enabled binary, with
32 verified controls and zero regressions.

Both compilers observe a 1,776-byte Runtime without the feature and 1,840 bytes
with it. The optional session occupies 64 bytes; both fit the existing 2,048-byte
base allowance. Raw startup still consumes 99,995 work units. Existing 128 KiB
execution witnesses pass in both configurations. These observations do not
measure physical stack frames or establish production security.

All 102 browser reference pairs pass; their 204 PNGs match the preceding Vulkan
checkpoint byte for byte. No new GPU execution or Chromium performance
comparison is claimed here. The [evidence archive](evidence/promise-test262-host/summary.json)
retains the original controls, failed parent observations, adapter phase and
test-only lint corrections, complete reports, final source and binary bindings.

These bounded profile results do not establish full Promise, ECMAScript or
Test262 conformance. Cross-realm execution, complete host hooks and rejection
notifications remain unfinished.
