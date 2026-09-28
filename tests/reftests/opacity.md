# Group opacity coverage

The six `opacity-*.html` pairs compare independently specified opaque SVG
rectangles against CSS group opacity. Reference pages do not use opacity,
translucent fills, or the grouping algorithm under test. They cover overlapping
children and borders, nested stacking contexts, fixed descendants and clips,
inline ancestors, a translucent PNG, and the root canvas background. These are
self-authored regressions, not imported Web Platform Tests or a claim of full
compositing conformance.

The implementation follows [CSS Color group opacity](https://www.w3.org/TR/css-color-4/#transparency)
and the [isolated-group and source-over model](https://www.w3.org/TR/compositing-1/#groupcompositing).
An element's background, borders, text, images and descendants first paint onto
a transparent group. Its opacity scales that result once when compositing over
the enclosing group. Primitive source alpha remains separate. For example,
half-opacity opaque red over white is `#ff8080`; overlapping opaque children do
not accumulate the ancestor's opacity independently. The PNG fixture deliberately
uses alpha 128/255, not exactly one half, and its reference accounts for that
source alpha before applying group opacity.

Layout creates opacity scopes after stacking order is resolved. Opacity contexts
remain isolated; real child contexts cannot escape them. Clip/fixed transitions
stay inside their enclosing opacity scopes, so a fixed descendant can escape an
ordinary ancestor clip while retaining ancestor opacity. Painting, clipping and
scrolling use the existing document/viewport coordinate rules. Inline runs share
one group across line wraps. Root opacity includes the propagated canvas
background. Successfully loaded transparent images do not paint a missing-image
placeholder or alt text underneath their pixels.

Temporary pixels use premultiplied RGBA16. Group opacity remains floating point
until source-over compositing, and the public framebuffer remains RGB8. This
avoids quantizing group alpha to eight bits before composition and reduces
rounding through nested groups. It does not add wide-gamut color management,
linear-light compositing, blend modes, filters, masks, or a GPU compositor.

Opacity zero preserves layout and hit regions while suppressing drawing and
surface allocation. Opacity-one protocol scopes pass through without a surface;
normal source-over grouping is invariant in this case. All other groups currently
allocate the caller viewport rectangle, not a tightly fitted content rectangle.
This includes potential fixed descendants outside the current clip.

Resource limits apply before allocation or scope emission:

- Offscreen storage has a 64 MiB peak limit, or 8,388,608 RGBA16 pixels across
  live groups, and a 128 MiB cumulative allocation limit per paint, or
  16,777,216 RGBA16 pixels. Descriptor/allocator overhead is separate.
- Surface initialization and every composited pixel charge the shared painter
  work budget, alongside ordinary drawing. That budget is at most 32 million
  pixels per paint and may be lower for small viewports.
- Clip, fixed and opacity scopes share the 128-level nesting limit. Layout
  reserves closure commands under its 200,000-command limit and removes hit
  regions for content beyond a stacking reconstruction cutoff.
- Invalid opacity values, mismatched or unclosed scopes, allocation failures and
  exhausted paint budgets discard unfinished surfaces, report painting as
  limited, and restore the caller clip and direct-paint state. Completed content
  before a failure may remain visible; there is no fallback that paints an
  unfinished group without its opacity.

These limits intentionally reject some otherwise valid content. In particular,
the maximum 16-megapixel framebuffer is larger than one permitted opacity
surface, and many small groups can consume viewport-sized allocations/work.
Tighter group bounds or tiled surfaces remain a performance improvement.

Unit tests cover premultiplied image alpha, glyph coverage, transparent holes,
source-order isolation, zero-opacity hit targets, wrapped inline runs, scrolled
fixed pixels, row-background ownership and root propagation. Adversarial tests
check allocation limits before reservation, zero-opacity nesting, malformed
mixed scopes, cleanup after work exhaustion, and a deep clipped/fixed/opacity
layout whose command inventory is actually truncated. The mutation harness and
native IPC validator separately enforce typed scope balance.
