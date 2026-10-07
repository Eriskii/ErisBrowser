# Object.hasOwn upstream coverage

The custom interpreter passes **124/124 modes** from the complete pinned
`test/built-ins/Object/hasOwn` directory, with all **64 harness controls healthy**.
All 62 upstream sources run unchanged in sloppy and strict modes. There are no
exclusions, negative-metadata tests or fixtures in this directory.

This checkpoint adds a corpus, runner profile and CI baseline for the method
implemented in the [Node cloning increment](../../docs/node-clone.md). It uses
that published release unchanged; it introduces no runtime implementation or
resource-limit changes. Passing this directory does not establish complete
ECMAScript conformance or universal host-object reflection.

## Inventory and checks

The Test262 revision is `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd` and the
directory tree is `c415403752259885fd6cfda1999381beef67ee70`. Its 62 test files
contain 33,881 bytes. Root-linked Git proofs authenticate every source, the
`assert.js`, `sta.js`, `propertyHelper.js` and `isConstructor.js` helpers, and
the retained legal files. The importer and runner reject changed, missing,
additional and unsafe inputs.

The profile admits only `Object.hasOwn`, `Symbol`, `Symbol.toPrimitive`,
`Reflect.construct` and `arrow-function`. Its controls retain the 32 common
checks and add eight positive/deliberately-wrong pairs in both modes: ownership,
accessor presence, target-before-key conversion, exotic symbol keys, symbol
results from `toString` and `valueOf`, property metadata and nonconstruction.
Each negative feature control requires both a Test262Error and its successful
positive prerequisite in the same mode. Missing methods or broken helpers
cannot satisfy these controls.

Coverage includes own versus inherited data/accessor descriptors, primitive
targets, key conversion and conversion order, symbols, function metadata and
nonconstruction. The directory does not cover every Proxy or host-object case;
Proxy, BigInt and complete host reflection remain unfinished in the browser.

```sh
cargo build --locked --release --bin eris-js
python3 tools/test262_conformance.py --profile object-has-own \
  --baseline tests/conformance/test262-object-has-own-current.json
python3 -m unittest discover -s tools -p 'test_test262*.py'
```

The [raw report](../../docs/evidence/object-has-own-upstream/report.json),
[baseline](test262-object-has-own-current.json) and
[validation record](../../docs/evidence/object-has-own-upstream/index.json)
retain the result and provenance. The 44 existing profiles' definitions,
policies and preflight contracts compare byte-for-byte unchanged: 19,727 case
modes and 4,564 controls. That comparison is a tooling check, not a fresh engine
replay of those profiles.

All 394 Python tooling tests pass, including ten new regression groups for
inventory integrity and harness health. Three initial historical-population
failures and their narrow exclusions are retained in the validation record.
