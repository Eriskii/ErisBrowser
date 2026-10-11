# Layered backgrounds

Background images now accept up to eight comma-separated `none`, supported
`linear-gradient()` or `url()` values. This extends the existing image, geometry
and box-selection paths; it adds no image kind, paint command or shader.
Try the [layer-order demo](../examples/background-layers.html):

```sh
./run.sh ./examples/background-layers.html
```

The demo places orange bands above a green/blue base, then reverses the images.
An empty middle slot stays part of both lists.

## Lists, shorthand and canvas ownership

The image list determines the number of used layers. Size, position, repeat,
origin and clip keep independent authored lengths, each from one to eight.
Shorter dependent lists repeat during used-value lookup; longer lists retain
unused entries. Inheritance copies those complete values. A `none` or unavailable
image keeps its authored index, so it cannot shift another layer's geometry.
The first image is on top. The single background color paints underneath all
images, using the clip selected for the last image slot.

For example:

```css
.panel {
  background-color: navy;
  background-image: linear-gradient(to right, red 50%, transparent 50%),
                    none, url(tile.png);
  background-size: 16px 100%, auto;
  background-repeat: repeat-x, no-repeat;
  background-origin: content-box, padding-box;
  background-clip: padding-box;
}
```

The third slot uses the first size, repeat and origin values. Each image keeps
its own positioning phase, clip and repeat axes. The existing physical position,
size, border/padding/content box and linear-gradient grammar still applies.

The shorthand expands atomically into seven longhands. Every layer resets its
omitted components; a color is allowed only in the final layer, including when
that color is transparent. Empty layers, a ninth value, malformed nesting or
mixed CSS-wide/list syntax reject the declaration. Token-aware splitting leaves
commas and comment-like bytes inside strings, URLs and nested functions intact.
Variables, CSS-wide values, `all`, inheritance and cascade-layer reversion use
the same stored lists and consuming-declaration URL base rules.

CSSOM keeps explicit longhand cardinalities in `cssText`. Its shorthand getter
uses the image count and returns an empty string for incompatible list lengths,
except a singleton dependent value with the exact canonical initial spelling
may be omitted. Repeated equal entries are not collapsed, and a singleton
`none` image is never expanded to fit a longer geometry list. Equivalent
noncanonical initial spellings can conservatively produce an empty getter.
This is a bounded serialization rule, not complete CSSOM minimization. See
[CSS Values coordinating lists](https://drafts.csswg.org/css-values-4/#linked-properties)
and [CSSOM serialization](https://drafts.csswg.org/cssom/#serialize-a-css-value).

Canvas propagation has one shared selector for resource preparation and layout.
A transparent root permits body propagation only when its image list is exactly
one `none`; `none, none` blocks that propagation. An unavailable root URL also
retains root ownership. This follows the selected reading of the
[CSS body-propagation rule](https://drafts.csswg.org/css-backgrounds-3/#body-background);
the inspected Gecko implementation agrees and Blink differs, so this is not a
cross-engine agreement claim. Canvas painting retains the existing root
geometry/current-color behavior and ignores the selected layer clip for its
canvas extent. The chosen owner's backgrounds are not painted a second time
on its principal box.

## Geometry and resource preparation

Principal boxes, existing table-row/cell paths and ordinary wrapped inlines
paint the layers in reverse authored order, below their border. Origin and clip
selection remain independent for every layer. Table stretching keeps the final
flow/hit geometry even if retile paint work is refused; accepted keys and
payloads are reused. Inline border/padding/content strip sums and physical end
cuts retain each owner's phase independently of missing or refused commands.
Broader table painting, writing modes, bidi, `box-decoration-break: clone`,
attachment modes, `space`/`round` repetition, blend modes and `background-clip:
text` remain outside this subset.

Page prepares one flat slot table addressed by node and authored image index,
then passes a borrowed view to layout using the same computed styles. Slots and
ranges are admitted before resource work. URL layers use the retained page
fetcher, requested-URL success/failure caches, taint and cumulative decode
allowance. Fragment aliases share an authenticated cached allocation; separate
requested URLs are not merged merely because they redirect to the same final
URL. Failed-cache hits still consume their existing URL/metadata tariffs.
Read-only layout performs no fetch, decoding or key publication.

## Bounds and rendering routes

The 4,096-byte CSS value cap and 20,000,000 style-work allowance remain. Immutable
background records replace six scalar fields; singleton lists are inline in the
record, and longer lists share interned slices. Lists, records, table growth and
gradient/URL atoms consume one cumulative 4 MiB pool. Comparison, copy, clone and
growth work is prepaid; allocation tariffs include aligned headers and named
slack, with target-dependent sizes. Discarded retained intermediates and failed
work are not refunded. Successful singleton parser and atom tariffs remain,
but the added record/list costs can refuse background records earlier; other
style properties and background color remain independently published.

Page preparation retains its 16 MiB temporary-metadata and 16,777,216 work caps,
4 MiB cumulative background metadata, original request/deadline authority and
64 MiB decode/live-image allowances. Checked range and slot totals include all
authored slots of selected URL-bearing nodes, including none/gradient gaps.
There is no fresh allowance per layer or resize.

For multiple image slots, principal groups reserve `5 + N` commands, canvas/row
groups `1 + N`, and inline groups `N + 4`, including inert slots. New principal
shape preparation prepays `16 + 16*N`; inline finalization charges additional
image work alongside its existing owner/geometry scans. Existing 200,000-command,
2,000,000 positioning-work and 4,096-per-key/65,536-total key-byte limits remain.
Group admission precedes key cloning or decoration payloads. Refusal does not
rewind flow or strip phase; later global restacking limits can still truncate
an emitted prefix, so this is not an atomic whole-frame guarantee.

Existing Canvas and experimental Native routes consume the emitted commands.
Native color masks, eligible integer square image/opaque-gradient boxes and
whole-frame fallbacks keep their existing limits and eligibility rules. More
layers can reach those shared limits sooner. No new acquired-window, GPU
comparison, performance or production-security result follows from this change.
These work and storage tariffs are policy bounds, not complete allocator or
instruction measurements.

## Validation record

The final Rust 1.88 suites pass all 2,648 default-feature tests and all 2,952
`vulkan-raster` tests, including ignored confined-worker tests. Strict Clippy
checks pass for every target on Rust 1.88 and 1.98. These are 50 additional
Rust test groups over the parent checkpoint; they cover parsing, storage,
CSSOM, layout, resource preparation, literal page pixels and worker mutation.
The worker test deletes the source images after its first render, then changes
layer order and resizes using the retained page cache.

All 94 self-authored render comparisons pass. Each side of the four new pairs
also matches a literal 320×240 RGBA frame frozen before implementation. The 180
validated PNGs from the 90 older pairs are byte-identical to output from the
published parent browser. Eligible page and worker cases also match an
independent evaluator of the Native packed plan. These checks do not establish
full web conformance or add a hardware/window result for this checkpoint.

DWARF from the Rust 1.88 x86_64 Native test binary reports `ComputedStyle` at
1,024 bytes, down from 1,096. `PaintOwner` (16), `BoxGeometry` (72), `Fragment`
(136), `InlineItem` (176) and `InlineKind` (152) retain their parent sizes.
This measures those compiled types, not total allocations, stack-frame size or
runtime performance. The rendered binary is a development build with debug
sections removed; no Chromium timing comparison was performed.

The [evidence summary](evidence/background-layers/summary.json) binds the source,
browser, checks and [source/check archive](evidence/background-layers/sources-and-checks.tar.gz).
Earlier failures are retained: the four pre-implementation pixel groups; two
missed old-test API adapters; a table fixture that incorrectly assumed
`border-spacing: 0`; and a Clippy simplification. The table input now cancels
the existing 2px offset explicitly, preserving all frozen pixel expectations.
Review also replaced repeated exact-capacity reservations with amortized
fallible growth before testing.

An initial Native suite failed in an unchanged timezone-helper test with
`Unavailable`. A traced run did not reproduce that failure and instead timed
out a different private large-request test. The launch cause remains
unconfirmed. A test-only spawn-error diagnostic now preserves the OS error;
production behavior and deadlines are unchanged. Both final suites pass. The
archive retains these failures alongside the earlier parent CI triage, without
claiming that the underlying intermittent failure is fixed.
