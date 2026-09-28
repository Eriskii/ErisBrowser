# Bounded CSS feature queries

Eris implements a conservative positive subset of CSS `@supports`, including conditional stylesheet imports. These are implementation regressions, not an imported WPT conformance suite. A positive query covers the syntax listed below; it does not assert complete conformance of an entire CSS module.

The normative references are [CSS Conditional Rules 3](https://drafts.csswg.org/css-conditional-3/#at-supports), [Conditional Rules 4 selector queries](https://drafts.csswg.org/css-conditional-4/#the-selector-function), [CSSOM value parsing](https://drafts.csswg.org/cssom/#parse-a-css-value), and [Cascade 5 conditional imports](https://drafts.csswg.org/css-cascade-5/#conditional-import).

## Condition grammar

Supported conditions are parenthesized declaration leaves, `selector(...)`, grouped conditions, `not <parenthesized-or-function-term>`, and homogeneous sequences joined by `and` or `or`. Mixed conjunction/disjunction at the same level, bare declaration conditions, trailing operators, and unmatched delimiters are rejected. Parentheses permit explicit mixed groups. Operators are ASCII case-insensitive and follow CSS identifier/function token boundaries; comments can separate tokens.

Unlike media queries, an unknown but valid feature is false and can be negated: `not (unknown-property:value)` and `not future(foo)` are true. Lexical failures, malformed recognized condition grammar, and condition resource exhaustion invalidate the complete query even in an otherwise true `or` branch or beneath `not`. General-enclosed handling is conservative: recognized nested condition syntax that fails its grammar is rejected instead of being reinterpreted as future syntax.

`css::supports_matches` accepts the strict condition form. The JavaScript one-argument `CSS.supports` entry and import `supports(...)` additionally accept an unparenthesized declaration, so `supports(display:flex)` and `supports((display:flex))` agree. A trailing `!important` is accepted on a declaration leaf and does not alter its support. It is not accepted as arbitrary embedded punctuation.

## Two-argument declaration API

The Rust `css::supports_declaration(property, value)` helper implements the bounded capability check used by JavaScript `CSS.supports(property, value)`. The two strings are validated independently and are never concatenated into a condition or declaration. Following [Conditional Rules 3's overload algorithm](https://drafts.csswg.org/css-conditional-3/#dom-css-supports) and [CSSOM value parsing](https://drafts.csswg.org/cssom/#parse-a-css-value), property names are compared literally with ASCII case folding only: leading/trailing whitespace, comments, escapes, delimiters and injected conditions do not name a supported property. Values must consume the supported property's entire standalone value grammar.

`!important` and extra declarations are rejected in this overload. Thus `CSS.supports("display", "flex!important")` is false even though the declaration condition `(display:flex!important)` is accepted. Values may have ordinary surrounding whitespace and comments; comment/token boundaries remain distinct, so `width` with `1/**/px` and `color` with `rgb/**/(1,2,3)` are false. The helper uses the same conservative positive property/value inventory below. Both overloads intentionally decline custom-property declarations and `var()` values because arbitrary custom-property token-list substitution is not implemented faithfully; the existing renderer's bounded string substitution is a partial feature, not a positive claim for the full grammar.

Names are capped at 256 bytes, values at 4 KiB, balanced component nesting at 16, and local charged evaluation work at 256 KiB. The crate-private budgeted entry consumes a supplied shared allowance and returns false for unsupported syntax or exhausted local limits. JavaScript argument conversion, overload selection and runtime quota behavior are covered separately by the script implementation. Full CSS value tokenization, escape decoding and EOF repair remain outside this positive subset.

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

`selector(...)` validates one complete complex selector without querying a fabricated document. Positive syntax includes unescaped type/universal selectors, class/ID selectors, ordinary attributes (`=`, `~=`, `|=`, `^=`, `$=`, `*=`), and descendant/child/adjacent/general sibling combinators. Attribute values can be an identifier or a quoted string without escapes; flags `i` and `s` are ASCII case-insensitive.

Positive pseudo-classes are ASCII case-insensitive `root`, `empty`, `first-child`, `last-child`, `only-child`, their `*-of-type` counterparts, `nth-child`, `nth-last-child`, `nth-of-type`, `nth-last-of-type`, and recursively checked `is`, `where`, and `not`. Nth syntax supports odd/even, signed `i64` integers, and basic An+B using CSS numeric/identifier token boundaries. Comments may separate tokens but cannot turn `2/**/n` into a dimension. Matching uses `i128` arithmetic for the accepted coefficient range. `of <selector-list>` is not supported. All branches of logical selector lists must be supported, even where ordinary selector matching is forgiving. Actual `:is`/`:where` matching discards invalid branches and permits an empty list that matches nothing; capability queries conservatively decline an empty or partially discarded list. `:not` and top-level matching lists are unforgiving.

Top-level selector lists, pseudo-elements, `:has`, `:scope`, interaction/state pseudo-classes, namespace syntax, escaped selector spellings, and other pseudos return false. Undeclared *named* namespace prefixes invalidate the whole condition, including under negation. `@namespace` declarations are not implemented. This deliberately avoids claiming availability for matcher shortcuts or unimplemented pseudo-element painting.

## Imports and bounds

Import grammar is URL/string, optional `layer`/`layer(name)`, optional `supports(...)`, then optional media list. A false or invalid supports condition neither fetches nor establishes a cascade layer, including when the URL was cached elsewhere. True conditions preserve source position and shared anonymous layer identity. A true condition can establish its layer even if fetching fails; media metadata still controls whether that layer and its rules apply after resize. Nested imports inherit the existing layer/media scope. No generated CSS wrappers or response bodies are needed to test capability conditions.

Each condition is limited to 16 KiB, 64 feature terms, 16 levels of token/condition nesting, and 256 KiB of local charged scan work. Stylesheet conditions also consume the existing shared 32 MiB parse-work budget. A selector is limited to 4 KiB, 64 compounds across its recursive logical lists, and 16 recursive levels, using the same work budget. Positive declarations are limited to 256-byte names and 4 KiB values. Import evaluation has a loader-wide 8 MiB work budget, alongside existing 8 MiB decoded/expanded stylesheet accounting, depth 16, and 256 imports/segments. Import condition extraction is bounded before allocating its retained string.

Selectors and feature conditions now use a shared borrowed token cursor. Comments emit no tokens; actual CSS whitespace remains explicit. A bounded selector parser validates grammar and emits a canonical matching string for the existing matcher, specificity calculation and candidate index. This is grammar-derived serialization: `div/**/.x` stays a compound, `div /**/.x` is a descendant selector, and `div/**/span`, `#/**/id` and `:is/**/(...)` cannot acquire valid meanings through deletion or inserted whitespace. Attribute strings retain comment-looking bytes and quoted delimiters; an attribute matcher may contain comments between `~` and `=`, but not real whitespace. Pseudo names and attribute flags normalize independently of case-sensitive type/class/ID/value text. The governing sources are [CSS tokenization](https://drafts.csswg.org/css-syntax-3/#tokenization) and [Selectors grammar](https://drafts.csswg.org/selectors-4/#grammar).

Direct conditions, raw `@supports` preludes and import conditions share the same token handling. A malformed selector is a false feature and may be negated; an undeclared named namespace, malformed condition or resource failure still invalidates the complete query. An EOF comment after a complete direct condition emits no token; inside an unfinished function it cannot supply the missing closing delimiter. Inside a stylesheet, an EOF comment may consume the following rule block, as normal stylesheet scanning dictates.

The cursor distinguishes function tokens, identifier/hash/numeric tokens, real whitespace, strings and URL tokens, but this is not a complete CSS Syntax implementation. Escaped selector names/values, namespace declarations, advanced selector features and full declaration tokenization remain unsupported. Existing media/declaration/layer consumers retain their earlier normalization boundaries; in particular the media comment preprocessor can still alter comment-looking bytes inside unquoted URL tokens. Raw supports conditions now retain those URL contents.

Token storage is capped at 16,384 borrowed spans, and the shared cursor rejects input above 512 KiB. Selector-list compilation permits at most 128 branches, 1,024 compound attempts and 64 nesting levels; each complex selector is limited to 4 KiB of source. DOM query APIs retain their 4 KiB total-input cap, and capability queries retain the tighter limits above. Byte scans and recursive serialization are charged before growth. CSS recompiles public rule values before indexing, charges compilation to the shared 20-million-unit cascade work allowance, and caps retained compiled selector/index records at 8 MiB of accounted storage. No complete stylesheet token array or per-element selector reparsing is introduced.

Coverage lives in `selectors::tests`, `dom::tests::selector_*`, `css::tests::supports_*` and `stylesheet_loading::tests::supports_*`: valid and malformed groups, negation, declaration boundaries, valid/invalid functional colors, recursive selector validation, condition and shared-budget exhaustion, computed-style effects, no-fetch/no-layer behavior, failed-fetch layer order, nested anonymous imports, and media resize behavior.
