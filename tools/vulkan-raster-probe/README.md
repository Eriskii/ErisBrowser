# Custom Vulkan rectangle, image and source-alpha compute experiment

This standalone probe executes custom WGSL for unrounded rectangles and nearest-neighbor images, including source alpha over an opaque RGB target. It is experimental groundwork for the browser backend, with its own dependency graph. It has no browser integration, native surface, window, compositor capture or performance claim. The CPU-only tests do not enumerate adapters.

The [alpha host record](evidence/host-alpha.json) records all **30 fixtures passing on each of the NVIDIA, AMD and software Vulkan adapters**: **921,620 exact compared bytes per adapter, 2,764,860 total**. All four processes, including enumeration, exited normally with empty stderr. It binds the actual source, binary, loader, adapter identities and raw output.

The earlier [image host record](evidence/host-images.json) retains its 21 opaque fixtures and **2,763,816 exact compared bytes** across the three adapters. Its source, binary and raw results remain attributed to that checkpoint. The original seven rectangle and fourteen image fixture definitions, expected pixels and protocol tuples remain unchanged in the current 30-case run.

The earlier [rectangle host record](evidence/host-raster.json) preserves seven exact offscreen pixel comparisons on each of the NVIDIA, AMD and software Vulkan adapters: 1,839,156 compared bytes in total. All processes exited normally with empty stderr. That record binds the rectangle checkpoint's source and binary; its results and raw logs remain unchanged. It does not establish image rendering on the GPU.

The GPU executes our WGSL and writes packed `0x00RRGGBB` pixels into a storage buffer. It receives immutable draw metadata, original decoded source colors and separable source-index tables. The output buffer has STORAGE and COPY_SRC usage, with no COPY_DST. Even the nonzero background clear is a GPU rectangle dispatch. Packed buffer readback uses four-byte pixels and buffer-copy alignment; texture 256-byte row alignment does not apply.

The CPU planner performs a bounded display-list walk, f32 translation/intersection in Canvas order, and integer coverage conversion. Rectangle starts use floor, clip starts use ceil, and ends use ceil after reconstructing the f32 visible extent. Translucent rectangles also enforce both half-open integer-origin clip edges, matching `Canvas::blend`; opaque rectangles retain the existing fast-path coverage. Typed clip/fixed scopes preserve caller-clip and viewport-offset resets, nested fixed restoration, and document offsets. No CSS/layout/DOM processing is introduced. Rounded geometry, text and group opacity reject the entire stream before device work; nonfinite/over-limit inputs and mismatched/unclosed scopes also reject. Finite nonpositive destination extents are no-ops after input validation.

Each shader applies source-over separately for each ordered draw. A channel is `(source * alpha + destination * (255 - alpha) + 127) / 255` using integer division, preserving Canvas's encoded-byte rounding. Target words stay `0x00RRGGBB`. Alpha 255 replaces the pixel; alpha zero leaves it untouched. Valid alpha-zero rectangles are omitted only after geometry/radius validation, while visible images keep their dispatch and source/table storage even when samples are transparent. Clear is unconditional and always opaque. This does not implement transparent intermediate targets, group opacity or color-space conversion.

Image sampling uses the original translated destination rectangle, including when clipping removes its start. Each axis table preserves Canvas's f32 subtraction, division, multiplication, clamp and truncation order; there is no half-pixel adjustment or GPU floating-point sampler. Both lower and upper integer-origin clip bounds are enforced. The shader reads two table entries and one original source color, then blends into the output. This is exact sampling within the selected CPU behavior, not complete image/CSS/color-space conformance.

Sources require nonzero dimensions and an exact RGBA byte length; every source alpha byte is supported. Invalid sources and source IDs reject even when unused, hidden or attached to an empty draw. This is deliberately stricter than Canvas's handling of some malformed/empty buffers. There is no CPU alpha scan or transparent-image optimization. If any image dispatch is visible, all supplied sources are packed once as `0xAARRGGBB`; repeated source IDs reuse those words. Plans own immutable input and parameter bytes. Rectangle records remain 32 bytes, image records are 64 bytes, and both occupy unchanged 256-byte slots. A plan with no image dispatch allocates no image input buffer.

One 8×8-workgroup compute dispatch handles each planned draw in a separate compute pass, in display-list order. Each invocation writes at most one pixel and has explicit extent/canvas/storage checks; image reads additionally check table and source ranges. There is no shader loop over commands or overlapping writers within one dispatch. Wgpu tracks storage usage and emits inter-dispatch/pass barriers; readback follows all passes in the same submission. Metadata and image input buffers remain immutable throughout that submission.

## Explicit experiment limits

- 320×240 nonzero viewport; 256 input commands; 32 combined clip/fixed scopes.
- Finite authored coordinates/offsets bounded to ±1,000,000; resulting translated coordinates stay bounded by their sum.
- At most 257 draws including clear; 4,000,000 rounded shader invocations per plan (including idle lanes).
- Output + readback + metadata + optional source/table arena share the existing 1 MiB explicit GPU-buffer ceiling. Rectangle-only plans retain their original allocation counts. A 320×240 source and destination with one image draw uses 924,352 explicit bytes, including its two axis tables and the GPU clear.
- At most 256 supplied source entries and 1 MiB aggregate declared RGBA bytes, checked before source-area scans or allocations. A separate structural ceiling permits at most 143,360 axis-table entries; tighter invocation/GPU limits still apply. All supplied sources are validated, including unused ones.
- The GPU sum excludes CPU inputs/expectations/plans, opaque driver/device/pipeline allocations and wgpu upload staging. These separate bounds do not prove total process/device memory or general allocator fallibility.
- One submission at a time, map completion/readback before the next fixture; no pending frame queue or mailbox.
- 5-second capped future/poll waits within a 20-second application deadline; the runner gives each process 25 seconds by default, captures at most 256 KiB combined output, and gives reaping 2 seconds after process-group SIGKILL. It uses `waitid(WNOWAIT)` before cleanup to retain leader identity and never spawns another adapter after unresolved cleanup.

Synchronous driver calls, future polling internals, device destruction, process creation and kernel-uninterruptible teardown are **not** proven to finish within those deadlines. Failure/timeout remains visible. The supervisor is Linux-specific, does not change driver settings, and requires a new output directory. It freezes exact adapter identity and checks all 30 expected PASS records plus COMPLETE; exit zero alone is insufficient. Actual execution requires `--allow-experimental-gpu`.

`Plan` and `Draw` storage is private with immutable accessors. Planning copies the source pixels into owned storage, so later caller mutations cannot alter a validated submission. There is no browser fallback or worker security integration here, and no supported external command ingestion API.

## Independent fixtures

The seven original rectangle cases retain their commands, expected pixels and protocol tuples: overlapping order, fractional clips, fixed restoration, out-of-bounds/empty rectangles, dispatch edges, ordered full-frame overwrites and GPU clear.

The independently [frozen image oracle](evidence/image-fixture-oracle.json) adds 14 cases: scaling, original-origin sampling after clipping, negative/fractional geometry, exact/ULP/subnormal edges, source reuse and draw order, nested fixed restoration, dispatch tails and empty images. Expected pixels are literal grids or a direct quadrant/border predicate, never candidate-produced pixels. Nine scalar table cases cover clamping and rounding separately. Malformed inputs, hidden invalid content and exact/over-limit plans retain explicit expectations; the over-depth case has matching pops so an unclosed scope cannot mask the depth check.

The earlier [image protocol arithmetic](evidence/image-protocol.json) requires 21 fixtures and 921,272 compared bytes per adapter. The image host record verifies those historical counts. Its [freeze record](evidence/image-fixture-freeze.json) preserves the draft, independent review and pre-implementation correction to the depth test.

The [independently frozen alpha oracle](evidence/alpha-fixture-oracle.json) adds nine tiny literal targets: alpha 0/1/127/128/254/255 over two backgrounds, mixed draw order, repeated low-alpha rounding, fractional clips, original-origin image sampling, fixed restoration, hidden transparent RGB and opaque clear through an empty clip. Its 87 pixels add 348 compared bytes per adapter. The [alpha protocol](evidence/alpha-protocol.json) fixes all 30 tuples before execution. Original fixture pixels and all command/source/scope/work/buffer limits remain unchanged; accepting valid source alpha is an explicit support-policy change.

## Reproduction

Pinned wgpu 30.0.1, defaults disabled, exactly `std,vulkan,wgsl`, edition 2024, minimum Rust 1.88. No additional direct dependencies or toolchain installs. `Cargo.lock` copies the published isolated transfer probe dependency resolution; only this root package name changes.

```
cargo test --offline --locked
cargo clippy --offline --locked --all-targets -- -D warnings
cargo build --offline --locked --bins
python3 -m unittest discover -s . -p 'test_*.py' -q
```

Python tests run only protocol checks and bounded fake CPU child processes. The alpha checkpoint passes all **25 Rust tests**, formatting, strict all-target Clippy and builds on both **Rust 1.88.0 and 1.98.0**, plus **ten Python tests**. Both shaders pass offline Naga 30.0.1 validation. The first alpha candidate failed to compile a test helper because of integer-mask type inference; explicit `u32` types on three lines resolved it. Production code, fixture expectations and limits were unchanged by that correction. Earlier image-checkpoint Clippy/linker failures remain in their original evidence.

To run the optional GPU check with an installed loader visible (caller-selected directory):

```
python3 run_host.py --allow-experimental-gpu \
  --binary target/debug/eris-vulkan-raster-prototype \
  --output-dir host-run-1 --loader-directory /path/to/installed/loader
```

Do not treat build/unit/WGSL-validation success as evidence of GPU pixel equality. Retain all raw output and failure records. No timing observations from this prototype are a performance result.

## Provenance and primary sources

Planner f32 bounds and fixed scopes were derived from the existing `src/graphics.rs` Canvas implementation. The recorded preparation hash binds that input. Safe async/request/map scaffolding and bounded supervisor conventions were adapted from `tools/vulkan-probe`; this supervisor adds non-reaping exit observation and exact raster result validation. No source files in the transfer probe were edited.

- WGSL specification: https://www.w3.org/TR/WGSL/ (compute entry points, storage buffers, integer arithmetic and bounds).
- Pinned official wgpu 30.0.1 API source in the Cargo cache: `api/compute_pipeline.rs`, `api/pipeline_layout.rs`, `api/compute_pass.rs`, `api/buffer.rs`.
- Pinned wgpu-core 30.0.1 `src/command/compute.rs`, `State::flush_bindings`: per-dispatch usage scopes and `CommandEncoder::drain_barriers` for conflicting storage usage. This is the synchronization mechanism; no unsafe raw Vulkan escape is used.
- Linux waitid ownership behavior: https://man7.org/linux/man-pages/man2/waitid.2.html ; process-group signals: https://man7.org/linux/man-pages/man2/kill.2.html .

All crate Rust sources forbid unsafe code; transitive wgpu/Vulkan driver internals are outside that claim. Exact source/lock/binary/provenance hashes and offline outcomes are recorded separately in the [rectangle](evidence/host-raster.json), [image](evidence/host-images.json) and [alpha](evidence/host-alpha.json) evidence. Earlier native-surface experiment failures remain unchanged; this offscreen prototype is no new compositor evidence. Browser integration, text, rounded coverage, group opacity, color conversion, production security and performance comparisons remain open.
