# Local TypedArray views and joined text

This independent source-only corpus selects Number TypedArray `subarray`,
`join`, and the exact existing `Array.prototype.toString` function alias.
It has **23 named bodies / 46 strictness modes** and **six paired controls /
24 control modes**, for **70 separate expected observations**. No JavaScript
parser, engine, compiler, browser or candidate implementation was executed
while authoring these files. These are expectations, not measured results.

The published predecessor is Reflect commit
`a3c0bb1d5167236cdf676247f17aea3720dcb866`. Its adapter is root-bound as
SHA-256 `c21074ad37ed121547e6386dd529c67c6c438a6a0b2a6583d453118c49ef78e8`.
The original smaller subarray/alias proposal and the additive join proposal
remain separate preserved inputs. The selected final scope includes join.

Each case must run in a fresh realm. Use the complete `cases.js` bytes plus:

```js
if (typedArrayViewCases.CASE_NAME() !== true)
  throw new Error("case did not return true");
```

The exact single-line appended bytes are recorded by the matrix contract.
The invocation has no arguments. Sloppy and strict modes both execute the
entire fixture and the selected invocation, with strictness supplied by the
existing adapter mode. Do not wrap only the invocation in a strict function,
rewrite the case body, or silently accept a false/undefined return. The
`source_sha256` is the fixture plus appended guard before adapter mode handling;
the `case_sha256` also binds mode, empty metadata and empty harness list using
the established Test262 fingerprint format.

All cases first establish real view storage, callable methods, a working shared
subview, correct simple join text and an own exact toString alias. Error cases
therefore cannot pass solely because an API is missing. The six standalone
control pairs use the same source prefix within each pair, differing only in
their final asserted literal. A wrong partner is healthy only when it produces
a runtime intrinsic `Error` and its positive partner succeeds in the same
mode. `TypeError`, parse/unsupported/resource errors, timeouts and missing
responses are unhealthy. The adapter's diagnostic Error text does not expose
authored `Error.message`, so message text is not an oracle.

`matrix.json` freezes every case/control identity, source hash, fingerprint and
expected observation. It was built by inert byte/JSON operations, without
importing the project runner or parsing/executing JavaScript. It is independent
of the upstream 103-source / 206-mode profile; the populations must never be
added together as one upstream result. Retain the old TypedArray foundation
and Reflect fixture/report bytes and their original outcomes separately.

Coverage and exclusions are in `inventory.json`; concrete oracle reasoning is
in `design.md` and the primary algorithm links are in `spec-notes.json`.
BigInt, shared buffers, Proxy, foreign realms, subclass syntax and unrelated
TypedArray methods are not asserted here. Ordinary custom/bound species
constructors are covered. Buffer resize/transfer uses implemented public APIs,
not an invented `$262` host hook. Storage fees, exact budget cuts and VM cleanup
belong in separate private implementation tests; this corpus does not guess
their future tariffs.
