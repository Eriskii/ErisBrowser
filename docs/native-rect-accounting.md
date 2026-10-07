# Native rectangle work accounting

The browser’s optional Vulkan raster route now reserves a rectangle’s clipped
CPU paint-loop area instead of a full framebuffer. A page containing forty
20×12 boxes at 512×512 previously reserved 10,485,760 pixels and exceeded the
4,194,304 allowance, despite its GPU plan fitting. Its rectangle reserve is now
9,600 pixels. No resource limit is raised.

```sh
cargo run --locked --release --features vulkan-raster -- \
  ./examples/vulkan-small-boxes.html --presenter=vulkan --raster=gpu
```

Click the first red box to turn it blue and change the title. The example uses
forty fixed-size absolute boxes with stable node identities; it creates no new
nodes on click. Other scene limits still apply when browser chrome, fonts,
viewport size and device scale are included.

## The bound matches the paint loop

The reservation follows the software painter’s floating-point translation and
intersection order. Its lower bounds use the visible rectangle’s floor, the
active clip’s ceiling and zero. Its upper bounds use the reconstructed visible
endpoint’s ceiling and framebuffer extent. The resulting integer area is
charged before rounded coverage or blend’s upper-clip rejection. Counting only
stored mask coverage can miss work at fractional edges.

A separate traversal uses the core’s shared typed coordinate state, with a fresh
state for each phase. Fixed scopes reset to the caller clip and viewport offset;
pops restore prior coordinates. Rectangles within transparent colors or zero
opacity still pay their geometric loop bound. Empty or completely clipped
rectangles pay zero, after full input validation. Their original command and
scope charges remain.

The existing rounded helper obtains the bound with a literal zero radius and
no mask allocation. The real radius remains validated and controls later
lowering. Mask dimensions, coverage, rows, tiling and resource checks are
unchanged. Image, Line and both opacity markers retain their full-frame
reservations. Geometry charges across all phases are summed with checked
arithmetic before the font session receives its remaining pixel allowance.
Rounded materialization does not debit the rectangle again.

The extra pass is bounded by the existing 256 original commands and four phases.
One coordinate-state scope vector is live at a time and drops before later
preparation. The existing metadata envelope covers that vector; no per-command
cache or new preparation allowance is introduced. Shader code, core planning,
GPU allocation formats and invocation counts remain unchanged. This change
improves admission precision and does not measure rendering speed.

## Evidence

The [evidence package](evidence/native-rect-accounting.json) retains the published
predecessor’s actual refusal for both a synthetic forty-box scene and the public
HTML example. The synthetic core plan already admits 41 draws, 539,648 padded
invocations and 2,107,664 planned GPU bytes. Those bounds are independent of the
browser’s CPU preparation reserve.

Pixel, scope, malformed-input, glyph-sharing and exact/one-over budget checks
cover the new reservation. Existing pixel literals and all Image/Line/Opacity
charges are retained. Older tests that used zero-size rectangles to consume
whole frames now use full-frame transparent rectangles for the same quota
boundary. Historical test sources remain in the evidence.

Rust 1.88 passes all 82 bridge groups and 2,161 native-feature browser tests,
including the ignored confinement checks. The direct Page and confined-worker
witnesses each compare the complete 512×512 scene before and after a real click,
retain 44 observed node identities and the complete parent/child graph, and
change exactly 240 pixels. The resulting plan retains 41 draws, 539,648 invocations
and 2,107,664 GPU bytes. Exact pixel-budget admission and a one-pixel excess are
both checked. Strict Clippy, formatting and the native release build pass.

This increment makes no new GPU execution or acquired-window measurement.
Full web compatibility, production security and Chromium-relative performance
remain unfinished.
