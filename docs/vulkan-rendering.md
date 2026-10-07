# Vulkan rendering milestones

An optional Linux **native shader route** is now available with `vulkan-raster`,
`--presenter=vulkan --raster=gpu`. It composes admitted loaded pages, overlays and
browser controls through custom WGSL into the actual acquired window texture.
CPU-prepared font and rounded masks supply coverage; GPU kernels perform color
compositing and surface conversion. Normal native frames contain no CPU target
or readback. Unsupported or over-budget scenes use the complete CPU painter.

The [native-window record](vulkan-native-window.md) now includes a gated 1280×880
comparison (4,505,600 acquired bytes) using two-strip rounded coverage and a
separate reference-free run. The earlier 1180×880 comparison, opacity/overdraw
fallback and initial refusals remain tied to their original executable. These
checks cover one NVIDIA surface/format before the compositor. Pixel agreement
alone establishes no speedup or Chromium comparison.
The default and headless paths remain software. `--presenter=vulkan` alone retains
the existing CPU upload route.

The native owner now retains one exact-size buffer lease after successful
retirement, scopes and presentation. Every frame still rewrites all inputs and
uniforms and executes the full clear/draw/conversion sequence. Mismatched shapes,
reconfiguration and CPU fallback evict before replacement. Idle retains the
charge under the existing 16 MiB native / 24 MiB with readback / 128 MiB presenter
bounds; failed or uncertain work cannot return to the cache. The
[reuse record](evidence/vulkan-native-buffer-reuse.json) separates three-adapter
changed-content checks from an actual native-window A/B comparison. Its exact
one-scene inputs and timing limits are described in the
[benchmark record](native-render-benchmark.md#native-baseline-versus-retired-buffer-reuse).

The [raster core](../crates/raster-core/README.md) and browser adapter now provide
whole-scene planning on the existing graphics owner. Earlier
[native prerequisites](../tools/vulkan-raster-probe/NATIVE_PREREQUISITES.md),
[glyph](../tools/vulkan-raster-probe/GLYPHS.md) and
[worker-text](../tools/vulkan-raster-probe/WORKER_TEXT.md) results remain separate
offscreen checks on three adapters. Native page zoom now scales admitted scenes
within the same resource limits. Broader viewport/scene admission, transparent
group opacity, shaping and native GPU screenshot capture remain unfinished.
The [bounded opacity route](vulkan-opacity.md) now admits nested groups with a
proved opaque rectangular backing and `k/256` opacity. Transparent backings and
arbitrary opacity values still use the complete CPU route.
The [integration design](vulkan-native-plan.md) records the original seams and
ownership requirements. Earlier experiments and failed compositor comparisons
remain recorded below.

Build and select the experimental presenter explicitly:

```sh
cargo build --locked --release --features vulkan-presenter
./target/release/eris-browser --presenter=vulkan ./examples/forms.html
```

The host must expose its installed Vulkan loader/ICD and desktop libraries to
the process. No driver is bundled or installed by the browser. Requesting Vulkan
in a build without the feature, on another platform, or in a headless mode is a
CLI error before window creation. `--vulkan-verify-frames N` (1–8) additionally
requires COPY_SRC and compares every packed byte of distinct acquired textures
before presentation. It fails on incomplete sampling, fallback, mismatch, stall
or device error. It does not drive redraws to manufacture samples; use real
content/viewport changes. Existing CPU screenshots are not GPU evidence.

The current [browser validation record](VALIDATION.md#optional-vulkan-upload-presenter)
and [source-bound evidence](evidence/vulkan-presenter.json) distinguish successful
readback, retained failed launch attempts, CPU references and untested scenarios.

## CPU upload boundary

The page process produces the custom renderer's `DrawCommand` list, hit regions
and decoded images. The parent validates that snapshot in
[`worker/codec.rs`](../src/worker/codec.rs). Native
[`Browser::draw`](../src/browser.rs) paints the snapshot with
[`Canvas::paint_with_viewport`](../src/graphics.rs), adds browser chrome, saves
any CPU screenshot, and transfers the completed Canvas to the
[`presenter facade`](../src/presenter.rs). Software validates and borrows its
pixel words without conversion. Vulkan moves the pixel Vec without cloning,
drops the rest of the Canvas, and uses a dedicated owner thread. Pixel words are
`0x00RRGGBB`; explicit BGRA/RGBA conversion always supplies alpha 255.

The UI retains only CPU scheduling state and a thread handle for Vulkan. The
owner thread holds all graphics API objects. Software surface construction is
permitted only after that owner positively acknowledges actual resource drops.
A stuck native call or destructor can therefore prevent fallback. Headless
rendering never initializes this presenter or Vulkan.

`Canvas` already owns the relevant paint semantics: command order, separate
document and fixed offsets, typed clip/fixed/opacity scopes, glyph masks,
nearest-neighbor images, rounded coverage, integer alpha blending and RGBA16
group-opacity intermediates. Its limits include 200,000 commands, 128 combined
scopes, a 16-megapixel framebuffer with an 8,192-pixel axis limit, bounded glyph
and pixel work, and deferred, bounded opacity allocations. A Vulkan path must
preserve those semantics and the explicit exhaustion indication.

The page sandbox denies GPU device access, almost all ioctls and new threads.
GPU initialization belongs outside that process. This implementation does not weaken
the page, broker or image-decoder sandbox.

## Host inventory and its limits

A 20-second-bounded diagnostic loaded an existing Vulkan loader, enumerated
physical devices and queue families, checked the swapchain extension, then
created and destroyed an empty logical device with one graphics queue for each
device. Instance creation and all three device creations returned `VK_SUCCESS`.
No commands, shaders, surfaces or swapchains were created.

| Item | Observed result |
| --- | --- |
| Loader | `libvulkan.so.1`, API `1.4.357` |
| NVIDIA device | GeForce RTX 4070 SUPER, vendor/device `10de:2783`, API `1.4.329`, discrete GPU |
| AMD device | AMD Ryzen 7 7800X3D 8-Core Processor (`RADV RAPHAEL_MENDOCINO`), `1002:164e`, API `1.4.354`, integrated GPU |
| Software device | llvmpipe (`LLVM 21.1.8, 256 bits`), API `1.4.354`, CPU device |
| Queue and extension | Graphics queue family 0 and `VK_KHR_swapchain` on each device |
| Desktop environment | Wayland session, with a Wayland display and X11 display advertised |

The loaded library was
`/nix/store/iahmf8kl215rjcvb3zz3l7i9w2sf9x5v-vulkan-loader-1.4.357.0/lib/libvulkan.so.1`.
Existing ICD manifests pointed to Mesa 26.2.3 and NVIDIA 595.99.02 libraries.
`vulkaninfo` was absent. The diagnostic used a temporary Python/ctypes program,
not an Eris graphics implementation; its session-local path was
`/tmp/eris-vulkan-inventory.py` and is not a repository test artifact.

The outer tool sandbox exposed PCI/sysfs inventory and loader files but hid
`/dev/dri` and `/dev/nvidia*`. The successful diagnostic ran outside that outer
sandbox under the existing host user. It installed nothing and changed no
settings. This distinguishes host availability from availability inside Eris's
deliberately restricted page process.

This result proves basic host loader/device initialization only. It does not
prove Wayland or X11 presentation, any particular surface format/usage, shader
execution, validation-layer availability, GPU recovery, CI availability or
performance. The host API versions are observations, not minimum requirements.

### Reproducible offscreen transfer probe

The independent [Vulkan probe](../tools/vulkan-probe/README.md) now exercises
safe Rust wgpu 30.0.1 APIs with only its `std`, `vulkan` and `wgsl` features.
Each of the three observed adapters passed six exact RGBA8 upload/readback
comparisons: three changed frames at 320×240 and three at 319×239, with the
second size exercising padded readback rows. The
[compact evidence](../tools/vulkan-probe/evidence/host-transfer.json) binds the
results to the published source and lockfile hashes.

The probe uses finite application waits and a separate process deadline per
adapter. It has no surfaces, shaders or browser integration, and establishes no
GPU rasterization, presentation or speed result. The existing host loader was
not discoverable by basename, so the successful run supplied its directory only
to the probe subprocess environment. The portable runner accepts an optional
loader directory; it contains no hardcoded host path. Driver calls and teardown
can still block beyond application waits, as documented in the probe README.

The pinned Linux graph has 58 dependencies. Declared MSRVs are compatible with
1.88, but ten packages omit that field. The recorded host build used Rust 1.95;
the subsequent [Rust 1.88 CI job](https://github.com/Eriskii/ErisBrowser/actions/runs/36396836593/job/108845129192)
also passed compilation, formatting, strict Clippy and the Rust/Python checks.
GPU execution was confined to the separately recorded host run. This is an
isolated dependency evaluation, not a change to the browser's dependency graph
or completion of milestone A.

### Isolated native surface experiment

A separate temporary safe-Rust wgpu 30.0.1/winit 0.30.13 prototype created one
owned native window. All three host adapters reported a compatible surface;
only the NVIDIA adapter was used to present. It selected `Bgra8Unorm` with
`Srgb` display color space, opaque composition, FIFO, and
`COPY_DST | COPY_SRC | RENDER_ATTACHMENT`. No shader, render pipeline or render
pass was created. The prototype uploaded opaque CPU pixels directly into the
acquired surface texture, copied that same texture to an aligned readback
buffer, compared every color/alpha byte, and then presented it.

Five acquired-surface readbacks were exact, totaling **3,316,800 bytes**. They
covered startup at 320×240, a 1.5-scale transition to 480×360, changed frames,
and resize to 600×390. Packed rows of 1,920 and 2,400 bytes used readback rows
of 2,048 and 2,560 bytes. The window exited normally and its tracked process
was reaped. This demonstrates the transfer path on that surface configuration,
not a browser implementation or a portable surface capability guarantee.

**Full compositor captures did not match exactly.** The first capture differed
at all 172,800 pixels; its cause remains unproven. Three later captures each
differed at 252 pixels, all within the four 16×16 corner squares, consistent
with the visibly rounded window corners. Inset comparisons are diagnostic
only and do not turn those failures into passes. A same-frame cursor move
strictly inside the owned window did not establish the cause of an earlier
central shape because elapsed time also changed. No cursor-inclusion flag was
used. GPU readback verifies texture bytes before the compositor; it does not
prove exact final display pixels.

The [compact evidence](evidence/vulkan-native-surface.json) preserves the exact
readback records, full mismatch counts and source/binary/log/capture hashes.
The source and host-specific supervisor were independently reviewed in a
temporary directory; they are not published as a portable test here. Private
capture geometry and images are omitted. The prototype used five-second
application waits, a cooperative twenty-second lifetime and a separate
twenty-five-second process-group watchdog. Forced driver-hang recovery was
not exercised, and internal driver calls or kernel teardown can outlast
application waits. No driver or compositor settings were changed.

Local formatting, strict Clippy and build checks passed on Rust 1.95. This
separate native crate was not built on Rust 1.88; the published offscreen
probe's CI result does not establish its MSRV. This remains an isolated native
presentation experiment, with no GPU rasterization or performance claim.

## Milestone A: Vulkan presentation

The optional implementation uses the following boundary. Broader platform,
fault-injection and compositor coverage remain open:

```text
Presenter = Software | Vulkan
Target = { generation, viewport_revision, physical_size, occluded }
Frame = { serial, generation, viewport_revision, width, height, opaque_pixels }
submit(Frame) -> Presented | Queued | Replaced | IgnoredStale | Stopped
```

Selection is explicit through `--presenter=software|vulkan`. Software remains
the default and the headless CPU path remains the reference.

1. Select an actual Vulkan adapter compatible with the window surface. Record
   its name, device type, driver and selected surface configuration. Report a
   CPU Vulkan adapter such as llvmpipe explicitly.
2. Upload the complete CPU frame, including chrome. Convert `0x00RRGGBB` to the
   selected RGBA/BGRA byte layout and set alpha to 255. Respect upload-row
   alignment with checked arithmetic. Do not accidentally interpret the unused
   softbuffer high byte as transparent alpha.
3. Use a transfer to a supported surface texture when possible. Query format,
   color space, alpha mode and usage support first; transfer-destination usage
   is a capability, not a universal assumption. Initially use software when
   the copy path cannot preserve the pixel values. A later fullscreen blit
   shader needs its own color-conversion tests. See
   [Vulkan surface capabilities](https://docs.vulkan.org/refpages/latest/refpages/source/VkSurfaceCapabilitiesKHR.html).
4. Keep one presenter as the exclusive owner of a native surface. Never operate
   softbuffer and Vulkan concurrently on the same window surface. A clean
   fallback waits for the former owner to release it.
5. Carry frame serials and navigation generations through the bounded queue.
   Coalesce pending resize/frame requests and discard obsolete pending work.
   Do not retain a history of full frames while the compositor is busy.

Native testing exposed two descriptor problems: transient loader manifest files
opened without close-on-exec, and a retained `/dev/udmabuf` descriptor after
hardware initialization. The page worker refused both before accepting page
content. A graphics/spawn mutex was tested and rejected as insufficient because
it cannot eliminate retained descriptors or independent driver threads.

Feature-enabled builds now enter an exec-only launch stage before each worker.
It validates the internal role and single-thread startup, marks every descriptor
above stdio close-on-exec through the safe `close_fds` API, then execs the same
binary in the requested worker role. Parent handles are unaffected. This stage
initializes no driver and handles no page data. The worker's existing strict
inspection remains after exec; a descriptor missed by sanitization still causes
rejection. Default builds retain their original direct launch path.

The GPU work in this milestone is transfer and presentation. The page is still
rasterized and composited by `Canvas`. Existing CPU screenshot output by itself
cannot verify that the Vulkan copy or presentation was correct.

### First integration contract

A review of the pinned binding and current browser identified these requirements
for implementation. The current owner follows these constraints; tests and host
evidence must still distinguish application simulations from actual driver behavior.

- Keep one active upload and one replaceable pending frame. Navigation and
  physical viewport changes advance the target epoch, including a resize back
  to earlier dimensions. Explicit occlusion/restore events resume a deferred
  frame; idle, zero-size and occluded states have no progress deadline. A frame's
  epoch identifies the current UI request, which may still display the prior
  document while navigation loads.
- Treat `Queue::write_texture` as a state change: it copies bytes into staging
  before submission. If the epoch changes after that call, flush and retire the
  queued upload without presenting, or release the owner. Do not start another
  upload or release its accounting merely because no submission occurred yet.
  Recheck epochs before conversion, writing and presentation; a native call
  already in progress cannot be atomically canceled.
- Attach fixed deadlines to initialization, active work and teardown. Pending
  replacement, resize traffic and notices must not postpone them. The UI services
  expiry even without redraws. Expiry requests shutdown and marks the owner
  stalled; it neither establishes release nor permits another native surface.
- Keep lifecycle state durable under a short lock; a coalesced event only wakes
  the UI. Observing state and clearing its wake flag must be atomic. Never hold
  that lock during a driver call or thread join, and never let a late ready
  notice overwrite stop, stall or confirmed release.
- Retry an Outdated surface configuration once. A Lost surface or device-loss
  event instead requests clean release and fallback in this first slice.
  Consume or drop acquired textures, including suboptimal ones, before any
  reconfiguration. Install bounded, nonpanicking error callbacks before GPU
  work; callbacks must not retain devices, queues or surfaces in ownership cycles.
- Publish release only after actual resource drops return. Unexpected thread
  panic, a timeout and a requested stop are insufficient. Join only a finished
  thread. A later confirmed release may enable software fallback after a stall.
- A bounded verification option must count distinct completed frame serials and
  fail its process result on fallback, missing copy-source support, insufficient
  samples, mismatch, device error or stall. Acquired-texture readback verifies
  bytes before the compositor; neither CPU screenshots nor a present call prove
  exact visible output.

These requirements follow the pinned
[queue](https://docs.rs/wgpu/30.0.1/wgpu/struct.Queue.html),
[surface](https://docs.rs/wgpu/30.0.1/wgpu/struct.Surface.html),
[acquisition outcomes](https://docs.rs/wgpu/30.0.1/wgpu/enum.CurrentSurfaceTexture.html)
and [device](https://docs.rs/wgpu/30.0.1/wgpu/struct.Device.html) APIs. CPU tests
should inject invalidation before and after queued writes, loss, callbacks,
delayed release and stuck cleanup. Queue and resize storms must preserve fixed
deadlines, bounded storage and exclusive surface ownership. Such simulations
do not establish recovery from a real driver hang.

## Milestone B: custom GPU rasterization

The isolated [raster probe](../tools/vulkan-raster-probe/README.md) executes custom
WGSL compute shaders for ordered unrounded rectangles and nearest-neighbor
images with source alpha over opaque RGB. Its [alpha host evidence](../tools/vulkan-raster-probe/evidence/host-alpha.json)
records **30 exact offscreen fixtures per NVIDIA, AMD and software Vulkan adapter,
totaling 2,764,860 compared bytes**. All four processes, including enumeration,
exited normally with empty stderr. The original seven rectangle and fourteen
image fixture definitions and protocol tuples are unchanged. Their
[rectangle](../tools/vulkan-raster-probe/evidence/host-raster.json) and
[image](../tools/vulkan-raster-probe/evidence/host-images.json) evidence retain
their original source/binary attribution.

The nine alpha literal targets cover alpha 0/1/127/128/254/255, draw order, repeated
rounding, clip/fixed behavior and transparent hidden RGB. Each ordered pass uses
Canvas's integer source-over formula, with a zero target high byte. Valid
alpha-zero rectangles are validated then omitted; transparent image samples
retain the planned dispatch and skip their GPU store. Clear stays unconditional
and opaque. Translucent rectangles enforce both half-open clip edges, while
opaque rectangles retain the prior fast-path coverage. There is no CPU alpha
scan, transparent-image suppression or color-space conversion.

The GPU receives original source colors, metadata and separable sampling tables
that preserve Canvas's scalar f32 operation order. Custom shaders perform the
clear and all target writes; the target is never uploaded. Plans own immutable
data, validate every supplied source, and retain the existing 1 MiB explicit
GPU-buffer and four-million-invocation limits. Those bounds exclude driver and
upload staging allocations. This remains a standalone prototype with its own
manifest and lockfile. The alpha checkpoint records 25 tests on each of Rust
1.88 and 1.98, formatting, strict Clippy and builds, ten Python tests and two
offline Naga shader checks. Its initial test-helper type-inference failure and
three-line explicit-`u32` correction remain preserved with that historical evidence.

The optional [browser bridge](../tools/vulkan-raster-probe/BROWSER_BRIDGE.md)
now borrows a validated worker snapshot or original display list and image store.
It accepts unrounded rectangles and nearest-neighbor images with source-over
onto opaque RGB, Canvas-style line rectangles and balanced clip/fixed scopes.
It preserves image-key resolution, shared image identities, missing-image
no-ops and earlier placeholder commands without cloning snapshots or staging an
extra RGBA copy; final validated plans own their packed source data.
Hidden text, rounding and opacity commands refuse the entire frame before GPU
execution; fallback paints the original complete frame with Canvas. GPU errors
and pixel mismatches remain failures.

The [bridge host evidence](../tools/vulkan-raster-probe/evidence/host-browser-bridge.json)
records **16 exact cases on each of the NVIDIA, AMD and software Vulkan adapters**,
including five actual confined-page captures per adapter. Nine cases use custom
GPU drawing and seven use whole-frame CPU fallback: **151 GPU pixels plus 46
fallback pixels**, or 788 packed bytes per adapter and **2,364 combined bytes**.
That total includes CPU fallback comparisons. Complete captured commands, image
keys, source RGBA and diagnostics are retained. The original 30 standalone cases
also pass on all three adapters with unchanged fixtures, shaders and protocol.

Preparation retains the 320×240 viewport, 256-command, 32-scope, 1 MiB explicit
GPU-buffer and four-million-invocation caps. Borrowed key lookup, source alias
accounting and CPU fallback work have separate conservative bounds. Workers and
snapshots are released before Vulkan initialization; a supervised capture grant
also checks descendants adopted by the outer Linux subreaper. This offscreen
process experiment does not establish production driver isolation or security.

Both Rust 1.88 and 1.98 pass 50 feature-enabled tests, formatting, strict Clippy
and builds. The 25 default tests per toolchain and original 30-case host run are
reused from candidate 1 because candidate 2 changed only a bridge test fixture.
Eleven process-supervisor and fourteen protocol tests pass. The unchanged shader
bytes retain their earlier Naga validation. Full compatibility and Chromium
performance remain unmeasured.

The native window still paints through Canvas and optionally uploads the result.
Its 400×250 minimum exceeds the bridge's 320×240 cap, and browser chrome needs
text rendering. Native-window integration and larger frame limits therefore
require separate implementation and validation.

The separate [bounded glyph slice](../tools/vulkan-raster-probe/GLYPHS.md) now
prepares existing bundled-font masks and composites them with a third integer
shader. Its 26 cases pass on all three adapters, including 14 comparisons against
the preserved CPU painter. These direct lists do not extend the original
worker-capture inventory. Reusing `ab_glyph` masks preserves the custom text-layout
path; it does not delegate HTML/CSS rendering to another engine. Rounded coverage
and alpha groups remain subsequent work. Alpha groups need isolated transparent
targets, nested composition, fixed-descendant clip behavior and deferred
allocation equivalent to the CPU implementation. Ordinary hardware blending
must not silently change the current integer rounding or encoded-color-space
behavior. An intentional color-model correction would be a separate,
independently validated change.

The [worker-text suite](../tools/vulkan-raster-probe/WORKER_TEXT.md) adds seven
actual HTML captures without changing those earlier inventories. Six frames
use the fonts-aware GPU adapter and one uses complete CPU fallback. All match
their same-snapshot Canvas references on NVIDIA, AMD and software Vulkan;
worker cleanup is verified before each capture process initializes Vulkan.

Milestone B needs its own pixel and resource tests before integration into
normal page rendering. Completing A does not complete B or the broader Vulkan
docket.

## Isolation, resource ownership and recovery

The implementation owns device, queue, surface and uploads on a dedicated
presentation thread. Winit window creation/input and the existing
page IPC remain on their current paths. The presenter receives a retained
window owner and bounded completed frames; it receives no page filesystem
authority, network requests, shader source or raw device handles.

A thread keeps ordinary surface waits away from input dispatch, but is not a
security or crash-isolation boundary. Drivers still execute inside the browser
process and kernel. A separately supervised GPU process is needed before
claiming recovery from arbitrary driver hangs or driver-process crashes; window
surface ownership and IPC for that process require a separate design.

Current application-controlled limits are:

| Resource | Initial policy |
| --- | --- |
| Submitted frames | One active upload/submission, retained until completion or release |
| Pending CPU frame | One replaceable latest frame |
| Presenter allocations | 128 MiB ledger: active/pending pixel Vec capacities, conversion storage and its growth transient, padded readback buffer, and a reserved incoming UI framebuffer |
| Upload work | One outstanding upload and one reusable conversion buffer; account row padding and staging copies |
| Vulkan input | At most 4,194,304 pixels and 8,192 per axis; larger frames require clean release before software fallback |
| CPU framebuffer | Existing 16-megapixel/8,192-axis software checks remain unchanged |
| Shader/pipeline sources | No application shader or render pipeline in this upload presenter; the binding retains internal shader machinery |

These are application pixel-buffer limits, not a total browser/process or GPU
memory cap. They do not bound undocumented driver allocations or wgpu's internal
allocator. At the four-megapixel Vulkan limit, active pixels, pending pixels and
converted bytes occupy up to 48 MiB; the UI may simultaneously paint another
16 MiB frame. Count buffer capacities and replacement transients, padded
readback storage, alignment and retirement, not just live logical dimensions.
Pending ownership is swapped under the accounting lock. Driver staging and
swapchain images are excluded from this ledger: approximately another packed
frame for upload staging and multiple configured images are estimates, not
measured or enforceable driver memory limits. Query effective device
texture/buffer limits and request only those needed by the implementation.
Limiting inspection to 16 adapters does not bound the binding's prior full
adapter enumeration or driver allocations.

Completion callbacks release frame slots; they do not run page code or request
another draw merely because a draw completed. Avoid indefinite application
waits and unbounded retry loops. Minimized/occluded windows defer presentation.
Outdated surfaces reconfigure only after old acquired textures are released;
Lost surfaces and repeated failures disable the experimental backend. Missing
adapters, unsupported formats and rejected
allocations fall back to software. Device-loss fallback requires successful
release of the old surface; a stuck driver is not safely canceled by dropping
a Rust future or abandoning a wait.

Vulkan present-wait semaphores have lifetimes tied to swapchain images, not just
submission fences. Let the selected safe layer maintain that synchronization
and retain application resources through its completion notifications. See
[Khronos's semaphore-reuse guidance](https://docs.vulkan.org/guide/latest/swapchain_semaphore_reuse.html).

## Binding evaluation and recommendation

**Binding adopted for the optional upload presenter:** wgpu exactly 30.0.1 with defaults disabled and only `std` and `vulkan` enabled,
behind an optional Eris feature. Select `Backends::VULKAN` explicitly at runtime.
Keep application `unsafe_code = "forbid"` and use the checked public APIs; do
not bypass validation through HAL access, trusted shader entry points or
SPIR-V passthrough. This is a graphics binding, not an
existing web renderer or implementation of the browser's WebGPU API.

| Question | Vulkan-only wgpu 30.0.1 | Vulkano 0.35.2 |
| --- | --- | --- |
| Declared minimum Rust | 1.87; current Eris manifest requires 1.88 | 1.75 |
| Native window handles | Safe surface creation accepts an owned window-handle provider; raw-window-handle 0.6.2 aligns with current winit | Safe `Surface::from_window` and raw-window-handle 0.6 support |
| Application shader path | Checked WGSL module creation plus safe public render/compute commands | Shader macro supplies a safe loader, but public `draw` and `dispatch` still require unsafe calls |
| Relevant dependency structure | wgpu-core, wgpu-hal, Naga validation/SPIR-V generation, ash and GPU allocation support | Vulkan validation/synchronization wrapper, ash, registry-based build generation; shader macro adds shaderc |
| Tradeoff | Larger validation/compiler stack, but a direct path compatible with Eris's unsafe prohibition through later GPU drawing | More Vulkan-specific control and lower declared MSRV, but the intended drawing path conflicts with the current application unsafe policy |

The [wgpu manifest](https://docs.rs/crate/wgpu/30.0.1/source/Cargo.toml) and cached
30.0.1 manifests confirm the Rust requirement and feature selection. Disabling
other backends does not eliminate Naga, core/HAL validation or every support
dependency: the native dependency configuration enables WGSL and RenderDoc
support transitively. Omitting the public `wgsl` feature removes its unused
public shader-source variant for the upload slice; it does not remove Naga,
the native WGSL frontend or internal shader machinery. The recommended feature
set avoids enabling the other
graphics backends and browser-facing web backend. Use the safe
[surface](https://docs.rs/wgpu/30.0.1/wgpu/struct.Instance.html#method.create_surface)
and [shader APIs](https://docs.rs/wgpu/30.0.1/wgpu/struct.Device.html#method.create_shader_module).

Vulkano's [manifest](https://docs.rs/crate/vulkano/0.35.2/source/Cargo.toml)
declares Rust 1.75 and defaults to its macros and X11 features. Its
[shader macro](https://docs.rs/vulkano-shaders/0.35.0/vulkano_shaders/) can hide
shader-module construction behind generated safe functions, but that does not
remove the explicit safety requirements of
[`draw` and `dispatch`](https://docs.rs/vulkano/0.35.2/vulkano/command_buffer/auto/struct.AutoCommandBufferBuilder.html#method.draw).
The [macro manifest](https://docs.rs/crate/vulkano-shaders/0.35.0/source/Cargo.toml)
also includes shaderc. A copy-only presenter would not require that shader
toolchain, but the eventual rasterizer would need a reviewed shader path and an
unsafe-policy decision. This proposal does not make that exception.

There are two specific wgpu lifecycle qualifications. Local 30.0.1 sources show
`wgpu-core/src/present.rs` uses an internal 1,000 ms acquisition timeout, which
the public `get_current_texture` call does not expose as an argument. Vulkan
swapchain cleanup can call `device_wait_idle`. Therefore the design must not
promise a nonblocking UI-thread acquire or bounded destructor time. Keep these
operations on the presenter thread. The UI observes durable state and deadlines
without invoking device polling or joining an unfinished thread. The driver-hang
limitation still applies.
[Published source](https://docs.rs/crate/wgpu-core/30.0.1/source/src/present.rs),
[Vulkan swapchain source](https://docs.rs/crate/wgpu-hal/30.0.1/source/src/vulkan/swapchain/native.rs).

### Footprint evidence, not binary-size estimates

Read-only inspection found cached source/archive copies of the wgpu 30.0.1
family, Naga, ash and allocation support. Vulkano and shaderc were not cached.
No dependency was added or built for that initial package-size comparison. The
subsequent standalone transfer probe compiled its own pinned graph as described
above; it did not change the browser's dependencies.

| Inspected material | Compressed archive bytes | Unpacked/source bytes |
| --- | ---: | ---: |
| Eight cached wgpu/Naga packages at 30.0.1 | 2,134,897 | 11,575,093 |
| Those eight plus cached ash 0.38, gpu-allocator 0.28, spirv 0.4 and renderdoc-sys 1.1 | 2,699,114 | 17,245,989 |
| Vulkano 0.35.2 package alone | Not measured locally | About 8.3 MB reported by docs.rs |

The eight-package subtotal comprises `wgpu`, `wgpu-core`, `wgpu-hal`,
`wgpu-types`, `wgpu-core-deps-windows-linux-android`, `wgpu-naga-bridge`, `naga`
and `naga-types`. These are package inventories, including code for disabled
targets/features, not complete resolved Linux dependency graphs or incremental
download estimates. Several common dependencies already exist in Eris's lock
file. At the time of this initial inventory, none of wgpu, Vulkano, Naga or ash
was in that lock file. The optional presenter now adds the pinned wgpu graph.
The Vulkano figure is a single-package measurement and must not be compared to
the wgpu family subtotal as if both covered equivalent graphs.
[Vulkano package metadata](https://docs.rs/crate/vulkano/0.35.2).

The combined optional graph adds 47 lockfile packages and removes none; existing
versions, sources and checksums remain unchanged. Eight existing dependency
lists change through feature unification or version disambiguation. The Linux
resolve has 191 enabled packages versus 156 by default. Both configurations are
compiled and tested on Rust 1.88, including strict Clippy; declared MSRVs alone
are not the evidence. The new browser evidence records final source/binary
hashes. No runtime-memory or performance comparison is established.

## Acceptance and measurement

- Run the complete current pixel-reference manifest through CPU painting and
  the Vulkan upload/readback path. Test actual GPU output, not only the CPU
  `Canvas` used to feed it. Use an offscreen target for deterministic readback
  when a native surface lacks copy-source support; record that distinction.
- Compare native Wayland and X11 presentation where available, including byte
  order, opaque alpha, color ramps, viewport cropping, fractional zoom, scroll,
  fixed descendants, text, images and nested opacity. GPU presentation must not
  be considered tested merely because device initialization succeeds.
- Exercise rapid resize/minimize/restore, pending-frame replacement, generation
  changes, dropped windows, adapter absence, unsupported formats, allocation
  rejection, surface loss and device loss. Check retired allocations and frame
  queues stay bounded. Keep driver-hang recovery claims separate from simulated
  API-error tests.
- Run GPU rasterization fixtures independently for milestone B, including
  overlapping rectangles, fractional clip edges, fixed-coordinate restoration
  and image sampling. Unsupported frames must demonstrably take the complete
  CPU fallback. Extend tests before expanding the supported command set.
- Check both observed hardware devices and llvmpipe. CI must distinguish an
  unavailable Vulkan environment from a tested GPU pass; CPU regression gates
  continue to run everywhere.

Benchmark identical documents, scripts, viewport, zoom and scroll states. Record
CPU paint time, upload preparation, GPU transfer/raster time when measurable,
presentation wait, input-to-frame latency and peak retained memory separately.
Separate initialization/pipeline compilation from warm frames; record adapter,
driver, present mode and whether CPU fallback occurred. Do not combine an
unrelated CPU optimization with the initial backend comparison. Upload-only
presentation still pays the full software-paint cost and may be slower.

Milestone A is complete when its upload/presentation output and bounded
lifecycle pass those gates. Milestone B is complete only for its explicitly
tested GPU-painted subset. Neither milestone establishes complete browser
compatibility, a complete GPU compositor, a new sandbox guarantee or the
requested Chromium performance threshold.

The opt-in [native frame benchmark](native-render-benchmark.md) now provides
completion-paced host intervals with a bounded exact-scene identity and a
separate correctness mode. The first CPU/native sequence retains 96 measured
frames and two exact acquired-pixel checks; its native median is lower while
first-frame and p95 times are higher. A later native baseline/reuse sequence
completes another 96 measured frames and 9,011,200 acquired correctness bytes.
Subsequent prepare-through-present median/p95 move from 2.086/4.790 ms to
1.309/4.117 ms, retaining every early second-frame submission tail. GPU clocks
were unlocked; environment snapshots changed from P5/360 MHz to P0/2505 MHz.
Those observations cannot establish per-frame clocks or attribute all gains to
reuse. Both sequences remain narrow host measurements, not a Chromium result.
