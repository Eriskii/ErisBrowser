# Performance measurement

No evidence currently establishes that Eris is within 30% of Chromium. Supporting fewer features can make a small fixture appear fast while doing much less work. A benchmark against a different renderer must check output/behavior equivalence before its timing is meaningful.

Build once with `cargo build --locked --release`. Then:

```sh
target/release/eris-browser eris:home --benchmark 100 --output artifacts/benchmark.png
```

The reported median and p95 cover CSS parsing/computation, layout and software painting, using the already loaded DOM and a warmed glyph cache. Network requests, HTML parsing, JS execution, image decoding, PNG encoding, native process startup, IPC encoding/validation/copying, and native presentation are excluded. Source-level units are milliseconds. One cold render's total includes more work and is reported separately.

The recorded development baseline used an AMD Ryzen 7 7800X3D, Linux x86-64, Rust 1.95.0, the release profile, a 1180×880 viewport, and 100 iterations on September 28, 2026 UTC:

| Local fixture | Median | p95 |
|---|---:|---:|
| Home | 4.456 ms | 4.911 ms |
| Gallery | 6.998 ms | 7.298 ms |
| Forms | 5.193 ms | 5.276 ms |

These are machine-specific development observations in an uncontrolled desktop environment, not portable scores or a comparison with Chromium. [The recorded JSON](benchmark-baseline.json) includes build/input hashes and exclusions. Re-run after changes and record hardware, operating system, Rust version, profile, viewport, fixture revision, number of iterations, and precisely included phases.

After the parser, JSON and flex compatibility increment, the same fixture/viewport/iteration configuration recorded medians of 4.475 ms (home), 7.047 ms (gallery), and 5.263 ms (forms); p95 values were 4.929, 7.402 and 5.615 ms. [The later record](benchmark-compatibility.json) retains its own hashes. These two uncontrolled observations are insufficient to establish a speed regression or improvement, and neither measures Chromium.

`python3 tools/benchmark.py` records timings for three local fixtures in `artifacts/benchmark.json`, including source/assets/dependency input hashes, binary hash, CPU, OS, compiler and excluded phases.

## Required comparison design

- Pin both engine builds and a representative local/offline corpus. Include text-heavy pages, flex/grid, large tables, scrolling, complex SVG, images and realistic scripts.
- Reject or separately classify pages that do not render or behave equivalently. Report unsupported cases, not just successful ones.
- Use the same machine, viewport, fonts, device scale, cache state, power policy and workload. Record cold/warm runs, uncertainty, memory, CPU time, frame latency, and interaction latency.
- Define whether “not 30% slower” means time ≤ 1.30× the baseline or throughput ≥ 0.70× baseline; these are different thresholds. Report raw measurements either way.
- Add full-platform benchmarks only when the APIs and semantics needed by those benchmarks exist. Timing partial execution of Speedometer/JetStream would be misleading.

## Current implementation choices

Arena DOM storage, indexed style-rule candidates, bounded selector matching, retained display lists during scrolling, cached glyph masks, clipped painting and a separate page worker avoid some repeated work. The painter remains a CPU rasterizer. It does not implement a GPU compositor or incremental style/layout invalidation. Changes currently recalculate layout, and significant performance work remains.
