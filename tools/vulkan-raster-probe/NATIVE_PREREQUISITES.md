# Native raster prerequisites

The raster core now provides a bounded Native planning profile, rounded-rectangle
coverage preparation and packed-RGB conversion to padded BGRA/RGBA bytes. These
are offscreen prerequisites. The normal browser still paints on the CPU; its
optional Vulkan presenter uploads completed CPU frames. No acquired window
surface or native browser composition is exercised by this checker.

## APIs and limits

`plan_with_masks_for_profile(Profile::Native, ...)` admits at most 1280×1024
pixels and 16 MiB of explicit planned GPU buffers. Existing planner entry points
and browser adapters retain `Profile::Probe`, its 320×240 viewport and 1 MiB
buffer cap. Existing fixture plans and shaders remain unchanged.

Both profiles retain 256 commands, 32 typed scopes, 256 combined image/mask
sources, 1 MiB of supplied image RGBA, 1,024 pixels per mask axis, 262,144
aggregate coverage bytes, 65,536 row origins and four million padded compute
invocations. Native derives its image lookup-table limit from the larger axes,
while charging the resulting storage. These are independent limits: fitting
the viewport alone does not admit a scene.

`RoundedShape::prepare` takes already translated geometry and the active clip.
It validates hidden geometry, preserves the original Canvas floating-point
operation order, and reports both cropped mask storage and full CPU loop work.
Before `materialize` allocates or evaluates coverage, the caller must admit
cumulative work and storage through its preflight callback. Empty and zero-radius
dispositions remain distinct; zero radius keeps the existing rectangle path.
Positive-radius rectangles are explicitly lowered to coverage/row inputs for the
glyph kernel rather than enabled in the old `Command::Rect` path.

This is hybrid CPU geometry preparation and GPU compositing. The CPU supplies
coverage only; the GPU applies source color, coverage alpha and ordered integer
blending. No CPU-painted final target enters either GPU encoder. The default
1180-pixel window's 984×34 address bar fits the mask-axis cap. At a 1280-pixel
window, the same bar is 1084 pixels wide and refuses preparation unless clipped
to a supported width; this checkpoint does not widen the cap or add tiling.

`SurfaceLayout::for_plan` checks Native storage/work before raster encoding.
`SurfaceConverter::encode` consumes the encoded raster frame in the same command
encoder and returns a `ConvertedFrame` owning both stages through completion.
It accepts only `Bgra8Unorm` and `Rgba8Unorm`, writes opaque alpha 255, and leaves
color-space policy to the caller. Submission, device/error scopes, cancellation,
texture copies and surface acquisition remain caller responsibilities. Explicit
destruction requires already proven completion; it does not wait.

Native plans charge one packed RGB target, a destination with 256-byte-aligned
rows, a 16-byte conversion uniform, drawing parameters and source/lookup/row
storage. They also reserve the conversion's
`ceil(width/8) * ceil(height/8) * 64` invocations within the four-million total.
The checker separately allocates padded readback and refuses a plan when
planned buffers plus readback exceed 24 MiB. Allocator, staging, pipeline and
driver internals are outside these explicit-buffer figures.

## Offscreen checker

`eris-vulkan-surface-check` has eleven fixed cases, each checked in both byte
formats. Six tiny rounded cases use manually derived literal coverage/blending
results: corners, repeated translucent drawing, radius clamping, quarter-radius,
fractional placement and clipping. Four literal geometry cases cover asymmetric
RGB channels, a padded 1180×880 frame, the 1280×1024 viewport boundary and the last
pixel of an odd-width row. The eleventh, the 1180×880 address-bar frame, uses the
original Canvas painter as an explicitly differential reference.

Full readback comparison includes all active pixel channels and alpha. Padding
is allocated but is not counted as pixel evidence. Pixel references are used
only by the checker, never by the core encoders. The host retains raw logs and
checks complete case/format records, process cleanup and source/binary/loader
bindings. A GPU or comparison error fails the run. Compilation and unit tests
alone do not establish GPU equality.

The [recorded run](evidence/native-prerequisites/index.json) passes all eleven
cases in both formats on NVIDIA, AMD and software Vulkan: 81,393,120 GPU-compared
bytes, split into 56,471,520 bytes against literal references and 24,921,600
against the Canvas address-bar reference. The original 30 standalone and 26
glyph cases also pass on all three adapters, with their separate reference
populations unchanged.

Both Rust 1.88 and 1.98 pass 66 default and 77 GPU-enabled core groups, six
default and fourteen feature-enabled probe groups, and 28 root bridge groups,
with strict Clippy, formatting and builds. All seven new Python host groups
pass. Initial GPU compilation failed on binding-limit integer types; the
two-file type correction, initial logs and successful default checks are
retained alongside the final checks. No pixel expectation or limit changed.

From the repository root:

```sh
cargo build --locked --manifest-path tools/vulkan-raster-probe/Cargo.toml \
  --features browser-bridge --bin eris-vulkan-surface-check
python3 tools/vulkan-raster-probe/run_surface_host.py --allow-experimental-gpu \
  --binary tools/vulkan-raster-probe/target/debug/eris-vulkan-surface-check \
  --output surface-host-1 \
  --loader-directory /path/to/installed/loader
```

The output directory must be new. This experiment does not establish native
presentation, full browser compatibility, production security or a performance
comparison. The existing [browser bridge](BROWSER_BRIDGE.md),
[glyph](GLYPHS.md) and [worker-text](WORKER_TEXT.md) suites keep their separate
scope and evidence.
