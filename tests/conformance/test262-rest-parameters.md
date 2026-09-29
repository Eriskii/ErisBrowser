# Pinned Test262 rest-parameter selection

The [constructor-policy follow-up](constructor-policy.md) admits Reflect call/construct
and `new.target` metadata. Current results: **16 passed, 6 unsupported**.
The inventory and source bytes are unchanged. Measurements and policy descriptions
below retain the history of earlier checkpoints.

This separate profile imports every direct `.js` file in
`test/language/rest-parameters` at the existing Test262 revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`. All **11 source tests**, four unchanged
harness files, `LICENSE` and `INTERPRETING.md` are retained. No source is edited
or omitted because its syntax is unimplemented.

The inventory requires **22 variants**: 11 sloppy and 11 strict. There are no
fixture files. `position-invalid.js` contributes two parse-negative variants,
both requiring an intrinsic `SyntaxError`. The old upstream frontmatter has no
feature labels for these files, so the adapter receives every variant, including
`array-pattern.js`, `object-pattern.js` and `with-new-target.js`. An unsupported
language construct must remain an explicit unsupported result, not be counted
as a negative-test pass.

The importer validates each fetched test against its pinned Git blob identity.
The manifest records original paths, byte sizes and SHA-256 values; the runner
checks the complete source inventory, required modes, execution metadata,
harness bytes and case fingerprints before execution. Manifest SHA-256:
`a3e15e061fc8c9cac736e081554f3e42b497d805ebfd7647ea49736cb2d83be5`.
The corpus retains its [upstream license](../upstream/test262-rest-parameters/LICENSE).

```sh
python3 tools/import_test262.py --profile rest-parameters
cargo build --locked --release --bin eris-js
python3 tools/test262_conformance.py --profile rest-parameters
```

The profile's allowed declared features are exactly the
[function profile](test262-functions.md)'s feature set plus `rest-parameters`.
The existing function, String/JSON, RegExp and template policies are unchanged.
This addition does not grant execution support for destructuring, `new.target`,
`eval`, async/generator execution or host hooks. Permission to execute a case
is not a support claim; the adapter's observed result determines its status.
The [runner contract](test262.md) specifies fresh bounded processes, unchanged
assertion helpers, strict modes, negative phase/type checks and outcome classes.

## Assertion preflight

The profile preserves the existing **48** core, strict-semantics and
function/default-parameter checks, then adds **16 rest checks**, eight per mode.
Positive checks exercise fresh arrays, empty rest arrays, positional offsets,
explicit `undefined`, unmapped arguments and function length. Deliberately wrong
array, offset, aliasing and length expectations must throw the unchanged
upstream `Test262Error`; an adapter that disables assertions cannot pass.
All **64 checks** must verify before this profile can record a healthy baseline.

Four focused Python regression groups verify the complete inventory, the exact
feature-set delta, retention of earlier preflights, rejection of disabled
assertions and pinned-blob verification during import. The shared runner's
negative, mode, resource, source-integrity and baseline guards also apply.

## Initial measurement

The [complete initial report](test262-rest-parameters-initial.json) uses the
frozen pre-rest adapter from engine checkpoint
`ed7dcb830a6b92a9f606cccd2c1c320155cd70da`. Its SHA-256 is
`77003eefd7a91dccb2b418e0d59bc234091edbfb1a7bb6a217de63cd1ece8b69`;
the recorded source-input digest is
`e6d4d857c922db8942abaec7cf230454b3487313f1121dbc6e7babbe0385bcd6`.

| Outcome | Variants |
| --- | ---: |
| Passed | 2 |
| Unsupported | 20 |
| Failed | 0 |
| Resource stop / timeout / adapter error | 0 |

The two passes are the parse-negative variants. All 48 earlier preflights verify,
while all 16 new rest preflights report unsupported. The overall preflight
therefore fails, the runner exits unsuccessfully and **no healthy baseline is
recorded**. The report retains every observation and source/harness/mode
fingerprint for later comparison; test bodies and execution policy must remain
unchanged when measuring implementation progress.

## Identifier-rest checkpoint

The [complete updated report](test262-rest-parameters-latest.json) records
**16 passed / six unsupported**, with all **64 preflights verified** and no
failure, resource stop, timeout or adapter error. Identical source/harness/mode
fingerprints and execution policy give **14 new passes and no lost passes**.
The six unsupported variants retain array-pattern, object-pattern and class
syntax; they remain in the inventory and denominator.

The runner records a [regression baseline](test262-rest-parameters-current.json)
that preserves those outcomes. GitHub CI checks this profile independently of
the existing function selection, whose policy still excludes declared rest
features. A healthy regression baseline means the runner and assertions execute
reliably, not that every retained language feature works.

```sh
python3 tools/test262_conformance.py --profile rest-parameters --baseline tests/conformance/test262-rest-parameters-current.json
```

The measured adapter SHA-256 is
`f7303916323638bd7965ffbffeee4afd27e8f630bed6c8357a8a51fc10621b56`.
Its source-input digest is
`a61c16761f309ea4e51f42b44ef2bd3471a04d1b5c8843a4d8ec15b4fc0d191c`.
The [implementation record](rest-parameters.md) describes exact grammar,
initialization behavior, allocation limits and remaining gaps.

This is one complete upstream directory, not full Test262 coverage, an executable
WPT harness, or a claim of complete rest-parameter or web compatibility.
