# Vulkan rendering milestones

Status: design, host inventory and an isolated transfer probe, recorded
September 28, 2026. No Vulkan backend or graphics dependency has been added to
the browser. The standalone [`tools/vulkan-probe`](../tools/vulkan-probe/README.md)
crate has its own pinned wgpu dependency and lockfile. This document expands the
[Vulkan docket](ROADMAP.md#vulkan-rendering-backend). The requested custom GPU
rasterizer and compositor remain future work; uploading software-rendered
pixels is an intermediate presentation milestone. No GPU speedup or Chromium
performance result is established.

## Existing boundary

The page process produces the custom renderer's `DrawCommand` list, hit regions
and decoded images. The parent validates that snapshot in
[`worker/codec.rs`](../src/worker/codec.rs). Native
[`Browser::draw`](../src/browser.rs) paints the snapshot with
[`Canvas::paint_with_viewport`](../src/graphics.rs), adds browser chrome and
copies the completed `Vec<u32>` into a softbuffer surface. Headless rendering
uses the same CPU painter.

`Canvas` already owns the relevant paint semantics: command order, separate
document and fixed offsets, typed clip/fixed/opacity scopes, glyph masks,
nearest-neighbor images, rounded coverage, integer alpha blending and RGBA16
group-opacity intermediates. Its limits include 200,000 commands, 128 combined
scopes, a 16-megapixel framebuffer with an 8,192-pixel axis limit, bounded glyph
and pixel work, and deferred, bounded opacity allocations. A Vulkan path must
preserve those semantics and the explicit exhaustion indication.

The page sandbox denies GPU device access, almost all ioctls and new threads.
GPU initialization belongs outside that process. This proposal does not weaken
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
1.88, but ten packages omit that field; the recorded build used the installed
Rust 1.95 toolchain, not 1.88. This is an isolated dependency evaluation, not a
change to the browser's dependency graph or completion of milestone A.

## Milestone A: Vulkan presentation

Introduce a native presenter boundary after the completed CPU frame. Proposed
interfaces are illustrative and are not currently implemented:

```text
Presenter = Software | Vulkan
Frame = { serial, generation, width, height, opaque_pixels }
present(Frame) -> Presented | Deferred | Unsupported | Failed
```

The initial Vulkan option should be explicit and experimental; software stays
the default. A possible CLI spelling is `--presenter=software|vulkan`, which
describes this slice accurately. The headless CPU path remains the reference.

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

The GPU work in this milestone is transfer and presentation. The page is still
rasterized and composited by `Canvas`. Existing CPU screenshot output by itself
cannot verify that the Vulkan copy or presentation was correct.

## Milestone B: custom GPU rasterization

Add a separate renderer entry that accepts the existing validated display list,
images, viewport clip and document/fixed offsets. Initially accept a coherent
subset: opaque, unrounded rectangles; opaque nearest-neighbor images; lines
with the existing rectangle interpretation; and balanced clip/fixed scopes.
Resolve scopes and clipped bounds with bounded CPU work, then execute custom
GPU kernels or draws in display-list order. Overlapping writes must be ordered;
a single unordered dispatch over primitives would be incorrect.

Reject unsupported frames before GPU execution and paint the complete frame
with the CPU backend. Do not drop unsupported commands or mix partial frames
without a defined compositing model. This first subset mostly exercises simple
rectangle/image pages; text-heavy pages still use software. Report which path
actually rendered each measured frame.

Subsequent bounded slices add existing CPU-generated glyph masks as a bounded
atlas, rounded coverage, translucent primitives and group opacity. Reusing
`ab_glyph` masks preserves the custom text-layout path; it does not delegate
HTML/CSS rendering to another engine. Alpha groups need isolated transparent
targets, nested composition, fixed-descendant clip behavior and deferred
allocation equivalent to the CPU implementation. Ordinary hardware blending
must not silently change the current integer rounding or encoded-color-space
behavior. An intentional color-model correction would be a separate,
independently validated change.

Milestone B needs its own pixel and resource tests before integration into
normal page rendering. Completing A does not complete B or the broader Vulkan
docket.

## Isolation, resource ownership and recovery

The first implementation should own device, queue, surface and uploads on a
dedicated presentation thread. Winit window creation/input and the existing
page IPC remain on their current paths. The presenter receives a retained
window owner and bounded completed frames; it receives no page filesystem
authority, network requests, shader source or raw device handles.

A thread keeps ordinary surface waits away from input dispatch, but is not a
security or crash-isolation boundary. Drivers still execute inside the browser
process and kernel. A separately supervised GPU process is needed before
claiming recovery from arbitrary driver hangs or driver-process crashes; window
surface ownership and IPC for that process require a separate design.

Proposed initial application-controlled limits are:

| Resource | Initial policy |
| --- | --- |
| Submitted frames | At most two in flight |
| Pending CPU frame | One replaceable latest frame |
| Presenter allocations | 128 MiB ledger for retained upload buffers, intermediate textures and surface-image estimates; include retired generations until completion |
| Upload work | At most two bounded framebuffer uploads outstanding; account row padding and staging copies |
| CPU framebuffer | Existing 16-megapixel/8,192-axis checks still apply; use software when the presenter budget cannot admit the window |
| Shader/pipeline sources | Fixed application-owned code, compiled at initialization; no page-provided code or unbounded per-style variants |

These numbers are design choices awaiting implementation measurements. They are
not hard bounds on undocumented driver allocations or wgpu's internal allocator.
For example, a 16-megapixel RGBA frame is 64 MiB; multiple uploads and surface
images can exceed the proposed presenter ledger before any opacity targets.
Pool retention, alignment and resize retirement must be charged, not just live
logical texture dimensions. Query supported device limits and request only
those needed by the bounded implementation.

Completion callbacks release frame slots and request redraw; they do not run
page code. Avoid indefinite application waits and unbounded retry loops.
Minimized/occluded windows defer presentation. Lost/outdated surfaces recreate
only after old acquired textures are released; repeated failures disable the
experimental backend. Missing adapters, unsupported formats and rejected
allocations fall back to software. Device-loss fallback requires successful
release of the old surface; a stuck driver is not safely canceled by dropping
a Rust future or abandoning a wait.

Vulkan present-wait semaphores have lifetimes tied to swapchain images, not just
submission fences. Let the selected safe layer maintain that synchronization
and retain application resources through its completion notifications. See
[Khronos's semaphore-reuse guidance](https://docs.vulkan.org/guide/latest/swapchain_semaphore_reuse.html).

## Binding evaluation and recommendation

**Recommendation for browser integration, not yet adopted by the browser:**
implement the first slice using
wgpu 30.0.1 with defaults disabled and only `std`, `vulkan` and `wgsl` enabled,
behind an optional Eris feature. Select `Backends::VULKAN` explicitly at runtime.
Keep application `unsafe_code = "forbid"` and use the checked public APIs; do
not bypass validation through HAL access, trusted shader entry points or
SPIR-V passthrough. This recommendation concerns a graphics binding, not an
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
support transitively. The recommended feature set avoids enabling the other
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
operations on the presenter thread, use nonblocking polling for application
completion handling, and retain the driver-hang limitation stated above.
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
file; none of wgpu, Vulkano, Naga or ash is currently in that lock file.
The Vulkano figure is a single-package measurement and must not be compared to
the wgpu family subtotal as if both covered equivalent graphs.
[Vulkano package metadata](https://docs.rs/crate/vulkano/0.35.2).

No release binary size, compilation time or runtime memory comparison has been
performed. Before adopting the dependency, resolve a pinned graph and record
its added packages, licenses, build time and stripped release size against the
same CPU-only build. Check the resolved graph with Rust 1.88; top-level package
MSRVs alone do not prove all newly resolved transitive versions are compatible.

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
