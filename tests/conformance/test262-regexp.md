# Pinned Test262 RegExp selection

The [constructor-policy follow-up](constructor-policy.md) admits Reflect call/construct
and `new.target` metadata. Current results: **262 passed, 28 unsupported**.
The inventory and source bytes are unchanged. Measurements and policy descriptions
below retain the history of earlier checkpoints.

The [general Window binding follow-up](window-global-bindings.md) raises this
profile to **252 passed / 38 unsupported**. Both detached `toString` modes now
pass because their throwing global accessor setup is supported. Source inventory
and policy are unchanged.

This separate selection measures Eris's custom bounded RegExp implementation
against unchanged Test262 sources and harness files. It does not replace the
[String/JSON selection](test262.md), and it is not full ECMAScript or RegExp
conformance.

The import uses the same pinned `tc39/test262` revision,
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`, and contains every direct JavaScript
file in these `test/built-ins/` directories:

| Directory | Files |
| --- | ---: |
| `RegExp/prototype/exec` | 79 |
| `RegExp/prototype/test` | 45 |
| `RegExp/prototype/toString` | 9 |
| `RegExp/prototype/source` | 12 |

All 145 files produce both default execution modes: **290 variants**. No file
was omitted based on an implementation result. The importer verifies pinned
Git blob identifiers, and the runner verifies the complete directory inventory,
all byte lengths and SHA-256 hashes, requested harness files, and path safety.
The original 326-file, 652-variant corpus remains a separate integrity boundary.
Upstream files, assertions and test bodies are not rewritten.

```sh
python3 tools/import_test262.py --profile regexp
cargo build --locked --release --bin eris-js
python3 tools/test262_conformance.py --profile regexp \
  --baseline tests/conformance/test262-regexp-current.json \
  --output artifacts/test262-regexp-gate.json
```

Import requires network access. Execution and baseline checking use local
sources. The same framed adapter, strict-mode implementation, constructor
identity checks, process limits and failure classifications described in the
String/JSON selection apply here.

The release baseline contains **250 passed and 40 unsupported variants**,
or 125 passed and 20 unsupported in each mode. There are no failed,
harness-error, resource, timeout or adapter outcomes in this measurement.
All 44 preflights pass: the original 32 plus twelve RegExp checks covering
successful execution, captures/indices/lastIndex, String integration, assertion
rejection of incorrect matches/captures/lastIndex, and explicit unsupported
Unicode mode in both execution modes. Baseline checking retains every case
fingerprint and rejects the loss of any previously passing case.

This profile permits the original declared features plus `regexp-dotall`,
`regexp-match-indices`, `regexp-named-groups` and `regexp-sticky`. Other declared
features remain explicitly unsupported; undeclared unsupported syntax or
runtime operations must be reported by the interpreter. The current remainder
includes Unicode `u`/`v` modes, duplicate named groups, dynamic eval, Reflect,
Symbol/cross-realm behavior, host-property reflection and conservatively gated
declared exponentiation. These results are included in the denominator.

The implementation uses UTF-16 code units without a regex or JavaScript engine
dependency. It supports literals and construction, `d/g/i/m/s/y`, ordered
alternatives, character classes, anchors/word boundaries, greedy/lazy bounded
and unbounded quantifiers, captures, ASCII named groups, numeric/named
backreferences and positive/negative lookahead. `exec`/`test`, `lastIndex`,
source/flag accessors and match index arrays use ordinary runtime properties.
String `match`, `search`, `replace` and RegExp `split` have focused implementation
tests, including empty matches, unmatched captures, replacement substitutions
and callbacks; this upstream inventory does not cover their entire suites.

Matching uses an explicit backtracking stack. Conservative two-code-unit
prefix filtering reduces failed-start work; a separate test compares optimized
and unoptimized capture results across generated pattern/input combinations.
Long binary chains use bounded flat evaluation, and literal string addition
chains can be folded without creating every intermediate string. The imported
sources remain unchanged.

Patterns are bounded to 8,192 UTF-16 units, 4,096 syntax nodes and 128 captures.
Parser/assertion nesting respects explicit limits and the available runtime
stack allowance. Compilation, matching, output growth and callback execution
consume work and cumulative allocation budgets. Adversarial backtracking and
recursive replacement callbacks terminate with uncatchable resource errors;
limits are not evidence of linear-time matching or a security audit.

Unicode modes/properties/sets, lookbehind, modifier groups, duplicate/non-ASCII
capture names, several Annex B escape forms, Symbol dispatch/species, complete
RegExp statics and the rest of the standard remain incomplete. Unsupported
syntax is reported explicitly. No broad compatibility or performance claim
follows from this small corpus.

The primary algorithms used for implementation and review are the current
[ECMAScript RegExp specification](https://tc39.es/ecma262/multipage/text-processing.html#sec-regexp-regular-expression-objects),
[RepeatMatcher](https://tc39.es/ecma262/multipage/text-processing.html#sec-runtime-semantics-repeatmatcher-abstract-operation),
[Canonicalize](https://tc39.es/ecma262/multipage/text-processing.html#sec-runtime-semantics-canonicalize-ch),
and [EscapeRegExpPattern](https://tc39.es/ecma262/multipage/text-processing.html#sec-escaperegexppattern).
