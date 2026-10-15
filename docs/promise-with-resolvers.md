# Promise.withResolvers

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
