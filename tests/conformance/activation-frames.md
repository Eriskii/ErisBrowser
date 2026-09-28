# Ordinary activation and default continuations

Ordinary function invocation, parameter defaults, bound forwarding and body
execution now run in the same driver as expressions and statements. Direct calls
schedule activation frames instead of invoking a Rust function recursively.
Defaults suspend with their parameter environment and actual values, then resume
initialization in source order. Bodies retain the separate environment required
by expression parameters. Normal body completion returns undefined; explicit
return completion carries its value.

The unchanged **32 logical calls** are shared by ordinary calls, bound wrappers,
targets, native builtins and callbacks. A queued invocation acquires its logical
count only after a successful frame reservation. Completion and ordinary-error
unwinding release that count before the caller resumes. Native callback entries
retain their external count and four-unit stack guard; their root frame does not
charge either twice. Direct native intrinsics reserve their own guard. Constructors,
JSON, event dispatch and other unconverted native paths remain guarded bridges.
Host resource/unsupported termination bypasses catch/finally and restores the
current drive's frame boundary and counters without allocating during cleanup.

Fully iterative JavaScript no longer consumes the native-stack guard or the old
96-expression ceiling. The expression count now tracks cleanup rather than
limiting execution. The old 193-slot continuation bound is replaced by
`MAX_HEAP / size_of::<Frame>()`, derived from the existing **8 MiB charged runtime
allowance**. Geometric growth prepays record capacity and relocation work before
checked reservation. The initial eight-slot cache remains. The **100,000 work**,
**32-call**, **96 native-stack-unit** and parser constants are unchanged. This is
a replacement of obsolete native-depth charges for iterative work, not a claim
that all previous depth behavior is unchanged.

Bound forwarding reserves one checked argument buffer and charges copying work
before extending it. Parameter evaluation releases its actual-value vector before
body execution; arguments objects and bindings retain their own values. Shared
code units and conservative legacy activation metadata allowances remain. Charges
are cumulative, including refused requests; the ledger is not a measurement or
proof of total process memory. Other runtime arenas and native allocations remain
incompletely accounted.

Seven new test groups check strict/sloppy depth boundaries, partial initialization
at every work cutoff in eight fixtures, storage refusal, mapped/unmapped arguments,
rest/default behavior, bound-wrapper counts, TDZ/default closures and cross-unit
finally identity. Native getter, coercion, constructor, apply and JSON recursion
still stops through the native guard without executing catch/finally. An exhausted
frame-growth check preserves already queued frames and a caller's logical count.

A thread requesting a **128 KiB native stack** completes **32 ordinary calls**
and **32 default-initializer calls**, with correct values in both modes; call 33
stops at the logical ceiling. These fixtures construct executable code directly
to isolate execution, with equivalent source parsing checked separately. The
previous small-stack tests now complete the 96-unary and 95/96-block fixtures.
The 97-block fixture stops in bounded declaration traversal, which independently
retains 96 ancestor slots.

The retained [480-case depth inventory](activation-frames-depth.json) records:

| Exact source shape | Previous completed calls | Current completed calls | First current stop |
| --- | --- | --- | --- |
| Shallow ordinary recursion | 13 | 32 | 33: logical call limit |
| Recursion in a default initializer | 15 | 32 | 33: logical call limit |
| Nested IIFEs | 10 | 10 | 11: parser limit |

Both modes agree. **72 mode/depth runs newly complete**; **32 other runs** change
their resource diagnostic to the logical-call limit. All parsing observations and
nested-IIFE outcomes remain unchanged. These are root-authored, shape-specific
probes, not upstream conformance tests or portable stack bounds. The original
upstream 32-nested-IIFE source is retained and still stops in parsing.

The [comparison record](activation-frames.json) preserves every fingerprint,
policy, preflight and observation across **20 Test262 profiles / 6,879 modes /
1,680 controls**. Every control verifies. Fifteen existing healthy baseline gates
pass; five resource-stopped profiles remain nonpassing observations, with no new
baselines. Validation passes **944 Rust tests**, **151 Python checks**, **57 exact
pixel references** and **15,000 deterministic mutation cases**, plus formatting,
strict all-target Clippy and release compilation. HTML remains at 3,868 matches,
two mismatches and six unsupported modes.

The parser and temporary owning syntax remain recursive and incompletely
accounted. Native bridges can still reach their guard before 32 calls. No
independent-agent review, native-window, Vulkan or Chromium-relative performance
result is assigned. Full compatibility, production security and the requested
performance threshold remain unverified. The [parser work](../../docs/ROADMAP.md#javascript-execution-depth)
remains separate from this execution improvement.

```sh
cargo test --locked --lib script::machine::calls::tests
```
