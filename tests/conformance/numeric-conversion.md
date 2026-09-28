# Number and coercing global predicates

Number(value), isFinite(value) and isNaN(value) now use the runtime's ordinary
numeric conversion. An object is read live for valueOf, called with the original
receiver when callable, then read for toString if a primitive has not been
returned. Noncallable properties are skipped. An accessor or callback exception
propagates with its identity and prior effects; two nonprimitive results produce
TypeError. Mutations made by the first hook affect the second lookup.

Number() produces positive zero; Number(undefined) produces NaN. Existing native
construction boxes the converted primitive, including negative zero, through
Number's intrinsic prototype. Bound construction and saved aliases use the same
conversion. The constructor path retains a Boolean boxing decision instead of
cloning its native name.

The global predicates convert their first argument before classification. They
have length 1, the standard names and function metadata, Function.prototype
ancestry, no own prototype, and are not constructors. Their global bindings are
nonenumerable. Number.isFinite and Number.isNaN remain non-coercing predicates.
Replacing the exposed global or function name does not change saved aliases.
General reflection on these global bindings is still unsupported.

These three calls dispatch before the generic native argument-content scan.
They ignore the receiver and extra values after ordinary language evaluation of
all argument expressions. Primitive numeric inputs need constant positive work
and no new runtime heap allocation. Public expression evaluation, argument
vectors, property access, boxing, bind and apply have their existing costs.

The shared numeric conversion path now precharges its literal property keys and
ASCII conversion scratch, including strings returned by hooks. Bounded trimming,
validation, copying and parsing passes consume proportional work. Existing heap,
work and stack ceilings are unchanged; resource termination is uncatchable.
Array-index recognition now decodes at most ten UTF-16 digits without temporary
UTF-8 or decimal strings. It retains canonical spelling and excludes 2^32-1.
Reduction's older conservative per-edge charge remains unchanged.

Eight focused groups cover live hooks and receiver identity; abrupt and argument
order; boxing and aliases; UTF-16 numeric grammar; large ignored extra values at
the heap ceiling; post-getter allocation failure; recursive and looping hooks;
and index boundaries with 30,000 comparisons against the previous decoder's
canonical definition. Page and confined-worker fixtures compare six exact pixel
samples before and after an event callback that uses saved conversion functions.

This scope does not implement Symbol.toPrimitive, Symbol or BigInt conversion,
Reflect/newTarget/subclass semantics, cross-realm behavior, Date, general host
object conversion, Number.parseInt/parseFloat aliases or numeric formatting.
The [complete global inventory](test262-numeric-conversion.md) and existing
[Number inventory](test262-number-statics.md) preserve those outstanding outcomes.

Primary algorithms:
[Number constructor](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-number-constructor-number-value),
[ToNumeric](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-tonumeric),
[OrdinaryToPrimitive](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-ordinarytoprimitive),
[global isFinite](https://tc39.es/ecma262/multipage/global-object.html#sec-isfinite-number),
[global isNaN](https://tc39.es/ecma262/multipage/global-object.html#sec-isnan-number).
