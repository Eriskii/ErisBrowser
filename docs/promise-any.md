# Promise.any

`Promise.any` consumes a synchronous iterable and fulfills the built-in Promise
result on the first fulfillment. If every input rejects, it rejects with an
intrinsic AggregateError whose dense `errors` Array preserves input order and
the original rejection values. Empty input rejects with an empty errors Array.
Generic receivers return their capability object. The method has name `any`,
length one and no constructor behavior.

Capability construction precedes a single `C.resolve` lookup, followed by
iterator acquisition and a cached `next`. Each input goes through that resolve
and an observable `then` lookup/call. The original capability resolve is passed
directly to every `then`; custom callbacks retain their return and throw behavior.
Iteration continues after synchronous fulfillment. Each rejection callback has
its own once gate. A sentinel prevents synchronous rejections from completing
the aggregate before normal iterator exhaustion; duplicate callbacks return
`undefined` without inspecting storage that may already have moved.

The all-rejected result uses the saved AggregateError prototype, its private
Error brand and the intrinsic Array machinery. It creates an own writable,
configurable, nonenumerable `errors` property, without own message or cause.
Replacing global constructors or inherited setters does not intercept creation.
A saved final rejection callback returns the custom reject result or propagates
its throw. At normal exhaustion, an ordinary throw from the first reject call
reaches the outer reject once more; a second throw escapes, and the completed
iterator is not closed. Terminal errors bypass this ordinary retry.

Ordinary iterator acquisition and step failures reject without closing. Later
ordinary resolve/then failures close an open iterator before rejection, retaining
the original pending error across ordinary close failures. Resource-limit and
unsupported-operation failures stop author close/reject work and suspend
unstarted jobs. Allocation failure during a final rejection callback leaves
its gate consumed and Group storage moved; later checkpoints do not resume the
queued work.

Existing Group/Element arenas and the shared FIFO are reused. Public dispatch
costs eight work units and the body 17 plus one retained reject copy. Creating a
rejection callback costs 136 before growth; first completion costs 28 and
duplicates 12. Intrinsic
error creation adds an eight-unit envelope before separately charged helpers:
134 units on the tested warm fixture, with growth additional. These logical
tariffs are not CPU instruction counts or bounds on recursive object destruction.

Late installation adds one metadata bag after AggregateError, preserving every
prior ID and the old isolated Promise/AggregateError installer fees. The measured
constructor order grows from length/capacity 9/9 to 10/10. This branch adds
189 work units and 3,147 accounted bytes, including the extra upfront object
slot. A fixture with physical order spare separately exercises the 171-unit
branch. Raw startup consumes 100,986 units and leaves 899,014 of 1,000,000;
accounted allocation is 1,765,018 bytes. There are 755 objects, 300 native entries,
26 prototypes and 213 globals. Successful public construction still supplies
100,000 author units, sharing the unchanged cumulative 8 MiB heap ceiling.

Four actual-layout runs retain the previous enum, record and runtime widths on
the tested x86_64 Linux configurations. No new arena or Runtime field is added.
These observations and logical heap charges are not RSS or native-stack
measurements and do not establish layouts on other targets.

The public controls pass 26 groups across 62 strict/sloppy mode attempts.
Eleven private groups pass 92 measured operations, covering admission, partial
publication, paid payload ownership, reserve failure and duplicate callbacks.
The additive final-callback group passes both modes and four sticky queue
readbacks.

The first full-suite attempt stopped after 2,848 library passes and nine
failures, before later targets: eight socket/TCP permission denials and one
missed old constructor-property count. The latter
receives a single assertion migration from nine to ten; production and selected
quota expectations remain unchanged. The rerun uses the required test permissions.

The complete runs pass 3,197 Rust 1.88 default tests and 3,551 Rust 1.99
tests with `test262-host,vulkan-raster`, each across 52 result rows with no
failed, ignored or filtered tests. The parent rosters survive four selected
boundary-name renames and gain all 38 new groups. Strict all-target Clippy passes
in both configurations, and formatting passes.

The rebuilt Rust 1.88 ordinary and host adapters preserve all 1,100 case
records and 400 harness records across the five existing profiles. Complete
arrays match the published parent, including failed and unsupported outcomes;
there are no baseline or policy changes. Later changes after those builds are
test-only, with unchanged production inputs bound in the evidence.

A dedicated Promise.any upstream selection remains separate work. The current
language subset still excludes classes, Proxy, async functions and multiple
realms. Conformance decoder identities and final-unhandled-rejection policy
remain unchanged; browser rejection notifications are still absent.
This checkpoint makes no full-web-compatibility, security-audit, process-memory,
GPU or Chromium-relative performance claim. Earlier chapter results and evidence
archives remain historical.

The [evidence manifest](evidence/promise-any/manifest.json) and
[source/check archive](evidence/promise-any/sources-and-checks.tar.gz) bind the
selected source, controls, accounting and observed validation records.
