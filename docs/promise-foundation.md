# Promise construction and reactions

The custom interpreter implements `new Promise(executor)`,
`Promise.prototype.then`, `Promise.resolve`, `Promise.reject`, and the
constructor's `Symbol.species` getter in the existing Window realm.
Executors run synchronously. Reactions and calls to captured thenable methods
join the same FIFO as `queueMicrotask` callbacks.

```js
var log = '';
var promise = new Promise(function (resolve) {
    log += 'E';
    resolve(10);
});
promise.then(function (value) { log += value; });
queueMicrotask(function () { log += 'Q'; });
log += 'S';
// After this script's checkpoint, log is 'ES10Q'.
```

Resolving with an object reads its `then` property synchronously and queues
the captured callable for later invocation. Each resolving pair shares one
private gate, consumed before that property read. The queued thenable job gets
a fresh pair. Repeated resolution, rejection, and a later thrown exception
therefore cannot replace a settled result. Self-resolution rejects with a
`TypeError`.

Handlers receive the original fulfillment or rejection value. A thrown
JavaScript object remains that same object; diagnostic inspection does not
run its getters. An absent handler forwards the original value. Ordinary
rejection alone does not emit a console message. An abrupt call to a generic
capability's resolve or reject function is reported by the host job reporter.

`then` performs observable constructor/species lookup and creates a generic
capability. It reads the source's settlement state again after those calls,
because a species constructor can settle the source reentrantly. Static
`resolve` and `reject` also support generic constructor receivers. A genuine
Promise whose observable constructor equals the `resolve` receiver is returned
unchanged before constructing a new capability. Default species uses the
saved realm intrinsic rather than the mutable global binding.

## Resource boundaries

Promise records, resolving gates, captured capabilities, callbacks, and job
payloads live in bounded runtime-owned arenas. Their IDs are never recycled.
Completed jobs leave empty payload slots; this implementation does not add a
garbage collector. Long-lived pages can exhaust the cumulative allowance.

Settlement reserves the final job and FIFO capacities together before
publishing any reaction. If the second reservation fails, the source remains
pending and its reaction list and the FIFO remain unchanged. Allocator spare
space may remain, and charged work and storage are not refunded. Generated
resolving functions publish their private records only after both callables
have been constructed.

The existing 100,000 work-unit and 8 MiB charged-storage limits are unchanged.
Promise jobs receive no separate work reset. Resource failure or unsupported
execution permanently suspends asynchronous work in that Runtime, retaining
unstarted jobs. A removed job is never replayed. These are logical admission
charges, not physical-memory measurements or a production-security guarantee.

The six intrinsic metadata bags add 930 bootstrap work units, bringing the
raw initialization boundary to 99,805 and leaving 195 units. Successful public
initialization still resets the author execution allowance. The shared queue
entry remains 24 bytes on the two checked x86-64 Rust toolchains.

## Remaining scope

`catch`, `finally`, Promise combinators, async functions, `await`, `fetch`,
rejection notifications, and multiple realms remain separate work. This is
not a complete ECMAScript Promise implementation or HTML event loop.

The algorithms follow the ECMAScript [Promise objects](https://tc39.es/ecma262/multipage/control-abstraction-objects.html#sec-promise-objects)
and [SpeciesConstructor](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-speciesconstructor)
operations and the HTML [Promise job hook](https://html.spec.whatwg.org/multipage/webappapis.html#hostenqueuepromisejob).

## Validation

All 41 new groups pass: 16 public semantic, 18 private resource-boundary,
three installer, two type-layout, one Page and one confined-worker group.
The unchanged parent compiled the 18 public semantic/integration groups and
failed as predicted; private APIs were absent there. Multi-row groups stop at
their first parent failure. Each rendering fixture completed its disabled-script
controls before failing the first enabled pixel comparison.

The final Rust 1.88 complete suites pass 3,148 tests with `vulkan-raster` and
2,844 with default features, across 45 result rows each, with none ignored.
Strict Clippy passes on Rust 1.88 and 1.98, formatting is clean, and actual
Promise layouts match the frozen mirror. All 102 rendering references pass;
all 204 PNGs remain byte-identical to the preceding startup-key checkpoint.
The new page/worker witnesses cover 12 independent literal frames across
20 layout or Render calls, including listener cleanup and repeated task polls.
Their packed native plans are interpreted on CPU; they add no GPU observation.

The first complete native run already passed. A newer Clippy lint then prompted
an equivalent standard-library ceiling-division expression; final checks bind
the corrected source. Original holds, the failed lint, explicit five-file
historical-test migrations and both source inventories remain in the
[evidence summary](evidence/promise-foundation/summary.json) and
[archive](evidence/promise-foundation/sources-and-checks.tar.gz), which bind
783 final source inputs. The authored tests are local regressions, not imported
Web Platform Tests or a conformance claim. Build caches were reused, and this
checkpoint adds no Chromium-performance or production-security result.

The subsequent [bootstrap stack correction](runtime-bootstrap-stack.md) fixes
a small-thread startup overflow found by Rust 1.99 CI after this checkpoint.
The original evidence remains unchanged.
