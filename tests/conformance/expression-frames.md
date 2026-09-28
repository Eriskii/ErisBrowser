# Expression and reference continuations

Supported expressions and assignment references now suspend in explicit runtime
frames. Each frame retains its immutable code unit, expression ID, environment
and pending values. The driver handles child expressions, computed keys, logical
short circuits, object entries, templates, callee/receiver resolution and argument
order. Identifier references borrow their name through a unit and ID, preserving
captured bindings without copying the name.

This is the first execution-driver stage. Statements, ordinary function bodies,
default initializers, constructors and native callbacks still use guarded native
recursion. Existing expression, combined-stack and logical-call limits remain.
The frame vector starts with eight precharged slots and grows with checked work
and cumulative storage charges, up to twice the existing 96 stack units: a guarded
expression can suspend one additional unticked reference job. That bound must be
revisited when statement and activation frames join the driver.

Native callbacks can reenter with another code unit. Each drive retains its own
frame boundary and restores its entry depth counters on success or failure;
cleanup cannot call author code or require allocation. Work, storage and logical
call budgets remain shared. Completed frames release their unit handles even
though their vector capacity remains cached. Array and argument value buffers are
precharged before evaluating children. Array property records are created before
the three parallel array arenas are updated, preventing a partial entry when
property allocation fails. Other runtime arenas and native helper allocations
have not received a complete accounting audit.

Six private test groups cover every work cutoff in eight expression fixtures,
frame growth/refusal/reuse, release of code handles, cross-unit getter/setter and
coercion reentry, exception identity, uncatchable host stops, captured/deleted
global references and array-arena alignment after allocation refusal. A thread
requesting a **128 KiB native stack** executes directly constructed 80- and
95-unary-expression chains; 96 hits the retained expression guard. The 80-unary
source is parsed separately on the normal test stack. This isolates expression
execution and does not establish deeper parsing or calls.

The [comparison record](expression-frames.json) retains identical case identities,
policies, preflight fingerprints and observations across **20 Test262 profiles /
6,879 modes / 1,680 controls**. All controls verify, every previous pass remains
and all observations are unchanged. Fifteen existing healthy baseline gates pass;
five resource-stopped profiles remain nonpassing observations. No baseline or
quota was changed. HTML remains at 3,868 matches, two mismatches and six unsupported
modes. Validation passes **932 Rust tests**, **151 Python checks**, **57 exact
pixel references** and **15,000 deterministic mutation cases**, plus formatting,
strict all-target Clippy and release compilation.

The [depth record](expression-frames-depth.json) preserves all **480** root-authored
source/depth/mode/parse-or-run observations from checkpoint `58a0ef7`. These exact
ordinary-call sources complete 13 simultaneous calls, with the first runtime stop
at 14; default-initializer sources complete 15, stopping at 16. Nested IIFE sources
complete ten levels, stopping in parsing at eleven. Sources, fingerprints and
observations are included; these are shape-specific probes, not upstream tests or
portable stack bounds. Both strict and sloppy modes retain the same frontiers.

Temporary parser syntax remains recursive and its allocations are incompletely
accounted. No independent-agent review, native-window, Vulkan or Chromium-relative
performance measurement is assigned. Full compatibility, production security and
the requested performance threshold remain unverified. Remaining statement/call
continuations and parser work are on the [roadmap](../../docs/ROADMAP.md#javascript-execution-depth).

```sh
cargo test --locked --lib script::machine::tests
```
