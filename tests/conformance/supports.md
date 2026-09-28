# Bounded CSS feature queries

Eris implements a conservative positive subset of CSS `@supports`, including conditional stylesheet imports. These are implementation regressions, not an imported WPT conformance suite. A positive query covers the syntax listed below; it does not assert complete conformance of an entire CSS module.

The normative references are [CSS Conditional Rules 3](https://drafts.csswg.org/css-conditional-3/#at-supports), [Conditional Rules 4 selector queries](https://drafts.csswg.org/css-conditional-4/#the-selector-function), [CSSOM value parsing](https://drafts.csswg.org/cssom/#parse-a-css-value), and [Cascade 5 conditional imports](https://drafts.csswg.org/css-cascade-5/#conditional-import).

## Condition grammar

Supported conditions are parenthesized declaration leaves, `selector(...)`, grouped conditions, `not <parenthesized-or-function-term>`, and homogeneous sequences joined by `and` or `or`. Mixed conjunction/disjunction at the same level, bare declaration conditions, trailing operators, and unmatched delimiters are rejected. Parentheses permit explicit mixed groups. Operators are ASCII case-insensitive and follow CSS identifier/function token boundaries; comments can separate tokens.

Unlike media queries, an unknown but valid feature is false and can be negated: `not (unknown-property:value)` and `not future(foo)` are true. Lexical failures, malformed recognized condition grammar, and condition resource exhaustion invalidate the complete query even in an otherwise true `or` branch or beneath `not`. General-enclosed handling is conservative: recognized nested condition syntax that fails its grammar is rejected instead of being reinterpreted as future syntax.

`css::supports_matches` accepts the strict condition form. It does not expose JavaScript `CSS.supports`. Import `supports(...)` additionally accepts an unparenthesized declaration, so `supports(display:flex)` and `supports((display:flex))` agree. A trailing `!important` is accepted on a declaration leaf and does not alter its support. It is not accepted as arbitrary embedded punctuation.

## Positive declaration inventory

Property names are ASCII case-insensitive. Every property in this table accepts the implemented CSS-wide keywords `initial`, `inherit`, `unset`, `revert`, and `revert-layer`.

| Properties | Positive ordinary values |
| --- | --- |
| `display` | `none`, `block`, `flow-root`, `inline`, `inline-block`, `flex`, `grid` |
| `float`; `clear` | `none`, `left`, `right`; clear also `both` |
| `position` | `static`, `relative`, `absolute`, `fixed` |
| `box-sizing` | `content-box`, `border-box` |
| `flex-direction`; `flex-wrap` | `row`, `row-reverse`, `column`, `column-reverse`; `nowrap`, `wrap`, `wrap-reverse` |
| `width`, `height`, `min-width`, `min-height`, `flex-basis` | Nonnegative simple length/percentage, unitless zero, or `auto` |
| `max-width`, `max-height` | Nonnegative simple length/percentage, unitless zero, or `none` |
| Physical `top/right/bottom/left`, `inset`, `margin`, physical `margin-*` | Signed simple length/percentage, unitless zero, or `auto`; `inset` and `margin` accept one to four values |
| `padding`, physical `padding-*` | Nonnegative simple length/percentage or unitless zero; shorthand accepts one to four values |
| `gap`, `row-gap`, `column-gap` | Nonnegative simple length/percentage, unitless zero, or `normal`; `gap` accepts one or two values |
| `color`, `background-color`, `background` | Current named/hex colors, `transparent`, `currentcolor`, strictly separated numeric `rgb/rgba/hsl/hsla`; `background` accepts only a single color |
| `opacity` | Finite CSS number; ordinary rendering clamps to 0–1 |
| `flex-grow`, `flex-shrink` | Nonnegative finite CSS number |
| `order`; `z-index` | Signed decimal integer representable by `i32`; `z-index` also `auto` |
| `grid-template-columns`, `grid-template-rows` | `none`, simple track lists, `minmax`, one non-nested integer `repeat` per list entry; at most 64 expanded tracks |
| `grid-auto-columns`, `grid-auto-rows` | Nonempty simple track/minmax lists, without `repeat` |
| `grid-column-start/end`, `grid-row-start/end` | `auto`, nonzero integer line, or positive integer `span` in either order accepted by the existing line parser |
| `grid-auto-flow` | `row`, `column`, `dense`, or one direction plus `dense` |

Simple lengths use the renderer's `px`, `em`, `rem`, `ex`, `ch`, viewport variant units, and absolute `pt/pc/in/cm/mm/q` units. Number spelling must consume one complete CSS number token: nonzero unitless lengths, `1.`, whitespace between coefficient/unit, nonfinite values, and coefficients outside ±1,000,000 are rejected. Track breadths add nonnegative `fr`, `auto`, `min-content`, and `max-content`; the minimum of `minmax` cannot be `fr`. Existing render bounds still apply to extreme sizes and line/span values.

Color validation distinguishes legacy comma-separated channels from modern space-separated channels with optional slash alpha. Legacy RGB channels must consistently use numbers or percentages. HSL saturation/lightness require percentages, with numeric/deg/rad/turn hue. Malformed mixtures and duplicate separators do not inherit the renderer color helper's permissive fallback behavior.

Queries intentionally return false for other properties and values, including `position:sticky` (accepted as a stored keyword but without sticky layout), custom-property declarations, `var()`, `calc()`, intrinsic sizing keywords for ordinary dimensions, arbitrary shorthands, background images, advanced colors, and unimplemented display forms. Escaped declaration identifiers/values return false because the current declaration application path does not decode them. Value spelling follows that path: `display`, `box-sizing`, flex direction/wrap, and shorthand `background:currentcolor` require lowercase keyword spelling; property names, float/clear/position/grid keywords, colors, and simple unit spellings are case-insensitive. These conservative false negatives are documented compatibility gaps.

## Positive selector inventory

`selector(...)` validates one complete complex selector without querying a fabricated document. Positive syntax includes unescaped type/universal selectors, class/ID selectors, ordinary attributes (`=`, `~=`, `|=`, `^=`, `$=`, `*=`), and descendant/child/adjacent/general sibling combinators. Attribute values can be an identifier or a quoted string without escapes; flags supported by the matcher are `i`, `I`, and `s`.

Positive pseudo-classes are lowercase `root`, `empty`, `first-child`, `last-child`, `only-child`, their `*-of-type` counterparts, `nth-child`, `nth-last-child`, `nth-of-type`, `nth-last-of-type`, and recursively checked `is`, `where`, and `not`. Nth syntax supports odd/even, integers, and basic An+B, with ordinary ASCII spaces in supported positions. `of <selector-list>` is not supported. All branches of logical selector lists must be supported, even where ordinary selector matching is forgiving.

Top-level selector lists, pseudo-elements, `:has`, `:scope`, interaction/state pseudo-classes, namespace syntax, escaped selector spellings, and other pseudos return false. Undeclared *named* namespace prefixes invalidate the whole condition, including under negation. `@namespace` declarations are not implemented. This deliberately avoids claiming availability for matcher shortcuts or unimplemented pseudo-element painting.

## Imports and bounds

Import grammar is URL/string, optional `layer`/`layer(name)`, optional `supports(...)`, then optional media list. A false or invalid supports condition neither fetches nor establishes a cascade layer, including when the URL was cached elsewhere. True conditions preserve source position and shared anonymous layer identity. A true condition can establish its layer even if fetching fails; media metadata still controls whether that layer and its rules apply after resize. Nested imports inherit the existing layer/media scope. No generated CSS wrappers or response bodies are needed to test capability conditions.

Each condition is limited to 16 KiB, 64 feature terms, 16 levels of token/condition nesting, and 256 KiB of local charged scan work. Stylesheet conditions also consume the existing shared 32 MiB parse-work budget. A selector is limited to 4 KiB, 64 compounds across its recursive logical lists, and 16 recursive levels, using the same work budget. Positive declarations are limited to 256-byte names and 4 KiB values. Import evaluation has a loader-wide 8 MiB work budget, alongside existing 8 MiB decoded/expanded stylesheet accounting, depth 16, and 256 imports/segments. Import condition extraction is bounded before allocating its retained string.

For capability queries, any complete condition combining `selector()` with an actual CSS comment is conservatively declined, including beneath `not` or in an otherwise true `or` branch. This avoids turning adjacent type tokens such as `div/**/span` into a supported descendant selector. Comment-looking bytes inside a quoted string do not trigger this policy. Stylesheet rule scanning now preserves raw supports preludes and skips comments when finding structural delimiters; ordinary selector, declaration, layer, and media consumers retain their earlier normalization boundaries. Direct query, stylesheet block, and import routes share the same policy. This is an explicit false-negative limitation, not full comment-aware selector parsing.

The reused bounded lexical cursor distinguishes unquoted URL tokens from functions and rejects bad URL/string tokens. It is not a complete CSS tokenizer. The existing global comment preprocessor can still alter comment-looking bytes inside unquoted URL tokens, as documented for media queries; future-syntax feature tests using those spellings can return false.

Coverage lives in `css::tests::supports_*` and `stylesheet_loading::tests::supports_*`: valid and malformed groups, negation, declaration boundaries, valid/invalid functional colors, recursive selector validation, condition and shared-budget exhaustion, computed-style effects, no-fetch/no-layer behavior, failed-fetch layer order, nested anonymous imports, and media resize behavior.
