# Float and clear coverage

This increment implements physical `float:left/right/none` and
`clear:left/right/both/none`. It follows the placement and line-exclusion model in
[CSS2 sections 9.4–9.5](https://www.w3.org/TR/CSS22/visuren.html#floats) and the
preferred-minimum/available/preferred-width calculation in
[section 10.3.5](https://www.w3.org/TR/CSS22/visudet.html#float-width).

Float margin boxes shorten overlapping line bands. Same-side and opposite-side
floats pack together or advance to an earlier float's bottom. A fitting float
encountered after inline text moves that text beside it on the same line.
Ordinary nested blocks share the exclusions, and their automatic heights exclude
escaped floats. Floats, inline blocks, flex/grid containers, `flow-root`, and
`overflow:hidden/auto/scroll` isolate their descendants and include internal
floats in automatic height. `overflow:clip` clips deferred float painting and hit
regions without establishing a formatting context, consistent with
[CSS Overflow](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-clip).

The six exact-pixel pairs use independently positioned reference rectangles or
complete text lines; none of the reference pages uses float or clear:

- `float-line-exclusion`: margin exclusion and restoration of full line width.
- `float-opposite-lines`: independent left/right float bottoms.
- `float-clear-margins`: same-side packing, downward placement, side-specific
  clearance, and float painting above a normal block background.
- `float-shrink-to-fit`: preferred versus constrained widths, padding, wrapping,
  and containment in `flow-root`.
- `float-formatting-context`: an independent context beside an outside float.
- `float-text-lines`: two shortened text lines followed by a full-width line.

Geometry tests additionally cover later paragraphs, floating clearance,
non-inherited values, invalid declarations, blockification, percentage margins,
mixed inline baselines, nowrap runs, and clipped hit regions. A quota regression
uses deeply nested clipped float fragments and 30,000 leaf boxes to exhaust
command creation while preserving every closing clip. Float inventory and
pairwise exclusion scans have shared limits; after scan exhaustion, subsequent
content conservatively moves below existing floats.

This is not complete block-formatting-context or CSS conformance. Parent/child
margin collapsing and its clearance interactions remain partial. Preferred
widths and line breaking approximate words split across inline descendants.
The deferred float layer covers normal block backgrounds, but overlapping
negative-margin inline content and positioned stacking do not yet implement
the full CSS painting order. Bidirectional/vertical writing, shapes, pagination,
and general stacking contexts are outside this increment. Resource limits can
truncate very large documents. The tests also share Eris's rasterizer and cannot
prove another browser would produce identical pixels.
