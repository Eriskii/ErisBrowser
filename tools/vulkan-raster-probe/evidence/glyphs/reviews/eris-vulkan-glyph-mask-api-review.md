# Bounded glyph-mask bridge: API review

Read-only proposal against `5ad2cfd3582f92f8368a0a0fa2fd3bb7dd7af2f0`.
No renderer, planner, compiler, worker or GPU was executed. No pixel oracle or
implementation was authored. Native-window integration and viewport expansion
are separate tasks. This proposal preserves opaque RGB targets and continues to
refuse every opacity-group and rounded-shape command, including hidden ones.

## Recommended smallest coherent increment

Expose bounded access to the existing bundled-font coverage masks and the exact
Canvas text walk. Add one ordered GPU glyph-mask draw per reached visible glyph,
using a small integer-only kernel and the existing input arena, uniform stride,
output/readback buffers and process supervision. Keep the existing rectangle and
image shaders byte-identical. The CPU may generate font coverage and placement;
it must neither read nor paint the destination target during GPU preparation.

This covers the current regular/bold/monospace selection and synthetic italic
placement, without introducing shaping, font discovery, text atlases with
eviction across submissions, native-window drawing or group compositing.

## Exact current semantics to extract

| Current source | Required behavior |
| --- | --- |
| `src/graphics.rs:109` | A mask owns signed integer bearings, dimensions and one coverage byte per cell. Cache identity is `(face index, Unicode scalar, quantized size)`, not text color, italic flag or destination position. |
| `src/graphics.rs:126` | Only the three bundled DejaVu fonts are used. Monospace selects face 2 even when bold is also true; otherwise bold selects face 1. No font-file loading is needed. |
| `src/graphics.rs:144` | CSS size scales by face height/em. Text clamps its finite requested size to 1..512. |
| `src/graphics.rs:173` | Bitmap size is rounded to the nearest eighth pixel, while advance and kerning use the original clamped size. Coverage is `(a * 255.0) as u8`, truncating rather than rounding. Position is based on the quantized mask's ascent/bounds. |
| `src/graphics.rs:593` | Text rejects nonfinite x/y/size and alpha zero, then performs the existing empty/vertical-clip early return. Do not replace this coarse culling with a glyph-bound interpretation: it is observable CPU behavior. |
| `src/graphics.rs:625` | Each reached Unicode scalar consumes a glyph before kerning/right-edge termination. Kerning is added before the pen check; the previous glyph is set before the break. A skipped left-hand glyph still advances the pen. Unicode iteration is Rust `chars()`, not UTF-16, shaping or grapheme iteration. |
| `src/graphics.rs:639` | For each reached mask, Canvas charges `max(width * height, 1)`, including off-clip mask cells and empty outlines. Cache hits do not remove this repeated paint charge. |
| `src/graphics.rs:646` | For row `gy`, italic shear is `(size - (bearing_y as f32 + gy as f32)) * 0.18`; destination x is `(pen + shear).round() as i32 + bearing_x + gx`. Destination y is `y.round() as i32 + bearing_y + gy`. Preserve each f32 operation and round-before-integer-bearing order. |
| `src/graphics.rs:481` | Integer pixel origins must be inside the active half-open clip and framebuffer. Effective alpha is **floor** `color.a * coverage / 255`; only then does RGB source-over use `+127` followed by `/255`. Two rounding stages must not be combined. Target high byte stays zero. |
| `src/graphics.rs:748` | Clip/fixed scopes remain typed. A fixed scope restores the caller clip and uses the viewport offset, then restores both prior clip and offset on pop. Text coordinates receive `x + dx`, `y + dy` once at line 828, before pen arithmetic or rounding. |

Independent glyph dispatches preserve overlap between adjacent glyphs and other
draw commands. Combining a whole text run into an unordered dispatch or a single
coverage image could change overlap and per-glyph rounding. Within one glyph,
different source rows have distinct integer destination y and each source cell
has a distinct destination x, so one invocation per visible destination pixel is
race-free. Clear remains the existing unconditional opaque pass.

## Proposed safe API boundary

The names below are interface sketches, not implemented or frozen signatures:

```rust
pub struct TextRun<'a> { /* borrowed text; x/y/size/color and three style flags */ }
pub struct GlyphMask { /* private bearings/dimensions/coverage/key */ }
pub struct GlyphPlacement<'a> {
    pub mask: &'a GlyphMask,
    pub pen: f32,
    pub y: f32,
    pub size: f32,
    pub color: Color,
    pub italic: bool,
}

impl Fonts {
    pub fn visit_text_masks(
        &self,
        run: TextRun<'_>,
        clip: Rect,
        budget: &mut TextPreparationBudget,
        visit: impl FnMut(GlyphPlacement<'_>) -> Result<(), TextPreparationError>,
    ) -> Result<TextWalkCost, TextPreparationError>;
}
```

`GlyphMask` should expose immutable accessors for coverage, checked dimensions,
bearings and an opaque equality/hash key. Do not expose mutable cache fields,
`FontArc`, font paths or caller-constructible mask records. A visit borrows one
immutable mask only for the callback. Any cache `RefCell` borrow must be released
before invoking the callback. The caller packs coverage into its private owned
plan immediately and records a small stable-key-to-arena-offset table; it does
not retain the worker snapshot, author string or a clone of mask bytes in a
second long-lived vector. Font/cache lifetimes end before GPU initialization,
as they already do in the checker capture phase.

The visitor shares the exact text positioning/early-return code with Canvas,
ideally through a small internal walk primitive. Canvas and GPU preparation
must have separate admission/error adapters: a stricter bridge refusal cannot
silently truncate a Canvas draw or change its public exhausted flag. Retain the
old CPU call ordering and old scalar behavior when extracting that shared walk;
do not combine an unrelated budget-policy fix with this extraction.

An alternative API returning `Arc<GlyphMask>` is possible, but every retained
reference, cache eviction survivor and duplicate cache miss then needs explicit
lifetime accounting. A callback with one temporary lease is the smaller first
boundary. A frame-local mask cache also avoids retained references surviving
global cache clearing; its byte/count limits must be decided before coding.

## Allocation prerequisite: current bitmap is not a bounded API

`Fonts::bitmap` currently computes outline bounds, performs `vec![0; w * h]`
and rasterizes the outline at `src/graphics.rs:183` **before** the Canvas pixel
debit at line 641. It has no checked area multiplication or fallible reserve.
The cache stores only masks smaller than 16,384 bytes and clears at 2,048
entries (`src/graphics.rs:209`), but that does not bound a single uncached mask
allocation, transient outline work, hash-map capacity or external Arc holders.

Therefore the bridge must not merely make this existing method public and claim
that the Canvas pixel budget precharges font work. A checked mask-acquisition
path must validate finite outline bounds and signed bearings, check dimensions
and area, and debit unique mask storage/raster work **before** allocation and
`outline.draw`. Use fallible storage growth; account any cache insertion/table
reserve before it occurs. The bundled immutable fonts and 512-size clamp bound
the input family, but an exact bound on the font library's outline setup/edge
work still needs a separate review. Counting only raster callbacks is not proof
of a bound on all outline setup work.

The peer bounds audit identified a concrete dependency boundary, independently
confirmed by reading the pinned cached source: `ab_glyph-0.2.32/src/ttfp.rs:263`
builds the outline before returning its bounds; `src/ttfp/outliner.rs:33`, `:42`,
`:54`, `:62` grow a `Vec<OutlineCurve>` with ordinary pushes. Thus a check after
`outline_glyph` cannot precharge or make that outline allocation fallible.
`ab_glyph-0.2.32/src/outlined.rs:107` creates a rasterizer and tessellates all
curves before coverage callbacks. `ab_glyph_rasterizer-0.1.10/src/raster.rs:43`
allocates `4 * (width * height + 4)` bytes of f32 scratch with ordinary `vec!`;
its `:261` callback scan visits zero-coverage cells too and cannot be cancelled
through the callback signature. Include that temporary scratch alongside the
coverage byte vector and row/plan buffers in peak-memory bounds, and reject
before `draw`. A checked caller reserve cannot make the dependency's allocation
fallible: retain this trusted pinned-font limitation explicitly, or introduce a
separately reviewed counting/bounded outline provider before claiming complete
allocation/work control. The full peer audit is
`/tmp/eris-vulkan-glyph-mask-bounds-review.md`, SHA-256
`ed5f73fb6eb019dc2ee6f91d15c3894da16f7f8fff68a02509b814c56a8b6846`.

Cache hits must be checked against this plan's limits too. Cache state must not
turn a semantically identical over-budget frame into a GPU success. Count the
same unique mask bytes and required per-occurrence paint work regardless of
whether another CPU paint already populated the cache. A fixed per-plan scratch
cache or conservative cold-work accounting is preferable to cache-dependent
admission. Persistent Fonts cache allocations remain separately bounded and
explicitly outside the GPU byte ledger.

## GPU representation and reuse

The image shader (`tools/vulkan-raster-probe/src/image.wgsl:30`) uses independent
x/y source-index tables. Italic x placement depends on the mask row, so a single
ordinary image rectangle with those separable tables cannot represent it.
Even nonitalic text should use exact integer mask indexing, avoiding unnecessary
divide/multiply image sampling and its rounding near integer boundaries.

Use a new mask draw kind with one u32 coverage word per mask cell initially,
plus a bounded row-origin table. It is simpler to validate than packed bytes;
packing four bytes per word can be a later independently checked optimization.
Keep text tint in the existing color uniform. A possible 64-byte layout uses
the existing first 32 bytes, then mask base/width/height/cell count, row-table
base/signed destination-y origin/row count/reserved. The current 256-byte slot
stride remains (`lib.rs:292`). Signed origins should use an explicit bitcast
contract, bounded well inside i32 before serialization.

CPU row entries compute the exact formula from Canvas for each glyph row and
are absolute framebuffer coordinates. The primitive must not subtract a document
offset and re-add it through ordinary Image translation: f32 cancellation and
rounding can change the pen. Its signed y origin is absolute for the same reason.
The integer GPU kernel derives `gy = destination_y - glyph_origin_y` and
`gx = destination_x - row_origin_x[gy]`, checks both before indexing the mask,
then computes effective alpha and the existing source-over formula. Its dispatch
box is the union of row spans intersected with the integer-origin clip and
viewport. Clip endpoint additions use the original f32 order before conversion;
do not replace Canvas's containment with pixel-center clipping. A broad clipped
bounding box may include empty row margins; those invocations must still count.

Extract the current normalized clip/offset and typed scope updates into one
private helper used by both preparation and planning. A font-aware adapter can
replay this helper during its bounded walk rather than implement a second set of
intersections/fixed restoration rules. A second bounded state walk is acceptable
if both paths use the same helper and preserve f32 operation order; it is less
risky than rebuilding a text run's active clip after rounding. The absolute
mask primitive consumes the current clip but does not receive another translation.

Reuse the existing storage bindings and arena (`gpu.rs:119`), add a third
pipeline/kernel and select it in the ordered pass loop at `gpu.rs:164`. Preserve
all old parameter bytes/shaders on frames without text. The current
`Plan::has_images()` means simply nonempty input (`lib.rs:194`); separate the
generic input-buffer predicate from the actual image/mask draw kind before
adding mask-only plans. No dummy source buffer is needed when neither is used.

CPU pre-tinting masks into RGBA would also avoid final-target painting, but it
duplicates coverage by color and italic placement, needs padded-row allocations,
and weakens reuse. Reusing the image shader by one image command per glyph row
would rapidly exhaust the existing command/draw limit. The small mask kernel
keeps those complications out of the first increment.

## Bounds and transactional fallback

Keep the existing 320×240, 256 original commands, 32 combined scopes, 1 MiB
explicit GPU buffers and four-million-invocation caps. Original input count and
expanded glyph work are separate inventories. Count glyph draws against a
separate lowered-operation cap no greater than 256 initially; clear makes at
most 257 actual draws. Include preserved scope operations in expansion checks
before vector growth. Do not raise command limits to fit a long string.

Before any font lookup, sum UTF-8 lengths with checked arithmetic, check all
text scalars/flags/geometry and complete typed scope structure, and reject every
unsupported group/rounded command even if later hidden. Suggested conservative
new preparation ceilings, pending root selection: 64 KiB total text bytes,
4,096 visited scalars across the frame, at most 256 unique masks, and at most
256 KiB unique coverage bytes. The last ceiling is not an extra GPU allowance:
u32 mask expansion, row tables, image input, uniforms and two target buffers
must **together** fit the unchanged 1 MiB ledger. Refuse earlier when their
checked lower bound cannot fit, before mask allocation. Precharge row-table
generation, coverage packing and mask-key comparisons independently.

The current bridge's nontext CPU guarantee (`browser_adapter.rs:394`) uses
original command count × frame area. That is insufficient for text: one text
command can visit many masks, and Canvas charges full off-clip bitmap areas.
A conservative replacement is:

`nontext_original_commands * frame_area + sum(max(mask_area, 1) for every reached mask occurrence)`

Require that bound to fit Canvas's unchanged `(frame_area * 16).clamp(1M,32M)`
budget. Also preserve 32,768 per-text and 100,000 frame visited-glyph limits
(`graphics.rs:5`, `:625`, `:737`); the proposed bridge scalar cap is stricter.
Charges include repeated cached masks, empty outlines and fully clipped cells
whenever Canvas reaches them. Respect Canvas's earlier no-op checks so claims
about its exhausted flag remain justified. The separate cold mask generation
cost is additional preparation accounting, not a substitute for this paint bound.

All preparation remains before any GPU work. A mask/placement/cache/expansion
refusal discards the entire tentative plan and paints the original display list
through Canvas, preserving its existing partial-work/exhaustion behavior.
An exhausted CPU fallback must be reported as such, never as an exact complete
frame. Device, readback or comparison errors remain errors, not fallback.

## Concrete edit map and unresolved steps

1. `src/graphics.rs:109`, `:173`, `:593`: define immutable mask views/key and
   checked acquisition, extract exact text walking/placement with an error
   boundary that leaves the current Canvas policy intact. Decide cache scope,
   preallocation and library outline-work accounting before implementation.
2. `browser_adapter.rs:240`, `:303`, `:367`: add a distinct fonts-aware entry,
   such as `plan_snapshot_with_fonts(snapshot, frame, fonts)` and the matching
   display-list variant; accept an explicit borrowed Fonts/session parameter,
   reuse shared typed coordinate scopes,
   bound text bytes/visited glyphs and CPU fallback work, retain source-command
   attribution for refusal. Preserve existing `plan_snapshot` and display-list
   entry points as their current no-text subset. Their 16 frozen GPU/fallback
   routes must remain unchanged; new text-aware route expectations belong in a
   separate inventory. Do not invent a new Fonts per command.
3. `lib.rs:125`, `:198`, `:292`, `:448`, `:479`: add immutable mask draft/kind,
   arena offsets and row tables; distinguish original and expanded counts;
   precompute complete allocation/dispatch bounds before final packing.
4. New `glyph.wgsl`; `gpu.rs:50`, `:119`, `:164`: shared input/binding layout,
   ordered per-glyph pipeline dispatch, unchanged old shaders/readback semantics.
5. Checker/fixture/protocol extension requires independent literal synthetic
   coverage targets plus separately frozen real-font mask data. Real masks alone
   cannot independently validate an extraction that generates those same masks.
   CPU-vs-GPU equality alone is also insufficient if both use one defective new
   placement helper. Preserve pre-refactor CPU observations before implementation.
6. Coverage needed later: non-eighth sizes, half-pixel and negative pen rounding,
   kerning/overlap, row-dependent italic shear, bold+monospace precedence, empty
   outlines/zero advance, all alpha stages, fractional clips, fixed escape and
   restoration, cached/uncached repetition, and every allocation/work boundary.
   These are test-design requirements only; no expected fixture pixels were
   authored or executed here.

No runtime limit, oracle expectation, native-window cap or resource policy has
been approved by this review. The numerical text preparation ceilings are
proposals for a bounded first implementation, not measurements.

## Source identities

| File | SHA-256 |
| --- | --- |
| `src/graphics.rs` | `1b51c4af2c8ae4eea66476397868022da35e566d3593f3fcddf29a156bae62a2` |
| `tools/vulkan-raster-probe/src/lib.rs` | `e5a27bd65142bb943b8ae25a2f9461fd49edb1871500aa4c67f3abbdcf0ab4bd` |
| `tools/vulkan-raster-probe/src/image.wgsl` | `ede39857bf369d5a33f5503e7ae96bd04bd5ca06a11937f28fa97793011b3446` |
| `tools/vulkan-raster-probe/src/gpu.rs` | `e1b639f231739542c85e28a2434861bd785a1f803b36cf3c0433c4458d9a4226` |
| `tools/vulkan-raster-probe/src/browser_adapter.rs` | `1c0da92ef487589d821097446d0ac89d278c98246416620eb8199eb2577fce12` |
| `tools/vulkan-raster-probe/src/bin/browser-check.rs` | `ea5f41fb64cad07338bb0b11d85222a9b76f31cb306ae40e95e7bf572112a9cc` |
