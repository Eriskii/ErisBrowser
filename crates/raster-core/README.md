# Eris raster core

This crate owns the browser-independent drawing planner and optional GPU
encoding used by Eris's Vulkan experiments. It does not parse HTML or CSS,
prepare fonts, start workers, create windows or select a graphics adapter.
The default feature set has no external dependencies. The `gpu` feature adds
the same pinned Vulkan/WGSL implementation used by the probe.

The planner accepts unrounded rectangles, decoded images, glyph coverage and typed
clip/fixed scopes. Native additionally accepts bounded opacity scopes. It validates the complete input before producing an owned,
immutable `Plan`. Existing limits, validation order, integer blending and
floating-point coordinate arithmetic are preserved from the standalone probe.
`Plan` and `Draw` fields remain private. Browser-specific whole-frame fallback
and bundled-font preparation belong to the separate browser adapter.
Adapters using the shared `CoordinateState` must stop on any error and discard
that state; failed scope operations are not a recoverable transaction.

## Admission profiles and rounded coverage

Existing `plan`, `plan_with_images`, `plan_with_masks` and
`CoordinateState::new` select `Profile::Probe`. Their limits and encoded drawing
bytes remain unchanged. The additive `plan_with_masks_for_profile` and
`CoordinateState::new_for_profile` accept the closed `Profile` enum:

| Limit | Probe | Native |
| --- | --- | --- |
| Maximum viewport | 320×240 | 1280×1024 |
| Planned explicit GPU buffers | 1 MiB | 16 MiB |
| Padded compute invocations | 4,000,000 | 4,000,000, including conversion |
| Image lookup-table entries | 143,360 | 589,824 |

Both profiles retain 256 commands, 32 typed scopes, 256 combined image/mask
sources, 1 MiB of supplied image RGBA, 1,024 pixels per mask axis, 262,144
aggregate coverage bytes and 65,536 row origins. The larger lookup-table bound
is derived from viewport axes and command count; its storage remains charged.
A Native viewport is an upper bound, not a promise that any scene at that size
fits the other limits.

`plan_native_phases` accepts a canonical full-target frame and up to four
`Phase` values. Each phase has its own caller clip and document/fixed offsets;
scopes must balance within that phase. Commands, sources, input storage and
dispatch work share one frame-wide ledger. The result clears once, draws the
phases in order and reserves one final surface conversion. Phase boundaries
cannot renew a limit or create an independently presentable prefix.

`Plan::retained_cpu_bytes` charges its actual vector capacities and metadata.
`native_planner_metadata_peak_bytes` exposes a conservative structural allowance
for callers accounting for preparation overlapping the finished plan. These
are explicit payload bounds, not process-RSS or driver-allocation bounds.

`rounded::RoundedShape::prepare` validates already translated rectangle/clip
geometry and computes placement, cropped coverage size and the full CPU loop
debit without allocating a mask. Its disposition distinguishes empty geometry,
zero radius and a positive-radius shape. `materialize` calls the caller's
cumulative preflight before fallible coverage/row allocation. It preserves the
CPU painter's floating-point coverage arithmetic and returns coverage plus
absolute row origins for `Command::Glyph`; it never paints RGB. Zero radius
must use the ordinary rectangle path. Positive-radius `Command::Rect` remains
unsupported unless the caller explicitly performs this lowering.

`rounded::RoundedTiles::prepare_native` additionally partitions a wide Native
shape into at most two adjacent horizontal masks, each still at most 1024
pixels wide. Coverage uses the original floating-point geometry and absolute
pixel coordinates; no new corners or overlapping pixels are introduced at the
split. One aggregate preflight charges all masks, extra row tables and lowered
operations before either tile is allocated. Original primitive loop work is
counted once, and allocation failure returns no partial collection. Aggregate
coverage, source, row, storage and dispatch limits remain unchanged. The legacy
single-mask API and Probe admission rules retain their existing refusals.

Native opacity groups require `k/256` values and support transparent descendants
without an opaque backing. Each materialized group
gets a disjoint cropped RGBA16 scratch region; clears and composites count toward
the same operation, buffer and dispatch ledgers. Zero scopes suppress pixels only
after input validation, and unit scopes preserve the current target. Non-grid
opacity returns an error for whole-frame fallback. Integer emulation preserves
the software painter's separate binary32 rounding stages for transparent pops.
See the [admission and arithmetic contract](../../docs/vulkan-transparent-opacity.md).

## GPU ownership

The optional GPU interface accepts a caller-owned device, queue and command
encoder. It uploads drawing parameters and original source data, then encodes
ordered compute passes into a GPU-only RGB output buffer. It takes no expected
pixels and performs no adapter enumeration, readback, comparison or presentation.
The caller supplies cancellation/deadline checks before preparation and each
draw, owns error scopes, and retains the returned frame resources until the
submission completes. On encoding failure, discard the incomplete encoder.

The raster output is packed `0x00RRGGBB`, with `STORAGE | COPY_SRC` usage and no
`COPY_DST`. The `surface` module, also gated by `gpu`, converts a Native frame
to padded `Bgra8Unorm` or `Rgba8Unorm` bytes with alpha 255. It performs no color
space conversion and accepts neither sRGB formats nor Probe plans.

Preflight `SurfaceLayout::for_plan` before raster encoding. Then append
`SurfaceConverter::encode` in the same caller-owned encoder; it consumes the
`EncodedFrame` and returns a `ConvertedFrame` owning both stages' resources.
The caller handles submission, error scopes, cancellation, surface acquisition
and any later texture copy. Explicit destruction does not wait for completion;
the caller must already have confirmed completion of every use.

The additive `surface::NativeEncoder` owns a device/queue/pipeline context for
exact-size Native reuse. `requirements` preflights a fresh Plan and format;
`allocate` returns one complete `NativeBufferLease` before any queue writes.
`encode` borrows that lease and retains its handles on every error. Compatibility
requires the same private context, dimensions/format/stride, parameter/input
sizes, opacity scratch size and actual buffer descriptors. Every encode rewrites the full arenas and
conversion uniform, clears the target and executes every ordered draw.

Only the caller can prove submission retirement and its complete frame policy.
After that proof it may call `mark_reusable_after_completion`; an encoding error
poisons the lease for reuse. An incomplete encoder must be discarded and queued
writes retired before explicit destruction. Uncertain completion uses ordinary
handle drop and owner teardown. There is no retained Plan, pixel content or
readback, and no implicit submission or wait. Existing one-shot and Probe APIs
remain unchanged. The browser keeps at most one retired lease, charges it across
idle, and drops it before incompatible replacement or CPU fallback.

Probe `Plan::gpu_buffer_bytes()` retains the original allowance for **two**
packed targets, parameters and input storage. The probe allocates its readback
separately and checks this total. Native instead reserves one packed target,
the conversion destination with 256-byte-aligned rows, its 16-byte uniform,
raster parameters, input storage and opacity scratch. Native `Plan::invocations()` also includes
one mandatory padded 8×8 conversion dispatch. Optional Native readback is
additional caller-owned storage. These counters exclude opaque driver,
pipeline, bind-group, staging and allocator overhead.

## Compatibility and tests

The [probe](../../tools/vulkan-raster-probe/README.md) reexports the original
planner API, keeps the same fixture inventories and owns supervised execution,
adapter/device selection, deadlines and complete pixel comparisons. Its public
`gpu::run_plans` entry point remains a test harness. Existing shader files under
the probe are retained reference copies; execution uses the core shader sources.

Private planner tests execute as core unit tests. Their existing source files
and pure fixture helpers are included from the probe under `cfg(test)` only;
production builds have no dependency on that harness. The two alpha-fixture
structural tests also remain in the probe. Testing the probe alone does not run
the core's unit tests, so CI invokes both crates explicitly:

```sh
cargo test --locked --manifest-path crates/raster-core/Cargo.toml
cargo test --locked --manifest-path crates/raster-core/Cargo.toml --features gpu
cargo test --locked --manifest-path tools/vulkan-raster-probe/Cargo.toml --features browser-bridge
```

CPU-only tests do not initialize a graphics device. Actual Vulkan execution
uses the separate supervised probe commands and their retained evidence.

The browser now uses these APIs through its optional
[native-window route](../../docs/vulkan-native-window.md). With the partition
and reuse tests, **107 GPU-feature groups** pass on Rust 1.88 and 1.98;
**84 default groups** also pass on Rust 1.88.
Core tests remain CPU-only. The browser owns actual acquisition, submission,
verification and presentation; separate window evidence is required for those
operations.

The [reuse evidence](../../docs/evidence/vulkan-native-buffer-reuse.json) records
separate real-GPU checks on three adapters: **84 frames, 45 reuse hits, 18
cancellation guards and 4,680 literal comparison bytes**. The **39 allocations
for successful frames** exclude the 18 cancellation leases. These checks cover
changed same-size contents, exact format/shape misses and retirement boundaries;
they are separate from the native-window timing observations.

The earlier [native prerequisite checker](../../tools/vulkan-raster-probe/NATIVE_PREREQUISITES.md)
exercises larger offscreen targets, rounded coverage and byte conversion.
Its eleven cases pass in both formats on three Vulkan adapters, with
81,393,120 bytes compared. Those results are distinct from the later acquired
browser-window checks. Arbitrary opacity values, broader compatibility and
performance remain unfinished.
