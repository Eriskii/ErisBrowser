# Microtask callbacks and host checkpoints

`queueMicrotask(callback)` schedules a callable in the current Window realm.
The runtime drains the FIFO when the outermost script or host callback finishes,
including callbacks appended during that drain. Enqueuing returns `undefined`;
missing or noncallable callbacks synchronously throw `TypeError`. A callback
receives no arguments and an undefined receiver through ordinary JavaScript
invocation, preserving strict, sloppy, arrow and bound-function behavior.

For example, the script below finishes with `SABC` after its checkpoint. The
script's own final expression still evaluates to `S`: cleanup does not change
an already computed completion value.

```js
var log = '';
queueMicrotask(function () {
    log += 'A';
    queueMicrotask(function () { log += 'C'; });
});
queueMicrotask(function () { log += 'B'; });
log += 'S';
log;
```

The installed function is one writable, enumerable and configurable Window
property, shared by bare, `window` and initial `globalThis` access. Saved
references survive replacement or deletion of that property. Detached calls
and null/undefined receivers use the operation's realm global; unrelated
receivers throw `TypeError`. Replacing the `globalThis` property does not change
that realm identity. The function has name `queueMicrotask`, length one, no
own `prototype` initially, and cannot be used as a constructor.

## Where callbacks run

An explicit author-entry counter distinguishes host callbacks from nested
JavaScript calls. A host event listener can therefore finish its queued jobs
before the next listener runs. Synchronous `dispatchEvent` called from a
script defers those jobs until the enclosing author entry finishes. An object
listener's `handleEvent` lookup and method invocation share one entry; cleanup
does not run queued work between its getter and method.

Listener cleanup checkpoints occur while `currentTarget`, `eventPhase` and
the passive-listener state still apply. Ordinary return handling, exception
reporting and propagation checks follow. Queued mutations settle before the
existing click default action, form-data collection, SVG refresh and layout.
Script checkpoints also precede the next script and readiness events. The
existing loader runs its bounded script list after parsing and initial resource
preparation; this adds no new async, defer or module scheduling.

Rendering, layout and resource preparation execute no author callbacks.
The worker protocol carries no callback queue. The existing `RunTasks`
mechanism continues to handle disclosure notifications; it does not provide
a separate microtask budget or resume an unfinished drain.

## Resource and failure policy

Jobs retain existing callable handles in a runtime-owned `VecDeque`. Logical
capacity grows geometrically, independently of any extra capacity granted by
the allocator. Capacity, relocation, enqueue, dispatch and cleanup are charged
before publication or removal. The queue and its jobs share the initiating
host entry's existing work allowance and cumulative runtime heap allowance.
There is no per-job reset or successful yield after a fixed number of jobs.

An ordinary callback exception is submitted once to the bounded
`Runtime.console` reporter, then the next callback runs. At its admission
threshold the reporter suppresses the message without suspending the queue. Callback return values
are ignored without coercion or thenable assimilation. Reporting computes its
required size from the immutable error record and pays for formatting and
storage before constructing the message. Its 1,024-entry admission threshold
is specific to this asynchronous reporter; existing console producers retain
their own policies. This path does not implement the full HTML
`ErrorEvent`/`onerror` reporting algorithm or copy messages to Page diagnostics.

A resource failure, unsupported execution or failed report permanently
suspends asynchronous work in that Runtime. A removed job is never replayed.
Unstarted jobs remain bounded and inert until the Runtime is destroyed;
further enqueue attempts fail. Page task state exposes suspension even if the
failure occurred outside disclosure dispatch. Repeated `Render`, `RunTasks`
or a later work-budget reset cannot resume the queue. Later synchronous input
may still execute its existing handlers. Full navigation creates a fresh
Runtime; same-document fragment navigation retains the current one.

These are the engine's explicit bounded-execution policies. Logical charges
are not physical-memory measurements or a guarantee against all allocator
failures. The worker's confinement, resource broker and request deadlines
remain in place.

## Scope

This foundation covers the existing single Runtime/Window realm. Promise jobs,
MutationObserver delivery, rejection notifications, hosted timers, animation
callbacks, worker-global realms and cross-realm callback settings remain
separate work. The existing script loader still runs supported classic scripts
after parsing and resource preparation; this change does not add parser-interleaved
execution. The runtime keeps one FIFO model for later job integration.
This is not a complete HTML event loop or a web-compatibility claim.

The behavior follows the HTML [microtask API](https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#microtask-queuing)
and [cleanup checkpoints](https://html.spec.whatwg.org/multipage/webappapis.html#perform-a-microtask-checkpoint),
Web IDL [callback invocation](https://webidl.spec.whatwg.org/#invoke-a-callback-function),
and DOM [listener invocation](https://dom.spec.whatwg.org/#concept-event-listener-inner-invoke).

## Validation

The Rust 1.88 complete suites pass with all ignored tests included: 3,093 tests
with `vulkan-raster` and 2,789 with default features, each across 43 result
rows. The 27 new groups comprise 11 runtime semantic, four binding, eight
private resource-accounting, two Page and two confined-worker tests.

Before implementation, all 19 new public-API groups compiled on the parent
and failed as expected. The eight private accounting groups depend on new
internal APIs and have no meaningful parent execution. The four Page/worker
parent failures stop at their initial pixel comparison; later ordering checks
are not parent observations. The final fixtures check 12 independently written
RGBA frames across 40 layout or Render calls, including nested events, passive
listeners, default link actions, readiness, fatal suspension and fresh/disabled
realms. Packed native plans are interpreted on CPU in these witnesses.

One initial complete-suite attempt passed 2,673 library tests and failed an old
disclosure test that expected retry after resource failure; Cargo stopped
before later targets. Its fatal branch now checks the selected permanent
suspension policy, repeated refusal without budget or DOM changes, and the
three retained notifications. A Rust fixture inspection checks that inert FIFO;
it does not resume callbacks. This follow-up changed no production code or
limits. Separately, an incorrect reporter-prefix length in the draft accounting
oracles was corrected from 29 to 30 bytes before implementation or execution.
Both original records and additive corrections are retained.

Strict Clippy passes on Rust 1.88 and 1.98, and the formatting check passes.
The final Rust 1.88 browser build renders all 102 reference pairs exactly;
all 204 PNGs also match the previous published checkpoint byte for byte.
The [evidence summary](evidence/microtasks/summary.json) and
[archive](evidence/microtasks/sources-and-checks.tar.gz) bind the final 760 source
inputs, held proposals/oracles, source reviews, complete run logs and source
history. These are authored local tests, not imported WPT conformance results.
Build caches were reused; timings are not cold-build measurements. This
checkpoint makes no new GPU, Chromium-performance or production-security claim.
