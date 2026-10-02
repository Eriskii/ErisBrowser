# Native wide rounded coverage: bounded next increment

Proposal only, after publication of the current native checkpoint. No source
changes, builds, rasterization or host execution were performed for this note.
The address-bar refusal follows from the current source; attribution of a
particular host failure still requires its recorded diagnostic.

At a 1280-pixel viewport, native chrome makes the address bar
`Rect(181, 31, 1084, 34)` with radius 7. `RoundedShape::prepare` currently rejects
its 1084-pixel cropped width against `MAX_MASK_AXIS = 1024`. The aggregate area
is only 36,856 cells, so this is an individual-mask width restriction, not the
262,144-byte scene coverage ceiling.

## Smallest useful change

Add an explicit Native-only rounded partition preparation API, used only by
`graphics::raster_bridge::native`. Keep the existing `RoundedShape::prepare`
API, its per-mask refusals, Probe limits and frozen probe routes unchanged.
Do not raise any axis, area, source, operation, row, work or byte limit.

Compute the original validated rounded geometry and visible integer window
once. Partition that window horizontally into at most two half-open strips,
each at most 1024 columns wide. Native width is at most 1280 and height at most
1024, so vertical tiling or a general paging framework is unnecessary. Retain
the aggregate area refusal: splitting a 1280×1024 shape must not admit its
1,310,720 coverage cells through smaller individual masks.

Use fixed-size metadata for the two possible strips. A possible API is
`RoundedTiles::prepare_native(frame, translated_rect, clip, radius)`, with
Empty/ZeroRadius/Tiles dispositions; aggregate information contains the
original loop work, tile count, coverage and row totals. Tile information
contains only absolute integer placement and storage dimensions. Keep the
primitive's loop-work field separate from tile metadata, so callers cannot
accidentally debit it twice. `materialize` first calls one aggregate preflight,
then fallibly reserves the bounded tile coverage/row vectors and returns a
complete collection. No partial collection escapes on either allocation failure.

Share a private scalar geometry/evaluation helper with the old implementation
only if the old ordering and observable refusals are preserved exactly. This
change needs no new shader, GPU source format, surface conversion or presenter
packet API. Each tile already fits `SourceMask` plus an absolute row table and
the existing `Command::Glyph` compositing path.

## Exactness and order

Preserve the original f32 intersection, reconstructed endpoint order, floor/
ceil bounds, radius clamping and upper-clip crop. Never intersect the floating
rectangle separately with a tile, clamp radius to tile dimensions, subtract a
tile origin from the geometry, or translate the result back. Those operations
can change coverage at f32 rounding boundaries or create artificial rounded
corners along the seam.

For each tile cell, use its original integer framebuffer `x` and `y` and the
original rectangle/radius in the unchanged Canvas expression:

```text
px = (x as f32) + 0.5
py = (y as f32) + 0.5
dx = max(rect.x + r - px, px - (rect.x + rect.width - r), 0)
dy = max(rect.y + r - py, py - (rect.y + rect.height - r), 0)
coverage = u8(clamp(r + 0.5 - sqrt(dx*dx + dy*dy), 0, 1) * 255)
```

This notation describes the existing explicit f32 operation sequence, not
permission to reassociate it or introduce fused operations. Coverage remains
geometry-only CPU output; color alpha flooring and rounded RGB blending stay
in the existing integer shader.

Emit all strips consecutively at the original command position, left to right,
before the next primitive or scope command. Half-open integer windows are
disjoint and their union equals the original cropped window, so a translucent
pixel blends exactly once. Row origins are absolute; do not apply document or
fixed offsets again. Leave the active clip/fixed state unchanged for every
tile. Keep zero-radius dispatch on its existing rectangle path, and preserve
geometry/scope validation before hidden or transparent omission. Do not skip
all-zero coverage tiles by scanning them: their geometry still determines
storage and dispatch policy.

## Global accounting

Preflight the complete tile group against the same scene ledgers before any
tile reserve or coverage evaluation. For K nonempty tiles replacing one
positive-radius rectangle:

- The original command count remains one. Lowered operations gain K−1 because
  the existing native `Ledger.operations` already counts the nontext rectangle.
  Reserve K source slots, K row-table slots and K rounded-output records.
- Coverage is the sum of tile areas, exactly the original cropped area. Rows
  are K times the common cropped height. Charge every row and its packed u32,
  all mask coverage words, and all additional 256-byte parameter records.
- Record the original Canvas loop rectangle's work once, before its upper-clip
  crop. Preserve the existing conservative whole-frame nontext debit once for
  the original rectangle; it already covers those coverage evaluations. Do
  not call `Ledger.rounded(info)` unchanged for both tiles: that would either
  duplicate the primitive's loop statistic or miss the extra lowered operation.
  Tiling metadata, row writes and packing remain separately bounded by their
  existing entry/storage ledgers; no work or budget resets occur per tile.
- Count each tile's real padded 8×8 dispatch in the final global GPU work
  check, including all other phases, clear and conversion. Do not assume
  cropped area equals dispatched work. The current final core refusal can
  remain after bounded CPU preparation; do not claim a new early-work guarantee.
- Keep the 256 combined sources/operations, 262,144 coverage bytes, 65,536 row
  entries, 16 MiB planned GPU bytes, 4M padded invocations, 16 MiB preparation
  reserve, 24 MiB active native/readback and 128 MiB application limits intact.
  Update the preparation-peak structural term for any new tile-owner type;
  the existing arrays reserved for at most 256 masks/operations must still
  bound retained headers and capacities. No extra monolithic coverage copy.

Implement group accounting transactionally on a local prospective ledger and
commit only after all checked additions succeed. Any refusal or allocation
failure still returns no scene plan and takes the complete CPU fallback.

For the actual 1084×34 bar the windows are `[181,1205)×[31,65)` and
`[1205,1265)×[31,65)`. Their dimensions are 1024×34 and 60×34, their areas are
34,816 and 2,040, and their row tables contain 34 entries each. The tiled packed
coverage/rows occupy `4*(36,856+68) = 147,696` bytes, excluding parameters.
Compared with a hypothetical single permitted mask this adds 136 row bytes
and one 256-byte parameter record: 392 explicit GPU bytes. Padded work is
`(128+8)*5*64 = 43,520` invocations, the same as the hypothetical monolithic
1084×34 dispatch because the 1024-column split is an 8-column multiple. Source
and operation counts still increase; there is no promise that every otherwise
near-limit full scene will be admitted.

## Files and focused checks

Likely production changes are confined to `crates/raster-core/src/rounded.rs`
(the explicit partition seam) and `src/graphics/raster_bridge/native.rs`
(group ledger, owned tiles and consecutive lowering). Extend their existing
`rounded_tests.rs` and native adapter private tests. Browser scene geometry,
Canvas, old Probe APIs/shaders, presenter and protocol need no semantic change.
If a new statistic is useful, keep `rounded_masks` as actual mask count and
add a distinct primitive count rather than silently relabeling it.

Freeze new expectations before implementation:

1. Literal wide-band case with radius 0.25: every included cell has coverage
   191, across the 1024 seam; check opaque and translucent source-over with
   independently calculated integer RGB targets. Radius-zero retains its
   existing fast-path clipping distinction.
2. Widths 1024, 1025, 1084 and 1280; exact nonoverlapping windows/row origins;
   the 1084×34 radius-7 bar plus analytic corner/seam/interior samples. Add a
   complete original-Canvas differential, labelled as a reference comparison
   rather than an independent golden.
3. Fractional/negative original origins and clips around an integer ULP;
   radius clamping; original geometry much wider than the viewport; fixed
   escape and restoration; interleaved image/text and overlapping translucent
   rectangles. Verify tiling changes no scope or original primitive order.
4. A crop where Canvas loop work exceeds the final mask area: primitive work
   recorded once, coverage cells visited once, and row/packing cost includes
   both tables. Verify one original command expands to two lowered operations.
5. Exact and one-over global coverage, source, row, lowered-command, packed
   storage and final invocation boundaries, including other phases/glyphs.
   Whole coverage overflow still refuses despite each tile being individually
   valid. An extra tile that exceeds a global limit must not allocate or return
   an earlier partial mask/plan.
6. Injected refusal before either reserve and failure of each first/second
   tile allocation; retained storage drops and no partial scene escapes.
   Existing Probe and legacy rounded refusal cases stay unchanged.

After these bounded checks, a new native host case can use the full 1280-wide
loaded scene and compare its acquired texture. Retain the preceding checkpoint
and any first-host refusal unchanged. Tiling removes one identified admission
blocker; it makes no performance, broad size-admission or native-host pass claim.

## Read-only source binding

Inspected source SHA-256 values:

```text
crates/raster-core/src/rounded.rs
b410aa8a65dae1b630c6bd951f3fac94c8bee074d9e89dccb6ba0ad1429eb6d4
crates/raster-core/src/rounded_tests.rs
a947006126aa5dc012b7e42253560013eb1c5b96425e2b2e8f39c73117d7a79a
src/graphics/raster_bridge/native.rs
e8290d6b2b578c29965e2439c3cf6a4d60e64294ebb6513b749daffe31601e29
src/browser/native_scene.rs
581a420a1e43ef4cdcc76ec27aba57b6605b5b137a099d32acae365ac65838a1
src/graphics.rs
90be6472b709eb447c39faf08e2db2c022a92c2b29abe153ed3680d83ef579ef
```

The independent script-agent spot-check identified the same f32 reintersection,
double-work debit and pre-axis-check seam risks. No engine outputs were used.
