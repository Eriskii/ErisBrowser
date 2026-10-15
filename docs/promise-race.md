# Promise.race

`Promise.race` accepts synchronous iterables and lets the first fulfillment or
rejection settle its intrinsic result. An empty input leaves that result pending.
The installed method has name `race`, length one and no constructor behavior.

The [standard algorithm](https://tc39.es/ecma262/multipage/control-abstraction-objects.html#sec-promise.race)
creates a capability from its receiver, reads `C.resolve` once, then acquires the
iterator and its `next` method. Each input goes through the cached resolve and
an observable `then` lookup and call. Those calls receive the original capability
resolve and reject functions. Custom callbacks keep their identities, repeated
call behavior, return values and thrown values; race adds no per-input once
wrapper. Iteration continues after an early settlement. Race itself does not
consult species, though observable calls may do so.

Ordinary iterator acquisition and step errors reject without closing. Later
ordinary resolve or then failures close an open iterator before rejection,
preserving the original thrown value across ordinary close failures. A throwing
custom reject callback escapes once. Existing resource-limit and unsupported
operation failures instead stop further author close/reject calls and suspend
unstarted jobs. Later readback entries do not resume those jobs. There is no
in-operation budget reset or automatic retry.

The method reuses existing capability, iterator and FIFO machinery without an
all-style collection arena. Ordinary capabilities, metadata and queued jobs
still incur their existing allocation charges. The body prepays 17 work units
for control and direct-handle cleanup; public dispatch adds eight, and reached
Get/Call/Construct, handle copies, argument vectors and other helpers remain
separately charged. These amounts are not a whole-call cost or a bound on
inherited object-graph destruction, native stack or RSS.

Installation adds 121 work units and one metadata bag. Raw startup uses 99,846
of the unchanged 100,000-unit allowance, leaving 154. The cumulative 8 MiB
accounted-heap limit and other runtime limits remain unchanged. Existing
function identities and metadata stay intact; the constructor gains `race`
after `all` in its string-property order.

The frozen controls comprise 17 ordinary public groups, six terminal-policy
groups, four private accounting/storage groups and two Page/worker integrations.
The integrations check full pixels and hits before titles across two viewport
widths, disabled scripts, task polling and actual clicks. Initial title
`SEPQW7` and clicked title `SEPQW7ACQR` accompany a red 4-by-4 rectangle at
x6 and x10; disabled scripts retain x2. A blue rectangle stays at x20.
Expected frames are generated from frozen literal geometry. Optional native
plans are checked through the existing CPU interpreter; this adds no device
GPU or Chromium-relative performance result.

All **29 new regression groups pass**, including both Page and confined-worker
integrations. The complete suites pass **3,084 tests on Rust 1.88** and **3,438
with `test262-host,vulkan-raster` on Rust 1.99**, across 51 result rows each
with no failed, ignored or filtered tests. Strict Clippy passes for all targets
in both configurations; formatting also passes. The 17 ordinary public groups
cover 110 observations across sloppy and strict modes. The six terminal groups
cover 24 operations and 48 sticky readbacks; the four private groups cover
28 operations.
Twenty existing groups receive the reviewed metadata/accounting updates;
no frozen new expectation was changed. Before implementation, all 17 public,
six terminal and two integration groups failed at their documented first
missing-method boundary. Private helper tests require the new implementation.

The release adapter preserves all **654 case-mode and 164 harness records**
from the existing Promise profiles: `Promise.all` remains 174 passed, 14 failed
and 8 unsupported out of 196; the earlier profile remains 420 passed, 8 failed
and 30 unsupported out of 458. All 164 harness controls pass. These checks preserve
existing outcomes; they are not a new `Promise.race` conformance result.

The authenticated 94-source `Promise/race` directory remains a separate upstream
profile follow-up; the retained 20-source sample is not a complete run. Authored
controls do not establish full Promise or ECMAScript compatibility. `allSettled`,
`any`, async functions, browser rejection notifications and multiple realms
remain unfinished.

[Source and validation evidence](evidence/promise-race/summary.json) binds the
complete build inventory, changed inputs, frozen contracts, parent observations,
review receipts and final logs. Its adjacent archive contains the selected
sources and checks; compiled binaries are excluded.
