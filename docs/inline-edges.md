# Inline physical edges and fragment geometry

Ordinary horizontal inlines now include left/right padding, border widths and
signed margins in their flow advance. The first surviving fragment consumes the
left edge; the last consumes the right edge. The closing edge participates in
the last content unit's fit test, so it cannot silently extend an already accepted
line. Nested spans keep their own boundaries, including decorated empty siblings
in DOM order.

For a span containing two 4px atomic children with 2px left and 3px right padding,
the literal wide-line positions are x=2 and x=6; a following 4px sibling begins
at x=13. At a 10px containing width, the literals place the children at (2,0)
and (0,4), with the sibling at (0,8). These examples use 4px line heights and
transparent decorations so their pixels isolate horizontal placement.

Collapsed leading/trailing spaces do not discard the element's physical edges.
Preserved spaces remain inside those edges. An undecorated whitespace-only span
adds no line, while an empty span with nonzero margin, padding or border retains
its own zero-content unit and nominal strut. Forced breaks apply the physical
edges once across the element. Percentage padding and margins use the full
containing width, before the existing float-band calculation narrows a line.
Margins affect signed advance and stay outside decoration rectangles.

Each owner retains separate content, padding and border rectangles for each line.
Their final unions are computed independently. Child relative positioning moves
the child without changing its parent's normal-flow advance or rectangle;
the owner's own relative movement applies to its retained geometry. A zero-inset
relative span containing only an absolutely positioned child keeps a zero-width
line-top anchor, preserving that child's containing block without making it
in-flow content. Float-only owner anchoring remains as in the predecessor.

## Metrics and scope

Owner content uses the engine's existing nominal ascent of `0.95 * font_size`
and descent of `0.35 * font_size`, excluding line-height leading. These are
approximations rather than measured font-outline metrics. Existing line
baseline/descent calculations and atomic vertical alignment remain in place.
Consequently, a decorated empty at font-size 1px/line-height 4px beside a
top-aligned 4px atomic box can produce a 5.7px line: the atomic baseline and empty
strut descent are combined. This checkpoint does not correct mixed-baseline
layout generally.

The separate rectangles prepare later decoration work. Ordinary inline
backgrounds still use the existing per-item painting path, including gradient
and raster fragment continuity. This change does not paint newly retained
padding/border regions through `BoxDecoration`, add `background-origin` or
`background-clip`, or implement decoration cloning. New bidi/RTL, vertical writing,
ruby, and block-in-inline fragmentation are outside this checkpoint.

The relevant standards distinguish [inline formatting and physical edges](https://www.w3.org/TR/CSS22/visuren.html#inline-formatting),
[non-replaced inline content height](https://www.w3.org/TR/CSS22/visudet.html#inline-non-replaced),
and [sliced decoration](https://drafts.csswg.org/css-break-3/#break-decoration).
The stated implementation limits remain narrower than those specifications.

## Bounds and validation

Side vectors retain spans, token owners, relative offsets and temporary line
plans. They use the existing 2,000,000-unit positioning/stacking allowance;
depth 128, 100,000 visits, 500,000 emitted text scalars and 200,000 commands remain
unchanged. Whole classification scans are prepaid separately from initialization.
Each reached line membership costs two units; touched-owner ordering and
position publication are reserved before the line commits. The isolated two-item,
one-owner plan costs 14 units, including its initial scan. A 13-unit budget
refuses before publication and retains seven units. These are logical work
charges, not elapsed time or allocator telemetry.

Line planning distinguishes a successful empty line from refusal. Forced-break,
soft-wrap and final-line callers stop on refusal without advancing that line;
previously completed lines remain. Source review identified and corrected the
initial missing scan charges, a lost positioned-only anchor and a forced-break
caller that treated refusal as an empty line. Their earlier sources and literal
regressions are retained.

The final native build retains the predecessor's emitted type sizes: `DrawCommand`
64 bytes, `Fragment` 136, `InlineKind` 152, `InlineItem` 184 and `Canvas` 232.
These DWARF measurements describe this x86_64 build, not total allocator usage.
The existing deep-layout regression also passes without increasing stack or
resource limits.

The minimum-Rust native suite passes 2,775 tests; the current-Rust default suite
passes 2,537. Both include confined-worker tests and report no failures or ignored
tests. The 16 focused inline groups, unchanged exact-coordinate regression,
three strict Clippy configurations and formatting check pass. All 81 self-authored
reference pairs match exactly: the original 76 retain their predecessor pixels,
and five new pairs match independently frozen whole-frame literals. Those five
input pages all fail against the predecessor. The positioned-only inline witness
also retains its exact predecessor pixels.

The initial full-suite attempts each exposed the same floating-point cancellation
regression in a pre-existing relative-positioning test. Computing the font offset
in local line coordinates before adding the page offset restores its exact
assertion; neither that assertion nor a tolerance was changed. Failed attempts,
source corrections, browser hashes, pixel comparisons and type-size measurements
are preserved in the [validation summary](evidence/inline-edges/summary.json) and
[bounded evidence archive](evidence/inline-edges/validation.tar.gz). These checks
establish this increment's regressions; they do not establish complete web
compatibility, general security, GPU execution or comparative performance.
