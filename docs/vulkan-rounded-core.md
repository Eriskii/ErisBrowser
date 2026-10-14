# Large rounded Color backgrounds in the Vulkan backend

Large rounded Color backgrounds can exceed the native mask allowance even when
most of their pixels are fully covered. The new route represents that interior
as one rectangle and stores coverage only for the remaining edges. It uses the
existing rectangle and glyph shaders, retaining their ordered alpha and group
opacity behavior.

Selection is deliberately narrow: an individually oversized stored mask, no
cutout, and integer geometry and scope offsets with an exact raw-to-screen
round trip. The existing pure rectangle and ordinary mask routes run first.
A collection of small masks cannot use this route to evade the shared limit.
Images and gradients keep their existing eligibility rules.

The helper retains the original candidate window and absolute sample positions.
It computes an inward integer interior, verifies all four sample-hull corners
inside the rounded shape, then partitions the remaining storage into disjoint
top, bottom and side bands. Width splitting produces at most seven edge tiles.
The core and edges never paint the same pixel. Used elliptical radii and
fractional clip membership retain the software painter's behavior.

All queries and scans are prepaid. For original candidate area P, ambiguous
edge candidates A, stored edge area E and total tile rows R, the two work stages
cost `12 + P` and `2P + 16A + E + R`. A failed second debit retains the first
charge. Transparent colors and zero-opacity groups retain this conservative
admission cost. These are logical resource fees, not measured CPU instructions.

The complete batch must fit the shared command, source, row, coverage and GPU
budgets before payload allocation. Every edge buffer is reserved before the
first coverage sample is generated. A failed reserve cannot publish a partially
filled batch. The existing limits remain: 256 commands/sources, 262,144 coverage
bytes, 65,536 rows, 16 MiB of planned GPU storage and four million padded shader
invocations. Structural preparation accounting includes the actual enlarged
reservation type and bounded temporary owners; it is not an RSS or stack-frame
measurement.

A 512 by 513 rounded box with radius one uses a 510 by 511 rectangle and four
edge masks: 2,046 coverage bytes and 1,024 row entries. A direct single rounded
box can fit the 1280 by 1024 native viewport, but a real Page's additional
full-viewport canvas paint still exceeds the unchanged invocation allowance at
that size. The selected Page fixture is 512 by 513.

The held tests cover opaque and translucent colors, opacity groups, suppressed
painting, fractional clips, seven edge tiles, exact resource boundaries and
allocation failures. The separate offscreen checker compares two output
formats against independently held pixels using the custom Vulkan shaders.
It does not establish acquired-window rendering, complete CSS coverage or
performance relative to Chromium.

Validation passed 3,213 tests with `vulkan-raster` and 2,886 default-feature
tests, including all 89 confined-worker tests in each run. The probe crate's
20 tests and four Python protocol test groups passed. Rust 1.88.0 formatting
and strict Clippy checks on Rust 1.88.0 and 1.99.0 passed; an earlier 1.99 lint failure
and the two test-only iterator corrections are retained in the evidence.
All 102 reference pairs passed, with all 204 PNGs unchanged from the preceding
Promise-finally checkpoint.

The frozen offscreen checker passed on NVIDIA GeForce RTX 4070 SUPER, AMD
RADV RAPHAEL_MENDOCINO and llvmpipe Vulkan adapters. Each adapter checked ten
fixtures in two formats: 20 frames and 29,405,216 active RGBA bytes, for
88,215,648 compared bytes across all three. One fixture comes through the
actual HTML/CSS/Page pipeline. GPU readback was compared in the checker;
the archive retains process output and receipts, not full raw GPU frames.

The [evidence summary](evidence/vulkan-rounded-core/summary.json) binds the
source inventory, frozen executables, literal oracles, retained failures,
reviews and checks. The accompanying `sources-and-checks.tar.gz` contains
the selected sources and validation artifacts; compiled executables are
identified by hash rather than included.
