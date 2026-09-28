# Media-query implementation and self-authored checks

This is a bounded implementation of part of Media Queries Level 4, not a WPT
conformance result. The tests are ordinary Rust unit tests in `src/css.rs`; no
upstream test inventory or recorded baseline is changed by this increment.

The primary references are the CSSWG [query grammar and evaluation rules](https://drafts.csswg.org/mediaqueries-4/#mq-syntax),
[range comparisons](https://drafts.csswg.org/mediaqueries-4/#mq-range-context),
[condition grouping](https://drafts.csswg.org/mediaqueries-4/#combining), and
[error handling](https://drafts.csswg.org/mediaqueries-4/#error-handling).
URL/function distinctions and bad-token rejection follow CSS Syntax
[ident-like tokens](https://www.w3.org/TR/css-syntax-3/#consume-ident-like-token),
[URL tokens](https://www.w3.org/TR/css-syntax-3/#consume-url-token), and
[`any-value`](https://www.w3.org/TR/css-syntax-3/#typedef-any-value).

## Implemented grammar and evaluation

- Comma-separated queries, media types, `not`/`only` type modifiers, and the
  existing `screen`/`all` environment. Other media types do not match; negating
  an unknown type can match. Reserved type keywords remain invalid.
- Parenthesized conditions, `not`, homogeneous `and` or `or` chains, and nested
  groups. Mixing operators at one level is invalid. A media type permits no
  top-level `or` in its following condition; a nested `or` group is permitted.
- Width and height comparisons using `<`, `<=`, `=`, `>=`, or `>`, with the
  feature on either side. Double ranges require the feature in the middle and
  two comparisons in the same direction, for example `200px < width <= 400px`
  or `400px >= width > 200px`. Equality cannot be a double-range operator.
- Existing `width`/`height` plain, boolean, and `min-`/`max-` forms. Plain and
  range equality compare resolved values exactly; the old 0.01-pixel tolerance
  is removed. Negative lengths are valid comparison values for these features.
- Strict CSS decimal/exponent number tokens, unitless zero, and supported
  length units: `px`, `em`, `rem`, `ex`, `ch`, `vw`, `vh`, `vmin`, `vmax`,
  `svw`, `svh`, `lvw`, `lvh`, `dvw`, `dvh`, `pt`, `pc`, `in`, `cm`, `mm`, `q`.
  Media font metrics are the initial 16-pixel em/rem and 8-pixel ex/ch model;
  authored font sizes do not affect them. Dynamic/small/large viewport units
  currently use the same supplied viewport. Arithmetic uses finite `f64`
  values converted from the renderer's `f32` viewport dimensions.
- Case-insensitive and escaped identifiers for types, logical keywords,
  supported feature names, units, and discrete values.
- Existing discrete features: orientation; light color scheme; reduced-motion
  preference; fine pointer/hover capability; browser display mode. These are
  the renderer's fixed environment values, not OS preference subscriptions.
  Boolean color, hover/any-hover, and pointer/any-pointer remain supported.
- Unknown features/values and balanced general-enclosed future syntax retain
  an unknown truth value. `not unknown` stays unknown; `false and unknown` is
  false and `true or unknown` is true. Unknown becomes non-matching only at the
  query-list boundary. A malformed top-level member does not invalidate other
  comma-separated members. A balanced group that does not match known syntax
  can instead be general-enclosed and therefore unknown, as the grammar allows.
  An unquoted `url()` token cannot serve as a general-enclosed function, while
  quoted URL functions and valid nested URL tokens can. Bad URL/string tokens
  invalidate their query member even inside nested future syntax. URL payload
  punctuation and escaped closing parentheses do not split query lists.

The same evaluator is used by stylesheet `@media` blocks and structured
`StyleSource.media` conditions, including imported layers. Non-matching
conditions do not register their associated layer.

## Explicit limits and missing behavior

Each evaluation is limited to 65,536 source bytes, 64 comma-separated members,
64 leaf terms across the entire list, 16 nested blocks, and 2 MiB of charged
parsing work. Identifier decoding retains the existing 1,024-byte bound.
Conditions inside style computation additionally consume the existing shared
32 MiB parser-work allowance. Structured source metadata retains its existing
32-condition/source and aggregate 8 MiB source/metadata bounds. Exhaustion
fails the evaluation closed, including when an earlier alternative matched.
Every syntactically valid branch is parsed even if its truth value would permit
short-circuit evaluation; an invalid trailing operator cannot be hidden.
Viewport dimensions must be finite and nonnegative. Numeric tokens and resolved
lengths must have absolute magnitude at most 1,000,000.

Unsupported feature families include aspect ratio, resolution, numeric color
depth, device dimensions, and additional media preferences. Percentages,
`calc()` and other math functions, custom media, `var()`, and range comparisons
on discrete features do not evaluate as supported features. The surrounding
CSS parser is not a complete CSS Syntax tokenizer: comments currently become
token separators, so a comment inserted inside a two-character comparison
operator is not supported. Comment-looking text inside an unquoted URL payload
is also changed into a separator by this earlier pass, so valid future syntax
such as `unknown(url(foo/**/bar)) or (color)` currently does not match. The media
URL boundary validator does not replace the surrounding CSS tokenizer.
No media-query serialization or live `matchMedia`
API is added here.

## Verification

Run `cargo test --locked --offline --lib css::tests`.

Eight new focused test groups exercise exact/fractional boundaries, reversed and
double ranges, negative and zero values, mixed units and escaped identifiers,
grouping and media-type restrictions, malformed numeric/operator syntax,
unknown truth tables, general-enclosed functions, work/depth/term/list limits,
URL-token/function distinctions, escaped names and payloads, bad URL/string
rejection and query-list recovery through direct, `@media`, and structured source
conditions, and conditional layer/source ordering. Existing media tests remain, with two
expectations updated because `or` and width range comparisons are now supported.
These tests are self-authored implementation checks, not an independent
cross-browser compatibility measurement.
