# Positioned layout and stacking coverage

The six `position-*.html` pairs exercise containing-block geometry, opposing
insets with maximum sizes and auto margins, isolation of real stacking contexts,
auto-z descendants crossing their parent's paint group while retaining its clip,
viewport-fixed geometry, and relative offsets that preserve normal flow. Each
reference expresses the expected colored areas as explicit SVG rectangles; it
does not use the positioned layout or z-index algorithm under test. These are
self-authored regression tests, not imported Web Platform Tests or a claim of
complete positioned-layout conformance.

The implementation follows the relevant algorithms in [CSS 2.2 containing
blocks and positioned sizing](https://www.w3.org/TR/CSS22/visudet.html),
[overflow clipping](https://www.w3.org/TR/CSS22/visufx.html#overflow-clipping),
[CSS 2.2 painting order](https://www.w3.org/TR/CSS22/zindex.html), and
[CSS Positioned Layout](https://www.w3.org/TR/css-position-3/).

Absolute boxes are deferred until normal-flow dimensions are known, then use
the nearest positioned ancestor's padding box, skipping intervening static
boxes. Pixel and percentage insets, content/border box sizing, opposing-inset
auto sizes, min/max constraints, shrink-to-fit widths and auto margins use the
same axis solver. Auto-height containing blocks use their final layout height.
Replaced auto dimensions retain their intrinsic sizing/aspect ratio. Direct
absolute children do not consume flex/grid slots. Ordinary percentage heights
resolve against definite containing heights; relative percentage top/bottom
insets remain unresolved when that height is indefinite.

Relative block offsets preserve their original flow size. Relative inline
ancestors shift painted runs and provide a containing rectangle for positioned
descendants. Fixed boxes use the viewport dimensions, escape ordinary ancestor
clips, and produce explicit fixed-coordinate display-list scopes and hit regions.
Scrolling is applied separately to document and fixed coordinates by the native
compositor. Unit tests check scrolled fixed pixels, clipped hits, and exclusion
of fixed boxes from document scroll height.

Painting groups distinguish real contexts from positioned auto-z, floating,
inline-block and auto-z flex/grid item groups. An item's ordinary descendants
paint together while its qualifying positioned descendants can participate in
the enclosing context. Negative/zero/positive levels, source order, order-modified
flex/grid items, static flex/grid item z-index and opacity context isolation are
covered. A higher child z-index cannot escape a lower real context; descendants
of an auto-z positioned group can participate in the enclosing real context.
The final pass reconstructs clip scopes around reordered commands and sorts hit
regions using the same ranks. Absolute descendants follow the supported
containing-block clip chain, so an intervening static overflow box does not
incorrectly clip them. Fixed scopes restore the caller's viewport clip and keep
their own nested overflow clips local. A mutation regression checks DOM child
order rather than allocation order in the arena.

Limits and remaining gaps:

- Horizontal left-to-right layout only; writing modes and directional
  over-constraint rules are not implemented.
- Static-position estimates do not yet implement every hypothetical inline box,
  flex alignment or grid-area rule when both insets on an axis are auto.
- Inline containing rectangles use the union of measured runs. Full first/last
  inline fragment padding boxes, bidi fragmentation, empty inline padding and
  nested relative-inline fragmentation are incomplete.
- Sticky scroll constraints, transforms, filters, perspective, anchor positioning,
  top-layer/popover painting and generated pseudo-element stacking are absent.
- Opacity now composites whole isolated groups; its surface/work limits and
  remaining compositor gaps are described in [opacity coverage](opacity.md).
- Table-specific positioned/stacking behavior, collapsed-border painting and
  several detailed inline decoration/background phases remain partial.
- Positioned work shares a two-million-operation budget for ancestry, geometry
  records, deferred jobs and stacking reconstruction. Layout retains its
  100,000-visit, 100,000-hit, 500,000-source-character and
  200,000-created-command limits. Text fragments and detached layout fragments
  share the hit allocation budget, matching the native snapshot limit.
  Source work is never refunded when fragments are measured again. Reordering
  may need extra clip transitions; it reserves closure slots and stops safely
  before exceeding 200,000 final commands or 128 combined clip/fixed/opacity scopes.
  Associated hit regions beyond the paint cutoff are removed.

The deep quota regression combines twelve fixed/overflow/opacity pairs with 40,000
positioned leaves. It checks actual truncation, typed scope nesting, closure
balance, command count and hit count. The mutation harness independently rejects
cross-kind scope closures, unclosed scopes and combined nesting beyond 128.

An additional long-text regression creates two detached inline-block fragments
with 120,000 source characters and verifies that their combined hit regions stay
within the native 100,000-hit limit. Geometry tests also cover intrinsic replaced
sizes with opposing insets, definite percentage heights and negative auto-margin
residue.
