# Borrowed worker-snapshot bridge: independent contract review

This is a proposed checker-only interface, based on read-only inspection of the
current browser and probe. No renderer, planner, engine, compiler or GPU was run,
and no repository file was edited. It does not change the interactive browser's
CPU painter or optional upload presenter. Original thirty probe fixtures and
protocol tuples remain separate, unchanged contracts.

## Interface and admission

Use an optional own-repository library dependency, disabled by default, and a
separate feature-required checker binary. The dependency package is
`eris-browser`, with library name `eris`; it need not enable the browser's
`vulkan-presenter` feature. The standalone compute dependency still needs WGSL.

```rust
// tools/vulkan-raster-probe/src/browser_adapter.rs, feature-gated
pub fn plan_snapshot(
    snapshot: &eris::worker::Snapshot,
    frame: Frame,
) -> Result<BridgePlan, FallbackReason>;

// Same checks; useful for independent direct DrawCommand fixtures.
pub fn plan_display_list(
    commands: &[eris::graphics::DrawCommand],
    images: &eris::graphics::ImageStore,
    frame: Frame,
) -> Result<BridgePlan, FallbackReason>;

pub struct BridgePlan { /* private Plan and fixed-size statistics */ }
impl BridgePlan {
    pub fn plan(&self) -> &Plan;
    pub fn stats(&self) -> &BridgeStats;
}
```

`plan_snapshot` delegates using borrowed `snapshot.layout.commands` and
`snapshot.images`; it neither mutates nor clones the snapshot. `Frame` carries
physical width/height, packed RGB clear, caller clip, document offset and fixed
viewport offset. The checker only requests zoom one. No implicit scaling,
scroll lookup, chrome, focus-ring or toolbar rendering enters this API.

Return a small enum/struct with a stable category and optional original command
index, not an error string built from author keys. Proposed categories are
`UnsupportedText`, `UnsupportedRoundedRect`, `UnsupportedOpacity`,
`InvalidGeometry`, `InvalidImage`, `InvalidScope`, `ViewportLimit`, `CommandLimit`,
`ScopeLimit`, `ImageStoreLimit`, `KeyBudget`, `CpuPaintBudget`, `PlannerLimit` and
`AllocationFailure`. Final spellings should be matched to the independently
frozen fixture protocol before implementation. Never infer categories by parsing
planner error strings; validate bridge categories directly, then retain a bounded
static planner detail where needed.

Every refusal is a whole-frame GPU admission refusal before Vulkan submission.
The caller keeps the original borrowed inputs and paints that entire list with
Canvas. GPU/device/map/readback/comparison errors are execution failures, never
fallback. `BridgePlan` construction stays private: callers cannot synthesize a
partially validated plan or replace its command/source metadata.

## Bounded lookup before allocation

Keep the existing limits: 320×240 framebuffer, 256 original commands, 32 combined
clip/fixed scopes, 256 source entries, 1 MiB source bytes, 1 MiB explicit GPU
buffers, 143,360 LUT entries and four million rounded dispatch invocations.
The clear remains an additional draw and counts in GPU buffers/invocations as
today; dropping missing images must not evade the original 256-command check.

`ImageStore` is actually `HashMap<String, Arc<RasterImage>>`. IPC retains aliasing
with shared Arcs and permits absent image keys. Do not collect map entries or
hash a command key before bounding the relevant input. Rust documents that map
iteration visits capacity, including empty buckets, so a len-only cap is not a
sufficient iteration bound for a public API receiving a sparsely reserved map.
[HashMap iteration documentation](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.iter).

Root-approved additional CPU admission limits: at most 256 store key entries,
map capacity at most 512 before iteration, at most 4,096 UTF-8 bytes in any one
key, and separate 65,536-byte aggregate limits for store keys and Image-command
keys. Sorting has a 16,777,216-byte comparison allowance; binary lookup has a
1,048,576-unit allowance.
These restrict the bridge; they do not raise a painter, IPC or GPU limit.

A bounded implementation can avoid creating a second hash table altogether:

1. Check framebuffer, original command length, map len/capacity and fixed input
   scalars using constant-size state. Scan commands once for key lengths and
   unsupported kinds, and scan at most the capped map capacity for key lengths
   and raster metadata. All sums/products use checked arithmetic. No key bytes
   are hashed/compared, pixel data scanned, vector grown or source copied yet.
2. Use fixed-capacity borrowed tables for store entries and unique raster
   identities, or fallibly reserve those metadata tables only after their exact
   counts have been checked. Sort borrowed entries by key with bounded insertion
   sort: at most `N*(N-1)/2` comparisons. Precheck
   `(N-1)*total_store_key_bytes` as a conservative byte-comparison allowance
   before sorting; it is at most 16 MiB under these caps. The sort needs no key
   copies or allocator growth. Deduplicate raster identity by `Arc::ptr_eq`, not
   bytes, key spelling or a cloned image. At most 256² pointer comparisons per
   bounded traversal are needed.
3. Resolve command keys against the sorted borrowed table with an explicit
   binary-search loop performing at most one lexical comparison per iteration.
   Before the first comparison, precheck
   `(ceil_log2(N+1)+1) * sum(command_key_length+1)` against 1,048,576 units.
   For N=256 the factor is ten, so even the maximum 65,536 command-key bytes
   plus 256 per-command units cost at most 657,920. This is independent of
   HashMap iteration order/collision behavior; no key cloning or content hashing
   is required. Sorting and lookup allowances are checked before either begins.
   Use zero sort allowance for N=0/1 to avoid unsigned N-1 underflow. These are
   CPU preparation bounds, not GPU invocations. The maximum sort allowance is
   255*65,536 = 16,711,680, below its 16,777,216 cap.
4. Assign numeric source IDs in first command-reference order. Repeated keys and
   different keys that point to the same Arc reuse one ID. Equal RGBA in distinct
   allocations remains two sources. Pointer values never enter the public digest
   or numeric ordering. Missing keys record a no-op and do not allocate a source.
5. Append still-unreferenced Arc identities in the sorted key order, skipping
   aliases already present. Every unique raster in the store now has exactly one
   borrowed `SourceImage` view. The existing planner checks dimensions, exact
   `width*height*4 == rgba.len()` and aggregate bytes for **all** views before its
   own metadata allocation or source-sized work. This reuses its current image
   validator. When an image draw is visible, it packs every supplied source once,
   including unused sources, into the already bounded arena. With no visible
   image it packs none. This final required packing is the only source byte copy;
   there is no cloned RGBA staging vector.

The explicit all-store validation is stricter than Canvas, which only reaches
looked-up images and accepts oversized RGBA tails. A malformed or oversized unused
entry therefore causes GPU fallback, not altered rendering. Packing unused but
valid sources may also conservatively reach the unchanged GPU byte cap. Report both store
entry count and unique stored/referenced raster counts/bytes so aliases cannot
hide the accounting policy. Stable output IDs and byte totals must not depend on
the map's randomized iteration order.

## Scalar and scope equivalence

Convert zero-radius Rects directly, preserving RGBA and all f32 bits. Validate
geometry/radius before alpha-zero omission. Reject nonzero radius even if hidden,
empty or transparent. Reject Text and both opacity scope variants everywhere,
including opacity one. Do not prune unsupported commands using visibility.

An Image with a missing key is a no-op without file/network access; it must not
discard earlier placeholder rectangles or skip validation of subsequent commands.
Validate its geometry as a bridge admission check even when the key is absent.
Images with alpha-zero samples retain normal coverage/dispatch and hidden RGB.
No alpha scan or source-content deduplication is permitted.

Canvas lowers Line as:

```text
x      = min(x1, x2) + current_dx
y      = min(y1, y2) + current_dy
width  = abs(x2 - x1).max(line_width)
height = abs(y2 - y1).max(line_width)
```

The adapter should emit an **untranslated** rectangle using the same scalar
`min`, subtraction, `abs` and `max` order; existing planner translation then
performs the one addition. Do not translate endpoints first, use f64, round to
integers, add line width to the endpoint span, or implement a geometric stroke.
Validate all original endpoint/width scalars before lowering, then validate the
lowered extent against the existing coordinate cap. Reverse diagonals and
zero-length lines are ordinary filled rectangles under this contract.

Use one typed clip/fixed stack, not independent depth counters. PushFixed saves
the current clip and translation, resets to the original normalized caller clip
and fixed viewport offset, and may escape an empty ancestor clip. PopFixed
restores both. Clips nested inside fixed scopes use fixed translation. Mismatched
pops and unfinished stacks refuse the whole frame. Keep the actual full scope
sequence rather than flattening or discarding apparently redundant wrappers.

Rect coverage is floor/ceil coverage, not center containment. The alpha/opaque
upper-clip distinction and image LUT f32 division order already implemented in
the probe must remain unchanged. This adapter should not add a second coverage
or sampling implementation. Worker IPC allows coordinates up to 16,000,000 and
128 scopes, while the probe has narrower geometry/32-scope limits; a valid IPC
snapshot can legitimately require CPU fallback.

## Preserve the CPU work bound

Before materializing any planner data, compute with checked integers:

```text
area = frame.width * frame.height
cpu_pixel_budget = clamp(area * 16, 1_000_000, 32_000_000)
upper_bound = original_command_count * area
require upper_bound <= cpu_pixel_budget
```

This conservative test covers every rectangular/image loop, even transparent,
missing or clipped commands, without copying Canvas's more permissive exact work
calculation. Clear is outside `paint_with_viewport`'s pixel budget, so it is not
added here. The unchanged 256/32 caps are below Canvas's 200,000/128 command/scope
limits. Glyph and opacity allocation are excluded from GPU admission. This test
may fall back on a frame Canvas would finish; that is intentional and visible.
Checking only GPU dispatch count is insufficient: Canvas charges image loop
pixels subsequently rejected by blend, whereas the probe trims those pixels.

CPU reference/fallback must call `Canvas::new`, clear, `set_clip`, and
`paint_with_viewport` using the **original** list, images and offsets. Record
`exhausted()` and require the independently frozen expectation. Never silently
increase Canvas budgets, drop its exhaustion flag, or call individual primitives
outside the display-list budget to manufacture a reference.

## Ownership, capture and failure boundary

Snapshot's public fields are not a validation token. Keep bridge checks for both
actual decoded snapshots and direct fixtures. Real capture uses scripts false,
hash-bound browser bytes, explicit local fixture paths, Load and Render through
WorkerClient, and the normal IPC decoder. Verify expected geometry/colors, image
RGBA and alias relationships before planning; unexpected commands are input
mismatches, not permission to revise the pixel oracle or claim an expected
fallback. Preserve diagnostics, including missing-image errors.

Collect workers sequentially before any Vulkan instance/device initialization.
For each capture, build its bounded plan and CPU result while borrowing the
snapshot, then drop that snapshot and all page/broker/decoder clients. Retain only
bounded plans, literal expected data, CPU results and command digests. Cap the
fixture count and aggregate retained plan/CPU bytes explicitly; per-frame 1 MiB
is not an aggregate memory bound. No navigation or worker restart after driver
initialization is included. A successful returned Plan owns its bounded packed
arena and no Arc/snapshot borrow, so snapshot destruction cannot invalidate it.

Current Channel teardown kills its child process group then performs blocking
`Child::wait()`. A returned Drop establishes completion of that code path; it is
not an independent finite teardown deadline. Page/broker children have their own
process groups, so killing only the checker group does not clean up all workers.
Do not initialize Vulkan until capture teardown has actually returned. No source
change to production Channel is required by this proposal, but the outer checker
supervision must handle timeout ownership honestly.

Root's proposed dedicated Linux subreaper supervisor is feasible without cgroup
or system setting changes. Establish `PR_SET_CHILD_SUBREAPER` in a new single-thread
supervisor before spawning its sole checker. Keep SIGCHLD at normal waitable
semantics: no ignored SIGCHLD, SA_NOCLDWAIT, background reaper, thread calling wait,
or incidental `Popen.poll()`/`wait()` before owned-group cleanup. Start the checker
in a private session. The subreaper adopts orphan descendants and can subsequently
wait for them; it must stay alive throughout cleanup.
[Linux subreaper interface](https://man7.org/linux/man-pages/man2/PR_SET_CHILD_SUBREAPER.2const.html).

At timeout/failure or observed exit, signal the owned checker and its group before
first reaping it. Observe exit with `waitid(...WEXITED|WNOHANG|WNOWAIT)`. After its
death, enumerate **only the supervisor's own** `/proc/<pid>/task/<tid>/children`,
with a byte/count cap, and verify each candidate is still a direct waitable/running
child using non-reaping waitid before signaling it. With a single waiter and no
automatic reaping, even a child that exits between checks keeps its PID until this
supervisor reaps it. A recorded PID or a /proc name alone does not prove ownership.
[Non-reaping/nonblocking wait semantics](https://man7.org/linux/man-pages/man2/waitpid.2.html).

Prefer signaling each proven direct child by PID. Signal a process group only
when the unreaped owned group leader and its private-session ownership are also
established; never reuse a saved PGID after reaping its leader. Repeat adoption,
signal and nonblocking reap passes until no owned descendants remain or a fixed
cleanup deadline expires. Reparenting is dynamic, so one children-file snapshot
is insufficient. Normal success also requires this cleanup check; children left
behind by a nominally successful checker are a failure, not a successful footer.

Limits remain: inaccessible procfs, subreaper setup failure, child-count overflow,
unexpected ECHILD, permission errors or an unkillable kernel wait must produce an
explicit supervision failure. A finite cleanup loop cannot guarantee that SIGKILL
has completed for every kernel state. Preserve unresolved ownership/status, do
not launch another checker/GPU phase, and do not report Released. Killing the
subreaper itself while descendants remain forfeits the ownership guarantee; an
outer deadline must not reinterpret that as clean teardown. This is a separate
supervisor failure contract, not a CPU-fallback reason. A Python ctypes prctl
wrapper can remain outside the Rust crate's unsafe-code-forbid boundary; a safe
Rustix wrapper is preferable if available in the pinned cache. Neither was run
or added during this review.

## Required focused checks before execution

- Borrowed key lookup: exact/one-over len, capacity and byte caps before table
  growth or comparisons; overreserved sparse map; long absent key; repeated and
  aliased keys; equal content in distinct Arcs; randomized insertion order giving
  identical stable source IDs, packed bytes and admission.
- All-store validation: invalid unused image, hidden malformed referenced image,
  exact/one-over unique byte total, alias counted once, distinct source counted
  twice; no image arena when all image draws are invisible or missing.
- Entire-frame refusal: hidden/transparent Text, radius and opacity; visible
  prefix and suffix retained by CPU fallback; original command count includes
  omitted images; typed mismatches/unclosed scopes; exact/one-over CPU bound.
- Scalar/direct fixtures: one-addition Line lowering, reverse/zero-length Line,
  fixed escape and nested restoration, fractional clips and alpha-zero samples.
  Compare both painters to independently authored literal targets, not each other
  alone. Genuine worker fixtures must additionally verify their command/source
  assumptions without normalizing surprises.
- Supervisor: checker timeout, output flood, inherited pipe holder, worker in a
  distinct PGID, parent exits leaving a grandchild, normal exit with a surviving
  child, adoption during cleanup, delayed exit and cancellation. Assert retained
  ownership/reaping records; do not use post-reap PID absence as ownership proof.

No finding requires changing the existing shaders or original probe oracles.
The two concrete integration risks are unbounded-by-len HashMap scans and the
blocking/separate-group worker teardown boundary. The contract above makes both
explicit while preserving all existing rendering limits and failure evidence.

## Read-only input bindings

The following SHA-256 values identify inspected source, not executed artifacts.

- `/tmp/eris-vulkan-next-step-review.md`: `62d3a2f434a21aeafe01cd3e19d83ef8c5751eaa07d7e3cffe26d1a9b88e0167`
- `src/graphics.rs`: `1b51c4af2c8ae4eea66476397868022da35e566d3593f3fcddf29a156bae62a2`
- `src/worker.rs`: `f4a24a86233ddb68ed7f64657d97653f8e1063104589b7db017fb69371ea4281`
- `src/worker/codec.rs`: `bdad12aa8ddde871aa121ccd6ae4b372e3ff42264d376a5422851771595f7d28`
- `src/worker/channel.rs`: `fa5e11334ea0cace8ac7e5b43c770f77dcaa7ccbf245a71365ff3540d9226d44`
- `src/worker_benchmark.rs`: `460d37f1df4ec99b61358479cb3b67a63a527dc67bae1f66de86905eec99e609`
- `src/browser.rs`: `fbe38ea35b104bf07fca84747f7b65ae8943ad1c7919e0199569d7cf0460a101`
- `Cargo.toml`: `015958175457d712165b649803198aa8aa5bba2f35e19dd49814def7308b03ec`
- `tools/vulkan-raster-probe/Cargo.toml`: `3150058ef3ee321212873e93aea1d745b7a5923faa39808e631ccbef28e4df23`
- `tools/vulkan-raster-probe/src/lib.rs`: `1b0eaa97320f254e96e1df4907f4045c186b3faf1c1f50687c1b9279428437a3`
- `tools/vulkan-raster-probe/src/main.rs`: `996c1b1bc20341f5e005d2bce9d70f0f083fbaaf2aa87d80b817c0891a47fcd2`
