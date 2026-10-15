# Shared Text metadata keys

The `wholeText` getter and `splitText` method now reuse the existing immutable
`length` and `name` key payloads from DOM interface initialization. Their two
function values, metadata bags and display-name values remain separate. Editing
or deleting one function's metadata does not change the other function, its
constructor or a saved callable. Property order, descriptors and Text operation
behavior remain unchanged; this adds no API coverage.

The Text installer pays an eight-unit handoff before its existing entry charge.
Each factory retains its original bag and display-text construction, then pays
two units before cloning the borrowed keys. The original metadata insertions,
owner insertions and native callable construction keep their charges and order.
Removing two fresh key pairs saves 72 work units; the handoff and four clones
cost 12, leaving a net saving of **60 work units and 336 accounted heap bytes**.
These are accounting reductions, not measured speed or resident-memory results.

Raw startup now consumes **99,786 work units**, leaving **214** under the unchanged
100,000-unit limit. The cumulative accounted-heap limit remains **8 MiB**. Object
count and capacity stay 751, native registry entries 299, prototypes 25 and global
bindings 212. Public construction still resets the execution work budget while
retaining initialization heap charges. No runtime field, arena, identity, API or
limit is added or changed.

All five new groups pass. Three use existing APIs: both bags' descriptors, order
and distinct identities precede physical key-backing comparisons; saved-callable
and metadata mutation/deletion behavior runs in sloppy and strict modes; raw
startup accepts 99,786, refuses 99,785 and preserves the public budget reset.
The parent passed the behavior group and failed the two intended new boundaries,
after their preceding preservation controls. No expectation changed. Two private
groups add 16 factory/installer operations covering exact and insufficient work
and heap allowances, clone staging and entry handoff. Six existing test
files receive only the reviewed aggregate accounting or cross-reference changes.

The Rust 1.88 default runs pass **3,089 distinct tests**: 2,965 in the ordinary
run and the complementary 124 through explicit `--ignored` execution. Rust 1.99
with `test262-host,vulkan-raster` and `--include-ignored` passes **3,443 tests**,
with no failed, ignored or filtered tests. Each of these three commands reports 51 result
rows. Both configuration rosters preserve their prior tests, apart
from two selected accounting-test renames, and add the five new groups.

Strict Clippy passes for all targets on Rust 1.88 default and Rust 1.99 with
`test262-host,vulkan-raster`; formatting also passes.

A rebuilt Rust 1.88 release host adapter preserves all **842 case-mode records
and 254 harness records** across the three existing Promise profiles. Complete
case and control arrays match the published parent, including existing failed
and unsupported outcomes. All 254 controls verify, with zero regressions, newly
passing cases or baseline changes. This comparison adds no conformance cases.

The selected source inventory now records **1,540 inputs**: an extended
1,538-input parent plus the two new test modules. The extension adds 177
previously omitted literal include inputs, independently authenticated as
unchanged from the published parent. Earlier selected maps and historical
archives remain retained; expanding this evidence inventory changes no source
input. The record describes the selected map rather than claiming the earlier
maps contained every literal dependency.

`Promise.allSettled` remains unfinished and belongs to a separate checkpoint.
This prerequisite adds no upstream cases, GPU measurements or Chromium-relative
performance result. The [evidence record](evidence/text-metadata-keys/summary.json)
binds the frozen tests, parent failures, source reviews, accounting migrations,
inventory supplement and validation receipts.
