# Pinned Test262 every and some inventory

This profile retains both complete direct directories at Test262 revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`. Selection does not depend on whether a
test passes. The [manifest](../upstream/test262-array-predicates/manifest.json)
records exact source bytes, hashes and complete pinned Git-tree inventory proofs.

| Directory | Sources | Sloppy modes | Strict modes | Total modes |
| --- | ---: | ---: | ---: | ---: |
| [Array/prototype/every](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/Array/prototype/every) | 218 | 218 | 215 | 433 |
| [Array/prototype/some](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/Array/prototype/some) | 219 | 219 | 215 | 434 |
| Total | **437** | **437** | **430** | **867** |

Seven `noStrict` sources account for the mode difference. There are no
frontmatter negative tests or fixture files. Upstream `assert.js`, `sta.js`,
`compareArray.js`, `propertyHelper.js`, `isConstructor.js`,
`resizableArrayBufferUtils.js` and `testTypedArray.js` remain unchanged, together
with the upstream license and interpretation guide. Manifest SHA-256:
`67cf3fa8c3b006412ee76f9f62e060cd0c938d4c129e8a3ac28d7aa95e2b16a9`.

The primary algorithms are
[Array.prototype.every](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.every)
and [Array.prototype.some](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.some).
The [implementation and frozen local fixtures](array-predicates.md) separately
cover generic receivers, captured length, live inherited/accessor properties,
callback ordering and short-circuit behavior.

## Results and retained dependencies

The [complete evidence](array-predicates.json) preserves before and after
observations, case fingerprints, harness controls and binary provenance.

| Outcome | Before implementation | Final candidate |
| --- | ---: | ---: |
| Passed | 34 | 835 |
| Failed | 817 | 12 |
| Resource limit | 0 | 4 |
| Unsupported by metadata | 16 | 16 |

The before count is **34 raw passing observations**, not a healthy profile:
the missing-method preflights failed. Implementation adds **801 passes**.
Four former failures now reach the work limit; four other former failures now
reach missing `Date`. All other **58 case observations** are identical,
including every previous pass. Sources, modes, expectations, admission policy
and preflight fingerprints are unchanged between measurements.

The **12 failures** are both modes of three sources in each directory:
`15.4.4.16-1-11.js`, `15.4.4.16-5-15.js`, `15.4.4.16-7-c-iii-21.js` and their
`15.4.4.17` counterparts. All report that `Date` is unavailable. They stay
executed failures rather than exclusions.

The **four resource stops** are both modes of `every/15.4.4.16-7-c-ii-2.js` and
`some/15.4.4.17-7-c-ii-2.js`. Each sets index `999999`, then requires scanning
the intervening absent indices without an early return. Sparse storage avoids
dense allocation, but each visited index still consumes the unchanged work
budget. No source is shortened and no budget is raised. These outcomes prevent
a healthy baseline for this profile.

The **16 metadata exclusions** are the four resizable-buffer sources in each
directory, in both modes. They declare unavailable `resizable-arraybuffer`;
the callback-resize pair also declares `TypedArray`. The profile reuses the
existing construction/RegExp feature policy; it introduces no new feature
admission. Proxy, typed-array and general host behavior are not claimed by
ordinary array-like support.

## Harness integrity and replay

All **128 preflights** verify: 32 existing core controls plus 96 variants from
twelve positive/deliberately wrong pairs per method, each in strict and sloppy
mode. Every method control first checks callability and both true and false
successful results, outside any expected-error assertion. The wrong partner
must fail with the unchanged assertion harness's `Test262Error`; a missing
method or unrelated exception cannot count as a verified negative control.
The [shared runner contract](test262.md) still applies, including fresh
runtimes, bounded execution and separate failure/resource/unsupported outcomes.

```sh
python3 tools/import_test262.py --profile array-predicates
python3 tools/test262_conformance.py --profile array-predicates
```

This is an observation report, not a `--baseline` gate. Across the full
**33-profile / 14,304-mode** comparison, four older descriptor metadata modes
also gain passes; the other **13,433 older observations** and all **2,612 older
controls** are identical. All **2,740 combined controls** verify and the
**26 existing healthy gates** are retained. No prior pass is lost. This evidence
does not establish complete ECMAScript or web compatibility.
