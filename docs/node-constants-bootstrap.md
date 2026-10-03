# Batched Node constant initialization

Runtime initialization builds the constructor and prototype property maps for
Node in two batches. Previously, it inserted each of the eighteen constants into
each map separately. Both objects remain ordinary, separate mutable property
bags; constants retain their numeric values and nonwritable, enumerable,
nonconfigurable descriptors. The original IDL creation order is retained.

Before consuming an old map, the initializer verifies the exact three existing
keys and their creation order, owner identity, key bounds and reserved order
capacity. It pays for the literal keys, both complete entry vectors, sorted
collection, possible sort scratch and fresh tree nodes. All quota checks and
fallible vector reservations finish before either old map is taken. Existing
descriptors, including accessor function handles, move into the completed maps.
Both maps remain local until completion, then their order vectors and values
are published without further allocation or callbacks.

An internal ordering violation after consumption rejects the private initializer;
it does not return a reusable partial runtime. Tests exercise duplicate and
unsorted staging on the first map. The fixed shared permutation and exact owner
guards make a second-only ordering failure unreachable through the production
caller; holding the first map until the second succeeds is also reviewed in source.

The allocation model remains cumulative. Already charged object, order-vector
and old-tree blocks are retained in the ledger. The reduction removes charges
for repeated insertions that no longer occur; it does not refund spent storage.
The fixed bootstrap/author work allowances, 8 MiB heap ceiling and existing entry
resets are unchanged. A successful runtime consequently has more logical heap
headroom. This change introduces no new Web API or conformance claim.

BTree and sort allocation still use standard-library infallible allocation.
This is not comprehensive physical out-of-memory recovery. The sorted-collection
cost and storage argument is tied to the retained Rust 1.88 and 1.98 source review.

## Resource accounting

The helper comparison invokes both implementations against authentic test-owned
preconstant bags. It measures the unchanged general insertion path rather than
repeating its fee formula in a test.

| Charged quantity | Individual insertion | Batch |
| --- | ---: | ---: |
| Helper work | 12,986 | 8,244 |
| Helper bytes | 103,140 | 33,876 |
| Complete bootstrap remaining work | 6,280 | 11,022 |
| Complete bootstrap bytes | 1,804,200 | 1,734,936 |

Both versions retain 679 objects with capacity 679, 321 native entries and 25
legacy prototype entries. These are logical accounting measurements, separate
from elapsed host time. The first bootstrap test run retained 24 passes and one
stale remaining-work assertion; only that descriptive literal changes after the
actual measurement. No quota or expectation in the independent JS cases changes.

## Measurement scope

The separately frozen benchmark times complete successful public Runtime::try_new
calls followed by destruction. Each fresh process performs 256 untimed warmups
and a timed batch of 1,024 initializations. Twenty-four alternating adjacent
baseline/candidate pairs run for Rust 1.88, followed by twenty-four for Rust 1.98.
No sample is discarded for being slow. Per-toolchain paired ratios and every
raw observation are retained.

Process startup, warmup and output are outside the timer; normal Result checks,
black-box calls, loop overhead, the author-budget reset and destruction are inside.
This workload creates no document, script, page, worker or renderer. Measurements
on one host include scheduling, allocator and clock noise. They do not establish
page-startup, Vulkan or Chromium performance.

The first standalone wrapper used lto=off and failed to link LLVM bitcode from
the release library. The retained correction uses thin LTO for all four wrappers.
No timing ran before that uniform command correction.


The 96 valid observations did **not demonstrate a host-time speedup**. The median
paired candidate/baseline ratio is 1.002219 on Rust 1.88 and 1.001350 on Rust 1.98:
about 0.22% and 0.13% higher times, respectively. Both interquartile ranges span
one. These small mixed differences are inconclusive on this host. The change is
retained for its lower charged work and storage, without a speedup claim.

| Rust | Baseline median (µs/init) | Candidate median (µs/init) | Median paired ratio | Ratio Q1–Q3 |
| --- | ---: | ---: | ---: | ---: |
| 1.88 | 341.251 | 341.963 | 1.002219 | 0.991496–1.013733 |
| 1.98 | 339.012 | 339.442 | 1.001350 | 0.992422–1.006604 |

Each row contains 24 pairs. Ratio minima/maxima are 0.934200/1.040843 and
0.977853/1.026967, respectively. Quartiles use linear inclusive interpolation.
The ratio median is computed from paired samples, not from the displayed arm
medians. All samples, source and binary hashes, build commands, environment
snapshots and pre/post input checks are retained in the evidence archive.

## Validation

Rust 1.88 passes all **1,688 default tests**, and Rust 1.98 passes all **1,811
native-feature tests**, including ignored confinement tests. Strict native Clippy
on 1.98, presenter Clippy on 1.88 and formatting pass. Sixteen new groups cover
literal descriptors and key order, retained metadata and native handles, two
simultaneously live runtimes, exact and one-short admission, and invalid private
staging. The passing focused commands contain 84 observations across 79 unique
groups. Only the retained descriptive bootstrap assertion needed correction.

The release replay preserves every complete record across 4,162 established
cases and 420 controls, plus all 14 new independent modes and 12 controls.
No input, outcome, expectation health or control record changes. Existing
unsupported behavior and failures remain visible. The [summary and archive](evidence/node-constants-bootstrap.json)
retain sources, builds, failed attempts, reviews and raw observations. The
preceding title commit passed all nine [CI jobs](evidence/document-title-ci.json).
