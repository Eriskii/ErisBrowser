# AggregateError upstream results

The complete pinned Test262 AggregateError subtree contains **25 sources and
50 required sloppy/strict modes**. Eris records **44 passed, four failed and
two unsupported**, with all **52 controls verified**. Every selected mode stays
in the denominator. This is a bounded selection, not full ECMAScript conformance.

The [manifest](../tests/upstream/test262-aggregate-error/manifest.json) binds
19 constructor-directory sources, six prototype sources, six unchanged helpers,
two legal/instruction files and seven complete nonrecursive proof documents.
Together with the manifest these are 41 files, pinned to revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`. No source or mode is removed because
of its feature tags or result.

The four failures are both modes of `newtarget-proto-custom.js` and
`newtarget-proto-fallback.js`. These untagged Proxy cases reach the adapter and
report a runtime `ReferenceError`, with intrinsic identity `ReferenceError`
and message `'Proxy' is not defined`. They remain failed cases. Both modes of
`proto-from-ctor-realm.js` remain unsupported because their metadata declares
`cross-realm`; their bodies are not dispatched.

The new profile uses ordinary ERJS1 requests and ERJR2 responses. Only
`aggregate-error` accepts the eighth intrinsic identity, `AggregateError`.
The default decoder, all 57 older profiles and every ERJR3 validator keep the
seven-name policy. The selected decoder rejects ERJR3 before delegation.
This changes no transport bytes or final-unhandled-rejection policy.

The gate combines 32 unchanged ordinary controls with 20 new mode rows covering
metadata, iterable data, the sequence helper, genuine error identity and forged
name/prototype objects. It rebuilds canonical source/helper identities and
checks actual observations, classification and prerequisite links independently.
A supplied verified flag or generic failure is insufficient. Any unhealthy or
malformed gate blocks all 50 cases and baseline recording.

All **592 Python tests pass**, including four new preservation, six importer
and thirteen profile groups. The importer reproduces every byte of the frozen
41-file corpus. Parent profile tests recorded one pass, three failing groups
and nine error groups at the missing-profile boundaries; later checks are not
claimed as reached. A retained parent-control invocation used an unsupported
CLI flag and was corrected before any control ran; literals were unchanged.

The measured baseline and replay retain the exact 50 case and 52 control
records, including the four failures and two exclusions. Both commands exit
successfully with healthy controls; that does not mean every upstream case
passed. The [baseline](../tests/conformance/test262-aggregate-error-current.json)
is the measured status/identity projection, with no manual outcome filtering.

Rust execution inputs are unchanged. The ordinary adapter was built with
Rust 1.88 in release mode, without optional features, from the published
[API checkpoint](aggregate-error.md) at `d3efc9aba871`. The four older Promise
profiles reuse that checkpoint's published `test262-host` adapter. Their full
**1,050 case and 348 control records** match the parent, including failures and
unsupported outcomes; no old policy or baseline changes. This checkpoint makes
no new full Rust-suite or Clippy claim. Binary identities and build logs are
bound in the evidence.

The adjacent ordinary CI gate uses this baseline. To reproduce it after
building the ordinary release adapter:

```sh
python3 tools/test262_conformance.py --binary target/release/eris-js \
  --profile aggregate-error \
  --baseline tests/conformance/test262-aggregate-error-current.json
```

No remote CI result is claimed here. `Promise.any`, Proxy and multiple realms
remain unfinished; these observations establish no production security,
CPU/GPU performance result or process-memory bound. The
[evidence manifest](evidence/aggregate-error-upstream/manifest.json) and
[source/check archive](evidence/aggregate-error-upstream/sources-and-checks.tar.gz)
retain the sources, controls, raw observations and validation receipts.
