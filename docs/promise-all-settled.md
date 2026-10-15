# Promise.allSettled

`Promise.allSettled` collects outcomes from a synchronous iterable in input
order. Each fulfilled input produces `{ status: "fulfilled", value }`; each
rejected input produces `{ status: "rejected", reason }`. With the built-in
`Promise` receiver, empty input fulfills the returned promise with an empty
Array. A generic receiver returns its capability object. Reactions run through
the existing microtask FIFO. The method has name `allSettled`, length one and
no constructor behavior.

The [standard algorithm](https://tc39.es/ecma262/multipage/control-abstraction-objects.html#sec-promise.allsettled)
creates a capability from its receiver, reads `C.resolve` once, then acquires
the iterator and its cached `next` method. Every input goes through the cached
resolve and an observable `then` lookup/call. Its two fresh callbacks share one
called gate: only the first invocation stores a result and decrements the
remaining count. A sentinel prevents synchronous callbacks from completing the
collection before normal iterator exhaustion. After exhaustion, the last
completing callback returns the custom capability resolver’s result or propagates
its thrown value. Duplicate callbacks return
`undefined`.

Each outcome is a fresh ordinary object using the saved intrinsic Object
prototype, with `status` inserted before `value` or `reason`. These are writable,
enumerable, configurable own data properties. The final dense Array uses the
intrinsic Array prototype. Replacing global constructors or installing inherited
setters does not intercept creation; snapshot payloads are retained without
thenable assimilation.

Ordinary acquisition and iterator-step errors reject without closing. Later
ordinary resolve/then failures close an open iterator before rejection, retaining
the original error across ordinary close failures. Resource-limit and unsupported
operation failures stop further author close/reject calls and suspend queued,
unstarted jobs. Separate readback entries do not resume those jobs. The operation
performs no budget reset or automatic retry.

The implementation reuses the existing Group/Element storage and shared FIFO.
Public dispatch adds eight work units; the body prepays 16. A callback pair costs
252 before growth. A first completion pays 12 for its gate, 14 for completion
validation and 17 for the snapshot envelope, before separately charged lookup,
object storage, text and property operations.
Duplicate callbacks pay the gate and return without inspecting moved-out result
storage. Snapshot refusal leaves the gate consumed while the Group slot and count
remain unchanged. These accounting envelopes do not bound native stack, RSS or
inherited object-graph destruction.

Installation adds 187 work units and one metadata bag. Raw startup uses 99,973
work units, leaving 27 under the unchanged 100,000-unit allowance. Exact and
one-short startup/installer controls pass in the complete default suite. The
cumulative accounted-heap limit remains 8 MiB. No new arena, Runtime field or
cap is introduced.

The focused runs pass all 35 new groups: 15 ordinary public, five terminal-policy,
12 private accounting/refusal, one actual-layout and two Page/worker integrations.
The private groups cover 138 measured helper operations, including staged work
and heap refusals, shared-gate state, late reserve failure and Ready transfer.

The Page/worker groups use a procedural pixel oracle: red x12 with title `SQA`,
then x16 with `SQACKDB` after a click; disabled scripts retain x2 and title `base`.
The blue rectangle stays at x20. Both routes check two widths, hits, task polling
and diagnostics. Optional packed-plan checks use the CPU interpreter; this is
not a GPU or Chromium-relative performance measurement.

One public fixture needed a setup correction. Its ordinary property descriptor
inherited `Object.prototype.value` after installing a poison accessor, making a
later accessor descriptor invalid. A null-prototype descriptor preserves the
intended poison-setter checks. All result expectations stayed unchanged, and no
engine change was made for this correction. The original fixture and failed
candidate observation remain in the evidence record. Parent missing-method runs
had stopped before this setup; later rows were not observed there.

A separate source review added an explicit spare object slot to the private
fixture measuring a warm snapshot after callback-pair creation. Its quota
expectations stayed unchanged; the correction preceded private execution.

The complete Rust 1.88 default suite passes 3,124 tests; Rust 1.99 with
`test262-host,vulkan-raster` passes 3,478. Both use `--include-ignored` and report
52 result rows with no failures, ignored or filtered tests. The first default
sandboxed attempt stopped at eight local TCP `PermissionDenied` preflights before
integration targets; the successful rerun used unchanged source.

Strict Clippy passes for all targets on both configurations, and formatting
passes. The rebuilt Rust 1.88 release host preserves all 842 case-mode records
and 254 harness records across the three existing Promise profiles. Complete
case and control arrays match the published Text checkpoint, including existing
failed and unsupported outcomes; all 254 controls verify. There are no
regressions, newly passing cases or baseline changes.

The selected source map records 1,549 inputs: all 1,540 parent inputs plus nine
new files. It is not a complete dependency/build-graph inventory. Earlier
selected maps and historical evidence archives remain unchanged.

An authenticated allSettled upstream profile is subsequent work; this checkpoint
adds no upstream case or pass count. Support remains within the current language
subset: synchronous iterables and the existing single-realm Promise machinery.
Classes, async functions, `Promise.any` and browser rejection notifications remain
unfinished. Full ECMAScript and browser compatibility remain incomplete; these
bounded resource and isolation checks are not a general security proof.
Historical [Text metadata accounting](text-metadata-keys.md) records its own
earlier checkpoint. The [evidence record](evidence/promise-all-settled/summary.json)
binds frozen controls, retained corrections, source reviews and validation.
