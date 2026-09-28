# Number constants and static predicates

The custom runtime exposes all eight Number constants with immutable,
nonenumerable, nonconfigurable data descriptors. EPSILON is binary64 spacing
above 1; MAX_VALUE is the largest finite value; MIN_VALUE is the smallest
positive subnormal (`f64::from_bits(1)`, not the smallest normal value).
MAX_SAFE_INTEGER and MIN_SAFE_INTEGER are ±9,007,199,254,740,991. Existing NaN
and signed infinity properties retain their values and flags.

Number.isFinite, isNaN, isInteger and isSafeInteger inspect only primitive
Number values. Missing arguments and all nonnumbers, including boxed numbers,
return false without conversion or property access. Both signs of zero are
finite, integral and safe. Very large finite binary64 values can be integral
without being safe. Classification uses the actual rounded value, so the
source decimal `4500000000000000.1` is classified as integral.

The methods ignore thisArgument and arguments after the first. Ordinary call
syntax still evaluates every argument first and preserves those effects and
exceptions. Each intrinsic has its standard name, length 1, Function.prototype
ancestry, no own prototype and no constructor behavior. Saved aliases survive
method/global replacement and author changes to their exposed name.

Dispatch precedes the generic native argument-content scan. It borrows the
first Value and uses constant work without allocating a result or inspecting
object/string/array contents. The call path also avoids cloning a dotted native
name merely to replace an ignored receiver. Existing public call, member access,
argument-vector and bind/apply costs still apply; the whole JavaScript expression
is not claimed to allocate nothing.

New bootstrap keys, property entries, native records and names are precharged
before creation, in addition to ordinary property-bag accounting. Runtime calls
share the existing work, heap and stack limits; no ceiling increases or quota
resets were added. Private checks use prebuilt large string/array handles at the
heap ceiling, exact work exhaustion and failed metadata preflight. Resource
termination remains uncatchable.

Ten focused groups cover exact values/bits, descriptor redefinition, strict and
sloppy writes/deletes, floating-point boundaries, noncoercion, aliases, actual
argument order, resource accounting and the unchanged near-limit reduceRight
source. Page and confined-worker fixtures exercise these values and retained
native aliases through an event callback.

The Number constructor and global coercing isFinite/isNaN functions are separate
APIs; their subsequent [ordinary conversion increment](numeric-conversion.md)
is measured separately. Number.parseInt and Number.parseFloat remain missing
aliases in the [complete pinned inventory](test262-number-statics.md). These
increments do not add Symbol, BigInt, Reflect, cross-realm behavior or numeric
formatting.

Primary algorithms and constant definitions:
[Number constructor properties](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-properties-of-the-number-constructor),
[isFinite](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-number.isfinite),
[isInteger](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-number.isinteger),
[isNaN](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-number.isnan),
[isSafeInteger](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-number.issafeinteger).
