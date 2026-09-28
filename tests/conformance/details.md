# Details and summary: implemented subset

This is focused, self-authored coverage, not an HTML conformance claim. The
implementation follows the [HTML disclosure algorithms](https://html.spec.whatwg.org/multipage/interactive-elements.html#the-details-element),
[activation model](https://dom.spec.whatwg.org/#concept-event-dispatch), and
[default rendering](https://html.spec.whatwg.org/multipage/rendering.html#the-details-and-summary-elements).

## DOM and activation

The first direct HTML-namespace `summary` child is the disclosure summary.
Preceding text/elements do not replace it; later summaries are ordinary content.
A closed disclosure suppresses all remaining content, including positioned
descendants. Author CSS hiding the first summary does not promote a later one.
An absent summary gets a generated “Details” header without adding a DOM node.

`open` is a presence-based boolean attribute, reflected by `details.open`.
The exact, nonempty `name` value groups disclosures in the same DOM tree.
Opening a member closes its currently open peer. Insertion or changing a name
preserves an existing open destination member and closes the incoming member.
Detached trees and template-content fragments have their own groups. Attribute
and tree mutation hooks maintain the index for parser and script operations.

Native activation captures the applicable summary from the original event path.
Click listeners run before the default action, and cancellation prevents the
toggle. The captured summary is checked again after listeners mutate the tree.
Supported nested links, buttons and input controls retain their own actions.
Clicking the details background does not toggle it. A typed generated-summary
hit action distinguishes the anonymous header; ERW7 validates its node kind and
absence of a real summary before accepting the hint.

Disclosure visibility is separate from document activity and disabled state.
Closed descendants remain queryable, their scripts/styles participate, and
successful form controls retain their submitted values. They cannot receive
native focus or native editing while closed. Accepted snapshots revoke a
newly hidden native edit focus even before a pending edit acknowledgement.

## Toggle checkpoint

DOM mutations queue records rather than synchronously invoking listeners.
Repeated transitions with a tracker coalesce, retaining the tracker's old state
and the latest new state; the replacement task moves to the queue's end. Equal
initial/final states are still observable. Merely changing a present `open`
attribute's text does not queue another transition.

The task queue, each element's task tracker, and the active task are separate.
Starting a task removes it from the queue without clearing or replacing its
element's tracker. A reentrant mutation inherits that tracker's old state.
Finishing clears the element's tracker unconditionally, even if a callback
replaced it, while preserving any queued replacement task. Such an untracked
task can coexist with a newer tracked task; starting it must preserve the newer
tracker. For example, closing a just-opened element from its first toggle
listener yields `closed → open`, then `closed → closed` notifications. Closing
and reopening it in that listener yields two `closed → open` notifications.

Page loading (after the current readiness dispatch), native clicks, and native
input edits run an explicit host checkpoint. It dispatches at most 64 records
under one 100,000-step interpreter budget. Event/path allocations and task-map
work are preflighted before beginning a task. Unstarted tasks survive a preflight
failure or batch limit for a later checkpoint. An active task is finished even
after callback resource termination; its callbacks are not replayed and its
queued replacements are preserved. Listener-triggered transitions also consume
that batch's budget. See [event coverage](events.md) for the implemented
`ToggleEvent` interface and interpreter limits.

The native host schedules bounded continuation batches while disclosure work is
pending, with ordinary input requests taking priority. A checkpoint error
suspends automatic task dispatch until reload, retaining unstarted tasks;
ordinary listener exceptions are reported without failing the checkpoint.
Direct Runtime callers retain explicit retry control. This is not a complete
browser task loop: timers, microtask checkpoints, general task-source ordering
and streaming-parser event timing remain unsupported. Native events use trusted
toggle state; script-constructed events remain untrusted.
Because the checkpoint consumes a record before callbacks, reentrant state
coalescing differs from HTML's tracker, which stays present during firing; the
[event coverage](events.md) gives the exact closed/open example.

## Rendering, input, and bounds

The normal block rendering path orders the summary before other children.
UA disclosure markers honor supported `display`, `list-style-type`, and
`list-style-position` overrides. Generated headers and markers use existing
text/display-list budgets. The fallback header contributes intrinsic width.
Visibility is precomputed in one tree traversal; first-summary lookups are
cached and invalidated by child-list mutations, avoiding repeated broad scans.

The group index retains at most one open member per group. FIFO tasks with
non-reentrant host checkpoints retain at most two live task records per node:
an older untracked task and a newer tracked task. Each node has at most one
tracker, and the document has at most one active task. Repeated transitions
replace the tracked task rather than appending unbounded stale records. These
structures remain bounded by the existing 100,000-node arena limit; retained
DOM text has its separate 32 MiB limit. Group/name work is precharged at script
mutation entry points, including
bulk cloning and fragment import. Existing layout visitation, geometry, paint,
glyph, and combined scope limits remain in effect.

Tab includes visible first summaries and visible generated headers. Enter and
Space use the native click path; pointer activation focuses the header. Native
keyboard handling remains the existing pressed-event model: full key event
cancellation, Space keyup timing, tabindex ordering, and an accessibility tree
are not implemented here.

Rendering does not model the specified internal shadow tree or a separate
`::details-content` wrapper. Unusual flex/grid styling of details is therefore
partial, and the generated fallback uses a normal block contents path. Inside
markers currently indent all wrapped lines, rather than only the first line.
`::marker` styling, general CSS counters, synthetic `HTMLElement.click()`
activation, and fragment/find-in-page ancestor revealing are not provided by
this increment. Script `dispatchEvent` does not acquire native default actions.

## Regression evidence

- DOM: first-summary cache invalidation, 20,000 broad siblings, exact name
  groups, detached roots, bulk moves, repeated transition coalescing, task
  identities, tracker cleanup and 20,000 deterministic queue transitions.
- Layout: summary-first order, closed fixed descendants, intrinsic widths,
  generated-header hits and clipping, no synthetic DOM insertion.
- Page: click cancellation and ordering, listener reparenting/removal,
  nested control behavior, background versus generated-header activation,
  active scripts/styles and successful controls within closed contents.
- Native/IPC: Tab and keyboard routing, clipped generated-header exclusion,
  outstanding-edit revocation, round trips and forged hit rejection.
- Independent rectangle references: `details-closed-open`,
  `details-nested-positioned`, and `details-first-summary-order` under
  [`tests/reftests`](../reftests/), at the shared 320 × 240 viewport.
