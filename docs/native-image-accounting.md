# Native image work accounting

The optional Vulkan raster route reserves each Image command’s clipped CPU
sampling-loop area instead of a full framebuffer. This lets small-image scenes
use the existing work allowance more precisely. Source decoding, sampling,
blending, packed drawing plans and shaders keep their existing behavior; no
resource limit is raised.

```sh
cargo run --locked --release --features vulkan-raster -- \
  ./examples/vulkan-small-images.html --presenter=vulkan --raster=gpu
```

The example repeats one four-color 2×2 PNG in forty 24×24 image boxes. Click the
first image to widen it to 32 pixels and change the title; click again to restore
it. The handler preserves the image resource and existing nodes. The native
window also includes browser controls, fonts and viewport scaling, so its
complete scene must still pass every admission check. Software remains the
default renderer and the headless path.

## The sampling-loop bound

The bound follows `Canvas::image` after document or fixed-coordinate translation
and intersection with the active clip. Lower integer bounds are the visible
origin’s floor clamped to zero. Upper bounds are the reconstructed visible
endpoint’s ceiling clamped to the framebuffer extent. The product of the two
nonnegative integer spans is charged before sampling or blending.

An image’s lower bound does **not** include the rectangle painter’s additional
clip-origin ceiling. At fractional clip edges, the image loop can sample pixels
that `blend` then rejects. Counting only changed pixels, GPU coverage or a
rectangle mask would omit that work.

The existing geometry pass now handles both Rect and Image. Each phase starts
with a fresh coordinate state. Fixed scopes use the caller clip and viewport
offset, then restore the previous state when popped; nested clips and opacity
scopes retain their structural checks. Image charges across phases are added
with checked arithmetic before fonts receive the remaining pixel allowance.
The shared CPU limit remains `(framebuffer_area * 16).clamp(1_000_000, 32_000_000)`.
Line and both opacity markers still reserve a full framebuffer each.

The estimate remains conservative: missing sources, transparent samples and
zero-opacity descendants still pay their image geometry. Empty or completely
clipped destinations cost zero after original input validation. Hidden commands
and malformed or unreferenced image resources still undergo the existing
validation. Image-store deduplication, source and lookup-table limits, glyph
allowances, rounded masks and whole-scene fallback remain in force.

## Preparation cost and storage

The pass is bounded by 256 original commands and four phases. Its scalar image
helper allocates nothing. An image-only phase now also allocates the bounded
coordinate-state scope vector already used for rectangles. One vector is live
at a time and drops before later image, glyph and lowering preparation. The
existing conservative metadata reservation already covers it; its formula and
all storage caps remain unchanged. There is no per-image geometry cache or extra
source lookup. A failure of this newly reached allocation still refuses the
whole native plan and uses complete CPU fallback.

This exchanges a small bounded preparation traversal for a tighter CPU estimate.
It establishes no rendering-speed improvement and adds no image-format or CSS
support.

## Frozen scene and observed checks

At a 512×512 Page viewport, both loaded Page and confined-worker checks match
these independent literal expectations:

| Quantity | Initial | After click |
|---|---:|---:|
| Image commands / decoded sources |40 /1|40 /1|
| Decoded source bytes |16|16|
| Parent Image pixel reservation |10,485,760|10,485,760|
| New Image pixel reservation |23,040|23,232|
| Unchanged CPU allowance |4,194,304|4,194,304|
| Core draws, including clear |41|41|
| Core invocations, including clear/conversion |547,328|547,520|
| Planned GPU buffer bytes |2,115,360|2,115,392|

The parent refuses both states with `CpuPaintBudget`, while the independent core
plan already fits. The candidate admits both states and matches that core plan’s
complete ordered draws, parameter bytes and input arena. The HTML and PNG are
identical across both revisions.

The [Page and confined-worker tests](../tests/vulkan_small_images.rs) each check
both phases through a real image hit and click, retain 44 node identities
and the full parent/child graph, and compare all 262,144 pixels with independent
literal color quadrants. Exactly 288 pixels change. Each path also checks
the decoded source bytes, actual image commands, CPU Canvas output and a CPU
decoder of the packed plan. The predecessor run passes both explicit-refusal
groups; the candidate run passes both admission groups.

Rust 1.88 passes all 65 focused native-preparation tests, including a 3,000-case
comparison against the actual `Canvas::image` pixel-budget debit. That finite
grid covers fractional/offscreen geometry and normalized caller clips; it is
not exhaustive floating-point coverage. The full native-feature suite passes
2,420 tests across 25 result groups, including the confined integration tests.

The first focused run is retained: 64 passed and one historical quota fixture
failed because its 1×1 missing image no longer consumed an entire 250×200 frame.
The fixture now uses an actual 250×200 destination for the same exact-budget and
over-budget assertions. No assertions or limits were relaxed; independent new
tests retain the small/missing-image debit checks.

Rust 1.98 also passes all 65 focused native tests. Strict Clippy passes on both
Rust versions, and the formatting check is clean. The native browser debug build
passes; no release timing comparison was performed.

## Acquired native texture

A separate real-window run draws the initial frozen icon scene with the custom
shaders on an NVIDIA GeForce RTX 4070 SUPER, using the NVIDIA driver and a
`Bgra8Unorm` surface with sRGB color space, opaque alpha and FIFO presentation.
The owned window is 800×600 logical pixels at 150% display scale, giving an actual
1200×900 physical texture. All **4,320,000 bytes** match the complete Canvas
reference before the desktop compositor. The native presentation records
`CPU_upload=false` and `reference=true`.

The presented scene is serial 9, generation 1, viewport revision 5: 40 page commands,
one image source and three composition phases. Its complete lowered scene,
including browser controls, contains 221 commands and one rounded mask, with
1,300,736 raster invocations and 2,385,536 including conversion. Another prepared
scene, serial 8, was not presented; startup viewport fallbacks remain in the raw
record. There is exactly one presented and verified frame, not two.

The first window attempt is retained as a failure. Requesting 1180×880 logical
pixels at the same display scale produced 1770×1320 physical pixels, exceeding
the unchanged 1280×1024 native limit. It took complete CPU fallback and verified
zero of the requested one native texture. The second checker requested the
smaller logical size; the engine, fixture, candidate binary and limits stayed
unchanged. Both attempts completed owned-process cleanup. All eight corrected
window-protocol tests pass.

The acquired check covers the initial scene only. The real click and changed
state are separately validated by the Page/worker CPU and packed-plan checks;
those checks alone are not GPU execution. This is a single adapter and surface
format, not compositor output, a driver survey or a rendering benchmark.

The [evidence summary](evidence/native-image-accounting.json) and
[archive](evidence/native-image-accounting.tar.gz) retain both window attempts,
source bindings, the original quota-fixture failure and correction, all selected
validation logs and the frozen image assets. Full web compatibility, production
security and Chromium-relative performance remain unfinished.
