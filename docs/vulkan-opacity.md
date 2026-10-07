# Bounded native group opacity

This document records the initial opaque-backed milestone. The current route
also admits [transparent groups](vulkan-transparent-opacity.md) at `k/256`;
the original measurements and refusal records below are historical.

The optional Vulkan raster route can composite nested opacity groups with an
opaque rectangular backing. It uses custom integer shaders and cropped RGBA16
scratch storage. The software renderer remains the default and handles complete
scenes that the native planner cannot admit.

```sh
cargo run --locked --release --features vulkan-raster -- \
  ./examples/vulkan-opacity.html --presenter=vulkan --raster=gpu
```

Click the red panel to change its opacity from 0.5 to 0.75. The example includes
a nested opacity panel, text, a translucent image and rounded decoration. A
large window can exceed the existing native limits and use CPU rendering.

## Admission and arithmetic

A materialized group must have opacity `k/256`, with integer `1 <= k <= 255`.
Its first effective direct draw must be an opaque, unrounded rectangle covering
the conservative pixel union of all descendants. Each nested group must prove
its own backing; a child composite does not prove its parent's backing. This
allows translucent images, glyph coverage and rounded decoration over that
rectangle. A union of several rectangles, an image-only group or a rounded-only
backing does not satisfy the certificate.

Opacity zero suppresses drawing after input validation; opacity one preserves
the current destination without an intermediate. Nonfinite, out-of-range and
non-grid values are refused even in hidden scopes. Clip, fixed-position and
opacity scopes share one typed stack. Fixed descendants that escape an inner
clip still contribute to the group's conservative bounds.

The backing makes every materialized source pixel opaque. For a premultiplied
16-bit source channel `S`, destination `D` and opacity numerator `k`, the pop
numerator is `N = S*k + D*(256-k)`. A parent group receives `(N+128)/256` with
integer division. For the root RGB8 target, `D` is first multiplied by 257 and
the result is `(N+32896)/65792`. Products and sums stay within exactly
representable binary32 integers, and these divisions reproduce the software
painter's rounding for the admitted domain. The shader does not rely on GPU
floating-point contraction or tie rounding. For example, red 65 at opacity 0.5
over black produces red 33.

Every group retains a disjoint cropped region at eight bytes per pixel. All
clears and composites count toward the frame's existing four-million padded
invocation limit. Scratch joins the existing 16 MiB explicit GPU-buffer ledger
and exact-size retired-buffer reuse key. Every frame clears its regions and
rewrites all parameters and inputs. There is no reuse between groups within a
frame, no per-phase budget reset and no increase to command, source or scope
limits. The original Probe entry points still refuse opacity; their four
shaders and encoded no-group drawing bytes remain unchanged.

## Checks

The offscreen checker uses fixed literal expectations recorded before GPU
execution. On NVIDIA RTX 4070 SUPER, AMD RADV and llvmpipe, each adapter passes
18 literal frames across BGRA8/RGBA8 plus 14 changed-content reuse frames:
824 compared bytes per adapter, 2,472 in total. Each also checks eight
cancellation boundaries and seven allocations, seven reuse hits and seven
evictions for the successful reuse sequence. Three unproved/non-grid scenes
are rejected before GPU execution. These are small arithmetic and lifecycle
checks, not representative performance workloads.

The actual browser window passes four acquired-texture comparisons at 1280×880
on NVIDIA/BGRA8, totaling **18,022,400 bytes**. One targeted mouse click follows
the initial verified frame. Two compared frames retain outer opacity 0.5, then
two show 0.75; the inner group stays at 0.5. One Escape redraw after the first
changed comparison fills the four-frame quota. This proves two content states,
not four distinct DOM changes. Both groups use 262,144 scratch bytes in total.
A separate normal window presents the same initial scene without a CPU reference
or readback. Both processes exit normally with no surviving owned descendants.
The comparisons cover the acquired texture before the desktop compositor.

The Page and confined-worker tests load the public example and a richer button
scene, dispatch real hit targets, retain DOM identities and compare the complete
native-plan interpretation with Canvas and selected fixed literal pixels.
Rust 1.88 passes **2,139 native-feature tests**, **128 GPU-feature core tests**,
**97 default core tests**, and strict Clippy. The unit tests initialize no GPU;
the separate host checks above supply the GPU evidence.

```sh
cargo build --locked --release \
  --manifest-path tools/vulkan-raster-probe/Cargo.toml \
  --bin eris-vulkan-opacity-check
python3 tools/vulkan-raster-probe/run_opacity_host.py \
  --binary tools/vulkan-raster-probe/target/release/eris-vulkan-opacity-check \
  --output /tmp/eris-opacity-check --allow-experimental-gpu
```

If the installed Vulkan loader is outside the library search path, pass its
directory with `--loader-directory`. The runner enumerates the available
adapters, bounds each child, preserves raw failures and requires a fresh output
directory. It does not install drivers.

The window runner `tools/native_opacity_host.py` requires a Hyprland session,
an explicit release binary and SHA-256, an installed loader directory and a
fresh output directory. It sizes and controls only its own launched window,
serves the fixed example over a bounded loopback origin after sizing completes,
and runs the verified and normal cases separately. See the
[evidence summary](evidence/vulkan-native-opacity.json) for exact commands,
source bindings and retained attempts.

Transparent-backed groups, arbitrary opacity values and broader scene admission
remain unfinished. These results establish neither full CSS compositing nor
production security, compositor output or performance within 30% of Chromium.
