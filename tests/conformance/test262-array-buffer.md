# Pinned Test262 ArrayBuffer observations

The complete recursive [`built-ins/ArrayBuffer`](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/ArrayBuffer)
subtree at revision `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd` contains
**221 unchanged sources / 442 modes** (221 sloppy and 221 strict), **15 directories**
and **261,335 original source bytes**. The [manifest](../upstream/test262-array-buffer/manifest.json)
and six original root-linked Git proof documents retain every selected blob,
including immutable-buffer proposal descendants outside this implementation.

The seven unchanged helpers are `assert.js`, `sta.js`, `compareArray.js`,
`detachArrayBuffer.js`, `isConstructor.js`, `propertyHelper.js` and
`testTypedArray.js`. Original [LICENSE](../upstream/test262-array-buffer/LICENSE)
and [INTERPRETING.md](../upstream/test262-array-buffer/INTERPRETING.md) remain intact.
The frozen policy is `ARRAY_CONCAT_FEATURES` plus `ArrayBuffer`,
`resizable-arraybuffer`, `arraybuffer-transfer` and
`align-detached-buffer-semantics-with-web-reality`. It admits **312 modes** and
retains **130 exclusions: 112 declared-feature exclusions and 18 conservative
Test262 host-hook source-check exclusions**. No execution outcome changes the
selection, helpers, expectations or policy.

| Observation | Published before | ArrayBuffer candidate |
| --- | ---: | ---: |
| Passed | 0 | 262 |
| Failed | 312 | 50 |
| Excluded | 130 | 130 |
| Resource / timeout / adapter error | 0 | 0 |
| Verified controls | 32 / 160 | 160 / 160 |

The [initial](test262-array-buffer-initial.json) and
[final](test262-array-buffer-final.json) reports preserve the exact bytes of the
single frozen before and candidate executions. The complete
[comparison](test262-array-buffer-comparison.json) retains every changed record:
**262 pass gains, zero pass losses**, 310 changed case records and 132 identical
case records. All 442 source/mode/expectation identities and all 160 control
identities remain unchanged. The before run has no incidental raw passes.
Both observation commands returned **status 1**, preserving nonpassing outcomes.

The **50 remaining failures** are prerequisite gaps. Forty-eight rows are the
24 fixed/resizable byte-copy sources under `prototype/transfer` and
`prototype/transferToFixedLength`, each in both modes: they construct missing
`Uint8Array` before reaching the transfer assertions. The other two rows are
both modes of [`options-non-object.js`](../upstream/test262-array-buffer/test/built-ins/ArrayBuffer/options-non-object.js):
its untagged `1n` literal produces `identifier immediately follows numeric literal`.
The source only declares `resizable-arraybuffer`, so the BigInt prerequisite
remains admitted and visible as a parse failure. The preparation record lists
all affected files and exact exclusion reasons; none is removed from the denominator.

The **160 controls** comprise 32 unchanged common controls and 32 independently
authored positive/deliberately-wrong pairs in both modes. Each feature control
requires successful buffer operations before checking an error, and each wrong
partner requires its successful same-mode positive partner. All 160 verify.

This formal run satisfies the unchanged baseline-health rule: every control
verifies and no case has a `resource`, `timeout` or `adapter-error` status.
The [current baseline](test262-array-buffer-current.json) retains **262 passes,
50 failures and 130 exclusions**, with one CI regression gate. It was projected
from this completed report after historical replay review, with no engine rerun.
A healthy known-state gate is not an all-pass conformance result.
The independent local suite separately retains two ordinary metadata instruction
limit stops and four expected terminal-resource cases. This formal health result
does not classify those local observations as passes.

The [preparation and bindings record](test262-array-buffer-preparation.json)
binds the frozen inventory, policy, controls, reports, execution limits and
candidate input `7cc8ea1321f029c705898aa7dd480e7a79bb878f454c342aeacfcba3f84274bf`.
The measured adapter is
`57eae22986e13bd11fe031716a9bd3554b74b31dc8a86c717fea9176ef06c18b`.
Publication projects the completed report offline and invokes no engine.
All **41 historical profiles / 18,121 modes / 4,092 controls** retain their
observations except four newly passing modes: `Array/from/items-is-arraybuffer.js`
and `Object/seal/seal-arraybuffer.js`, each in both modes. All 33 old gates passed
before baseline changes. Only the `array-from` and `object-integrity` baselines
are strengthened for those four gains; the other 31 remain byte-identical.
Seven older local suites retain **1,378 modes / 316 controls** with no complete
observation change. The [offline baseline validation](test262-array-buffer-baseline-validation.json)
checks all **34 current baselines** against completed candidate reports with
zero regressions and zero unrecorded improvements. The complete catalog now has
**42 profiles / 18,563 modes / 4,252 controls**; the eight observation-only
profiles receive no new gate. Runtime and local evidence are recorded separately
in [the ArrayBuffer checkpoint](array-buffer.md).

```sh
python3 tools/test262_conformance.py --profile array-buffer --output artifacts/test262-array-buffer.json
```

This observation command returns nonzero while the retained failures and
exclusions remain; it does not record or update a baseline.

The known-state CI command preserves the recorded passes while retaining the
reported prerequisite failures and exclusions:

```sh
python3 tools/test262_conformance.py --profile array-buffer --baseline tests/conformance/test262-array-buffer-current.json --output artifacts/test262-array-buffer-gate.json
```
