# Bounded tiled raster painting

This checkpoint adds `DrawCommand::BackgroundImage` for a raster that is already
present in ImageStore. The command stores a paint rectangle, positioned tile,
independent horizontal/vertical repeat flags, corner radius and image key. One
command describes all repeats. It does not resolve a URL, fetch or decode an
image. CSS URL background images remain unsupported; connecting CSS and retained
Page resources is a later change.

## Sampling and composition

The software painter visits each admitted destination pixel once. It never
enumerates repeated tiles or allocates a resized/tiled bitmap. Each destination
pixel center is transformed into the tile's local coordinates in f64. A repeated
axis uses Euclidean remainder to keep the phase for negative offsets. A
nonrepeated axis paints only centers inside its half-open tile interval.
Nearest sampling floors the normalized phase times the source dimension and
clamps the resulting index to the final texel. If floating-point remainder rounds
up to the tile extent, that clamp retains the last texel rather than wrapping to
zero. Positive tiny tiles therefore need no epsilon or tile-count loop.

The paint outline and caller clip retain the gradient painter's existing
geometry rules. Caller/nested clipping uses the destination pixel's integer
origin; tile sampling and nonrepeat inclusion use its center. Radius belongs to
the paint outline, not to each repeated tile. The existing source-alpha blending,
rounded coverage and isolated opacity groups remain responsible for composition.
Document/fixed coordinate changes translate both the paint rectangle and tile.
A missing key leaves the destination unchanged and causes no resource request.

The existing `Image` command keeps its edge-coordinate sampling, including its
identity-extent correction. The new primitive does not change loaded `<img>`
pixels or reinterpret the old image command's wire payload.

## Work and transport bounds

For a nonempty candidate window, the sampler prepays:

`8 + P * (3 + repeated_axes)`

`P` is the number of destination candidates after clipping and one-time
nonrepeat-axis restriction. `repeated_axes` is 0, 1 or 2. Transparent source
pixels and candidates discarded by rounded coverage still consume this
allowance. This is a source-derived work tariff, not a measured instruction
count. Refusal to pay the scan cost occurs before any destination pixel changes;
opacity-layer allocation/composition retains its own existing admission checks.

No cap increases: paint still allows at most 200,000 commands and 128 nested
scopes, with a viewport-derived work allowance capped at 32 million. Opacity
surfaces retain their 8,388,608 live-pixel and 16,777,216 cumulative allocation
limits. The source dimensions and available RGBA bytes are checked before
indexing. Missing, empty or invalid geometry produces no image paint.

Worker tag 12 carries repeat flags, the paint/tile rectangles, radius and key.
Unknown flag bits, nonfinite/out-of-range geometry, negative extents/radius,
malformed strings and truncated payloads are rejected. The wire rejects nonpositive tile extents; direct software painting treats
a zero tile as inert. Image aliases, old `Image` command keys and new background keys share the
existing 16 MiB decoded key allowance. The sender now enforces the same combined
allowance, so it rejects an oversized frame instead of emitting one that the
receiver would reject. Valid old command bytes, including image tag 4 and
gradient tags 10/11, remain exact. Existing frame, raster and typed-scope limits
continue to apply; a key never grants fetch or file authority.

## Whole-frame software fallback

Primitive, font-aware and Native bridges report
`UnsupportedBackgroundImage` / `unsupported-background-image` from complete
command preflight. This applies even to a missing raster, empty clip or hidden
command. Native refusal retains the original phase and command index, including
inside an opacity-zero scope. The existing primitive/font bridge refusal for an
earlier opacity command still takes precedence, as do existing early malformed
frame, command-count and image-store count/capacity guards.

No source table, mask or generated pixel payload is prepared for a refused
background command, and no GPU prefix is submitted. The caller can repaint the
complete original list in software, retaining commands before and after the
background. Legacy GPU checkers explicitly reject this new primitive without
changing their old fixture rosters. GPU lowering is deferred.

## Validation status

Independent literal pixels cover repeat axes, fractional/negative phases,
nearest scaling, rounded edges, alpha/opacity, fixed coordinates, tiny tiles and
the separate old Image contract. Bridge tests cover missing/hidden refusal,
original phase/index, source/font preparation ordering and complete-list software
repaint. Wire tests cover literal tag 12 bytes, all invalid repeat masks, shared
key admission and malformed payloads.

The first focused compile failed with E0689 in a new test's ambiguous float
literal. An explicit `f32` suffix fixed the annotation without changing production
code or expected values. That failed attempt and its before/after source are
retained. The corrected focused run passed 30 tests. Final full runs passed 2,645
native tests on Rust 1.88 and 2,436 default tests on Rust 1.98, with no failed or
ignored tests and 30 result rows each. All 39 new groups passed: 12 raster,
11 codec, seven bridge and nine transform groups.

Strict Clippy passed for native Rust 1.88/1.98, default Rust 1.98 and the probe
on Rust 1.98. Probe unit suites each passed 15 tests on Rust 1.88 and 1.98;
root and probe formatting checks also passed. The final browser passed all 71
unchanged self-authored render comparisons with zero differing pixels and zero
channel error. These are unit/reference checks, not a new GPU adapter run.

The final 555-input snapshot also updates the workflow's pinned bridge inventory
from 107 to 114 for the seven new refusal tests. Both feature-list checks and the
extracted CI discovery script passed. This source revision changes only that
workflow; runtime bytes remain identical to the previous validated source, and
the final rebuilt browser is byte-identical. The published parent `dd6ac11d` CI
run completed with eight successful jobs and one cancelled `vulkan-raster-probe`
job. The initial annotation failure was the only unsuccessful local check.

The [evidence summary](evidence/tiled-image-raster.json) and
[retained sources and observations](evidence/tiled-image-raster.tar.gz) bind the
source revisions, failed attempt, fixtures, final browser and raw checks. This
foundation makes no new GPU, performance, full compatibility or
production-security claim.
