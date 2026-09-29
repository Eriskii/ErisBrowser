# RegExp constructor classification and conversion

The constructor now performs `IsRegExp` before identity checks or allocation.
Calling it without `new` can return an object classified through `Symbol.match`
when its constructor is the intrinsic RegExp and flags are undefined. Disabling
`Symbol.match` on an actual regexp suppresses that identity shortcut while still
copying its internal pattern and flags into a new instance.

For regexp-like objects, source and flags getters run before the target's
prototype getter. Their returned values are retained, then converted in source/
flags order after allocation. Explicit flags skip the pattern's flags getter.
Functions and arrays use ordinary string-hint conversion; exact UTF-16 units,
including lone surrogates, remain intact.

The separate abstract `RegExpCreate` path directly converts its pattern, without
constructor classification, source/flags getters or the identity shortcut. It
continues to use the intrinsic constructor even when the global binding changes.
This distinction is covered directly in Rust; complete String match/search
symbol dispatch remains separate work.

Successful compilation and failed parses both retain their work and allocation
charges in the caller's budget. The object and its index property are allocated
before conversion callbacks. Tests cover every work cutoff for representative
valid/invalid parses, exact parser charge transfer on success and failure,
recursive classification/source/flags/prototype/conversion callbacks, repeated
caught syntax errors, heap refusal before conversion, and frame cleanup. No
quotas, dependencies or native process authorities change.

## Complete pinned inventory

The new `regexp-constructor` profile contains all **488 direct sources / 976
modes** in Test262's `built-ins/RegExp` directory at revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`. This includes its pattern-language
tests, not just constructor-specific filenames. Its manifest is
`1a015d223e88fcd113ee1ec1c1272692291e4466c82d8024b50fb6a56cf675da`.
Every upstream byte, mode fingerprint and selection policy stays unchanged
between the preserved `f3a786b` release and the new implementation.

| Inventory | Before | After |
| --- | --- | --- |
| Complete constructor directory | 752 passed, 16 failed, 200 unsupported, 8 resource stops | 768 passed, 200 unsupported, 8 resource stops |
| Local constructor fixture, 36 modes | 2 passed, 34 failed | 36 passed |

All **16 upstream failures become passes**. Every other case observation is
identical, including all eight resource stops. All **8,102 older case objects**
and **2,216 older assertion controls** remain identical. The combined inventory
is **28 profiles / 9,078 modes / 2,288 verified assertion controls**. The new
profile's 72 controls include 40 paired constructor assertions; Python checks
reject no-op assertions and wrong exception identities.

The 200 unsupported modes comprise 158 metadata exclusions for modifiers,
duplicate named groups and realms, plus 34 Unicode-pattern modes, four eval
dependencies, two legacy numeric class escapes and two legacy control escapes.
The eight resource stops are retained in full:

- Four modes build 200 nested capturing/noncapturing groups and exceed the
  current regexp parser's nesting limit. Capturing-group capacity also needs work.
- Two modes exceed the shared regexp work budget while testing XML patterns.
- Two modes exhaust the script budget in a loop over the UTF-16 code-unit space.

This profile is an observation inventory. Its resource stops prevent a healthy
regression baseline; no runner checks or existing baselines are relaxed. Run
`python3 tools/test262_conformance.py --profile regexp-constructor` to reproduce
the report; its resource stops produce a nonzero exit status. Existing CI gates
and the new frozen local Rust regressions remain active.

## Fixture review and validation

All 20 original diagnostic source/mode fingerprints are retained. One added
local case initially returned Object.prototype from the alternate target and
then incorrectly expected public RegExp getters to be inherited. It now returns
RegExp.prototype to test the intended capture-before-prototype behavior. The
original fixture and both initial measurements are retained locally; the corrected
fixture was measured again on both the preserved old binary and the new binary.
The final 36 before/after mode fingerprints are identical. Neither the upstream
sources nor their expected outcomes were changed.

The first Python run exposed two inventory-test assumptions: the historical
identifier policy's list of profiles admitting `u180e`, and a realm test that also
declares the already-supported Reflect feature. Those assertions now account for
the new profile without altering any existing profile policy.

Rust 1.88 and 1.95 pass strict all-target Clippy and **1,017 default / 1,028
Vulkan-feature tests**, none ignored. Both release builds preserve all **57 CPU
pixel references**. **174 Python tests**, **45,000 mutation cases** and the
unchanged HTML baseline pass. The [local cases](regexp-constructor.js) and
[machine-readable evidence](regexp-constructor.json) retain observations and
validation hashes. No independent agent review, GPU exercise or Chromium
comparison was performed.

Unicode regexp parsing, flat bounded group parsing, broader legacy grammar,
realms and remaining String/RegExp protocols remain incomplete. The measurements
do not establish full ECMAScript/web compatibility, complete allocation accounting
or audited security.

Algorithm references: [RegExp constructor](https://tc39.es/ecma262/multipage/text-processing.html#sec-regexp-pattern-flags),
[RegExpCreate](https://tc39.es/ecma262/multipage/text-processing.html#sec-regexpcreate),
[RegExpInitialize](https://tc39.es/ecma262/multipage/text-processing.html#sec-regexpinitialize),
[IsRegExp](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-isregexp).
