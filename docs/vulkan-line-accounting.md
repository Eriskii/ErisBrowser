# Native Line work accounting

The optional native raster route now reserves each `Line` command's clipped
rectangle loop area instead of one full framebuffer. Forty 24×1 Lines in a
512×512 frame reserve **960 pixel work units**, down from **10,485,760**, and fit
the unchanged **4,194,304** allowance. This changes admission precision while
preserving the existing drawing operation, packed plans, shaders and limits.

## Geometry and admission

The existing [Line conversion](../src/graphics/raster_bridge.rs) forms an
axis-aligned rectangle: its origin is the endpoint minimum and each extent is
the larger of the absolute endpoint difference and width. It retains `f32`
operation order. The active document or fixed offset is added once, then the
existing zero-radius Rect helper intersects the active clip and derives integer
loop bounds. Lower bounds use the visible origin's floor, the clip origin's
ceiling and zero; upper bounds use the reconstructed endpoint's ceiling and
framebuffer extent. The nonnegative span product is the reserved work.

This is the renderer's existing Line behavior, including reversed endpoints,
fractional coordinates and finite nonpositive widths. It adds no new stroke
semantics. Transparent colors and zero-opacity descendants still pay geometry;
empty or fully clipped inner loops cost zero. Complete original geometry and
scope validation runs first, including hidden commands and invalid later input.

The [native geometry pass](../src/graphics/raster_bridge/native.rs) now includes
Line-only phases. It reuses the bounded coordinate state, preserving fixed-scope
escape, restoration and independent phase clips. There is no per-Line allocation
or mask. One existing scope vector is live per visited phase; its fallible
allocation and conservative storage reservation remain bounded as before.

Line work is now debited in original command order alongside other geometry.
For example, at 512×512, sixteen full-frame Lines exhaust the allowance before
a following one-pixel Box and report that Box's indexed refusal. With the Box
first, its existing 11-unit charge precedes the Lines, and the last Line reports
the aggregate unindexed pixel-budget refusal. Both refuse the complete scene.
Opacity markers retain their global full-frame precharge.

The CPU allowance remains `(framebuffer_area * 16).clamp(1_000_000, 32_000_000)`.
The counter covers inner pixel visits and opaque span elements, not every CPU
instruction. A fractional clip can leave no inner x visits but nonempty outer y
iterations; the independent 256-command and 1024-height limits bound those rows
to 262,144 across a scene. The four-phase, scope, source, mask, GPU-buffer and
invocation limits are unchanged. More precise CPU admission does not guarantee
that a scene passes the later GPU checks.

## Literal controls and loaded page

Nine independent unit groups cover small Lines, literal pixels and Rect packed
plans, `f32` cancellation, clips and offsets, phase/fixed restoration, alpha,
exact/one-over pixel admission, structural caps, hidden invalid input and
ordered Box refusals. Their expected work comes from frozen numeric literals,
not the implementation helper or observed candidate output.

The [loaded-page witness](../tests/vulkan_lines.rs) uses the
[decoration example](../examples/vulkan-lines.html) at 512×512 with scripts
disabled. Its nine nonbreaking spaces produce nine empty glyph occurrences and
18 underline/strike-through Lines, preserving all 27 display commands. The
literal pixel oracle has 360 colored pixels. Two passes on the same loaded
layout verify Canvas and decoded packed-plan pixels, exact draw order and a
**369-unit** total: 360 Line visits plus nine existing glyph charges. The chosen
bundled glyph has an empty outline; this does not test font-outline rasterization.

## Validation and retained attempts

The published parent runs all ten new groups: **two pass and eight fail** at
the expected admission or work assertions. Failed groups stop at their first
assertion; later rows are not claimed as parent observations. The first compiled
candidate passes all **ten groups** on Rust 1.88 with `raster-bridge`, preserving
the frozen oracles. Before the parent measurement, a page-helper compile failure
required an explicit `usize` closure argument and removal of an unused `mut`.
Those test-only fixes, the failed compile and the parent failures are retained.
The first full suites also passed before strict Clippy flagged the exact binary32
literals `25.6328125` and `500000.03125`. Narrow function-level lint allowances
retain those values and all assertions; the initial lint failure remains recorded.

Final complete runs pass **3,344 default Rust 1.88 tests** and **3,708 Rust 1.99 tests
with `test262-host,vulkan-raster`**. Both have 54 result rows and no failed,
ignored or filtered tests. The default roster is unchanged; the feature roster
preserves its parent plus ten groups and one selected historical rename.
Strict all-target Clippy passes for Rust 1.88 default and `raster-bridge`, and
Rust 1.99 with `test262-host,vulkan-raster`; formatting also passes.

The Vulkan-only release build passes. Both actual window checks exited before
completing a native check frame: the compositor supplied a 3825×2085 tile, exceeding the
presenter's existing four-megapixel surface limit. This is distinct from the
Native profile's 1280×1024 limit. The retry's owned-process controller could not
resize the window before startup exited and released no HTML response. Both
runs recognized an NVIDIA GeForce RTX 4070 SUPER with `Bgra8Unorm`, but completed
zero native check frames with no acquired-texture comparison. Both cleanups finished with
zero descendants. The attempts are retained; actual GPU correctness for this
checkpoint remains unverified.

The final source inventory equals candidate 2 and binds **1,838 selected
inputs**; it is not a complete build graph. Existing JavaScript conformance
profiles were not rerun for this native-only change.

The CPU packed-plan decoder is not shader execution. Existing rectangle/image
chapters retain their earlier checkpoint observations. This accounting change
establishes no browser compatibility, general rendering-speed or security claim.
The [evidence manifest](evidence/vulkan-line-accounting/manifest.json) and
[source/check archive](evidence/vulkan-line-accounting/sources-and-checks.tar.gz)
retain the contracts, source reviews, corrections and observed runs.
