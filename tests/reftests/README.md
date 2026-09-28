# Focused rendering reference tests

These thirteen self-authored cases compare Eris rendering against independently
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
