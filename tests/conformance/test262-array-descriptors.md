# Complete pinned property-definition and array-length inventory

This profile retains all **1,793 direct sources / 3,574 strict/sloppy modes**
from Test262 revision `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`:

| Directory | Sources | Modes |
| --- | ---: | ---: |
| Object/defineProperty | 1,131 | 2,250 |
| Object/defineProperties | 632 | 1,264 |
| Array/length | 30 | 60 |

The [manifest](../upstream/test262-array-descriptors/manifest.json) includes
unchanged test/harness bytes and retained Git-tree proofs. The importer walks
nonrecursive trees from the pinned commit/root and verifies binary Git tree
hashes and source blob hashes. The runner repeats that proof offline. This
includes entries beyond the GitHub Contents API's 1,000-item limit. Tampered,
truncated, omitted, duplicate and extra inventory records are rejected.

The published `48fce52d` binary records **2,802 passed / 88 failed / 12 resource
stops / 672 unsupported**. The descriptor implementation records **3,466 passed /
88 failed / 20 unsupported**, gaining **664 passes** with unchanged source,
mode and feature policies. No previous pass is lost. All 88 previously failed
observations remain unchanged: 76 reach missing Date, eight inspect missing
Object integrity methods and four inspect missing Array every/some. Eighteen
modes remain excluded by their declared unsupported features (Reflect.set,
cross-realm, BigInt, Proxy or resizable buffers); two reach unsupported host
property definition. This is a complete selection, not complete ECMAScript.

The profile executes unchanged `assert.js`, `sta.js`, `propertyHelper.js`,
`compareArray.js`, `isConstructor.js` and `resizableArrayBufferUtils.js` as needed.
Its **84 assertion controls** include guarded positive/deliberately-wrong pairs
for indexed descriptors, accessors, length conversion/shrink/rollback,
nonwritable length, staged and partial definitions, keys, metadata and Reflect's
boolean rejection. See the [implementation evidence](array-descriptors.json)
for before/after controls and the retained correction of one initial setup.

The runner's existing resource, timeout, crash and malformed-output rules apply.
Sources and cases are not removed when the engine cannot execute them. Callback,
heap and instruction limits are unchanged.

The candidate satisfies the existing healthy-run criteria and has a
[regression baseline](test262-array-descriptors-current.json) exercised in CI.
This preserves passes without treating the remaining failures as conformance.

```sh
python3 tools/test262_conformance.py --profile array-descriptors --baseline tests/conformance/test262-array-descriptors-current.json
```
