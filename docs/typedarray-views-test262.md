# TypedArray views and string conversion

The `typedarray-views` profile retains 103 original Test262 files at revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`, each in sloppy and strict mode.
The 206 modes cover `subarray`, `join`, and `toString`. The preparation reports
remain retained alongside the implementation results below.

## Implemented behavior and current results

All ten Number TypedArray kinds now have `subarray` and `join`. Default
subarrays share their original backing buffer; begin/end coercion and species
construction retain their specified order, with fresh validation after authored
callbacks. Fixed and tracking views preserve their different resize behavior.
`join` captures the initial length, converts its separator once, and reads each
current indexed element into exact UTF-16 text. The shared `toString` property
is the identical function object installed at `Array.prototype.toString`, so a
replaced `join` is observed through that function's ordinary dispatch. BigInt
views, shared buffers, foreign realms and the remaining TypedArray methods are
outside this increment.

| Report | Case modes | Verified controls |
| --- | --- | ---: |
| [Independent local matrix](../tests/conformance/typedarray-views-local-current.json) | 46 passed | 24 / 24 |
| [Selected upstream profile](../tests/conformance/test262-typedarray-views-after.json) | 46 passed, 64 resource-limited, 96 unsupported | 56 / 56 |
| [Foundation follow-through](../tests/conformance/test262-typedarray-after-views.json) | 590 passed, 34 failed, 488 unsupported, 126 resource-limited | 64 / 64 |

The final adapter verifies all local cases and controls. Only the 46 local
case modes and 24 paired-control modes form
the new **70-mode CI gate**. The Test262 runner treats `resource` as an unhealthy
run status. Its 64 upstream resource outcomes therefore prevent baseline
recording, even with all 56 controls healthy. No upstream baseline or gate is
created from this report, and no failure or resource observation is waived.

The larger shared prototype initially made both modes of the previously passing
`DefineOwnProperty/key-is-not-numeric-index.js` case exhaust the work budget.
Those exact original modes now pass again. Three bounded changes remove real
repeated work: for-in snapshots retain an identity-bound negative TypedArray
brand result; a single Set operation reuses only its immutable numeric-key
classification; and a visited length bucket stores its first name inline until
a second distinct live name requires a tree. Descriptors, prototypes and
buffer state remain live, hidden properties still shadow inherited names, and
all work and heap quotas are unchanged. First failed attempts and the original
reports remain in the evidence record.

Bootstrap construction also changed. DataView's 21 methods/accessors use their
existing metadata bags directly, while its constructor keeps its legacy route.
The TypedArray graph adds the two method bags and the existing Array `toString`
alias without changing prior intrinsic identities or property order. These are
source- and budget-checked implementation changes, not measured performance or
full compatibility claims.

The [implementation evidence summary](evidence/typedarray-views.json) and
[archive](evidence/typedarray-views.tar.gz) are the publication destinations for
source holds, original failures, corrections, raw reports and validation
receipts. All 2,283 tests pass in the full Rust 1.88 Vulkan-feature suite,
including confined browser tests. Rust 1.98 passes all 1,279 script tests.
Both toolchains pass strict Clippy checks. Five subsequent private-test lint
corrections were checked by rerunning the 44 own-key and 13 classified-key
groups on Rust 1.88 and the complete script set on Rust 1.98. The final
adapter SHA-256 is
`23ef852a16398e8d9a1ebcc15dbc9b847f1bc8aabc502b81953bb5cf9cbb49ad`.
A private test initially called absent `Array.prototype.fill`; its original
failure and bodies are retained, and independently reviewed `splice` witnesses
now exercise the same distinct-Receiver and post-detachment conversion paths.
That correction is recorded as a revised fixture, not an unchanged passing test.

## Preserved preparation and corpus policy

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

Preparation validation passed all 284 Test262 protocol groups, including eight
new groups. The separate local runner passed ten protocol groups and verified
all 70 frozen input modes.
The 47 predecessor profile registrations and policies remain unchanged.
Sixteen historical protocol files exclude the new registration from their
original fixed populations without changing expected counts or contract hashes.
The new profile brings the registration count to 48.

The [preparation summary](evidence/typedarray-views-preparation.json) and its
archive bind the original sources, independent fixtures, held tools, before
adapter, reports and execution receipts. The executable SHA-256 is
`c21074ad37ed121547e6386dd529c67c6c438a6a0b2a6583d453118c49ef78e8`.
That preparation commit contained no runtime implementation; its adapter and
reports remain separate from the later implementation records.

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
This selected increment does not establish full language or web compatibility,
production security, or performance within 30% of Chromium.
