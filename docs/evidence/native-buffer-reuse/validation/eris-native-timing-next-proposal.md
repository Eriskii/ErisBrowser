# Proposed next change: reuse one retired native buffer set

**Proposal only; no source change, build, benchmark or GPU execution performed.**
Keep one exact-shape, owner-local native buffer set after successful retirement,
and reuse it for the next compatible frame. Continue rebuilding the Plan,
rewriting all input/parameter bytes and executing every ordered draw and the
surface conversion. This is allocation reuse, not scene, glyph or framebuffer
content caching.

## Measurement supporting this choice

Read-only input: `/tmp/eris-native-timing-1/host-1/timing-host-results.json`,
SHA256 `107ea53d4118ff1bd35e5ee8d57d969b7bc1980fe5b54e11d4113d9444154638`.
All eight runs succeeded with release
`038ffe37d58b0e27189ee3681ec6a84ab183beba9433f4a22cca766e47af08de`.
One NVIDIA GeForce RTX 4070 SUPER, Bgra8Unorm FIFO, 1280×880, three fresh
processes per route, 16 measured frames per process. The following are pooled
**45 steady observations per route**, excluding only the explicitly separate
first frame. Units are milliseconds; p95 is the existing nearest-rank statistic.

| Direct measured phase | CPU median / p95 | Native median / p95 |
|---|---:|---:|
| UI preparation | 0.503 / 0.539 | 0.276 / 0.330 |
| Encode/upload | 1.476 / 1.662 | 0.383 / 0.425 |
| Submission | 0.023 / 0.032 | 0.273 / 2.866 |
| Completion wait | 0.344 / 0.414 | 0.704 / 0.881 |
| Cleanup | absent | 0.361 / 0.381 |
| Preparation through present | 2.429 / 2.625 | 2.085 / 4.830 |
| Owner total | 1.925 / 2.091 | 1.788 / 4.553 |

Every native process's first **and second** frame exceeds 4 ms through present.
Their submission phases are respectively 3.138/3.015, 3.462/2.866 and
4.076/3.529 ms. This repeated early-run pattern is consistent with deferred
driver/command work, but does not identify its cause. `submit_ns` includes
`encoder.finish()` and `queue.submit()`. It is not a GPU timestamp. OS scheduling,
driver initialization and other host/device activity remain possible causes.
The small number of processes cannot establish a general tail distribution.
Do not delete frame 2 or reclassify it as warm-up to improve the headline p95.

The code provides a separate concrete optimization target: each admitted frame
creates five buffers for this input-backed scene (packed RGB, draw parameters,
input arena, padded converted pixels, conversion parameters), three bind groups,
then explicitly destroys all five buffers after tracked completion. The two
pixel buffers alone total **9,011,200 bytes** at 1280×880; the conversion uniform
adds 16 bytes, and draw/input arenas add their exact Plan sizes. The measured
cleanup phase includes scope collection as well as destruction, so its full
0.361 ms median must not be attributed to destruction. Reuse removes known
allocation/destruction operations; improvement in early submission tails is an
experimental hypothesis, not an asserted result or estimated speedup.

## Small implementation boundary

- `crates/raster-core/src/gpu.rs:218`: retain the existing `Rasterizer::encode`
  public behavior. Factor allocation/binding construction into a private owned
  resource bundle and add an explicitly Native-only reuse entry point. Keep
  Plan admission, alignment/device checks, unconditional clear and per-draw
  callback/pass order identical.
- `crates/raster-core/src/surface.rs:307`: similarly separate converted output,
  16-byte uniform and conversion bind group from per-frame metadata. One
  non-Clone lease owns the raster and conversion resources together. The old
  `EncodedFrame`/`ConvertedFrame` APIs and Probe routes continue to allocate and
  destroy exactly as before.
- `src/presenter/vulkan/native.rs:103`: store at most one idle bundle in the
  existing `Gpu`/`Kernels` owner, move it into the next frame lease, then return
  it only after successful tracked completion, scope collection and successful
  frame processing. `src/presenter/vulkan.rs:688` remains the sole owner loop.
  No UI thread, callback, Snapshot, Font session, packet, acquired surface
  texture or CPU reference owns/cache-shares this bundle.

The exact compatibility key is owner/device/pipeline identity, Native profile,
width, height, output format and padded stride, draw-parameter byte length,
input byte length and input-presence flag. Compare actual buffer sizes/usages
before reuse. A different scene with the same sizes is compatible because all
metadata/input bytes are overwritten and the full output is cleared. Recompute
all per-frame layout/invocation metadata from the current admitted Plan; never
reuse old frame stamps, lengths, draw lists or uniforms implicitly. Retain the
same STORAGE/COPY_SRC target usage, without CPU target uploads or COPY_DST.

Use exact sizes for the first increment: no geometric growth, best-fit cache,
multiple slots, persistent command buffers, buffer aliasing, or content hashes.
This avoids hidden unused capacity and makes the explicit resource sum equal to
the Plan's existing `gpu_buffer_bytes()`. An incompatible idle set is destroyed
and dropped before replacement is allocated; old and new complete sets never
overlap. Initialization remains lazy at the first admitted frame; there is no
unmeasured synthetic draw or pipeline warm-up hidden in the benchmark.

## Ownership and resource constraints

1. A bundle has only `IdleCached -> InFlight -> IdleCached` on a fully successful
   frame, or `... -> Discarded/OwnerRelease` on failure. Never write a reused
   buffer until the previous submission's specific index is retired. No second
   frame may encode against the same bundle while work is outstanding.
2. Preserve the current `QueueProgress::before_encode`, empty-submit flush on
   partial/cancelled encoding, and prohibition on submitting an incomplete draw
   prefix. Any queued writes still require tracked retirement. Keep all three
   error scopes and original deadlines. An unresolved poll/scope/device error
   drops ordinary handles as part of whole-owner release; it never explicitly
   destroys or returns the bundle to the idle cache early. A cancelled frame
   may be safely discarded after its flush retires; initially do not recycle it.
3. Add a checked retained-native-buffer byte term to
   `State::live_bytes()` (`control.rs:226`). At lease checkout/return transfer
   the exact charge between idle-retained and active terms, without double
   charging or briefly dropping it. `idle()` currently zeros active GPU bytes
   (`:759`); that must not erase the cache's charge. `released()` zeros it only
   after the outer owner has dropped every API object (`:845`).
4. Keep the current 16 MiB Native Plan cap, 24 MiB active native+readback cap,
   128 MiB application ledger and 16 MiB next-UI reservation. Readback remains a
   separate per-check allocation, never cached or introduced for timing.
   Explicitly include an idle set when admitting pending/UI/CPU-upload work.
   Evict it before the CPU route needs its existing scratch budget, and on
   incompatible reconfiguration; never raise a cap to preserve the cache.
5. Keep existing driver/allocator-overhead exclusions explicit. API object and
   binding overhead is not a claim of measured VRAM. The cache lives only on
   the surface owner and is destroyed on owner release; occlusion/size/format
   changes may evict it after all uses retire.

## Fallback and acceptance tests

On cache miss or key mismatch, use the original one-shot allocation path after
safe eviction. If reuse admission cannot preserve the existing accounting,
evict and use that path; do not convert a device/allocation/scope failure into
success or a late CPU fallback. Existing complete-scene CPU admission fallback
stays before native submission. Keeping the old path also provides a narrowly
controlled performance A/B without altering shaders, planner limits or inputs.

Before actual runs, add pure compatibility/accounting and ownership-state tests:
exact reuse versus each key mismatch; one-byte budget boundary; cached charge
survives idle and participates in pending/UI/CPU reservations; eviction before
replacement; no reuse before retirement; cancellation after every queued-write
cutpoint flushes once; scope/poll/deadline failures never recache; shutdown drops
cache before release. Existing clean-exec/lease tests remain required.

GPU correctness should alternate equal-sized but **different** scenes and input
bytes, opaque and alpha draws, image/rounded/text interleaving, hidden/empty work,
then shrink/grow/format changes. Verify that the unconditional clear prevents
stale pixels and metadata/input overwrite prevents stale draws. Replay the old
Probe suites unchanged and both surface formats, with distinct correctness
processes. A test-visible allocation counter should show one bundle allocation
for compatible frames and an eviction/reallocation for mismatches; do not add
new clocks to the disabled production path.

Only then repeat the same single-URL, exact-scene, 16-frame/three-pair timing
protocol and retain all samples/failures. Compare direct first-frame/steady
prepare-to-present and owner-total distributions, plus encode, submission and
cleanup phases. A lower cleanup/encode cost without a lower early submission
tail is still possible; report that distinction. No improvement or Chromium
comparison is claimed from this proposal.

## Source bindings inspected

- `src/presenter/vulkan/native.rs`: `5b7d0127b89f47037d2b09c13931150713c7043640d99f3906b1f736c296a327`
- `src/presenter/vulkan/control.rs`: `cefca28896617574f1a66f5d019849139fc22c03d9f7f39de830b3f1d3da8268`
- `src/presenter/vulkan.rs`: `50d90b8664b136bf1713e073e4155ce01182260eba43b91c9e2edb3b5c536ddb`
- `crates/raster-core/src/gpu.rs`: `6897765438995db542e40a2f91223a597c3dcc2e2bb8dd89beca1576050a9f69`
- `crates/raster-core/src/surface.rs`: `a8057e3641f98120a7a4e3bd4445facf814cad51f0490cf50a7c7090e4575869`
- `crates/raster-core/src/lib.rs`: `942ece33951829f3e6640b8305a0e5ea5979077ed0e54bd8ec97327b78b4363e`
