# Array lastIndexOf

`Array.prototype.lastIndexOf` now boxes primitive receivers, reads and converts
length once, and searches live properties in descending order. Missing
`fromIndex` starts at the end; explicit undefined starts at zero. Empty inputs
return before converting that argument. Negative positions count from the saved
length, and generic array-like lengths retain the full safe-integer range.

Presence checks skip holes and include inherited properties. Reads invoke getters
with the original object receiver, so earlier getters and position conversion can
change later reads without changing the captured length. Matching uses the
existing strict-equality implementation: object/symbol identity, equal signed
zeros, no NaN match and no author conversion hooks. String comparison work is
charged. The method has its specified name/length/descriptors and is not a
constructor.

The scan allocates no collection from logical length. Each visited index and
prototype edge uses the existing reduction helpers' work/storage charges; getters
and conversion callbacks share the existing call/stack/work/heap budgets. Huge
sparse scans stop at those budgets. Tests cover recursive length/index getters,
length/position conversion, sparse-range exhaustion, uncatchable termination,
heap refusal before index getters and frame cleanup. No quotas or process
authorities change. Host objects without ordinary property support remain
explicitly unsupported.

## Complete pinned inventory

The new `array-last-index-of` profile contains all **198 direct sources / 395
modes** from Test262's `built-ins/Array/prototype/lastIndexOf` at revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`. All source bytes match the complete
pinned GitHub directory listing and Git blob identities. Its manifest is
`e37ad6bb6fa9a5a251b575190d2e7d61f3d5b9afcc552dcb22fcef54c1e03609`.

The preserved `e52d3fa` release records **10 passed / 330 failed / eight resource
stops / 47 unsupported**. The implementation records **338 passed / two failed /
eight resource stops / 47 unsupported**, gaining **328 passes**. Every other new
observation is identical. The two remaining failures require Date. Eight modes
hit the existing array index limit during setup. Thirty-nine reach unsupported
array indexed/length descriptor mutation; metadata excludes two Proxy modes and
six resizable-arraybuffer modes. No source or mode is removed.

Because resource stops remain, this profile is an **observation inventory**, not
a healthy baseline gate. The runner's exit/failure policy is unchanged and refuses
to record a healthy baseline for this result. The full results remain in
[machine-readable evidence](array-last-index-of.json).

The existing String.lastIndexOf selection gains its two remaining modes and now
passes all 50. All other **9,466 older observations and 2,448 older controls** are
identical. Prior policies and case fingerprints are unchanged. The combined
inventory is **31 profiles / 9,863 modes / 2,528 verified controls**. Existing 25
healthy gates remain, with only those two String.lastIndexOf entries promoted.
The new profile's 80 controls include 48 paired assertions: 56 verify before and
all 80 after implementation. Python checks reject no-op assertions and wrong
exception types.

## Validation and remaining work

All **24 unchanged local modes** pass, versus 24 failures in the saved release.
Rust 1.88 and 1.95 pass strict all-target Clippy and **1,033 default / 1,044
Vulkan-feature tests**, none ignored. **183 Python tests**, **15,000 mutation
cases**, unchanged HTML observations and both releases' **57 CPU pixel references**
pass. The source, binaries and result hashes are retained in the evidence.
No independent agent review, GPU exercise or Chromium comparison was performed.

Sparse array storage, indexed/length descriptors, Date, Proxy, resizable buffers,
BigInt and broader language/web compatibility remain incomplete. This is not a
security audit or proof of complete allocation accounting.

Algorithm: [ECMAScript Array.prototype.lastIndexOf](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.lastindexof).
