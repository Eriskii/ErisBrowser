# Bounded custom properties and `var()` substitution

This increment corrects the renderer's existing custom-property subset. It is
not a full CSS Variables, CSS Values 5, CSSOM, or WPT conformance claim.
`CSS.supports()` and `@supports` keep their conservative custom-property/`var()`
answers; this change adds no positive capability claims there.

## Implemented behavior

- Custom-property names are case-sensitive identifiers, excluding the reserved
  `--` name. Escaped identifier spellings are decoded consistently for
  declarations and literal-name references.
- Only actual `var()` function tokens trigger substitution. Function spelling is
  ASCII case-insensitive and supports CSS identifier escapes. Quoted strings,
  comment contents, URL tokens, `myvar()`, and `var/**/(` do not become `var()`.
- Declarations containing custom values or substitutions retain raw token input
  through parsing. Declaration separators inside strings, comments, URLs, and
  nested blocks do not split the declaration. Trailing `!important` uses tokens.
- Fallbacks include the complete token sequence after the first argument comma,
  including further commas, nested functions and quoted delimiters. Empty
  custom values and empty fallbacks are valid and distinct from absence or the
  guaranteed-invalid value.
- Each element computes its local custom values before descendants inherit
  them. Redefining a referenced name on a child does not retroactively change
  an inherited alias. A local value that fails during computation removes the
  inherited value of that name. Explicit `inherit`/`unset` retain the parent's
  already-computed value; `initial` removes the value.
- Selected references are resolved with guarded contexts and cached results.
  Every member of an actually traversed cycle becomes invalid, even if a
  fallback inside that cycle would otherwise supply a value. References after
  an earlier failure are still visited so later cycle participants are marked.
  Unselected fallbacks are not evaluated. A property outside a cycle can use a
  fallback for a cyclic property.
- Replacement token boundaries survive substitution and inheritance. For
  example, `--n:25; width:var(--n)px` cannot manufacture a dimension token.
  Internal serialization inserts empty comments where adjacent tokens would
  otherwise merge. Existing property parsers then consume the resolved value.
- Existing importance, layer rollback and pending shorthand handling apply to
  custom declarations and supported ordinary declarations containing variables.
  Case-sensitive names remain intact through shorthand expansion.

## Specification scope

The implementation follows current
[CSS Variables replacement semantics](https://drafts.csswg.org/css-variables-1/#using-variables)
and [CSS Values 5 guarded substitution](https://drafts.csswg.org/css-values-5/#substitution).
The current draft's short-circuiting change is recorded in
[CSS Variables' changes since the 2022 snapshot](https://drafts.csswg.org/css-variables-1/#changes-2022).
In particular, the older 2022 rule that added dependency edges from every
fallback is **not** used here. Token recognition and boundary preservation use
[CSS Syntax tokenization](https://www.w3.org/TR/css-syntax-3/#tokenization)
and its [serialization rules](https://www.w3.org/TR/css-syntax-3/#serialization).

The supported function grammar requires a literal custom-property name followed
by an optional comma/fallback. Dynamic names such as `var(var(--name))`, spread
syntax, other arbitrary substitution functions, registered properties,
`@property`, animation taint and animation/transition dependencies remain
unsupported. Unsupported name grammar is rejected rather than interpreted as
an implemented capability. EOF repair of unterminated strings/functions and
complete CSS preprocessing are not implemented by this value parser.

Custom-property values are internal computed data; exact authored comment
preservation and CSSOM custom-property serialization are not exposed. Final
ordinary-property parsing still uses the renderer's existing limited value
parsers and comment normalization. This increment does not make all ordinary
property grammar or invalid-at-computed-value behavior complete. A
`revert-layer` obtained by substituting **inside a custom property's value**
requires an additional layer-selection pass and currently becomes invalid;
literal custom-property `revert-layer` and supported ordinary-property rollback
continue to work.

## Resource bounds

- At most 128 retained custom names per element; each declared name is at most
  256 bytes and each authored value at most 4 KiB, including raw comments.
- Each expanded value is at most 64 KiB. The shared token cursor permits at
  most 16,384 tokens per invocation. Nested component blocks/fallbacks and the
  active custom-property resolution stack each have a limit of 16.
- Flat lists of substitutions do not consume recursion depth per sibling.
  Token scans, output copies, map preparation and shared cascade work are
  charged before the corresponding operations. Cached values are not rescanned
  after the current output has already become invalid.
- Inherited maps and computed strings use `Arc` sharing. The existing 8 MiB
  aggregate retained-variable budget counts names, values and a conservative
  per-entry charge; a new map that cannot fit is discarded. This is a retention
  counter, not a claim that every allocator metadata byte is measured.
- Declaration parsing keeps the existing 8 MiB source, 4,096 input declaration,
  8,192 expanded declaration and retained declaration-byte limits, and now
  charges token scanning to its bounded parse budget. Cascade computation
  shares the existing 20-million-unit work budget. Budget exhaustion produces
  bounded incomplete styling; it does not authorize continued expansion.

## Verification

Eight focused Rust test groups reproduce the original literal substitution,
uppercase function, truncated comma fallback, inherited alias and token-joining
failures. They cover selected and unselected fallback cycles, multi-property
cycles, references after an earlier failure, empty values, computed keyword
fallbacks, escaped names, comment boundaries, importance, layers and shorthands.
Adversarial cases exercise exponential expansion, deep nesting, 100 flat
substitutions, property-count and retained-byte caps, and shared work exhaustion.
These are self-authored regression tests, not an imported upstream test suite.

Run the focused groups with:

```sh
cargo test --locked --offline --lib css::tests::custom_propert
```
