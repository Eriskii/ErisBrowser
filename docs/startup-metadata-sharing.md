# Shared startup metadata payloads

Document.title accessors now borrow the existing immutable `length` and `name`
keys. Six ProcessingInstruction and CharacterData methods reuse their function
name payload as the owner property key, and CharacterData.length reuses the
pooled `length` key. Mutable metadata bags, function values, descriptors,
property order and existing object IDs remain separate and unchanged. This adds
no API behavior.

The title installer pays an eight-unit handoff before its existing entry charge;
each of its two factories pays two units before retaining the borrowed keys.
Each of the ten PI factories pays one unit after display-text admission to
retain that immutable payload as a key. Six callers consume the key for their
owner property; four accessor callers discard it. CharacterData.length pays
one unit before retaining the pooled key. Existing object, property and native
allocation admissions keep their order and charges.

Together these changes save **279 startup work units and 1,132 accounted heap
bytes**: title keys save 60/336, PI owner keys save 200/708, and the length owner
key saves 19/88. Raw startup consumes **99,694 work units**, leaving **306**
under the unchanged 100,000-unit limit. The cumulative accounted-heap limit stays
**8 MiB**. Public Runtime construction still resets execution work after startup
while retaining initialization heap charges.

The focused Rust 1.88 default observations record accounted allocation falling
from **1,745,449 to 1,744,317 bytes**. Both observations retain 752 ordinary objects
and capacity, 299 native registry entries, 25 prototypes and 212 global bindings.
These are metered accounting observations, not resident-memory or CPU-performance
measurements. Portable private assertions use type-size formulas instead of
hardcoding these absolute allocation totals.

All three parent-compatible witnesses fail at their intended new boundaries:
the first method owner/display pointer comparison, the title getter's borrowed
length-key comparison, and 99,973 versus the new 99,694 raw-work expectation.
Their preceding descriptor, identity and count controls pass. The same three
groups pass on the candidate with unchanged expectations. Four additional
private groups pass 84 operations covering paid clone boundaries, retained
metadata prefixes, exact title factory/installer fees and owner publication
refusals. Existing full PI factory quota controls remain in their original
module. Eight existing test files receive only the reviewed signature,
accounting or explanatory migrations. All 90 focused tests pass: 66 PI tests
and 24 title tests, including all seven new groups.

The first full default run stopped in the library target with 2,790 passes and
one failure. The initial seven-file migration list had omitted the existing
typed-array cumulative startup ledger: its work assertion still expected 27
remaining units and observed 306. The heap assertion and later test targets were
not reached. A separately retained eighth migration appends the already selected
+279 work and -1,132 accounted-byte terms, preserving all prior terms and
assertions. Production and the new controls remain unchanged.

Rust 1.88 default passes **3,131 tests**, and Rust 1.99 with
`test262-host,vulkan-raster` passes **3,485 tests**. Both complete runs use
`--include-ignored` and report 52 result rows, with no failed, ignored or filtered
tests. Their rosters retain every prior test apart from three selected
accounting-test renames and add exactly the seven new groups.

Strict all-target Clippy passes on both configurations. The Rust 1.99 formatting
check also passes.

A rebuilt Rust 1.88 release host adapter preserves all **1,050 case-mode records
and 348 harness records** across the four existing Promise profiles. Complete
case and control arrays match the published parent, including existing failed
and unsupported outcomes. All 348 controls verify, with zero regressions,
improvements or baseline changes. This comparison adds no upstream cases.

Historical checkpoint chapters and conformance baselines retain their original
scope. Promise.any and AggregateError remain separate work: the additional
startup headroom does not establish their admission or implementation. This
checkpoint makes no new GPU, security or Chromium-relative performance claim.

The [evidence manifest](evidence/startup-metadata-sharing/manifest.json) binds
the selected sources, frozen controls, parent observations and validation
receipts. Its [source and check archive](evidence/startup-metadata-sharing/sources-and-checks.tar.gz)
retains the corresponding artifacts.
