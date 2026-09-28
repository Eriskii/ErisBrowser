# Flat executable ownership and paged tokens

Compiled JavaScript now owns flat expression, statement and function records in
an immutable shared unit. Typed IDs connect records. Function defaults and nested
bodies use those same IDs; closures retain a unit plus a function ID, with no
owning links between syntax records. Keeping one closure keeps the complete unit,
including otherwise unused code. Runtime readers carry the unit explicitly, so
callbacks into a different compiled script do not replace a caller's code context.
Inline handlers use this representation too.

The guarded parser still produces temporary owning syntax. Iterative lowering
uses its remaining work and cumulative allocation ledger, charging record/edge
vectors, worklist growth, copied text and the unit header. Successful compilation
releases the temporary tree. Known edge-list sizes are reserved once; statement
leaves are emitted directly without queued revisits. This does not comprehensively
account for the temporary parser's allocations or prove total peak memory.
Conservative legacy per-closure and per-activation metadata allowances remain,
even though executable records are shared rather than copied at those boundaries.

The first lowering candidate exhausted the compile allowance on the existing
8,327-declaration Unicode 10 case. Token storage now uses 128-record pages with
charged descriptor/page growth, avoiding repeated relocation of the full token
prefix. Together with exact edge reservations and direct leaf emission, this
preserves that unchanged case in both modes. No source, token, work, allocation,
parser-depth, native-stack or logical-call ceiling was raised. Lexical rescans
retain cumulative charges and preserve token offsets and lexical goals.

Six private code test groups cover every syntax edge by a bounded test-only
round trip, identical declaration-walk order/work, terminal depth/work failures,
allocation/copy refusal, shared compile limits, cross-script closure/callback
lifetimes and final release. Three token groups check stable record addresses,
page-boundary access and iteration, truncation/refill, cumulative charges and
work/storage/count refusal. Existing diagnostic, completion, recursion, inline
handler, page and confined-worker checks continue to pass.

The release check constructs a unit directly on a thread requesting a **64 KiB
native stack**, then releases its last handle: **32,000 expression records,
8,000 statement records and 8,000 function records**. This isolates flat ownership.
It does not show that such deep source parses or executes. The parser, evaluator,
ordinary calls and native callback bridges retain their existing recursion guards.
RegExp payload ownership and runtime object graphs remain separate concerns.

The [comparison record](flat-code.json) preserves every case fingerprint, runner
policy and preflight fingerprint/observation across **20 Test262 profiles /
6,879 modes / 1,680 controls**. All controls verify and every prior pass remains.
The unchanged escaped Unicode 5.2 start-character source now passes in both modes:
[identifiers](test262-identifiers-flat-code.json) has **509 passed / 154 unsupported /
six compile resource stops**, up from 507 passes and eight resource stops.
[Array sort](test262-array-sort-flat-code.json) remains **57 passed / 46 unsupported /
four runtime resource stops**; the two 2,048-element stability modes now stop on
instructions rather than allocation. All other observations are identical.
Fifteen healthy baseline gates pass; the five resource-stopped profiles remain
nonpassing observations, with no new baselines recorded.

Final validation passes **926 Rust tests**, **151 Python checks**, **57 exact pixel
references** and **15,000 deterministic mutation cases**, plus formatting, strict
all-target Clippy and release compilation. The existing 144 declaration-name
syntax/diagnostic variants and 240 scalar completion variants also pass. HTML
remains at 3,868 matches, two mismatches and six unsupported modes.

No independent-agent review, native-window, Vulkan or Chromium performance
measurement is assigned. Full web compatibility, production security and the
requested performance threshold remain unverified. Explicit execution
continuations and an iterative parser remain on the
[roadmap](../../docs/ROADMAP.md#javascript-execution-depth).

```sh
cargo test --locked --lib script::code::tests
cargo test --locked --lib script::tokens::tests
```
