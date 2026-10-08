# TypedArray views and string conversion test preparation

The `typedarray-views` profile retains 103 original Test262 files at revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`, each in sloppy and strict mode.
The 206 modes cover `subarray`, `join`, and `toString`. This preparation records
the published engine before implementation; it is not a passing baseline.

| Selection below `test/built-ins/TypedArray/prototype` | Files |
| --- | ---: |
| Complete `subarray` tree | 67 |
| Complete `join` tree | 32 |
| Complete `toString` tree | 3 |
| Direct sibling `toString.js` | 1 |

The importer authenticates complete Git trees and original blob identities.
All eight original helpers and both legal files are retained unchanged. Twelve
original Git responses prove the selection; the existing 32-document proof
limit is unchanged. The direct prototype tree is authenticated in full before
selecting the sibling test, which also exists in the foundation profile.

The execution policy schedules 110 modes and explicitly excludes 96: 74 depend
on BigInt, ten require a detach host hook, and twelve require the complete
`resizableArrayBufferUtils.js` helper. That helper contains unimplemented class
and BigInt dependencies. The last exclusion is restricted to this new profile;
the source and helper remain present. A retained source review corrected the
initial assumption that the existing host policy would already exclude it.
Other failures and resource stops remain actual observations.

The [before report](../tests/conformance/test262-typedarray-views-before.json)
uses the frozen adapter validated for published commit `a3c0bb1d`: ten passed,
88 failed, 96 unsupported and twelve resource-limited modes. Only 36 of 56
harness controls verify. Calling an absent method can produce a TypeError
expected by a negative test, so those ten passes do not establish method
support. The unhealthy preflight and nonzero exit are retained.

The [independent local report](../tests/conformance/typedarray-views-local-before.json)
matches zero of 46 case modes and verifies four of 24 controls, with no runner
errors or binding drift. Its 23 bodies cover shared storage, coercion order,
species constructors, resizable and detached views, joined UTF-16 text, and
the exact `Array.prototype.toString` alias. Each body requires working feature
guards before its assertions. Six positive/wrong control pairs run in both
modes; a wrong partner requires intrinsic `Error` identity and a successful
positive partner. Missing-method TypeErrors cannot count as healthy controls.
The adapter does not expose authored Error messages; no message check is claimed.

All 284 Test262 protocol groups pass, including eight new groups. The separate
local runner passes ten protocol groups and verifies all 70 frozen input modes.
The 47 predecessor profile registrations and policies remain unchanged.
Sixteen historical protocol files exclude the new registration from their
original fixed populations without changing expected counts or contract hashes.
The new profile brings the registration count to 48.

The [preparation summary](evidence/typedarray-views-preparation.json) and its
archive bind the original sources, independent fixtures, held tools, before
adapter, reports and execution receipts. The executable SHA-256 is
`c21074ad37ed121547e6386dd529c67c6c438a6a0b2a6583d453118c49ef78e8`.
Runtime implementation and its eventual reports are separate from this commit.

```sh
python3 tools/import_test262.py --profile typedarray-views
python3 -m unittest discover -s tools -p 'test_test262_*.py'
python3 tools/test262_conformance.py --profile typedarray-views \
  --binary /absolute/path/to/eris-js --jobs 4 --timeout 3 \
  --output /absolute/path/to/fresh-typedarray-views-report.json
```

The language behavior is specified by the ECMAScript algorithms for
[subarray](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.subarray),
[join](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.join),
and [toString](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.tostring).
This selected preparation does not establish full language or web compatibility,
production security, or performance within 30% of Chromium.
