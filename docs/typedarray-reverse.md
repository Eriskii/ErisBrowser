# Reversing Number TypedArrays

`TypedArray.prototype.reverse` reverses all ten Number element kinds in place
and returns the original view. Other views observe the same backing bytes.
Only elements inside the selected view move; bytes before and after it remain
unchanged. Empty and single-element views still validate their backing buffer.

The method uses the current length of fixed or tracking views over resizable
buffers. Detached or out-of-bounds receivers throw. It reads each outer pair
before writing its lower and then upper element, following the
[ECMAScript reverse algorithm](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.reverse).
It does not consult an authored `length`, constructor, species or numeric
prototype property. Ordinary argument expressions run before dispatch; the
method does not coerce their values.

Numeric reads and writes preserve signed zero, infinities and numeric values.
Visited NaNs use the existing scalar codec's canonical representation; an odd
middle element is untouched, including its original NaN payload. Each complete
scalar write is admitted before mutation. If a work limit interrupts a pair,
the lower write may remain without the upper write. The method body adds no
heap allocation. BigInt, shared and immutable buffers remain unsupported.

Open the shared-buffer example and click twice to reverse and restore its
selected view:

```sh
./run.sh ./examples/typedarray-reverse.html
```

## Validation

The [complete upstream report](../tests/conformance/test262-typedarray-reverse-after.json)
retains 22 original Test262 bodies and all 44 modes:

| Result | Modes |
| --- | ---: |
| Passed | 22 |
| Resource limit | 4 |
| Failed | 0 |
| Unsupported | 18 |

All 52 harness controls pass, including five paired positive/wrong-result
probes in both modes. The exclusions retain 12 BigInt, two immutable-buffer,
two complete-resizable-helper and two host-hook modes. Both modes of
`preserves-non-numeric-properties.js` and `reverts.js` still reach the instruction
limit. This report is unhealthy for baseline recording and is not an upstream
CI gate. No quota or original test body was changed.

The [before report](../tests/conformance/test262-typedarray-reverse-before.json)
records four passes, 22 failures and 18 exclusions, with unhealthy method
controls. Those four passes did not establish working reversal support.

The final Rust 1.88 suite with `vulkan-raster` passes **2,342 tests**. Twelve
focused groups cover literal numeric and byte results, metadata, invalid
receivers, resized bounds, ignored conversion hooks, exact resource boundaries
and retained partial stores. Two integration tests render the example through
direct Page handling and the real confined worker. Each checks the initial
frame and two genuine hit-tested clicks, including literal text and changed
result pixels. All **1,333 script tests** pass on Rust 1.98. Strict Clippy passes
on both toolchains, and all **311 Test262 tooling tests** pass.

On the recorded x86_64 build, method metadata adds the predicted **227 work
units and 2,110 charged bytes**, including the reserved object slot. Old method
identities and creation order are preserved. The existing nonnumeric-descriptor
upstream regression still passes both modes, with 2,441 work units remaining.
The method's own body uses `25 + floor(length / 2) * pair_cost` work units;
literal pair costs range from 300 to 384 across the supported codecs.
These accounting units are not elapsed-time measurements.

All 43 previous reports retain their complete **18,236 case rows and 4,180
control rows** exactly. The 38 historical baseline gates pass without changes;
existing TypedArray resource stops remain visible. The
[evidence summary](evidence/typedarray-reverse.json) and
[archive](evidence/typedarray-reverse.tar.gz) bind sources, the candidate engine,
source reviews and complete observations, including the initial Clippy failure
and its test-only type-alias correction.

Full web compatibility, security assurance and the Chromium performance target
remain unfulfilled.
