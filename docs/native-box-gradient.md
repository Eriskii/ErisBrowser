# Gradient boxes on Vulkan

Native `BoxPaint::LinearGradient` can prepare opaque pixels on the CPU and
submit them through the existing Vulkan image kernel. This extends the
[opaque-gradient route](vulkan-gradients.md) to eligible `background-origin`
and `background-clip` boxes. Interpolation remains on the CPU; no shader,
worker command, CSS grammar, image cache or fetching behavior changes.

The translated outer rectangle must have integer coordinates and dimensions,
each corner must have a zero horizontal or vertical radius, and there must
be no cutout. The existing geometry/clip proof must establish full coverage
for its candidate pixels. Fractional clips may qualify, and the independent
positioning tile may have fractional origins or positive fractional dimensions.
Rounded or fractional translated outer shapes and cutouts retain whole-frame
fallback, even when empty or clipped away.

Every stop must be opaque after `currentColor` substitution. A translucent
current color is harmless when no stop references it. Every retained opacity
scope refuses admission, including values zero and one. Existing directions,
finite length/percentage stops and repeat modes remain available. The Box
stop scale must be finite, positive and at most **16,000,000**, independently
of Native's **1,000,000** coordinate-field bound.

Raw shape, tile, gradient and scale validation precedes the complete metadata
eligibility pass and source/mask preparation. Invalid input retains geometry
refusal; valid unsupported shape, alpha or opacity cases refuse the whole
scene. Finite stop coefficients can still overflow while being resolved in
materialization, before allocating that generated source. Previously prepared
unrelated payloads may already exist. Primitive and text-aware Probe planners
continue to refuse box decorations.

Generated windows become absolute `ScreenImage` commands. Document/fixed
translation is applied once, while tile phase, crop and paint order remain
unchanged. Full opaque coverage and the absence of opacity groups let the
existing evaluator round channels once to RGBA8 without losing intermediate
coverage or group precision.

## Resource accounting

The reservation is one atomic checked debit of **`B + G + M`**:

- `B` is zero for an empty outer candidate window; otherwise `8 + 2P`, where
  `P` counts its original pixels before no-repeat narrowing.
- `G` is the existing opaque-gradient evaluation allowance. For `N` stops,
  let `s = ceil(log2(N)) + 2`. Both repeated axes use `4N + sQ`; with `d`
  nonrepeated axes it is `16 + 4N + (s + d)Q`. An empty gradient window costs
  zero. `Q` is the gradient candidate count.
- `M` is the stored generated-window area, charged for pixel packing; its
  RGBA payload occupies `4M` bytes.

These are bounded-work policy units, not instruction measurements. A disjoint
no-repeat tile can remove `G` and `M` while retaining `B`. `box_gradients`
counts admitted commands, including empty ones. `box_loop_work` includes the
whole reservation; existing gradient evaluation/preparation counters also
include `G`/`M`. Those counters overlap and are not additive CPU totals.

All windows reserve their shared source storage before materialization. Pixels
are transient per command/frame, with no persistent cache or deduplication by
gradient Arc. Existing source-entry, RGBA, lookup, command, CPU and GPU caps
remain. A 512×512 window occupies the entire **1 MiB** RGBA allowance by itself;
loaded sources compete for that allowance. A 1280×1024 gradient needs 5 MiB
and still falls back. No shape-mask bitmap is introduced.

## Validation

The final source passed 2,900 tests with Rust 1.88 Native features and 2,598
with Rust 1.98 defaults, each across 35 result groups with no failures or
ignored tests. Four strict Clippy lanes, root/probe formatting and nine probe
CPU tests passed. All 90 browser reference pairs passed with zero differing
pixels; their 180 PNGs and decoded frames match the published predecessor.

The six new cases cover ramps, currentColor and stop scale, four repeat
modes, fractional clips, fixed/document restoration, empty windows and
subnormal tiles, and actual CSS origin/clip geometry. Their 268 independent
literal pixels matched Canvas, packed Native and both offscreen formats on
NVIDIA RTX 4070 SUPER, AMD RADV and llvmpipe: 36 frames and 6,432 compared
bytes. Including the three unchanged checkers, the total is 162 frames and
31,537,392 bytes. The browser and all four checker binaries bind the same
670-input final source snapshot.

All retained validation attempts exited successfully. These comparisons
establish neither acquired-window behavior nor performance. The
[summary](evidence/native-box-gradient/summary.json) and
[archive](evidence/native-box-gradient/validation.tar.gz) bind the final source,
literals, reviews and actual receipts.
