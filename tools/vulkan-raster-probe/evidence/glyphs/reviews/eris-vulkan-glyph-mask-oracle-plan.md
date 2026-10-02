# Bounded glyph-mask oracle plan

Status: read-only preparation against published `5ad2cfd3582f92f8368a0a0fa2fd3bb7dd7af2f0`. No font rasterizer, Canvas, planner, compiler, worker or GPU was executed. No fixture, mask, pixel golden or implementation was authored. The complete custom-browser goal remains unfinished.

The next useful text increment is **GPU compositing of CPU-generated glyph coverage masks**, retaining the current custom layout and font selection. Keep two separately reported test populations: independently specified mask/placement arithmetic, and pinned-font integration comparisons against the preserved CPU implementation. Neither population alone proves general text or font-rendering correctness.

## Current semantics that the oracle must preserve

Source of truth is `src/graphics.rs` (`Fonts` at lines 126–220, `Canvas::blend` at 481, `Canvas::text` at 593, display-list translation at 819). Current SHA-256 is `1b51c4af2c8ae4eea66476397868022da35e566d3593f3fcddf29a156bae62a2`.

- The three faces are regular DejaVu Sans, bold DejaVu Sans, and regular DejaVu Sans Mono. Monospace takes precedence over bold; italic selects no additional font. It is a synthetic per-row shear. There is no font fallback or shaping pass in `Canvas::text`; it iterates Unicode scalar values, looks up glyph IDs, applies pair kerning and advances.
- `size` is clamped to `[1,512]` after rejecting nonfinite text inputs. The `ab_glyph` scale is evaluated in this order: `size * height_unscaled / units_per_em`, with a 2048 fallback only if that font API reports no em value. Advances and kerning use the original clamped size.
- Coverage is cached at `q = round(size * 8)` and generated at `q / 8`. Thus matching masks do not imply matching advances. Cache identity is `(face index, Unicode scalar, q)`, not color, position, italic flag or glyph ID. The quantization round is Rust's `f32::round`, not Python's default ties-to-even round.
- A mask is rasterized at `(0, scaled ascent)`, with integer bounds from `outline.px_bounds()`. Stored mask offsets are those bound minima. Coverage bytes are `(coverage_f32 * 255) as u8`: truncating/saturating conversion, not nearest integer rounding. No-outline glyphs produce a zero-sized mask. A missing character maps to glyph ID 0 in the pinned font implementation; it must not be assumed blank.
- Kerning is added before each glyph's placement; advance follows the glyph. For mask row `gy`, italic shear is `(size - (bitmap.y + gy)) * 0.18`, evaluated as the existing `f32` expression. Destination coordinates are `round(pen + shear) + bitmap.x + gx` and `round(y) + bitmap.y + gy`. The display-list document/fixed offsets have already been added to text `x,y`. Do not apply them twice, rasterize at the fractional pen, round the advance, or replace the row-dependent shear with one glyph translation.
- Clips test integer destination pixel origins through `Rect::contains`, with both upper edges exclusive. Fixed scope restores the caller clip and switches to viewport offsets; popping restores the prior clip and document/fixed offset. Text color alpha and mask coverage are combined before the existing integer source-over operation.
- The current text path has observable culling and accounting order: early nonfinite/transparent/suppressed/vertical-clip exits; then per-character glyph charging, kerning and right-edge stop; then left-edge selection and mask acquisition; then a charge for `max(mask width * height, 1)`, including off-clip mask pixels. Spaces/no-outline glyphs and repeated zero-advance scalars must not become free work. Per-text 32,768 and per-paint 100,000 glyph limits and paint pixel limits remain unchanged.

The pinned library distinguishes layout advances from actual outline pixel bounds, which may extend left or right of those advances. Use the bounds for mask storage, not for text layout. Its API explicitly provides glyph IDs, unscaled metrics and kerning separately. [ab_glyph 0.2.32 Font documentation](https://docs.rs/ab_glyph/0.2.32/ab_glyph/trait.Font.html), [OutlinedGlyph documentation](https://docs.rs/ab_glyph/latest/ab_glyph/struct.OutlinedGlyph.html).

The exact local primary implementation was also inspected, without executing it: `ab_glyph-0.2.32/src/{font,scale,outlined,ttfp}.rs` under the cached Cargo registry. Two version-pinned web documentation requests failed; local pinned source supplied those details. In particular, `ttfp.rs` maps absent characters to ID 0, and `outlined.rs` uses outline bounds and coverage callbacks. No fact here depends on another browser's output.

## Independent mask and arithmetic population

Prepare this population before the GPU glyph implementation or any candidate outcome. It should accept explicit tiny mask bytes and explicit placement metadata at the same lower-level boundary used by real glyph masks. A test-only fake font/provider is acceptable if it cannot alter the production font selection. Do not obtain the masks by rasterizing a font, screenshotting Canvas, reading candidate atlas storage, or approving a first GPU result.

Use hand-authored small masks containing zero, full coverage and values immediately around relevant byte thresholds. Independently compute every output channel with integer arithmetic:

```text
effective_alpha = floor(color_alpha * mask_coverage / 255)
output = floor((source_channel * effective_alpha
              + destination_channel * (255 - effective_alpha) + 127) / 255)
```

Apply that formula after each draw in original order; retain the opaque target's zero high byte. In particular, rounding the first alpha multiplication or blending all overlaps only once is a different oracle. A separate simple arithmetic author/reviewer can check literal rows without invoking any rendering code. Freeze row bytes, integer reasoning, fixture/source metadata and hashes before execution. An exhaustive 256×256 alpha/coverage scalar check may be useful later, but remains an arithmetic test, not a font golden.

A compact proposed family set, to turn into actual independently reviewed fixtures later:

| Family | What can be established independently |
| --- | --- |
| Coverage and colors | Zero/full/intermediate mask bytes; opaque and translucent source colors; a nonwhite destination; zero source alpha; opaque output high byte and clear. |
| Ordered overlaps | Repeated use of one mask with distinct colors/alpha, coincident and partial overlaps, with a rectangle/image between glyph draws. Atlas reuse must not merge draws or reorder rounding. |
| Signed placement | Explicit positive/negative bearings, empty masks and non-square masks; integer destinations on and beyond all framebuffer edges. |
| Fractional placement | Explicit `f32` operands just below/at/above positive and negative half-integer rounding boundaries. Record bit patterns and original addition order; avoid decimal approximations and Python rounding as the authority. |
| Row shear | Explicit mask offsets, size and row coordinates whose separately reviewed row shifts differ. Check left/right footprint and clipped rows; do not let a reference call candidate placement logic. |
| Clips and fixed scopes | Fractional lower and upper clip edges, zero clips, document offsets, nested fixed escape and restoration, negative viewport offsets. |
| Mask/atlas integrity | Literal identity reuse, distinct masks with identical bytes if identity matters, padding/tail bytes, zero-sized masks, retained source lifetime and rejected out-of-range dimensions/offsets/row spans. |
| Budgets and refusal | Exact cap and cap+1 at the chosen glyph/atlas/row-metadata/draw/work boundaries, with a visible suffix proving whole-frame fallback. Hidden or alpha-zero text must not bypass required structural checks. |

This establishes exact mask sampling/compositing and the specified placement arithmetic on those inputs. It does **not** establish that a particular font's outline generated the right mask. Do not label the population “independent DejaVu goldens.”

## Actual bundled-font population

Prepare a separate fixed inventory before candidate execution. Its expectations are equivalence to the preserved published CPU text path, not independently known font coverage. Preserve the published reference source/binary, dependency locks, toolchain/target, input command lists, font hashes, flags and full reference images before implementation changes. Later CPU/GPU comparisons must consume the identical original display list, clip/offset state and image inputs. If an extraction refactors `Fonts`, retain a comparison against the old path as well as the shared new path; two outputs consuming the same new broken mask are insufficient regression evidence.

Use modest direct `DrawCommand::Text` inputs first, then a small number of real worker HTML snapshots. Freeze the expected commands/face flags and validate the complete captured stream before GPU work. HTML layout `text_y` is currently computed by Eris layout (`visual_y + baseline - size * 0.95`), and must not be guessed as a conventional font baseline. Actual worker command verification remains necessary even if a direct text test passes.

The inventory should cover all three faces; bold+monospace precedence; italic false/true; opaque/translucent colors; kerning-sensitive pairs and repeated glyphs; mixed Unicode scalars; spaces and an independently verified no-outline character; and a character proven absent from the bundled cmap. A possible pair or missing codepoint is only a candidate until its metadata is checked. Do not assume that a familiar pair has nonzero kerning, that a proposed space has no outline, or that an arbitrary high Unicode scalar is missing. A bounded data-only inspection of pinned cmap/hmtx/kern/outline metadata can establish those prerequisites separately from rasterization.

Include two original sizes sharing a quantized mask key but different metrics, sizes straddling a `1/8` quantization threshold, signed half-pixel pen/y placements, clipped descenders/negative bearings, fixed descendants, overlapping colored text, and glyph/rectangle/image interleaving. Compare cold-cache and repeated warm-cache output. Cache retention/eviction is an additional bounded private resource test rather than a reason to create thousands of host image fixtures.

Retain exact full-frame mismatches, raw command/mask metadata when available, hashes and exhaustion/refusal status. A diagnostic difference image or cropped match cannot replace the full comparison. Output from the preserved CPU reference may be frozen as a **reference-derived baseline**; never relabel it an independent mathematical golden or overwrite it with a candidate result. A common `ab_glyph` defect may survive CPU/GPU differential testing, and that limitation must remain explicit.

Font identity bindings at this checkpoint:

| Asset | Bytes | SHA-256 |
| --- | ---: | --- |
| `assets/DejaVuSans.ttf` | 759676 | `58568c88a01b80dfc056daca3eeafd434a0380df1a48dda53f0de0d716109bbd` |
| `assets/DejaVuSans-Bold.ttf` | 708876 | `7c7aef63328a765586cda41bf9d2c227ff1ad82816326dae4d7fe2fd1b7f613a` |
| `assets/DejaVuSansMono.ttf` | 343096 | `9d9bfebceb1c3f6f4ad383ded568a6926086208f43f7f92f92f7e93a1383fa38` |
| `assets/FONTS-LICENSE.txt` | 8816 | `7a083b136e64d064794c3419751e5c7dd10d2f64c108fe5ba161eae5e5958a93` |

Keep the bundled font license with any future redistributed font bytes. The current root `Cargo.lock` SHA-256 is `fc2d595602095e8a4333975b5f2e8ba748a1b82fbdc84505e64abcbd2b92a0de`; it pins `ab_glyph 0.2.32` (crate checksum `01c0457472c38ea5bd1c3b5ada5e368271cb550be7a4ca4a0b4634e9913f6cc2`) and `ab_glyph_rasterizer 0.1.10` (checksum `366ffbaa4442f4684d91e2cd7c5ea7c4ed8add41959a31447066e279e432b618`). Freeze the probe lock and the complete resolved font-parser dependency chain too; a font filename alone is not an identity.

## Prerequisites and review gates

`Fonts::bitmap`, the face/scale accessors and `GlyphBitmap` are currently private. A narrow bounded shared mask/placement API will need design and static review before code. It should expose or borrow immutable masks without modifying the CPU algorithm, losing original bearings or holding mutable cache borrows across external work. Do not silently reimplement font rasterization in the shader. Keep CPU-generated mask coverage and GPU compositing as separate claims.

Preflight text byte/scalar counts, distinct mask records, mask dimensions/total storage, row-placement metadata, ordered draws, dispatch work and staging/readback buffers before the associated growth. Bound hidden text and long zero-advance strings as well as visible ink. Existing cache bounds do not establish the new atlas budget: the cache has up to 2048 small masks and larger masks are transient, while the probe retains plans and GPU buffers separately. `Fonts::bitmap` currently allocates/rasterizes before `Canvas::text` charges its pixel loop; a new atlas planner must not claim allocation precharging by merely reusing that later charge. Decide a bounded preflight interface explicitly, preserving existing quotas.

An initial bridge cap can be stricter than Canvas and refuse the whole frame transparently. It must not truncate text or omit an unsupported case to stay under the cap. Keep existing 320×240/command/scope/GPU-work/storage caps, original 30 probe cases and the complete 16-case bridge inventory intact. Native browser windows still exceed that viewport cap. Rounded rectangles and opacity groups remain explicit fallback until separately implemented; no group-opacity claim follows from glyph alpha support.

Likely affected paths, for a later separately authorized implementation: narrow shared access in `src/graphics.rs`; probe `src/browser_adapter.rs`, planner `src/lib.rs`, shader/GPU bindings in `src/gpu.rs` and a mask shader or explicitly reviewed image-shader reuse; new literal-mask fixture/private-test modules; `src/bin/browser-check.rs` plus `browser_protocol.py` for bounded mask/font identities and text admission; and the existing fixture/evidence documentation. No unrelated layout, shaping, font replacement or native presenter changes are needed for the first offscreen mask-compositing step.

Freeze independent fixture definitions and any published-CPU reference inventory before implementation; peer-review arithmetic and source/metadata assumptions; retain initial failures; run synthetic tests before authorized real font/worker/GPU checks. Report independent-mask and actual-font comparison counts separately. Success would demonstrate the bounded new offscreen path, not full browser GPU rendering, font conformance, native presentation, security or a performance improvement.
