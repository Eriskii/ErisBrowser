# Promise.withResolvers

## Complete pinned upstream profile

The `promise-with-resolvers` profile retains all six original files in
`built-ins/Promise/withResolvers` at Test262 revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`, in both sloppy and strict modes.
It records **10 passed and two unsupported modes**, with all **44 controls
verified**. Both unsupported rows are `ctx-ctor.js`: its declared `class`
feature is excluded before adapter execution. No source or mode is removed
because of its outcome.

The 19-file package includes the six sources, three unchanged helpers
(`assert.js`, `sta.js` and `propertyHelper.js`), legal files, complete directory
proofs linked to the pinned root, and a manifest. An offline importer replay
reproduces every packaged byte. All six sources are synchronous; this profile
uses ordinary ERJS1 requests and ERJR2 responses with the existing seven
intrinsic error identities. It adds no Promise host requirement or async
completion policy. Class syntax remains unsupported.

The canonical gate retains 32 existing ordinary controls and adds three
positive/mismatch pairs in both modes. They exercise generic capability
capture and constructor validation, genuine pending Promises and resolver
metadata, and fresh result objects with exact callback identities and data
properties. Each mismatch changes one final assertion. Pending `then` is a
synchronous brand check, not asynchronous settlement coverage. Control success
means the expected result was verified, including deliberate assertion failures.

Admission rebuilds source, metadata, mode and ordered-helper fingerprints;
checks exact result and prerequisite records; validates the six ordinary raw
fields; and recomputes classification. A completion-policy key is refused
before encoding or dispatch, and ERJR3 responses are rejected. A bad gate blocks
all 12 source modes, including metadata-excluded rows, and cannot create or
overwrite a baseline.

All **652 Python tests pass**, retaining the 624 parent test identities and
adding 28 groups. The first focused candidate run passes all 28. Healthy
recording and replay retain the exact 12 case and 44 control arrays. The new
baseline stores a compact map of the 12 statuses and case/source fingerprints,
with corpus, policy and binary bindings; it is not a copy of the full reports. The six older profiles keep
their complete 1,288 case and 502 control records, including failed and
unsupported outcomes, with unchanged policies and baselines.

Two orchestration assertions were corrected after complete reports had been
written: the initial recipe expected exit zero despite retained unsupported
rows, and the first replay verifier compared the compact baseline with a full
report. Neither correction changes source, controls, test expectations or raw
results. Those initial records remain in the evidence. A separate evidence-parser
correction deduplicates repeated parent-test summary lines without changing raw
logs or project results.

This checkpoint changes tooling, corpus data, tests and the adjacent ordinary
CI gate. It reuses the published runtime checkpoint's exact Rust 1.88 ordinary
and host adapters; Rust sources, build settings, wire bytes and runtime limits
are unchanged. No new Rust suite, Clippy or build result is claimed. The CI step
is configured here; no remote result for this checkpoint is claimed. These
selected results establish neither full ECMAScript compatibility nor a security
or performance result.

The [upstream evidence manifest](evidence/promise-with-resolvers-upstream/manifest.json)
and [source/check archive](evidence/promise-with-resolvers-upstream/sources-and-checks.tar.gz)
retain the source contracts, original reports, compact baseline and validation
receipts. The earlier runtime record below keeps its historical scope.

## Runtime checkpoint

`Promise.withResolvers()` returns a fresh ordinary object containing `promise`,
`resolve` and `reject`, in that property order. Each is an own writable,
enumerable and configurable data property. The method has name `withResolvers`,
length zero and no constructor behavior.

The operation first creates a capability using its dynamic receiver as the
constructor. A custom constructor can return an arbitrary object and supply any
callable resolve/reject values; their exact identities move into the result.
The result uses the saved intrinsic Object prototype and own data properties,
so replacing the global Object or installing inherited setters does not
intercept publication. The method performs no `C.resolve`, species or iterator
lookup and does not invoke the returned callbacks. Extra arguments are ignored
after ordinary argument evaluation.

With the intrinsic Promise constructor, the returned callbacks retain existing
first-settlement, thenable assimilation and shared FIFO behavior. Ordinary
constructor throws propagate synchronously, preserving the thrown value.
Invalid constructors, missing callbacks and noncallable callbacks throw
TypeError. Resource and unsupported-operation failures retain terminal
suspension: author catch/finally work and queued jobs do not resume through a
later checkpoint.

The implementation reuses existing capabilities and adds no Runtime field,
state type or callback variant. Public dispatch retains its eight-unit charge;
the outer operation adds three units for capability ownership. The result
factory costs 156 on the tested warm fixture, with capability creation and
growth charged separately. These are logical work tariffs, not CPU instruction
counts or process-memory measurements.

Late installation adds one metadata bag after `Promise.any`, preserving prior
IDs and isolated helper fees. The measured constructor order grows from
length/capacity 10/10 to 11/11. That branch adds 326 work units and 3,253 accounted
bytes, including the extra upfront object slot; a spare-order fixture covers
the 306-unit branch. Raw startup consumes 101,312 units, leaves 898,688 and
accounts for 1,768,271 bytes in the recorded x86_64 run. There are 756 objects,
300 native entries, 26 prototypes and 213 globals. Trusted initialization stays
bounded at 1,000,000 work units; successful public construction still supplies
100,000 author units under the cumulative 8 MiB heap ceiling.

The published parent fails all 15 new public groups at the missing-method guard
in their first sloppy attempt. Later semantic phases, strict attempts and
terminal readbacks are unreached there. All 24 new groups pass on the first
candidate run: 12 semantic groups, three terminal groups and nine private
groups covering 67 measured operations. No candidate expectation correction is
needed.

Complete `--include-ignored` runs pass 3,221 Rust 1.88 default tests and 3,575
Rust 1.99 tests with `test262-host,vulkan-raster`. Each has 52 result rows and no
failed, ignored or filtered tests. The full parent rosters survive four boundary
renames and gain the 24 new groups. Strict all-target Clippy passes in both
configurations, and formatting passes.

Rebuilt Rust 1.88 ordinary and host adapters retain all 1,288 case records and
502 harness records across the six existing upstream profiles. Complete arrays,
policies and corpus identities match the parent, including existing failed and
unsupported outcomes; no baseline changes are made. Tooling and CI gates are
unchanged, and no full Python-suite rerun is claimed.

There is no dedicated withResolvers upstream profile in this checkpoint. Six
pinned sources were authenticated as references only. Classes, async functions
and multiple realms remain outside the current language subset. The evidence
binds 1,747 selected inputs plus a separate tracked-source census; this is not a
complete build-graph inventory or a full-conformance, security or performance
result. Earlier chapters and validation sections retain their historical scope.

The [evidence manifest](evidence/promise-with-resolvers/manifest.json) and
[source/check archive](evidence/promise-with-resolvers/sources-and-checks.tar.gz)
bind the selected source, controls, accounting and observed validation records.
