# Native color box decorations

The Native renderer admits `BoxDecoration` with `BoxPaint::Color`, including elliptical corners, translucent fills, and borders represented by an outer shape and a cutout. This extends the [background box selection checkpoint](background-boxes.md). Completed source4 checks and the separately bound source3 GPU observations are recorded below.

The CPU prepares coverage masks. The existing custom Vulkan glyph and opacity-group kernels apply the supplied color and composite the masks in command order. This path does not upload a completed CPU frame or evaluate ellipses in a new shader. Ordinary image, tiled image, gradient, text, and scalar rounded-rectangle paths keep their existing behavior.

Each pixel uses the software painter’s sixteen sample positions. A sample contributes only when it is inside the outer shape and outside the cutout; coverage is `floor(255 * hits / 16)`. Both tests use the same sample. Subtracting or multiplying separately quantized masks would give different fractional-border results. Supplied radii remain used radii: a narrow inner shape is not normalized again.

Geometry follows the existing document, fixed-coordinate, and rectangular clip scopes. Translation occurs once before coverage preparation. Integer pixel origins determine the final clip crop. A mask wider than 1,024 pixels is split into at most two strips; both retain the original shape and sample in absolute framebuffer coordinates. The split introduces no new edge or sampling phase. Source alpha and nested opacity use the existing integer alpha and RGBA16 group-composition paths.

`BoxPaint::Image` and `BoxPaint::LinearGradient` still reject the whole Native scene before materialization, including offscreen commands, missing resources, and commands inside zero-opacity scopes. Primitive and text-aware Probe planners also retain their existing `BoxDecoration` refusal. Invalid geometry, allocation failures, and shared-budget refusals select whole-scene software rendering. Worker tag 13 and its key/gradient allowances are unchanged; Color masks acquire no resources.

## Admission and storage

Mask preparation shares the existing limits with fonts, scalar rounded masks, and image sources: 256 source entries, 256 lowered commands, 262,144 mask coverage bytes, 65,536 row entries, four million padded GPU invocations, and the Native profile’s 16 MiB GPU-buffer allowance. Native viewport bounds remain 1,280 by 1,024. A mostly empty or fully covered shape still needs its admitted mask window; sparse compression and interior-rectangle extraction are outside this increment. A 1,280 by 205 mask alone exceeds the coverage limit.

The adapter preserves the CPU work allowance `clamp(viewport_area * 16, 1,000,000, 32,000,000)`. For a nonempty candidate window, let `P` be candidate pixels, `S` be one shape or two with a cutout, `A` be partially covered candidates, `M` be stored coverage cells, and `R` be rows across all strips. It prepays `8 + P*S` before counting partial candidates, then `P*S + 16*A*S + P + M + R` before materialization. These are policy work units, not instruction measurements. Empty candidate windows pay no mask work after validation; transparent or suppressed Color commands conservatively retain preparation charges.

Complete partition storage, source slots, and extra lowered operations are admitted through the shared ledger before either strip allocates or samples coverage. A failed stage publishes no partial scene. Preparation accounting includes simultaneous owned coverage bytes, row origins, packed input, reservation tables, and plan metadata; it does not measure allocator or driver overhead. Combined final core limits remain checked before GPU submission.

## Independent checks

Ten fixtures were frozen before candidate execution: an ellipse, a correlated fractional ring, an unnormalized truncated ellipse, document/fixed restoration, source alpha, nested opacity, an identical cutout, a fractional caller clip, a two-strip boundary, and a Page-derived translucent rounded CSS border over blue.

The checker first compares unchanged Canvas output and an independent packed-plan interpreter with literal RGB arrays. Native refusal is an error. A device-free probe unit test runs all ten checks during ordinary probe testing. The dedicated `eris-vulkan-native-box-check` then uses the existing `surface_gpu` harness to compare every active pixel and alpha in BGRA8 and RGBA8. The frozen inventory contains 2,756 pixels, so a completed adapter run must report ten fixtures, twenty converted frames, and 22,048 compared bytes. References are supplied only to readback comparison, never to GPU encoding.

This is offscreen validation. It establishes neither acquired-window presentation nor a performance result, and the fixtures do not establish complete CSS compatibility.

## Recorded observations

The source3 checker completed all ten fixtures on three Vulkan adapters. Each run compared twenty converted frames and 22,048 active bytes exactly in both formats, for sixty frames and 66,144 bytes overall. The adapter inventory and all three completion logs agree:

| Adapter | Driver | Observed result |
| --- | --- | --- |
| NVIDIA GeForce RTX 4070 SUPER | NVIDIA 595.99.02 | 20 frames, 22,048 bytes exact |
| AMD Ryzen 7 7800X3D integrated graphics | RADV, Mesa 26.2.3 | 20 frames, 22,048 bytes exact |
| llvmpipe, LLVM 21.1.8, 256 bits | Mesa 26.2.3 software Vulkan | 20 frames, 22,048 bytes exact |

These results bind the 655-input `compiled-source-3.json` inventory, SHA-256 `4f8f88d5001412848948de816f20be05398307567ab0374159f800fa7bb370a0`, and the immutable 36,084,600-byte probe, SHA-256 `84726f40d158cede4c4e84b2a511c14b0b76ec6488b3ca179fde5ab9552a9eaf`. Its build and adapter receipts retain the actual commands and hashed logs. The binary was not stripped. These are source3 observations; they are not relabeled as a later final-source run.

Source4 contains 656 inputs, bound by SHA-256 `bab801002cc90c1137cce8e95085ed8944d55ada9c652c1037bbe2e7e4886805`. Its complete Rust 1.88 Native suite passed 2,865 tests; the Rust 1.98 default suite passed 2,598. Each log contains 35 result groups with zero failures or ignored tests. An independent source3-to4 review verifies that only two added prepass tests, their `cfg(test)` registration, and the exact CI test roster changed; production, helper, core, shader, and probe inputs remain unchanged. This explains the retained source3 GPU evidence without relabeling its build or executions.

Strict all-target Clippy passed for Native on Rust 1.88 and 1.98, default on Rust 1.98, and the browser-bridge probe on Rust 1.98, with warnings denied. Root and probe formatting checks passed. All six device-free probe tests passed, including the group that runs all ten frozen Canvas/literal/packed-Native fixtures.

The source4 browser passed all ninety reference pairs with zero differing pixels or channel error. The unchanged manifest and fixtures were retained, and all 180 candidate PNGs matched the published background-box checkpoint in both decoded pixels and encoded bytes. The immutable browser is 62,642,920 bytes, SHA-256 `a77cec276a89a36f70f444b66c44e424deb610bd31a5c472db54742b52b312af`, joined to the source4 build and separate strip receipt.

The [evidence summary](evidence/native-box-color/summary.json) and [retained validation archive](evidence/native-box-color/validation.tar.gz) bind the source inventories, frozen literals, source reviews, test-only refinements, browser/probe identities, raw command logs, and reference inputs. Historical holds and earlier-source observations remain separately identified.
