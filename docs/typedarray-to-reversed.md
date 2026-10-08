# Number TypedArray reversed copies

`TypedArray.prototype.toReversed()` creates a reversed copy of all ten Number
view kinds, including `Float16Array`. It validates the source's current bounds,
captures its length, and allocates a fresh same-kind result. The source and its
backing bytes remain unchanged. The result has byte offset zero and distinct,
fixed, nonresizable storage, including when the source is empty.

The method uses the saved intrinsic constructor. Replacing the global
constructor or adding source `constructor`, species, `length` or numeric
prototype properties does not redirect copying. Extra argument values are not
converted; argument expressions still execute before the call and can resize or
detach the source. Detached or out-of-bounds sources are rejected on entry.

Every copied element passes through the existing numeric codecs. This includes
an odd-length midpoint, so a noncanonical NaN payload may become canonical in
the result while the original source bytes remain unchanged. Allocation,
zeroing, numeric reads and complete scalar writes retain their existing resource
checks. A resource stop can leave an internal, unreturned result partly written;
it does not change the source or roll back preceding authored effects.

The implementation follows the
[ECMAScript copying algorithm](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.toreversed)
and its intrinsic same-kind allocation. BigInt views, shared memory and the
remaining TypedArray methods are separate unfinished work.

## Browser example

```sh
./run.sh ./examples/typedarray-to-reversed.html
```

The example starts with a selected `Uint16Array` range `11 22 33 44`. Clicking
**Make reversed copy** produces `44 33 22 11`; clicking again produces
`11 22 33 44`. The original six-word buffer remains `99 11 22 33 44 88`.
The page displays that the result uses separate fixed storage.

The direct Page and Linux confined-worker tests exercise both real hit-tested
clicks. They check literal DOM and drawn text, a stable button identity, solid
button pixels, text ink and changed result pixels. These are bounded browser
interaction checks, not a general browser compatibility claim.

## Complete upstream profile

The `typedarray-to-reversed` profile retains all nine original files at Test262
revision `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`, their harness dependencies
and the Git tree proof. The source tree has no separate BigInt child directory.
Both modes are retained for every body:

| Outcome | Modes |
| --- | ---: |
| Passed | 8 |
| Resource limit | 8 |
| Failed | 0 |
| Unsupported by policy | 2 |

All **52 controls** are healthy: 32 shared controls and 20 observations from
five independently frozen positive/wrong pairs. Each wrong partner requires a
successful positive partner in the same mode and the intended intrinsic Error.
The eight resource outcomes still make the **complete report unhealthy**. No
upstream baseline or gate is recorded.

The resource stops are both modes of `ignores-species.js`, `immutable.js`,
`length-property-ignored.js` and `reverses.js`. Here `immutable.js` checks that
copying leaves the source unchanged; it is not an immutable-ArrayBuffer feature.
The two exclusions are the host-dependent `this-value-invalid.js` body, retained
whole with its mixed dependencies. Neither bodies nor expectations were reduced
to fit the unchanged execution budget.

The published predecessor produced **16 failures and two exclusions**. Its
32 shared controls passed, while all 20 method-control observations were
unhealthy. The [before report](../tests/conformance/test262-typedarray-to-reversed-before.json)
and [after report](../tests/conformance/test262-typedarray-to-reversed-after.json)
retain every observation and its source, policy and binary bindings.

## Validation and limits

The final source passed 15 focused method groups, eight metadata groups, both
browser interaction groups and the full **2,359-test** native suite on Rust
1.88. The Rust 1.98 script suite passed **1,348 tests**. Both strict Clippy runs,
format checking and all **320** conformance-protocol tests passed.

The new intrinsic metadata adds **236 startup work units and 2,125 charged heap
bytes**, including one object. The measured realm has 739 objects, with 1,612
startup work units remaining and 1,701,350 charged bytes. The existing public
execution quota is unchanged. The original descriptor regression still passes
in both modes with only **361 work units remaining**; this narrow margin is a
measured result, not a guarantee of future headroom. No timing benchmark was
performed for this increment.

All **44 prior reports** retain their **18,280 case rows and 4,232 control rows
exactly**, with no prior-pass loss or policy drift. All 38 historical baseline
gates pass. The new 18-mode copying profile is separate from those totals.

The [evidence summary](evidence/typedarray-to-reversed.json) and
[raw evidence archive](evidence/typedarray-to-reversed.tar.gz) bind the candidate,
sources, complete reports and validation attempts. The retained comparison
history includes a root invocation of a nonexistent alternate comparator; the
original held comparator was unchanged and its subsequent invocation passed.

This is a bounded Number TypedArray increment. It does not establish complete
ECMAScript or web conformance, production security, or the project's target of
performance within 30% of Chromium.
