# String.prototype.concat

The custom runtime now provides the canonical `String.prototype.concat` builtin
with name `concat`, length 1, standard property flags and no construction support.
Primitive and boxed strings inherit the same function. Borrowed calls, `apply`
and `bind` use their actual receiver. Deleting or replacing the prototype property
changes subsequent member lookup; an existing reference remains callable.

The method rejects null/undefined receivers, converts its receiver to a string,
and converts each argument in order. These are ordinary ECMAScript string
conversions: `Symbol.toPrimitive` receives the string hint, ordinary conversion
tries `toString` before `valueOf`, and implicit Symbol conversion throws TypeError.
Earlier hooks can change later hooks. An abrupt completion preserves earlier
author effects and prevents later conversions.

Arguments are evaluated before entering the method. Converted strings are
retained as immutable fragments, then copied into one final UTF-16 buffer. This
avoids repeatedly copying a growing prefix. Lone surrogates, NUL and replacement
characters retain their distinct code units. A zero-argument call returns the
converted receiver as a primitive string without another code-unit buffer.

Fragment storage, output storage, Vec-to-Rc conversion and copying work are
charged before allocation. Reservations are fallible; the accumulated string
length is checked after each conversion. The existing limits apply to callbacks,
and resource failures unwind without running author recovery code. No quota
increased. The allocation ledger remains an estimate.

## Evidence and fixture correction

The [complete pinned directory](test262-string-concat.md) contains 22 sources and
44 modes. Forty-two pass; two remain metadata-unsupported because their unchanged
constructor-check helper requires `Reflect.construct`. The new assertion preflight
contains 56 controls, including paired deliberately wrong concat results.

Seven Rust groups cover the original probes, argument evaluation and conversion
order, callback mutations and thrown identity, canonical method identity and
prototype deletion, storage/work refusal, maximum UTF-16 length, and callback
cleanup. The page-facing text setter check also preserves the documented lossy
DOMString boundary; concat itself retains the original UTF-16 units.

The 14 original self-authored sources remain unchanged in
[the fixture](../fixtures/string-concat.tsv) and the [comparison](string-concat.json).
Of their 28 modes, 26 gain passes. Two modes of `metadata-identity` remain failed
because that probe calls the upstream `verifyProperty` helper without requesting
restoration. Checking configurability deletes `String.prototype.concat`, so the
probe's next operation acts on undefined. The initial focused Rust run exposed
this fixture mistake. The regression test retains the original source and checks
both its TypeError and the deleted property. A separate corrected metadata probe
uses `{restore:true}` and passes in both modes. No upstream helper or source was
modified, and the original failures are not counted as conformance passes.

This does not implement all String methods, `Reflect.construct`, Date, Proxy,
iteration, full ECMAScript or full browser compatibility. Security and the
Chromium performance target remain unproven. Vulkan browser integration is still
separate work.

Primary algorithm: [String.prototype.concat](https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.concat).
