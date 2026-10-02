# Native rendering measurements

`--benchmark-native N` measures 2–128 completed frames through the existing
Vulkan window owner. It compares Eris's CPU framebuffer upload and native
shader route in the same `vulkan-raster` build. This is an opt-in experiment;
source, unit and CLI checks pass, and one controlled desktop comparison is
retained below. It establishes
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
| `encode_upload_ns` | CPU channel conversion and texture upload, or native buffer allocation, metadata writes and raster/conversion/copy encoding. |
| `submit_ns` | Encoder finish and queue submission where reached. |
| `completion_wait_ns` | Existing wait for the tracked submission. |
| `cleanup_ns` | Native error-scope collection and frame-resource retirement; absent for ordinary measured CPU upload. |
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

The [API evidence](evidence/vulkan-native-timing-api.json) retains 1,387 passing
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
