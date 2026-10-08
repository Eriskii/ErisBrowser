# Local TypedArray view/string runner

The runner executes the separately frozen 23 bodies in both modes and six
positive/wrong pairs in both modes: 46 case rows and 24 control rows. It does
not run the upstream Test262 profile or change its population. The fixture,
controls, original matrix, manifest and preparation notes are copied byte-exact.
`freeze.py` is retained authoring history, not a portable execution entry point.

```sh
python3 tools/typedarray_views_local.py --verify-only
python3 -m unittest discover -s tools -p 'test_typedarray_views_local.py'
python3 tools/typedarray_views_local.py \
  --engine /absolute/path/to/eris-js \
  --engine-sha256 EXACT_SHA256 \
  --output /absolute/path/to/fresh-report.json
```

Source verification and mocked protocol tests do not launch an adapter. An
observation run requires the caller's exact binary hash and exclusively creates
its output before launching requests. It uses the existing bounded process
supervisor, three-second per-request deadline, ERJS1 request and ERJR2 response
validation. Shared helper bytes and all local inputs are hashed before import
and checked again after every row has been collected; the binary is checked
before execution and again afterward. Case bodies have a true-return assertion.

The complete report retains every case and control, including resource stops,
exceptions, unsupported syntax, timeouts and adapter errors. Resource outcomes
are never matched as successful negative controls. A wrong control additionally
requires a successful positive partner in the same mode; missing APIs cannot
establish health. The runtime's intrinsic error identity is checked, not its
diagnostic message text. Matched wrong controls keep their raw `failed` status
and have a separate healthy-control flag.

Exit 0 requires every case and paired control to match with unchanged inputs.
Exit 1 retains semantic/unhealthy outcomes; exit 2 records post-run binding
drift; exit 3 records internal runner/identity errors. Before-run input errors
raise without starting the adapter. Existing outputs are never overwritten.

The mocked tests cover the 70-row contract, malformed/tampered inputs, absent
method false positives, positive-partner failure, terminal resource results,
fresh-output/binary admission, late source/binary drift, complete exception
retention and verification without execution. Preparation itself supplies no
engine result, conformance score or performance/security guarantee.
