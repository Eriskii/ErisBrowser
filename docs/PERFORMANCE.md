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

`python3 tools/benchmark.py` records nine measurements across eight local pages in `artifacts/benchmark.json`, including source/assets/dependency input hashes, binary hash, CPU, OS, compiler and excluded phases. The template/Grid, positioning, event/compositing responsive and disclosure fixtures use their initially loaded documents; cloning, import loading and click handlers execute outside the measured warm-render loop. The event page is measured both from the top and scrolled to its panel so the opacity group is visible. Opacity allocation and compositing, when needed, are inside that loop.

## Confined worker measurements

```sh
target/release/eris-browser examples/responsive.html --benchmark-worker 100 --output artifacts/worker.png
python3 tools/benchmark_worker.py --fresh-runs 5 --iterations 100
```

The CLI prints a JSON record with raw warm samples and writes the final frame.
The Python runner measures the same nine fixture views as `benchmark.py`,
retains all samples, records source/binary/environment hashes, and rejects
changed inputs or page diagnostics. Linux confinement is required; this mode
does not fall back to execution in the calling process. `--no-scripts` is
available for individual CLI measurements. Clicks, DOM dumping and desktop
options are incompatible with the worker benchmark.

| Phase | Included work |
|---|---|
| Startup | Address/file authorization, renderer spawn, sandbox handshake |
| Load | Load round trip, renderer font setup, resource broker startup and reads, parsing, script setup and any requested image decoding |
| Cold render exchange | First style/layout computation, snapshot generation, IPC copying, decoding and validation |
| Cold clear/paint | Parent canvas clear and software paint with a fresh glyph cache |
| Warm render exchange | Repeated style/layout and validated snapshot round trips |
| Warm clear/paint | Parent canvas clear and software paint with a warmed glyph cache |
| Warm total | Render exchange, fragment lookup, clear/paint and snapshot destruction |
| Teardown | Worker/broker closure, termination and wait |

Parent font/canvas initialization, PNG encoding, the desktop event loop and
native presentation are excluded. “Cold” means fresh processes and parent glyph
cache; operating-system and filesystem caches are not flushed. Cold values have
one sample per fresh CLI invocation. Warm samples are pooled across invocations,
with an upper-middle median and nearest-rank p95. Five cold samples cannot
establish a reliable tail latency. Frame totals are measured directly, so their
quantiles need not equal sums of the component quantiles. These are local
implemented-workload measurements, not a Chromium comparison.

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


## Disclosures, feature queries and confined-worker baseline

Recorded September 28, 2026 UTC on the same machine, after heavy checks and
native window inspection finished. Both reports have verified source and release
binary hashes. The [warm-render record](benchmark-disclosures.json) uses 100
iterations per view. The new [confined-worker record](benchmark-worker-disclosures.json)
uses five fresh processes per view, each followed by 100 warm frames. It retains
all raw samples, phase definitions and runner hashes.

| Local fixture | In-process warm median / p95 | Confined warm frame median / p95 |
|---|---:|---:|
| home | 5.032 / 5.412 ms | 5.284 / 5.658 ms |
| gallery | 8.194 / 8.372 ms | 9.143 / 9.902 ms |
| forms | 6.090 / 6.510 ms | 6.162 / 7.091 ms |
| templates | 1.143 / 1.197 ms | 1.269 / 1.448 ms |
| positioning | 1.130 / 1.225 ms | 1.281 / 1.506 ms |
| events | 1.140 / 1.192 ms | 1.296 / 1.513 ms |
| events-visible | 12.296 / 12.943 ms | 12.682 / 13.696 ms |
| responsive | 1.974 / 2.024 ms | 2.097 / 2.429 ms |
| disclosures | 1.222 / 1.297 ms | 1.371 / 1.565 ms |

The columns measure different paths: the second includes repeated validated
snapshot round trips and destruction; its software painter runs in the parent.
They are uncontrolled local observations and cannot by themselves establish an
IPC regression or a cross-engine speed result. Loading, script execution and
process creation are outside both warm columns; their separately measured
confined phases are retained in the JSON. PNG encoding and native presentation
remain excluded. The disclosure fixture and confined path have no earlier
recorded equivalent baseline. Vulkan and the requested Chromium threshold remain
unverified.


## Token-aware selectors and native task continuation

Recorded September 28, 2026 UTC with the same 1180×880, 100-iteration
warm style/layout/software-paint configuration. The [measurement record](benchmark-tasks-selectors.json)
retains verified source and release-binary hashes.

| Local fixture | Median | p95 |
|---|---:|---:|
| home | 5.065 ms | 5.265 ms |
| gallery | 8.205 ms | 8.601 ms |
| forms | 6.072 ms | 6.195 ms |
| templates | 1.151 ms | 1.405 ms |
| positioning | 1.127 ms | 1.252 ms |
| events | 1.147 ms | 1.378 ms |
| events-visible | 12.359 ms | 13.427 ms |
| responsive | 2.001 ms | 2.427 ms |
| disclosures | 1.216 ms | 1.293 ms |

The warm loop includes the new shared selector compilation and matching path.
It excludes script callbacks and native idle scheduling, along with the
previously listed loading, IPC and presentation phases. No confined-worker
benchmark was repeated for this checkpoint. These uncontrolled desktop
measurements do not establish a causal speed change or the requested Chromium
threshold. The [Vulkan design](vulkan-rendering.md) is published, but no GPU
backend or GPU performance result exists yet.

## Computed custom-property checkpoint

Recorded September 28, 2026 UTC at 1180×880 over 100 warm iterations per view.
The [before record](benchmark-custom-properties-before.json) uses the verified
`dde3ff1` browser binary and a separate frozen source/assets checkout. The
[after record](benchmark-custom-properties.json) uses the completed variable
and descriptor corrections. Both input/binary identities are verified; all
nine decoded final frames match exactly between those builds.

| Local fixture | Before median / p95 | After median / p95 |
|---|---:|---:|
| home | 5.038 / 5.225 ms | 5.218 / 9.199 ms |
| gallery | 8.118 / 8.856 ms | 8.685 / 10.419 ms |
| forms | 5.972 / 6.485 ms | 6.251 / 7.860 ms |
| templates | 1.161 / 1.219 ms | 1.195 / 1.570 ms |
| positioning | 1.142 / 1.171 ms | 1.221 / 1.514 ms |
| events | 1.160 / 1.300 ms | 1.236 / 1.674 ms |
| events-visible | 12.241 / 12.582 ms | 12.756 / 15.354 ms |
| responsive | 1.952 / 2.034 ms | 2.004 / 2.130 ms |
| disclosures | 1.235 / 1.283 ms | 1.308 / 1.721 ms |

Observed medians and tail values are higher in the later run. These are
uncontrolled desktop measurements; separate native Vulkan prototype work was
also active during development, so the samples do not isolate a causal cost.
Token-aware declaration parsing and computed-variable resolution are inside the
warm loop; descriptor calls and other script execution are outside it. There
was no repeated confined-worker measurement or Chromium comparison. The
browser still paints and presents through its software path.
