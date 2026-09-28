# Cascade layer coverage

The renderer implements author cascade layers directly in its CSS parser and
cascade. This is a bounded subset of [CSS Cascade Level 5](https://www.w3.org/TR/css-cascade-5/),
checked against the [current editor's draft](https://drafts.csswg.org/css-cascade-5/)
for layer naming, conditional ordering and important `revert-layer` behavior.
No external CSS parser or browser engine is used.

Run the self-authored CSS regression suite:

```sh
cargo test --locked --offline --lib css::tests
```

The suite currently contains 35 tests, including ten new groups covering layer
behavior and its resource limits. These are targeted implementation tests, not a
complete imported CSS WPT inventory or a web-compatibility percentage.

## Implemented behavior

- Named and anonymous `@layer` blocks, comma-separated order statements, nested
  and dotted names, reopening an existing layer, case-sensitive names, CSS
  identifier escapes, comments between tokens, and EOF-terminated statements.
  `default` is a valid layer name; CSS-wide keywords are reserved. Names and
  at-keywords retain their separate case rules.
- Shared layer order across stylesheet boundaries. Each parent has an implicit
  final sublayer; ordinary declarations favor later layers, while important
  declarations favor earlier layers. Unlayered author rules and inline style
  declarations occupy their specified, distinct positions.
- `revert-layer` removes the current layer and the intervening normal/important
  tiers. Important inline rollback removes element-attached declarations while
  preserving stylesheet important declarations. `revert` returns to the existing
  UA/inherited fallback. Rollback applies independently to supported longhands,
  shorthand expansions, property aliases, `all`, and unregistered custom
  properties. `all` does not reset custom properties. Shorthands with `var()` are
  substituted before their longhand values are interpreted.
- Failed global `@media` and supported `@supports` conditions do not register
  their layers. Element-dependent `@container` groups register nested layer
  order without applying declarations, because container matching is not yet
  implemented. Existing media/supports grammar limits still apply.
- Imported layers use structured `StyleSource` metadata: named identities merge
  under their parent, and one anonymous `Arc<CascadeLayer>` identifies all
  segments belonging to a single anonymous import. Empty sources register a
  supplied layer only when every associated media condition matches. This lets
  the loader preserve failed-import registration without inventing CSS names or
  concatenating stylesheet text.

The tests explicitly check source order versus specificity, normal and important
precedence, nested implicit sublayers, anonymous identity, conditional empty
sources, rollback across tiers, shorthand and variable substitution, global
border keywords, rejected layer handling, and allocation/work limits. Page-level
import and pixel comparisons are separate integration tests; their results do
not expand the scope of this CSS unit suite.

## Bounds and remaining limitations

A computation accepts at most 256 structured source segments and 8 MiB of their
combined source/media text, with at most 32 media conditions per source. Parsing
shares a 32 MiB scanned-text work allowance across those sources. It retains at
most 10,000 style/layer records, 10,000 layer declarations, 64 KiB of authored
layer names, 1,024 distinct non-root layers, and 16 nested layer components.
Decoded layer-name storage also has a separate 64 KiB limit. Layer identifiers
are at most 1,024 bytes and each complete supplied name is at most 4,096 bytes.
Rejected layers are not promoted to unlayered author rules.

Expanded declaration names and values share a 16 MiB stylesheet budget. A direct
or inline declaration-list parse has a 1 MiB budget; each value is limited to
4,096 bytes and each declaration list to 8,192 expanded entries. One element's
cascade retains at most 4,096 layer/importance/property candidates and 1 MiB of
candidate names and values. Selector matching, candidate ordering and variable
substitution share the existing 20-million-unit cascade work allowance. Rules
and declarations beyond these budgets are ignored; these limits deliberately
restrict compatibility for unusually large stylesheets.

Only the renderer's existing properties, value grammars, selector subset and
horizontal logical-property aliases participate. This work does not implement
CSS nesting inside style rules, `@scope`, shadow-tree encapsulation, user
stylesheets, animations/transitions, registered custom properties, container
matching, the complete custom-property dependency algorithm, or CSSOM layer
interfaces. The current draft's `revert-rule` is reserved as a layer name but is
not implemented as a property value. General CSS tokenization and invalid-value
recovery remain partial; passing these layer tests does not establish complete
CSS Syntax, Cascade, or Variables conformance.
