# Encode a TypedArray fill value once

Number TypedArray `fill` now prepares one encoded scalar and reuses it for the
selected elements. The previous loop repeated scalar encoding and TypedArray
validation for every write. This change is confined to `typed_array/fill.rs`;
the shared scalar codec, ordinary indexed writer, ArrayBuffer writer, metadata
and runtime limits are unchanged.

Initial validation and value/start/end conversion keep their order. After the
last conversion, the method obtains a fresh copied buffer handle, byte offset
and view length. All author callbacks have completed before it uses this
snapshot. The remaining loop cannot resize or detach the admitted nonshared
buffer. It still checks every byte offset and calls the existing checked
ArrayBuffer writer, which admits the complete scalar copy before changing any
bytes. A resource stop can leave completed elements; it cannot leave a partial
scalar. Earlier callback effects remain observable.

The same codec preserves integer wrapping, clamped ties, signed zero, canonical
NaNs and direct Float16 rounding. Empty ranges still perform conversions and
final validation, then return without preparing a scalar.

## Work accounting

For a nonempty fill with `n` elements of byte width `w`, let `C` be the unchanged
codec charge: `48 + 2*w`, plus 16 for Uint8Clamped. Let `B` be the existing
validation/conversion prefix: 58 for omitted or undefined end, or 62 for a
defined end, plus argument-conversion work.

| Body | Previous charge | Current charge |
| --- | ---: | ---: |
| Empty range | B | B |
| Nonempty range | B + n × (26 + w + C) | B + 8 + C + n × (13 + w) |

The new eight-unit preparation charge pays for fixed witness, width, codec and
encoded-slice setup. Each scalar retains loop, checked-offset and full-copy
charges. The body allocates no runtime heap storage. Numeric resource cut
positions change because repeated work has been removed; the tests retain
independent byte-prefix expectations and exact/one-short admission checks.

## Validation and measurements

All **2,328 tests** pass in the Rust 1.88 Vulkan-enabled suite, including the
existing direct-page and confined-worker fill/click checks. All **1,321 script
tests** pass on Rust 1.98, and strict Clippy checks pass on both toolchains.
Sixteen focused fill groups retain the original semantic inputs and byte
oracles, with two explicit accounting migrations and four added groups for
repeated special encodings, offset sentinels, empty-range cuts and record growth
inside conversion callbacks.

Across 43 preserved reports, all **18,236 case rows and 4,180 control rows** are
identical to the published fill checkpoint. No quota, policy, exclusion,
original upstream body or baseline changed. The
[complete fill selection](../tests/conformance/test262-typedarray-fill-after-encoding.json) remains
**22 passed, 32 resource-limited and 50 unsupported modes**, with 52 verified
controls. This optimization does not resolve those resource stops; the report
remains unhealthy for baseline recording.

On an AMD Ryzen 7 7800X3D Linux host, the complete checked-execute benchmark ran
four alternating before/after rounds: 6,720 four-call samples and **26,880 timed
executions**, with no rejected or incomplete sample. Each variant has 84 samples
per type/length. Both binaries use Rust 1.88 release builds, identical harness
source and matching dependency versions. The p95 below is the nearest-rank
percentile of the four-call sample means, not of individual call durations.

For **512-element fills**, pooled median and p95 durations in microseconds were:

| Element type | Before median | After median | Before p95 | After p95 |
| --- | ---: | ---: | ---: | ---: |
| Int8Array | 16.402 | 13.588 | 19.863 | 15.236 |
| Uint8Array | 16.172 | 13.421 | 18.981 | 14.675 |
| Uint8ClampedArray | 17.223 | 13.379 | 19.036 | 14.615 |
| Int16Array | 18.116 | 13.607 | 19.587 | 17.573 |
| Uint16Array | 18.212 | 13.638 | 19.825 | 15.239 |
| Int32Array | 18.405 | 13.318 | 20.103 | 14.465 |
| Uint32Array | 18.507 | 13.325 | 19.757 | 14.555 |
| Float16Array | 17.166 | 13.270 | 19.076 | 15.023 |
| Float32Array | 17.115 | 13.201 | 18.896 | 13.992 |
| Float64Array | 15.001 | 13.246 | 18.352 | 14.194 |

These 512-element medians were 11.7–28.0% lower. The smaller cases do not support
an across-the-board speedup: empty-range medians were 1.2–3.2% higher,
single-element medians were 0.3–2.0% higher, and 16-element medians ranged from
0.8% lower to 0.6% higher. All raw samples, including tails, are retained. The
host used its performance governor but was not isolated or frequency-locked;
these are descriptive measurements without a statistical-significance claim.

The [evidence summary](evidence/typedarray-fill-encoding.json) binds the source,
selected engine, complete comparisons and timing results. Its
[archive](evidence/typedarray-fill-encoding.tar.gz) retains the unmodified raw
reports, exact accounting decisions, reviews, harness, build records and every
sample. The standalone benchmark manifests record the two absolute worktree
paths; reproducing elsewhere requires changing those paths and the matching
runner roots, while keeping source and release settings identical.


The benchmark uses the public Rust Runtime API with identical harness source
and release settings for both engine versions. It measures complete JavaScript
executions, including parsing, method dispatch, fill and identity/endpoint
checks. Setup, warmup and full-element verification are outside the timers.
Every timed call changes the array value, and all elements are verified before
accepting the sample. Resource stops or incorrect results fail the run.

Local host timings do not establish Chromium parity or overall browser speed.
The full web-compatibility, security and performance requirements remain open.
