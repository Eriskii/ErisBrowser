# String lastIndexOf

`String.prototype.lastIndexOf` now uses the existing ordinary String receiver
conversion, converts its search argument with the string hint, then converts the
position to a number before checking for empty or oversized searches. Missing or
NaN positions search from the end; finite positions truncate and clamp. Matching
uses lossless UTF-16 code units, including unpaired surrogates, and returns the
rightmost eligible index. It neither performs `IsRegExp` nor invokes symbol match
hooks. The native method has the specified length, name and property attributes
and cannot be constructed.

The reverse scan allocates no search buffer and charges worst-case comparison
work before each candidate. It is still a bounded naive search, not a claim of
linear-time matching or Chromium performance. Recursive receiver, search and
position conversions use the existing shared work/heap/stack guards; quota errors
remain uncatchable and unwind frames. No quota or process authority changes.

## Complete pinned inventory

The new `string-last-index-of` profile contains all **25 direct sources / 50
modes** in Test262's `built-ins/String/prototype/lastIndexOf` at revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`. Source bytes match the complete pinned
GitHub directory listing and Git blob identities. Its manifest is
`2ed8995ab41234ccc82eaf827a5c406f7fca62d2f530fc36fbc0b8f1e73224bb`.
No modes are excluded by metadata. The saved `569991c` release records **two
passed / 48 failed**; the implementation records **48 passed / two failed**.
The remaining source (`S15.5.4.8_A1_T12.js`, both modes) actually invokes the
separate missing `Array.lastIndexOf` method and remains an unchanged failure.

The previous match/search profile gains **12 passes**, reaching **316 passed /
four failed / 20 unsupported**. Its four failures still use BigInt literals.
Every other **9,406 older observation** and all **2,368 older assertion controls**
are identical. No old pass is lost, and every old source fingerprint and runner
policy is unchanged. The combined inventory is **30 profiles / 9,468 modes /
2,448 verified controls**. Only the twelve newly passing match/search baseline
entries change; the new profile adds a 25th regression gate. Its 80 controls
include 48 paired assertions. The before release verifies 56; the new release
verifies all 80. Python tests reject no-op assertions and wrong exception types.

## Local validation and fixture correction

The [local fixture](string-last-index-of.js) covers 14 cases in both modes,
including conversion order, abrupt completion, UTF-16, empty searches,
negative zero, non-finite positions and method descriptors. All **28 modes** pass.
The final fixture was also run against the preserved before binary: all 28 fail.

The initial descriptor case omitted `{restore:true}` on its first `verifyProperty`
call. That helper intentionally deletes configurable properties, so the test then
read the deleted method. The original fixture and observations are preserved in
[evidence](string-last-index-of.json): 28 failed before, then 26 passed / two
TypeError failures after implementation. One line was corrected in that case,
and both binaries were rerun. The other 26 case fingerprints are unchanged.
This corrects a local test error; no pinned upstream bytes were modified or cases
removed. The original failing local observations remain recorded.

Rust 1.88 and 1.95 pass strict all-target Clippy and **1,030 default / 1,041
Vulkan-feature tests**, none ignored. Four new Rust tests cover the local fixture,
**12,288 deterministic code-unit search comparisons**, work refusal before
matching, adversarial comparison exhaustion and recursive-conversion cleanup.
**180 Python tests**, **15,000 mutation cases**, unchanged HTML observations and
both builds' **57 CPU pixel references** pass. Machine-readable evidence binds
reports, source and binaries. No GPU exercise, independent agent review or
Chromium comparison was performed.

Array lastIndexOf, BigInt, Unicode regexp grammar, replacement/matchAll protocols
and broader web compatibility remain incomplete. This is not a security audit or
proof of complete allocation accounting.

Algorithm: [ECMAScript String.prototype.lastIndexOf](https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.lastindexof).
