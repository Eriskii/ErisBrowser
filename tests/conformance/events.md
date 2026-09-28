# DOM events implementation and focused checks

Eris implements a bounded synchronous Event/EventTarget core in its custom Rust
interpreter. This is a focused implementation, not full DOM, HTML, Web IDL or
browser event conformance. No browser or JavaScript engine supplies dispatch.

The implemented interfaces are `Event`, `CustomEvent` and `EventTarget` constructors;
`addEventListener`, `removeEventListener` and `dispatchEvent`; the Event phase
constants and state getters; `preventDefault`, `stopPropagation`,
`stopImmediatePropagation`, `composedPath`, `initEvent` and `initCustomEvent`.
`document.createEvent` supports Event/Events/HTMLEvents and CustomEvent aliases.
A limited `DOMException` constructor supplies the InvalidStateError used for
uninitialized or recursively redispatched events, and NotSupportedError for
unsupported legacy event interface names. AbortError (code 20) and ABORT_ERR
are also provided for default abort reasons. Its complete legacy code table and
Web IDL property layout remain incomplete.

Event state lives in private runtime slots. Ordinary writes cannot change
readonly metadata or the dispatch algorithm's state. `isTrusted` is an own,
non-configurable getter. Author construction and `dispatchEvent` produce
untrusted events; native entry points create trusted events. `timeStamp` is a
monotonic millisecond value relative to runtime creation. CustomEvent detail
preserves the supplied JavaScript value and its identity.

Dispatch fixes the parent path before invoking callbacks. Connected nodes reach
their Document and then Window, which have distinct listener registries.
Detached subtrees end at their root; template contents end at their separate
DocumentFragment and do not propagate to the template host. A Document's `load`
event has no Window parent. Capture runs from the outermost ancestor toward the
target, then target and bubbling invocations run outward when applicable.
Capture and bubble listeners both see AT_TARGET on the target. The later target
bubble invocation is suppressed when a target capture listener stops propagation;
other listeners in that same capture invocation still run unless immediate
propagation is stopped. This follows DOM's separate invocation steps.

Listener identity includes type, callback object and capture, and does not
include once or passive. Null callbacks are ignored. Callback functions receive
the current target as `this`; callback objects dynamically retrieve and invoke
`handleEvent` with that object as `this`. Each invocation snapshots listener IDs,
while removals remain visible through retained removed flags. Additions can run
in a later invocation or dispatch, and once listeners are removed before calling
them, including for recursive dispatch. Options conversion reads capture, once,
passive and signal in order. Supplied signal values must be genuine AbortSignal
objects; undefined is absent, while null and forged objects throw TypeError even
for null callbacks or duplicate registrations. Already-aborted signals suppress
registration. Default passive selection for touchstart/touchmove/wheel/mousewheel
on Window, Document, documentElement and body is implemented.

`AbortController` has a stable readonly `signal`, and idempotent `abort(reason)`.
`AbortSignal` inherits EventTarget and supplies readonly `aborted` and `reason`,
`throwIfAborted`, the `onabort` handler, and static `AbortSignal.abort(reason)`.
Signals cannot be constructed directly. Private slots hold controller identity,
reason and listener-removal registrations; public property overrides cannot
change that state. Supplied reasons retain their exact value and identity, while
omitted or undefined reasons create an AbortError DOMException. Throwing the
reason does not inspect its properties. Static abort creates an already-aborted
signal without dispatching an event.

Controller abort removes registered listeners before synchronously firing a
trusted, non-bubbling, non-cancelable abort event at the signal. Listeners removed
this way are skipped in active dispatch snapshots. Duplicate registrations do
not change the original listener's signal; removing and re-adding a listener
creates a distinct registration. Reentrant abort preserves the first reason and
does not fire a second event. Synthetic `dispatchEvent(new Event('abort'))` does
not change signal state. Ordinary callback exceptions are reported while later
listeners continue. No fetch cancellation, dependent-signal `any`, timer-based
`timeout`, asynchronous task queue, or cross-thread signal transfer is provided.

Cancellation respects cancelable and passive state. `dispatchEvent` returns false
only when the event was canceled. Legacy returnValue/cancelBubble setters cannot
undo cancellation or propagation stopping. Common HTML `on...` IDL and content
handlers share listener ordering, support replacement/removal and honor a false
return value. Non-callable object assignments to these legacy handler properties
are preserved but do nothing when invoked. Content handlers compile lazily;
syntax failures and ordinary callback exceptions are reported to the runtime
console without aborting later listeners. Resource termination and explicit
unsupported-feature errors propagate. Dispatch always clears currentTarget,
eventPhase, path, passive and dispatch/stop flags, including on termination;
ordinary target and canceled state remain available afterward.

The existing native API remains `dispatch_event(node, type, document) ->
Result<()>` plus `last_default_prevented`, and `dispatch_click` remains a wrapper.
Native click/submit and the listed cancellable UI event types can cancel their
host action; input/change are non-cancelable. The readiness hook fires
DOMContentLoaded at Document (bubbling to Window), then load at Window with the
legacy Document target override, once per runtime. This hook is synchronous and
does not implement the complete HTML loading/task lifecycle.

Work accounting covers path construction, listener scans/snapshots, long UTF-16
event-type map comparisons, default-passive DOM scans, cached handler-source
comparisons, content-handler parsing,
callback execution and result/error storage. Paths have at most 258 targets;
reentrant dispatch shares the existing weighted native stack allowance,
100,000-step execution allowance and 8 MiB cumulative allocation allowance.
Signal state and observer IDs are charged to the same allocation allowance.
Abort preflights the complete observer-list traversal before changing state or
removing listeners; abort callbacks use the same dispatch and stack limits.
Removed observer IDs may remain charged until their signal aborts. Throwing a
string reason reserves proportional work and UTF-8 diagnostic allocation before
formatting; the same preflight applies to ordinary throw statements, without
reading properties from object reasons.
Resource failures are uncatchable, including inside event callbacks. Retained
listener tombstones, event objects and detached nodes remain charged; there is
no garbage collector.

Run the focused checks with:

```sh
cargo test --locked --offline --lib events_
cargo test --locked --offline --lib abort_
cargo test --locked --offline --lib script::tests
```

The seventeen focused event/abort test groups cover readonly state and dictionary conversion,
phase/receiver/path order under DOM mutation, cancellation/passive behavior,
listener identity and mutation, callback objects, once/reentrancy, reported
exceptions, inline handler ordering, detached/template paths, readiness/trust,
signal receiver/identity/reason semantics, option conversion and duplicate handling,
abort-time removal ordering, and quota termination with cleanup. Assertions made inside listeners are also
checked through the runtime console so swallowed assertion exceptions cannot
silently pass a test. The pre-existing click/input/textarea/page tests continue
to exercise native integration.

No new upstream WPT pass count is claimed: an unchanged browser WPT harness and
its required host environment are not yet supported. Existing pinned HTML and
Test262 inventories are unchanged. Future event WPT imports must retain source
bytes and explicit unsupported outcomes instead of replacing the harness with
native assertion stubs.

Remaining work includes Shadow DOM and retargeting/closed-tree composed paths,
AbortSignal.any/timeout and cancellation integration with other APIs,
UIEvent/MouseEvent/PointerEvent/KeyboardEvent and
other specialized event interfaces, relatedTarget/touch lists, activation/default
behavior for author-dispatched events, asynchronous tasks and microtasks,
cross-realm dispatch, full Window/global property bindings (including window.event),
body/frameset handler forwarding, complete handler-name/IDL reflection, special
ErrorEvent/beforeunload handling, and the HTML element/form/document name scopes
used by inline handlers. The DOM hierarchy has no shadow roots, so composed is
stored and exposed but cannot yet cross a shadow boundary. The current DOM
wrapper prototype hierarchy is simplified. These limitations prevent a general
web compatibility claim.

The primary references are the [DOM Event interface](https://dom.spec.whatwg.org/#interface-event),
[EventTarget interface](https://dom.spec.whatwg.org/#interface-eventtarget),
[event dispatch algorithms](https://dom.spec.whatwg.org/#dispatching-events),
[AbortController](https://dom.spec.whatwg.org/#interface-abortcontroller),
[AbortSignal](https://dom.spec.whatwg.org/#interface-abortsignal),
[HTML event handlers](https://html.spec.whatwg.org/multipage/webappapis.html#event-handlers),
and [Web IDL callback conversion](https://webidl.spec.whatwg.org/#es-callback-function).
# Disclosure toggle events

`ToggleEvent` extends the existing Event implementation with private `oldState`,
`newState` and `source` data. Its constructor performs type and dictionary
conversion in Web IDL order, preserves UTF-16 state strings, and accepts an
Element or null as source. Its readonly getters validate the receiver; synthetic
events are untrusted. The existing event dispatch, cleanup, exception reporting
and uncatchable resource limits apply. Shadow-root retargeting remains outside
the DOM subset, so source is returned directly within supported ordinary trees.

HTML `details` elements reflect `open` as a boolean and `name` as a string;
foreign elements named details do not acquire these bindings. Boolean writes do
not invoke author conversion methods. String writes use string-hint conversion
and the existing explicit UTF-16-to-UTF-8 DOM boundary. The `ontoggle` property
and inline attribute use the existing event-handler machinery.

The DOM coalesces disclosure notifications. The host calls
`Runtime::dispatch_details_toggles` at its integration checkpoints; mutations do
not synchronously dispatch an event. Each checkpoint processes at most 64 queued
notifications with one shared 100,000-step allowance, the persistent 8 MiB
allocation allowance and existing stack limits. Native notifications are trusted
ToggleEvents with non-bubbling, non-cancelable defaults and null source. Ancestor
capture listeners can observe them. Reentrant changes queue another notification.

Event/path allocation and bounded task-map work are preflighted before starting
a queued notification. A task that has begun dispatch is finished even if a
listener exhausts quota; its callbacks are not replayed. Tasks not yet started,
including callback-created replacements and those beyond the 64-task limit,
remain queued. A nested host checkpoint is rejected before resetting work.
The native host continues pending batches with input priority; a checkpoint
error suspends automatic dispatch until reload. Direct Runtime callers retain
explicit retry control. This bounded host checkpoint does not implement the
complete HTML event loop, microtask checkpoints, timers, general task-source
interleaving or full timing guarantees. No popover/dialog activation or
`beforetoggle` default action is supplied by the constructor.

Task identity, the element's tracker and active dispatch state are independent.
Starting a task preserves the existing tracker; mutations during callbacks
inherit its old state and cancel its referenced task only if still queued.
Finishing clears that element's tracker even if a callback replaced it, without
removing the replacement task. A listener closing an element during its
closed-to-open notification therefore queues a closed-to-closed notification.
An untracked older task can coexist with a newer tracked task and must preserve
that newer tracker when it begins. Nested synthetic dispatch changes none of
these host task identities. FIFO ordering and rejection of nested host
checkpoints bound storage to two live task records per element, one tracker
per element, and one active task per document.

Ten focused runtime groups are selected by
`cargo test --locked --offline --lib script::tests::disclosure_`. They cover
constructor getter/coercion order, receiver brands and readonly state, reflection
and namespaces, deferred/coalesced delivery, trusted flags and capture,
reentrant/detached targets, multiple listeners, nested synthetic dispatch,
named-group effects, untracked/newer-task interleaving across the batch boundary,
reentrancy guards, queue limits, preservation of unstarted tasks, and
uncatchable constructor/mutation/callback exhaustion and preflight of named-group
work before clone/import allocation or target replacement. These are local regression
tests, not upstream WPT conformance counts. DOM/layout/activation scope is
documented separately in [details.md](details.md).

Primary references: [HTML ToggleEvent](https://html.spec.whatwg.org/multipage/interaction.html#the-toggleevent-interface)
and [details notification and grouping algorithms](https://html.spec.whatwg.org/multipage/interactive-elements.html#the-details-element).
