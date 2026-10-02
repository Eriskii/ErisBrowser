# Offscreen worker-snapshot bridge oracle contract

Preparation only. No browser, planner, Canvas, compiler or GPU has been run. The
16 cases contain 11 direct `graphics::DrawCommand` lists and five local HTML
documents, with nine expected GPU admissions and seven whole-frame CPU fallbacks.
Their 197 output pixels are independent literal grids: 591 RGB bytes / 788 packed
little-endian RGB bytes. Existing 30 probe fixtures and every existing limit stay
unchanged. This is not native browser rendering or completion of the browser goal.

The future checker receives a framebuffer, clear color, caller clip, document and
fixed offsets, an ordered display list and an image store. Zoom is exactly one.
GPU acceptance covers only zero-radius rectangles, images, Canvas-style Line
rectangles and typed clip/fixed scopes. Valid missing image keys are no-ops; no
bridge lookup performs IO. All text, nonzero-radius rectangles and opacity scopes
(including opacity one) refuse the entire frame before any GPU submission, even
when hidden, transparent or zero-sized. Budget rejection also refuses the entire
frame. Rejection is neither a GPU error nor an observed GPU pass.

For direct inputs, the first unsupported command's zero-based index is specified
where useful. Reason codes are the proposed stable categories in `fixtures.json`;
diagnostic wording is not the oracle. Command-count rejection precedes traversal;
balanced depth-33 fixed scopes reach the separate scope rejection. These cases
are valid for the CPU painter's larger unchanged command/scope limits. The
fallback paints the original complete list; partial GPU work is forbidden. All
cases expect `Canvas::exhausted()` false. Genuine driver, mapping or comparison
failures must remain failures rather than being reclassified as fallback.

Image entries with the same `backing_id` use the same Arc-backed RasterImage in
direct cases. Key spelling and alias identity are distinct from RGBA content.
Assign deterministic first-reference IDs without depending on HashMap iteration.
Keep alpha-zero source words and all source validation; do not suppress image
dispatch by scanning alpha. Reject malformed source storage before GPU work.
The bridge must bound key/entry inspection and allocation before growth; an
additional conservative admission bound is allowed only after root reviews its
explicit contract. No broader key/resource policy is established by these small
semantic fixtures. The existing 320×240, 256-command, 32-scope, 1 MiB explicit GPU
buffer and four-million-invocation caps remain unchanged. Accepted inputs must
also fit the CPU paint-work budget, conservatively if necessary. Separate future
boundary tests must cover bridge key/metadata allocations and CPU-budget refusal;
this inventory tests the existing command and scope caps only.

For each accepted input, the future CPU reference and actual GPU readback must
independently match the frozen literal target; agreement with each other alone
is insufficient. Packed output has high byte zero. Clear is unconditional over
the complete framebuffer, including outside the caller clip. For a fallback,
compare the complete CPU output to the same literal target, assert its reason,
and demonstrate no GPU submission for that case. The visible prefix/suffix and
path assertion together guard whole-frame behavior. The future protocol must
require all 16 unique case IDs, expected nine GPU / seven fallback dispositions,
exact comparison sizes and a completion footer; missing/repeated cases fail.

HTML execution is explicitly unperformed. A later authorized checker must use
the exact local files and PNG bytes, scripts disabled and no network documents,
an explicitly hash-bound browser executable, `WorkerClient::spawn_at`, Load and
Render at the listed dimensions. Worker results must pass the existing IPC
decoder. Capture generation, diagnostics, image identities and the complete
display-command stream, then verify each listed `worker_command_assumptions`.
Check all geometry/color/source values and typed scope order relevant to the
paint; extra unsupported commands are a fixture-input mismatch, not permission
to alter expected admission. The full command stream may include balanced
equivalent scope wrappers and the white propagated canvas background. Do not
discard those wrappers before executing the actual adapter. Dynamic node IDs or
load timings are not pixel-oracle values.

Missing image loading is expected to leave no image-store entry while retaining
its placeholder and Image command. Its worker diagnostic remains visible. The
image loader must decode `two-pixels.png` to the explicit eight RGBA bytes;
different decoding, a placeholder on a loaded image, unexpected text or altered
layout must stop acceptance and be reported. Current source inspection supports
these assumptions; only the future worker run can verify them. No worker output
may redefine the frozen target silently. If a source assumption proves wrong,
retain the original fixture/report and require an explicit reviewed correction.

Complete/drop worker, broker and image-decoder clients before creating Vulkan in
this first bounded checker. No worker restart or navigation after GPU startup is
part of this contract. GPU ownership stays outside the sandboxed page process.
Native windows, chrome, zoom integration, text, rounded GPU coverage, opacity
compositing, new caps and performance claims remain separate work.
