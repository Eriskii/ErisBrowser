# Filling Number TypedArrays

`TypedArray.prototype.fill` writes a selected range in all ten Number element
kinds and returns the original view. Aliases observe the same backing bytes.
Integer wrapping, clamped-byte rounding and floating-point encoding use the
existing scalar storage operations.

The method validates the initial view, captures its length, and converts the
value and start, then end if it is provided and not `undefined`. Each conversion
happens once; an omitted or `undefined` end uses the captured length. After all
conversions it checks the current buffer again, even if the requested range is
empty. Shrinking a tracking view limits the written range; growing it does not
extend the captured end. A fixed view may become temporarily out of bounds and
recover during later conversions, but a final invalid or detached view throws.
These rules follow the
[ECMAScript fill algorithm](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.fill).

Each scalar's complete byte write is admitted before mutation. If a later work
limit stops a fill, completed elements and earlier callback effects remain.
The implementation adds no per-element heap allocation. BigInt, shared and
immutable buffers remain outside this implementation's supported scope.

Open the example with two views of one buffer:

```sh
./run.sh ./examples/typedarray-fill.html
```

The later [encoding optimization](typedarray-fill-encoding.md) prepares one scalar
per nonempty call while preserving these results and the checked writer. Its
separate record includes complete before/after timing samples and remaining
resource stops.

## Validation

The [upstream report](../tests/conformance/test262-typedarray-fill-after.json)
retains 52 original Test262 bodies and all 104 modes:

| Result | Modes |
| --- | ---: |
| Passed | 22 |
| Resource limit | 32 |
| Failed | 0 |
| Unsupported | 50 |

All 52 harness controls pass, including five paired positive/wrong-result probes.
The exclusions retain 36 BigInt, two immutable-buffer, four complete-helper and
eight host-hook modes. Resource outcomes make this complete report unhealthy
for baseline recording; it is not an upstream CI gate. The
[before report](../tests/conformance/test262-typedarray-fill-before.json) records
four passes, 44 failures, six resource stops and 50 exclusions with unhealthy
controls. Those four earlier passes did not establish working fill support.

Both original search failures that called the missing `fill` now pass. The
[unchanged search selection](../tests/conformance/test262-typedarray-search-after-fill.json)
records **80 passes, 78 resource stops and 132 exclusions**, with all 56 controls
verified. Its resource stops still prevent an upstream baseline.

The full Rust 1.88 suite with `vulkan-raster` passes **2,324 tests**, including
both direct and confined-worker rendering of the example before and after a
real hit-tested click. Twelve focused fill groups cover literal codec/range
results, callback order, fresh bounds, saved identity, scalar write prefixes,
exact and one-short resource cuts, and terminal cleanup. All **1,317 script
tests** pass on Rust 1.98; strict Clippy checks pass on both toolchains. All
**302 Test262 tooling tests** pass. The Rust groups and browser integration
checks run in the existing CI suite.

On the recorded x86_64 runs, metadata installation adds the predicted 218 work units and 2,095 charged
bytes, including its reserved object slot. Old method identities and creation
order are preserved. The earlier nonnumeric-descriptor upstream regression
still passes both modes with 4,561 work units remaining. No runtime quota,
original test body, historical profile policy or existing baseline was changed.
Across the 38 historical profiles and earlier TypedArray suites, 18,130 of
18,132 prior case rows remain exact; the two changes are the recovered search
cases. All 4,128 prior control rows remain exact, with no lost passes.

The [evidence summary](evidence/typedarray-fill.json) binds source, candidate
engine and validation reports. Its [archive](evidence/typedarray-fill.tar.gz)
retains raw inputs, source reviews and complete before/after observations.

Full web compatibility, security assurance and the Chromium performance target
remain unfulfilled.
