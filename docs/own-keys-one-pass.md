# Collect stored own keys in one pass

The existing ordinary string-key snapshot now retains accepted string handles
while scanning creation order, removing a second pass over the same entries.
The path still requires at least eight stored entries and no virtual indices or
length. It rejects any digit-leading string and uses the unchanged generic
builder for numeric classification and ordering. It adds no new JavaScript API
or supported receiver category.

String order and shared UTF-16 payloads are preserved, including empty names,
non-ASCII text and lone surrogates. Symbols stay out of string-key output.
Nonenumerable names remain in the snapshot so they still suppress inherited
duplicates. The snapshot helper invokes no author callbacks. For-in continues
to obtain live descriptors and prototypes, and authentic TypedArray enumeration
still checks current buffer bounds through its existing route.

## Work and storage tradeoff

For a successful nonempty stored-string snapshot, let `N` be the number of
creation-order entries, `S` the retained strings, and `j` the ordinal of the
first string. Including the existing four-unit caller route, the helper's
logical work charge changes from `16 + 10*N + 2*S` to `16 + 6*N + 4*S`.
The four units per string pay for reacquiring its handle after admission and
then cloning/pushing it. All-symbol snapshots use `8 + 6*N` work and reserve
no output heap. These charges describe this snapshot path, not a whole script
or a timing estimate.

The output reserves the remaining suffix once, at its first accepted string.
Previously it charged `S` string-handle slots; it now charges `N-j` slots.
Later symbols therefore add `(N-j-S) * size_of::<JsString>()` charged bytes.
Leading symbols need no output slots. A digit-leading first string allocates
nothing, while a later rejected string can leave paid temporary storage and
cloning work before the unchanged generic fallback. Those cumulative charges
are retained. This is not a universal work or heap reduction.

Allocation and handle-copy work are admitted before the corresponding action.
On refusal, the private temporary vector is dropped without publishing changes
to the receiver's keys or descriptors. All existing runtime limits remain
unchanged, as do visited-name accounting and live property/prototype checks.

The original nonnumeric descriptor regression passes in both modes with
**1,241 work units remaining**, up from 361. Its repeated shared-prototype
snapshots remove 880 work units and add 640 charged bytes on this build's
16-byte string handles. The actual allocated totals are 4,503,410 bytes in
sloppy mode and 4,501,440 in strict mode. This is one measured resource-margin
result, not a guarantee that another feature fits.

## Compatibility and validation

All **45 earlier reports** retain their **18,298 case rows and 4,284 control
rows exactly**. There are no new passes, changed observations, prior-pass losses
or policy changes. All 38 historical baseline gates pass. The unchanged
upstream failures, unsupported modes and resource stops remain visible; healthy
controls do not make resource-containing reports healthy. No new upstream
baseline or gate is introduced.

The Rust 1.88 suites pass **2,214 default tests** and **2,371 native-feature
tests**. The Rust 1.98 script suite passes **1,359 tests**. Both strict Clippy
runs and format checking pass. Eleven focused optimization groups check literal
key order, UTF-16 payload identity, exact work/heap cuts, symbol capacity,
numeric fallback, callback mutation and fresh TypedArray bounds. All 62
own-key groups pass with the separately recorded old accounting migrations.

The data-only comparison retains complete rows and authentic control checks.
Its first protocol run exposed a test-oracle error: four historical module
rows were incorrectly attributed to the for-of profile alone. The corrected
test names the two for-of and two labeled-statement rows explicitly. All eleven
protocol groups pass after that test-only correction; the comparator and its
successful 45-report comparison remain unchanged. The failed attempt is retained.

This checkpoint also repairs an existing Date host test fixture after the
predecessor's CI failure. It replaces an unframed PID marker and busy wait with
a complete-line marker and a nonblocking FIFO handshake, and adds a marker
framing test. The original log does not establish the failure's precise cause;
the identified fixture races are source findings. Production Date host behavior
and its **1,000 ms deadline** are unchanged. Both current full local suites pass;
CI for this checkpoint remains pending publication.

## Local complete-script benchmark

On an AMD Ryzen 7 7800X3D Linux host, three alternating before/after process pairs
completed all eight fixtures. Each process used 21 fresh-runtime samples per
fixture, two untimed warmup executions and five timed executions per sample.
That is **1,008 accepted samples and 5,040 timed executions**, with 63 samples
per fixture and variant. All six processes completed successfully; no sample
or tail was dropped.

Both variants use Rust 1.88 release builds, identical benchmark source and
resolved lockfiles, and the same wrapper. Every execution's result is checked
against the expected key order outside the timer. Timed work includes JavaScript
parsing, interpreter execution, complete for-in snapshots, live descriptors,
visited-name accounting and string concatenation. Runtime/object setup, warmup,
result checking, process startup, I/O and rendering are outside the timers.

Pooled values below are **microseconds per execute**. Each sample is the mean
of five individually timed calls. p95 is nearest rank `ceil(0.95*N)` over the
63 sample means, not over individual call durations.

| Fixture | Before median | After median | Before p95 | After p95 |
| --- | ---: | ---: | ---: | ---: |
| strings8 | 18.449 | 18.419 | 28.192 | 25.691 |
| strings20 | 36.762 | 35.389 | 54.389 | 49.340 |
| strings64 | 103.839 | 101.316 | 127.364 | 140.152 |
| mixed20 | 32.620 | 32.630 | 43.589 | 41.771 |
| symbols20 | 5.981 | 5.991 | 6.717 | 10.446 |
| digit-first | 35.874 | 35.918 | 39.535 | 43.375 |
| digit-last | 36.483 | 36.243 | 47.458 | 48.180 |
| prototype-chain | 42.567 | 41.853 | 61.875 | 49.181 |

The median changes range from 3.7% lower to 0.2% higher. Four p95 values regress:
`strings64`, `symbols20`, `digit-first` and `digit-last`; the symbol-only p95
rises from 6.717 to 10.446 microseconds. The mixed and digit fixtures retain
both the extra-capacity and fallback costs in the complete-script measurement.
All process ordering and raw timings are retained. The host used the performance
governor with all 16 logical CPUs available; this was not an isolated or
frequency-locked experiment. These descriptive results do not establish a
statistically significant improvement, an isolated native-helper speedup,
overall browser performance or Chromium parity.

The [evidence summary](evidence/own-keys-one-pass.json) and
[archive](evidence/own-keys-one-pass.tar.gz) bind the source, adapters, unchanged
reports, validation attempts and complete benchmark records. The benchmark's
standalone manifests use absolute worktree paths; reproducing elsewhere
requires updating those paths and the corresponding runner bindings while
retaining identical source and build settings.

Full web compatibility, production security and the project's target of
performance within 30% of Chromium remain unmet.
