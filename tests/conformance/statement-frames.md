# Shared statement continuations

All supported statement and statement-list execution now shares the explicit
driver with expressions and references. Pending work retains a code unit,
statement/list identity, environment and phase. Typed list locators select program,
function, block and catch bodies without copying their statement vectors. `for`
iteration environments borrow binding names from the declaration record instead
of building a temporary name vector.

The driver schedules declarations, blocks, branches, labeled control flow,
while/do/for/for-in loops, switch, return/throw and try/catch/finally. It retains
the distinction between empty completion and an explicit undefined value. List
and loop continuations preserve prior values; finalizers retain or override
pending completions. `for-in` holds its own key snapshot and visited set, checks
live property state and reads the next prototype after body mutations.

Ordinary exceptions travel through suspended frames to the appropriate try
continuation. Resource and unsupported-feature errors terminate the current
drive immediately, bypassing catch/finally. Each reentrant native drive preserves
its frame boundary; failure cleanup cannot allocate or invoke author callbacks.
All expression and statement depth counters remain. The continuation storage
ceiling changes from 192 to 193 slots to include a program root list: each existing
guarded expression/statement can suspend one reference/list job, while reentrant
function lists retain their four-unit native-call guard. Frame growth charges
actual record storage and relocation work before checked reservation. This does
not raise execution-depth, logical-call, work, heap or parser ceilings.

Ordinary invocation and default initialization still enter through native
recursion. Function bodies execute through the shared driver, but their calling
activations do not yet suspend there. Constructors and native callback bridges
also remain guarded. The next stage must schedule activation, defaults and body
entry together before claiming deeper ordinary calls.

Five new private test groups cover every work cutoff across eight statement
fixtures, storage cutoffs across five fixtures, callback exception identity,
finally overrides, host termination, per-iteration closure capture and live
for-in mutation. Frame cleanup restores counters and releases code handles.
A thread requesting a **128 KiB native stack** executes directly constructed
80- and 94-block chains. A 95-block chain plus its expression statement/operand
hits the retained 96-unit guard. The frame vector reaches its 193-slot ceiling
without a frame-limit failure. These fixtures isolate execution from the still
recursive parser; they do not establish deeper source acceptance or calls.

The [comparison record](statement-frames.json) preserves every identity, policy,
preflight fingerprint and observation across **20 Test262 profiles / 6,879 modes /
1,680 controls**. Every control verifies and all previous results are unchanged.
Fifteen existing healthy baseline gates pass. Five resource-stopped profiles
remain nonpassing observations; no new baseline is recorded. The
[480 depth observations](statement-frames-depth.json) are also unchanged: these
exact ordinary-call sources first stop at 14 calls, default-initializer sources
at 16 and nested IIFEs at parser depth eleven, in both modes. Those are
shape-specific root-authored probes, not portable limits or upstream tests.

Validation passes **937 Rust tests**, **151 Python checks**, **57 exact pixel
references** and **15,000 deterministic mutation cases**, plus formatting,
strict all-target Clippy and release compilation. The 144 declaration-name
variants and 240 scalar completion variants continue to pass. HTML remains at
3,868 matches, two mismatches and six unsupported modes.

Parser syntax remains recursive and incompletely accounted, and runtime arena
allocation accounting remains incomplete. No independent-agent review,
native-window, Vulkan or Chromium-relative performance result is assigned.
Full compatibility, production security and the requested performance threshold
remain unverified. See the [remaining execution work](../../docs/ROADMAP.md#javascript-execution-depth).

```sh
cargo test --locked --lib script::machine::statements::tests
```
