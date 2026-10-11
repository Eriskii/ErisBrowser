# Table spacing

`border-spacing` supplies horizontal and vertical gaps to Eris's existing table
layout. The computed style holds an inherited `[horizontal, vertical]` pair in
pixels. One authored value sets both axes; two values retain their order.
[CSS's initial value](https://www.w3.org/TR/CSS22/tables.html#separated-borders)
is `0`, while the [HTML user-agent rule](https://html.spec.whatwg.org/multipage/rendering.html#tables)
gives each table `2px 2px`. An unstyled nested table therefore keeps that HTML default;
`inherit` or `unset` explicitly selects its parent's computed pair.
Font-relative lengths are resolved on the declaring element: inheriting a
computed `20px 40px` pair does not recompute it at the child's font size.

The parser accepts the existing absolute units (`px`, `pt`, `pc`, `in`, `cm`,
`mm`, `q`), font units (`em`, `rem`, `ex`, `ch`), and supported viewport units.
`ex` and `ch` retain the current half-font approximation; dynamic/small/large
viewport unit variants use the current viewport basis. Bounded additive
`calc()` supports sums, differences and nesting. Unitless zero is accepted.
Negative literal lengths, percentages even when canceled inside `calc()`,
nonzero unitless numbers, commas and a third value are rejected. A completed
negative calculation clamps to zero: `calc(1px - 3px)` gives zero. Products,
division, `min()`, `max()` and `clamp()` remain unsupported.

Declarations, custom-property substitution, supports checks and inline CSSOM
share the validator. `style.borderSpacing` and
`style.setProperty("border-spacing", value)` target one longhand. Value and
priority queries, removal, `cssText`, CSS-wide keywords and `all` use the
existing declaration machinery. An invalid setter preserves both previous
axes; exhausted work remains a resource error. `border` shorthand updates do
not reset spacing. CSSOM retains the authored value rather than exposing a
new computed-style API.

Horizontal spacing is subtracted before track distribution and included once
for every internal colspan gap. Vertical spacing participates in rowspan
height and deficit distribution. Outer gaps follow the table content edge,
after its border and padding. For a 20px content width, two columns and
`2px 3px` spacing, the tracks are 7px each; rows of 4px and 6px occupy a 19px
content height. Final span geometry still drives background-layer retiling,
retaining resource keys and the existing paint-admission behavior. Spacing on
a row or cell does not override its owning table's grid.

The table model remains limited:

- Track sizing distributes remaining space over the existing explicit/auto
  tracks; it does not implement full intrinsic min/max table sizing. Oversized
  fixed cells retain the current overflow and row-rectangle policy.
- Row backgrounds currently paint across inter-cell gaps. The focused pixel
  fixtures use transparent rows so table color remains visible there; complete
  separated-background painting is not established.
- Collected captions remain stacked at the content origin before the first
  vertical grid gap. `caption-side` and general caption placement are absent.
- A table without rows returns its caption height without grid gaps; an empty
  row still contributes two outer vertical gaps. This preserves existing
  empty-table behavior rather than establishing full empty-grid conformance.
- `border-collapse`, `table-layout`, `cellspacing` attribute mapping, anonymous
  table construction and broader writing-mode/RTL table behavior are not added.
  The fixtures explicitly set content-box sizing and reset margins, padding
  and borders; broader HTML table defaults are unchanged.

Both components share the existing 4,096-byte value limit, depth-16 and
256-term calculation bounds, and caller work budget. Layout normalizes the
pair once through its existing finite 1,000,000px extent bound. There is no
new per-cell allocation, scan, resource fetch, command type or rendering
budget. On the same Rust 1.88 x86-64 target, DWARF reports `ComputedStyle`
growing from 1,024 to 1,032 bytes; `PaintOwner` (16), `BoxGeometry` (72),
`Fragment` (136), `InlineItem` (176) and `InlineKind` (152) are unchanged.
These type sizes do not measure stack usage or total allocator memory.

Run `./run.sh ./examples/table-spacing.html` for the unequal-axis, zero,
single-value and spanning-cell examples. Four authored 320×240 reference pairs
use independent full-frame RGBA expectations and positioned solid rectangles.
They cover selected fixed-width geometry, not full table conformance.

Validation covers 714 bound source inputs, inventory SHA-256
`a28de672a0bfe7a1cb18f38870748d256d890b8b85b93e12f9139d018b9c44dc`:

- Default and Vulkan-raster suites pass 2,675 and 2,979 tests respectively,
  with zero failures or ignored tests. This adds 27 groups, including literal
  Page pixels and confined-worker CSSOM mutation, resize and hit regions.
- Rust 1.88 and 1.98 strict Clippy checks pass for all Vulkan-raster targets.
- All 98 visual reference pairs pass. Both sides of each of the four new pairs
  match their independently frozen RGBA frame. All 188 old PNGs are byte-for-byte
  identical to the published parent evidence.

The four Page pixel tests first failed against the unchanged parent. An initial
focused run exposed one incorrect CSSOM test expectation: a setter replaces
its own important declaration and clears priority when passed an empty priority.
That test-only correction follows the [CSSOM setter requirements](https://drafts.csswg.org/cssom/#set-a-css-declaration);
the original failure and narrow correction remain in the evidence. Initial
Clippy failures required only a test initializer and shared test-helper import
cleanup. Production implementation bytes were unchanged by these corrections.

[Evidence summary](evidence/table-spacing/summary.json) binds the source,
checks, literals, reviews and [source/check archive](evidence/table-spacing/sources-and-checks.tar.gz).
The frozen browser used a Rust 1.88 development build with debug information
stripped. Native comparisons here use the existing packed-plan CPU witness;
this checkpoint adds no hardware/window validation or Chromium performance
measurement, and does not establish full table conformance.
