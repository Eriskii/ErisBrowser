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
adapters. A separate temporary native prototype presented changed/resized
frames on the NVIDIA adapter; five acquired-surface readbacks matched all
3,316,800 tested bytes. Full compositor captures remained non-exact, with
counts and caveats preserved in the
[native evidence](evidence/vulkan-native-surface.json).

The browser now passes a checked, borrowed completed CPU frame to a focused
[software presenter](../src/presenter.rs), which exclusively owns its native
surface. Software remains the only browser path; no graphics dependency,
Vulkan backend, presenter thread or frame queue was added. Native Vulkan
integration and custom GPU rasterization remain separate milestones.

The [first integration contract](vulkan-rendering.md#first-integration-contract)
now specifies one active upload and one pending frame, viewport/visibility
invalidation, fixed operation deadlines and confirmed release before fallback.
The review also accounts for staging allocated before submission and distinguishes
texture readback from final compositor output. These remain implementation
requirements, with no browser Vulkan path enabled yet.

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

The Vulkan browser backend remains planned. The isolated experiments evaluate
one binding and transfer/presentation path; production driver requirements,
platform coverage and binding adoption remain undecided. Ongoing standards and
security work continues alongside preparation for this backend.

## JavaScript execution depth

The retained Test262 test with 32 nested immediately invoked functions currently
stops at the parser budget in both modes. A separate investigation of checkpoint
`9d6aa9e` found that a controlled chain of ten IIFEs parses and executes, while
eleven stop in parsing. A shallow recursive function reaches the combined
runtime stack limit independently. Parser changes alone will therefore leave
an execution barrier.

The planned progression is:

1. Introduce flat executable-code ownership. Declaration-name collection and
   hoisting now use a bounded borrowed traversal; switch scope checking no longer
   clones syntax trees. Finish auditing name-set work/storage and account for
   retained code and temporary lowering storage. Preserve
   existing parser limits while recursive syntax ownership remains.
2. Move ordinary expression, statement and function execution to an explicit
   continuation stack. Preserve evaluation order, lexical environments,
   return/throw/finally behavior and quota cleanup. Keep logical call limits and
   guards on native helpers that can call author code.
3. Convert the recursive parser paths to bounded continuations with flat syntax
   ownership. Preserve strict-context checks, RegExp/template lexical goals,
   early errors and safe destruction of partially parsed input.

Each stage needs independent allocation, callback, cleanup and small-native-stack
checks. Keep the original 32-function test and all previous passing cases in the
inventory throughout; only the combined parser and runtime work can establish
that test's improved result. The borrowed declaration traversal is an initial
step; flat code ownership, explicit execution continuations and an iterative
parser remain unimplemented.
