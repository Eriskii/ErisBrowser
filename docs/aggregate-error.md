# AggregateError

This chapter records the AggregateError API checkpoint at `d3efc9aba871`.
Its validation and decoder limitations below describe that checkpoint. The
separate [upstream profile](aggregate-error-upstream.md) records the later
profile-local ordinary-response identity capability and conformance selection.

`AggregateError(errors, message, options)` works as a call or with `new` in the
supported language subset. It collects a synchronous iterable into a fresh,
dense intrinsic Array, preserving input order and value identities. Empty input
produces an empty `errors` Array. The constructor has length two and inherits
from the saved `Error` constructor; its prototype inherits from the saved
`Error.prototype`.

Construction reads `NewTarget.prototype` before converting an optional message,
checking the optional cause and acquiring the iterable. An undefined message
creates no own `message` property. Object options use observable cause presence
and then a value read, including inherited properties and present undefined
values. Primitive options supply no cause. Created `message`, `cause` and
`errors` properties are writable and configurable but nonenumerable; inherited
setters cannot intercept their definitions.

Iterator acquisition reads the iterator method and caches `next`. Each reached
step reads `done` before `value`; an exhausted result's value is not read.
Ordinary acquisition, step and value errors propagate without `IteratorClose`.
The result Array uses the retained intrinsic Array machinery, so replacing the
global `Array` does not replace it. A saved callable constructor supplies its
own target even after the global or `prototype.constructor` changes. Existing
`Reflect.construct` validation and explicit NewTarget prototype selection remain
in effect; a non-object prototype falls back to the retained AggregateError
prototype.

Instances carry the existing private Error brand. `Object.prototype.toString`
now uses `[object Error]` as the fallback for genuine existing Error instances
and AggregateError instances. A string `Symbol.toStringTag` still overrides the
fallback; a non-string tag is not coerced. Getters retain their observable error
behavior. Copying a name, constructor or prototype does not forge the private
brand, and the ordinary AggregateError prototype itself is not branded. This
change does not otherwise extend the older Error constructors.

Construction shares the current author work and cumulative allocation budgets.
Its temporary value list grows through the existing checked planner, with a
maximum derived from the heap allowance and the actual Value size. Moving that
list into the existing dense-array helper grants no refund. Resource-limit and
unsupported-operation failures are terminal: later author steps and unstarted
jobs do not run, and later readbacks do not resume the suspended queue. No
iteration retry, author `return` call or in-operation budget reset is introduced.
These are bounded subset semantics, not a complete implementation of all
possible iterable, realm or language behavior.

The selected installation ledger adds **1,103 work units** and two appended
metadata bags. Its arithmetic gives **100,797 consumed units and 899,203
remaining** from the finite 1,000,000-unit trusted allowance. Successful
public Runtime construction still returns 100,000 author work units, and
both phases share the cumulative 8 MiB allocation ceiling. The typed heap delta is
`2*S + 2*A + 2*D + 6*K + 2*Bpp + N + 3*Bstatic + 4*Bnative + 4*Bglobal + 2634`.
Here `S`, `A`, `D`, `K` and `N` are the sizes of ScriptObject,
Option<AbortSlot>, Option<f64>, PropertyKey and Native; each `B` is the existing
typed B-tree node allowance for its property, static-key, native-map or global
map. On the previously observed x86_64 Linux layouts this is **17,554 accounted
bytes**, including both upfront object slots. These figures are source-derived
accounting, not process-memory or timing measurements.

Installation preserves every preceding object ID and map entry. The new
prototype and constructor bag follow them, and the global is published last.
A failed initialization can retain prepaid private metadata or earlier map
entries inside a realm that is never returned. Existing raw tests retain their
99,693, 99,694 and 100,000 inputs as refusals and add 100,796/100,797 and the
default 1,000,000 boundaries. Eighteen existing test files receive selected
count/ledger changes, with four explicit boundary-name renames. Earlier-stage
fixtures exclude only the new late entry they cannot retain; their original
order, isolated fees and early 25-prototype assertions remain intact.

The authored controls cover observable ordering and identities, Error tag
fallback, terminal suspension, and private work/storage admission. Parent runs
passed the string-tag override group in both modes. The other thirteen semantic
groups and five terminal groups stopped in their first sloppy attempt; later
readbacks and strict attempts were not reached. Their generic Error diagnostics
do not identify every internal check statement.

Retained attempts include an unsigned arithmetic compile failure,
a bootstrap run with 47 passes and six earlier-stage DataView fixture failures,
and a strict lint refusal. The corrections reorder unchanged ledger terms,
exclude only the new late prototype from the earlier DataView stage, and borrow
a one-element setup slice instead of cloning it. Separately, source review
moved an iterator arrow into setup before measurement so its metadata could not
distort the private allocation witness. All selected traces, quota expectations
and production behavior remain unchanged. The corrected bootstrap-focused run
passes all 53 tests; the earlier records remain retained.

The raw diagnostic records 899,203 work remaining and 1,761,871 accounted bytes,
confirming the 1,103-work and 17,554-byte deltas against the published parent.

The final candidate passes **3,159 Rust 1.88 default tests** and **3,513
Rust 1.99 tests with `test262-host,vulkan-raster`**. Each complete
`--include-ignored` run has 52 result rows with no failed, ignored or filtered
tests. Both preserve the parent roster after four selected boundary-name
renames and add exactly 27 groups: fourteen semantic, five terminal, two
installer and six private runtime groups. The installer controls cover 23
operation rows; the private runtime controls cover 57. Both full suites were
rerun after the lint-only fixture correction. Strict all-target Clippy passes
in both configurations, and formatting passes.

The rebuilt Rust 1.88 release host adapter preserves all **1,050 case records
and 348 harness records** across the four existing Promise profiles. Complete
arrays match the published parent, including failed and unsupported outcomes;
all controls verify. There are no regressions, improvements or baseline changes.
The tooling, corpora and baselines remain unchanged; no separate full Python
suite rerun is claimed.

`Promise.any`, classes, Proxy, async iteration and multiple realms remain
unfinished. The optional conformance host's existing response decoder rejects
an uncaught `AggregateError` intrinsic identity; this checkpoint does not widen
that decoder or its final-unhandled-rejection policy. Caught JavaScript values
retain their identity, but no new AggregateError upstream profile or pass count
is claimed. This is not full ECMAScript compatibility, production security,
a CPU/GPU performance result or an RSS bound. Historical checkpoint records and
archives keep their original counts and limits.

The [evidence manifest](evidence/aggregate-error/manifest.json) and
[source/check archive](evidence/aggregate-error/sources-and-checks.tar.gz) are the
publication targets for the selected sources, controls and observed receipts.
