# Reflect calls, constructor targets and new.target

The [Date checkpoint](test262-date.md) records **38 passed** on this
unchanged profile. Earlier measurements and prerequisite descriptions below
are retained as historical evidence; the final Date comparison is at the end.

The later [dynamic Function checkpoint](function-constructor.md) updates current
counts and records remaining string-conversion failures. The measurements below
retain this earlier checkpoint's source and policy boundaries.

The later [constructor-policy checkpoint](constructor-policy.md) fixes Symbol as
an alternate constructor target and expands the older feature policies. The
measurements and policy boundaries below describe this original checkpoint.

The custom interpreter now implements `Reflect.apply`, `Reflect.construct`, and
`new.target` in supported ordinary functions, methods, accessors, defaults and
lexically nested arrows. Constructor targets live in private execution state.
Ordinary calls supply `undefined`; arrows inherit their enclosing binding.
Bound construction substitutes its target only when the incoming `newTarget`
is that bound function. Object return values still replace allocated instances.

Reflect validates callable/constructable targets before reading the array-like
argument list. It snapshots and converts `length`, reads indices in ascending
order, propagates abrupt completions, and uses the existing 65,536-argument cap.
`Function.prototype.apply` shares that bounded conversion while retaining its
special handling of null/undefined lists. Reflect methods have stable intrinsic
identity and ordinary writable/configurable, non-enumerable properties.

Alternate targets select the allocation prototype for ordinary constructors and
supported Object, Array, String, Number, Boolean, error and RegExp constructors.
Primitive prototype values fall back to the relevant intrinsic, including the
actual exotic Array prototype. Conversion/prototype ordering is tested for
boxed primitives, arrays and errors. Existing RegExp incompleteness remains;
this change does not establish full RegExp constructor conformance.
Alternate Web IDL constructor targets explicitly report unsupported.

## Upstream coverage

The import retains every direct JavaScript file from the pinned Test262
`built-ins/Reflect/apply` (9), `built-ins/Reflect/construct` (10), and
`language/expressions/new.target` (14) directories at
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`. Source and helper bytes are unchanged.
The two manifests validate the complete inventories and Git blob hashes.

| Profile | Modes | Before | After |
| --- | ---: | --- | --- |
| reflect-construction | 38 | 2 passed, 34 failed, 2 unsupported | 32 passed, 4 failed, 2 unsupported |
| new-target | 28 | 4 passed, 18 failed, 6 unsupported | 20 passed, 8 unsupported |

The **46 additional passes** use the same source, modes, helpers and execution
policy on preserved before/after binaries. Pre-existing parse-negative passes
and tests that already accepted a TypeError are not counted as gains.
At that checkpoint, the four retained failures reached the then-unimplemented
`Date.now` binding. They pass in the final Date comparison below.
Unsupported modes require dynamic Function construction, classes/super,
asynchronous completion, or tagged templates. No failed/unsupported source is
removed. Each profile also runs 80 assertion controls, including paired
success/mismatch checks that require the exact Test262Error failure identity.

The older 22 profiles retain their existing feature policies. In particular,
older profiles may still exclude Reflect.construct metadata even though the new
selection exercises it. This preserves their comparison boundaries; expanding
those policies needs its own inventory review.

## Local coverage and limits

[Independent local cases](construction.js) exercise both language modes: lexical
capture, alternate prototypes, nested bound constructors, argument/getter order,
receiver values, abrupt completion, boxed native slots, intrinsic fallbacks,
metadata/deletion, unary/member syntax, and separation from author properties.
Rust tests additionally check illegal parse contexts, assignment restrictions,
uncatchable resource failures, counter/frame cleanup and storage prepayment.
A bound arrow is rejected as nonconstructable before copying bound arguments,
even when the heap ledger is already exhausted. A failing regression check was
recorded before that ordering fix and passes afterward.

The first local fixture accidentally wrote through an undefined strict-mode
receiver. That fixture was corrected by guarding the write; its original source
and failed run remain in the local evidence. A later valid fallback assertion
exposed use of an ordinary object in place of the exotic Array prototype. The
implementation was fixed and the assertion retained.

[Machine-readable evidence](construction.json) binds the binaries, sources,
corpora, controls and results. Full web/ECMAScript compatibility, independent
security review and the requested Chromium performance comparison remain open.

Algorithm references: [Reflect](https://tc39.es/ecma262/multipage/reflection.html#sec-reflect.construct),
[meta properties](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-meta-properties),
[bound construction](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-bound-function-exotic-objects-construct-argumentslist-newtarget),
and [Object construction](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object-constructor).

### Final Date comparison

Final frozen adapter `ab40a96f2cc1908359186e7c0648cecddc5ad8f8b2adf8a60de0e0d2c9a8f220`
records **38 passed**: **4 new passes and zero lost passes**
against the published before release. Complete observations and controls match
the first frozen candidate; all original source/helper/mode/policy identities
remain unchanged. The existing known-state baseline was strengthened to protect these new passes.
The [complete comparison](test262-date-profiles-comparison.json) and
[baseline receipt](test262-date-baseline-recording.json) retain exact hashes.
Original before and first-candidate reports remain preserved in the
[Date evidence](date.json). No script quota was increased.
