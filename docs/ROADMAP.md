# Development docket

The [DOM binding checkpoint](../tests/conformance/dom-string-conversion.md) adds
checked string conversion and receiver handling to the supported DOM operations,
building on [Symbol identities and hooks](../tests/conformance/symbols.md).
Next work includes complete DOMString storage and Web IDL interfaces, global
reflection, the remaining Symbol consumers, iterator infrastructure and broader
ECMAScript dependencies.
These sit alongside the Vulkan milestones below.

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

The original Test262 source with 32 nested immediately invoked functions now
passes in both modes. Its source and the complete function inventory remain
unchanged; [the latest comparison](../tests/conformance/parser-continuations.md)
records 511 passes and 620 unsupported variants, with a new healthy CI baseline.

The implemented progression is:

1. Syntax ownership is flat from direct parsing through execution. Charged record
   pages and typed IDs cover partial syntax, defaults, methods and function bodies.
   Closures and callbacks retain their unit. The old owning tree/lowering path is
   test-only. Declaration walks and name validation use bounded charged storage.
2. Expressions, statements, ordinary activation, defaults and bound forwarding
   share an execution driver. Reentry and cleanup preserve frame boundaries and
   logical-call ownership. Native callbacks and constructors retain weighted guards.
3. Supported JavaScript grammar uses a separate bounded continuation stack over
   flat records. Function context, strict checks, cover grammar and RegExp/template
   lexical goals are preserved. The former grammar-depth and member/constructor
   chain guards are removed; source/token/work/storage quotas remain unchanged.

The [480 depth observations](../tests/conformance/parser-continuations-depth.json)
now show all three shapes parsing through the tested depth of 40 and executing
through 32 calls; call 33 reaches the existing logical ceiling. Relative to direct
flat parsing, nested IIFEs gain 104 completions and change 16 resource observations
from parsing to the logical runtime limit. No completion is lost. Separate source
and malformed-cleanup tests request a 128 KiB native stack.

Remaining depth work includes the independent 96-ancestor declaration traversal
limit, native callback/constructor bridges and wider allocation accounting.
Keep source inventories, strict/early-error checks, work/heap refusal tests and
small-stack cleanup checks as these boundaries evolve. No stage establishes full
language coverage, production security or the requested Chromium performance.
