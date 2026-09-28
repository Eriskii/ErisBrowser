# Ordinary equality conversion

The custom interpreter dispatches loose equality by ECMAScript value type.
Same-type values use strict comparison: ordinary objects compare by identity,
strings by UTF-16 units, signed zeros are equal, and NaN is unequal even to itself.
Null and undefined compare loosely equal to each other. Other nullish pairs
remain unequal, so false == null no longer incorrectly succeeds.

For mixed Number/String values, the string receives numeric conversion. A
Boolean reduces to its numeric value and dispatch repeats. A Number or String
paired with an ordinary object invokes live default-hint primitive conversion:
valueOf, then toString if necessary. The original receiver is retained, callable
hooks are looked up when needed, and the first primitive result is used.
A boxed value can equal its primitive; two different boxes remain unequal.
Object/object and object/nullish pairs never read conversion hooks.

Both operand expressions execute before equality conversion. Saved operands
survive later binding replacement; later expressions may replace conversion
hooks before use. Getter/call exceptions preserve exact identity. Inequality
negates only a successful equality result. Strict === and !== preserve their
existing noncoercing behavior. UTF-16 equality neither normalizes text nor merges
isolated surrogates. No new exotic conversion or host-native identity model is
introduced; Date, Symbol.toPrimitive, Symbol, BigInt and HTMLDDA remain gaps.

Abstract coercion steps use a loop, avoiding additional native recursion.
Boolean reduction costs one work step. String content is charged only when
needed: both string scans before equality, or existing numeric-parser work and
scratch storage before String/Number conversion. A large String paired with
null or undefined can return by type without accessing content. When an object
must be converted, that hook runs before result-string work checks. Callback
recursion/looping uses the shared uncatchable resource limits. No ceiling changes.

Seven focused groups cover the primitive/boxed type lattice, live conversion and
saved operands, exact abrupt completion, skipped hooks and identity, UTF-16 and
strict comparisons, work/storage boundaries, and recursive/resource behavior.
A repeated-hook fixture was corrected to mark its redefined getter configurable;
no runtime relaxation was needed. Page and confined-worker fixtures check six
green samples followed by six blue samples after a click.

The [complete pinned inventory](test262-equality.md) retains every source and
unsupported prerequisite across all four equality operators. This increment is
not full equality, ECMAScript or web-platform conformance.

Primary algorithms:
[IsLooselyEqual](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-islooselyequal),
[IsStrictlyEqual](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-isstrictlyequal),
[equality expression evaluation](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-equality-operators-runtime-semantics-evaluation).
