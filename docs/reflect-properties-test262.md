# Reflect property method Test262 preparation

The `reflect-properties` profile retains 108 original Test262 files at revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`. Each runs in sloppy and strict mode,
giving 216 mode cases. This is a fixed source selection prepared before the
nine-method implementation; it does not claim those cases pass.

The [published-engine observation](../tests/conformance/test262-reflect-properties-before.json)
uses the frozen adapter from `b4ce657`: 46 passed observations, 154 failures and
16 unsupported modes. Harness preflight verifies only 36 of 52 controls, so
this is an unhealthy pre-implementation report, not a conformance baseline.
Missing methods can themselves produce the TypeErrors expected by invalid-target
tests; those individual passes do not establish that the methods work. The
nonzero run exit and every original observation are retained.

The [separate local observation](../tests/conformance/reflect-properties-local-before.json)
matches zero of 28 cases and verifies four of 20 controls, with no runner errors
or source/binary drift. The [preparation evidence](evidence/reflect-properties-preparation.json)
binds the corpus, source holds, runner, executable and both reports. The engine
implementation is a separate change; neither preparation report is replaced
by later candidate results.

All 276 Test262 protocol tests pass, including seven new groups for this profile.
The first run exposed three omitted historical profile exclusions; the failed
run and narrow registration corrections remain in the evidence archive. No old
profile expectation or feature policy was changed to resolve those failures.

| Directory below `test/built-ins` | Original files |
| --- | ---: |
| `Reflect` (direct object tests) | 3 |
| `Reflect/deleteProperty` | 11 |
| `Reflect/get` | 11 |
| `Reflect/getOwnPropertyDescriptor` | 13 |
| `Reflect/getPrototypeOf` | 10 |
| `Reflect/has` | 10 |
| `Reflect/isExtensible` | 8 |
| `Reflect/preventExtensions` | 10 |
| `Reflect/set` | 18 |
| `Reflect/setPrototypeOf` | 14 |

All files in the nine selected method directories and all three direct Reflect
object tests are included. The method directories have no nested directories
at this revision. The root tests cover `Symbol.toStringTag`, the object's
prototype, and its global descriptor and non-callability. Existing apply,
construct and ownKeys profiles retain their original populations; the new
profile does not claim the entire Reflect subtree.

The importer authenticates the complete nonrecursive Git trees from the pinned
commit through each selected directory. Original sources are checked against
their Git blob identities. The exact `assert.js`, `sta.js`, `propertyHelper.js`,
`compareArray.js` and `isConstructor.js` helpers, plus `LICENSE` and
`INTERPRETING.md`, are retained unchanged and checked against fixed blob pins
independently derived from that revision's root and harness trees. Metadata,
source text, requested helper order and both modes remain unchanged.

The profile admits the observed `Reflect`, `Reflect.set`,
`Reflect.setPrototypeOf`, `Reflect.construct`, `Symbol`, `Symbol.toStringTag`
and `arrow-function` feature tags. Eight original files declare `Proxy`:
their 16 modes remain in the report as explicitly unsupported. The other 200
modes are eligible for execution under this policy, not predicted passes.
Module, asynchronous and host-hook policy remains unchanged. Unsupported
dependencies encountered without an upstream feature tag remain actual raw
observations; they do not justify removing a source or changing its oracle.

The existing 32 harness preflight modes are followed by 20 modes from five
independently frozen positive/wrong control pairs. The controls cover UTF-16,
the original get receiver, an accessor receiver on ordinary set, distinct
TypedArray target/receiver behavior and resizable-view extensibility. A wrong
partner requires runtime intrinsic `Error` identity and a successful positive
partner in the same mode. An absent method's `TypeError` cannot establish
healthy negative coverage. Every result is retained even when preflight fails.

The separate [local fixture](../tests/conformance/reflect-properties-local/RUNNER.md)
contains 28 case modes and 20 control modes with explicit true-return checks;
those 48 rows are not included in the 216 upstream modes. The shared profile
does not change existing profile feature sets, case fingerprints, timeout or
adapter supervision. Historical Python contract tests exclude this newly
registered profile from their earlier population hashes, and the current
registered-profile count increases from 46 to 47.

```sh
python3 tools/import_test262.py --profile reflect-properties
python3 -m unittest discover -s tools -p 'test_test262_reflect_properties.py'
python3 tools/test262_conformance.py --profile reflect-properties \
  --binary /absolute/path/to/eris-js --jobs 4 --timeout 3 \
  --output /absolute/path/to/fresh-reflect-report.json
```

Import, protocol tests and engine observations are separate steps. The project
owner records their actual results and binary identity. Preparation by itself
provides no candidate result or full ECMAScript conformance claim. The selected
language behavior is defined by the
[ECMAScript Reflect object algorithms](https://tc39.es/ecma262/multipage/reflection.html#sec-reflect-object).
