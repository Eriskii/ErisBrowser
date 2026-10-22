# Plain CSS feature queries

`@supports`, import conditions and `CSS.supports()` now report the existing
primitive `font-size` subset and accept generic escaped property names in
conditions. This changes query validation; font layout, painting and CSSOM
mutation are unchanged. [CSS Conditional Rules Level 3](https://www.w3.org/TR/2024/CRD-css-conditional-3-20240815/#support-definition)
requires an implemented declaration to be usable, not merely stored by a parser.

Primitive font values include nonnegative lengths in the existing supported
units, percentages and unitless zero, plus `initial`, `inherit`, `unset`,
`revert` and `revert-layer`. Examples include `12px`, `12PX`, `1.5em`, `125%`
and `-0`. [CSS Fonts Level 4](https://www.w3.org/TR/2026/WD-css-fonts-4-20261016/#font-size-prop)
defines the nonnegative range and percentages relative to the parent font size.
Eris retains its existing finite-number and magnitude bounds, approximate font
metrics and computed font-size clamp. Named sizes, calculations and other font
math, escaped numeric units, and other omitted ordinary property grammars remain
outside this slice. Existing false negatives for unrelated values remain.

The new font query checks the source mantissa before conversion to `f32`.
A negative nonzero mantissa is refused, and a unitless value must have an all-zero
mantissa. Exponent digits do not change that decision: `-0.000e+9` is admitted,
while `-1e-1000px` and unitless `1e-1000` are refused. Tiny positive dimensions
remain subject to the existing numeric parser. These are selected Eris admission
boundaries. [CSS Values Level 4](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#numeric-types)
allows implementation-defined precision; these controls do not require every
browser to distinguish arbitrarily tiny nonzero values. Existing cascade and
CSSOM floating-point behavior is unchanged, so complete query/render parity at
numeric extremes is not claimed.

Conditions require one complete identifier token for the property name, following
[CSS Syntax's declaration rule](https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-a-declaration).
They decode the name and fold ASCII case while preserving each existing
validator's handling of values and comment boundaries. `(w\69 dth:1px)` and
`(font\2d size:125%)` can therefore succeed. Token-spliced names, extra name
tokens, custom properties and newly inferred aliases are not admitted. Existing
value-escape exceptions and priority handling remain property-specific.

The [two-argument CSS API](https://www.w3.org/TR/2024/CRD-css-conditional-3-20240815/#the-css-interface)
keeps property names literal. In JavaScript, `CSS.supports('(w\\69 dth:1px)')`
is true, while `CSS.supports('w\\69 dth','1px')` is false; the two source
backslashes produce one CSS backslash. The one-argument API and import
`supports(...)` accept implied declarations without outer parentheses. A
trailing condition `!important` is distinct from a separate value argument,
where it is refused. [Variable queries](css-supports-variables.md) keep their
existing 88-longhand, 29-shorthand and 13-alias admission and legacy syntax;
these new plain-query checks do not substitute custom properties.

All existing query, work, token, nesting and Runtime limits remain. The new
font helper prepays `16*v+256` work for the cleaned complete value. Generic
escaped-name preparation pays for tokenization, structural checks and decoding
before allocation. Ordinary grammar refusal remains distinct from exhausted
resources, which cannot become success through `not` or an earlier true `or`.
The private exact cut for `font-size`/`12px` is 455; finishing a private CSS
check at zero does not bypass the Runtime's existing zero-step resource error.

The storage argument is incremental. New preparation tokens must be dropped
before special image, spacing, background or object validators run. Canonical
name ownership replaces their old lowercase-name allocation, and no synthetic
query or value clone is added. Existing validators can retain additional token
vectors; the new preparation/font estimates are not a blanket peak-memory proof
for those paths. The existing proportional scratch allowance remains unchanged.

The source contracts define 20 public groups and five private budget groups.
The latter contain 30 calls with literal exact, one-short, shared-work and raw
value-boundary expectations. Direct and confined worker controls share 48
literal pixels at 8 by 6. Their optional native-plan witness reconstructs pixels
on the CPU and provides no GPU hardware evidence.

On published Q (`64cd81af`), the 20 public groups report three passes and 17
failures across four suites: the first 16 groups report two passes and 14
failures, and the four special-validator groups report one pass and three
failures. A group can stop at its first failed assertion; these totals do not
mean every individual row was reached. Parent Q passed all 11 GitHub CI jobs
in run `39703906403`.

The first candidate passes all 25 focused groups across 66 result rows. Its
first full Rust 1.88 run reports 3,621 passes and one failure: an additional old
escaped `background-clip` condition expected false. Independent review derives
true from the already frozen generic-name design and existing value support.
The supplemental historical migration keeps the exact original literal, group
identity and two-argument false result. Only that old test file changes for
Candidate2; production and all 25 new controls remain unchanged. The failed
full run is retained, and the historical census is not claimed complete before
first execution.

Final Rust 1.88 default and Rust 1.99 `test262-host,vulkan-raster` runs pass
3,622 and 3,987 tests respectively, with zero failures or ignored tests across
67 result rows each. Ignored tests were explicitly included. Both preserve the
prior passing rosters and add exactly 25 groups. Strict Clippy passes in both
configurations; formatting and Rust 1.88 release compilation pass. All 102
existing visual-reference pairs pass, with 204 PNGs byte-identical to Q.

The tested Candidate2 inventory binds 15,335 selected inputs and 11 changed
source paths. The later `tests/conformance/supports.md` prose update is separately
bound because that path occurs in the tested inventory. Executable and test-code
inputs remain as tested; whole-tree equality to every inventory row after the
documentation update is not claimed. See the [validation record](evidence/supports-plain/validation.json)
and [source/check archive](evidence/supports-plain/sources-and-checks.tar.gz).
CI for this checkpoint is pending at publication.

The prior variable-query chapter and [validation entries](VALIDATION.md) retain
their checkpoint totals. This is a bounded query correction, not a complete
WPT inventory, whole-browser compatibility result, security audit, new GPU
measurement or Chromium-relative performance comparison.
