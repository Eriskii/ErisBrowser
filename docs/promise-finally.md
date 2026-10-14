# Promise.prototype.finally

`Promise.prototype.finally` runs cleanup after fulfillment or rejection and waits
for its result. Successful cleanup preserves the original value or reason;
failed cleanup supplies the new rejection. It uses the existing bounded
microtask FIFO and the active script or host-callback budget.

The method requires an object receiver and observes its constructor/species
before checking whether the finalizer is callable. Generic objects can supply
their own `then`. Generated callbacks retain the chosen constructor, invoke
cleanup without arguments, use internal Promise resolution and observe the
result's `then`. They do not look up a mutable `C.resolve` property. The returned
value from each observable `then` call is preserved. This follows the
[ECMAScript algorithm](https://tc39.es/ecma262/multipage/control-abstraction-objects.html#sec-promise.prototype.finally)
for the runtime's represented values and single realm.

Reusable callbacks store their values in a separate append-only capture arena.
They do not consume the one-shot gates used by resolving functions. Metadata
allocation and admission finish before retained captures and callback records
are published. A refusal can leave unexposed ordinary metadata bags and charged
work behind; it cannot return a partially published callback. Fatal resource
errors suspend pending jobs under the existing policy.

Fifteen semantic groups cover generic invocation, species/getter ordering,
exact abrupt values, repeated callback calls, function metadata, thenable
cleanup, SameValue retention and fatal suspension. Seven private groups cover
exact and one-short work/heap boundaries, staged failure, physical spare
capacity, immutable reads and capture identity. A separate layout group checks
actual types against the pre-implementation mirrors.

Page and confined-worker tests each check twelve rendered or laid-out frames,
including disabled scripts, initial cleanup, polling, a click and subsequent
polling at two sizes. The fixed title traces are `SFGQTMV7` initially and
`SFGQTMV7ACKDB` after clicking. Runtime assertions construct expected pixels
from fixed rectangles; twelve separately archived RGBA files represent the
same geometry. These checks use CPU painting and CPU interpretation of native
plans, without new GPU output measurements.

All fifteen semantic groups and both integration groups failed on the parent
before implementation. Each integration group completed its disabled-script
controls before failing the first enabled pixel check. Later assertions were
not observed in those failing baseline groups. The private-API tests could not
run on the parent. Final full suites pass 3,190 tests with `vulkan-raster` on
Rust 1.99 and 2,886 default tests on Rust 1.88, including all 89 confined-worker
tests with none ignored. Both compilers pass strict Clippy; formatting passes.
The existing 102 browser reference pairs pass and all 204 PNGs match the parent
byte for byte.

Installation adds one method object, 122 startup work units and one empty arena
header. The selected ledger puts raw initialization at 99,995 of the unchanged
100,000 allowance. Runtime callback fees remain additional to ordinary
property lookup, calls, construction and job processing. These are policy
charges, not hardware performance measurements.

Combinators, async functions/`await`, complete rejection notifications, multiple
realms and garbage collection remain unfinished. These authored regressions
are not a full Test262 result. The current upstream adapter still excludes
asynchronous completion tests until it has a supported `$DONE` host contract.

[Evidence](evidence/promise-finally/summary.json) records the held expectations,
parent observations, source reviews, resource ledger and final checks.
