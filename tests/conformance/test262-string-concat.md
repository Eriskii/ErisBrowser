# Pinned Test262 String concat profile

This profile imports every direct `.js` file in
`test/built-ins/String/prototype/concat` at Test262 revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`: **22 sources / 44 modes**. The complete
nontruncated directory tree is `f29ea44d5643132cf2f726ed726ce1012b23c2f5`.
Source Git blob hashes were verified before import. The manifest records exact
source/harness bytes and SHA-256 hashes; the importer refuses inventory or blob
mismatches. No browser or JavaScript engine implementation is imported.

The [implementation](string-concat.md) records **42 passed / two unsupported**.
The two excluded modes retain the original `not-a-constructor.js` source and
metadata. Its `isConstructor.js` helper requires unimplemented `Reflect.construct`.
The profile uses the existing default feature policy; no other profile policy
is expanded. Direct `new` rejection is checked separately by local tests.

The frozen preceding binary records 40 failed, two passed and two unsupported
modes under exactly the same source inventory and policy. The two passes are
legacy construction-rejection modes (`S15.5.4.6_A7.js`) that accept any exception
other than Test262Error. A missing method also passes that check, so those modes
alone do not establish implementation. Its feature-aware preflight
fails (34 of 56 controls verify), so that report is not a healthy baseline.
The new binary verifies all 56 controls. Deliberately wrong results must fail
with the upstream Test262Error identity, preventing a missing method's TypeError
from satisfying those failure controls.

```sh
python3 tools/import_test262.py --profile string-concat
python3 tools/test262_conformance.py --profile string-concat \
  --baseline tests/conformance/test262-string-concat-current.json
```

CI preserves the passing-case baseline and the complete mode inventory. The
[comparison](string-concat.json) retains exact before/after cases, controls,
policies and hashes. This is one complete builtin directory, not full Test262 or
web-platform conformance.
