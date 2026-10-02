# Bounded glyph-mask preparation review

Read-only preparation for the optional offscreen browser bridge, after checkpoint `5ad2cfd`. No compiler, Canvas, font rasterizer, worker, GPU or new oracle was executed. No implementation or existing limit was changed. The proposed admission ceilings below are a design recommendation for root review, not newly accepted policy.

The smallest honest increment supplies coverage masks from the existing three bundled fonts, then lets a custom integer GPU kernel apply placement, clipping and source-over. It must not upload a CPU-painted target, color-composited text rectangle or final frame. Matching these masks reproduces the current CPU text painter; it does not add shaping, browser-complete typography or arbitrary font support.

## Existing bounds and their exact reach

| Boundary | Current value and source | What it actually bounds |
|---|---|---|
| CPU display list | 200,000 commands; combined typed scopes 128 (`graphics.rs:5–11,716`) | Commands/scopes consumed by one `paint_with_viewport` call. |
| CPU text loop | 32,768 Unicode scalar iterations per Text command; 100,000 across a paint (`graphics.rs:625`) | Each reached scalar is charged before glyph lookup, kerning and horizontal termination. These are not grapheme counts. |
| CPU pixels | `clamp(width*height*16,1,000,000,32,000,000)` | Rect/image loops and full reached glyph bitmap area. A glyph without a bitmap still debits one. |
| Existing bridge | 320×240, 256 original commands, 32 scopes, 256 image sources, 1 MiB declared RGBA, 1 MiB explicit GPU buffers, 4,000,000 invocations | Keep unchanged. On its largest frame the CPU pixel budget is 1,228,800. The explicit GPU sum includes output, readback, metadata and input arena. |
| CPU glyph cache | At most 2,048 entries; only masks smaller than 16,384 bytes are cached | Retained alpha payload is at most `2048*16383 = 33,552,384` bytes, plus bitmap/Arc/hash-table overhead. Clearing retains hash-table capacity. Large masks are rasterized but not cached. This is not a cumulative allocation or work limit. |
| Worker decoder | 8 MiB shared text bytes, 500,000 scalars, text size 0..512, finite geometry; normal typed scopes | The decoded snapshot is bounded more loosely than paint/bridge admission. A public Snapshot or direct command list is not an admission token. |
| Layout | 500,000 source/emitted scalars, 200,000 commands (`layout.rs:10,1210`) | Bounds normal layout output. It does not replace independent bridge validation. |

`consume_pixels` is called **after** `Fonts::bitmap`. On a cold cache miss, outline allocation, alpha-vector allocation, raster scratch allocation and rasterization have already happened. Therefore the existing Canvas pixel debit is not a precharge for font rasterization. Public direct `Canvas::text` calls outside painting also have no shared PaintBudget, though the per-command 32,768 cutoff remains. `Fonts::measure` itself walks all supplied scalars without this paint budget.

## Required semantic identity

`Fonts::face_index` selects mono before bold: mono+bold uses DejaVuSansMono, not a fourth face. Italic is synthetic shear, not an italic font. Size is finite and clamped to 1..512. Pen advances and pair kerning use this unquantized size. The mask cache uses `(face index, Unicode scalar, round(size*8) as u16)` and builds the outline at `q/8`; the baseline is its quantized font ascent. Consequently two sizes can share coverage while having different advance and italic placement. Do not quantize the pen or use glyph advance as the bitmap bounds.

For each scalar, preserve this order: scalar charge; glyph ID; kern from previous glyph; record previous ID; stop if `pen > clip.right + size`; if `pen + size >= clip.left`, obtain mask and debit `max(width*height,1)`; paint all bitmap pixels; then add horizontal advance. Zero-advance/combining/control scalars cannot bypass scalar limits. Missing codepoints use the font's glyph ID zero. The current cache keys by scalar even when several scalars resolve to that same glyph.

The bitmap placement is:

```
row_x = round(pen + (italic ? (size - (bitmap_y + row))*0.18 : 0))
x = row_x + bitmap_x + column
y = round(text_y) + bitmap_y + row
```

Retain the original f32 operation order and Rust rounding. An integer kernel can consume precomputed checked row origins. A single sheared quad or ordinary image rectangle does not express these independently rounded rows. Compute/check each row origin and all integer sums before publishing a draw; offsets, negative bearings and italic extents can exceed nominal font size. Neither a 512 size cap nor horizontal advance proves a 512×512 mask bound.

Coverage bytes are `(coverage_f32*255.0) as u8` using the current saturation/truncation behavior, including fully uncovered cells. Effective alpha is **floor**(`text_alpha*coverage_byte/255`); RGB then uses `(src*a + dst*(255-a) + 127)/255` on each pass. The destination high byte stays zero. Keep coverage separate from text color/alpha for reuse; never combine overlapping glyphs into one preblended mask. Draw ordering matters for overlap, kerning and repeated low alpha.

## Cold-raster boundary: concrete dependency limits

The pinned `ab_glyph 0.2.32` implementation first builds `Vec<OutlineCurve>` through ordinary `push` calls inside `outline_glyph`; pixel bounds are available only afterward. `OutlinedGlyph::draw` creates `ab_glyph_rasterizer 0.1.10::Rasterizer` with an ordinary `vec![0.0; width*height+4]`. Its scratch payload is exactly `4*(M+4)` bytes for mask area M, before allocator overhead. It walks/tessellates outline curves and finally invokes the coverage callback exactly M times, including zero coverage. The callback cannot stop this infallible `draw` operation early.

Area bounds alone do not account for outline work. Quadratic subdivision depends on scaled control-point deviation; cubic subdivision has depth 16 and may generate many line segments. Raster lines scan rows and possibly horizontal spans. TrueType composite recursion has a depth cutoff of 32 in the pinned parser, but that is not a total component/curve or allocation quota. Font-size bounds alone cannot substitute for these facts.

For the first increment, the acceptable narrow trust boundary is **only the hash-bound bundled font bytes and pinned parser/rasterizer**, with no author fonts, system-font discovery or font upload. Do not claim that a checked mask allocation makes all dependency allocation fallible. There are two honest choices before implementation:

1. Keep existing trusted bundled outline/raster internals as an explicit bounded-input dependency limitation. Add mask/scratch dimension admission **before draw**, bound cold requests/cumulative mask work, and state that outline construction and rasterizer internal reserves can still abort on allocation failure. This is a scoped optional path, not a general untrusted-font service.
2. If the API promises fully fallible/precharged font preparation, add a reviewed outline-provider/counting seam first: bound raw font-table/composite traversal and curves before storage growth, reserve curve storage fallibly, and use a rasterizer with a fallible scratch constructor plus charged tessellation. Calling current `outline_glyph` and checking its result afterward cannot fulfill that promise.

A static per-font/per-glyph complexity inventory could justify a conservative cold-work allowance for fixed inputs, but no such inventory or worst-case proof was produced here. Do not invent one from successful sample glyphs. The existing callbacks and cache do not prove a CPU-time deadline.

## Proposed preparation contract

Keep the published rectangle/image-only adapter entry points and their frozen 16-case routes unchanged. Add a separately named fonts-aware entry point with a separately frozen oracle; hidden Text must not silently change the old API's unsupported-text result. Share reviewed clip/fixed normalization rather than duplicating it.

Use a borrowed TextRun plus active typed clip/offset state, shared per-frame preparation budget, and a callback receiving an immutable mask view, stable semantic key and placement information. Release all `RefCell` cache borrows before invoking callbacks; pin the mask with a local owner if needed. A callback failure aborts preparation and discards the entire private plan before any GPU submission. Returning a plan must retain no snapshot, string or borrowed glyph memory.

Recommended conservative additional admission ceilings, subject to root acceptance:

- Aggregate UTF-8 Text payload at most 65,536 bytes; at most 32,768 bytes per Text command. Check lengths before scanning; scalar limits remain separately at or below the existing 32,768/run and 100,000/paint ceilings. Byte limits are a new narrower bridge admission rule, not a change to layout or Canvas.
- At most 256 distinct nonempty glyph-mask sources and at most 256 lowered drawing records **combined with rectangles/images**; clear remains the existing extra draw. Use the existing source/draw ceilings, not a second independent allowance of 256 each. Scalar/no-outline visitation has its own budget and must not allocate one record per character.
- Before rasterizing any new mask: finite integral nonnegative dimensions, checked M=`w*h`, at most 1,024 per axis and 262,144 pixels per glyph, and M no larger than remaining cumulative preparation/CPU pixel allowance. These are optional conservative refusal limits; bundled glyphs outside them fall back as a whole frame. They do not assert actual font maxima or silently truncate coverage.
- Debit `max(M,1)` for **every reached glyph occurrence**, including cache hits, off-clip mask pixels and empty outlines, against the same prospective CPU paint-work sum as ordinary commands. Charge metadata traversal/UTF-8/visited scalars separately. A valid Plan must leave the original CPU comparison able to complete under unchanged Canvas quotas.
- Count cold outline requests and raster work separately from occurrences/retained masks. Deterministic admission should not depend on an unrelated previously warmed or cleared CPU cache; either conservatively admit as cold or use an explicitly bounded frame-local preparation cache. Color, alpha, placement and italic do not change coverage identity; font identity/version, face, scalar and quantized size do.
- Reserve bounded metadata tables/row origins/coverage copies fallibly before growth. Cap live raw mask payload separately at 1 MiB and preflight the cold mask plus `4*(M+4)` raster scratch; a maximum-size proposed glyph needs 262,144 alpha bytes and 1,048,592 scratch bytes, **not** just its alpha payload. Existing global Fonts-cache memory remains separate and must be disclosed or avoided by a frame-local provider.
- Atlas is optional: a packed linear input arena avoids rectangle-packer search and fragmentation. For byte-packed masks account `4*ceil(M/4)` plus row-origin words and metadata; for one-u32-per-coverage account `4*M`. Charge the actual selected representation and checked offsets, including padding, before allocation. Images, source-index tables, glyph masks, output/readback and all metadata share the unchanged 1 MiB explicit GPU-buffer cap. Mask source totals do not authorize raising it.
- Preserve the unchanged invocation limit using actual rounded dispatch work; many tiny glyphs cost at least one workgroup each. Reject before publication if source, placement, arena, dispatch or cumulative work limits fail. Never split into unbounded batches or resume another GPU pass after a failed whole-frame admission.

If root chooses different new conservative text/mask admission numbers, freeze them explicitly before any outcome measurement. The important requirements are preflight order, combined counters, actual representation sizes and honest dependency limits, not treating these recommendations as already implemented constants.

## Hidden text, clipping and fallback

CPU `text` returns before scalar lookup/rasterization for zero alpha, nonfinite inputs, suppressed opacity, empty clip, or its vertical coarse exclusion (`y > clip.bottom` or `y + size*1.5 < clip.top`). Horizontal termination is inside the scalar loop and still charges that reached scalar and kerning lookup. Preserve these exact behavioral boundaries; a new helper should not rasterize hidden text eagerly merely to build an atlas.

The bridge must still validate original Text geometry/style/byte ceilings and typed scope structure before treating it as hidden. Nonfinite/malformed hidden input remains admission refusal. Unsupported opacity groups (including opacity 1) and rounded rectangles retain whole-frame refusal even when invisible. Fixed scopes reset to caller clip/viewport offset and restore the prior state. Final integer pixel origins must satisfy the same lower/upper clip tests as `Canvas::blend`; bounding-box intersection alone does not reproduce fractional upper edges.

On any limit/refusal, the CPU fallback paints the original **complete** list and keeps its normal exhaustion status. No partial GPU prefix, cropped mask that silently changes visible pixels, opacity flattening or rewritten string is an acceptable fallback. Runtime/raster/driver failures remain failures, distinct from a planned unsupported/limit admission category.

## Independent checks to freeze before implementation

1. Pixel cases: ordinary/bold/mono (including mono+bold), synthetic italic rows, fractional/negative pen and y, negative bearings, quantization boundaries with equal masks but unequal advances, kerning pairs, repeated alpha 1/128/255, overlapping masks and fixed/fractional clips.
2. Scalar cases: spaces/no outline, missing glyphs, combining/zero-advance sequences, multibyte UTF-8, horizontal early stop and vertical/alpha-zero/empty-clip early return. Retain no-shaping expectations explicitly.
3. Budget cases: exact/one-over bytes, scalars, nonempty sources, lowered records, raw M, raster scratch, cumulative repeated-glyph work, row-origin storage, arena bytes and rounded dispatches. Test cold and warm cache behavior and cache clearing without pointer-identity assumptions.
4. Failure cases: preflight refuses before raster callback/storage publication; callback error unwinds with no live cache borrow; earlier prepared masks/plans are dropped; no partial GPU submission. Keep independent CPU exhaustion controls separate from glyph preparation refusal.
5. Private accounting tests should use an instrumented mask provider to prove zero cold-raster calls for rejected dimensions/budgets. Exact-mask tests bind trusted font hashes; custom GPU readback must match both the original CPU painter and independently frozen expectations for chosen cases.

## Typography and evidence limits

Current text is Unicode-scalar glyph lookup plus pair kerning and advances. It does not perform grapheme shaping, script shaping, ligatures, bidirectional reordering, font fallback, color emoji, variable-font selection, subpixel LCD rendering or complete CSS typography. Layout has already chosen positions/line breaks; the glyph-mask path must not add an inconsistent second shaper. Missing glyphs and synthetic italic/bold/mono behavior remain current CPU behavior. The existing rounded/opacity restrictions remain unchanged.

Primary implementation sources are the local pinned crate source, not inferred dependency behavior. Attempts to open the published pinned documentation URLs below failed in the web tool; no remote content was used as evidence: https://docs.rs/ab_glyph/0.2.32/ab_glyph/struct.OutlinedGlyph.html#method.draw and https://docs.rs/ab_glyph_rasterizer/0.1.10/ab_glyph_rasterizer/struct.Rasterizer.html#method.new .

## Exact source bindings

- `/home/eriskii/Documents/Programming/Projects/ErisBrowser/src/graphics.rs` — `1b51c4af2c8ae4eea66476397868022da35e566d3593f3fcddf29a156bae62a2`
- `/home/eriskii/Documents/Programming/Projects/ErisBrowser/src/layout.rs` — `bfbec78e9c7ca3b5ec45dfb6a2b948b5192de40f93d86b19e6425437579e83e4`
- `/home/eriskii/Documents/Programming/Projects/ErisBrowser/src/worker/codec.rs` — `bdad12aa8ddde871aa121ccd6ae4b372e3ff42264d376a5422851771595f7d28`
- `/home/eriskii/Documents/Programming/Projects/ErisBrowser/Cargo.lock` — `fc2d595602095e8a4333975b5f2e8ba748a1b82fbdc84505e64abcbd2b92a0de`
- `/home/eriskii/Documents/Programming/Projects/ErisBrowser/assets/DejaVuSans.ttf` — `58568c88a01b80dfc056daca3eeafd434a0380df1a48dda53f0de0d716109bbd`
- `/home/eriskii/Documents/Programming/Projects/ErisBrowser/assets/DejaVuSans-Bold.ttf` — `7c7aef63328a765586cda41bf9d2c227ff1ad82816326dae4d7fe2fd1b7f613a`
- `/home/eriskii/Documents/Programming/Projects/ErisBrowser/assets/DejaVuSansMono.ttf` — `9d9bfebceb1c3f6f4ad383ded568a6926086208f43f7f92f92f7e93a1383fa38`
- `/home/eriskii/Documents/Programming/Projects/ErisBrowser/tools/vulkan-raster-probe/src/browser_adapter.rs` — `1c0da92ef487589d821097446d0ac89d278c98246416620eb8199eb2577fce12`
- `/home/eriskii/Documents/Programming/Projects/ErisBrowser/tools/vulkan-raster-probe/src/lib.rs` — `e5a27bd65142bb943b8ae25a2f9461fd49edb1871500aa4c67f3abbdcf0ab4bd`
- `/home/eriskii/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ab_glyph-0.2.32/src/outlined.rs` — `a884185f073677b3682a4eae81edabac9c21f0d348c13d3f01ea709a85ee4a75`
- `/home/eriskii/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ab_glyph-0.2.32/src/ttfp.rs` — `6b4f515a31a4d271dc23671d63ac36480cc6f9b042f616457b0d1a36ea7fc127`
- `/home/eriskii/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ab_glyph-0.2.32/src/ttfp/outliner.rs` — `e4ef3695155e5da8ab7260eeac5ac3e9e5f2d64d588a5ee625571e0eed63154c`
- `/home/eriskii/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ab_glyph_rasterizer-0.1.10/src/raster.rs` — `83012926ebd07a965de9831b35081638b8f066e8f4bb5ed4e453eae4f642acce`
- `/home/eriskii/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ttf-parser-0.25.1/src/tables/glyf.rs` — `6a03b8d92f12566fd1dc244f4fd0cd9a8babb7f79917d9cf5812eef1350df809`
- Coordination notes `/tmp/eris-vulkan-glyph-root-notes.md` — `7c549de4ab1505fb93aa459634cb908c331489e658ffbadc5091606b66d11881`; the proposed absolute integer row origins and separate full-mask work accounting agree with this review.
