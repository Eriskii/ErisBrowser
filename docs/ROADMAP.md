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
runtime stack limit independently. Ordinary activation continuations have since
removed that earlier combined-stack barrier for shallow calls, up to the existing
32-call ceiling. Native helpers remain separately guarded.

The planned progression is:

1. Syntax ownership is now flat from parsing through execution, with direct
   construction into charged record pages and explicit unit handles for closures
   and callbacks. Parser lists, records and publication share one compile ledger;
   the old owning tree/lowering path is test-only. Declaration analysis and
   hoisting use bounded cursors; declaration-name validation uses charged buffers
   and comparisons. Preserve grammar guards during the remaining parser work.
2. Ordinary execution now uses [shared continuations](../tests/conformance/activation-frames.md),
   including expressions, statements, activation, defaults, bound forwarding and
   try/finally. Actual values and code units survive reentry; cleanup retains the
   correct frame boundary and logical-call ownership. Fully iterative work uses
   charged frame storage instead of native-depth charges. Retained shallow call
   and default-initializer probes now complete 32 calls; call 33 reaches the
   existing logical ceiling. Native callbacks and constructors retain weighted
   guards. Continue validating those bridges as execution features expand.
3. Convert recursive grammar calls to bounded continuations over the now-flat
   syntax records. Preserve strict-context checks, RegExp/template lexical goals,
   early errors and safe destruction of partially parsed input. The ownership
   prerequisite is implemented; the continuation conversion remains pending.

Each stage needs independent allocation, callback, cleanup and small-native-stack
checks. Keep the original 32-function test and all previous passing cases in the
inventory throughout; only the combined parser and runtime work can establish
that test's improved result. Flat executable ownership, bounded declaration
traversal and ordinary execution continuations are implemented. The iterative
parser remains unimplemented. The recorded [480 depth observations](../tests/conformance/activation-frames-depth.json)
include 72 newly completed ordinary/default runs and 32 changed resource messages;
all parsing and nested-IIFE outcomes remain unchanged. [Direct flat parsing](../tests/conformance/flat-parser.md)
also preserves all 480 observations. Grammar continuations, remaining allocation
accounting and the independent declaration-traversal depth limit need further work.
