# Native rendering measurements

`--benchmark-native N` measures 2–128 completed frames through the existing
Vulkan window owner. It compares Eris's CPU framebuffer upload and native
shader route in the same `vulkan-raster` build. This is an opt-in experiment;
source, unit and CLI checks pass. The original CPU/native comparison and a
later native buffer-reuse comparison are retained separately below. They establish
no Chromium comparison or general performance claim.

## Usage and equal inputs

Build one release executable, then use the same loaded document and URL for
both routes:

```sh
cargo build --locked --release --features vulkan-raster
./target/release/eris-browser ./tools/native-raster-fixtures/admitted.html --presenter=vulkan --raster=cpu --benchmark-native 32
./target/release/eris-browser ./tools/native-raster-fixtures/admitted.html --presenter=vulkan --raster=gpu --benchmark-native 32
```

Linux, the desktop Vulkan presenter and the `vulkan-raster` build are required
for either route. Timing cannot be combined with verification, screenshots,
headless rendering or the other benchmark modes. An `--exit-after` guard may
terminate a stalled experiment; early termination cannot count as a completed
sample set. Ordinary browsing retains its existing redraw behavior.

Sampling starts after a current loaded worker snapshot and an idle presenter
are available. Both routes replace dynamic load-time chrome with the identical
status text `Native timing scene`; the original load time is reported separately.
The canonical scene identity contains full bounded drawing/image inputs and
paint-affecting browser state. Exact identity is checked outside the measured
spans. A navigation, input, resize, visibility or scene change aborts the run.
For HTTP pairing, retain one listener and URL across processes: a different
port changes the address-bar glyphs. Match physical viewport, scale, adapter,
surface format and FIFO mode as well as document bytes.

Run separate correctness processes with `--benchmark-native-check` in place
of `--benchmark-native N`, once per raster route. They use the same fixed
status and identity gate, then compare one acquired texture to a complete
Canvas reference. Startup frames cannot consume this check: the idle owner
must be explicitly armed for the selected packet. A frame that retires without
verification fails the check. Correctness mode contains no timing samples;
measured frames contain no reference Canvas, readback or pixel comparison.
Acquired-texture agreement is before compositor output.

## Recorded intervals

Samples contain integer nanoseconds from the host monotonic clock. Unreached
phases are JSON `null`, not invented zero durations.

| Field | Boundary |
| --- | --- |
| `ui_prepare_ns` | Browser route preparation start to packet acceptance; includes Canvas painting or native preparation, packet validation and mutex admission. |
| `queue_ns` | Packet acceptance to owner trace start after taking the packet. |
| `acquire_ns` | Existing acquisition/configuration/retry phase; `configured` records whether configuration occurred. |
| `encode_upload_ns` | CPU channel conversion and texture upload, or native buffer checkout/allocation, metadata writes and raster/conversion/copy encoding. |
| `submit_ns` | Encoder finish and queue submission where reached. |
| `completion_wait_ns` | Existing wait for the tracked submission. |
| `cleanup_ns` | Native error-scope collection and reached buffer destruction/readback drop; absent for ordinary measured CPU upload. Successful reusable buffers remain owned beyond this bracket. |
| `present_call_ns` | Window pre-present notification and queue present call. |
| `owner_total_ns` | Owner trace start through packet drop and the idle transition. |
| `prepare_to_present_ns` | Direct preparation-start interval through successful present and its existing deadline/failure checks. |

Owner initialization is recorded separately, covering adapter/device setup and
applicable native pipeline creation. It excludes page loading and whole-process
startup. These are host durations with instrumentation overhead: allocation,
driver scheduling and FIFO acquisition waits can contribute. They are neither
GPU timestamps nor display latency. Compute total quantiles from direct total
samples rather than summing phase quantiles.

Each sample keeps its route, frame identity, outcome and submission kind.
Draw submission, flush-only cancellation, non-presentation and failure remain
distinct. The browser permits only one measured request at a time and advances
only after a presented packet has been dropped and the owner is idle. A failed,
skipped or overlapping request terminates sampling; native admission fallback
is not substituted into a native timing distribution. The report is emitted
only after positive destruction acknowledgment from the owner. A stalled or
finished thread alone cannot authorize output or software recovery.

## Scope, budgets and interpretation

Every native sample prepares a fresh plan, CPU glyph/rounded coverage and
packed inputs. Repetition does not measure a retained-plan or native mask cache.
The current owner can retain one exact-size retired GPU buffer set; the later
A/B comparison below compares that allocation change. It still rewrites every
input/uniform and clears and draws the complete target each frame.
CPU painting may reuse its existing glyph cache. Report initialization, first
eligible loaded sample and subsequent samples separately; even the first sample
may follow startup CPU chrome that warmed fonts. Fresh processes do not imply
cold operating-system or driver caches. Repeated, interleaved route pairs and
retained failures are needed to assess variation.

There are at most 128 retained records. Their actual vector capacity and a
256 KiB identity/metadata allowance are charged against the unchanged 128 MiB
presenter ledger. Each canonical identity is at most 64 KiB; JSON output is
bounded to 256 KiB and streamed after release. These are explicit application
storage bounds, not process RSS limits. Existing borrowed assets, font caches,
trusted outline allocation, driver/surface and allocator overhead retain the
[documented exclusions](vulkan-native-window.md#resource-and-ownership-contract).

Native admission retains its 1280×1024 viewport, 16 MiB planned GPU buffers,
4,000,000 padded invocations and scene/source/coverage limits. The original
active deadline and queued-write abort retirement are unchanged. Timing adds
no optional clock calls, sample allocation or completion redraws when disabled.
Rounded and font coverage still run on the CPU; shaders composite those masks,
sample images and produce the surface bytes. See the
[native renderer contract](vulkan-native-window.md) for supported rendering and
whole-frame fallback outside measurement mode.

## Validation status

The earlier [API evidence](evidence/vulkan-native-timing-api.json) retains 1,387 passing
native-feature tests on Rust 1.88 and 1.98, strict Clippy across supported
feature variants, 95 browser/presenter groups and 15 compiled CLI rejection
cases. The 32 new private groups cover identity/serialization, completion,
verification gating, optional clocks and bounds. Initial lint and outer-sandbox
preflight failures remain retained with their successful follow-ups.

## First controlled release comparison

The [retained host evidence](evidence/vulkan-native-timing-host.json) contains
one complete eight-process sequence using the same release executable, loopback
URL and exact 269-byte canonical paint identity. Both 1280×880 correctness
processes match all **4,505,600 active bytes**, separately for CPU upload and
native raster. Six measurement processes each complete 16 frames without
verification or readback. Every process exits successfully and its owned children
are reaped. All 96 focused Python groups pass, including 25 new host groups.
The complete tool suites also pass: 347 root-tool and 90 raster-probe groups.
The preceding [API checkpoint CI](evidence/vulkan-native-timing-api-ci.json)
passes all nine jobs.

The selected surface is NVIDIA GeForce RTX 4070 SUPER, driver 595.99.02,
BGRA8, opaque sRGB, FIFO; the window reports scale 1. CPU/GPU order alternates
across three pairs. The direct preparation-through-present durations are:

| Population | CPU median | Native median | CPU p95 | Native p95 |
| --- | ---: | ---: | ---: | ---: |
| First loaded frame, three per route | 3.039 ms | 5.744 ms | 3.140 ms | 6.468 ms |
| Subsequent frames, 45 per route | 2.429 ms | 2.085 ms | 2.625 ms | 4.830 ms |

The native subsequent-frame median is 14.1% lower, but its p95 is 84.0% higher
and its first-frame median is 89.0% higher. Every native process has an elevated
second submission interval; those samples remain in the subsequent-frame
population. The submit bracket includes encoder finish and queue submission,
so it does not identify a driver mechanism. Raw phase records and owner totals
remain in the evidence; no phase quantiles were added to manufacture totals.

This is one tiny admitted document on one adapter. CPU governor/preference were
`performance`; display settings remained 59.951 Hz/scale 1 and 143.999 Hz/scale 1.5
on two outputs, with VRR disabled. GPU clocks and power were not locked.
Before/after snapshots describe the surrounding environment, not frequencies
during individual frames. First-frame results do not imply cold system/driver
caches. UI redraw scheduling between samples, compositor work and scanout are
excluded; these durations cannot be inverted into a browser FPS result.

To repeat with a newly built release executable, record its SHA256 and use a
fresh output directory and the installed Vulkan loader directory:

```sh
python3 tools/native_raster_timing_host.py \
  --binary target/release/eris-browser --binary-sha256 YOUR_RELEASE_SHA256 \
  --output /tmp/eris-native-timing-new --loader-directory /path/to/vulkan/lib \
  --frames 16 --repeats 3 --allow-experimental-gpu
```

The host requires Hyprland's `hyprctl` and controls only its own unreaped browser
children. One inherited loopback TCP listener keeps the exact URL constant;
that descriptor is closed across the browser launch. The bounded existing
supervisor remains responsible for cleanup. This comparison provides no
Chromium performance, complete web compatibility or production security claim.

## Native baseline versus retired-buffer reuse

The [reuse evidence](evidence/vulkan-native-buffer-reuse.json) and
[artifact index](evidence/native-buffer-reuse/index.json) retain a separate
eight-process comparison. Both executables use `--raster=gpu` and were compiled
with Rust 1.98 in the normal release profile with `vulkan-raster`. The baseline
is the earlier `51ca9ed` executable (`038ffe37…`), bound to its preserved
120-file source tree; the candidate (`7ba4070d…`) has its own 123-file compiled
manifest. No stale-source exception is used to accept the baseline.

Two correctness processes each match **4,505,600 acquired bytes**. Three
alternating baseline/candidate pairs then complete **96 timed frames**, sixteen
per process, with no verification overhead. All eight processes share one URL,
the exact same 269-byte paint identity, scale 1 and NVIDIA RTX 4070 SUPER
BGRA8/opaque sRGB/FIFO configuration. All exit successfully with no remaining
owned descendants. No preliminary frames are inserted into the timing runs.

| Direct interval and population | Baseline median | Reuse median | Baseline p95 | Reuse p95 |
| --- | ---: | ---: | ---: | ---: |
| Prepare through present, first frame (3 each) | 5.600 ms | 5.264 ms | 5.683 ms | 5.413 ms |
| Prepare through present, subsequent frames (45 each) | 2.086 ms | 1.309 ms | 4.790 ms | 4.117 ms |
| Owner total, first frame (3 each) | 5.307 ms | 5.000 ms | 5.322 ms | 5.099 ms |
| Owner total, subsequent frames (45 each) | 1.818 ms | 1.062 ms | 4.482 ms | 3.874 ms |

For this sequence the subsequent preparation-through-present median is 37.3%
lower and p95 is 14.0% lower. Sample 2 remains in every subsequent population:
its submission bracket is 2.698–2.919 ms for baseline and 2.859–2.959 ms for
reuse. The change does not remove that early tail or establish its cause.
Initialization and individual phases are retained separately. Successful
cache-return bookkeeping follows the present timestamp: it belongs to
`owner_total_ns`, not `prepare_to_present_ns` or the cleanup bracket. Frame
intervals exclude destruction of the final cached lease during positive
owner release; report output still waits for that release. These
are one-scene host observations with unlocked GPU clocks, not GPU execution
times, throughput, compositor latency or a Chromium/general-browser result.
Environment snapshots report P5/360 MHz before and P0/2505 MHz after the
sequence. They do not establish clock speeds during individual frames, and
the entire measured difference cannot be attributed solely to buffer reuse.

The owner retains at most one complete retired buffer lease. Exact context,
dimensions, format, stride, parameter/input sizes and buffer usages must match.
Every encode rewrites all inputs and uniforms, performs the full clear and all
ordered draws, and converts the result; no Plan, pixels, CPU reference or
readback is cached. A mismatch, reconfiguration or CPU fallback drops the old
lease before replacement allocation. Reuse requires tracked submission
retirement, successful scopes, presentation and a final current/deadline check.
Cancellation still discards the incomplete encoder and flushes queued writes
once; uncertain retirement never returns buffers to the cache.

The retained charge survives idle and clears only after actual drop. The
unchanged limits are **16 MiB** for the native bundle, **24 MiB** with optional
readback, and **128 MiB** for the explicit presenter ledger, including the next
UI's **16 MiB** reservation. Queue staging allocations and the documented driver,
binding and allocator overhead remain outside that explicit-buffer ledger.

Separate offscreen tests exercise changed same-size contents, both output
formats, shape misses and cancellation on three adapters: **84 frames, 45 reuse
hits, 18 cancellation guards and 4,680 literal comparison bytes**. The **39
successful-frame allocations** exclude the 18 additional cancellation leases.
They do not count allocations in the measured window processes. Rust 1.88 and
1.98 each pass **1,397 native-feature tests** and **107 GPU core groups**;
Rust 1.88 also passes **84 default core groups**. All **450 Python groups** pass.
The initial checker compile failure (`E0599` during Clippy), preexecution oracle
literal correction and host source-sort correction remain retained with their
corrected records; there was no probe unit-test failure in this increment.

To reproduce A/B mode, retain each release's compiled manifest and source
snapshot at build time, then supply the separately pinned manifest hashes.
Each manifest records `binary`, `bytes`, `sha256`, `profile: "release"`,
`toolchain`, `features: ["vulkan-raster"]` and complete `source_files` rows
(`path`, `bytes`, `sha256`). Build both releases with the same toolchain/profile.
The candidate source is the current checkout; `--baseline-source` points to the
separate complete baseline tree. The host verifies both inventories and
executables before and after cases, and copies both manifests into the result.

```sh
python3 tools/native_raster_timing_host.py \
  --comparison native-ab \
  --binary /path/to/candidate --binary-sha256 CANDIDATE_BINARY_SHA256 \
  --candidate-manifest /path/to/candidate-release.json \
  --candidate-manifest-sha256 CANDIDATE_MANIFEST_SHA256 \
  --baseline-binary /path/to/baseline \
  --baseline-manifest /path/to/baseline-release.json \
  --baseline-manifest-sha256 BASELINE_MANIFEST_SHA256 \
  --baseline-source /path/to/baseline-source \
  --output /tmp/eris-native-ab-new --loader-directory /path/to/vulkan/lib \
  --frames 16 --repeats 3 --allow-experimental-gpu
```

This mode requires exactly three pairs of sixteen frames. It stops at the first
failure and keeps that attempt's raw records. Omitting `--comparison native-ab`
retains the original single-executable CPU/native comparison mode.
