# Local TypedArray foundation observations

The four original files (`cases.js`, `controls.json`, `inventory.json`, and
`README.md`) are exact copies of the independently frozen preparation. Their
historical base and source-only statements describe that original hold. The
portable `manifest.json` binds those bytes; `matrix.json` lists all 86 expected
mode rows before any local baseline or candidate observation.

From the repository root, verify inputs without launching JavaScript:

```sh
python3 tools/typedarray_foundation_local.py --verify-only
```

Root records the baseline against a separately built, explicitly hashed adapter:

```sh
python3 tools/typedarray_foundation_local.py \
  --engine /absolute/path/to/eris-js \
  --engine-sha256 ACTUAL_64_DIGIT_SHA256 \
  --output /absolute/path/to/fresh-local-before.json
```

The output path must be new. A candidate uses the same command and frozen
inputs with its own binary hash and fresh output. This local population remains
separate from the upstream Test262 source/mode population and its exclusions.
There is no feature filtering of the 27 local bodies or 16 control sources.

Each named body runs in a fresh adapter process in both sloppy and strict
modes. The ordinary adapter mode flag applies strict semantics to the whole
source. Every invocation receives the literal `false` or `true` mode argument;
an appended guard requires a literal `true` return. The adapter does not expose
return values in its observation fields, so that requirement is enforced by
the executed source guard rather than by an invented report field.

The runner reuses `test262_conformance.run_case` and its existing three-second
per-request deadline and 96 KiB/64 KiB output/error bounds. All 54 case rows and
32 control rows are retained, including prerequisites that throw TypeError,
unsupported/resource responses, malformed output, timeouts and runner errors.
Raw adapter status remains distinct from local expectation matching. A wrong
control is healthy only if it has the exact runtime intrinsic Error observation
and its positive partner in the same mode succeeds. Authored Error messages
are not assumed to appear in the adapter's diagnostic text.

The runner binds itself, the three shared Python helpers, all local input files
and the actual binary before execution, then checks those same bytes afterward.
Exit 0 requires every case and paired control healthy. Exit 1 retains ordinary
unmet expectations (including an absent feature on the baseline); exit 2 reports
input/binary drift; exit 3 reports an internal runner failure. No baseline or
candidate result is included in this source preparation.
