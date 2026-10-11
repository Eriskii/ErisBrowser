# Rectangular color fills on Vulkan

Native `BoxDecoration` color fills can now use the existing rectangle shader without preparing a coverage mask. A full 1,280 × 1,024 solid background fits this route; the preceding mask-only path exceeded its 262,144-byte coverage allowance. Rounded fills and border cutouts retain the [coverage-mask path](native-box-color.md).

The shortcut requires no cutout, a zero axis at every corner, and integer geometry after the existing document or fixed-coordinate translation. A constant-size check proves that every candidate pixel lies inside the shape and active clip. Fractional clips remain supported when this proof succeeds. Other valid shapes retain masks; invalid geometry and unsupported image/gradient paints retain their existing refusals.

Lowering passes the original rectangle through the existing scope machinery, which applies the offset once. Full coverage preserves source alpha, paint order and opacity-group behavior. No shader, worker protocol or resource-loading authority changes.

The route reserves the conservative software paint cost `8 + 3P` for a nonempty candidate window of `P` pixels, including transparent or suppressed commands. Empty windows reserve zero. This proof performs no per-pixel classification, and the route allocates no mask coverage or row payload. Existing command, CPU, GPU-buffer and invocation limits still apply. Large rounded boxes and rings can still select whole-scene software fallback.

`box_rectangles` counts commands admitted to this route, including commands that later draw nothing. `box_loop_work` includes both rectangle and mask reservations; it describes policy work, not measured CPU instructions.

## Validation

The final source passed 2,880 tests with Rust 1.88 Native features and 2,598 with Rust 1.98 defaults, each across 35 result groups. All four strict Clippy lanes, root/probe formatting, and seven probe CPU tests passed. All 90 reference pairs passed exactly; their 180 PNGs and decoded frames match the published predecessor.

Five new rectangle cases and ten retained mask cases passed both output formats on NVIDIA RTX 4070 SUPER, AMD RADV and llvmpipe: 90 offscreen frames and 31,524,528 compared bytes. The final browser and both immutable checker binaries bind the same source snapshot. One initial adapter invocation failed because its loader path was mistyped; the retained corrected retry passed without source or oracle changes. These comparisons establish neither acquired-window behavior nor performance.

The [summary](evidence/native-box-fastpath/summary.json) and [bounded archive](evidence/native-box-fastpath/validation.tar.gz) retain the source, literal tests, reviews, actual receipts and failed invocation.
