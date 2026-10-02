# Eris raster core

This crate owns the browser-independent drawing planner and optional GPU
encoding used by Eris's Vulkan experiments. It does not parse HTML or CSS,
prepare fonts, start workers, create windows or select a graphics adapter.
The default feature set has no external dependencies. The `gpu` feature adds
the same pinned Vulkan/WGSL implementation used by the probe.

The planner accepts rectangles, decoded images, glyph coverage and typed
clip/fixed scopes. It validates the complete input before producing an owned,
immutable `Plan`. Existing limits, validation order, integer blending and
floating-point coordinate arithmetic are preserved from the standalone probe.
`Plan` and `Draw` fields remain private. Browser-specific whole-frame fallback
and bundled-font preparation belong to the separate browser adapter.
Adapters using the shared `CoordinateState` must stop on any error and discard
that state; failed scope operations are not a recoverable transaction.

## GPU ownership

The optional GPU interface accepts a caller-owned device, queue and command
encoder. It uploads drawing parameters and original source data, then encodes
ordered compute passes into a GPU-only RGB output buffer. It takes no expected
pixels and performs no adapter enumeration, readback, comparison or presentation.
The caller supplies cancellation/deadline checks before preparation and each
draw, owns error scopes, and retains the returned frame resources until the
submission completes. On encoding failure, discard the incomplete encoder.

The output is packed `0x00RRGGBB`, with `STORAGE | COPY_SRC` usage and no
`COPY_DST`. It still needs a presentation conversion pass before use with a
native RGBA/BGRA surface. Explicit destruction does not wait for GPU completion;
the caller must already have confirmed completion before requesting it.

`Plan::gpu_buffer_bytes()` retains the original allowance for **two** targets,
parameters and optional input storage. A caller that does not allocate readback
uses only one target, while retaining the stricter admission allowance. The
probe allocates the second target separately and verifies that its readback
plus core-owned buffer bytes exactly match the plan's allowance. These counters
exclude opaque driver, pipeline, bind-group and staging allocations.

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

This extraction does not connect custom GPU rasterization to the browser
window. The 320×240 viewport, operation, source, storage and work caps are
unchanged. Rounded geometry, opacity groups, larger native scenes, presentation
and broader web compatibility remain separate work; see the
[native integration proposal](../../docs/vulkan-native-plan.md).
