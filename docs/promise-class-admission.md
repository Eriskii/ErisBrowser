# Class metadata admission in Promise profiles

Seven existing Promise profiles now admit their declared `class` feature after
the [class runtime](classes.md) checkpoint. This changes metadata admission and
the prerequisite gate; it adds no Rust semantics, syntax or resource allowance.
The complete original corpora and their source/helper fingerprints are retained.
Earlier profile chapters preserve the results and policies of their own
checkpoints; this table records the subsequent admission measurement.

| Profile | Modes | Passed | Failed | Unsupported | Healthy controls |
| --- | ---: | ---: | ---: | ---: | ---: |
| Promise methods | 458 | 444 | 12 | 2 | 86 |
| Promise.all | 196 | 180 | 16 | 0 | 102 |
| Promise.race | 188 | 176 | 12 | 0 | 102 |
| Promise.allSettled | 208 | 194 | 14 | 0 | 106 |
| Promise.any | 188 | 178 | 6 | 4 | 114 |
| Promise.withResolvers | 12 | 12 | 0 | 0 | 56 |
| Promise.try | 30 | 30 | 0 | 0 | 106 |
| AggregateError, unchanged | 50 | 44 | 4 | 2 | 52 |
| **Total** | **1,330** | **1,258** | **64** | **8** | **724** |

The seven selected profiles retain 1,280 modes. Exactly 40 previously excluded
class-tagged modes reach the adapter: 36 pass and four reach the unchanged
final-unhandled-rejection audit. The latter are both modes of
`Promise/reject/ctx-ctor.js` and `Promise/any/ctx-ctor.js`, each reporting
`unhandled-rejection` with one outstanding rejection. Those failures remain in
the denominator and raw reports; they are neither hidden nor treated as proof
of an algorithm defect. All other case records and all 640 old control records
are exact, with no previously passing case regression. The remaining eight
metadata exclusions concern cross-realm support and destructuring. Passing all
12 withResolvers modes or all 30 Try modes establishes only these bounded
selections.

Each selected profile adds three positive/mismatch pairs in both modes: class
construction/call guards, species/alternate returns, and that profile's
subclass behavior. The 84 new rows use independent source, helper, mode and
fingerprint oracles. The canonical gate authenticates raw schemas, recomputes
classifications and verifies the ordered prerequisite graph, including exact
old references. Duplicate/missing rows, forged results, malformed fields and
cycles refuse the gate. A failed gate blocks the entire loaded case inventory
before adapter dispatch or a baseline update. Synchronous assertion diagnostic
text is bounded rather than used as an oracle; its raw error type/identity and
all completion fields remain checked.

withResolvers retains ordinary **ERJS1/ERJR2** transport. The other six selected
profiles retain **ERJS2/ERJR3**, authenticated completion and the final unhandled
rejection audit. Existing negative-error identity boundaries, including the
Any-specific AggregateError capability, remain unchanged. All 54 other profile
policy maps are exact. Author work remains 100,000 units, trusted initialization
1,000,000 and cumulative accounted heap 8 MiB.

The ordinary Rust 1.88 and host Rust 1.99 adapters were rebuilt and frozen before
parent and candidate measurements; both phases use the same binary bytes.
The complete Python suite passes **696 tests with no skips**, retaining all 683
parent identities plus 13 new adversarial groups. The first full run's 695
passes and one failure are retained: a legacy withResolvers mock needed the
selected `class` feature in its expected set. That one-set fixture correction
changed no producer, oracle or runtime behavior.
Seven baselines were updated after independent review. All eight fresh baseline
gates exit zero, with complete reports equal to the measurements except elapsed
time. Their cumulative 56 new passes and 14 newly reached failures include the
prior runtime checkpoint's 20 and ten; this admission adds only 36 and four.
Failed and unsupported rows remain recorded; unselected baselines are unchanged.
A source-review correction retained the old cross-mode prerequisites while
restricting only new class edges to the same mode. No runtime change, full
Rust suite rerun or GPU validation belongs to this admission checkpoint.

For a measurement without baseline writes, use the ordinary adapter for
withResolvers and AggregateError, and the host adapter for the other profiles:

```sh
python3 tools/test262_conformance.py --profile promise-with-resolvers \
  --binary /path/to/ordinary/eris-js --jobs 1 --output /tmp/with-resolvers.json
python3 tools/test262_conformance.py --profile promise-try \
  --binary /path/to/test262-host/eris-js --jobs 1 --output /tmp/promise-try.json
```

Without a baseline flag, a retained nonpassing case produces a nonzero exit;
the raw report still distinguishes ordinary failures from unsupported cases.
The evidence runner records all eight profiles serially and preserves every
before/after transition. See the [manifest](evidence/promise-class-admission/manifest.json)
and [source/check archive](evidence/promise-class-admission/sources-and-checks.tar.gz)
for exact bindings and commands. Full JavaScript/web compatibility, measured
full-browser correctness, a security audit and a Chromium-relative performance
result remain open. Fields, private elements, static blocks, `super` properties
and async/generator methods retain the class subset's documented exclusions.
