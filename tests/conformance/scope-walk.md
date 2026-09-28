# Bounded borrowed declaration traversal

Declaration-name collection and runtime var hoisting now share an iterative
preorder walk over borrowed statements. A fixed array holds at most 96 ancestor
cursors; roots have a separate iterator. Wide blocks and switches do not grow
that storage. The walk visits loop initializers/bodies, both branches, labeled
bodies, switch cases and try/catch/finally in source order. It excludes function
bodies and expressions, preserving declaration scope boundaries.

Each cursor movement consumes the caller's compile/runtime work before advancing.
Empty switch case lists also consume work, including precharges for flattened
root scans. A failed precharge or depth overflow makes the walker terminal.
Traversal allocates no heap storage and clones no owned syntax. Parser/evaluator
recursion and syntax ownership remain separate work; their limits are unchanged.

Switch scope validation now passes borrowed statements from all cases instead
of constructing a recursively cloned combined body. Parsing records whether each
scope has direct lexical declarations, including block-level function declarations
where applicable. Without them, lexical/var conflict analysis is unnecessary and
is skipped. Otherwise duplicate checks and the first sorted conflict diagnostic
retain their existing order. Runtime hoisting preserves owner scope, unreachable
declarations, global lexical conflicts and validation-versus-insertion behavior.

An initial attempt charged new analysis passes over scopes without lexical names.
The preserved 8,327-declaration Unicode 10 identifier test then hit the compile
work limit. Recording lexical presence during parsing removes those actual scans
and temporary name-set construction; the unchanged test passes again in both
modes. No quota or source filter was changed. A subsequent review added the empty
root-case precharge before final validation. Preliminary checks and the failure
log remain local evidence.

Four focused Rust groups cover traversal/hoisting order, function boundaries,
96/97-depth refusal, terminal work failures, empty cases, 16,000-statement width,
mutation refusal at zero work, lexical scope conflicts and diagnostic ordering.
The existing nine completion groups, including 240 direct strict/sloppy scalar
cases and recursive statement forms, also pass.

The [comparison record](scope-walk.json) preserves all **20 Test262 profiles /
6,879 modes / 1,680 controls**, including every case and preflight fingerprint,
policy and observation. All controls verify and fifteen healthy gates pass.
Existing resource stops remain nonpassing; no baselines were re-recorded. HTML
retains 3,868 matches, two mismatches and six unsupported modes.

Final validation passes **911 Rust tests**, **151 Python checks**, **57 exact
pixel references** and **15,000 deterministic mutation cases**, plus formatting,
strict all-target Clippy and release compilation. No independent-agent review,
native-window, Vulkan or Chromium performance measurement is claimed.

Borrowed name-set comparisons and temporary set storage still need resource
accounting work. This change does not establish complete parser accounting,
safe execution of arbitrarily deep syntax, full compatibility or production
security. Flat code ownership and explicit execution/parser continuations remain
on the [roadmap](../../docs/ROADMAP.md#javascript-execution-depth).

Run the focused checks with:

```sh
cargo test --locked --lib script::tests::scope_walk
cargo test --locked --test script_completion
```
