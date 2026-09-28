# Bounded Array.prototype.sort

The custom interpreter implements stable `Array.prototype.sort` for arrays and
the existing ordinary property model: ordinary array-like objects, functions,
mapped/unmapped arguments and supported boxed primitives. The intrinsic has
length 1, name `sort`, ordinary writable/nonenumerable/configurable prototype
placement, stable identity and no constructor behavior. Saved aliases, `call`,
`apply` and `bind` use the same implementation.

Comparator validation precedes receiver boxing and the single observable
`length` read/conversion. Collection visits the saved index range in ascending
order through actual `HasProperty` and `Get` operations. It distinguishes holes
from present `undefined` values, includes inherited and nonenumerable indices,
and observes earlier getter changes to later properties. No intrinsic receiver
write occurs until collection and comparison finish successfully.

An iterative merge of collected-value indices preserves equal-value order.
Defined values precede `undefined`; undefined values never enter the comparator.
Default comparison converts the left value before the right on each comparison
and compares UTF-16 code units, including lone surrogates. Author conversions
remain live across comparisons. An explicit comparator receives two arguments
and an undefined receiver; ordinary strict/sloppy, arrow and bound-function
receiver rules apply. Its result undergoes numeric conversion, with NaN and
either signed zero treated as equal. Inconsistent comparators have no promised
ordering, but the merge always makes bounded forward progress.

Successful comparison is followed by ascending strict indexed writes and then
ascending strict tail deletions. The method returns the original boxed receiver
and does not explicitly assign its length. Author changes to length, properties
or prototypes remain observable. Getter/comparator/conversion effects survive
an exception before copy-back, and successful earlier writes/deletions survive
a later failure. Sloppy callers still receive TypeError for failed writes or
deletions. A nonempty boxed string therefore fails its readonly index write,
including the single-character case.

The unchanged 65,536 array-like length cap is enforced after observable length
conversion. Checked, precharged `try_reserve_exact` allocations retain one value
buffer and two index buffers. Decimal index keys use five-unit stack scratch and
precharged UTF-16 storage. Collection, merging, comparisons, property walks and
callbacks share the existing 100,000-step, 8 MiB cumulative allocation and call/
weighted-stack limits. Dropped scratch does not refund the ledger. Exhaustion is
uncatchable, and inputs below the length cap can still exceed the work or heap
budget. Resource failure during copy-back may preserve earlier writes.

Host array-like receivers remain explicitly unsupported. This addition does not
implement Proxy traps, Symbol/BigInt conversion, typed arrays, cross-realm
semantics, `toSorted`, locale sorting or complete Array exotic descriptors.
Defining indexed/length descriptors on actual arrays remains unsupported;
ordinary array-like objects exercise accessor/readonly/nonconfigurable paths.
No adjacent `reduce` implementation is added. The unchanged upstream stability
files depend on that missing method, and their larger callback workloads can
also encounter runtime limits. Complete-file outcomes must retain these
dependencies and resource results; passing a focused sort assertion is not a
pass for the enclosing upstream source.

Thirteen `array_sort_` regression groups cover metadata and ordering, generic
receivers, holes/inheritance, UTF-16 and stability, author coercions, mutation and
reentrancy, strict partial write-back, exact thrown values and resource failures.
Private tests verify collection/merge/callback allocation failures, malformed
prototype cycles and preservation of prior getter effects without copy-back.
The existing pinned `statements/function/S13.2.1_A5_T1.js` runs unchanged in both
modes with the unchanged upstream assertion harness. No corpus is filtered or
rewritten by these runtime tests.

```sh
cargo test --locked --offline --lib script::tests::array_sort
```

Primary algorithms: [Array.prototype.sort](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.sort),
[SortIndexedProperties](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-sortindexedproperties),
[CompareArrayElements](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-comparearrayelements)
and [LengthOfArrayLike](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-lengthofarraylike).
