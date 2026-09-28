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
652 variants: one sloppy and one strict variant per source. Strict variants stay
in the report as unsupported until strict execution is implemented. No test is
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

A thirteen-case preflight runs the upstream assertion and property-helper implementations, including
deliberate failures, signed zero, expected exceptions with matching and differing
constructors, mismatched arrays, and deliberately incorrect writable, enumerable
and configurable descriptors. Failure of this preflight makes the run
unhealthy and prevents recording a baseline.

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

The recorded release measurement contains **265 passed, 5 failed and 382
unsupported variants**, with all thirteen preflights verified and no harness,
resource, timeout or adapter errors. All 326 strict variants are unsupported.
The unchanged `propertyHelper.js` now executes using ordinary property
descriptors, accessors, deletion, `for...in`, and bound functions. Before recording,
the previous feature policy preserved all 108 previous passing cases and added
155; enabling `for-in-order` added two more. All 652 source/mode/harness
fingerprints remain unchanged. The freshly recorded baseline gate passes with
zero regressions. CI runs this gate after building the release adapter.

Ordinary object descriptors support writable, enumerable and configurable data
properties and getter/setter properties. Array indexed/length descriptor
mutation, array extensibility restrictions and host-object reflection are
explicitly unsupported. Strict-mode assignment failures, full lexical binding
rules, proxies, symbols and other exotic objects remain incomplete. Passing
this selected corpus does not establish full ECMAScript conformance.

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
features. This baseline also enables `for-in-order`; the previous-policy gate
and the subsequent feature-policy comparison both preserved all prior passes.
The pinned source inventory remains unchanged.

The pinned data can be reimported deliberately with
`python3 tools/import_test262.py`. A different upstream revision requires editing
the pinned revision and reviewed directory inventory, followed by an explicit
new baseline. Upstream execution rules are preserved in
[`INTERPRETING.md`](../upstream/test262/INTERPRETING.md).
