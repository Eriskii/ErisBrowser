# Root notes: next glyph-mask checkpoint (proposal only)

Parent 5ad2cfd3582f92f8368a0a0fa2fd3bb7dd7af2f0 is pushed; its CI is running.
No new production source, fixture oracle, font rasterization or GPU execution is
assigned to this proposal yet. Existing pixel references remain immutable.

The browser bridge has removed the snapshot/driver boundary gap. Text is the next
coverage gap. Root inspected Fonts::bitmap, Canvas::text/blend, and the probe
scope/plan logic. Existing text combines unquantized-size kerning/advance with
1/8-pixel-quantized masks; italic changes each row origin before f32 round; y is
rounded once. Effective alpha truncates color.alpha * coverage /255 before the
existing integer source-over rounding. These are current-Eris behavior, not a
claim of complete shaping or CSS typography.

A plain translated Image primitive risks changing pen/offset/rounding order and
cannot directly encode italic row origins. Prefer reviewing a bounded glyph-mask
primitive with integer absolute row origins and y, borrowed coverage bytes,
color, and the existing clip/fixed state. Each glyph is one ordered dispatch;
each mask coordinate maps to at most one output pixel, and separate glyph passes
preserve overlap order. A dedicated integer shader could reuse the existing
output/64-byte uniform/input-buffer binding layout without changing either
existing rectangle/image WGSL file. No CPU-painted final framebuffer upload.

The adapter must not round then subtract/re-add offsets. Text preparation needs
the current normalized caller/nested clip and f32 document/fixed offsets. Shared
scope logic is preferable to a second subtly different implementation. Existing
plan_snapshot can preserve its documented rectangle/image subset while a clearly
separate fonts-aware API admits text; old16case frozen routes must not silently
change. Any broadened route requires an explicitly separate new expectation.

Fonts currently rasterizes before Canvas charges loop pixels. New bounded mask
preparation must preflight outline dimensions, allocation and callback work before
rasterization, including cache misses; current cache counts are not a cumulative
work guarantee. Existing original_commands*frame_area bound must be extended with
full charged glyph bitmap work, including off-clip pixels and empty-mask max(1).
Bound original UTF-8 bytes/visited scalars, expanded draws, unique masks, source
bytes, atlas/row tables and temporary CPU storage. Reusing a cached bitmap must
not bypass input/work admission. No existing frame/scope/GPU quota increase is
part of this step; native-window minimum size remains a separate milestone.

Oracle strategy must separate independently authored coverage-mask/pixel cases
from actual-font CPU/GPU differential cases. A matching shared font helper is not
independent proof of font rasterizer correctness. Font bytes/version bindings,
literal integer blend oracles, row-origin placement and parser metadata give
separate attributable evidence. No golden may be derived from the candidate's
final-frame output and relabelled independent.

Await independent API, bounds and oracle proposals before selecting exact caps
or authorizing implementation. Full web compatibility/security/Chromium target
remain unfinished.
