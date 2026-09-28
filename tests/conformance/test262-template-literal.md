# Pinned Test262 template literal selection

This separate inventory contains every direct JavaScript file in
`test/language/expressions/template-literal` at `tc39/test262` commit
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`: **57 sources and 114 execution
variants**, including 16 parse-negative sources. Both default sloppy and strict
modes run. No test was selected or removed based on an implementation result.
This small selection does not establish full template-literal or ECMAScript
conformance.

The importer preserves exact bytes, including literal CR/CRLF sequences, and
checks the pinned Git blob identifiers. The manifest records every path, byte
length and SHA-256. The runner enforces the complete directory inventory,
requested harness files, source hashes, execution modes and safe local paths.
The existing String/JSON and RegExp inventories keep their own profiles and
unchanged feature policies. A language directory cannot substitute for a
built-ins directory at corpus validation.

```sh
python3 tools/import_test262.py --profile template-literal
cargo build --locked --release --bin eris-js
python3 tools/test262_conformance.py --profile template-literal \
  --baseline tests/conformance/test262-template-literal-current.json
```

Import requires network access; execution uses local test data and the custom
Rust interpreter. The original upstream assertion helpers execute unchanged.
The process framing, negative-error identity/phase checks, source fingerprinting,
timeouts and resource limits described in the [original selection](test262.md)
also apply here. Parse-negative test bodies never execute.

The measured release inventory has **82 passed and 32 unsupported variants**,
with no failed, harness-error, resource, timeout or adapter outcomes. Each mode
has 41 passes and 16 unsupported cases. All **44 preflights** pass: the original
32 assertion/strictness checks plus twelve template checks for nesting, cooking,
coercion order, line normalization, deliberate assertion failures and explicit
tagged-template rejection. Three Python regressions verify the new corpus's
complete inventory, negative modes, preserved line endings, feature-policy
separation and rejection of disabled assertions.

Running the preserved preceding release against this same corpus and feature
policy yielded 34 passed, 60 unsupported and 20 failed variants. All previous
passes are retained, with 48 newly passing variants. The 20 previous failures
are now explicitly classified as unsupported tagged templates; that change is
not credited as successful implementation. All 114 source/mode/harness
fingerprints are unchanged in this comparison. The old release fails the new
template-specific preflight, so its report is diagnostic evidence rather than
a recordable healthy baseline.

The 32 unsupported variants are retained in the report and baseline:

- 28 contain tagged templates, including files mixing tagged and untagged
  checks. Entire file variants remain unsupported; individual passing fragments
  receive no partial credit.
- Two require dynamic `eval`.
- Two declare `numeric-separator-literal`, which this profile does not enable.

The profile permits the original declared features plus `template` and `u180e`.
The former is not currently declared by these sources; `u180e` allows the
Mongolian vowel separator test to run. Other unsupported requirements retain
their ordinary runner/interpreter classification. The default runner command
exits nonzero unless all variants pass; the reviewed baseline command instead
retains every outcome and rejects loss of any previous pass.

See [implementation details and limits](template-literals.md) for the supported
untagged grammar, string-hint conversion, allocation/work bounds and missing
tagged-template call-site objects. Upstream data retains its
[Test262 license](../upstream/test262-template-literal/LICENSE).
