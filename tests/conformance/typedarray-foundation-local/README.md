# Independent Number TypedArray foundation fixtures

Load cases.js unchanged and invoke one named `typedArrayFoundationCases` function
per fresh realm. Prepend `"use strict";` before both definitions and invocation
for strict mode. Pass the literal mode argument: false for sloppy, true for
strict. The delete case consumes it. All 27 bodies return true and require
complete/runtime with empty error type/identity; caller must reject a false or
missing return. This source was not parsed by a JavaScript parser or executed.

Controls are standalone source strings, eight positive/wrong pairs × two modes
=32 control modes. Wrong tails test a concrete different semantic expectation,
not an unconditional throw. A wrong control needs the exact runtime Error and a
successful same-mode positive partner. Genuine constructors/index reads and
operation-specific positives precede negatives. Prerequisite TypeErrors,
parse/resource/timeout/crash failures and missing observations are unhealthy.
The common UTF-16 pair is independent of all new globals. Before installation,
4 healthy common control modes and 28 unhealthy feature modes are an unexecuted
prediction only.

Coverage includes ten Number kinds, all constructor branches, direct Float16
ties, raw NaN copying, callback collection/conversion order, reentrant object
record creation, fixed/tracking RAB bounds and transfer, every represented
indexed exotic operation, generic consumers, integrity partial effects,
authentic getters, selected metadata, isView and keys/values/entries including
live bounds and sticky completion. The binding-specific alias byte tests use
the explicitly selected little-endian storage policy. Float NaN numeric tests
do not require an invented canonical NaN payload; same-kind copy tests preserve
one deliberately written payload verbatim.

Later TypedArray methods/static from/of, BigInt/shared buffers, proxies, true
realms, class syntax and host structured clone remain outside this local scope.
We do not require a complete all-standard own-key inventory on the selected
shared prototype. NewTarget callback tests use an existing bound constructor
with an authored prototype getter; the successful Reflect.construct path is
checked before either negative branch. Exact/one-short resource tests need the
concrete implementation ledger and are not guessed here.

Primary algorithm sources consulted during preparation:

- [TypedArray constructors and view operations](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-typedarray)
- [TypedArray exotic operations](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-typedarray-exotic-objects)

`author-controls.py` only serializes the authored strings and hashes them. Its
data-only invocation does not execute the JavaScript. Upstream original bytes,
mode metadata and the import/profile design are held separately under upstream.
