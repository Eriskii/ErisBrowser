# Proposed reusable raster-core extraction

Read-only map, 2026-10-02. This is preparation, not an implementation or native GPU support claim. No Cargo, compiler, worker, Canvas, font, GPU or dependency-resolution command was run. Current source and manifests remain untouched.

## Smallest dependency direction

Introduce a path crate such as `crates/raster-core` (`eris-raster-core`, Rust 1.88, edition 2024, unsafe forbidden). Its default library is the bounded planner with no external dependency. An optional `gpu` feature enables exact `wgpu = 30.0.1`, defaults disabled, `std/vulkan/wgsl`. Neither feature depends on Eris, fonts, workers, fixtures, windowing or Python.

Initially: `probe -> core`, and `probe --features browser-bridge -> eris-browser`; leave the browser package untouched. Later: `eris-browser -> core` plus its own browser/font adapter; the probe can reexport that adapter through its existing optional Eris dependency. Never add `eris-browser -> probe`: the probe's existing optional dependency on Eris would create the cycle being removed.

## Concrete ownership map

| Current file/API | Destination and ownership |
|---|---|
| `src/lib.rs:3–707`: all `MAX_*`, `PARAM_STRIDE`, `Result`, `Rect`, `Command`, `rect`, `Frame`, `SourceImage`, `SourceMask`, `DrawKind`, `Draw`, `Plan`, `plan`, `plan_with_images`, `plan_with_masks`; private packing/validation/coverage/LUT helpers | Core planner owner. Move algorithms together; retain private fields and constructors. `Plan` owns only validated draws, parameters and input bytes; no snapshot/font lifetime crosses the API. |
| `src/scope.rs`: typed clip/fixed stack and `CoordinateState::{new,clip,offset,apply,finish}` | Core planner owner. Preserve original f32 operation order, validation-before-culling, exact errors and all caps. |
| `src/{rect,image,glyph}.wgsl` | Core GPU owner. Move exact bytes once; preserve entry points, bindings, parameter offsets, pass order and integer blend semantics. |
| `src/gpu.rs:52–236`: pipeline storage, GPU buffer creation/upload/ordered encoding; `:291–408` pipeline creation | Reusable core GPU portion, after the expected-pixel split below. |
| `src/gpu.rs:238–290,409–424`: enumeration, adapter-line printing, selected adapter/device setup, batch count, callback; expected-pixel comparison at `:207–233` | Probe harness owner. Preserve public `gpu::run_plans` signature and output protocol. Deadlines/error scopes must have one explicitly documented owner after split. |
| `src/{fixtures,image_fixtures,alpha_fixtures,browser_fixtures}.rs`, `src/glyph_fixtures/*`, `worker-text-fixtures/*`, and all frozen evidence/oracle assets | Probe only. No fixture constructors, expected RGB or font inventories in production core. |
| `src/browser_adapter.rs` and `browser_adapter/text.rs` plus their tests | Currently probe adapter owner; later move together to an Eris module (for example `graphics::raster_bridge`) depending on core. `FallbackKind/Reason`, `BridgeStats/Plan`, `TextBridgeStats/FontBridgePlan`, and `plan_{snapshot,display_list}{,_with_fonts}` belong here, not in core. They use Eris `DrawCommand`, `RasterImage`, `ImageStore`, `Snapshot`, `Fonts` and `text_masks::Session`. |
| root `src/graphics/text_masks.rs`, bundled fonts, original CPU Canvas | Stay in Eris. Font cache identity, text/scalar/cold/coverage/occurrence allowances and dependency scratch boundary do not move into the primitive renderer. |
| `src/main.rs`, `src/bin/{browser-check,glyph-check,worker-text-check}.rs`, `src/bin/checker_support/mod.rs` | Probe binaries only. The shared support module owns EWB1, file URLs, bounded procfs checks, capture deadlines and exact GPU grant. It depends on Eris worker types and must never become a renderer dependency. |
| Python supervisors, runners, protocol parsers and tests | Probe tooling only; preserve capture-before-driver grant, outer deadlines, cleanup ownership, raw observations and exact output formats. |
| root `src/presenter*`, `src/browser.rs`, worker launcher/channel | Future native owner, unchanged in extraction. Core supplies no surface lifecycle, scene/chrome composition or clean-exec policy by itself. |

## Preserve old imports without duplicate implementations

The probe `lib.rs` becomes a facade: explicitly reexport the existing planner constants/types/functions from core; retain public fixture modules and `gpu` comparison wrapper at their existing paths. Thus callers keep `eris_vulkan_raster_prototype::{Plan, Frame, Command, plan_with_masks, ...}` and existing `gpu::run_plans` calls. Use one underlying type, not wrappers or conversions. Keep exact error strings and validation precedence, including the old image-entry check before combined image/mask count.

Only two production privacy seams need new core APIs:

* Both adapter variants call current private `source_pixels` before callback/raster work. Expose a bounded pure validator with that exact behavior (for example `validate_source_images(&[SourceImage]) -> Result<usize>` returning checked pixel count), and use it in the core planner and adapter. The probe can keep a crate-private `source_pixels` alias temporarily to minimize diffs. Do not reproduce or weaken the all-supplied-source validation.
* The font adapter imports `scope::CoordinateState`. Expose the state type and its six existing methods through a narrow public core module, leaving fields/stack private. The probe's crate-private `scope` module can reexport it initially. Do not make `Rect` internals, `Draw` fields or arbitrary `Plan` construction public to satisfy the move.

`Result<T> = Result<T,String>` is the current planner API; replacing it with a new error model would expand this checkpoint. Adapter typed refusal/category/original-index behavior remains separate and unchanged.

## GPU executor split: no expected pixels in production

Current `raster` receives `expected`, allocates both output and readback, dispatches, waits, maps, compares and destroys. Current `run_plans` also prints inventory, creates a fresh instance/device and drives the fixture batch. Reexporting that whole module as production would preserve the wrong boundary.

A small reusable interface should accept a caller-owned `Device`/`Queue` and immutable `Plan`, prepare pipelines/buffers, and encode ordered passes into an encoder. Return an owned frame resource with a read-only output-buffer accessor, dimensions and allocation accounting; retain parameter/input resources through submission completion. It must not accept expected colors, paint a CPU target, create a native surface, enumerate/print adapters, invoke fixture callbacks, or own worker grants. Shader creation can remain conditional on glyph use exactly as now. The output keeps `STORAGE | COPY_SRC` and no `COPY_DST` usage; only original colors, masks, row/LUT words and parameters are uploaded.

Keep the first extraction's planner accounting **unchanged**: two target-sized buffers plus parameters/input, <=1 MiB; padded work <=4M. A reusable encode operation may reserve the second target allowance without allocating it, while the probe explicitly allocates/records its readback buffer. Distinguish actual owned bytes from the unchanged allowance: do not claim equality for a no-readback caller. The probe path must still assert its actual sum equals `Plan::gpu_buffer_bytes()`. Do not remove the second-buffer reserve to admit larger plans during refactoring.

The probe wrapper allocates readback, adds the copy, submits, waits/maps under the existing deadlines, compares every `u32` against expected pixels, unmaps/retires resources, drains validation/OOM/internal scopes, and only then emits existing callbacks. Preserve the 5-second API and 20-second run deadlines, 1..30 selected batch and 1..16 adapter limits, and all-error abort behavior. Core encode must retain a bounded cancellation/deadline check before each reached pass (for example a supplied check callback); do not replace the old per-draw checks with an unbounded device operation. Error-scope ownership and resources after a failed poll need explicit cleanup tests, not implicit lifetime assumptions.

Native composition/conversion/presentation is a later increment. The reusable output is still packed high-byte-zero RGB; it is not yet an RGBA/BGRA surface. Nothing in this extraction solves rounded chrome, opacity groups, page/chrome budgets, paging, stale-frame retirement or the native release acknowledgment.

## Tests and fixture privacy

`src/tests.rs` accesses `Plan` and `Draw` private fields and calls `sample_index`; `glyph_tests.rs` also calls private `mask_storage`. Merely moving them to probe integration tests would not compile. Do not expose those internals or silently drop tests.

Move their **execution ownership** to core unit tests, preserving test bodies/names/expectations. For the smallest first patch, core test-only `#[path]` modules can include the unchanged repository-owned probe `tests.rs`, `glyph_tests.rs`, and pure `fixtures/image_fixtures/alpha_fixtures` modules. Their `crate::` references then resolve to the core API; production core builds include none of the fixtures. This is an explicit temporary test-source dependency, not a Cargo dependency from core back to probe. A later mechanical move can give core-owned test paths. Include every path in source/archive manifests.

The probe keeps adapter tests, binary/helper tests and `tests/glyph_contracts.rs`; the last uses the stable facade and exact frozen literal inventory. The pending-future timeout test follows the deadline owner. **Testing a crate does not run dependency unit tests**: add an explicit core default/GPU test invocation to validation/CI, preserving an inventory of every old test name rather than relying on the old probe total. Expected aggregate counts may redistribute, not disappear. Validate the core with no Eris feature/dependency to catch accidental browser coupling.

## Cargo and lock implications

Current root package is edition 2024/MSRV 1.88, defaults empty. Linux `vulkan-presenter` enables optional exact wgpu 30.0.1 with only `std,vulkan` plus `close_fds`; it is CPU-frame upload presentation. The standalone probe has its own `[workspace]` and lockfile, defaults empty, but wgpu is unconditional with `std,vulkan,wgsl`; `browser-bridge` enables optional root package with default features disabled.

The read lockfiles contain 316 root and 327 probe package records (not claims about enabled compile graphs). Both pin wgpu/naga 30.0.1 to the same registry checksums. Their selected transitive graphs differ; do not use package counts alone as equivalence proof.

For stage one keep the probe's workspace/lock ownership and executable names. Add a path dependency to core, enable core/GPU for the probe, and retain the probe's direct exact wgpu dependency while its wrapper owns device/readback types. This uses the same version identity; check resolved features when authorized. Avoid a workspace conversion or resolver-wide refresh. Core can be an independent path package; choose a deliberate standalone test/workspace arrangement and track any core lockfile rather than accidentally creating one during validation.

Stage one's root Cargo.toml/Cargo.lock should remain byte-identical. When moving the adapter into Eris, add an optional planner-only root feature/dependency and have probe/browser-bridge select that feature explicitly. The adapter uses no GPU and must not initialize a driver. Only a later native GPU feature enables core/GPU. Enabling WGSL in the root build is a deliberate feature-graph change even if the wgpu version is unchanged. Preserve the existing Linux clean-exec/FD safeguards in `src/main.rs` and `src/worker/channel.rs`; currently they are keyed to `vulkan-presenter`, so a future driver feature must imply that safeguard or update both conditions explicitly.

## Suggested bounded stages and owners

1. **Planner extraction:** one owner moves planner/scope and adds facade/API seams; manifest owner adds only the local core dependency. No native call sites. Freeze old/new source and all shader/fixture bytes before validation.
2. **Executor split:** GPU owner extracts pipeline/encoding/owned output; harness owner retains exact `gpu::run_plans` adapter printing/readback/comparison. Preserve all legacy 30 literal, 16 browser, 12 glyph, font differential and seven worker-text input/protocol contracts. Run pure test inventory first, then existing controlled CPU/offscreen checks when authorized.
3. **Browser adapter relocation:** root graphics owner moves both adapter variants together, exposes planner-only feature, and probe reexports the same public adapter paths. Keep font-library behavior and budgets fixed. Compare complete admission/refusal/stats, ordered parameter/input bytes and original failure observations.
4. **Native integration separately:** scene construction, rounded chrome, broader reviewed frame policy, presentation conversion and single graphics-owner lifecycle need their own frozen contracts. Extraction alone changes no native renderer selection.

No build or execution was used to certify this proposal. Ownership, API names and the core standalone-workspace choice require implementation-time agreement; the exact existing semantics and caps above are constraints, not open tuning parameters.

## Read input bindings

SHA-256 values below bind this read-only map. All paths are relative to the repository root.

- `Cargo.toml`: `015958175457d712165b649803198aa8aa5bba2f35e19dd49814def7308b03ec`
- `Cargo.lock`: `fc2d595602095e8a4333975b5f2e8ba748a1b82fbdc84505e64abcbd2b92a0de`
- `tools/vulkan-raster-probe/Cargo.toml`: `a5bdf9aa44c9dc1d8db466f9d73578c79dc220495b583edc94abada391292303`
- `tools/vulkan-raster-probe/Cargo.lock`: `c3ec1b8cecc74528ecb7e183cc782933124b879a7201e1978bce18ee046dd9d3`
- `tools/vulkan-raster-probe/src/lib.rs`: `e134b9c963e7a120465a03a978325aff58f5857a8aeb2e77470662e903f3fc8b`
- `tools/vulkan-raster-probe/src/scope.rs`: `258c1f8e2f9c36e46d0a0256cad3bcffb713325788540e572e66188d1db10efe`
- `tools/vulkan-raster-probe/src/gpu.rs`: `f60141761316a3170a573937b0219e95e63181a415cf734a0bd7f720d040d6b2`
- `tools/vulkan-raster-probe/src/rect.wgsl`: `b0f67e88f3b5630cb98c8406ee20df6e8890b3ca42410451f105f64fb2a68afa`
- `tools/vulkan-raster-probe/src/image.wgsl`: `ede39857bf369d5a33f5503e7ae96bd04bd5ca06a11937f28fa97793011b3446`
- `tools/vulkan-raster-probe/src/glyph.wgsl`: `a045be94313471d703e7a797f36d1956785b43e69129851d9eb8a99ad9dc86b7`
- `tools/vulkan-raster-probe/src/browser_adapter.rs`: `ef74f617036456661c0ff6ede702d6b721b7615031cb9085dc2d9b645fef9fa3`
- `tools/vulkan-raster-probe/src/browser_adapter/text.rs`: `d2db12bdef76b85f20dfb4b5586a12de44d8410fb0335fb9481b2251fe01947a`
- `tools/vulkan-raster-probe/src/tests.rs`: `0453f0431e7f352015876ee801c28bfcda49d9be6c5d4c5605144629534bef61`
- `tools/vulkan-raster-probe/src/glyph_tests.rs`: `44c3339ff5df30becefbc8e6e434322a4898abc98e971f01b4c4e2e6ad3accbd`
- `tools/vulkan-raster-probe/tests/glyph_contracts.rs`: `1d0b10a0b603f603fbb8510fdd703286b99e2c473d440e165b92d893008f1944`
- `tools/vulkan-raster-probe/src/bin/checker_support/mod.rs`: `3716cf3c921ef02fb5fd955bbd07599f7143dfcf40e95a2d30dc59c0b45b7cdf`
- `src/graphics/text_masks.rs`: `d47609697dbdf8b6cdcf310aa28ec36a8873c0339d30fc20a0e92fa036a1b4e6`
- `src/presenter.rs`: `f9bd2a4d817fe0ded8abd695296abf5b4151dbe13a5a00313a1f981fcb0d3094`
- `src/presenter/vulkan.rs`: `d3de9ac6382a8776448c48ebe571debe70e4eab85878429070f95fbaa40df68b`
- `src/worker/channel.rs`: `fa5e11334ea0cace8ac7e5b43c770f77dcaa7ccbf245a71365ff3540d9226d44`
- `src/main.rs`: `81e98514b9b51762d7cd20d0eda1c2e30b85ebec6b38f6d01b0a6ff2ebee2dbf`
