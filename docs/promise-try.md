# Promise.try

`Promise.try(callback, ...args)` invokes the callback synchronously and uses its
completion to return a Promise or a generic capability result. This checkpoint
implements the callback-first algorithm in the [current ECMAScript 2027
draft](https://tc39.es/ecma262/multipage/control-abstraction-objects.html#sec-promise.try).
The method has name `try`, length one and no constructor behavior.

A primitive receiver throws TypeError before invoking the callback. An object
receiver reaches the callback before constructor validation. The callback
receives `undefined` as its supplied `this` and the remaining arguments in order;
ordinary function strictness and bound-function behavior still apply. Values
retain their identities, and no argument iterator or `length` getter is read.

On normal completion, the existing abstract PromiseResolve operation receives
the captured receiver and callback result. A genuine Promise whose observable
`constructor` equals that receiver is returned unchanged, even when the receiver
is not a constructor. Constructor-getter errors escape synchronously. Other
results use a new capability and its resolve callback; failure in either step
also escapes synchronously. The method does not read `C.resolve` or species.
Existing thenable adoption still separates synchronous getter access from
deferred jobs and reactions on the shared FIFO.

An ordinary callback throw takes a different path: create the capability,
convert the retained error through the existing exception helper, then call its
reject callback once. Explicitly thrown values preserve identity. Constructor,
conversion or reject failures take precedence and escape synchronously; they
are not retried. Terminal resource and unsupported-operation failures bypass
this ordinary rejection path. Through public execution they keep author
catch/finally work and unstarted jobs suspended across later checkpoints.

Public dispatch retains its eight-unit fee. The new body prepays 16 work units
for control and ownership, then one for the callback copy. A separate tail
helper prepays `8 + 3n` work and `32 + n * sizeof(Value)` accounted bytes before
copying `n` values. The empty tail keeps the 8-unit/32-byte logical charge. The
helper rejects lengths above 65,536 after its fixed entry fee, before element
work or storage admission. This internal bound does not enlarge the ordinary
100,000-unit work allowance. Existing Call, capability, PromiseResolve and error
materialization charges remain separate and unchanged. These tariffs are
logical accounting, not instruction counts or complete physical memory costs.

Late installation adds one metadata object after `withResolvers`, preserving
earlier IDs and helper tariffs. The private controls cover 246 work units with
spare owner order capacity and 268 when the 11-entry order must grow. The
recorded x86_64 startup takes the latter branch: order length/capacity moves from
11/11 to 12/12, work rises by 268 and accounted heap by 7,595 bytes, including
the extra upfront object slot. Raw startup consumes **101,580** units, leaves
**898,420** and accounts for **1,775,866 bytes** with 757 objects. Native entries,
prototypes and globals remain 300, 26 and 213. Trusted initialization stays
bounded at 1,000,000 work units; successful public construction still restores
the 100,000-unit author allowance under the cumulative 8 MiB heap ceiling.

The published parent fails all 15 public groups at the availability guard in
their first sloppy source. Later phases, strict attempts and terminal readbacks
are unreached there. All **25 new groups pass on the first candidate**, comprising
twelve semantic, three terminal and ten private groups. The completed controls
cover 140 public execute calls, 78 private measured attempts and four private
sticky readbacks. No candidate expectation correction was needed.

Complete `--include-ignored` runs pass **3,246 Rust 1.88 default tests** and
**3,600 Rust 1.99 tests with `test262-host,vulkan-raster`**. Each has 52 result
rows and no failed, ignored or filtered tests. The full parent rosters survive
four selected boundary renames and gain the 25 new groups. Strict all-target
Clippy passes in both configurations, and formatting passes.

Rebuilt Rust 1.88 ordinary and host adapters preserve all **1,300 case records
and 546 control records** across the seven existing upstream profiles. Complete
arrays, policies and corpus identities match the parent, including existing
failed and unsupported outcomes; no baseline changes are made. Python tooling,
host protocols and CI gates are unchanged, and no full Python-suite rerun is
claimed.

The first source-inventory comparison was interrupted before writing its output
because the lexical matcher was slow; its replacement avoids redundant matching.
The final inventory also corrects inherited descriptive scope text while keeping
the same base and all 1,778 source hashes. Both bookkeeping corrections are
retained; neither changes candidate source, tests, expectations or outcomes.

There is no dedicated Promise.try upstream profile or outcome in this runtime
checkpoint; readiness references do not establish one. Classes, async functions
and multiple realms remain outside the current language subset. The evidence
binds 1,778 selected inputs plus a separate tracked-source review, rather than
claiming a complete build graph. Logical fees and regression results establish
no full-conformance, production-security or Chromium-performance result.
The existing withResolvers chapter and earlier validation sections remain
byte-exact with their historical scope.

The [evidence manifest](evidence/promise-try/manifest.json) and
[source/check archive](evidence/promise-try/sources-and-checks.tar.gz) retain the
source contracts, accounting, parent observations and final validation evidence.
