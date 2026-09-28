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
unsupported legacy event interface names. Its complete legacy code table and
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
passive and signal in order. Non-null AbortSignal options are explicitly
unsupported. Default passive selection for touchstart/touchmove/wheel/mousewheel
on Window, Document, documentElement and body is implemented.

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
Resource failures are uncatchable, including inside event callbacks. Retained
listener tombstones, event objects and detached nodes remain charged; there is
no garbage collector.

Run the focused checks with:

```sh
cargo test --locked --offline --lib events_
cargo test --locked --offline --lib script::tests
```

The eleven focused event test groups cover readonly state and dictionary conversion,
phase/receiver/path order under DOM mutation, cancellation/passive behavior,
listener identity and mutation, callback objects, once/reentrancy, reported
exceptions, inline handler ordering, detached/template paths, readiness/trust,
and quota termination with cleanup. Assertions made inside listeners are also
checked through the runtime console so swallowed assertion exceptions cannot
silently pass a test. The pre-existing click/input/textarea/page tests continue
to exercise native integration.

No new upstream WPT pass count is claimed: an unchanged browser WPT harness and
its required host environment are not yet supported. Existing pinned HTML and
Test262 inventories are unchanged. Future event WPT imports must retain source
bytes and explicit unsupported outcomes instead of replacing the harness with
native assertion stubs.

Remaining work includes Shadow DOM and retargeting/closed-tree composed paths,
AbortController/AbortSignal, UIEvent/MouseEvent/PointerEvent/KeyboardEvent and
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
[HTML event handlers](https://html.spec.whatwg.org/multipage/webappapis.html#event-handlers),
and [Web IDL callback conversion](https://webidl.spec.whatwg.org/#es-callback-function).
