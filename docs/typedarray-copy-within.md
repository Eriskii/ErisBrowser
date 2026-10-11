# Number TypedArray overlapping copies

`TypedArray.prototype.copyWithin(target, start, end)` copies a range within each
of the ten Number view kinds, including `Float16Array`, and returns the receiver.
It selects the overlap direction before copying complete raw elements. NaN
payloads and signed-zero bits are preserved; other views of the backing buffer
observe the completed writes.

The method validates the receiver and captures its initial length before
converting target, start and a defined end, in that order. Omitted or explicit
`undefined` end uses the captured length. An object converting to `undefined`
still undergoes numeric conversion. Abrupt conversions preserve their thrown
identity and prevent later conversions and copying.

An initially positive copy count requires a fresh buffer check after all
conversions, including self-copy. Detached or out-of-bounds views then throw.
A length-tracking view that shrank clips both source and destination ranges;
growth does not enlarge the captured range. An initially nonpositive count
returns without that final check, even if conversion detached the backing.

The implementation follows the
[ECMAScript algorithm](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.copywithin).
It authenticates internal view records and does not consult replaceable
constructor, species, length or numeric prototype properties. Checked raw reads
and writes admit each complete scalar before mutation. A resource interruption
can retain completed copies and earlier callback effects; no rollback is promised.
BigInt views, shared memory and other unimplemented methods remain separate work.

## Browser example

```sh
./run.sh ./examples/typedarray-copy-within.html
```

The example starts with buffer `99 11 22 33 44 88` and view `11 22 33 44`.
The first click copies right, producing view `11 11 22 33`; the second copies
left, producing `11 22 33 33`. The buffer's two sentinel values remain `99` and
`88`, and the returned object is the same view.

Both direct Page and Linux confined-worker tests perform the real hit-tested
clicks. They check literal DOM and drawn text, title changes, stable button
identity and changed result pixels.

## Complete upstream selection

The `typedarray-copy-within` profile retains all **65 source bodies** at Test262
revision `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`: 41 direct files and 24
BigInt files. Nine original harness dependencies, legal files and the Git tree
proof accompany the import. Both modes remain present for every body.

| Outcome | Before | After |
| --- | ---: | ---: |
| Passed | 4 | 20 |
| Failed | 56 | 0 |
| Resource limit | 6 | 46 |
| Excluded by policy | 64 | 64 |

The four predecessor passes are the two receiver TypeError-only files in both
modes. They do not first require a callable method: reading `.call` from the
missing method supplies the expected TypeError. They are preserved observations,
not evidence of predecessor method support.

All **56 controls** now verify: 32 shared controls and 24 observations from six
independent positive/wrong pairs. Each wrong partner requires its same-mode
positive partner to pass and the intended intrinsic Error. Before implementation,
the shared controls verified but all method controls failed.

The candidate adds 16 passing modes; 40 former failures advance to work-limit
refusals. All 46 resource outcomes are retained, so the **complete report remains
unhealthy**. No upstream baseline or gate is recorded. The exclusions remain
48 BigInt modes, six modes needing the separate resizable-buffer helper, eight
host-hook modes and two immutable-ArrayBuffer modes. No source body, harness or
expectation was shortened to fit the unchanged execution budget.

The complete [before report](../tests/conformance/test262-typedarray-copy-within-before.json)
and [after report](../tests/conformance/test262-typedarray-copy-within-after.json)
retain case identities, source hashes, observations, policy and adapter bindings.

## Enumeration and resource accounting

Adding the method exposed a real resource regression in an existing descriptor
test. The first candidate exhausted its fixed work limit in sloppy mode; its
strict run was not reached. That failed attempt remains in the evidence.

The visited-name set now keeps two names of the same UTF-16 length inline and
allocates a search tree only for a third distinct live name. Equal names retain
the original string payload. Duplicate rejection precedes descriptor lookup;
missing descriptors leave the set unchanged, and hidden properties still shadow
ancestors. Promotion is fully admitted before allocation or publication.

On the tested 64-bit layout, the bucket grows from 32 to 40 bytes. The charged
outer-node allowance rises from 960 to 1,088 bytes per new length. A second name
needs no additional heap; a third admits the existing 576-byte inner allowance.
The established general-tree path and all script quotas remain unchanged.
Programs dominated by unique name lengths therefore pay more charged storage.

With the new method and optimization, the unchanged descriptor test passes in
both modes with **1,781 work units remaining**. Charged heap is 4,464,255 bytes
in sloppy mode and 4,462,285 in strict mode. Against the published predecessor,
that is 540 more remaining work units and 39,155 fewer charged bytes per mode.
These are results for this specific complete witness.

The new intrinsic metadata costs 492 additional startup work units and 2,125
charged heap bytes. The measured realm has 740 objects, 1,120 startup work units
remaining and 1,703,475 charged bytes. A stale initial-admission test expectation
was caught and corrected; its original failure and the exact correction are
retained separately from the production regression.

## Validation and local timings

The final source passes 21 focused method groups, 75 enumeration groups, seven
metadata groups, both browser interaction tests and the full **2,407-test**
Vulkan-enabled suite on Rust 1.88. The default-renderer suite passes **2,250**
tests; the Rust 1.98 script suite passes **1,393**. Both strict Clippy runs,
format checking and all 329 conformance-tool protocols pass.

All **45 prior reports** retain their **18,298 case rows and 4,284 control rows
exactly**, with no prior-pass loss or policy drift. All 38 historical baseline
gates pass. The new 130-mode profile is separate. The historical comparator's
17 protocol groups and the new-profile comparator's 11 groups pass.

The first historical comparison stopped at two changed shared-tool hashes from
the added profile. Its correction pins the reviewed before/after tool bytes,
reconstructs both complete old tool ASTs by reversing only the new registration,
and keeps the other 13 local input rows exact. The original failure is retained.

The isolated benchmark compares the first candidate with the Pair optimization.
Both variants already contain identical `copyWithin` metadata. Eight workloads
run in AB/BA/AB process order, yielding 1,008 retained samples and 5,040 timed
executions: 63 five-call sample means per variant and workload. All eight literal
UTF-16 results are checked before each process starts any timer. Bootstrap,
object setup, warmup, result checking, process startup and rendering are excluded.
Parsing, interpretation, enumeration and output concatenation are included.

Times below are nanoseconds per execute. P95 is the nearest-rank percentile of
the five-call sample means, not an individual-call tail.

| Workload | Before median | After median | Before p95 | After p95 |
| --- | ---: | ---: | ---: | ---: |
| same1 | 7237.8 | 7600.4 | 9474.2 | 17184.6 |
| same2 | 8742.6 | 8856.8 | 10970.8 | 18493.0 |
| same3 | 10403.8 | 10245.4 | 14870.4 | 13477.6 |
| same12 | 23102.0 | 22991.6 | 25271.8 | 24103.8 |
| varied12 | 23331.8 | 22999.6 | 32365.4 | 25891.2 |
| hidden-chain | 15108.4 | 14898.2 | 24424.4 | 15884.2 |
| long2 | 9806.6 | 9551.8 | 12205.0 | 10876.8 |
| long3 | 12251.2 | 12555.8 | 13543.6 | 13633.6 |

Five medians decrease and three increase. The increases are 5.01% for `same1`,
1.31% for `same2` and 2.49% for `long3`. Their p95 changes are +81.38%, +68.57%
and +0.66%, respectively; all other p95s decrease. Every raw sample and tail is
retained. These short local measurements establish neither statistical
significance nor a general speedup.

`same1/2/3/12` use equal-length names; `varied12` uses twelve different name
lengths; `hidden-chain` exercises shadowing across three objects. `long2/3` use
512-unit names with lone surrogates and distinct final units. Both release
binaries use identical locked dependency versions, checksums, toolchain and
features. The raw source inventory correction, formatting provenance and initial
lock-audit/locked-build refusal remain in the evidence.

The [evidence summary](evidence/typedarray-copy-within.json) and
[raw archive](evidence/typedarray-copy-within.tar.gz) bind implementation sources,
complete observations, independent tests, source reviews and retained attempts.
This increment does not establish complete web compatibility, production security
or performance within 30% of Chromium.
