# Development docket

The [Window global-binding checkpoint](../tests/conformance/window-global-bindings.md)
adds general data/accessor definitions, exact UTF-16 property keys, private
execution receivers and global declaration validation. The
[String concat checkpoint](../tests/conformance/string-concat.md) adds bounded,
ordered conversion and exact UTF-16 assembly. The
[constructor checkpoint](../tests/conformance/construction.md) adds Reflect calls,
private `new.target` bindings and alternate ECMAScript allocation targets.
Next work includes alternate Web IDL targets, remaining native constructor
semantics, Function source retention/toString and lossless UTF-16 source parsing,
Window extensibility and interface coverage, complete DOMString storage and Web
IDL interfaces, remaining Symbol consumers, iterator infrastructure and broader
ECMAScript dependencies. The
[constructor-policy review](../tests/conformance/constructor-policy.md) now enables
88 older modes and corrects Symbol constructor classification. These sit alongside
the Vulkan milestones below. [Dynamic Function construction](../tests/conformance/function-constructor.md)
now adds 157 passes in a complete new 200-source selection and 35 older passes.
The [String conversion follow-up](../tests/conformance/string-conversion.md)
fixes its two exposed slice failures and adds 18 search/split passes.
[RegExp split/species](../tests/conformance/regexp-split.md) adds 86 passes with
observable construction, execution and captures.
[RegExp constructor conversion](../tests/conformance/regexp-constructor.md)
adds classification and ordered conversion with 16 more passes.
[Flat bounded regexp group parsing](../tests/conformance/regexp-deep-groups.md)
now adds four deep-pattern passes. [Required-literal rejection](../tests/conformance/regexp-required-literals.md)
adds two XML-pattern passes without raising budgets.
[Match/search symbol protocols](../tests/conformance/regexp-match-search.md) now add
138 passes in a complete new inventory.
[String lastIndexOf](../tests/conformance/string-last-index-of.md) adds 46 passes
in its complete directory and 12 older passes.
[Array lastIndexOf](../tests/conformance/array-last-index-of.md) adds 328 passes
and closes the two remaining String.lastIndexOf failures.
[Array descriptors and sparse storage](../tests/conformance/array-descriptors.md)
now support indexed accessors, writable length, partial shrink failure and
nonextensibility. Live array methods consume these properties.
[Array every and some](../tests/conformance/array-predicates.md) now add generic
short-circuiting predicates, with all 164 frozen local variants passing. Their
[complete paired inventory](../tests/conformance/test262-array-predicates.md)
gains 801 passes and four older descriptor metadata modes also pass. Twelve
Date failures, four long-sparse-scan work stops and 16 resizable-buffer metadata
exclusions remain visible; no healthy predicate gate is recorded.
Date and BigInt dependencies,
replacement/matchAll protocols, general matching performance, the remaining
full-UTF-16 loop, Unicode regexp syntax and Function source retention remain
concrete next dependencies.

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

The optional [Linux Vulkan presenter](vulkan-rendering.md) now uploads completed
CPU-painted frames through a Vulkan-only wgpu binding. It uses one owner thread,
one active upload and one replaceable pending frame, fixed operation deadlines,
and confirmed resource release before software fallback. Verification reads
back actual acquired surface textures; it does not certify compositor output.
Software remains the default/headless path. Custom GPU rasterization remains
unimplemented, and driver isolation/performance work remains open.

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

The custom Vulkan rasterizer remains planned. The optional upload presenter
adopts one transfer/presentation path; production driver requirements and
broader platform coverage remain undecided. Ongoing standards and
security work continues alongside preparation for this backend.

## JavaScript execution depth

The original Test262 source with 32 nested immediately invoked functions now
passes in both modes. Its source and the complete function inventory remain
unchanged. The [parser checkpoint](../tests/conformance/parser-continuations.md)
recorded 511 passes; the later [constructor policy](../tests/conformance/constructor-policy.md)
brought coverage to 515 passes; dynamic Function construction now brings it to
546 passes and 585 unsupported variants. A healthy baseline
runs in CI.

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
