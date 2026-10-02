# Next native timing increment

Implement one opt-in, completion-paced native-window benchmark comparing
`--presenter=vulkan --raster=cpu` with `--presenter=vulkan --raster=gpu` in the
**same release binary**, on one fixed loaded scene. Add host wall-clock phase
records to the existing presenter; do not create a second rendering executor,
change its queue policy, or add GPU timestamp queries in this first increment.
This note is source-only preparation. No benchmark, renderer or GPU was run.

## Existing measurements and the gap

`tools/benchmark.py` measures repeated style/layout/software paint through
`--benchmark`. `tools/benchmark_worker.py` and `src/worker_benchmark.rs` add
fresh confined startup/load and repeated validated snapshot exchanges plus
CPU paint. Both exclude native presentation. Their median/p95 helpers and raw
sample reporting are useful, but their results cannot supply native submission
or completion costs.

The native owner already waits for its tracked submission before presenting;
timing that existing wait adds no synchronization step. Its native and CPU
paths share adapter selection, opaque FIFO surface configuration and acquired
texture ownership. They differ where expected: CPU paint/format conversion/
upload versus CPU scene/mask preparation and GPU encoding/compositing/conversion.
First native pipeline creation occurs in `Gpu::new`, before normal frame work.

## Smallest useful implementation

Add a proposed desktop option `--benchmark-native N`, bounded to 2–128 completed
samples, incompatible with verification, screenshot, headless and the two
existing benchmark modes. Keep raster selection explicit. Use the existing
`tools/native-raster-fixtures/admitted.html` and image at a controlled physical
1180×880 viewport initially; do not expand the old nine-page corpus or treat
CPU fallbacks as native timings.

Start only after the owner is initialized and a current loaded snapshot exists
at the settled viewport. Capture a stable scene identity outside timed spans:
complete worker drawing/image inputs, viewport/scale, scroll/zoom, title/address,
focus/selection and other chrome state. Retain the immutable snapshot and a UI
revision token. Reject an input change, resize, occlusion or navigation during
the measured sequence instead of silently mixing scenes. Compare the canonical
input identity across CPU/GPU runs; command counts alone are insufficient.

Time `Browser::draw`'s selected route on each requested redraw, rebuilding the
native plan every sample or painting the CPU canvas every sample. Do not replay
an already-prepared plan while charging CPU painting in the other route. The
native branch must use `reference_requested=false`; no Canvas, expected pixels,
readback, mapping, pixel hashing or comparison belongs inside timed samples.
Run correctness checks separately in fresh processes against the exact same
release executable and input identity.

For this explicit benchmark only, add a bounded completion notification after
the owner has dropped/retired frame resources and called `Shared::idle`. It
requests the next measured redraw. Existing application rendering retains its
one-active/one-latest-pending policy. This first experiment measures serial,
completion-paced frame latency; it does not measure producer overload or
maximum pipelined throughput. Do not busy-loop redraws and then report only
the successful minority of replaced packets.

Likely files:

- `src/main.rs` and `src/presenter.rs`: validate the optional benchmark config.
- `src/browser.rs`: stable-scene gate, CPU/native preparation brackets and
  bounded redraw progression; optionally split these helpers into a small
  `src/browser/native_benchmark.rs`.
- `src/presenter/vulkan/control.rs`: optional packet timing metadata, exact
  route/serial outcomes and bounded completion records.
- `src/presenter/vulkan.rs` and `native.rs`: the timing boundaries below,
  owner initialization timing and completion notification.
- New `tools/benchmark_native.py` and synthetic tests: supervised paired runs,
  strict records/input binding, raw samples and summaries. Reuse the current
  native host's owned-window and subreaper mechanisms; do not copy the older
  benchmark wrapper's simpler process-group cleanup into this new runner.

## Timing boundaries

Use `Instant` and integer nanosecond durations, with optional values for phases
not reached. Capture timestamps into bounded records; serialize after sampling,
not with per-draw logging inside measured spans. Record these separate fields:

| Field | Existing bracket |
| --- | --- |
| `ui_prepare_ns` | `native_scene::prepare(..., false)` plus packet preparation, or `paint_canvas` plus CPU packet preparation |
| `queue_ns` | Packet accepted by `Shared::submit` to returned by `Shared::take` |
| `acquire_ns` | `Gpu::acquire`, including any existing configure/retry; flag whether configuration occurred |
| `encode_upload_ns` | Native buffer creation, queue metadata writes and raster/conversion/copy encoding; CPU format conversion plus `queue.write_texture` |
| `submit_ns` | The existing `queue.submit` call, including encoder finish where applicable |
| `completion_wait_ns` | Existing tracked `device.poll(Wait)` until confirmed retirement |
| `cleanup_ns` | Error-scope resolution and frame-resource retirement/drop before present; absent scope work on CPU remains explicitly absent |
| `present_call_ns` | `pre_present_notify` and `queue.present` returning |
| `owner_total_ns` | Direct owner interval from taken packet to terminal frame outcome and resource release |
| `prepare_to_present_ns` | Direct interval from UI preparation start to successful present-call return |

Keep initialization/device/pipeline time separate from frame samples. Record
terminal status explicitly: presented, obsolete, replaced, acquisition timeout,
occluded, admission fallback, flush-only cancellation or failure. A flush-only
retirement is not a completed raster sample. Preserve the original deadline,
MayHaveWrites cleanup, scope collection and positive Drop release rules on all
instrumentation paths; telemetry must not turn an error into success.

These are **host durations**, not isolated shader execution time. Encoding
includes allocation/driver staging; retirement includes scheduling and driver
waits; FIFO acquisition may wait for compositor pacing. Present-call return is
not photon/display latency. Total quantiles must be computed from direct total
samples, not sums of phase medians. GPU timestamp queries can be a later,
separate feature-capability experiment if isolating kernel time is necessary.

## Cold, steady-state and fairness

Run repeated fresh processes for each route, interleaving route order across
repetitions. Report owner initialization, first eligible loaded-scene sample,
and subsequent samples separately. Call the latter **steady-state repeated
scene** measurements. Native preparation creates a fresh `text_masks::Session`
per frame and does not use `Fonts`' retained CPU glyph cache; rounded masks and
packed plan inputs are also rebuilt. CPU painting can reuse its glyph cache.
That asymmetry is part of the implemented routes, not grounds to relabel native
glyph work as cached or secretly add a cache for the benchmark.

Even the first loaded frame is not a guaranteed cold-font sample: startup CPU
chrome may already have populated `Fonts`. Fresh process does not imply cold
filesystem, shader-driver or GPU caches. Record these limits, native cold-mask
request counts and configuration events; do not flush OS caches or modify
driver settings implicitly. Whole-process startup/load remains a separately
labelled interval or the existing worker benchmark's workload.

Build once with the recorded compiler, `--locked --release --features
vulkan-raster`, and preserve the build log and executable hash. Stripping a
debug binary is not a release build. Use the same executable, adapter, surface
format, FIFO mode, physical viewport, device scale, assets and frozen UI state
for both routes. Record CPU/GPU/driver/loader/compositor versions, display refresh
and available power/clock information. Do not run builds or other controlled GPU
jobs concurrently; retain environmental variation rather than claiming isolation.

The old `benchmark.input_digest()` omits `crates/raster-core` and its shaders.
The new runner must bind root Rust/Cargo inputs, core Rust/WGSL/Cargo inputs,
bundled assets, scene fixtures and its host/parser sources before and after the
run. Reuse a complete source manifest rather than silently inheriting that old
digest's scope. Preserve every raw sample and every failed attempt. A fallback
in the native comparison invalidates that pair; report its reason and measured
attempt rather than dropping it from a successful-frame distribution.

## Tests and first deliverable

Before actual timing, add synthetic-clock tests for boundaries, route/serial
joins, dropped/replaced/flush-only outcomes, exactly N successful samples and
no completion-triggered redraw when the benchmark is disabled. Parser/host
tests reject missing or duplicate records, negative/overflowed durations, changed
scene identity, verification/readback flags, mismatched adapter/viewport, silent
fallback and incomplete cleanup. Keep the record buffer fixed-capacity and
charge its retained capacity within the existing application ledger.

First deliverable: one release-binary paired result for the admitted fixture,
with multiple fresh runs, raw phase samples, first/steady-state summaries,
completion counts and explicit exclusions. Then decide from measured costs
whether native CPU coverage, per-frame allocation, upload or GPU retirement
deserves optimization. Update the now-historical CPU-only wording in
`docs/PERFORMANCE.md` when adding this result; keep its older reports intact.

This establishes only a local comparison of Eris's two native routes. Chromium
comparison still needs pinned Chromium builds, equivalent rendered/interactive
workloads across a representative corpus, matching viewport/fonts/scale/cache
and presentation settings, comparable end-to-end phase definitions, repeated
controlled runs and uncertainty. Missing functionality must remain visible.
No present source check or pixel match establishes Chromium parity.

Read inputs: `tools/benchmark.py` SHA256
`8c721c7553d1282fa64f079572de7e27dd47fe6236f565e89a8bf07aefab84b0`,
`tools/benchmark_worker.py` SHA256
`d860eecba7e2821074422cbae0ecc17310653ade2715a870cc0647583af5610e`,
and current `src/worker_benchmark.rs`, browser scene preparation, native/CPU
presenter owner paths, `text_masks::Session`, native host and performance docs.
