# Dynamic ordinary Function construction

`Function(...)`, `new Function(...)`, and supported Reflect/bound variants now
compile ordinary functions in Eris's custom interpreter. Argument conversion
runs in order before syntax checks and prototype lookup. Parameters and the
newline-delimited body use separate token streams and grammar entry points;
comments or delimiters cannot cross their boundary. The existing function early
errors then check strictness, duplicate parameters and lexical conflicts.
Construction does not execute the generated body.

Generated functions use the global lexical/object environment, without capturing
the caller's locals or creating a private `anonymous` binding. Their body controls
strictness independently of the caller. They retain supported defaults, identifier
rest parameters, ordinary construction and lexical `new.target` capture. Alternate
targets select the function's prototype after parsing; a primitive prototype
falls back to the callable intrinsic Function prototype. Ordinary functions and
native constructors now create `length` before `name`.

Compilation shares the active caller's remaining work and cumulative allocation
allowances, including failures. It does not use the top-level execute entry point
or reset its instruction counter. String conversion/copying, tokenization,
continuations, flat code publication and closure creation remain charged. Existing
argument/source/call/stack limits remain in force. The browser's conservative CSP
fallback still disables scripts when a policy is present; this is not a complete
CSP or `unsafe-eval` implementation.

## Pinned coverage

The complete direct inventories of Test262 `built-ins/Function` (179 files),
`Function/length` (13), `Function/internals/Construct` (6), and
`Function/internals/Call` (2) are imported at
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`. The importer checks inventory counts
and Git blob hashes; the runner checks source, helper, metadata and mode hashes.
The Function/prototype subtree is outside this selection.

| Selection | Before | After | Added passes |
| --- | --- | --- | ---: |
| New Function constructor profile, 291 modes | 84 passed, 2 failed, 205 unsupported | 241 passed, 50 unsupported | 157 |
| Existing function syntax profile, 1,131 modes | 515 passed, 616 unsupported | 546 passed, 585 unsupported | 31 |
| Existing Reflect profile, 38 modes | 32 passed, 4 failed, 2 unsupported | 34 passed, 4 failed | 2 |
| Existing string/JSON profile, 652 modes | 558 passed, 94 unsupported | 560 passed, 2 failed, 90 unsupported | 2 |

All other existing case observations are identical. No prior pass is lost across
all **7,522 modes**. All **2,008 old assertion controls** remain identical; the
new profile adds **64 verified controls**, including 32 paired dynamic-function
checks that were unverified on the preserved old binary. All old policies remain
unchanged. Three existing baselines and one new baseline record the reviewed
results; resource-observation profiles remain observations.

The two newly reached string-slice failures are retained: dynamic construction
now succeeds, exposing incorrect function-to-string behavior. The new profile's
50 unsupported modes comprise 47 policy exclusions (caller, realms, classes or
Proxy dependencies) and three runtime `eval` dependencies. None is removed or
counted as a pass. The source/helper bytes are unchanged between runs.

The later [String conversion checkpoint](string-conversion.md) fixes those two
slice failures while retaining this historical before/after record.

[Local cases](function-constructor.js) retain the same 30 mode fingerprints before
and after, moving from unsupported to passed. Rust checks cover every work cutoff
on representative dynamic parses, heap refusals, failed-parse charges, recursive
compilation/coercion, and counter/frame cleanup. A valid rest-parameter case
initially exposed the parser's closing-parenthesis assumption; the implementation
was fixed while the fixture stayed unchanged. One resource-test draft terminated
normally because it constructed from `undefined`; it was corrected to recurse
through an explicit global source string.

[Machine-readable evidence](function-constructor.json) binds the frozen before
commit, binaries, inventories, changes, controls and final validation results.
The CI gate requires preservation of the 241 new-profile passes.

## Remaining work

Function source retention and `Function.prototype.toString` are incomplete.
Dynamic source containing literal unpaired UTF-16 surrogates reports unsupported;
it is never silently replaced or rewritten into context-insensitive escapes.
Escaped surrogate literals and scalar Unicode source use the existing parser.
Destructuring, classes/super, realms, Proxy, generators/async functions, dynamic
eval and broader Annex B behavior remain incomplete. This selection is not full
Function, ECMAScript or web compatibility. No audited-security or Chromium
performance conclusion follows from it.

Algorithm reference: [CreateDynamicFunction](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-createdynamicfunction).
