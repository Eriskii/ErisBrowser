# CSS visibility: visible and hidden

`visibility: hidden` now hides a box's own painting and hit targets while
retaining its layout space. A descendant can set `visibility: visible` and
remain painted and interactive. Block/inline flow, flex/grid tracks, table
geometry and document scroll extent continue to include hidden content. This is
the inherited `visible`/`hidden` subset described by [CSS Display](https://www.w3.org/TR/2026/CRD-css-display-3-20260605/#visibility).
`collapse` remains unsupported; this checkpoint does not implement its table or
flex behavior.

The initial value is `visible`. The existing cascade supports `inherit`,
`initial`, `unset`, `revert` and `revert-layer`, including importance and layer
ordering. Values are single identifiers, ASCII-insensitive and escape-aware;
comments cannot join separate identifiers into a keyword. Quoted values,
additional tokens and `collapse` do not replace an earlier valid declaration.
Ordinary variable substitution remains supported. If the winning variable value
is invalid at computed-value time, visibility inherits as `unset` rather than
reviving the earlier declaration, following [CSS Variables](https://drafts.csswg.org/css-variables-1/).

Inline `style.visibility`, `setProperty`, removal and serialization use the
existing bounded CSSOM machinery. Invalid setter values remain no-ops. Feature
queries accept supported visibility identifiers, including escaped spellings.
Condition syntax can decode an escaped visibility property name; the
two-argument `CSS.supports(property, value)` overload retains its literal
property-name rule and rejects escapes in that argument. The existing
conservative feature-query treatment of unresolved `var()` remains unchanged.

Painting is filtered after layout and scope construction. Hidden ancestors do
not cut off visible descendants or erase clip, fixed-coordinate and opacity
scopes. Native focus similarly checks each candidate's own value after deriving
reachability from hit geometry; a visible child does not make its hidden parent
focusable. Offscreen visible controls remain eligible. Accepted hiding clears
native editing focus, and showing a node does not automatically focus it.
DOM event paths remain intact, so a visible child's event can still bubble
through a hidden ancestor; visibility does not make the subtree inert.

The canvas background remains painted when its selected html/body source is
hidden, while ordinary hidden borders and content are suppressed. This follows
the [canvas-versus-element background distinction](https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/#special-backgrounds).
Table background layers need a separate qualification. Existing row fragments
use their originating cell's visibility while retaining row geometry, rank and
opacity: a hidden row with a visible cell paints that row layer; a visible row
with a hidden cell does not. A visible descendant inside a hidden cell paints
its own content without reviving the cell's row layer. This is the selected
cell-rendered interpretation of [CSS Tables 3](https://www.w3.org/TR/2026/WD-css-tables-3-20261008/#drawing-cell-backgrounds),
a working draft explicitly marked not ready for implementation. Its recorded
browser disagreements and open visibility question remain relevant; these
controls are not a claim of settled cross-browser interoperability.

Each layout carries one typed visibility value for every DOM node. Construction
checks the existing 100,000-node bound and reserves the complete table before
copying values. Refusal returns no paint/hit records and no table; it cannot be
encoded as a successful worker snapshot for a nonempty DOM. Hidden content
still consumes the existing resource preparation and layout/glyph/command
allowances. No resource cap is raised.

The worker protocol advances to ERWB. After the DOM records, it carries an
exact count and one 0/1 byte per node: `4 + N` additional bytes, at most 100,004.
The decoder checks cardinality, available bytes and every discriminant before
allocating the typed vector. Hidden-node hits are refused in either direction;
older protocol magic is rejected. Missing metadata excludes all native focus
candidates. These checks validate transport shape, not the correctness of an
untrusted worker's CSS computation, and add no sandbox authority.

Actual Rust 1.88 measurements on this host keep `ComputedStyle` at 1,032 bytes.
`PaintOwner` grows from 16 to 24 bytes and `LayoutResult` from 56 to 80. The
visibility vector adds one byte per stored node; capacity and allocator overhead
are additional. At the unchanged 200,000-command limit, one equally sized owner
roster gains 1,600,000 payload bytes. That arithmetic is not a peak-allocation
bound: construction can retain more than one vector.

The independent controls use literal CSS/JS expectations and reference pages
without visibility. Rectangular scenes compare complete RGB canvases with fixed
rectangles; the text pair uses transparent text to retain layout. All sixteen
parent integration groups compiled and failed, including the missing CSSOM
property's Unsupported result. Separate direct and confined dynamic witnesses
also failed on the parent's hidden red pixel. The final suites include all
thirty-six added groups covering CSS, pixels/CSSOM, focus, codec, dynamic
behavior and resource boundaries, with no old groups removed.

The selected source passed 3,521 tests on Rust 1.88 and 3,885 on Rust 1.99 with
`test262-host,vulkan-raster`, across 58 result rows each, with zero failures or
ignored tests. Both strict Clippy configurations and the formatting check
passed. The release browser passed all 102 existing reference pairs; all 204
test/reference images also matched the published parent exactly. The selected
inventory contains 15,307 source/build/tool/test inputs and 35 changed source
paths; it excludes documentation, build outputs and external dependencies.

An eighteen-process CPU comparison used the parent and candidate release
browsers with fixed visible, hidden and reference scenes. All process, image and
cleanup checks passed. Three alternating-order pairs per rendering path measured
the all-visible dense scene with 100 warm iterations per process. The observed
candidate/parent median ratios were:

| Pair | Direct paint | Confined worker total |
| --- | ---: | ---: |
| 1 | 1.0307 | 1.0430 |
| 2 | 0.9782 | 1.0640 |
| 3 | 1.0384 | 1.0416 |

Pooling the worker's 300 warm samples per revision gives median totals of
2.824274 ms for the parent and 2.964620 ms for the candidate. Direct results are
rounded process summaries and are not pooled as raw samples. These are local,
uncontrolled measurements, with no speed threshold; reduced hidden painting is
a behavioral change, not evidence of a universal speedup. No Chromium comparison
or GPU execution was performed for this checkpoint.

Failures and corrections remain in the evidence. Before execution, source
review made table fixture widths explicit, repaired a converter's missing
property name and supplied a missed visibility field in an old layout fixture.
The first full run's top-level result was 3,142 passes and twelve failures:
four old `all` fixtures had stale roster/byte assumptions, and eight socket
preflights met outer sandbox `EPERM`. The fixture migration accounts for 87
longhands while preserving the exact 256-record and 65,536-byte limits; its
one-byte filler received a source-only lint refinement. Full suites were then
rerun with the required socket permission. The first measurement attempt stopped
at the parent before accepting any samples because `TZ=UTC` required unavailable
timezone data. Cleanup completed; runner2 uses the equivalent POSIX offset
`UTC0`, preserving scene bytes, order, pixels and supervision.

The published parent's eleven successful CI jobs remain a parent result. This
checkpoint makes no complete CSS/browser compatibility, production-security or
Chromium performance claim. Sources, commands, earlier holds and failed attempts
are retained in the [evidence manifest](evidence/css-visibility/manifest.json)
and [source/check archive](evidence/css-visibility/sources-and-checks.tar.gz).
