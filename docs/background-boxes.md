# Background origin, clipping and inline decoration

Single-layer backgrounds now accept `background-origin` and `background-clip`
for colors, supported gradients and prepared URL images. This connects the
[rounded decoration primitive](rounded-boxes.md) and
[inline geometry prerequisite](inline-edges.md) to CSS painting. Their earlier
validation records remain historical checkpoints; this chapter describes the
combined behavior.

Try the [interactive example](../examples/background-boxes.html):

```sh
./run.sh ./examples/background-boxes.html
```

Its buttons select border, padding or content boxes together; a second sample
shows a wrapped inline decoration.

## CSS and inline style mutation

| Property | Values | Initial value |
|---|---|---|
| `background-origin` | `border-box`, `padding-box`, `content-box` | `padding-box` |
| `background-clip` | `border-box`, `padding-box`, `content-box` | `border-box` |

Origin selects the rectangle used for image size and position. Clip selects the
background color and image painting area, independently of the border and child
content. The supported shorthand assigns one box keyword to both properties, or
two keywords in origin/clip order. These are the
[CSS Backgrounds box rules](https://drafts.csswg.org/css-backgrounds-3/#the-background).

```css
.badge {
  padding: 4px;
  border: 2px solid black;
  background: linear-gradient(to right, red 4px, blue 4px)
              0 0 / 100% 100% no-repeat border-box content-box;
}
```

The shorthand now expands to seven records: color, image, size, position, repeat,
origin and clip. Omitted components reset their initial values, including both
new boxes. Both longhands are non-inherited by default; explicit inheritance,
CSS-wide values, `all`, variables and existing layer/revert handling include them.
Stylesheets, supports checks and inline setters share strict validation, including
identifier escapes for the three box keywords. Unsupported ordinary values are
rejected atomically; an invalid substituted winner keeps the existing `unset`
behavior.

A shorthand getter requires all seven inline records with compatible priorities;
computed defaults or stylesheet values do not fill missing records. A mixed-wide
or incomplete block returns an empty string. Serialization omits the default
padding/border pair, emits one equal box, or emits origin followed by clip.
Pending variable shorthands retain their shared identity. This follows the
[CSSOM getter model](https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-getpropertyvalue)
within the supported subset, without claiming complete canonical CSS serialization.

## Principal boxes, tables and the canvas

The outer rounded shape is normalized using its original corner radii. Padding
and content shapes are each derived directly from that outer shape: subtract
border widths for padding, and the combined border-plus-padding insets for
content. Truncated inner radii are not normalized again before deriving content.
The internal shape supports elliptical inner corners; CSS radius syntax remains
its existing scalar subset.

Selected inner clips use `BoxDecoration` even when corners are square. The
background clip never encloses child commands or hit regions. Rounded borders
use an outer-minus-inner ring when the old opaque shortcut cannot preserve the
background. A transparent ring leaves the interior visible; a translucent border
blends over the background beneath it. For a blue 12px square with radius 4px
and a 2px red border at alpha 128, the straight top-edge sample is `#80007f` and
the center stays blue. Safe opaque legacy cases retain their earlier paint path.

Table cells recompute selected geometry and image tiles after final stretching,
using the original available width for percentage insets. A private zero-size
tile may become positive at this stage; a tile still empty at final publication
is omitted. Retile preserves a uniquely owned image payload and its admitted key.
The current table-row model has coincident content, padding and border boxes, so
all nine box combinations select the same row area. Broader table background
layering is outside this change.

Canvas propagation uses the selected background owner's origin/size/position
values on the final root element geometry, including when the body supplies the
background. It retains root-used `currentColor`; `background-clip` does not limit
the canvas painting area. The source element does not paint that propagated
background twice. See the
[canvas rules](https://drafts.csswg.org/css-backgrounds-3/#special-backgrounds).

## Ordinary wrapped inlines

Each owner uses its retained line-fragment geometry. Border, padding and content
widths are summed separately into three joined strips. Image size and phase use
the selected strip; each real fragment shows its corresponding slice. Margins
and a descendant's relative displacement do not enlarge the owner's strip.
The original containing-width basis remains available for percentage padding,
rather than reconstructing insets by subtracting translated endpoints.
Pending decorations also retain their sampling phase through fragment placement
and table alignment. An out-of-range translated origin omits that decoration;
it is not clamped to a different phase. Retained strip widths, prefixes and
fixed-scope behavior survive this refusal.

For two 4px atomic children separated by a forced break, with 3px total inset on
each physical end, the joined border/padding/content widths can be 14/11/8px.
Selecting a different origin therefore changes image phase without changing the
children's positions. Padding, border and corner ends occur only on the physical
first and last fragments. Internal horizontal cuts stay hard; physical endpoints
and vertical clip windows extend outward to preserve the rounded painter's
existing subpixel coverage. This implements a bounded horizontal
[sliced-decoration model](https://drafts.csswg.org/css-break-3/#break-decoration).

Owner height uses the existing nominal ascent `0.95 * font_size` and descent
`0.35 * font_size`, excluding line-height leading. It is not measured glyph
geometry. A font-size 1px owner can therefore paint a 1.3px-high strip spanning
two pixel rows. This deliberately replaces earlier per-child background heights;
retained migrations derive the changed edge pixels from the fixed 4×4 coverage
rule. Nested font sizes, decorated empties and unavailable images retain their
own color/border geometry. Omitted image commands do not change strip phase.

New bidi/RTL or vertical layout, ruby, block-in-inline fragmentation, pagination,
and `box-decoration-break: clone` remain outside this work. Multiple background
layers, `background-clip: text`, attachment modes and `space`/`round` repetition
are also deferred.

## Admission and rendering route

The new box parser retains the 4,096-byte value cap and shared style work.
Seven-record shorthand expansion prepays 448 units plus copied value bytes.
CSSOM adds no new record container or quota: the extra real records share the
existing 256-record and 64 KiB inline limits. Valid direct longhands retain the
existing single-validation storage path.

Principal paint retains five slots, or six with an image, and prepays 16 units
before new shape preparation. The table helper reserves its geometry count plus
16 before retile; a one-owner helper requires 17 units. These are local admission
boundaries, not a transaction over all preceding table layout.

Line planning prepays one additional unit per touched owner before committing
its accumulated f64 strip prefix. The complete geometry reservation succeeds
before that commit, so refused planning or publication does not advance a later
fragment's phase. The retained original width basis is fixed metadata, not a
copy of the owner's style or image.

Each paintable inline owner fragment reserves five balanced slots together:
clip opener, color, image, border and closer. Geometry/prefix retention precedes
this paint decision. Finalization prepays one command-count classification scan,
then `node_count + 8*geometry_count + command_count + 16*pending_slot_count` before
its temporary node table and remaining preparation. Image-key copying keeps its
separate 4,096-byte per-key, 65,536-byte aggregate and key-length-plus-one work
charges. Prefixes and joined strips are finite and bounded to 16,000,000.
These logical charges use the existing positioning/stacking allowance; they are
not measured instruction counts or complete allocator accounting.

The existing `Arc<BoxDecoration>` command, tag 13, mask tariff, source-key pool,
gradient pool and paint/opacity limits remain in use. Primitive, text and Native
bridges refuse a frame containing this command before payload preparation,
including after zoom. Eligible older commands retain their existing Native
routes. This integration adds no shader, GPU lowering, fetch authority or tile
bitmap allocation.

The final x86-64 Rust 1.88 debug browser's DWARF records give these type sizes,
compared with the inline-edges predecessor built on the same host:

| Type | Predecessor bytes | This build bytes |
|---|---:|---:|
| `Canvas` | 232 | 232 |
| `DrawCommand` | 64 | 64 |
| `BoxGeometry` | 64 | 72 |
| `Fragment` | 136 | 136 |
| `InlineItem` | 184 | 176 |
| `InlineKind` | 152 | 152 |
| `PaintOwner` | 16 | 16 |
| `LinePlan` | 48 | 72 |
| `Occurrence` | 32 | 40 |
| table `Cell` | 176 | 184 |

These are emitted structure sizes, not allocator overhead or total live memory.
They record the retained geometry/basis/phase metadata and removal of an unused
inline-item owner field without a larger `DrawCommand` slot.

## Validation record

Independent source fixtures cover all nine
principal origin/clip pairs, rounded insets and rings, canvas propagation, table
retile, seven-longhand mutation and admission cuts. Inline fixtures cover forced
and soft wrapping, nested nominal metrics, physical ends, fractional cuts and
command/key refusal. Direct Page and confined-worker fixtures use a gradient and
an already prepared local PNG through click changes and widths 32→20→32, checking
whole frames, hit regions, DOM identity and cached image identity.

The first focused layout attempt aborted with stack overflow in the existing
deep-input regression. Principal background preparation had placed its command
arrays in recursive `layout_box_sized`. Moving that unchanged preparation body
to a nonrecursive, non-inlined helper reduced the retained x86-64 debug prologue's
stack reservation from 21,608 to 5,320 bytes, without increasing the thread stack,
depth cap or work allowance. This is one observed function-frame measurement,
not a total stack or cross-platform size bound. The deep regression then passed
within a focused run that still had two other failing assertions; that run is
not reported as a complete success.

Source review also found that passing the helper an already-clamped rectangle
could move a default padding tile at the coordinate boundary. An independent
actual-caller regression freezes raw x=-1,000,001 with a 2px left border and the
unchanged tile origin -999,999. The installed correction passes the raw rectangle
and leaves each original output clamp in place. The subsequent source-7 focused
layout run passes all 222 tests, including that boundary regression and the deep
input case. This focused result does not stand in for the final full suites.
The failed run, original source, prologues and additive regression are retained.

The selected source-9 inventory contains 650 inputs and has SHA-256
`9803e8817c0526bb09fb9307bc832855287a6f904470b150ebf0823127920b70`.
Its complete suites pass **2,836 native-feature tests on Rust 1.88** and
**2,598 default-feature tests on Rust 1.98**, each across 35 result groups with
zero failures or ignored tests. Both run with `--include-ignored` and four test
threads. Strict all-target Clippy passes for native builds on Rust 1.88 and 1.98
and default builds on 1.98; the formatting check also passes.

The immutable browser built from that source is 62,573,056 bytes after debug
stripping, with SHA-256
`f5980381ee84542cc217ea4df39dcef82042682cca2f2bcdc74a31db0e069c06`.
Its complete **90 self-authored reference pairs** pass with zero differing pixels
and zero maximum channel error. A separate literal checker verifies nine new
cases against frozen frames, the one explicit nominal-inline reference migration
against authored rectangles, and the other 80 pairs against the inline-edges
predecessor pixels. The record binds all 180 candidate and 160 predecessor PNGs.
The migrated case retains its input HTML and table rectangles; only its inline
edge pixels change as described above.

The [summary](evidence/background-boxes/summary.json) and
[bounded archive](evidence/background-boxes/validation.tar.gz) bind the selected
source, immutable browser, type records, literal inputs and actual results.
Unsuccessful attempts and additive source/oracle corrections remain retained.
No new GPU execution, elapsed-time improvement, complete web compatibility or
production-security result follows from these checks.
