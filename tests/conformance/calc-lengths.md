# Additive length-percentage calculations

This is a bounded implementation of length/percentage sums and differences in
`calc()`, not full CSS mathematical functions or a claim of CSS Values
conformance. The CSS declaration parser, variable-substitution boundary and
inline CSSOM validator share the same typed token grammar. Existing feature
queries remain conservative: `CSS.supports` and `@supports` still return false
for calculation values rather than claiming the entire calculation grammar.

## Grammar and retained values

Accepted operands are supported length dimensions, percentages, signed numeric
dimensions/percentages, nested `calc()` and parenthesized sums. Binary `+` and
`-` require actual whitespace tokens on both sides. Function and unit names are
ASCII case-insensitive and support CSS identifier escapes. Comments do not join
tokens or create whitespace; a hex escape's optional terminating space belongs
to that token. Quotes containing `calc(...)` are ordinary strings. Original math
tokens are validated before legacy property comment normalization, including
after `var()` substitution. A malformed authored math declaration does not hide
an earlier valid declaration; a malformed substituted value takes `unset`.

Supported units are `px`, `pt`, `pc`, `in`, `cm`, `mm`, `q`, `em`, `rem`, `ex`,
`ch`, `vw`, `vh`, `vmin`, `vmax`, `dvw`, `dvh`, `svw`, `svh`, `lvw` and `lvh`.
Existing engine approximations still apply: `ex`/`ch` use half the font size and
small/large/dynamic viewport units use the current viewport dimensions.

A computed `Length::Calc` keeps the pixel coefficient, percentage coefficient
and a separate percentage-presence flag. Thus `calc(10px + 0%)` and
`calc(10px + 25% - 25%)` remain dependent on a percentage basis. Font/viewport
units become pixel coefficients at style computation. Coefficients are summed
in `f64`, then stored in `f32`; individual converted terms are not prematurely
clamped. No calculation tree is retained in every computed style.

The implemented type checks follow the length/percentage part of
[CSS Values 4 calculation syntax](https://drafts.csswg.org/css-values-4/#calc-syntax)
and [type checking](https://drafts.csswg.org/css-values-4/#calc-type-checking).
Unitless zero in `calc()` is a number, so `calc(0 + 1px)` is rejected here.
Products, division, numeric-only calculations, `fr` arithmetic, numeric
constants, `min()`, `max()`, `clamp()` and other mathematical functions remain
unsupported. Ordinary property grammar and shorthand serialization retain the
limits described in [inline CSSOM](cssom-inline.md).

## Resolution contexts

Retained calculations feed the existing width/height, min/max sizes, flex basis,
margin, padding, inset, gap and Grid track consumers. A percentage dependency
requires the appropriate definite basis, even when its coefficient is zero.
Preferred sizes and flex basis in indefinite contexts retain their existing
auto/content behavior; a maximum height alone does not make an automatic height
or flex basis definite. Grid intrinsic measurement treats cyclic percentage
tracks as auto, then resolves the percentage when the used track basis exists.

Intrinsic contributions keep the distinctions in
[CSS Sizing 3 cyclic percentages](https://drafts.csswg.org/css-sizing-3/#cyclic-percentage-contribution):
cyclic preferred/max sizes and percentage-as-zero minimum/edge calculations
are different cases. The absolute part of a zero-percentage calculation is
retained when that rule applies. Replaced-element minimum and maximum
contributions use their separate rules. In
[percentage gaps](https://drafts.csswg.org/css-gaps-1/#gap-percent), Grid uses zero
percentages for intrinsic gap contributions and resolves against the resulting
content size; cyclic Flex gaps keep a zero percentage contribution. Existing
Grid/Flex/inline formatting limitations are not removed by this increment.

Nonnegative sizes, padding, gaps and tracks clamp a calculation after resolving
its arithmetic. A percentage-free negative result can clamp during style
computation; a mixed result waits for its used basis. Margins and offsets retain
negative values. Font-size percentages use the parent font and line-height
percentages use the element font. Existing font-size bounds of 1–512 px and
line-height's 4096 px cap remain. This follows the clamping stages of
[CSS Values 4 range checking](https://drafts.csswg.org/css-values-4/#calc-range),
within the engine's stated numeric limits.

Borders accept percentage-free length calculations only. The existing radius
storage is one scalar: percentage-bearing radius calculations are rejected,
including a cancelled or zero percentage term. Pure length radius calculations
work within the existing single-radius rendering approximation. Legacy plain
percentage-radius behavior is unchanged and is not claimed correct here.

## Bounds and verification

- Each calculation/declaration value is at most 4096 bytes. The shared token
  cursor precharges source scanning and each consumed arithmetic token.
- Math nesting, including nested `calc()` and grouping, is at most 16; numeric
  operands are at most 256 across a declaration's math validation. The 16-level
  limit is below CSS Values 4's required minimum of 32 and remains an explicit
  compatibility limitation.
- Each authored numeric coefficient must be finite with magnitude at most
  1,000,000. Stored coefficients must fit finite `f32`; resolved calculations
  cap at ±1,000,000 px before applicable nonnegative clamping. An absent/nonfinite
  required basis does not silently become zero.
- Declaration validation and computed-property parsing reserve bounded scan
  costs from the existing shared style budget. CSSOM uses the caller's shared
  work budget and reports actual/structural exhaustion as an uncatchable
  resource error in JavaScript; setters commit no partial mutation.
- Immutable Grid arrays remain interned. Keys include both coefficients and the
  percentage flag; retained track storage remains capped at 4 MiB using the
  updated `size_of::<GridTrack>()`, rather than assuming the old enum size.

Focused tests exercise coefficient/dependency retention, absolute-unit
cancellation, escaped tokens, comments and operator whitespace, grouping,
negative clamps, authored/substituted invalidation, Grid keys, CSSOM atomicity
and depth/term/work/source limits. Layout tests independently exercise definite
and indefinite bases, cyclic Grid/Flex gaps, replaced elements, intrinsic
contributions, positioned offsets, signed margins and border-box sizing.
These are self-authored regression checks, not an imported complete CSS Values
conformance suite. Primary drafts above were consulted on 2026-09-28.
