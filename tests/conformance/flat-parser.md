# Direct flat parser ownership

The production JavaScript parser now emits expression, statement and function
records directly into the executable unit. Child IDs connect records, including
parameter defaults, nested functions, object methods and statement bodies.
There is no production owning AST or second lowering pass. Completed and partial
syntax therefore release without recursively following child ownership. The
previous grammar and owning AST remain under `cfg(test)` as a frozen private
comparison oracle; they are absent from release builds.

Record storage uses 128-slot pages. Appending parser records preserves existing
record addresses; page descriptors can relocate, with copying work prepaid.
Record pages, parser lists, names/operators, folded strings and publication share
the existing compile ledger and checked reservations. Root bodies reserve an
ID-slot bound from the already lexed prefix, capped at 32,768 slots. RegExp and
template rescans can extend that prefix and trigger further charged growth.
Small nested lists grow separately under the same ledger.

The initial implementation exhausted compile work on the previously passing
8,327-declaration Unicode fixture because incremental record/root-list growth
repeatedly copied prefixes. Pages and the root reservation remove that work;
the original fixture passes with the same 100,000-work and 8 MiB compile limits.
No source, token, grammar-depth, logical-call or native-stack constants increased.

Cover grammar still parses once. Arrow reinterpretation moves names and default
IDs out of unreferenced records; charged tombstones remain in the unit. RegExp
and template rescans do not rewind tokens. IdentifierName metadata remains
transient for object shorthand/accessor validation. Declaration and parameter
checks borrow flat records, preserving strict directives, source-order diagnostic
selection, function/label boundaries and body-scope conflicts. Inline handlers
use the same builder and existing event-parameter/body policy. Conservative
activation metadata charges retain their previous size.

Seven parser test groups compare grammar payloads and early errors against the
frozen oracle, exercise cover/default identity, every work cutoff and sampled
heap refusals in five fixtures, handler metadata, and partial-unit cleanup.
Two record-page groups check stable indices, bounded descriptor work, refusal
before growth, retained payloads, and checked extension of singleton records.
Existing expression, statement and activation source fixtures now use the
production parser; direct synthetic executable fixtures remain separate.

A thread requesting a 64 KiB stack releases directly constructed 16,000 unary
edges and 8,000 labeled-statement edges, both after successful publication and
a refused publication. This isolates destruction, not deep-source parsing.
Grammar calls still recurse under their existing guards. All
[480 depth observations](flat-parser-depth.json) remain identical: shallow
ordinary/default recursion completes 32 calls, while nested IIFEs complete ten
and stop in parsing at eleven. The unchanged upstream 32-IIFE case still stops
in parsing. Grammar continuations remain the next step.

The [comparison record](flat-parser.json) verifies identical fingerprints,
policies, preflights and every observation across 20 Test262 profiles, 6,879 modes
and 1,680 controls. Fifteen healthy baseline gates pass. Five profiles still
contain resource stops; no new baseline was recorded. HTML retains 3,868 matches,
two mismatches and six unsupported modes. All 953 Rust tests, 151 Python checks,
57 exact pixel references and 15,000 mutation cases pass; the mutation run catches
no panic or invariant failure.

Pages and root reservations can retain unused slots. Cover tombstones and
otherwise unreachable code remain until the entire unit is released. Charges
are cumulative, including refused requests. Diagnostic formatting, allocator
overhead, runtime arenas and native helpers still need accounting review; this
ledger does not prove total process-memory bounds. Declaration traversal retains
its separate 96-ancestor bound. No full compatibility, production security,
Chromium-relative performance or production Vulkan result is claimed here.
