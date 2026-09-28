# Pinned Test262 String and JSON selection

This runner measures Eris's own bounded JavaScript interpreter against unchanged
upstream test bodies and harness code. It is a selection of Test262, not complete
ECMAScript conformance and not a benchmark against another engine.

The import is pinned to `tc39/test262` commit
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`. It contains every direct `.js` file
from these nine `test/built-ins/` directories:

| Directory | Files |
| --- | ---: |
| `JSON/parse` | 77 |
| `JSON/stringify` | 66 |
| `String/prototype/charAt` | 30 |
| `String/prototype/charCodeAt` | 25 |
| `String/prototype/codePointAt` | 16 |
| `String/prototype/slice` | 38 |
| `String/prototype/substring` | 46 |
| `String/fromCharCode` | 17 |
| `String/fromCodePoint` | 11 |

All 326 files lack execution flags, so the required default inventory contains
652 variants: one sloppy and one strict variant per source. Both modes execute.
The adapter selects a strict parse/execution goal equivalent to the required
leading strict directive while preserving the imported source bytes. Harness
sources retain their own directives and are evaluated separately. No test is
removed because of its result. This selection contains no frontmatter-negative
tests; parse/runtime negative handling is covered by runner and adapter
regressions. `_FIXTURE` files, if added in a future selection,
are retained as fixture inventory and are not executed as standalone tests, as
required by upstream.

`tests/upstream/test262/manifest.json` records every upstream path, byte length
and SHA-256, plus the directory inventory. `LICENSE`, `INTERPRETING.md`, and all
five requested harness files are included. The importer also compares downloaded
test bytes with GitHub's pinned Git blob identifiers. It does not change source
bytes or line endings. The runner checks the complete inventory, all hashes,
unlisted files and path safety before starting an adapter.

```sh
cargo build --locked --release --bin eris-js
python3 tools/test262_conformance.py
python3 -m unittest discover -s tools -p 'test_test262_conformance.py'
```

Each executable variant starts a new `eris-js` process and a fresh Runtime. The
Python runner bounds input/output and wall-clock time with nonblocking pipes;
the Rust adapter bounds frame sizes and, on Linux, address space. The Runtime's
existing source, token, recursion, instruction and allocation limits remain in
force. The adapter is a test executable; it does not load a browser engine or
expose filesystem, network, process or host-eval functions to scripts.

The adapter parses the test source before evaluation. Parse-negative tests are
never evaluated. For ordinary execution it evaluates the exact upstream
`assert.js`, then `sta.js`, then declared includes in their specified order, in
the same global environment as the test. Each source is passed separately, so a
harness exception cannot satisfy a negative test. `raw` mode would use no
harness. No native assertions, assertion replacements, source-body rewrites or
result normalization are used.

Negative built-in errors use the interpreter's intrinsic error identity and the
reported parse/runtime phase. Writable diagnostic names cannot satisfy this
check: a user constructor renamed to `TypeError` is rejected, while renaming the
real `TypeError` constructor does not change its identity. Ordinary objects with
a `constructor` property cannot impersonate an intrinsic error. Negative tests
requesting other constructors are explicitly unsupported until their identities
can be checked. The framed adapter response keeps identity separate from the
diagnostic name.

A 32-case preflight runs the original thirteen assertion/property-helper checks
in both sloppy and strict modes, then checks strict receivers, unresolved writes,
readonly properties, deletion, arguments and lexical initialization. The unchanged
upstream helpers must accept successes and reject deliberate failures, signed-zero
mismatches, wrong exception constructors, mismatched arrays and incorrect
property descriptors. Failure makes the run unhealthy and prevents recording a
baseline.

The current declared-feature support policy permits `arrow-function`,
`String.fromCodePoint`, `well-formed-json-stringify`, and `for-in-order`. Other declared features
are explicitly unsupported. In particular, `isConstructor.js` catches failures
from `Reflect.construct`: running its tests without Reflect could accidentally
pass negative constructor assertions, so those cases are gated as unsupported.
Modules, asynchronous completion, agent blocking policy, locale requirements,
and Test262 host hooks are unsupported. A conservative source check retains
tests mentioning `$262`, `$DONE` or `print(...)` as unsupported; it can also
match a mention inside a comment or string. These limitations are included in
the denominator. Unsupported grammar or semantics encountered during execution
remain ordinary failures unless the interpreter explicitly identifies them as
unsupported; they are never inferred from a matching error message.

The report distinguishes:

- **passed**: successful ordinary execution, or a negative test with both the
  expected intrinsic error identity and the expected parse/runtime phase;
- **failed**: a test exception, missing expected exception, wrong exception type,
  or wrong phase;
- **harness-error**: an include failed before test execution;
- **unsupported**: an execution mode, declared feature, host requirement, or
  interpreter-reported feature is unavailable;
- **resource**: an explicit interpreter budget was exceeded;
- **timeout**: the parent killed an adapter after its deadline;
- **adapter-error**: process failure, malformed response or transport error.

The default command exits nonzero unless every mode case passes. A reviewed
baseline can gate incremental work while preserving the full failure inventory:

```sh
python3 tools/test262_conformance.py --record-baseline tests/conformance/test262-current.json
python3 tools/test262_conformance.py --baseline tests/conformance/test262-current.json
```

The recorded release measurement contains **534 passed and 118 unsupported
variants**, with all 32 preflights verified and no failed, harness, resource,
timeout or adapter outcomes. Each execution mode has 267 passes and 59
unsupported cases. The RegExp increment preserved all 532 previous passes,
all 652 source/mode/harness fingerprints and the original runner policy, then
added both variants of the existing JSON RegExp-object case. The recorded
baseline gate has zero regressions. CI runs it after building the release
adapter.

Unsupported outcomes retain their declared-feature or interpreter reasons,
including Reflect, Proxy, Symbol, BigInt, complete JSON source-context semantics,
computed object keys, dynamic eval/Function construction and array descriptor mutation. A separate
[RegExp prototype selection](test262-regexp.md) adds its own unchanged corpus,
mode inventory, report and baseline without replacing this selection. The separate
[template literal selection](test262-template-literal.md) likewise retains every
source in its pinned language directory, including tagged-template cases that
remain unsupported.

Strict code retains exact raw directives, inherits strictness lexically, checks
restricted bindings/assignments and duplicate simple parameters, rejects legacy
literals and identifier deletion, preserves the supplied receiver, and throws
on unresolved or failed property writes. Strict arguments are unmapped with a
restricted callee accessor; supported sloppy arguments map indices to parameters
until deletion or descriptor changes detach them. Lexical bindings include
initialization checks, declaration conflicts and per-iteration loop environments.
The implementation remains a subset: classes/modules, destructuring/default/rest
parameters, dynamic eval/Function, full Unicode regular expressions, labeled control flow,
complete Annex B behavior and other grammar/runtime features are absent.

Ordinary object descriptors support writable, enumerable and configurable data
properties and getter/setter properties. Array indexed/length descriptor
mutation, array extensibility restrictions and host-object reflection are
explicitly unsupported. Proxies, symbols and other exotic objects remain
incomplete. Passing this selected corpus does not establish full ECMAScript
conformance.

Recording and checking are mutually exclusive. A baseline binds the upstream
revision, manifest bytes, runner feature policy and timeout, and every case's
original source, mode, metadata and ordered harness hashes. Missing cases,
changed identities, newly broken passing cases, failed assertion preflights and
resource/timeout/adapter errors fail the gate. An explicit record operation does
not write a baseline when the run is unhealthy. Includes that cannot execute are
retained as `harness-error` limitations, distinct from transport errors.
Report output cannot overwrite the baseline, including through a symlink alias.
The adapter executable's hash must remain unchanged throughout the measurement.
Runner policy format 2 records intrinsic error identities and supported declared
features. This baseline enables strict execution in addition to `for-in-order`;
the previous-policy gate and subsequent explicit policy comparison preserved all
prior passes. The pinned source inventory remains unchanged.

The pinned data can be reimported deliberately with
`python3 tools/import_test262.py`. A different upstream revision requires editing
the pinned revision and reviewed directory inventory, followed by an explicit
new baseline. Upstream execution rules are preserved in
[`INTERPRETING.md`](../upstream/test262/INTERPRETING.md).
