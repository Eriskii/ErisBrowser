# Pinned Test262 Array.prototype.concat observations

This observation-only profile retains the complete recursive
[`Array/prototype/concat`](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/Array/prototype/concat)
subtree at revision `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`:
**69 sources / 137 modes**, one directory and **77,035 original source bytes**.
There are 69 sloppy and 68 strict variants; the duplicate-parameter arguments
source alone declares `noStrict`. No source or helper is rewritten or omitted.
The [manifest](../upstream/test262-array-concat/manifest.json) and eight original
Git proof documents authenticate the root-linked subtree
`6a606717eec7a333a5136fc03dc7b941a28dcd3e` and every selected blob.

The five unchanged helpers are `assert.js`, `sta.js`, `compareArray.js`,
`propertyHelper.js` and `isConstructor.js`. Original
[LICENSE](../upstream/test262-array-concat/LICENSE) and
[INTERPRETING.md](../upstream/test262-array-concat/INTERPRETING.md) are retained.
The policy is exactly `ARRAY_SPLICE_FEATURES.copy()`: **121 admitted modes**
and **16 metadata exclusions** (12 Proxy and four cross-realm). Untagged class
and typed-array prerequisites remain admitted. No execution outcome changes
selection, metadata, helpers or policy.

| Observation | Published before | Concat candidate |
| --- | ---: | ---: |
| Passed | 14 | 113 |
| Failed | 105 | 4 |
| Metadata excluded | 16 | 16 |
| Additional parse unsupported | 2 | 2 |
| Runtime resource limit | 0 | 2 |
| Timeout / adapter error | 0 | 0 |
| Verified controls | 32 / 96 | 96 / 96 |

The [initial](test262-array-concat-initial.json) and
[final](test262-array-concat-final.json) reports are exact copies of the
single frozen before and candidate executions. **99 modes gain a pass**;
none loses a pass. The 14 before passes are seven exception-only TypeError
sources in both modes: `Array.prototype.concat_array-like-to-length-throws.js`,
`create-ctor-non-object.js`, `create-species-non-ctor.js`, and the four
`create-species-non-extensible*` / `create-species-with-non-configurable-property*`
sources. Invoking the absent method already produced their expected TypeError.
Those incidental passes did not establish concat support; the before feature
controls all remained unverified.

The four remaining failures are both modes of the small/large typed-array
sources, each retaining its missing `Uint8Array` ReferenceError. Both modes of
`Array.prototype.concat_non-array.js` retain the untagged class parse limitation.
The two newly reached resource stops are both modes of
`Array.prototype.concat_spreadable-sparse-object.js`. Its unchanged source
checks a five-hole object followed by a **4,000-hole** object, and reports
`script instruction limit exceeded` at runtime. These are ordinary upstream
success expectations; the resource stops are not passes, expected ECMAScript
exceptions, or rewritten resource-policy tests.

The **96 controls** comprise 32 unchanged common controls plus sixteen
independently authored positive/deliberately-wrong pairs in both modes.
Every new control first completes Array and generic-receiver concat calls.
A wrong partner requires the unchanged harness's Test262Error and its
successful same-mode positive partner. All 96 controls now verify, with exact
source, mode, helper and expectation fingerprints preserved.

**No concat baseline or CI gate is created.** The unchanged runner includes
`resource` in `BAD_RUN_STATUSES`, so these two resource observations prevent
baseline recording even though every control verifies. The formal CLI status
remains **1**. The [preparation and bindings record](test262-array-concat-preparation.json)
retains this decision, exact source/binary/report hashes and complete comparison
references. This is partial conformance evidence, not an all-pass result.

All **40 prior profiles / 17,984 cases / 3,996 controls** retain identical
complete observations, and all **33 existing baseline gates** pass. Those 33
baseline files remain byte-identical. No quota, prior profile policy, corpus
source or external expectation changed. The separate local/runtime evidence is
maintained in [the concat checkpoint](array-concat.md).

```sh
python3 tools/test262_conformance.py --profile array-concat --output artifacts/test262-array-concat.json
```

This observation command returns nonzero while the retained resource and other
nonpassing observations remain. It does not record or update a baseline.
