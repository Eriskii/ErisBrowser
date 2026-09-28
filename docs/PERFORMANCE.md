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

After the broker, namespace and Test262 increment, the same warm-render configuration recorded medians of 4.399 ms (home), 7.053 ms (gallery), and 5.203 ms (forms); p95 values were 4.542, 7.519 and 5.273 ms. [This record](benchmark-broker.json) retains the measured build/input hashes. It still excludes native process startup and IPC, so these timings do not measure the new broker's overhead or establish a Chromium comparison.

`python3 tools/benchmark.py` records eight measurements across seven local pages in `artifacts/benchmark.json`, including source/assets/dependency input hashes, binary hash, CPU, OS, compiler and excluded phases. The template/Grid, positioning, event/compositing and responsive fixtures use their initially loaded documents; cloning, import loading and click handlers execute outside the measured warm-render loop. The event page is measured both from the top and scrolled to its panel so the opacity group is visible. Opacity allocation and compositing, when needed, are inside that loop.

## Required comparison design

- Pin both engine builds and a representative local/offline corpus. Include text-heavy pages, flex/grid, large tables, scrolling, complex SVG, images and realistic scripts.
- Reject or separately classify pages that do not render or behave equivalently. Report unsupported cases, not just successful ones.
- Use the same machine, viewport, fonts, device scale, cache state, power policy and workload. Record cold/warm runs, uncertainty, memory, CPU time, frame latency, and interaction latency.
- Define whether “not 30% slower” means time ≤ 1.30× the baseline or throughput ≥ 0.70× baseline; these are different thresholds. Report raw measurements either way.
- Add full-platform benchmarks only when the APIs and semantics needed by those benchmarks exist. Timing partial execution of Speedometer/JetStream would be misleading.

## Current implementation choices

Arena DOM storage, indexed style-rule candidates, bounded selector matching, retained display lists during scrolling, cached glyph masks, clipped painting and a separate page worker avoid some repeated work. The painter remains a CPU rasterizer. It does not implement a GPU compositor or incremental style/layout invalidation. Changes currently recalculate layout, and significant performance work remains.

The fragment/descriptor/float/image-isolation checkpoint recorded warm medians
of 4.457 ms (home), 7.118 ms (gallery), and 5.291 ms (forms), with p95 values of
4.569, 7.419 and 5.470 ms under the same 100-iteration, 1180×880 configuration.
[The measurement record](benchmark-fragments-floats.json) includes build/input
hashes. These uncontrolled local timings exclude image decoding, process startup,
IPC and native presentation; they do not establish the cost of isolated image
loading or performance relative to Chromium.

The template/strict-mode/Grid/encoding checkpoint recorded medians of 4.459 ms
(home), 7.086 ms (gallery), 5.340 ms (forms), and 1.085 ms (the new template/Grid
fixture). The corresponding p95 values were 4.540, 7.299, 5.769 and 1.119 ms.
These use the same 100-iteration, 1180×880 configuration. The final measurement
ran after the native smoke-test window closed, and its source and binary hashes
were checked against the frozen build. [The recorded JSON](benchmark-templates-grid.json)
contains the complete environment and phase definitions. It is an uncontrolled
local observation; the new fixture has no earlier recorded comparison, and none
of these measurements establish Chromium-relative performance.

The HTML/base-URL/RegExp/positioning/import checkpoint recorded warm medians of
4.470 ms (home), 7.106 ms (gallery), 5.250 ms (forms), 1.119 ms (templates), and
1.117 ms (the new positioning fixture). Corresponding p95 values were 4.497,
7.256, 5.292, 1.187 and 1.204 ms. This used the same 100-iteration, 1180×880
configuration after the native window and other heavy checks finished.
[The recorded JSON](benchmark-positioning-regexp.json) retains verified source
and binary hashes. The positioning fixture has no earlier recorded baseline;
its CSS imports and script setup finish before measurement. These local
observations do not establish a speed change, import/RegExp execution cost or
performance relative to Chromium.

The cascade-layer/event/group-opacity checkpoint recorded warm medians of
4.938 ms (home), 7.982 ms (gallery), 5.797 ms (forms), 1.141 ms (templates),
1.128 ms (positioning), and 11.047 ms (the new events fixture). Corresponding
p95 values were 5.033, 8.673, 5.979, 1.183, 1.390 and 11.986 ms. This used
100 iterations at 1180×880 after the native window and heavy checks finished.
[The recorded JSON](benchmark-layers-events.json) retains verified source and
release-binary hashes. The event fixture includes opacity allocation and
compositing, even when the group's content is outside the viewport: intermediate
surfaces currently use the full caller viewport. Group bounds or tiling remain
performance work. Loading and event dispatch remain outside these measurements.
These local observations do not establish Chromium-relative performance.

After deferred opacity allocation and AbortSignal support, the seven recorded
medians were 5.002 ms (home), 8.216 ms (gallery), 6.010 ms (forms), 1.115 ms
(templates), 1.121 ms (positioning), 1.155 ms (events from the top), and
12.385 ms (events scrolled to `#panel`, with its opacity group visible).
Corresponding p95 values were 5.405, 8.445, 6.118, 1.281, 1.153, 1.391 and
13.712 ms. [The final record](benchmark-abort-opacity.json) retains verified
build/input hashes and the same 100-iteration, 1180×880 configuration.

The offscreen event case previously measured 11.047 ms. It now avoids allocating
8,307,200 bytes and charging 2,076,800 initialization/composite pixels for a group
that contributes no visible pixels. A separately recorded
[visible-group baseline](benchmark-opacity-visible-before.json) on the previous
frozen build measured 11.763 ms median and 11.945 ms p95; the later visible-group
measurement was higher. Visible groups still allocate the full caller viewport.
Nine decoded-pixel comparisons, covering both event views, desktop/narrow clicked
pages and the other benchmark pages, match the previous build exactly.
These are uncontrolled local observations, not evidence of a general speedup or
performance relative to Chromium. Abort dispatch is outside the measured loop.


The template-literal/media-condition/flex-wrapping checkpoint records eight warm
measurements at 1180×880 over 100 iterations, after heavy checks and the native
window finished. [The measurement record](benchmark-responsive.json) retains
verified source and release-binary hashes:

| Local fixture | Median | p95 |
|---|---:|---:|
| home | 5.029 ms | 5.687 ms |
| gallery | 8.204 ms | 9.163 ms |
| forms | 6.002 ms | 6.490 ms |
| templates | 1.126 ms | 1.154 ms |
| positioning | 1.110 ms | 1.380 ms |
| events | 1.169 ms | 1.467 ms |
| events-visible | 12.296 ms | 12.982 ms |
| responsive | 1.949 ms | 2.004 ms |

The responsive page is a new fixture without an earlier equivalent baseline.
Its click handlers and template interpolation execute outside the timed loop.
These uncontrolled local observations cover implemented layout/paint behavior;
they do not establish a general speed change, script throughput, GPU performance
or the requested Chromium threshold. Vulkan remains planned on the
[development docket](ROADMAP.md), without a backend or performance result yet.
