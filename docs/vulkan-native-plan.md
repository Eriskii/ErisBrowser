# Proposed native GPU raster integration

This proposes native scene construction and presentation. The offscreen
checkpoints do not change native window rendering. Native
Eris still paints a complete CPU `Canvas`; its optional Vulkan presenter
uploads those pixels. The custom probe renders bounded offscreen plans and
compares readback. Neither result establishes native GPU raster performance
or general web typography support.

The [raster-core extraction](../crates/raster-core/README.md) is now implemented:
primitive planning and optional GPU encoding have a browser-independent owner.
The browser adapter now lives in `graphics::raster_bridge`, enabled by a
planner-only feature with no GPU dependency. Native scene/presentation portions
below remain work to implement. Existing probe imports continue through reexports.

## Current seams

[`Browser::draw`](../src/browser.rs) paints the page, focus outline, scrollbar,
process errors, toolbar, address selection and status into one canvas. Page
coordinates include zoom, document scroll and fixed-position offsets. Chrome
uses direct canvas calls after the page's paint budget ends. The address bar
has radius 7 and the scrollbar has rounded corners: lowering the entire native
scene through the present adapter would therefore hit its rounded-rectangle
refusal even for a small window and a trivial page.

[`Presenter`](../src/presenter.rs) and its
[`Packet`](../src/presenter/vulkan/control.rs) accept completed CPU pixels,
not display lists or GPU plans. The
[`probe executor`](../tools/vulkan-raster-probe/src/gpu.rs) creates offscreen
buffers, compares them with supplied expected pixels, and destroys them. It
does not return a presentation image or own a native surface.

## Reusable core and scene construction

Extract a small renderer core that owns primitive inputs, validated plans,
the shared coordinate state and optional device execution. It must not depend
on browser, worker, DOM or font types. Put the browser/font adapter above that
core. Both the browser and probe can then depend on the core; making the root
package depend on the current probe would create a dependency cycle because
the probe's browser feature already depends on the root package. Keep frozen
probe fixtures and comparison executors outside production renderer APIs.

Build one immutable native scene in the existing paint order. Keep the page's
document and fixed coordinate spaces explicit, and retain the viewport clip
that excludes toolbar and status. Add focus, scrollbar, error overlays and
chrome after the page. Preserve the current zoom arithmetic order; do not
change glyph placement by recomputing scaled coordinates inside shaders.
Preparation should produce owned bounded buffers, with no snapshot or font
borrow crossing into asynchronous GPU work.

The first coherent composition increment needs rounded-rectangle support for
chrome. Specify its coverage from the existing CPU primitive, including radius
clamping, fractional edges and translucent corner blending; freeze independent
small cases before implementing it. Do not approximate radius 7 with a square
or silently drop a scrollbar. Group opacity remains a separate unsupported
primitive until its intermediate surfaces, rounding and storage accounting
are implemented. A remaining unsupported operation rejects the complete
scene for custom GPU rendering.

Keep an explicit complete CPU fallback before submission. It paints the
original page and chrome through the existing path, including the separation
that prevents page exhaustion from starving chrome. Its pixels may use the
existing software/upload presenter, but this must remain identified as CPU
rasterization. Uploading a CPU-painted page as an image is not the custom GPU
render path. Likewise, mask preparation may supply coverage, never a painted
final framebuffer disguised as glyph input.

## Frame-wide budgets and bounded paging

Preserve the existing probe caps and their tests. Its 320×240 viewport is only
one restriction: 256 original/lowered commands, 32 scopes, 256 combined
sources, 1 MiB of explicit GPU buffers and four million padded invocations
also constrain admission. Text has separate complete-input, cold-request,
coverage, row-table and per-occurrence CPU limits. Existing native presentation
has its own bounded pixel capacity and application-memory ledger; those do
not automatically authorize larger custom-raster plans.

If later work introduces tiles or submission pages, specify a frame-wide
ledger first. Charge all decoded/prepared inputs, expanded glyph operations,
source packing, temporary buffers and padded dispatches across the entire
scene. Splitting a list must not reset command, scope, text, allocation or work
limits. Deduplicate source identity once per admitted frame. Count any retained
old and replacement buffers during growth and any overlap between pending and
active frame storage. A broader native budget needs an explicit reviewed
contract; paging alone does not justify increasing the probe's caps.

Each tile must preserve original draw order and paint each output pixel once
per contributing primitive. Maintain absolute glyph placement and translate
only at a defined tile-addressing boundary, with checked integer arithmetic.
Clip/fixed state spans list pages and cannot reset at each batch. A page that
begins inside a clip or fixed scope needs the validated state at that point.
Opacity, once supported, cannot be split as if its children independently
blend into the final surface. The unconditional clear must run exactly once
per output pixel, not erase previous batches.

Validate admission for the whole scene before native submission. A failure in
a later preparation page must not expose an earlier partial frame. Temporary
GPU work must finish into an offscreen target before presentation, or use a
separately specified transactional surface strategy. Refusal and partial
resource construction must release their owned storage without increasing
retained-cache allowances.

## Ownership, presentation and failure

Reuse the native presenter's single graphics owner and bounded replacement
queue. A new packet can carry either a validated owned GPU plan or an explicitly
selected CPU fallback, plus the existing generation, viewport revision and
serial. Recheck staleness after preparation, acquisition and submission and
before presenting. Retire every queued submission exactly once even if it
became stale. Resize, occlusion and navigation must never present an obsolete
plan or accumulate unrestricted pending work.

Custom raster output is packed `0x00RRGGBB`. Native surfaces currently choose
RGBA8 or BGRA8 with opaque alpha and SRGB color space. Add a GPU conversion or
presentation pass that writes the selected channel order with alpha 255;
copying packed storage words directly would not preserve that contract. Keep
integer raster blending distinct from surface color-space handling. Verify
the acquired native texture, including odd widths, channel order and alpha,
before claiming presentation equivalence. The normal rendering path should
not require CPU comparison pixels or readback every frame.

Admission refusal is distinct from device, submission, mapping or presentation
failure. Preserve the existing positive acknowledgment after actual graphics
resource destruction before another native surface owner can take over.
A timeout or finished thread handle alone is not proof that ownership was
released. Keep bounded cancellation and retirement behavior rather than
retrying a failed frame indefinitely.

Worker startup after driver initialization must retain the clean-exec launcher
and the worker's independent descriptor checks. Any new feature configuration
that enables a driver must also preserve this boundary; it must not bypass it
because the old condition was named `vulkan-presenter`. Renderer workers do not
gain driver descriptors, filesystem access or GPU authority.

## Preparation and validation sequence

1. Complete real-worker text capture tests with frozen geometry, styles, scopes
   and image assumptions. Keep literal coverage oracles distinct from outputs
   derived from the published CPU font renderer.
2. Freeze the native scene and packet interfaces; separate the reusable core
   without changing the old probe inputs, shaders or refusal rules.
3. Add rounded chrome and independently validate whole-scene composition,
   zoom/scroll/fixed order, complete fallback and page/chrome budget isolation.
4. Validate any paging ledger and GPU surface conversion before connecting
   native presentation. Test stale-frame replacement, resize/occlusion, device
   failure and release acknowledgment using the existing ownership model.
5. Measure cold/warm preparation, CPU fallback frequency, GPU submission and
   native presentation separately. No latency, throughput or Chromium-relative
   claim follows from the current offscreen correctness results.

The bundled-font session still depends on pinned font-library outline and
raster routines with infallible internal allocations and noncancellable work.
Its explicit caps do not create an instruction deadline inside those routines.
Author/system fonts, shaping, bidirectional layout, fallback-font selection and
full typography remain outside this proposal's initial rendering slice.
