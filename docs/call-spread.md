# Spread arguments in calls and constructors

Ordinary calls and `new` now accept spread arguments, including mixed, multiple,
empty and trailing-comma argument lists. For example, `fn(1, ...items, 2)` and
`new Box(...dimensions)` consume each synchronous iterator before evaluating the
next argument. Calls retain the originally resolved function and receiver;
constructors retain the existing `new.target`, prototype and bound-function rules.
Calls without spread keep their original argument storage and dispatch path.

Iterator acquisition reads `Symbol.iterator`, calls it with the original source,
requires an object, and caches `next` once. Each step calls the cached method with
the iterator as receiver, requires an object, and reads `done` before `value`.
Truthy `done` skips the value getter. Iterator exceptions keep their original
JavaScript values. Argument evaluation precedes the final callability or
constructability check. These spread operations never look up or call `return`
on failure, as required by ArgumentListEvaluation.

The collector admits up to 65,536 arguments, subject to the existing shared work,
heap, call and stack limits. Admission starts at eight slots and doubles to the
cap. Every growth charges the full new logical buffer cumulatively; allocator
spare capacity cannot grant unpaid admission. Failed work, heap or allocation
checks preserve the admitted prefix. At the cap, the next step still reads
`done`; a nonterminal step reads `value` before append refuses. No attempt is
prepaid before it is reached, and no failure refunds a quota.

The selected accounting charges eight units on entry after the ordinary
expression tick, six per reached syntax item before evaluating its operand,
eight per append, and eight plus twice the old length per growth. Iterator
acquisition and each reached attempt have separate eight-unit charges, with
explicit handle-copy charges and existing property/call charges. Fixed property
names use the existing accounted text constructor. Resource and unsupported
errors follow the existing terminal path: later operands, target calls, author
catch/finally/return code and queued jobs do not run. The suspended runtime cannot
resume queued jobs; new work requires a fresh runtime.

Compiled syntax uses flat expression IDs, with distinct call/new parents for
spread lists and one marker per spread operand. Malformed internal markers are
rejected in value and reference contexts. The parser carries spread flags
per argument continuation and retains the existing rest-parameter grammar and
fees. This does not implement array/object spread, classes or `super`, optional
chaining, async iteration or dynamic `eval`. The subsequent
[argument-context correction](call-argument-in.md) admits `in` expressions in
these arguments inside a `for` initializer.

The frozen controls contain 18 public interpreter/parser groups, 26 private
resource/layout groups and two page/worker groups. Parent controls retain three
passes and fifteen failures for missing syntax, plus both page/worker pixel
failures. Private helper tests depend on the new internal API; their complete
composition was not treated as a runnable parent baseline. The independent
existing-type layout control is explicitly identified as preservation coverage.

All 46 groups pass. Complete suites pass **2,999 tests on Rust 1.88** and
**3,353 with `test262-host,vulkan-raster` on Rust 1.99**, each across 50 result
rows with no failed, ignored or filtered tests. Strict Clippy passes for all
targets in both configurations, and formatting passes. No old tests were migrated.
The page and confined-worker checks compare complete literal pixels and callback
ordering; the optional raster bridge is checked without a GPU-device measurement.

Actual x86_64 Linux sizes match the prospective mirrors: expression records stay
48 bytes, parser expression frames 72, parser transitions 224 and outer execution
frames 184. The new argument phase increases the phase from 88 to 96 bytes and
its expression frame from 120 to 128, without changing the outer frame or its
limit. Runtime sizes stay 1,808 bytes by default and 1,872 with Test262 hosting.
Bootstrap remains 99,725 work units and 1,740,043 accounted heap bytes, leaving
275 work units under the unchanged limit. These measurements are ABI-specific.

The release adapter improves the pinned `Promise.all` selection to **174 passed,
14 failed and 8 unsupported across 196 modes**, with all 90 harness controls
passing. The only four changed observations are the strict/sloppy modes of the
two `invoke-resolve-on-*-every-iteration-of-promise.js` sources, which previously
failed at `boundPromiseResolve(...args)`. All other case and harness records are
unchanged. The older 458-mode profile remains 420 passed, 8 failed and 30
unsupported, with all 74 controls passing and no changed case records.

The [evidence record](evidence/call-spread/summary.json) retains selected plans,
frozen tests, parent observations, independent source reviews and execution
receipts. Resource accounting and type-size observations do not establish native
stack safety, physical memory use or Chromium-relative speed. This checkpoint
does not establish full web compatibility or production security.

Specification references: [ArgumentListEvaluation](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-argument-lists-runtime-semantics-argumentlistevaluation),
[EvaluateCall](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-evaluatecall),
and [IteratorStepValue](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-iteratorstepvalue).
