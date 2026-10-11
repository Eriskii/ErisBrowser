# Focused rendering reference tests

These sixty-eight self-authored cases compare Eris rendering against independently
expressed reference layouts at 320 × 240 pixels. They exercise specific HTML/CSS
behaviors; they are not the Web Platform Tests, a browser compatibility score, or
a Chromium performance comparison. Passing a pair cannot detect a defect shared
by both expressions or a rasterizer defect shared by all pages.

Run from the repository root:

```sh
cargo build --offline
python3 tools/reftest.py
```

Use `--binary target/release/eris-browser` after a release build to test that
binary. The runner parses PNG pixels using Python's standard library, demands an
exact RGBA match, and saves the two rendered images and a JSON report under
`target/reftests`. Mismatches also produce red/white pixel difference images.
Render failures, timeouts, malformed PNGs, and pixel differences fail the run;
no case is silently skipped. `manifest.json` describes each assertion.

The six float cases and the associated geometry coverage are described in
[`floats.md`](floats.md), including the remaining formatting limitations.

The seven additional grid cases and the associated geometry coverage are
described in [`grid.md`](grid.md), including supported syntax and remaining
sizing and placement limitations.

The six positioned-layout cases and their geometry/stacking coverage are
described in [`positioning.md`](positioning.md), including the remaining
static-position, inline-fragment and compositing limitations.

The six opacity cases use independently specified opaque SVG rectangles to
check grouped backgrounds, borders, images, inline fragments, nested contexts
and fixed descendants. [`opacity.md`](opacity.md) describes their coverage and
the painter's surface/work limits.

Three cascade-layer cases cover normal and important order, inline precedence,
nested and implicit layers, `revert-layer`, variable substitution and anonymous
layer identities across imported source segments. Their references use opaque
SVG rectangles without cascade layers. The supported grammar and rollback
limits are recorded in [`cascade-layers.md`](../conformance/cascade-layers.md).

Five image-fitting cases cover fit modes, physical positions, content-box
padding/borders, nested overflow/fixed/opacity scopes, and the Images 4 combined
scale-down forms. Their references use literal rectangles rather than fitted
images. [Image fitting](../../docs/image-fitting.md) records the syntax, clipping
behavior and remaining scope limits.

Three identity-image cases compare 240-pixel SVG/PNG bands against literal CSS
rectangles, covering one-pixel rows and columns, offset clipping, and each
identity axis beside a scaled axis. The
[identity-sampling follow-up](../../docs/image-fitting.md#identity-sampling-follow-up)
records the distinct sampling fix and retained predecessor failures.

Three linear-gradient cases compare interpolated and hard-edge colors,
premultiplied transparency, default background boxes and canvas propagation
against literal rectangles. The [gradient checkpoint](../../docs/linear-gradients.md)
records the bounded syntax, software evaluator and explicit native raster
fallback. These add no general CSS Images or color-management claim.
