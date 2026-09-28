# Numeric parsing and Number aliases

Global parseInt and parseFloat now convert their first argument using the
ordinary string hint: live toString lookup/call, then valueOf if necessary.
Noncallable methods are skipped; exceptions retain their identity and effects.
The original object is the callback receiver. parseInt completes that string
conversion before converting its radix through ToNumber and ToInt32. Invalid
radices still follow string conversion. parseFloat ignores further values.
Normal call syntax evaluates all argument expressions before these operations.

Number.parseInt and Number.parseFloat initially refer to the same function
objects as their global counterparts. The Number properties are writable,
nonenumerable and configurable; global bindings are nonenumerable. Intrinsic
functions have their standard names and lengths (2 and 1), Function.prototype
ancestry, no own prototype and no constructor behavior. Saved or bound aliases
survive independent global/Number-property replacement and exposed-name changes.
General global-property reflection remains unsupported.

The existing parsing helpers retain longest-prefix behavior, ECMAScript leading
whitespace, signs, optional hexadecimal prefixes and decimal exponent rollback.
Binary/octal string prefixes are not auto-detected by parseInt. Negative zero is
preserved. Decimal inputs use Rust's primitive binary64 parser; power-of-two
radices use the custom guard/sticky-bit converter. Other parseInt radices retain
implementation-approximated multiply/add accumulation; exact mathematical rounding
for those radices is not claimed. No external JavaScript engine is used.

These calls dispatch before generic native argument-content scanning. Shared
string-hint conversion now precharges its literal lookup keys. Only the required
converted input receives parsing work and scratch charges: one work unit per
UTF-16 unit plus fixed work, and 32 + 6 times its length in bytes for worst-case
UTF-8 capacity growth. Charges precede allocation and remain in the shared ledger
after failure. This temporary lossy UTF-8 boundary preserves numeric-prefix
termination: non-ASCII scalars and isolated-surrogate replacement characters
both stop ASCII numeric syntax. No input string is mutated. Invalid radix
returns after both conversions without allocating parsing scratch.

Seven focused runtime groups cover live hooks/radix order, abrupt effects,
metadata/alias identity, grammar/rounding, ignored large values, post-hook scratch
failure and recursive/looping hook termination. A private check verifies that
failed charges are retained; its initial incorrect refund expectation was fixed
without changing runtime accounting. Page and actual confined-worker fixtures
check six exact pixel samples before and after retained aliases run from a click
callback. Shared heap, work and stack limits are unchanged.

The [complete pinned inventory](test262-numeric-parsing.md) retains unrelated
unsupported prerequisites, unchanged harness failures and full-loop resource
stops. Symbol conversion/hooks, BigInt values, general host conversion/reflection,
cross-realm infrastructure and full Number formatting remain separate gaps.

Primary references:
[parseInt](https://tc39.es/ecma262/multipage/global-object.html#sec-parseint-string-radix),
[parseFloat](https://tc39.es/ecma262/multipage/global-object.html#sec-parsefloat-string),
[Number.parseInt](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-number.parseint),
[Number.parseFloat](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-number.parsefloat).
