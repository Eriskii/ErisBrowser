# Native gradient fragments

The native raster path accepts an opaque, square-corner linear gradient outside
all retained opacity scopes. The CPU evaluates its visible pixels with the same helper
as Canvas, then the existing GPU image shader copies that fragment into the
scene. Gradient interpolation remains on the CPU.

```sh
cargo run --locked --release --features vulkan-raster -- \
  ./examples/vulkan-gradients.html --presenter=vulkan --raster=gpu
```

The example has an interpolated ramp, a clickable hard edge using `currentColor`,
and a fixed-position solid gradient. Clicking changes the two hard-edge colors
without changing the element tree or hit regions. Native admission uses a
physical target of at most 1,280 × 1,024 pixels; display scaling or a larger
desktop window can still select software fallback. The browser’s `--width` and
`--height` rendering options do not set desktop window dimensions.

## Admission and pixels

Every resolved stop, including `currentColor`, must have alpha 255; the radius
must be zero. Any enclosing retained display-list opacity scope refuses the
gradient, including `PushOpacity(1)`. Ordinary CSS `opacity:1` creates no such
scope and does not itself make a gradient ineligible. Transparent stops and
rounded gradients also retain whole-scene software fallback even when clipped or hidden. Existing non-gradient
opacity rendering is unchanged.

The helper samples the original translated tile at framebuffer pixel centers.
It preserves angle/corner geometry, length and percentage stops, zoom scaling,
background repetition, fractional clips and document/fixed offsets. The fragment
covers only the effective integer window; cropping does not restart its tile.
No cache or persistent generated image is added.

A bounded core `ScreenImage` command places that fragment at an absolute integer
framebuffer origin. It checks source, window, viewport and active clip bounds,
then uses the existing source packing, identity lookup tables and image shader.
Offsets are not applied a second time. The browser display-list/worker command
remains `LinearGradient`; `ScreenImage` is internal to native preparation.

## Shared limits

For a nonempty prepared window with `N` stops and `P` candidate pixels in
Canvas's clipped destination loop,
the existing evaluation tariff is `T = 4*N + P*(ceil(log2(N)) + 2)`.
Native preparation reserves `T + Q`, where `Q` is the number of pixels written
to the generated fragment. Fully clipped eligible gradients generate no source
or sampling debit. Shared work/source/lookup/GPU-byte admission precedes fragment
allocation. Existing final core invocation and group checks run after bounded
preparation and before any GPU submission.

Loaded images and generated fragments share the unchanged **1 MiB RGBA-source
allowance** and **256 source slots** across the whole scene. A large gradient
may therefore still fall back. Viewport, command, scope, phase, invocation and
GPU-buffer caps are unchanged. Partially prepared owners are dropped on refusal;
no partial native scene is submitted.

Statistics keep loaded-image and generated-gradient counts/bytes separate and
report generated evaluation/preparation work. Generated pixels are uploaded to
the GPU, so avoiding a whole CPU framebuffer upload does not mean avoiding CPU
pixel generation or all pixel uploads. Preparation peak storage includes both
the temporary generated owners and their packed-plan copy.

## Validation

The same three frozen literal frames exercise interpolation/px-stop scaling/
currentColor, repeated corners with fractional clipping, and document/fixed
scope restoration. Canvas and packed Native records match their independent
literal pixels. Direct Page and confined-worker click witnesses match each
state’s entire pixel canvas while preserving the element graph and hit regions;
all three predecessor checks also pass with only the native-admission
expectation changed to refusal.

Final-source Rust runs pass 2,520 native-feature tests on Rust 1.88 and 2,326
default-feature tests on Rust 1.98. The standalone core GPU-feature suite passes
154 tests; the probe has 15 passing tests and all 107 bridge tests are discovered.
All 68 self-authored browser references pass. Strict browser/core/probe Clippy
checks and formatting checks pass.

NVIDIA RTX 4070 SUPER, AMD RADV Raphael and llvmpipe each pass all three fixtures
in BGRA and RGBA: six offscreen comparisons and 608 active pixel bytes per
adapter, 18 comparisons and 1,824 bytes total. Every pixel agrees with both
Canvas and the frozen literals. Earlier GPU fixture rosters and caps are unchanged.

The preceding software-gradient commit's CI passed eight of nine jobs; its
probe job failed because two legacy checker matches lacked gradient arms.
This checkpoint adds explicit rejection outside those checkers' fixture domain,
retains their old fixture rosters, and passes the strict probe check. A test-only
Clippy clone warning is also corrected; failed attempts remain in the evidence.

No acquired-window or timing run is claimed. These bounded checks do not establish
performance, full CSS conformance, security or broad hardware compatibility.

The [evidence summary](evidence/vulkan-gradients.json) and
[archive](evidence/vulkan-gradients.tar.gz) retain sources, checks and failures.
