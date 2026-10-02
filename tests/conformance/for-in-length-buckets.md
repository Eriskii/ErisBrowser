# For-in visited names grouped by UTF-16 length

For-in now indexes visited property names by their UTF-16 length before searching
full names. Unequal-length names need only an integer comparison; equal-length
names retain the conservative text-search bound. This changes the actual search
structure without increasing the 100,000-work-unit limit or other quotas.

The live own-property descriptor is checked only for an unvisited name. A missing
descriptor inserts nothing; a present nonenumerable property still shadows its
inherited counterpart. Snapshot order determines enumeration order independently
of bucket order. Prototype traversal remains lazy. No author callback runs while
the tree entries are borrowed, and property getters are not evaluated by
enumeration. These rules follow the
[For-In Iterator algorithm](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-%foriniteratorprototype%.next).

The extra integer tree costs storage. Each distinct length prepays an outer node
allowance including its inner-map header; each inserted name pays its inner-node
allowance. Both insertions are prepaid before publishing a new bucket, and a
refusal leaves membership unchanged. Many distinct lengths can use more heap
allowance than the prior single tree. Names still share their UTF-16 payloads.
The supported Rust 1.88/1.98 B-tree layouts underpin conservative logical-work
and cumulative-storage bounds; these are not instruction counters or allocator
telemetry. General property accounting and allocator fallibility remain separate
work, as described in [security](../../docs/SECURITY.md).

## Results under unchanged inputs

The original ArrayBuffer local case
`getter-method-flags-and-nonconstructability` uses the unchanged property helper
to check 24 descriptors. Both modes previously exhausted the instruction budget.
The complete original source now passes in both modes, leaving 12,200 work units
in the focused runtime test. Diagnostic reductions were used to locate the
search cost; none replaced the original test, helper or expectation.

| Population | Observed result |
| --- | --- |
| ArrayBuffer local, 104 modes | 90 passed, eight failed, two unsupported, four expected resource outcomes |
| ArrayBuffer local expectations / controls | 94/104 verified; 76/76 controls verified |
| All eight local suites | 1,482 cases and 392 controls; exactly two changed records |
| Other local records | 1,872 complete records unchanged, including all 392 controls |
| Explicit local resource expectations | All 36 unchanged, including four ArrayBuffer cases |
| All 42 upstream profiles | 18,563 modes and 4,252 controls unchanged |
| Existing regression gates | All 34 passed; no baseline writes |

The remaining ArrayBuffer gaps still require typed arrays, DataView, shared
buffers, Proxy or foreign realms. The formal ArrayBuffer profile retains its
262 passes, 50 failures and 130 exclusions. The original reports remain intact;
the [follow-up evidence](for-in-length-buckets.json) binds the new release and
complete comparisons. No source, helper, expectation, exclusion or quota was
relaxed. These observations establish neither complete web compatibility nor
production security, and there is no elapsed-time or Chromium comparison here.

## Validation and retained failure

All 26 private own-key groups pass, retaining 19 groups and adding seven.
Coverage includes the 86-mode independent fixture, four Number constant controls,
UTF-16 equality with empty/NUL/surrogate names, shared payload identity, integer
and text tree bounds, split thresholds, same-length hostile prefixes, exact and
one-short work/heap admission, hidden-property shadowing, deletion and ordering.

Rust 1.88 and 1.98 each pass **1,287 default and 1,404 native-feature tests**.
Strict all-target Clippy passes for default, `vulkan-presenter` and `vulkan-raster`
on both toolchains; formatting passes. The Rust 1.98 default release supplies
the adapter used for all local and formal replay.

The first Rust 1.88 native run failed the existing
`date_host::tests::host_actual_helper_success_nonzero_malformed_and_flood_are_distinguished`
test with `Unavailable`. The isolated unchanged test and one unchanged full
rerun passed. The cause remains undetermined, and the failed log is retained
alongside the successes. No timeout or assertion was relaxed. The earlier
[Date helper failure history](array-buffer-date-host-followup.json) also remains
available. This increment does not claim a Date helper fix.

Data-only preparation errors are retained: the local replay wrapper initially
read the wrong historical expectation field, and the root summary script assumed
one log-hash field name across two receipt schemas. The first publication attempt
also classified a repository before-report path as a temporary path and stopped
before creating its destination. All were corrected without changing any engine
input or running an additional conformance attempt.

The [evidence inventory](../../docs/evidence/for-in-length-buckets/index.json)
lists the SHA-256 and original path of every payload in the compressed records
archive. It includes complete before/after observations, raw execution logs,
source overlays, compiler receipts, diagnostic requests and independent reviews.
The archive contains no executable builds. Candidate source is reconstructed
from parent `45dd387375a95d2bfd89412f35cdb55739e07503` plus its three Rust overlays;
the complete 407-file source inventory is also bound.

The committed private regressions can be run directly:

```sh
cargo test --locked --lib script::own_keys::tests
cargo test --locked --features vulkan-raster -- --include-ignored
```

For an individual upstream regression gate:

```sh
cargo build --locked --release --bin eris-js
python3 tools/test262_conformance.py --binary target/release/eris-js \
  --profile array-buffer --baseline tests/conformance/test262-array-buffer-current.json
```

The archived replay drivers retain the exact original machine paths and input
bindings for auditing. They require restoring that layout or preparing a fresh
explicitly bound layout before reuse; they are not portable launch scripts.
