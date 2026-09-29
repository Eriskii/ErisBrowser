# String conversion, search and custom splitting

Generic String methods now use the runtime's ordinary string-hint conversion for
their receiver, including functions and arrays with custom or inherited hooks.
Search needles follow the same conversion path. `includes`, `startsWith` and
`endsWith` consult `Symbol.match` through ordinary property lookup before
converting the needle, so getters, abrupt completions and explicit RegExp
reclassification are observable in the required order.

`split` rejects a nullish receiver first, then looks up an object separator's
`Symbol.split`. A callable hook receives the original receiver and limit, with
the separator as `this`; its result is returned unchanged, including `undefined`.
Primitive separators are not boxed for this lookup. Literal splitting converts
the receiver, then the limit, then the separator, including when the limit is zero.
Strings retain exact UTF-16 code units throughout these paths.

Callbacks use the existing shared work, allocation and nesting limits. Hook
argument storage is charged before invocation. Regression tests cover recursive
receiver/needle conversion and Symbol getters/calls, counter/frame cleanup after
exhaustion, and refusal to invoke a split hook when the heap budget is exhausted.
These checks do not establish complete memory accounting or audited security.

## Frozen comparisons

The new `string-search` profile imports every direct `.js` source from the pinned
Test262 `String/prototype/{includes,indexOf,startsWith,endsWith,split}` directories:
**242 sources / 484 modes**, at revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`. The manifest is
`51f7812dbdccfac8179261013188f15a8ba1b965bf0bfb9ebb61e8aff742338b`.
Sources, harnesses, mode fingerprints and selection policy remain identical
between the preserved `707ada1` binary and the final binary.

| Inventory | Before | After |
| --- | --- | --- |
| New String search/split selection | 452 passed, 20 failed, 12 unsupported | 470 passed, 2 failed, 12 unsupported |
| Existing String/JSON selection | 560 passed, 2 failed, 90 unsupported | 562 passed, 90 unsupported |
| Local conversion fixture, 40 modes | 8 passed, 32 failed | 40 passed |
| Local split-hook fixture, 14 modes | 2 passed, 12 failed | 14 passed |

All **7,522 older modes** preserve their fingerprints and previous passes; only
the two existing string-slice failures change to passes. Every other older case
observation is identical. The complete inventory is now **26 profiles / 8,006
modes / 2,144 verified assertion controls**. All 2,072 older controls remain
identical. The new profile has 72 controls: 32 general controls and 40 paired
method checks. Deliberately incorrect assertions must throw the actual
`Test262Error` identity; no-op assertions and wrong exception identities are
rejected by Python regression tests. Existing resource-observation profiles
remain observations, not healthy baseline gates.

The remaining two failed modes are the unchanged
`split/cstm-split-on-bigint-primitive.js` test. It uses BigInt syntax without
declaring the BigInt metadata feature, so its parse failure remains visible.
The 12 unsupported modes comprise four metadata exclusions for BigInt, six
runtime eval dependencies and two unsupported RegExp identity escapes. No
resource, timeout or adapter failures occur in this new selection.

[Conversion cases](string-conversion.js), [split-hook cases](string-split-hooks.js)
and [machine-readable evidence](string-conversion.json) preserve the inputs,
before/after observations and validation hashes. A separate CI baseline retains
all 470 new-profile passes and the entire unchanged inventory.

## Validation and boundaries

Rust 1.88 and 1.95 pass strict all-target Clippy and all **1,011 default / 1,022
Vulkan-feature tests**, with none ignored. Both release configurations preserve
all **57 CPU pixel references**. **168 Python tests** and **45,000 deterministic
mutation cases** pass; upstream HTML case observations are unchanged. No native
GPU rendering, independent-agent review or Chromium comparison was performed
for this change.

The built-in `RegExp.prototype[Symbol.split]` intrinsic and its species/generic
execution algorithm remain missing. Actual regexps without a callable custom
split hook still use the earlier direct regexp path, including an incorrect
fallback when their split hook is explicitly null or undefined. Other String
symbol protocols, broader RegExp parsing, Function source printing, BigInt and
dynamic eval remain incomplete. This checkpoint does not establish full String,
ECMAScript or web compatibility.

Algorithm references: [String.prototype.split](https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.split),
[IsRegExp](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-isregexp),
[ToString](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-tostring).
