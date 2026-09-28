# Pinned Test262 Array.prototype.sort selection

This isolated profile retains all **54 direct JavaScript sources** in
[Test262's Array.prototype.sort directory](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/Array/prototype/sort)
at revision `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`. There are no nested
directories or fixtures at this pin. The complete source inventory and hashes
are in the [manifest](../upstream/test262-array-sort/manifest.json).

The 54 sources produce **107 variants: 54 sloppy and 53 strict**. Only
`S15.4.4.11_A8.js` specifies `noStrict`; every other source runs in both modes.
There are no negative metadata cases, modules, asynchronous flags or per-test
timeouts. Runtime exception assertions inside these positive tests remain part
of the unchanged programs.

Sources total **150,796 bytes**. The selection retains unchanged `assert.js`,
`sta.js`, `compareArray.js`, `propertyHelper.js`, `isConstructor.js` and
`resizableArrayBufferUtils.js`, plus the upstream
[LICENSE](../upstream/test262-array-sort/LICENSE) and `INTERPRETING.md`.
The total imported size is **200,865 bytes**. Every source was checked against
its pinned Git blob; the runner verifies every source, harness, metadata and
mode fingerprint. No test is shortened, rewritten or removed to accommodate
missing capabilities.

Manifest SHA-256:
`d6f43dd379ae49ae08f959596852e28e6bd33a0420eb699d763ff696b0e961d2`.

```sh
python3 tools/import_test262.py --profile array-sort
python3 tools/test262_conformance.py --profile array-sort
python3 -m unittest discover -s tools -p 'test_test262_conformance.py'
```

## Reduction-method checkpoint

The [complete reduction comparison](test262-array-sort-reduce.json) preserves
all 107 case identities, 80 preflights and the same policy. Results are now
**57 passed / 46 unsupported / four resources**, with no lost passes.
Implementing reduce enables the unchanged 5- and 11-element stability sources
in both modes. The 513-element source now reaches the shared runtime instruction
limit in both modes; these two earlier missing-method failures remain nonpasses.
The two 2,048-element modes retain their runtime allocation stops.

All preflights verify. No healthy baseline is recorded while these resource
stops remain. The adapter SHA-256 is
`2b0e59686ff8fb8db409c8a54b8f92df592f00668fcd9ff00da2de0b3585705a`;
the source-input digest is
`be187dd3a00025c8461dc08013ca49696231b350d5b4490ea8166c15de83a79f`.
The following measurements remain historical evidence.

## Identifier compiler checkpoint

The [full later comparison](test262-array-sort-identifiers.json) preserves all
107 cases, 80 preflights, policy and result categories: **53 passed / six failed /
46 unsupported / two resources**. The two unchanged 2,048-element stability
modes now reach the runtime allocation limit rather than the earlier instruction
limit because initial token/compiler storage is accounted cumulatively. Their
exact before/after diagnostics are retained; this is not a new pass. The final
adapter SHA-256 is
`b7f77cbbc11f14297b7b34716668a10c0522be62562a33a07756dcc89f17d12f`.
The following original sort measurements remain historical evidence.

## Isolated policy and retained limitations

The policy adds only `stable-array-sort` to the existing core feature set:
`String.fromCodePoint`, `arrow-function`, `for-in-order`, and
`well-formed-json-stringify`. This schedules **93 variants** for execution;
**14 variants** remain metadata-unsupported. Admission is not a passing claim.
The previous seven profiles, their policies, sources and preflights are
unchanged. An independent comparison preserved all **2,317 prior case
fingerprints** and **360 prior preflight fingerprints**, including manifest
hashes, fixtures and supported-feature sets.

| Metadata-unsupported sources | Variants | Unavailable declared features |
| --- | ---: | --- |
| `call-with-primitive.js` | 2 | BigInt, Symbol |
| `comparefn-nonfunction-call-throws.js` | 2 | Symbol |
| `comparefn-grow.js`, `comparefn-resizable-buffer.js`, `resizable-buffer-default-comparator.js` | 6 | resizable-arraybuffer |
| `comparefn-shrink.js` | 2 | Array.prototype.includes, resizable-arraybuffer |
| `not-a-constructor.js` | 2 | Reflect.construct |

These are whole-file outcomes. Ordinary cases mixed into Symbol/BigInt files
are not extracted as replacements. The unchanged resizable-buffer helper also
uses typed arrays, dynamic construction and other exotic capabilities; keeping
its bytes does not imply implementing them. Proxy remains outside the policy.

The four stability files retain **5, 11, 513 and 2,048 literal objects**. Their
source sizes are 840, 1,192, 16,507 and 64,618 bytes. All four call
`Array.prototype.reduce` after sorting; the current runtime does not implement
reduce. This independent dependency remains observable rather than replacing
the assertions with shorter checks. The large programs also retain their full
callback, storage and work requirements. Resource failures, if observed, stay
resource failures and prevent a healthy baseline; they are not filtered or
relabelled as unsupported to improve the score.

The 19 precise-behavior sources cover eight getter mutations, eight setter
mutations, two prototype cases and comparator abrupt completion. The 16 direct
getter/setter files install indexed Array accessors, currently an unsupported
descriptor operation. These files still execute and report that limitation.
Prototype reads/setters, generic receivers, holes versus undefined, conversion
order and native method metadata remain in the complete inventory.

Execution-policy SHA-256 with the standard three-second process deadline:
`8621a5b10295e4a41806178744c4e906157ef78c884220a154860fae829fd205`.
The [shared runner contract](test262.md) applies, including bounded fresh
processes, unchanged assertions, stable adapter hashes and complete outcomes.

## Paired assertions

This profile keeps **32 unchanged core preflights** and adds **48 variants**:
12 positive/deliberately incorrect pairs, each in sloppy and strict mode.
All **80 preflights** must verify before a healthy baseline can be recorded.

The added checks cover method availability/receiver identity, default string
ordering, stable custom comparison, undefined and holes, comparator validation
before length access, callback receiver/arguments/numeric coercion, UTF-16
default ordering and observable string conversion, generic/captured length,
collection before writeback, inherited indexed accessors, exact thrown reasons,
throwing writes/deletes with prior effects retained, and native descriptors.
They follow [Array.prototype.sort](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.sort),
[SortIndexedProperties](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-sortindexedproperties)
and [CompareArrayElements](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-comparearrayelements).

Every new check starts with a callable-method assertion and a successful sort
whose result and receiver identity are asserted. These guards are outside
`assert.throws` callbacks, so a missing method or failed `.call` lookup cannot
establish success for a positive TypeError check. The deliberately incorrect
partner reuses the same setup and changes only its final assertion; it must
raise the unchanged harness's `Test262Error`, not an unrelated intrinsic error.
A wrong partner passing in isolation does not prove health: its positive partner
must also pass. Supplemental ordering checks use ordinary-object accessors to
avoid depending on indexed Array descriptor support; the original Array
accessor tests remain unchanged. The preflights do not require reduce and do not
assert an implementation-specific comparison schedule.

Five new Python test groups check exact inventory, mode/source sizes and
unchanged supporting bytes, isolated feature admission, preservation of core
preflights, disabled-assertion/wrong-error detection, the availability guard and
paired setup, and importer rejection of corrupt blobs or an incomplete listing.
The focused Python module has **45 passing tests** at this checkpoint.

## Frozen initial measurement

The [complete initial report](test262-array-sort-initial.json) measures the
preserved adapter from checkpoint `cbe6efa7bceb46e7eb0b52b590a67b81656563c0`, before
Array.prototype.sort implementation.

| Outcome | Variants |
| --- | ---: |
| Passed | 0 |
| Failed | 61 |
| Unsupported | 46 |
| Resource stop, timeout or adapter error | 0 |

All **32 core preflights** verify. All **24 new positive variants** fail the
method-availability guard. Their **24 deliberately incorrect partners** also
raise Test262Error at that guard and therefore meet their expected failure
classification, but the positive failures keep overall health false:
**56/80 verified**, with **no healthy initial baseline**. An actual requested
baseline write was refused. No individual expected-error outcome is used to
claim method implementation.

Exact observed failure groups are:

| Variants | Error type / intrinsic identity | Actual message | Cause |
| ---: | --- | --- | --- |
| 51 | TypeError / TypeError | `value is not callable` | Missing sort method or its callable use |
| 4 | TypeError / TypeError | `cannot convert null or undefined to object` | Method name/length descriptor tests reach an absent method |
| 6 | Test262Error / empty | `uncaught exception: [object Object]` | Unchanged assertions reject missing method behavior/descriptor or a wrong thrown value |

The six authored assertion failures are both modes of
`S15.4.4.11_A5_T1.js`, `precise-comparefn-throws.js`, and `prop-desc.js`.
The four descriptor TypeErrors are both modes of `length.js` and `name.js`.
Every source/mode and observation is retained in the linked report.

Of 46 unsupported variants, **14** are the explicit metadata cases above and
**32** reach `UnsupportedFeature` with message
`array indexed/length descriptor mutation is not implemented` in the 16 precise
indexed getter/setter sources. No new synthetic source exclusions were added.
The four stability sources are retained as eight actual failed observations;
the absent sort method is reached before their reduce dependency.

Initial adapter SHA-256:
`3bcf5b560c662c70568c0748c6646f85fe056710231bd339b0317b67ec3c7663`.
Source-input digest:
`7ab85433d856ca68c585e4a02c3f52a96dcb3ae90f72414b8e8282b1f6415975`.
Sanitized full initial-report SHA-256:
`f7b2bb66e0d06aa20a24663f8cff03e147bb2692561c8c132f08a9ff387fee6d`.

## Corrected preflight contract

The initial preliminary runs exposed a mistake in the supplemental metadata
preflight: unchanged `verifyProperty` deletes configurable properties unless
its `restore` option is true. A later check then read the deleted `m.length`.
This was a preflight setup error, not an engine defect. Only the new metadata
pair now requests `{restore:true}` on its three helper calls; upstream helpers
and all corpus sources remain unchanged. Four new preflight fingerprints
changed (the positive/wrong pair in both modes); every other preflight and all
107 case identities stayed fixed.

Both frozen adapters were rerun with the same corrected 80-preflight contract.
The [correction record](test262-array-sort-preflight-correction.json) retains
before/after tooling hashes and the four exact fingerprint changes. The full
[preliminary initial](test262-array-sort-preliminary-initial.json) and
[preliminary candidate](test262-array-sort-preliminary-latest.json) reports are
retained as superseded observations. Their candidate's two metadata-preflight
failures are not presented as implementation gains. The authoritative initial
and latest reports use the corrected identical contract.

## Implemented-sort measurement

The [complete latest report](test262-array-sort-latest.json) retains every
outcome and the exact before/after comparison:

| Outcome | Initial | Implemented sort |
| --- | ---: | ---: |
| Passed | 0 | **53** |
| Failed | 61 | **6** |
| Unsupported | 46 | **46** |
| Resource stop | 0 | **2** |
| Timeout or adapter error | 0 | **0** |
| Verified preflights | 56/80 | **80/80** |

All **107 source/harness/mode fingerprints**, the manifest and policy, and all
**80 corrected preflight fingerprints** are unchanged between the two runs.
There are **53 newly passing variants** and no prior passing variants to lose.
All **46 unsupported observations** are unchanged, including exact diagnostics.
The full report lists every status transition; the two failed-to-resource
transitions remain explicit rather than being counted as successes.

Both modes of the 5-, 11- and 513-element stability files now reach their
unimplemented reduce call and fail with intrinsic TypeError, message
`value is not callable`. Both modes of the complete 2,048-element stability
file stop with ResourceLimit, message `script instruction limit exceeded`.
Their full literal data and shared runtime budgets remain unchanged. No reduce
implementation, indexed Array descriptor support, metadata exception or quota
increase was added to improve these results.

**No healthy regression baseline exists for this profile.** An actual candidate
`--record-baseline` request was refused because the two resource outcomes
violate the runner's health criterion, even though all 80 preflights pass. No
baseline JSON or passing baseline CLI gate was created. The ordinary nonbaseline
command exits 1 and retains the complete report. This is an honest compatibility
measurement, not a full Array or Test262 conformance claim.

Measured adapter SHA-256:
`c039eb6be9360342f9cec2fd8da08cd0efc4245a04370bca0c114e1ff812825c`.
Source-input digest:
`dce62370c4dbfa806975d8c207bc38ad955997f89d653357848bc2e1b36a148c`.
Runtime source SHA-256:
`6d065ac12feee90995e5ecd8d40386f3bb609f89fdb1e57e9f0b0b7cce10955d`.
Sanitized latest-report SHA-256:
`582a810ea3464210665dd12217c9b577de37fccfe66d39705ee08a9c5c7919ca`.

The [implementation scope](array-sort.md) records the supported sorting and
receiver behavior and its resource limits. The seven earlier profiles and their
policies remain intact; no new passing CI baseline is claimed for this slice.
