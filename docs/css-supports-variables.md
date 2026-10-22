# Variables in CSS feature queries

`@supports` and `CSS.supports()` admit syntactically valid legacy `var()` values
for the existing implemented ordinary property names. The selected roster has
88 longhands, 29 shorthands and 13 aliases. A query can report support for
`width:var(--missing)` without finding a custom-property value. The final value
is checked after substitution when a declaration is computed; a feature query
does not perform that substitution. This follows the parse-time rule in
[CSS Variables Level 1](https://www.w3.org/TR/2022/CR-css-variables-1-20220616/#using-variables)
and the declaration-support model in
[CSS Conditional Rules Level 3](https://www.w3.org/TR/2024/CRD-css-conditional-3-20240815/#support-definition).

The validator recognizes real, case-insensitive function tokens, including
escaped names and nested functions. Text inside a string or comment does not
create a variable reference. It checks the complete existing variable grammar:
the first argument is a literal custom-property name, and every fallback must
be structurally valid. Empty fallbacks and nested fallback blocks are allowed;
an invalid nested reference is still invalid. Valid pending values can contain
final grammar the renderer does not understand, such as
`calc(var(--x) + )` or `var(--x,visiblePainted)`. Accepting their pending syntax
does not implement those final values.

The two-argument form keeps property names literal. For example,
`CSS.supports('w\\69 dth','var(--x)')` is false, while the escaped name in
`CSS.supports('(w\\69 dth:var(--x))')` is decoded as CSS syntax. These JavaScript
examples contain two source backslashes to pass one CSS backslash. The
one-argument Runtime form also accepts an implied declaration without outer
parentheses. A trailing `!important` is accepted in a declaration-form condition
and rejected as part of the separate value argument. Runtime argument conversion
remains left to right, including conversion of a value for an unknown property;
additional arguments do not acquire extra API conversions. Ordinary thrown
values retain their identity.

The query's ordinary-declaration structure follows
[CSS Syntax Level 3, consume a declaration](https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-a-declaration).
A sole top-level curly block such as `{var(--x)}` is allowed. Mixing that block
with another non-whitespace component, as in `{} var(--x)`, is refused. A block
nested inside a fallback, such as `var(--x,{foo:bar;})`, remains allowed. This is
a query-only structural check; existing declaration substitution is unchanged.

Pending-value parsing exposes invalid syntax separately from exhausted resources.
The existing Option-returning syntax callers still map either failure to `None`.
The supports evaluator uses the shared work allowance: resource exhaustion
cannot turn into success through `not` or an earlier true `or` branch. This does
not make every local cap non-negatable. The 4,096-byte raw pending-value guard
returns ordinary false, as do the retained local syntax/name refusals; a
condition may negate such a result. The 256-byte variable-name grammar boundary
and 1,024-byte identifier-decoder resource boundary remain distinct. Existing
work, token, depth and outer-query caps are not raised. A private CSS check can
finish at exactly zero allowance; the Runtime's existing zero-author-step rule
still raises a resource error.

Unknown ordinary properties and custom-property support queries remain excluded,
even though custom-property declarations themselves work. The newer draft's
broader first-argument substitution grammar, `attr()`/`env()` feature admission
and registration are outside this checkpoint. Existing no-var answers stay
unchanged, including false negatives for some implemented values and escaped
ordinary condition names. A false no-var query such as `font-size:12px` remains
a reporting limitation rather than evidence that the browser cannot lay out
that value. Earlier [validation entries](VALIDATION.md) retain the original
conservative variable-query behavior; the current pointer and visibility
chapters link to this subsequent improvement.

The frozen contracts define 20 test groups. On the published `339b257f` parent,
one of the 14 public groups passes and 13 fail; those observations are retained.
The first candidate passes all 20 focused groups. Full Rust 1.88 default and
Rust 1.99 `test262-host,vulkan-raster` runs pass 3,597 and 3,962 tests respectively,
with ignored tests included, zero failures or ignored tests, and 64 result rows
each. Both preserve the prior passing rosters and add exactly 20 groups. Strict
Clippy passes in both configurations; formatting and release compilation pass.
All 102 existing visual-reference pairs pass, and their 204 PNGs are byte-identical
to the preceding pointer-events checkpoint.

Candidate code, tests and expected outputs needed no correction. A summary
helper initially missed test names with intervening ignored-test attributes;
its census expression was corrected without changing the tests, expectations
or execution commands. The original helper and correction record are retained.
The controls distinguish declaration/query forms, coercion order,
computed-value independence, exact budget cuts and whole-frame pixels. A URL
fallback admission row is not instrumentation proving that no network request
occurred. The direct and confined pixel controls share literal expectations;
their optional native-scene witness reconstructs packed plans on the CPU.

The tested inventory contains 15,328 selected inputs and 11 changed source
paths. The final evidence separately binds the later documentation-only
`tests/conformance/supports.md` update, because that prose path was included in
the tested inventory. Executable and test-code inputs stay as tested; whole-tree
byte equality after documentation updates is not claimed. See the
[validation manifest](evidence/supports-variables/validation.json) and
[source/check archive](evidence/supports-variables/sources-and-checks.tar.gz).
The preceding `339b257f` commit passed all 11 GitHub CI jobs. CI for this
checkpoint is pending at publication.

No GPU hardware result, new benchmark, Chromium-speed comparison, whole-browser
compatibility or security certification is claimed.
