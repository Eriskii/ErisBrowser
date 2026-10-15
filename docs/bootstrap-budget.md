# Separate trusted bootstrap allowance

Fixed intrinsic initialization now has a finite **1,000,000-unit work allowance**.
Author entries retain **100,000 work units**, and initialization and execution
share the unchanged **8 MiB cumulative allocation budget**. This increases the
initialization cap; it does not reduce charged work or raise author limits.

The private `Runtime::uninitialized` sets `BOOTSTRAP_STEPS`. The only production
caller, `try_new`, completes object reservation, frame preparation and intrinsic
installation before resetting work to `MAX_STEPS` and returning the realm.
An initialization error propagates before that reset or publication. Convenience
constructors retain their documented panic behavior on the same failure.
Initialization allocations remain charged after the author allowance is granted.

The constructor installs fixed compiled metadata before any author callback or
document-dependent script execution. It retains the existing monotonic start
timestamp. The supplied Date host is assigned after initialization, without
sampling its clock or discovering a timezone during bootstrap. Page, worker,
adapter and stress-tool construction use the same successful-return boundary.

Script and host-event resets still grant only the author allowance. Nested
callbacks, conversions, Promise jobs and microtasks spend its remaining work.
Parser budgets retain their existing limits. Test262 observer setup occurs after
public construction, harnesses use ordinary script entries, and the final audit
spends the body's remaining allowance. No transport, reporting policy or
conformance baseline changes.

The Rust 1.88 focused diagnostics observe **99,694 consumed startup units** and
**900,306 remaining** under the new cap. Accounted allocation remains
**1,744,317 bytes**, with 752 objects and capacity, 299 native registry entries,
25 prototype entries and 212 global bindings. Public constructors still return
with exactly 100,000 work units. The additional raw remainder is private test
observability, not an author-accessible allowance.

One new control checks that exactly 1,000,000 raw work units are admitted and
that another unit, or a first request for 1,000,001 units, refuses with zero
remaining work and no allocation or metadata publication. The published parent
fails its first raw-budget assertion after passing the author/heap constants;
its later exact/oversized admission checks are not reached. The candidate passes
this unchanged control. Twelve existing groups in ten files change only
raw-origin accounting assertions or derivations. Their operation costs, heap
assertions and semantic checks remain intact. The three explicit 99,693/99,694/
100,000 initialization matrices retain their exact refusal and success results.
Existing constructor, Date-host, callback and host-audit tests preserve the
success-only author grant and shared-budget boundaries.

The focused Rust 1.88 bootstrap run passes **53 tests**, including the new
control and the existing raw/refusal/public-entry witnesses.

Complete `--include-ignored` runs pass **3,132 Rust 1.88 default tests** and
**3,486 Rust 1.99 tests with `test262-host,vulkan-raster`**. Both report 52 result
rows with no failed, ignored or filtered tests. Each retains the complete prior
roster and adds exactly the one new control, with no test-name changes. Strict
all-target Clippy passes in both configurations; the formatting check passes.

The rebuilt Rust 1.88 release host adapter preserves every **1,050 case-mode
record and 348 harness records** across the four existing Promise profiles.
Complete case and control arrays match the published parent, including failed
and unsupported outcomes. All controls verify, with zero regressions,
improvements or baseline changes. No new standalone Python-suite or upstream
selection result is claimed.

This is a resource-policy change, not a CPU-performance or resident-memory
measurement. Initial bindings, B-tree/Rc/string allocation and other existing
paths still contain infallible allocations; the logical ledger does not account
for all process memory or establish production security. Historical checkpoint
measurements and archives retain their original limits and results. Promise.any
and AggregateError remain separate work; additional initialization headroom
alone does not establish their implementation or admission.

The [evidence manifest](evidence/bootstrap-budget/manifest.json) binds the
selected sources, controls, parent observation and validation receipts. The
[source and check archive](evidence/bootstrap-budget/sources-and-checks.tar.gz)
retains the corresponding artifacts.
