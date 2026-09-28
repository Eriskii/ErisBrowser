# Grid placement and track sizing

The grid increment implements a bounded, horizontal layout subset. It does not
establish CSS Grid conformance. The implementation was checked against the
[CSS Grid item placement algorithm](https://www.w3.org/TR/css-grid-2/#auto-placement-algo),
[track sizing algorithm](https://www.w3.org/TR/css-grid-2/#algo-track-sizing), and
[alignment rules](https://www.w3.org/TR/css-grid-2/#alignment).

Supported syntax and behavior:

- Numeric `grid-row` / `grid-column` start, end and positive spans; negative lines
  count from the explicit grid. The numeric `grid-area` shorthand is supported.
  Explicit overlaps retain order-modified painting; DOM order is unchanged.
- `grid-auto-flow: row | column` with optional `dense`, definite placement before
  auto-placement, and implicit tracks before and after the explicit grid.
- Explicit rows and columns, integer `repeat()`, cyclic lists in `grid-auto-rows`
  and `grid-auto-columns`, fixed lengths, percentages in definite axes, `auto`,
  `min-content`, `max-content`, and `minmax()` with a non-flexible minimum.
- Fractional track allocation after gaps and fixed tracks, freezing when a
  track's base exceeds its fractional share. Indefinite fractional rows use
  content contributions, independently of the viewport height.
- Content contributions from spanning items, grouped by span, with planned
  increases combined independently of document order. Growth limits set by
  shorter items constrain later spanning items; finite `minmax()` maxima are
  considered before distributing beyond those limits.
- Separate `row-gap` and `column-gap`, including the `gap` shorthand. Cyclic
  percentage row gaps contribute zero while computing intrinsic height and are
  resolved for final positioning, where they can overflow the content height.
- Start/end/center/stretch item alignment, `justify-items`, `justify-self`,
  `align-items`, `align-self`, auto margins, and content alignment including
  space distribution. Padding, borders, box sizing and explicit item min/max
  dimensions are included. A direct item's percentage height resolves against
  its final grid area.
- Rows are measured at final column widths. Unchanged measured fragments are
  reused; a changed used height triggers reflow, preserving clipping and nested
  flex/grid layout. Out-of-flow items do not occupy cells or size tracks.

The seven new pixel pairs use absolute rectangles in their reference pages,
without grid layout. Their dimensions are specified independently from the
implementation:

| Pair | Assertion |
| --- | --- |
| `grid-explicit-spans` | Signed lines, two-axis spans, and gaps |
| `grid-dense-placement` | Earlier holes filled by later single-cell items |
| `grid-column-implicit` | Column flow, row spans, and implicit columns |
| `grid-minmax-fraction` | Minimum freezing and definite fractional rows |
| `grid-intrinsic-spans` | Planned growth from overlapping spans |
| `grid-track-item-alignment` | Content versus item alignment and auto margins |
| `grid-implicit-negative` | Cyclic implicit sizes before and after explicit tracks |

Geometry tests additionally cover sparse placement, row-locked items, `order`,
overlaps and hit testing, box-model bounds, percentages, automatic rows,
indefinite fractional rows, the short-item/spanning-item growth-limit example,
non-rendering and positioned nodes, 32 nested grids, coordinate bounds, and work
exhaustion. The complete pixel suite now contains 26 cases, including all 19
previous cases. These are self-authored regression checks, not imported WPT
results or comparison measurements against a production browser.

Limits and remaining gaps:

- Track lists retain at most 64 tracks. Layout allows at most 256 tracks in each
  axis and 4,096 considered items per grid. Signed explicit line coordinates are
  clamped to `[-128,128]` before implicit-grid offsets; spans are capped at 256.
  A shared budget of 2,000,000 units charges placement scans and track sizing.
  Computed track lists use shared immutable arrays. A per-computation interning
  pool shares equal resolved arrays from explicit declarations as well as
  inheritance. It retains at most 4 MiB of unique track data plus bounded lookup
  keys and metadata; new distinct lists beyond that cap use the shared initial
  value (empty explicit tracks or one implicit auto track). Existing traversal, intrinsic
  text, glyph, command and depth budgets also apply. Whitespace eligibility scans
  consume the shared source-work budget, and reflow refunds only discarded paint,
  never input scanning or tokenization work. Exhausted budgets can produce partial layout. Translated geometry is
  clamped to the engine's finite coordinate range.
- Named lines/areas, `grid-template` / `grid` shorthands, automatic repeat counts,
  `fit-content()`, subgrid, masonry, and `inline-grid` are not implemented.
  Unsupported track-list components invalidate the complete declaration.
- Intrinsic widths use the engine's existing text/box estimator. Full intrinsic
  sizing for nested grids, replaced elements, spanning mixed flexible tracks,
  and CSS automatic minimum-size rules is incomplete. Percentage widths,
  margins and padding in cyclic intrinsic sizing require further work.
- A percentage row track in an indefinite axis is treated as intrinsic; the
  full second sizing pass for cyclic track percentages is not implemented.
  Container min/max block sizes do not yet trigger every required track-sizing
  rerun. Direct item percentage heights are supported, but general descendant
  containing-block definiteness remains incomplete.
- Vertical writing modes, RTL track progression, baseline alignment groups,
  safe/unsafe alignment, aspect-ratio transfer, full anonymous item grouping,
  grid-area containing blocks for absolutely positioned descendants,
  z-index/stacking contexts, pagination and fragmentation remain incomplete.
- Measurements and bounds are implementation limits, not evidence of security
  or a guarantee of Chromium-relative performance.
