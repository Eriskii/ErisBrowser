# Array reversal and Number radix conversion

The custom interpreter implements `Array.prototype.reverse` through ordinary
property operations and nondecimal `Number.prototype.toString` for finite safe
integers. These are bounded additions to the existing runtime, not a claim of
complete Array, Number or exotic-object support.

`reverse` boxes primitive receivers, rejects null and undefined, obtains and
coerces `length` once, and uses ToLength semantics before enforcing the runtime's
65,536-element array-like length limit. It preserves the receiver's identity and
does not write its length. Each mirrored pair checks/reads the lower property
before checking/reading the upper property, then performs the specified ordered
writes and deletions. Inherited properties, holes, accessors, mapped/unmapped
arguments and side effects from earlier operations participate in that order.
Writes and deletions throw on failure even for sloppy callers; mutations already
performed before an exception remain visible. Boxed strings retain their
read-only indexed properties.

Generic ordinary objects, functions, arrays and supported boxed primitives are
accepted. Host-object array-likes, Proxy traps and unsupported array-exotic
descriptor operations remain unsupported. The length cap is checked after
observable length conversion and before pair traversal. Pair work, prototype
walks, temporary index strings and getter/setter calls consume the existing
100,000-step, 8 MiB allocation and weighted stack budgets. Resource termination
cannot be caught by page code; reversal is not transactional.

`Number.prototype.toString` checks the Number brand before converting its radix.
It applies numeric coercion and truncation, requires a radix from 2 through 36,
and validates that radix before formatting NaN or infinities. Number wrappers
use their private primitive value rather than author-defined receiver conversion
methods. Undefined radix selects ten. Decimal formatting retains the existing
Number conversion; other radices support positive/negative safe integers,
positive/negative zero, NaN and infinities. Integer digits use exact bounded
division/remainder and lower-case `a` through `z`.

For nondecimal radices, finite fractions and magnitudes above
9,007,199,254,740,991 produce explicit UnsupportedFeature results. They are not
rounded to an approximate integer string. BigInt and Symbol conversion remain
outside this runtime's supported subset. Radix coercion callbacks share the
ordinary work and stack quotas, and output allocation is charged before copying
the fixed-size digit buffer.

The adjacent function-allocation regression verifies that owned parameter and
name copies consume work and heap before closure allocation, including temporary
function copies during calls. Immutable function bodies remain shared.

Focused regression groups are `array_reverse_` (four), `number_radix_` (two),
`function_parameter_copies_consume_work_and_heap_before_retention`, and
`array_number_methods_execute_unchanged_json_ascii_case`. The last executes the
existing pinned JSON ASCII-escaping source unchanged in both sloppy and strict
modes using the unchanged upstream assertion harness. It adds no new upstream
inventory and changes no corpus policy.

```sh
cargo test --locked --offline --lib script::tests
```

Primary algorithms: [Array.prototype.reverse](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.reverse)
and [Number.prototype.toString](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-number.prototype.tostring).
