# Custom Vulkan rectangle compute experiment

This standalone probe executes custom WGSL rectangle rasterization through Vulkan. It is experimental groundwork for the browser backend, with its own dependency graph. It has no browser integration, native surface, window, compositor capture or performance claim. The CPU-only tests do not enumerate adapters.

The [host record](evidence/host-raster.json) preserves seven exact offscreen pixel comparisons on each of the NVIDIA, AMD and software Vulkan adapters: 1,839,156 compared bytes in total. All processes exited normally with empty stderr. The tested Rust sources, manifest, lockfile and supervisor are byte-identical to this published copy.

The GPU executes our WGSL and writes packed `0x00RRGGBB` pixels into a storage buffer. It does **not** receive a CPU-painted target. The only uploaded bytes are immutable 32-byte rectangle/color/viewport records in 256-byte aligned slots. The output buffer has STORAGE and COPY_SRC usage, with no COPY_DST. Even the nonzero background clear is a GPU rectangle dispatch. Packed buffer readback uses four-byte pixels and buffer-copy alignment; texture 256-byte row alignment does not apply.

The CPU planner performs a bounded display-list walk, f32 translation/intersection in Canvas order, and integer coverage conversion. Rectangle starts use floor, clip starts use ceil, and ends use ceil after reconstructing the f32 visible extent. Typed clip/fixed scopes preserve caller-clip and viewport-offset resets, nested fixed restoration, and document offsets. No CSS/layout/DOM processing is introduced. Unsupported alpha (including zero), rounding, text, images and opacity reject the entire stream before instance creation; nonfinite/over-limit inputs and mismatched/unclosed scopes also reject. Nonpositive rectangle extents are no-ops in the accepted subset.

One 8×8-workgroup compute dispatch handles each planned rectangle in a separate compute pass, in display-list order. Each invocation writes at most one pixel and has explicit extent/canvas/storage bounds checks. There is no shader loop over commands, no concurrent overlap within a dispatch, and no attempt to use a workgroup barrier for cross-dispatch ordering. Wgpu tracks storage usage and emits the required inter-dispatch/pass barriers; readback is encoded after all passes in the same submission. One immutable uniform buffer prevents the common error of overwriting one parameter buffer before several queued dispatches execute.

## Explicit experiment limits

- 320×240 nonzero viewport; 256 input commands; 32 combined clip/fixed scopes.
- Finite authored coordinates/offsets bounded to ±1,000,000; resulting translated coordinates stay bounded by their sum.
- At most 257 draws including clear; 4,000,000 rounded shader invocations per plan (including idle lanes).
- 307,200-byte output + 307,200-byte readback + at most 65,792-byte metadata buffer: at most 680,192 explicitly requested GPU-buffer bytes, under the enforced 1 MiB ceiling. This excludes opaque driver/device/pipeline allocations and wgpu upload staging. Host fixture expectations and plans are bounded static inputs, not external data.
- One submission at a time, map completion/readback before the next fixture; no pending frame queue or mailbox.
- 5-second capped future/poll waits within a 20-second application deadline; the runner gives each process 25 seconds by default, captures at most 256 KiB combined output, and gives reaping 2 seconds after process-group SIGKILL. It uses `waitid(WNOWAIT)` before cleanup to retain leader identity and never spawns another adapter after unresolved cleanup.

Synchronous driver calls, future polling internals, device destruction, process creation and kernel-uninterruptible teardown are **not** proven to finish within those deadlines. Failure/timeout remains visible. The supervisor is Linux-specific, does not change driver settings, and requires a new output directory. It freezes exact adapter identity and checks all seven expected PASS records plus COMPLETE; exit zero alone is insufficient. Actual execution requires `--allow-experimental-gpu`.

`Plan`/`Draw` fields are public in this fixed-fixture experiment. A reusable integration must prevent callers from constructing or mutating validated plans. There is no browser fallback or worker security integration here, and no supported external command ingestion API.

## Independent fixtures

Seven cases: overlapping order; fractional clips; fixed reset/restoration; out-of-bounds/empty rectangles; 319×239 dispatch edges; 40 ordered full-frame overwrites; nonzero GPU-clear-only. Tiny expected results are literal pixel maps; large expectations use direct band predicates or constants, independently of planner/shader outputs. Combined readback is 613,052 bytes per adapter. Additional CPU tests cover tiny subpixels, exact/just-over integer edges, touching clips, fixed escape from empty ancestor clip, typed-scope rejection, all resource caps and invalid inputs.

## Reproduction

Pinned wgpu 30.0.1, defaults disabled, exactly `std,vulkan,wgsl`, edition 2024, minimum Rust 1.88. No additional direct dependencies or toolchain installs. `Cargo.lock` copies the published isolated transfer probe dependency resolution; only this root package name changes.

```
cargo test --offline --locked
cargo clippy --offline --locked --all-targets -- -D warnings
cargo build --offline --locked --bins
python3 -m unittest discover -s . -p 'test_*.py' -q
```

Python tests run only bounded fake CPU child processes. Local Rust 1.88.0 and 1.98.0 test, strict Clippy and build results are hash-bound in the host record. A separate offline check also validated WGSL with pinned Naga before the recorded GPU runs; that temporary checker is not required for these reproduction commands.

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

All crate Rust sources forbid unsafe code; transitive wgpu/Vulkan driver internals are outside that claim. Exact source/lock/binary/provenance hashes and offline outcomes are recorded in [the host evidence](evidence/host-raster.json). Earlier native-surface experiment failures remain unchanged; this offscreen prototype is no new compositor evidence.
