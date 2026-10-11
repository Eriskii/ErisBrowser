# Image boxes on Vulkan

Native `BoxPaint::Image` can use the existing raster-background lookup tables
and Vulkan image kernel. This admits selected padding/content clips from
`background-origin` and `background-clip` when their painted shape meets the
same geometry proof as [rectangular color fills](native-box-fastpath.md).

Each corner must have a zero horizontal or vertical radius, there must be no
cutout, and the outer rectangle must have integer coordinates and dimensions
after document or fixed-coordinate translation. A bounded check proves full
coverage for its candidate pixels and compatibility with the active clip.
Fractional clips may pass this proof. The positioning tile may still have
fractional origins or positive fractional dimensions; it is independent of the
selected painting box.

The structural pass validates raw shapes, tiles and image keys before a
separate metadata eligibility pass examines every image box. Unsupported
shapes refuse the whole scene even when empty, hidden, missing their source,
or enclosed by zero opacity. Rounded shapes, fractional translated outer shapes, cutouts and
`BoxPaint::LinearGradient` retain fallback. Primitive and text-aware Probe
planners still refuse box decorations.

Admitted commands retain their original rectangle and tile. Existing scope
handling applies document/fixed offsets once; center sampling, repeat and
no-repeat phase, source alpha, paint order and isolated opacity groups stay on
the existing image path. The implementation adds no shader, worker protocol,
image decoding or fetching authority.

## Resource accounting

The reservation is one checked, atomic debit of `B + T + L`:

- `B` is zero for an empty outer candidate window; otherwise `8 + 2P`, where
  `P` counts the original box candidates before no-repeat restriction.
- `T` is the existing background-image loop allowance: zero for an empty image
  window, otherwise `8 + Q * (3 + repeated_axes)`.
- `L` is the existing per-axis lookup allowance: zero for an empty final
  window, otherwise `8 + (3 + repeat_x) * W + (3 + repeat_y) * H`.

Repeat flags are zero or one. These are conservative policy units, not CPU
instruction measurements. Missing, transparent and suppressed sources retain
the reservation. A disjoint no-repeat tile can eliminate `T` and `L` while
leaving `B` intact. No mask coverage, mask rows or destination bitmap is allocated.

Box images borrow keys and share loaded sources by allocation identity with
ordinary images and raster backgrounds. Existing key, source-entry, RGBA,
lookup, command, CPU and GPU limits remain unchanged; all retained rasters
still receive source validation. Missing keys emit no image draw and keep the
existing missing-image count. `box_background_images` counts admitted routes,
including missing or empty ones; `box_loop_work` includes their complete
`B + T + L` reservation alongside existing color-box work.

## Validation

The final source passed 2,893 tests with Rust 1.88 Native features and 2,598
with Rust 1.98 defaults, each across 35 result groups. Four strict Clippy
lanes, root/probe formatting and eight probe CPU tests passed. All 90 browser
reference pairs passed exactly; their 180 PNGs and decoded frames match the
published predecessor.

The six new image cases passed both formats on NVIDIA RTX 4070 SUPER, AMD RADV
and llvmpipe: 36 offscreen frames and 6,432 compared bytes. Including both
retained color checkers, the total is 126 frames and 31,530,960 bytes. The
browser and all three immutable checker binaries bind the same final source.
These comparisons establish neither acquired-window behavior nor performance.

The evidence retains an initial missing-`--` test invocation, a focused run
with 175 passes and one legacy key-precedence assertion failure, and a
`collapsible_if` lint failure. The command delimiter and key expectation were
corrected; equivalent conditions were joined for Clippy. The final checks
above passed without changing the frozen pixel expectations.

The [summary](evidence/native-box-image/summary.json) and
[archive](evidence/native-box-image/validation.tar.gz) bind the selected source,
literals, reviews and actual receipts.
