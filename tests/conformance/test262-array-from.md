# Pinned Test262 Array.from inventory

This profile retains the **complete 47-source / 90-mode**
`test/built-ins/Array/from` subtree at Test262 revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`. The
[manifest](../upstream/test262-array-from/manifest.json) preserves all 50,155
source bytes, 45 sloppy and 45 strict modes, five unchanged helpers, legal
files and seven original Git API proof documents. The proofs authenticate the
pinned commit/root through the Array directory to the recursive subtree and
reconstruct directory and blob identities. No source was selected or removed
based on its result.

The pinned Git root is `91b2052adad1f066ae031e2ff3a1e9bd6d732886`;
the Array/from subtree is `e3a97b42c65283fc23a15d65610c2461e6f02715`.
The original helpers are `assert.js`, `sta.js`, `compareArray.js`,
`propertyHelper.js` and `isConstructor.js`; `LICENSE` and `INTERPRETING.md`
are retained unchanged.

Admission was frozen before execution as exactly
`CONSTRUCTION_FEATURES | REGEXP_FEATURES | {'for-of', 'let', 'const', 'Array.prototype.values'}`.
No `Array.from` feature tag occurs in this inventory, so no additional tag was
admitted. All **86 admitted modes** execute; the four metadata exclusions are
both modes of `iter-set-elem-prop-non-writable.js` (`generators`) and
`proto-from-ctor-realm.js` (`cross-realm`). Untagged prerequisites remain
admitted, including the missing dependencies below.

## Observations and remaining gaps

The [initial report](test262-array-from-initial.json) and
[final report](test262-array-from-final.json) are exact copies of the frozen
reports, including every observation, exclusion and raw control result.

| Outcome | Before | Final |
| --- | ---: | ---: |
| Raw passed | 8 | 82 |
| Failed | 78 | 4 |
| Metadata unsupported | 4 | 4 |
| Resource, timeout or adapter error | 0 | 0 |
| Verified controls | 32 / 96 | 96 / 96 |

The eight before passes are both modes of `items-is-null-throws.js`,
`mapfn-is-not-callable-typeerror.js`, `mapfn-is-symbol-throws.js` and
`source-object-length-set-elem-prop-err.js`. Their unguarded TypeError
assertions also pass when `Array.from` is absent: calling the missing method
throws before the intended behavior is reached. All new successful
availability/array-like/custom-iterator controls failed before implementation,
so these historical raw passes did not demonstrate Array.from support.

The final replay adds **74 failed-to-passed modes with zero raw pass losses**.
The other 16 complete case records remain identical, including all eight
historical raw passes. All 32 original controls remain identical; all 64 new
controls now verify. The final report preserves four failures:

| Source, both modes | Retained result and dependency |
| --- | --- |
| [elements-deleted-after.js](../upstream/test262-array-from/test/built-ins/Array/from/elements-deleted-after.js) | TypeError, `value is not callable`; its mapper calls the missing `Array.prototype.splice`. |
| [items-is-arraybuffer.js](../upstream/test262-array-from/test/built-ins/Array/from/items-is-arraybuffer.js) | ReferenceError, `'ArrayBuffer' is not defined`; construction fails before Array.from is called. |

The selected algorithm covers iterable and array-like inputs, optional
mapping, custom constructors, element creation and the required iterator-close
boundaries. The normative reference is ECMA-262's
[Array.from algorithm](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.from).
This profile does not establish generator, cross-realm, ArrayBuffer, Proxy,
async iteration or general host-object support. Shared work, heap, call-depth
and collection limits remain in force; resource exhaustion is terminal rather
than an ordinary catchable author exception. Passing this selection is not a
claim of complete ECMAScript or unrestricted host compatibility.

## Controls and identities

There are **96 controls**: 32 unchanged common controls plus 16 independently
prepared positive/deliberately-wrong pairs in both modes. Each new setup first
proves method availability and successful array-like and ordinary custom
iterator behavior. The pairs exercise constructor selection, mapping and
receiver behavior, property metadata, live input, element definition, final
length and abrupt iterator-close boundaries. A wrong partner verifies only
when the unchanged assertion harness produces Test262Error and its separate
same-mode positive partner verifies. Its own missing-method TypeError cannot
satisfy this rule. Control sources, modes and expectations were frozen before
execution and copied exactly into the formal runner.

Before adapter: `66bf753b598273161e6cc88f5004d83b4d5d1f88e2919e96db2414ccc2cc1316`,
from published commit `cbb970d0e0b117dd50a3861336afd59ac19f9aaa`, input digest
`cd50007f53a7b9431c104be33996295c2e4da96440963637f3eef20a02aedfa8`.
Final candidate adapter: `6b2a21601bbc850c3a7890b99b8990f7e101b1da6f4246f4eb06711d2ff1208c`,
input digest `a9e209d3902f80f62e941328d53b7abc94862b06613bd40797d624f1a63ea7de`.

| Bound record | SHA-256 |
| --- | --- |
| Initial report | `11539b906d61ad298cf4af78104c32c20f3b40ae43b545363981ed2dee0c88b6` |
| Final report | `dde935f55627d0ec35c8dfe0b2505b6aa3306e740228f06433d6093fdc8f4478` |
| Current baseline | `7fc83d01ec50f3784b64a9f2b6123df999ad303e6d0067cf16a55914603fdf5c` |
| Corpus manifest | `9fac107410a7e10f895b1c9900c802626f1f3652040877d662c9892b056180d5` |
| Runner policy | `3ffb7cd01ba9148d4ef61806a0d476c682ebbb9f9dce37190188a963006a73da` |
| Frozen source/mode/helper case records | `667254f669caa01a380dfa64ee10a3b1f079d845969f0a47897e61a56d313d56` |
| Frozen control records | `2bff59affb5eb692f3bdfbc6d6c7cee413d50b2751f8139579c880d937c03b3a` |
| Frozen proof-file records | `051606ab3d791ac7922b1fd14e8db440aff8d1d69ad5c3b1f8e0f0ef95f741c5` |
| Formal tooling/corpus freeze | `c464ad86fc8e39a254cb5c7dae1272af478a8459b6f002c6b1b7450edc87f26c` |
| Before execution ledger | `4b8fa6865e9d5237c75c21d79c867e899d6588aeca1cebd2cf245ad148195969` |
| Final execution ledger | `527c9496b70c4fe19d9ca285b93b459d629fbbab20dce19d7c6c4577624e1065` |
| Full observation/control comparison | `2cec6c517c7c2f1faa44b1ad00b0b7f6ce8020866bb52c9949f259de4cb01b0f` |

The case/control/proof digests use sorted-key, compact JSON from the frozen
contract's corresponding fields. Both runs verified all 72 formal frozen
files and their binary/tool identities before and after execution. The final
run additionally verified all 98 production inputs against the candidate
manifest. Every case/control fingerprint and metadata exclusion remained
unchanged. Runs used three-second per-case limits, four workers, bounded
output and a 600-second outer deadline, with process-group cleanup before
reaping the leader. There were no retries or normalized failures.

## Known-state regression gate

The [current baseline](test262-array-from-current.json) is the runner's exact
`current` shape derived from the final report. It preserves **82 passed,
4 failed and 4 metadata-excluded cases**. The full final report retains all
**96 verified controls**; the compact baseline itself stores case identities
and statuses, not control observations. Baseline admission requires all
controls to verify and no `BAD_RUN_STATUSES` (`resource`, `timeout`,
`adapter-error`). These conditions hold for this observation.

`check_baseline` against the same final report returns **zero regressions and
zero unrecorded improvements**. This is a healthy known-state regression gate,
not an all-pass standards-support claim. Publication copied exact report bytes
and checked the derived baseline in memory; it did not execute another engine
or alter older baselines. A run without a baseline returns nonzero for the
retained failures and exclusions.

```sh
python3 tools/import_test262.py --profile array-from
python3 tools/test262_conformance.py --profile array-from --baseline tests/conformance/test262-array-from-current.json
```

## 2026-09-29 splice follow-up

The splice follow-up resolves both modes of `elements-deleted-after.js`:
**84 passed / 2 failed / 4 metadata excluded**, with all **96 controls**
verified. These two gains are the only changed complete observations in this
90-mode profile. The two remaining failures still require missing ArrayBuffer.
The current baseline now records the gains; the original final report and the
historical counts/baseline hash above remain unchanged as checkpoint evidence.
This remains a healthy known-state gate retaining two failures, not an all-pass
standards claim.

The [splice profile](test262-array-splice.md) records the new implementation
and fixed selection. Candidate adapter: `f9ffdf6172908c875308ee40fe5783d3b673bbdccf7685a61c608b331348f721`.
Full follow-up report SHA-256: `17c000505a74a0824f90598c97f61fc6d7ad155f277daa75760b5b3e83cfae0b`.
Updated current baseline SHA-256: `9057d437c4f6ef9d8f758549f456377258b6c8af024cc50ae3b214c110378daf`.
Source/helper/mode/policy identities and exclusions are unchanged. The old
gate passed before this baseline update; no engine rerun or outcome
normalization was used to derive the projection.
