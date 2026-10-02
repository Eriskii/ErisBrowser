# Native buffer reuse: proposed core API

Source-only design; no repository edits, builds or runs. This seals one exact-size
native buffer-set experiment. Existing `Rasterizer::encode`,
`SurfaceConverter::encode`, `EncodedFrame`, `ConvertedFrame`, their destruction
methods, and all Probe entry points remain available with their existing behavior.
The new path is additive and Native-only. A/B policy and switches are outside this
note.

## Identity decision from the pinned dependency

Inspected local registry `wgpu-30.0.1`, not an assumed `global_id()` API:

- `src/api/device.rs:19–26` and `api/queue.rs:14–21`: Device and Queue are Clone
  and implement Eq/Ord/Hash through their internal dispatch handle.
- `src/backend/wgpu_core.rs:750–753`: CoreDevice and CoreQueue equality proxies
  `.id`, unlike ContextWgpuCore's Arc-address comparison.
- `wgpu-core-30.0.1/src/id.rs:55–63`: an Id names a Global's Hub entry;
  `identity.rs:135–146` initializes a manager with index zero and epoch allocation
  starts at one. Standalone Device/Queue equality therefore does not establish
  a process-global cross-Instance identity. Do not use numeric IDs, adapter names,
  device limits, native handle casts, or Device::eq alone as the cache authority.
- Queue and CommandEncoder expose no public owning-device accessor here.
- `api/buffer.rs:338–350`: `size()` and `usage()` return the actual allocation
  descriptor size/usages. Compare these in addition to the planned key.

Use one private `Arc<NativeContext>` created by the new facade. The context owns
cloned Device/Queue handles and its own existing Rasterizer/SurfaceConverter.
All new allocations and writes use these captured handles; encode does not accept
replacement Device/Queue arguments. Requirements and leases retain that same Arc;
compatibility uses `Arc::ptr_eq` plus exact key and buffer descriptors. Creating
two facades over the same pair intentionally produces different cache owners.
This is conservative and does not depend on wgpu's internal ID uniqueness.

The initial Device/Queue must be the pair returned by one `request_device`;
this remains a caller precondition because public wgpu cannot introspect that
relationship. The caller's encoder must also belong to that Device. wgpu error
scopes still validate actual use; the token cannot prove a foreign encoder's
origin. Existing old-path Device/Queue preconditions stay unchanged.

## Public signatures

Place the facade and combined lease in `surface.rs` (already depends on gpu.rs).
New private helpers in gpu.rs expose raster allocation/binding/encode parts only
within the crate. No new dependency, feature or public resource field is needed.
`Result<T>` below is the existing `Result<T, String>` alias.

```rust
// All fields private. NativeEncoder and NativeBufferLease are NOT Clone.
pub struct NativeEncoder { /* Arc<NativeContext> */ }
#[derive(Clone)]
pub struct NativeRequirements { /* same context Arc + checked numeric data */ }
pub struct NativeBufferLease { /* full owned set + private lifecycle state */ }

impl NativeEncoder {
    // Caller pushes validation/OOM/internal scopes first. Calls check before
    // each existing pipeline constructor and after them. On Err no queue writes
    // or frame commands exist; caller still collects every construction scope.
    // Internally creates Rasterizer::new(device, true), then SurfaceConverter::new.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        check: impl FnMut() -> Result<()>,
    ) -> Result<Self>;

    // No buffer allocation/upload/encoding. Reuses the current native Plan,
    // BufferAccounting, SurfaceLayout and device-limit checks.
    pub fn requirements(
        &self,
        plan: &Plan,
        format: wgpu::TextureFormat,
    ) -> Result<NativeRequirements>;

    // Checks requirements' context identity and calls check before allocation.
    // All fallible Rust guards precede first buffer creation. Then construct
    // the bounded full set without any intervening Result-returning callback,
    // and return it intact. Driver validation/OOM arrives via caller scopes.
    // No queue writes or encoder commands occur here.
    pub fn allocate(
        &self,
        requirements: &NativeRequirements,
        check: impl FnMut() -> Result<()>,
    ) -> Result<NativeBufferLease>;

    // Caller owns the lease before this call, including on EVERY Err. The
    // encoder may contain a partial prefix and queue writes may exist on Err.
    // Recompute requirements from the current Plan/format before using buffers.
    pub fn encode(
        &self,
        lease: &mut NativeBufferLease,
        plan: &Plan,
        format: wgpu::TextureFormat,
        encoder: &mut wgpu::CommandEncoder,
        check: impl FnMut() -> Result<()>,
    ) -> Result<()>;
}

impl NativeRequirements {
    pub fn buffer_bytes(&self) -> u64;
    pub fn layout(&self) -> SurfaceLayout;
}
impl NativeBufferLease {
    // False unless Fresh or Reusable, exact context/key and actual descriptors.
    // No uploads, allocations, lifecycle mutation or ownership transfer.
    pub fn is_compatible(&self, required: &NativeRequirements) -> bool;
    pub fn buffer_bytes(&self) -> u64;

    // Only valid after encode returned Ok; private state rejects all other use.
    pub fn output_buffer(&self) -> Result<&wgpu::Buffer>;
    pub fn layout(&self) -> Result<SurfaceLayout>;

    // Encoded -> Reusable only. Caller must first prove the exact submission
    // retired, collect all three scopes successfully, and resolve its own copy,
    // verification, present/cancellation policy. Does not poll or pop scopes.
    // On Err the caller STILL owns the unchanged lease.
    pub fn mark_reusable_after_completion(&mut self) -> Result<()>;

    // Same explicit caller-completion precondition as existing destroy methods.
    // Legal for unused Fresh or caller-retired leases; never for uncertain uses.
    pub fn destroy_after_completion(self);
}
```

There is no error type that consumes the lease and accidentally drops its only
handles: `encode`/recycle borrow it mutably. Initial allocation errors occur
before frame resources are created, or arrive later through caller scopes while
the returned full lease is retained. Infallible wgpu constructors can report
asynchronous error objects; `buffer.size()` is not proof of allocation success.
Panics and synchronous driver-call interruption are not newly solved by this API.

A single lifecycle enum suffices: **Fresh, EncodingFailed, Encoded, Reusable**.
`encode` first verifies it was Fresh/Reusable and sets EncodingFailed before any
potential queue write; it sets Encoded only after complete conversion encoding.
Any encoding error permanently poisons that lease for reuse, including a callback
refusal before writes. Invalid repeated encode attempts must also poison/reject,
never reset an uncertain state. `mark_reusable...` accepts only Encoded. No Drop
implementation explicitly destroys GPU buffers. Ordinary drop retains wgpu's
existing recorded/submitted-reference lifetime behavior.

This API deliberately does not pretend that a safe Rust method can prove an
external submission index retired. The presenter must maintain its private
`retired && scopes_ok && complete_frame_ok` witness; only that branch calls
`mark_reusable...`. Tests enforce this owner transition. It is the same caller
completion boundary as existing explicit destruction, now also guarding reuse.

## Exact private key and resource set

The numeric key includes Native profile, target width/height, format, padded
bytes-per-row, packed/padded target byte sizes, draw-parameter length, input
length/presence, and uniform alignment used in admission. Pipeline ownership is
provided by the context Arc. No serial, generation, clear color, source contents,
draw contents or total invocation count belongs in the reuse key: those are
current-frame metadata which is revalidated and rewritten on every encode.
Current SurfaceLayout/invocation metadata is copied from the newly checked Plan,
not restored from a previous frame's requirements.

One lease owns the following entire graph, never independent reusable slots:

| Buffer | Exact bytes | Exact usage |
|---|---:|---|
| Packed RGB target | width × height × 4 | STORAGE \| COPY_SRC |
| Draw uniforms | current Plan parameter length | UNIFORM \| COPY_DST |
| Optional source/input arena | current Plan input length | STORAGE \| COPY_DST |
| Padded conversion target | padded row bytes × height | STORAGE \| COPY_SRC |
| Conversion uniform | 16 | UNIFORM \| COPY_DST |

It also owns rectangle, optional image/glyph-input, and conversion bind groups,
plus the context/pipeline references. This is five buffers/three groups for the
measured input-backed scene; clear/rectangle-only work still has **no dummy input
buffer**, so four buffers/two groups there. Preserve existing omissions.

`is_compatible` checks every `.size()` and `.usage()` for exact equality with this
table, input presence, private key and Arc owner identity. Shader target usage
never gains COPY_DST. No capacity rounding, oversized backing allocations or
mapped/readback buffers enter the lease. The checked sum returned by
`buffer_bytes()` must equal both the actual five/four descriptor sizes and the
current `Plan::gpu_buffer_bytes()` including conversion's 16-byte uniform.
Readback is excluded and remains separately charged by the presenter. This
explicit buffer sum retains the existing driver/binding/allocator-overhead
exclusions; it is not a VRAM/RSS measurement.

On every encode, preserve source/input write, full draw-uniform write, ordered
clear/draw passes, conversion-uniform write and conversion pass. Do not skip
writes merely because bytes match. No command buffer, acquired texture, CPU
reference or browser Plan is retained. Full viewport clear remains the first
operation and every color pass retains its current shader/binding/order rules.

## Presenter use and error ownership

The existing native Kernels owner can contain the facade instead of two public
pipeline handles; its construction still happens under the same three scopes
and deadline checks, with the same pipeline order. Core never creates an
Instance, adapter, surface, submission, polling loop or readback. It allocates
only the admitted resource graph and appends commands to the caller's encoder.

The presenter owns `Option<NativeBufferLease>` for at most one idle set. It calls
requirements and checks the unchanged Plan/readback/application budgets first.
A compatible set moves from the cache to the active local owner; an incompatible
retired set is explicitly destroyed/dropped before allocating its replacement.
There is no simultaneous complete old/new graph. On cache miss allocate a Fresh
lease after reserving its exact bytes. Choosing never to recache yields a fresh
allocation each frame using this same API; root chooses the experimental A/B
strategy separately.

Before encode, preserve `QueueProgress::before_encode()`. An Err leaves the lease
in the presenter's local variable and may leave queued writes: discard the
partial encoder, empty-submit once if needed, wait for that exact submission,
collect all scopes, then destroy only if completion is known. Never put a
poisoned lease in the cache. A poll/device/scope/deadline failure keeps uncertain
handles until the existing whole-owner teardown; it does not become a cache miss
or CPU fallback. New readback/copy/present errors follow the same rule. The full
successful branch alone marks Reusable and moves the lease to the idle cache.

Transfer its byte charge atomically between retained and active fields under
Shared's mutex; `idle()` must not erase a retained charge. Eviction releases the
charge only after the owned graph is destroyed/dropped. On full release clear
it only after Gpu/context/cache drops. Preserve Native16MiB, active+readback24MiB,
application128MiB and next-UI16MiB bounds. CPU-upload admission includes any idle
native set or safely evicts it first. No reference/readback retention is added.

## Minimal verification additions

Pure tests can use a private key/descriptor model without constructing devices:
all individual key mismatches, 4/5-buffer sums and overflow, exact usages, context
identity mismatch, lifecycle rejection after every error, and retained/active
ledger transfer. A real-device follow-up must also check that two contexts over
identical-looking devices cannot exchange leases, same-size changed inputs and
clear colors overwrite prior contents, and changed sizes/formats evict before
replacement. Existing Probe and old encode tests remain intact. Cancellation,
empty-submit flush, late scope/poll failures and owner release tests must prove
that no uncertain lease is recached. No tests or GPU runs were executed for this
note.

One remaining performance boundary is explicit: pinned Queue::write_buffer's
own documentation (`api/queue.rs:140–175`) says native staging memory is newly
allocated per call and released after submission. This proposal leaves those
staging allocations unchanged. Reusing destination buffers does not establish
that all allocation or the observed early submission tail disappears.

## Inspected source bindings

- `crates/raster-core/src/gpu.rs`: `6897765438995db542e40a2f91223a597c3dcc2e2bb8dd89beca1576050a9f69`
- `crates/raster-core/src/surface.rs`: `a8057e3641f98120a7a4e3bd4445facf814cad51f0490cf50a7c7090e4575869`
- `src/presenter/vulkan/native.rs`: `5b7d0127b89f47037d2b09c13931150713c7043640d99f3906b1f736c296a327`
- `src/presenter/vulkan/control.rs`: `cefca28896617574f1a66f5d019849139fc22c03d9f7f39de830b3f1d3da8268`
- `wgpu-30.0.1/src/api/device.rs`: `6adbd3b5bb966fb20601cd55f0cf1b0cffd48fc615c72029dfe3c116b9c88e8f`
- `wgpu-30.0.1/src/api/queue.rs`: `14dae3b36f75911a5ac4ed556d48875930e097990649110f9a5e53deada8d846`
- `wgpu-30.0.1/src/api/buffer.rs`: `a304040166251cb1f1e60d6e1407b2e951dc3836144b40b95852749005095042`
- `wgpu-30.0.1/src/backend/wgpu_core.rs`: `1a9e1f8cb8c7c2dc9d23dce7e22cd9c7b06ce57c3a053189d2fb22aede8860fa`
- `wgpu-30.0.1/src/dispatch.rs`: `86d9f89d32841f16fb3ef35b49125ad87470f2d4fb17f67e30ddae102cd72bf1`
- `wgpu-core-30.0.1/src/id.rs`: `529303b55ca4f159ee224540c798ebc4421e93256d49ef16a4ad775624c427e6`
- `wgpu-core-30.0.1/src/identity.rs`: `712ccda267eb602655c53672a94346aa4c9f88c0044ae6edcd658a814c9a50cf`
