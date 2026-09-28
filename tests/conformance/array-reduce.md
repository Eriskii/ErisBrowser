# Array.prototype.reduce and reduceRight

The custom runtime implements both methods through a bounded streaming loop.
They support existing ordinary objects, arrays, functions, arguments objects,
and boxed string/number/boolean receivers. Each method has length 1, its standard
name, stable native identity, ordinary built-in property flags, and no constructor
behavior. Saved aliases and existing call/apply/bind operations use the same
implementation.

Receiver boxing precedes a single length read and ToLength conversion. Callback
validation follows those observable operations, including for empty inputs.
Omitting the initial value differs from supplying undefined. Without an initial
value, the first present property in the selected direction supplies the
accumulator; an entirely absent range throws TypeError. Present undefined values
are retained and passed to callbacks normally.

Traversal captures length, then reads presence and values live in ascending or
descending order. Inherited and nonenumerable indexed properties participate;
true holes are skipped. Separate presence and value traversals preserve ordinary
property/accessor behavior, and getters receive the original boxed object.
Callbacks receive exactly accumulator, value, numeric index and that object,
with undefined as their supplied receiver. Existing actual-callee rules handle
strict/sloppy functions, arrows and bound functions. Extra arguments have no
additional method meaning and are not coerced. Callback results remain values
without implicit conversion.

Getters and callbacks can change future indices, prototypes or length. These
changes affect later property reads within the original range. The methods do
not intrinsically write, delete or resize the receiver. Author exceptions
preserve their identity and all earlier effects; resource exhaustion remains
uncatchable and can occur after a getter effect but before its callback runs.

Logical lengths cover ToLength's full safe-integer range using a u64 cursor.
There is no length-proportional collection or allocation. A bounded 16-digit
stack formatter produces index keys, so a very large sparse object may throw
from an early callback without first traversing its entire range. An enormous
absent range instead reaches the existing shared work limit. This does not
change actual array storage limits or sort's separate collection-size limit.

Every visited index, prototype edge, key allocation and four-argument callback
buffer is charged before the associated operation. The scoped property walker
precharges temporary storage on each edge for the existing indexed-property
machinery, including decimal/UTF-8 conversion and boxed-string code units.
Constant `length` comparisons use exact UTF-16 units without allocating a
temporary string. Prototype traversal retains the existing depth guard; native
callbacks share the 100,000-step, 8 MiB cumulative allocation, call-depth and
weighted-stack limits. No budget resets or refunds occur inside either method.

Host receivers and host prototype nodes reached during a property search return
UnsupportedFeature. An ordinary own property found before such a node still
works. The scoped walker prevents an uncharged generic host lookup fallback.
This does not add Proxy, Symbol, BigInt, typed-array/resizable-buffer behavior,
or indexed/length descriptor definitions on actual Arrays. Ordinary-object
accessor descriptors remain supported. At the reduction checkpoint,
Number.MAX_SAFE_INTEGER and Date were independent missing prerequisites in some
unchanged upstream tests. The later [Number static builtin increment](number-statics.md)
supplies the constant; Date remains unimplemented.

Fourteen focused groups cover descriptors/aliases, call and conversion order,
initial presence, sparse/inherited/undefined values, primitive boxing and UTF-16
units, live mutation, actual callback receivers, abrupt effects, reentrancy,
safe-integer indices and host boundaries. Private tests check exact key/argument
allocation boundaries, preserved getter effects, per-edge scratch, valid-ID
prototype cycles, and uncatchable recursive/long-running cases. The unchanged
5- and 11-element pinned sort stability files also run in both modes. Those
examples do not narrow any upstream inventory or guarantee that larger files
fit existing resource limits.

```sh
cargo test --locked --offline --lib script::tests::array_reduce
```

Primary algorithms: [ECMAScript reduce](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.reduce)
and [reduceRight](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.reduceright).
