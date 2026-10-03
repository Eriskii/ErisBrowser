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

The later [Node constant batching checkpoint](node-constants-bootstrap.md)
replaces repeated property insertion with two admitted sorted builds. Its raw
bootstrap leaves 11,022 work units and charges 1,734,936 bytes, with 679 objects,
321 native entries and 25 legacy prototype entries. The fixed work/heap limits
and success-only author grant remain unchanged. Existing blocks stay charged;
eliminating unperformed insertions leaves more logical heap for author work.

The later [Text operations checkpoint](text-operations.md) installs two ordinary
cached function bags for `splitText` and `wholeText`. Raw bootstrap leaves
10,636 work units and charges 1,741,271 bytes, with 681 objects and capacity,
321 native entries and 25 legacy prototype entries.
No work or heap limit and no author-entry reset site changes.

The later [Node normalization checkpoint](node-normalize.md) installs one ordinary
cached method bag and admits the distinct prototype/constructor constant maps.
Measured bootstrap leaves 10,199 work units and charges 1,744,501 bytes, with
682 objects/capacity, 321 native entries and 25 legacy prototype entries. This
adds 437 charged work units and 3,230 bytes. Numeric limits and reset sites stay
unchanged. The method's descendant walk, exact payload copy and removals spend
the existing author allowance; retained effects are not rolled back on refusal.

The later [Node predicate checkpoint](node-predicates.md) installs three ordinary
cached method bags for `hasChildNodes`, `isSameNode` and `contains`. Measured raw
bootstrap leaves 8,807 work units and charges 1,755,715 bytes, with 685 objects
and capacity, 321 native entries and 25 legacy prototype entries. This adds
1,392 charged work units and 11,214 bytes. The larger prototype constant map
needs four allocated tree nodes; the constructor map still needs three.
Numeric limits and author-entry reset sites remain unchanged. Predicate bodies
spend the existing allowance without allocating traversal or payload storage;
generic invocation and inherited diagnostic allocation retain their own limits.
