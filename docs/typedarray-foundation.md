# Number TypedArray foundation

Eris implements the ten Number element types: Int8Array, Uint8Array,
Uint8ClampedArray, Int16Array, Uint16Array, Int32Array, Uint32Array,
Float16Array, Float32Array and Float64Array. They share the existing non-shared
ArrayBuffer storage and Number codecs with DataView. TypedArray storage uses
little-endian byte order; DataView retains its explicit byte-order argument.

Constructors accept lengths, buffers with offsets and optional lengths, other
typed arrays, iterables and array-like objects. Same-kind copies preserve raw
floating-point NaN payloads and signed zero. Different-kind copies convert each
Number. Iterable construction collects values before element conversion, while
array-like construction interleaves indexed reads and conversions. Uint8Clamped
conversion saturates to 0–255 and rounds exact half-integers to even.

The shared prototype supplies buffer, byteLength, byteOffset and length getters,
the type tag, and values/keys/entries iterators. Each concrete constructor and
prototype has BYTES_PER_ELEMENT. Saved constructor/prototype identities are
private realm state; replacing a global does not replace the intrinsic used for
new backing storage.

Indexed reads, writes, property definitions, deletion, presence tests and own-key
enumeration consult authentic private view records. Canonical numeric strings
include invalid indices such as `"-0"`, fractions and infinities; those names
must not fall through to prototype properties. Ordinary names and symbols remain
expandos. Valid indexed descriptors are writable, enumerable and configurable,
but deleting a live element or redefining those flags to false fails.

Views over resizable buffers may have a fixed element count or track the current
buffer length. Detachment and out-of-bounds views are checked against current
buffer metadata. A write converts its value before its final bounds check, so a
conversion callback can grow the buffer and make an index valid. Prototype Set
operations retain the original receiver and their different conversion rules.
Iterators validate the live view on each advance and keep completion sticky.

Every new storage allocation and scalar/index operation has a logical resource
charge. Failure retains earlier author callbacks and completed writes. Native
method receiver binding admits its temporary owned name before allocation.
Work and heap limits remain unchanged. The raw startup diagnostic reconciles
721 object slots, 210 globals and 1,850,331 charged bytes with the independent
installation ledger; the public realm receives its normal author-work budget
only after successful initialization.

The frozen local corpus has 27 bodies in both strictness modes, with 32 paired
positive/wrong controls. It covers aliases, scalar bytes, constructor ordering,
resize/detach callbacks, property behavior, iterators and generic Array/JSON
operations. The separate upstream population retains 632 original Test262
bodies and ten original helpers; its selection and policy are described in
[the preparation record](typedarray-foundation-test262.md). Unsupported bodies,
runtime failures and resource stops remain in the reports.

The [local report](../tests/conformance/typedarray-foundation-local-current.json)
records **54/54 cases and 32/32 controls**. The
[upstream report](../tests/conformance/test262-typedarray-foundation-current.json)
records **572 passes, 86 failures, 488 unsupported modes and 92 work-limit stops**,
with all 64 harness controls verified. It remains an observation report with a
nonzero runner exit, not a healthy conformance baseline. Of the unsupported
modes, 472 are frozen policy exclusions and 16 reach destructuring syntax.
Source triage associates the failures with absent Reflect methods, subarray,
toString, BigInt or class-dependent helpers; the precise stop point for those
class-dependent helper failures is inferred from source. Large unchanged
constructor matrices account for several provable work-limit outcomes.

All 2,203 native-feature tests pass. The 18 private TypedArray groups pass on
Rust 1.88 and 1.98, with strict Clippy on both. Across 37 unchanged existing
CI-gated profiles, all 16,136 mode fingerprints are preserved: ArrayBuffer gains
48 passes and object-integrity gains 18. Every other observation is identical
to the frozen published adapter. The
[evidence record](evidence/typedarray-foundation.json) binds these executions,
source reviews, retained failures and final adapter.

Adding the ten global names exposed a work-limit regression in an unchanged
Window enumeration test. The preceding `4990c3c` commit replaces full sorting
with bounded creation-order buckets for dense histories; sparse histories keep
the original sort. It admits the additional bucket storage before allocation,
and all original Window tests remain unchanged.

The [browser fixture](../tests/conformance/typedarray-foundation-browser/typedarray-foundation.html)
keeps views across a real click. Direct Page and confined-worker tests compare
the entire canvas with independent literal references, verify displayed values,
and retain seven exact node identities and the parent/child graph.

```sh
cargo test --lib script::typed_array
cargo test --test typedarray_foundation -- --include-ignored
python3 tools/typedarray_foundation_local.py --help
python3 tools/test262_conformance.py --profile typedarray-foundation \
  --binary /absolute/path/to/eris-js --output /new/path/to/report.json
```

This is a foundation, not the complete TypedArray API. Static from/of,
bulk/search/sort/copy/string methods, BigInt views, shared memory and foreign
realms remain unfinished. The complete browser compatibility, production
security and Chromium performance requirements also remain open.
