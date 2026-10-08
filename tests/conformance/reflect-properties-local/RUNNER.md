# Local Reflect runner

The frozen population contains 14 named bodies in two modes (28 case rows) and
five positive/wrong pairs in two modes (20 control rows), for 48 retained rows.
The fixtures are exact copies of the independent hold. The manifest binds their
seven source/document files and published foundation commit
`b4ce6570f42ab4f45862cc29445a4181d5dafec5`. Original context documents retain
their earlier pending-base wording as history; the manifest is the actual base.

Each case request contains the whole `cases.js` file followed by its literal
zero-argument invocation. A result other than the Boolean true throws Error.
The shared adapter applies strict or sloppy mode to the complete request, so
the named function bodies inherit that mode. Every request uses a fresh adapter
process; there is no shared realm or state between cases or controls.

The `matrix.json` rows bind the exact complete request source, mode, metadata,
empty harness and expected observation. The new runner adapts the published
TypedArray local runner's identity/population and frozen schema; it does not
change that earlier population or the shared supervisor.

After integration, root can verify inputs without launching an adapter:

```sh
python3 tools/reflect_properties_local.py --verify-only
```

The actual baseline requires an explicitly supplied immutable adapter digest
and a fresh output path:

```sh
python3 tools/reflect_properties_local.py \
  --engine /absolute/path/to/held-eris-js \
  --engine-sha256 ACTUAL_64_CHARACTER_SHA256 \
  --output /absolute/path/to/new-reflect-before.json
```

The runner hashes its own source, all three shared helper files, all fixture
inputs, manifest, matrix and this document before importing helpers. It checks
those bindings again after matrix preparation, verifies the supplied binary,
opens output exclusively before process launch, then retains every row including
exceptions, resource refusals, timeouts and adapter/runner errors. Every request
keeps the inherited three-second deadline, bounded stdout/stderr and process
cleanup. After all rows it rehashes inputs and binary and retains any drift.

A case needs successful runtime completion after its true-return assertion.
A wrong control needs the exact intrinsic runtime Error type and identity,
and its same-mode positive partner must complete successfully. A missing method
producing TypeError cannot verify a wrong partner. No authored Error.message
matching is assumed. Return codes are zero only for all verified rows, one for
semantic mismatch, two for binding drift, and three for runner errors.

The preparation authored source and reconstructed the matrix as inert data.
It did not import the runner/helpers, invoke `--verify-only`, run an adapter,
compile the project or generate a result report. Root owns those executions.
