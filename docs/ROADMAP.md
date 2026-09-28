# Development docket

The goal remains an independent, fully web-compatible Rust browser with a
defensible security boundary and measured performance within the requested
Chromium threshold. The current implementation does not satisfy that goal;
[compatibility](COMPATIBILITY.md), [security](SECURITY.md), and
[validation](VALIDATION.md) track concrete coverage and gaps.

## Vulkan rendering backend

Requested for future implementation on September 28, 2026. Add a custom Vulkan
backend for rasterization and compositing.

The [Vulkan milestone design](vulkan-rendering.md) records the host device
probe, separates upload/presentation from custom GPU rasterization, and compares
bindings. The standalone [Vulkan transfer probe](../tools/vulkan-probe/README.md)
pins wgpu 30.0.1 in its own crate and lockfile. Its bounded offscreen uploads and
readbacks preserve the tested bytes on the NVIDIA, AMD and software Vulkan
adapters. No graphics dependency or Vulkan backend has been added to the browser;
native presentation and custom GPU rasterization remain separate milestones.

Acceptance work:

- Define a backend interface for the existing validated display list and image
  resources, preserving clipping, fixed coordinates, text coverage, alpha and
  nested opacity behavior.
- Bound GPU allocations, command work, uploads and retained resources. Decide
  the GPU process/driver boundary explicitly; GPU access must not broaden the
  renderer's filesystem or network authority.
- Handle synchronization, resizing, surface loss, device loss and resource
  cleanup. Keep the software renderer available for comparison and fallback.
- Compare supported output against independent pixel references and the
  software renderer. Record justified rasterization tolerances explicitly.
- Measure equivalent workloads, including upload/synchronization/presentation
  costs, memory and frame latency. Vulkan alone does not establish a Chromium
  performance result.

The Vulkan browser backend remains planned. The isolated probe evaluates one
binding and transfer path; production driver requirements, platform coverage
and binding adoption remain undecided. Ongoing standards and security work
continues alongside preparation for this backend.
