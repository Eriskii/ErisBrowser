# String and RegExp match/search protocols

`String.prototype.match` and `search` now look up an object's symbol hook before
converting the receiver. Custom hooks receive that original receiver and use the
pattern as `this`. Primitive pattern arguments skip hook lookup. The fallback
converts the receiver, performs the separate `RegExpCreate` operation, then invokes
the resulting regexp's observable symbol method, including prototype overrides.

`RegExp.prototype[Symbol.match]` and `[Symbol.search]` are now real native methods
with their specified descriptors. They accept generic object receivers and
perform string-hint conversion in observable order. Match reads stringified flags,
uses `RegExpExec`, returns non-global results unchanged, and builds global results
with private own array elements. Empty matches advance over surrogate pairs for
custom `u`/`v` flags; this does not implement Unicode regexp pattern parsing.

Search saves the exact `lastIndex`, distinguishes negative zero, resets when
required, and restores after normal execution before reading the result's raw
`index` property. An abrupt execution propagates without restoration. Callback
arguments and result growth are charged, and native callbacks retain the existing
work/heap/stack guards. Tests exercise recursive hooks/getters/conversion,
unbounded custom results, allocation refusal before the first global exec, and
frame cleanup. No quotas or process authorities change.

## Complete pinned inventory

The new `regexp-match-search` profile includes all **170 direct sources / 340
modes** from four Test262 directories at revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`: String match (51), String search (43),
RegExp Symbol.match (53), and RegExp Symbol.search (23). Source bytes were verified
against complete pinned directory listings and Git blob hashes. Its manifest is
`0a9f5f366e513f35451b0bb02cd0ca71ab5ecd57e397333752c199a603009690`.

The preserved `d7cee45` release records **166 passed / 154 failed / 20 unsupported**;
the implementation records **304 passed / 16 failed / 20 unsupported**, gaining
**138 passes**. Every other new case observation is identical. Twelve remaining
failures require `String.lastIndexOf`; four use BigInt literals without declaring
that feature and remain visible parse failures. Ten modes are excluded by
metadata for duplicate named groups or Unicode sets, and ten encounter unsupported
Unicode regexp syntax at runtime.

All **9,078 older observations and 2,288 older controls** are identical. The full
inventory is now **29 profiles / 9,418 modes / 2,368 verified controls**. Existing
policies and 23 regression baselines remain unchanged. The new profile adds a
24th gate that preserves its current passes without hiding known failures.
Its 80 controls include 48 paired protocol assertions; 56 controls verified before
implementation and all 80 verify afterward. Python checks reject no-op assertions
and wrong exception identities. A historical constructor-policy test now excludes
this later profile from its original 88-mode delta, without altering that policy.

## Local coverage and validation

The [46-mode local fixture](regexp-match-search.js) was frozen before production
changes: **38 failed / six passed / two unsupported** becomes **44 passed / two
unsupported**, with identical fingerprints. All 24 original diagnostic modes are
retained. The Array.prototype indexed-accessor case still stops at that separate
unimplemented API; its source and outcome remain visible. A separate inherited
Object.prototype setter case verifies private result-array creation successfully.

Rust 1.88 and 1.95 pass strict all-target Clippy and **1,026 default / 1,037
Vulkan-feature tests**, none ignored. Both releases preserve **57 CPU pixel
references**. **177 Python tests**, **45,000 mutation cases** and unchanged HTML
observations pass. [Machine-readable evidence](regexp-match-search.json) retains
the comparisons and hashes. No independent agent review, GPU exercise or Chromium
measurement was performed.

Replacement and matchAll symbol protocols, Unicode regexp grammar, Array indexed
descriptors, BigInt and broader web compatibility remain incomplete. This is not
an independent security audit or proof of complete allocation accounting.

Algorithm references: [String match](https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.match),
[String search](https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.search),
[RegExp Symbol.match](https://tc39.es/ecma262/multipage/text-processing.html#sec-regexp.prototype-%symbol.match%),
[RegExp Symbol.search](https://tc39.es/ecma262/multipage/text-processing.html#sec-regexp.prototype-%symbol.search%).
