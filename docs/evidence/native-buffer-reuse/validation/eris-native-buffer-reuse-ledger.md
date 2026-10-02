# Proposed presenter ledger for one retired native buffer set

Design only, against the currently published timing implementation. No repository edits, compilation, test, browser, desktop or GPU execution. This is exact allocation reuse, not reuse of a Plan, content, scene identity, submitted command buffer, target pixels or glyph preparation.

## Ownership and unchanged admission

Keep one non-Clone retired resource bundle on the graphics owner, preferably `Gpu::retired_native: Option<NativeBufferLease> (slot entries must be Reusable)` beside the immutable pipeline owners. It must never be in `Shared`, a Packet, a callback, the UI thread or an acquired texture. `Kernels` can hold the slot instead if borrow layout remains simple. There is one active frame at a time (`run_owner`, vulkan.rs:717); at most one complete native bundle exists across the slot and active lease. No clone or temporary replacement allocation is allowed.

Define B = the current Native Plan's exact explicit allocation bytes, including packed output, exact draw parameters/input, padded conversion output and its 16-byte uniform. B must equal the core lease's checked actual buffer-size sum and be <=16 MiB. Readback R is a separate per-check allocation, never cached; B+R <=24 MiB. The global application formula becomes:

    16 MiB next-UI reservation
    + active Packet actual retained capacities
    + pending Packet actual retained capacities
    + retained CPU conversion scratch actual capacity
    + active native bundle B
    + retired native bundle C
    + current readback R
    + existing benchmark/identity reservation
    <=128 MiB

Use checked arithmetic throughout. Preserve the complete existing 16 MiB UI reserve even when no new frame is pending. The cache is <=16 MiB and participates in every existing `live_bytes()` consumer, including pending submission, timing setup and CPU scratch growth. At most one of active B and cached C is nonzero. Driver allocation, allocator bookkeeping, queues, surfaces, pipelines and bind-group object overhead retain their existing exclusions; this is an explicit-buffer ledger, not measured VRAM.

Do not retain the snapshot, Fonts session, CPU reference, Plan or an old per-frame stamp alongside the cache. Exact compatibility comes from fresh core preflight: same owning device/pipeline identities, Native profile, dimensions/format/stride, exact parameter and input lengths/presence, buffer usages. Different Plan content with identical sizes is permitted only because each frame rewrites the complete input/parameter arenas and conversion uniform, clears the full packed target and executes every draw plus conversion. A key mismatch is an ordinary cache miss, not a renderer failure.

## Narrow state and methods

Add only `retired_native_bytes: usize` to `State`, initially zero, and include it in `live_bytes()` beside `active_gpu_bytes` (control.rs:226). Keep phase/deadline ownership unchanged. The following names are proposed presenter-private methods; core type spellings can follow the separately sealed API.

- `checkout_retired_native(expected: usize, readback: usize) -> Result<(), String>`: require a nonzero exact retained charge, matching `expected`, zero active GPU/readback charge, current active frame and a healthy owner. Validate B<=16 MiB, B+R<=24 MiB and the recomputed global total. Commit `(C=B, active=0, R=0) -> (C=0, active=B, R=R)` in one mutex critical section. Failed preflight changes no charge.
- `reserve_fresh_native(planned: usize, readback: usize) -> Result<(), String>`: require C=0 and active=0, repeat the same caps, then commit active B/R before creating any API buffer. This can replace the internals of existing `reserve_native` while retaining its wrapper for existing tests. Do not keep its current mutate-first/global-check-later pattern for a transfer.
- `retain_completed_native(packet: &Packet, expected: usize) -> Result<bool, String>`: caller proves specific submission retirement, all scopes successful, readback unmapped/destroyed/dropped, successful native presentation and requested verification. Under the lock, check exact active B, C=0, current packet, stop/stalled/failure state and the unchanged deadline. A current healthy frame atomically transfers active B to retained C, and clears the now-dropped R. A stale/cancelled/failed frame returns refusal without changing the active charge; a charge mismatch is an error. The owner may then discard the safely retired lease outside the lock. No budget is recovered on a failed transfer.
- `retired_native_dropped(expected: usize) -> Result<(), String>`: called only after the owner has destroyed/dropped the retired handles outside the mutex. Check the exact retained charge, then clear C. An unexpected charge is an invariant error and remains conservatively charged until whole-owner release.

The owner-only slot and the CPU-only charge are intentionally separate: the ledger transition never moves/drops API handles while holding `Shared`'s lock. A move between local variables is allocation-free and retains the prior charge until the next atomic transition. On checkout, leave the slot charged until the transfer succeeds; only then `take()` into the outer frame lease. On return, retain the local lease under the active charge until the transfer succeeds; convert/move it into the empty retired slot immediately afterward. These final moves must be infallible and must not overwrite a populated slot. Precheck slot emptiness. Call the core’s fallible `mark_reusable_after_completion(&mut lease)` after the full success witness but before the under-lock charge transfer; a refusal retains the lease and active charge. If a concurrent stop/current check then declines the ledger transfer, destroy/drop this still-local retired lease rather than storing it. Thus only the final `Option` move follows the ledger commit.

A conservative charge may briefly outlive a successfully dropped allocation, but an allocation must never outlive its charge. A later asynchronous failure after a valid cache-return commit is a new owner failure: stop prevents checkout and whole-owner release drops the cache. The design does not promise atomicity with future driver callbacks.

`Shared::idle()` (control.rs:759) clears active Packet/GPU/readback charges only after their existing drop proof, and **does not clear C**. `complete_timing()` (control.rs:398) should continue to demand zero active GPU/readback state; it must allow a separately charged retired cache. Timing-ready/idle do not imply graphics-owner release. `released()` (control.rs:845) clears C only after the outer owner has dropped all API objects, exactly as it currently clears other graphics charges. `begin_release()`, timeout, panic, a finished thread handle and pending-packet replacement do not clear C or manufacture release.

## Per-frame ordering

1. After `take()` starts the existing five-second deadline, inspect the native request using pure core preflight before any replacement allocation. Compute exact key/B/R and check reference shape/device limits. All revalidation uses the new Plan, never cached dispatch metadata.
2. Before CPU routing, evict a retired native slot. Before dimension/format/configuration mismatch, evict before `surface.configure`, acquisition and replacement allocation. `Gpu::configure` (vulkan.rs:448) is a suitable backstop, including its one Outdated retry. Eviction takes the slot, destroys/drops it outside `Shared`, then clears C; only afterward reserve/allocate the replacement. Check the same original deadline before/after synchronous work; do not reset it. Same-size reconfiguration may conservatively evict.
3. Acquire using the existing path. An acquired-none result must not check out the cache or allocate a fresh bundle. A still-compatible untouched retired cache may remain charged; a known resize/CPU route has already evicted it.
4. For an exact cache hit, transfer cached charge to active plus R, then move its handles into the frame's outer lease. For a miss, finish eviction/drop, reserve B/R, then allocate exactly one bundle. If reuse-specific admission cannot fit without raising a cap, evict first and try the ordinary fresh admission once; any device/allocation/scope failure remains a renderer failure, not a late CPU fallback.
5. Establish the same three error scopes before API allocation/upload/encoding. Keep an outer `Option<NativeBufferLease>` even when encoding returns `Err`; the encode entry must borrow the lease mutably or return it with the error. Fresh allocation should be distinct from queue writes so allocation failure cannot lose a write-bearing bundle. No `?` may bypass the current submission/retirement/scope cleanup region after a possible queued write.
6. Set `QueueProgress::before_encode()` before the first queue-write-capable operation. Preserve exactly one Draws submission for a complete encoder, or one empty Flush submission after partial writes/cancellation. Drop the incomplete encoder before the flush; never submit a partial draw prefix. A callback cannot transfer/cache/drop a resource lease.
7. Poll the specific submission index with the original remaining five-second deadline. Retain the lease/readback until this retirement is known. Preserve all reverse scope pops and first-error retention. Only a successful draw submission plus successful scope/readback processing can reach presentation. Readback remains per-check and is dropped in the existing cleanup phase.
8. Keep the retired active lease alive through presentation, final deadline/current/stop checks and requested `shared.verified` bookkeeping. Only afterward request cache-return transfer and move the lease into the empty owner slot. `cancelled` and “not presented” are explicitly ineligible even though cancellation currently normalizes its returned outcome to `Ok(())`; `retired` alone is insufficient. A suboptimal frame may conservatively be discarded to force a clean reconfiguration.
9. On a successfully flushed cancellation, known-retired buffers may be explicitly destroyed/dropped, then the existing successful-return/idle path clears active charges. On any encoding/scope/map/compare/present/deadline error, never recache. If retirement is known, explicit destruction is permitted; if unresolved, use ordinary handle drops and whole-owner release, retaining the conservative active ledger until `released()`. The owner error path already skips `idle()` (vulkan.rs:733).

Resize invalidation can originate on the UI thread while the owner waits; that thread must not touch GPU resources. The minimal design evicts when the owner next processes a packet, before configure/allocation. During the wait the old retired slot stays bounded and charged. If immediate idle resize/occlusion eviction is later required, add an explicit owner maintenance wake/action; do not pretend a `Shared::invalidate` byte reset destroyed anything. Immediate idle eviction is unnecessary for this first reuse increment.

## Lock, callback and release boundaries

All `wgpu` calls, buffer/key inspection, creation/destruction, scope pops, waits, packet-reference comparisons and lease drops run without a Shared mutex guard. Queue/device callbacks may call `Shared::fail`; holding that lock around an API operation would reintroduce deadlock. Ledger methods only inspect/replace integers and existing state. Condvar/winit notifications remain outside the guard, as current `notify` requires.

The resource owner alone changes C/B. UI submissions may alter pending bytes and target state between preflight and checkout, so every ledger commit recomputes the current total under the lock; a previously measured free-space total is not authority. No resource allocation follows a refused reservation. On shutdown, cache handles must be dropped along with `Gpu` before the outer `owner.released()` acknowledgment (vulkan.rs:57–70). A blocking drop remains an unresolved owner, not release. The cache contains no acquired surface texture, so its lifetime does not grant surface ownership or verification success.

## Focused tests before measured acceptance

1. Pure ledger exact limits: cached B survives idle; both hit and fresh B+R exactly24 MiB pass, one-over/overflow fails unchanged; B exactly16 MiB versus one-over; global total exactly128 MiB versus one-over, including UI16 MiB, scratch, benchmark and pending/active Packet capacities.
2. Transfer atomicity: hit preserves total apart from R; successful return removes dropped readback only; no state has both C and active B; wrong expected bytes, duplicate checkout/return/drop and failure leave charges intact. Failed-transfer models must preserve ownership, not drop the only lease.
3. Eviction-order fake owner: CPU scratch growth, dimensions, format, exact parameter/input lengths and presence mismatch all drop the old retired bundle before any new allocate callback. Current untouched cache remains charged across acquire-none. Pending replacement cannot reset the active deadline or C.
4. Cancellation at every queued-write cutpoint: flush once, incomplete encoder never submitted, lease retained through specific retirement, no cache return. No-writes cancellation may skip submission, but also does not recache its checked-out frame bundle.
5. Scope/poll/map/compare/presentation/current/deadline/stop failures never return a reusable frame; scope collection still happens on earlier failures; unknown retirement never explicitly destroys a buffer or clears its charge early. Inject a stop during post-poll/pre-return to test the under-lock recheck.
6. Timing and release: successful cache return followed by packet drop/idle allows exactly one sample completion; an outstanding active lease does not. Retired C does not make `benchmark_owner_released` true. Delayed/panicked/stalled actual drop preserves C and refuses release; real outer release clears it.
7. Core/GPU acceptance remains separate: alternate equal-size Plans with changed image/mask/uniform contents, empty/hidden work and alpha; compare full acquired targets, including resize/format miss sequences. Count actual bundle allocations/evictions without adding default clocks. Existing Probe APIs/bytes, shaders and old one-shot behavior remain unchanged.

## Core API alignment and source bindings

- `/tmp/eris-native-timing-next-proposal.md`: `d0a075768d7177ac813783c91e0f5bf4a0420d5548bbbdd07f78bdf9658b5457`.
- `src/presenter/vulkan/control.rs`: `cefca28896617574f1a66f5d019849139fc22c03d9f7f39de830b3f1d3da8268`.
- `src/presenter/vulkan/native.rs`: `5b7d0127b89f47037d2b09c13931150713c7043640d99f3906b1f736c296a327`.
- `src/presenter/vulkan.rs`: `50d90b8664b136bf1713e073e4155ce01182260eba43b91c9e2edb3b5c536ddb`.

The sealed `/tmp/eris-native-buffer-reuse-api.md`, SHA `a67bc99c8cba5a330aa8c7b27a7068942d30169be6661860de15beb75c9aed50`, provides the needed seam: `NativeEncoder::requirements(plan, format)` and `NativeRequirements::{buffer_bytes, layout}` before allocation; `NativeBufferLease::{is_compatible, buffer_bytes}` for exact checkout; `NativeEncoder::allocate` before any queued write; `encode(&mut lease, ...)` preserving the full lease on every error; and `mark_reusable_after_completion(&mut lease)` before the final ledger transfer. The owner cache admits only the Reusable lifecycle state, even though the core compatibility check also accepts a never-used Fresh lease. The private Arc context is the device/pipeline authority; Device equality or an adapter name is not substituted.

No performance improvement is claimed from this design. Queue staging allocations remain outside this destination-buffer reuse change, and the current first/second-frame submission tails must not be removed from future timing samples.
