# Pinned Test262 isPrototypeOf selection

The [constructor-policy follow-up](constructor-policy.md) admits Reflect call/construct
and `new.target` metadata. Current results: **14 passed, 6 unsupported**.
The inventory and source bytes are unchanged. Measurements and policy descriptions
below retain the history of earlier checkpoints.

This separate profile imports every direct `.js` file in
`test/built-ins/Object/prototype/isPrototypeOf` at Test262 revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`. All **10 source tests**, six unchanged
harness files, `LICENSE` and `INTERPRETING.md` are retained. The importer checks
each fetched test against its pinned Git blob. No test or harness source is
rewritten, abbreviated or removed because its APIs are unavailable.

The complete inventory has **20 variants**, ten sloppy and ten strict. There
are no fixture files or negative-metadata tests. The manifest records original
paths, byte sizes and SHA-256 identities; the runner verifies every source,
required mode, metadata record and requested harness before execution.
Manifest SHA-256:
`f019dcc5e150cf14a6b144577a30937ed1314c6b5de3dd331ad574585f16d461`.
The corpus retains its [upstream license](../upstream/test262-is-prototype-of/LICENSE).

```sh
python3 tools/import_test262.py --profile is-prototype-of
cargo build --locked --release --bin eris-js
python3 tools/test262_conformance.py --profile is-prototype-of
```

The profile uses the existing core String/JSON feature policy unchanged. It
does not enable `Proxy`, `Reflect.construct` or `Symbol`: their five source
tests remain ten explicit unsupported variants. Both Reflect-tagged files
remain excluded by their original metadata, even where a body contains checks
that might separately execute. Tests are never sliced into passing fragments.
The old String/JSON, RegExp, template, function and rest profiles are unchanged.

The five admitted sources cover `length`, `name`, null/undefined receivers with
object arguments, and a constructor-created prototype chain. Their actual
adapter outcomes determine their status; admission is not a support claim.
The shared [runner contract](test262.md) still requires fresh bounded processes,
unchanged assertions, intrinsic error identities, stable binary hashes and
complete source/harness/mode fingerprints.

## Assertion preflight

The profile retains the **32** core assertion and strict-semantics checks and
adds **32 method checks**, 16 in each mode. Paired positive and deliberately
wrong assertions exercise:

- Prototype-chain identity and exclusion of the argument object itself.
- A primitive argument returning false before null/undefined receiver conversion.
- Intrinsic `TypeError` for a null/undefined receiver with an object argument.
- Internal prototype traversal without invoking `__proto__`, `prototype`,
  `valueOf` or `toString` user properties.
- Fresh primitive boxing identity, array/function prototype chains and the
  method's native function metadata and nonconstructibility.

These expectations follow the current
[ECMAScript isPrototypeOf algorithm](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.prototype.isprototypeof).
The positive and bad checks execute through unchanged upstream `assert.js`,
`sta.js` and, where requested, `propertyHelper.js`. A deliberately incorrect
expectation must produce `Test262Error`; unrelated exceptions do not satisfy it.
All **64 checks** must verify before a healthy regression baseline can be
recorded. No Proxy trap or Reflect behavior is claimed by these checks.

Four focused Python regression groups verify the exact inventory and modes,
the separate unchanged core feature policy, preservation of earlier preflights,
rejection of disabled assertions/wrong errors, and pinned-blob/inventory checks
during import. Existing shared baseline, timeout, resource and integrity tests
also apply.

## Initial measurement

The [complete initial report](test262-is-prototype-of-initial.json) uses the
frozen adapter from the identifier-rest implementation, before isPrototypeOf
was added. Adapter SHA-256:
`f7303916323638bd7965ffbffeee4afd27e8f630bed6c8357a8a51fc10621b56`.
Source-input digest:
`a61c16761f309ea4e51f42b44ef2bd3471a04d1b5c8843a4d8ec15b4fc0d191c`.

| Outcome | Variants |
| --- | ---: |
| Passed | 4 |
| Failed | 6 |
| Unsupported | 10 |
| Resource stop / timeout / adapter error | 0 |

All 32 earlier preflights verify; only four of the 32 method checks verify.
The four upstream passes are the receiver-error variants: calling a missing
method also raises `TypeError`, so those passes alone demonstrate no method
support. The complete method preflight fails, the runner exits unsuccessfully,
and **no healthy baseline is recorded**. The full report preserves every
observation and source/harness/mode fingerprint for an implementation comparison.

## Implemented-method checkpoint

The [complete updated report](test262-is-prototype-of-latest.json) records
**10 passed / 10 unsupported**, with all **64 preflights verified** and no
failure, resource stop, timeout or adapter error. Every source/harness/mode,
metadata, policy and preflight fingerprint matches the initial measurement.
Six variants newly pass: both modes of `length.js`, `name.js` and
`this-value-is-in-prototype-chain-of-arg.js`. All four earlier passes remain.
The unchanged Proxy, Reflect and Symbol exclusions retain all ten unsupported
variants in the report and denominator.

After verifying these comparisons, the runner recorded a
[regression baseline](test262-is-prototype-of-current.json). A subsequent CLI
baseline check completed successfully with zero regressions and zero additional
changes. The standalone non-baseline run exits 1 because unsupported variants
remain; recording/checking the healthy regression baseline exits 0. A healthy
baseline indicates reliable execution and preserved results, not full support.

```sh
python3 tools/test262_conformance.py --profile is-prototype-of --baseline tests/conformance/test262-is-prototype-of-current.json
```

The measured adapter SHA-256 is
`566886403cd490547bf5475078b60f68fdc0a5d5c5f3007d36ee1179a50f6e0c`.
Its source-input digest is
`ebcdac762343020bcb6ba88090738526933567fdf496d0468c204706b71f9a17`.
The execution-policy SHA-256 remains
`81b61b3419f5a73997b1f29d352177e4847797cf9c8d8fcc353a52c0acb6be2e`.
The [implementation record](is-prototype-of.md) describes the supported internal
prototype graph, primitive boxing, metadata and shared work/depth limits.

This is one complete upstream directory, not full Object or ECMAScript
conformance, and does not execute the WPT browser harness.
