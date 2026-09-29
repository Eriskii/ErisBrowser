# RegExp splitting and species construction

The [Date checkpoint](test262-date.md) records **90 passed, 6 unsupported** on this
unchanged profile. Earlier measurements and prerequisite descriptions below
are retained as historical evidence; the final Date comparison is at the end.

`RegExp.prototype[Symbol.split]` now constructs a separate splitter through the
observable constructor and `Symbol.species` properties. Its intrinsic method has
the standard name, length and property attributes. `RegExp[Symbol.species]` is a
configurable, non-enumerable getter that returns its actual receiver.

The split operation validates its receiver, converts the input string, resolves
species, reads and converts flags, adds sticky matching when needed, constructs
the splitter, and finally converts the limit. Even a zero limit observes the
earlier constructor work. Each match uses the splitter's live `exec` method;
`lastIndex` writes and reads, result length and capture getters remain observable.
Captures are stored without conversion. Limits stop further capture reads, and
the original regexp's `lastIndex` is untouched unless author callbacks modify it.

Empty strings, empty matches and trailing fields follow the protocol. Custom
species splitters support surrogate-pair advancement for `u` and `v` flags;
this does not add Unicode pattern parsing to the regexp engine. A regexp whose
`Symbol.split` is explicitly null or undefined follows literal String splitting.
The previous direct-pattern split implementation is removed.

Flags, argument storage, output substrings and array growth consume the existing
shared resource budget. The output array is private during construction, so
indexed appends define its own values without invoking inherited setters. Tests
exercise recursive conversion/getters/species/exec, hostile capture lengths,
backward-moving custom match indices, exhausted heap refusal and frame cleanup.
No quota, dependency or native process authority changes.

## Frozen upstream comparison

The new `regexp-split` profile imports every direct `.js` file in
`RegExp/prototype/Symbol.split` (44) and `RegExp/Symbol.species` (four) from Test262
revision `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`: **48 sources / 96 modes**.
Its manifest is
`7e380ee0ee12fed435b33bebb97ce784c024e21faee72809e8cda2d97f1e5028`.
The preserved `cd1c016` source archive, release binaries and reports provide the
before measurement. Every source, harness, mode fingerprint and policy stays
identical between measurements.

| Inventory | Before | After |
| --- | --- | --- |
| Complete split/species selection | 2 passed, 88 failed, 6 unsupported | 88 passed, 2 failed, 6 unsupported |
| Frozen local fixture, 34 modes | 8 passed, 24 failed, 2 unsupported | 32 passed, 2 unsupported |

The **86 additional upstream passes** require implementation changes. The two
remaining failures now reach `Date.now`, which is missing. Two cross-realm modes
remain metadata-excluded; four Unicode regexp modes remain explicit parser
unsupported results. Those Unicode tests omit the corresponding metadata feature
and remain in execution rather than being filtered out.

All **8,006 older case objects** and **2,144 older assertion controls** are
identical. The complete inventory reaches **27 profiles / 8,102 modes / 2,216
verified controls**. The new 72 controls include 40 paired method assertions;
wrong exception identities and no-op assertion behavior cannot satisfy them.
Only one new baseline is added; no previous baseline or resource-observation
profile is relaxed.

The [local fixture](regexp-split.js) retains two unsupported modes requiring
indexed accessor definitions directly on Array.prototype. A separate Rust case
checks inherited setter avoidance through Object.prototype, which is supported.
[Machine-readable evidence](regexp-split.json) binds the inputs, observations,
binaries, controls and validation hashes.

## Validation and limits

The later [constructor checkpoint](regexp-constructor.md) addresses Symbol.match
classification and regexp-like source/flags conversion in the default constructor.
This record retains the earlier checkpoint's measured boundaries below.

Rust 1.88 and 1.95 pass strict all-target Clippy and **1,014 default / 1,025
Vulkan-feature tests**, none ignored. Both release builds preserve all **57 CPU
pixel references**. **171 Python tests**, **45,000 deterministic mutations** and
the unchanged upstream HTML baseline pass. No native GPU rendering, independent
agent review or Chromium comparison was performed for this change.

Unicode regexp parsing, realms, Date, other RegExp symbol methods and complete
RegExp constructor conversion remain incomplete. In particular, the existing
default constructor still does not implement all `Symbol.match`-classified
pattern-like objects. Shared limits and these regressions do not establish
complete allocation accounting or audited security. This selection does not
establish full RegExp, ECMAScript or web compatibility.

Algorithm references: [RegExp.prototype[Symbol.split]](https://tc39.es/ecma262/multipage/text-processing.html#sec-regexp.prototype-%symbol.split%),
[SpeciesConstructor](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-speciesconstructor),
[RegExp species getter](https://tc39.es/ecma262/multipage/text-processing.html#sec-get-regexp-%symbol.species%).

### Final Date comparison

Final frozen adapter `ab40a96f2cc1908359186e7c0648cecddc5ad8f8b2adf8a60de0e0d2c9a8f220`
records **90 passed, 6 unsupported**: **2 new passes and zero lost passes**
against the published before release. Complete observations and controls match
the first frozen candidate; all original source/helper/mode/policy identities
remain unchanged. The existing known-state baseline was strengthened to protect these new passes.
The [complete comparison](test262-date-profiles-comparison.json) and
[baseline receipt](test262-date-baseline-recording.json) retain exact hashes.
Original before and first-candidate reports remain preserved in the
[Date evidence](date.json). No script quota was increased.
