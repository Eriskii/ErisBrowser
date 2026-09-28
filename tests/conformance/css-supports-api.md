# JavaScript CSS.supports subset

The custom runtime exposes `CSS.supports(conditionText)` and
`CSS.supports(property, value)` through the same bounded feature evaluator used
by stylesheets and conditional imports. This is a focused implementation record,
not a complete CSSOM, Web IDL or CSS conformance claim. No third-party JavaScript
engine, native assertion substitute or rewritten upstream fixture is involved.

## Calling and object behavior

`CSS` is a stable ordinary namespace object with `Object.prototype`, the class
string `CSS`, and an enumerable, writable, configurable `supports` method. The
global binding is writable and configurable and has non-enumerable internal
metadata. General JavaScript reflection on `window` remains unsupported by the
runtime; this change does not add Window own-key or descriptor reflection.

The method has name `supports`, length `1`, `Function.prototype`, no own
`prototype` property, and no constructor behavior. Its `name` and `length` are
non-writable, non-enumerable and configurable. Calls do not require a particular
receiver, so an extracted method and `.call(null, ...)` work. Ordinary namespace
property mutation, deletion and extension use the existing object machinery.

Zero arguments throw `TypeError`. One argument selects the condition overload;
two or more select the property/value overload. The chosen arguments undergo
string-hint conversion from left to right, including author getters and
conversion methods. Conversion exceptions propagate, and an invalid property
does not skip conversion of its value. Extra argument expressions are evaluated
by JavaScript but their resulting values are not converted by this API.

CSSOM permits either DOMString or USVString for CSSOMString; this implementation
chooses **USVString**. Paired UTF-16 surrogates retain their scalar value, and lone
surrogates become U+FFFD at the CSS parser boundary. JavaScript string storage
itself remains UTF-16. Symbols, `Symbol.toPrimitive`, the namespace's observable
`Symbol.toStringTag` property, other CSS namespace methods, and worker realms are
not implemented.

## Query behavior

The one-argument form accepts conditions and implied declaration parentheses,
such as `CSS.supports('display:grid')`. The two-argument form compares the
property name literally, using ASCII-insensitive matching for supported CSS
properties, and parses the value independently. It does not trim or CSS-unescape
the property name, and does not turn arbitrary argument text into a condition.
`!important` is permitted by declaration conditions but rejected in the standalone
value overload. Extra declarations, injected condition delimiters and invalid
token splits do not produce positive results.

Both forms inherit the shared evaluator's conservative positive inventory and
limitations, described in [supports.md](supports.md). For example, custom
properties and sticky positioning return false. Token/comment boundaries and
whole-query namespace invalidation use the shared selector parser; a named
namespace does not become supported beneath `not` or `or`. A true result reports
the implemented query subset and does not imply complete layout compatibility.

## Bounds and validation

Author conversions share the existing runtime stack/call, 100,000-work and
8 MiB cumulative allocation limits. Before UTF-8 conversion the binding charges
one work unit per UTF-16 unit plus one, and up to three allocation bytes per unit
plus 24. Strings exceeding the shared 16 KiB query bound return false after the
selected author conversions, provided runtime quotas remain available. The
combined UTF-8 size of the two arguments also cannot exceed 16 KiB.

Before evaluation, scratch storage is conservatively charged at 256 bytes per
actual UTF-8 input byte plus 4096 bytes, covering token-vector growth and bounded
normalization. The evaluator consumes the same remaining runtime work counter;
exhausting it raises an uncatchable resource termination. Existing per-query
grammar/depth/term/selector caps can instead decline a query as false. Temporary
allocations remain charged cumulatively, so repeated queries are bounded too.

Five Rust regression groups cover metadata, extracted calls and namespace
mutation; overload selection, getters, fallback coercion, exceptions and extra
arguments; comment/namespace/injection/property-value syntax and surrogate
conversion; ordinary repeated calls, large input, shared work/heap exhaustion
and recursive coercion; and DOM-visible initial/click-driven results. These are
focused tests, not an unchanged upstream CSSOM/WPT harness run.

## Primary references

- [CSS Conditional Rules: CSS namespace and supports algorithms](https://drafts.csswg.org/css-conditional-3/#the-css-interface)
- [CSSOM: CSSOMString choice](https://drafts.csswg.org/cssom/#cssomstring)
- [Web IDL: overload resolution](https://webidl.spec.whatwg.org/#dfn-overload-resolution-algorithm)
- [Web IDL: operation functions](https://webidl.spec.whatwg.org/#es-operations)
- [Web IDL: namespace objects](https://webidl.spec.whatwg.org/#namespace-object)
- [CSS Conditional Rules 4: recursive selector support](https://drafts.csswg.org/css-conditional-4/#support-definition-ext)
