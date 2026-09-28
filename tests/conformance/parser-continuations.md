# Bounded JavaScript grammar continuations

The production parser now uses an explicit stack for supported JavaScript
expressions, statements, functions, defaults, cover grammar and template
substitutions. Suspended records contain typed IDs, bounded lists and saved
function context. Partial syntax and pending contexts release without following
recursive syntax ownership. The separate RegExp compiler retains its own bounds.

Grammar dispatch, frame relocation and capacity growth consume the existing
shared 100,000-work / 8 MiB compile ledger. The frame slot bound derives from that
storage limit. Source and token limits remain 256 KiB and 32,768. The obsolete
96-level grammar and 48-link member/constructor guards are removed; active
labels and declaration traversal retain their separate 96-entry limits. Runtime
logical calls remain capped at 32, and native helpers retain weighted guards.

Bounded leaf paths finish bare declarations and empty statements inside their
body/switch frame, and terminated numeric/string literals in one expression
dispatch. They use constant lookahead and retain the same validation/emission
charges. This avoids suspending grammar layers that do no work. An initial full
comparison found that redundant literal dispatch caused both variants of the
unchanged 2,048-element sort test to exhaust compile work before execution.
The leaf path restores their previous runtime instruction-limit outcome without
raising quotas. A regression test parses that original source in both modes and
compares 28 literal-boundary sources in four contexts against the frozen oracle.

The original upstream `S13.2.1_A1_T1.js` now passes in both modes. Its 32 nested
IIFEs also parse, execute and release on a thread requesting a 128 KiB native
stack, with runtime counters restored. Separate parsing/release checks on the
same stack request cover 256 nested IIFEs, 512 blocks, 128 arrays, 128 arrows in
a default and 512 nested function declarations. These larger cases test parsing,
not execution beyond the logical-call limit. Other new checks cover deep malformed
input cleanup, failed frame growth and 128-link member/call/constructor chains.
The retained oracle, work-cutoff, heap-refusal and flat-destruction checks still run.

All [480 existing depth sources](parser-continuations-depth.json) remain unchanged.
All three shapes now parse through the tested depth of 40 and execute through
32 calls. Call 33 stops at the existing logical ceiling. Compared with the direct
flat parser, this produces 104 new completions and 16 parse-to-runtime resource
changes, all in the nested-IIFE shape, with no lost completion. Ordinary/default
observations are identical.

The [comparison record](parser-continuations.json) covers all 20 retained Test262
profiles, 6,879 modes and 1,680 verified controls. Exactly two observations improve;
every other case and every control is identical. The complete function inventory
now has 511 passes and 620 unsupported variants, with no failure or resource stop.
Its first healthy [baseline](test262-functions-current.json) runs in CI, bringing
the healthy gate count to sixteen. Four other profiles still contain resource
stops; their full inventories remain measured. No upstream source, admission
policy, assertion helper or previous baseline changed.

All 959 Rust tests (777 library), 151 Python checks, 57 exact pixel references and
15,000 final mutation cases pass. The mutation run catches no panic or invariant
failure. The first implementation was also tested under the old grammar guards
before those guards were removed; historical guard-refusal assertions then changed
to acceptance while retaining their original inputs and adding larger quota checks.

This compile ledger is cumulative, including refused charges, and does not prove
total process-memory bounds. Diagnostic formatting, allocator overhead and runtime
arenas still need review. Whole units retain unused slots, cover tombstones and
otherwise unreachable code. Private oracle comparisons are not independent-agent
review or exhaustive standards/security proof. Full web compatibility, production
security, Chromium-relative performance and production Vulkan remain unverified.
