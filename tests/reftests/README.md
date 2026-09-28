# Focused rendering reference tests

These thirty-two self-authored cases compare Eris rendering against independently
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
