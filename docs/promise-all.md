# Promise.all

`Promise.all` collects fulfillment values in input order, regardless of their
completion order. It uses the existing synchronous iterator protocol, generic
Promise capabilities and shared microtask FIFO. The native method has length
one and is not a constructor. Other combinators and async iterators are outside
this change.

The [standard algorithm](https://tc39.es/ecma262/multipage/control-abstraction-objects.html#sec-promise.all)
constructs the capability from its receiver, reads `C.resolve` once before
acquiring the iterator, and calls that cached function with each input.
It observes each result's `then` and supplies an independent fulfillment callback
plus the original reject callback. It does not replace these observable calls
with an internal Promise shortcut or consult species itself. A custom reject
callback can run repeatedly; rejection does not cancel iteration. The last
fulfillment callback returns the custom resolve call's result. Empty-input
completion instead returns the original capability promise.

Ordinary errors preserve their thrown values. Iterator acquisition and step
errors do not trigger iterator closing; later ordinary resolve/then errors
close a still-open iterator before rejection. A resource
limit or unsupported-operation failure stops before further author close or
reject calls and suspends unstarted jobs through the existing fatal-error
policy. No path receives a fresh work allowance or automatic retry.

One append-only arena stores groups and per-input once gates. A group starts
with a remaining-count sentinel, so a synchronously invoked custom `then`
cannot finish it before normal iterator exhaustion. An ordinary abort leaves
that sentinel intact: earlier saved callbacks remain usable but cannot complete
the abandoned group. Each first fulfillment writes its reserved index once;
duplicates return before accessing a completed group's moved values. Author
calls carry stable IDs, with fresh state checks afterward. No arena borrow or
capacity forecast survives an author call.

Metadata, records and counters are admitted before publication. A late refusal
can retain charged, unexposed metadata bags or physical reserve capacity, but
cannot publish a partial callback or count increment. Readiness moves the whole
values arena, resetting both its vector and logical admission. The result
publisher reserves an object and all four parallel Array rows before publishing
a fresh dense Array with the cached intrinsic prototype. It bypasses mutable
global `Array`, species and inherited numeric setters. Gates already consumed
before a later resource refusal are not replayed.

All existing limits remain: 100,000 work units per admitted script/host entry,
8 MiB cumulative accounted heap and the existing call, prototype and job bounds.
Installation adds 108 work units and 2,867 accounted heap bytes relative to the
Node-accessor-key checkpoint. Raw startup consumes 99,975 units, leaving 25,
and accounts for 1,741,555 heap bytes.
Current x86-64 type observations are 256 bytes for Promise State and 1,808 bytes
for Runtime without `test262-host`, or 1,872 with it. Both remain within the
existing 2,048-byte base envelope. Callback records and FIFO/job widths are
unchanged. These are resource-policy charges and type sizes, not stack, RSS or
performance measurements.

The authored controls comprise 31 public semantic groups, 16 private quota
groups, one actual-layout group and two integrations: 50 new groups in total.
Private controls cover exact and one-short work/heap cuts, duplicate gates,
ordered value retention, completed-group refusal and injected late reserve
failures with scoped cleanup. Before implementation, the original 24 public
groups recorded one pass and 23 failures on `720127cb`; seven additional abort
groups recorded no passes and seven failures after the Node prerequisite
(`8e3db775`). Both integrations on `720127cb` completed their disabled-script
controls before failing the first enabled pixel, with no passing integration
groups; later enabled assertions were unreached.

Page and confined-worker routes each check twelve frames across two viewport
widths, including disabled scripts, load, polling, click and later polling.
The title changes from `SQA` after load to `SQACDKB` after clicking. The red
4-by-4 rectangle moves from x2 to x12 and then x16; the blue rectangle stays
at x20. Literal geometry and titles were frozen before implementation. Runtime
assertions build expected pixels procedurally and check hits before titles;
Native plans are interpreted on the CPU where enabled. These tests add no GPU
measurements or Chromium performance comparison.

All 50 new groups pass on source1. Its complete suites pass 2,945 default tests
on Rust 1.88 and 3,299 tests with `test262-host,vulkan-raster` on Rust 1.99, each
across 49 result rows with no failures or ignored tests. Source2 changes only
an equivalent conditional in a private test. It passes all 17 private groups
and strict Clippy in both configurations, plus final formatting. The full-suite
totals remain attributed to source1; production code is unchanged by the cleanup.

The complete pinned Test262 `Promise/all` directory has 98 authenticated sources
and 196 required modes. The [subsequent conformance record](promise-all-test262.md)
reports 170 passed, 18 failed and 8 unsupported modes, with all 90 harness controls
passing. The earlier 232-source Promise-methods profile retains its exact
outcomes on replay. These authored controls and selected upstream results do not establish
full Promise or ECMAScript conformance. Other combinators, async functions,
complete rejection notifications, multiple realms and garbage collection remain
unfinished.

The [evidence record](evidence/promise-all/summary.json) retains the
frozen controls, parent observations, source reviews, resource ledger and final
validation bindings.
