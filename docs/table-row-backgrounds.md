# Table row backgrounds

A row's color and supported gradient/URL layers now paint through the final
border rectangles of cells originating in that row. Inter-cell spacing shows
the table background. A colspan keeps its internal nominal track gaps filled;
a rowspan retains its originating row's paint over the whole cell, including
the vertical gap inside the span. This follows the [CSS table-layer rule](https://www.w3.org/TR/CSS22/tables.html#table-layers)
within the current rectangular table model.

Painting area and image positioning area remain separate. All cells in a row
use one tile origin and size derived from that row, so percentages, centering
and repetition do not restart at each cell. A no-repeat image on a rowspan
stays within its original row tile; repetition can reach the remaining cell
area. Row width for image positioning uses the final track sum and internal
horizontal gaps. For two fixed 6px tracks in a 10px table with 2px spacing,
that area is 14px wide, even though the existing row hit rectangle remains
6px wide. This changes painting phase without changing the table's sizing or
hit policy.

Missing grid slots have no originating cell and receive no invented row
paint. A real empty `td` or `th` still receives its row background under the
supported default-show behavior. The [Tables 3 draft's missing-cell rule](https://www.w3.org/TR/2025/WD-css-tables-3-20251216/#drawing-cell-backgrounds)
and its [WPT source](https://github.com/web-platform-tests/wpt/blob/master/css/css-tables/paint/row-background-paint-remove-last-cell.html)
make that distinction explicit. This is source-backed scope selection, not a
cross-browser execution result. An entirely empty row can keep its existing
layout height while contributing no cell background paint.

Image indices, layer order and resource bindings remain attached to the row.
A `none` or unavailable layer keeps its slot. The first authored image stays
on top; the row color is below its image layers, and cell backgrounds paint
above the row. Existing restacking retains row opacity and descendant order.
The same loaded keys and raster allocations are used; this change adds no
resource request or new rendering command.

A fully transparent row with exactly one `none` image returns before extra
work, command or key admission. For other rows, the final track sum is computed
lazily once after a prepaid column-count charge. Each real cell then reserves
one complete `1 + N` command/owner group and `16 + 16*N` units of the existing
geometry-work allowance, where `N` is the retained image-slot count. An opaque
color-only row has `N = 0`. Existing URL key clones additionally cost `len + 1`
work and `len` cumulative key bytes, within the unchanged 4,096/65,536-byte
per-key/frame bounds.

Refusing group capacity or geometry work omits that cell's row decoration
without changing its already computed layout or hits. Invalid principal
geometry is omitted without clamping its phase or substituting a child's
bounds. A refused URL key leaves that layer empty while other admitted layers
survive. Paint work is not refunded. Painted rows may now consume more
commands and key copies; the existing limits can therefore omit paint or
trigger whole-frame Native fallback earlier. There is no new per-cell style
clone, table membership matrix, shader, quota or resource authority.

Full intrinsic table sizing, row-group and column backgrounds, `empty-cells`
parsing/hide, `border-collapse`, general relative/rounded-cell clipping,
anonymous table construction and writing-mode/RTL table behavior remain
outside this increment. Caption, spacing and empty-table geometry retain
their earlier policies. These are focused table-background corrections,
not a claim of full table conformance or hardware performance.

Run `./run.sh ./examples/table-row-backgrounds.html` for gaps, spans and shared
image phase. Four 320×240 pairs use independently frozen literal RGBA frames
and solid positioned-rectangle references, including fixed-track overflow,
no-repeat and layer order.

Validation used source inventory `3ae18e91afd14480e770fdc6a48096c17a4fe23eab4e96ed6c2a2941dd612315`
(727 inputs). Rust 1.88 passed all 2,998 tests with `vulkan-raster` and 2,694
with default features, including ignored tests. Strict all-target Clippy passed
on Rust 1.88 and 1.98; formatting passed. Nineteen new Rust groups cover layout,
Page pixels and the confined worker. All five new Page tests failed on the
unchanged parent at the intended coverage/phase pixels before implementation.

All 102 self-authored reference pairs passed. The eight new PNGs match four
independently frozen full RGBA frames, and all 196 prior PNGs are byte-identical
to the parent's published archive. The demo rendered without diagnostics;
its identical image asset resides beneath the local document directory.
Native witnesses interpret packed plans on the CPU; these checks do not
establish GPU, acquired-window or Chromium-relative performance.

Rust 1.88 x86-64 DWARF reports unchanged sizes for `ComputedStyle` (1,032 bytes),
`PaintOwner` (16), `BoxGeometry` (72), `Fragment` (136), `InlineItem` (176) and
`InlineKind` (152). These measurements precede a test-fixture-only correction;
the relevant production definitions are unchanged. They do not measure total
allocator usage or stack frames.

The first default run retained a timezone-helper fixture failure with Linux
`ETXTBSY` (error 26). A controlled reproduction showed that an inherited
writable descriptor can cause that error after the parent closes its copy;
it did not identify the original retaining process. The test helper now writes
its executable only in a waited child process. Existing script bytes, error
assertions and deadlines are unchanged, with no production retry. All 18 helper
tests and both complete suites passed after this fixture correction. The first
failure, exact patch and controlled witness are retained in the evidence.

The frozen browser SHA-256 is
`d6fd6c803fe9fe380f99f1694aedb3c2e042d997f4f9305c489f6ee236b06ab9`.
It was built from source inventory 1; only `src/date_host/tests.rs` changed in
inventory 2. Every production input remains identical. The parent commit
`c06bce2a` passed all nine [GitHub checks](https://github.com/Eriskii/ErisBrowser/actions/runs/38157087380);
this increment's CI runs after publication.

The [evidence summary](evidence/table-row-backgrounds/summary.json) and
[source/check archive](evidence/table-row-backgrounds/sources-and-checks.tar.gz)
bind the selected source, raw results, literal expectations, reviews and
retained failures. These local cases do not establish full table conformance
or production security.
