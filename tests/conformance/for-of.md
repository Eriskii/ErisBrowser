# Synchronous iteration checkpoint

Eris now executes synchronous `for…of` loops with identifier or member targets,
`var`, `let` and `const` bindings, custom iterator protocols, and native Array,
String and arguments iterators. Array `values`, `keys` and `entries` use live
length and indexed properties. String iteration preserves UTF-16 surrogate pairs
and lone surrogates. Iterator brands and progress remain private even when the
ordinary iterator object is frozen.

The flat parser and execution driver preserve RHS temporal dead zones, fresh
lexical bindings per iteration, cached `next`, live `return`, target evaluation
order, completion values and labelled exits. Iterator closing distinguishes
step failures from assignment/body departures and preserves ordinary throw
precedence. Terminal resource or unsupported stops remain uncatchable and skip
author cleanup. Existing limits are unchanged; [allocation and work evidence](for-of-limits.json)
is separate from any security guarantee.

The [independent local fixture](for-of.js) was frozen before implementation:
202 sources, 404 sloppy/strict modes, 48 paired feature controls and twelve
assertion controls. Its [original expectations](for-of-fixture.json) are unchanged.
The [published-before observations](for-of-initial.json) contain 36 incidental
parse-negative passes and 368 failures; none of the 48 feature controls verify.
The [candidate observations](for-of-final.json) contain 380 passes, twelve expected
terminal resource stops and twelve unsupported prerequisite modes. Thus **392
of 404 expectations verify**, and all 60 controls verify. The twelve unmet
expectations remain visible: four destructuring cases, one generator case and
one async-declaration case, each in two modes. They were not removed or relabelled.

Two complete pinned inventories add separate evidence:

| Profile | Sources | Modes | Passed | Failed | Unsupported | Verified controls |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| [for-of](test262-for-of.md) | 751 | 1,442 | 137 | 2 | 1,303 | 80/80 |
| [Core iterators](test262-core-iterators.md) | 76 | 144 | 106 | 0 | 38 | 132/132 |

The two failures need the untagged `Proxy` prerequisite. For-of retains 1,253
metadata exclusions and 50 runtime unsupported outcomes. Nine previous
incidental parse-negative passes now report unsupported syntax; complete early
errors for unavailable declarations and destructuring are still missing.
These are recorded explicitly, including their original observations.

All 36 older profiles retain their source, mode, policy and control identities.
Their 16,146 modes gain exactly four Date year-zero passes, lose none, and have
no other changed observations. All 3,592 older controls remain identical and
verified; all 29 existing baseline gates pass. Only the four Date passes
strengthen an existing baseline. The two new gates preserve observed passing
cases while keeping failures and unsupported cases in their inventories.

The [validation record](for-of-integration-validation.json) binds the inputs
frozen before building and all eight release binaries. Rust 1.88 and 1.98 each
pass strict Clippy, 1,168 default tests and 1,179 presenter-feature tests, with
no failed or ignored tests. All 234 Python tests pass. Both releases pass 57
pixel references with separately retained output. Six document samples also
check initial execution and click callbacks through the confined worker.
One deterministic mutation run covers 15,000 cases without a caught panic or
invariant failure. The unchanged DOM adapter bytes justify retaining the prior
HTML observations without another run.

Initial private-test setup failures, a test-only legacy-parser stack regression,
and a Date pipe-test readiness assumption are retained with their corrections.
The pipe test now proves readable data with a duplicate writer still alive,
then waits for actual hangup under its original deadline. Production timezone
code and all quota definitions remain unchanged.

[Provenance and limitations](for-of.json) bind adapter
`66bf753b598273161e6cc88f5004d83b4d5d1f88e2919e96db2414ccc2cc1316`
to input digest
`cd50007f53a7b9431c104be33996295c2e4da96440963637f3eef20a02aedfa8`.
This checkpoint does not establish full ECMAScript or web compatibility,
security, or Chromium performance parity. Destructuring, generators, async
iteration, Proxy, typed arrays and broader iterator consumers remain ahead.
