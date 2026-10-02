# Runtime initialization and author work

Runtime construction and author execution each have a fixed 100,000-unit work
allowance. `try_new` first completes the existing charged bootstrap. Only a
successful result receives a fresh author allowance; a failure returns unchanged
without publishing a realm. Allocation accounting stays cumulative across both
phases, under the existing 8 MiB ceiling.

This makes the boundary explicit before adding DOM interface metadata. More
intrinsic installation work can consume the bootstrap allowance without reducing
the work available to a newly returned runtime. The browser's script and host
event entry points already grant their own fixed allowances; those reset points
are unchanged. Calls, getters, conversions, constructors and nested script-driven
event dispatch continue sharing the current turn's remaining work.

At this checkpoint, raw initialization left 70,656 units, with 738,522 charged bytes,
352 objects, 321 native entries and 25 prototype entries. Bootstrap diagnostics
now observe that raw phase directly. Their cost, exact/one-short refusal and
allocation assertions remain intact. The returned runtime starts at 100,000;
its direct helpers spend that allowance without replenishing it.

This is an accounting boundary change, not a faster initialization algorithm or
a conformance gain. It does not make uncharged or infallible allocation paths
safe, and it does not establish production security.

Rust 1.88 passes 1,426 default tests, and Rust 1.98 passes 1,549 native Vulkan
tests, including ignored confinement checks. The two added groups compare
first-script results and resource debits across the raw/new constructor paths,
and prove that getters, conversions and nested event callbacks terminate on the
shared work limit with clean state. Existing raw initialization and heap-failure
tests retain their exact boundaries. See the [validation record](evidence/runtime-budget-boundary.json).
