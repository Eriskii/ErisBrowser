# Bounded Vulkan pixel-transfer probe

This standalone crate tests whether a safe Rust Vulkan upload/readback path
preserves pixel bytes. It is **not a browser backend**, native presenter,
GPU rasterizer, performance benchmark, or WebGPU implementation. The browser
package does not depend on this crate or its dependencies.

The probe pins `wgpu = 30.0.1` with default features disabled and only
`std`, `vulkan`, and `wgsl` enabled. It selects Vulkan explicitly, creates no
surface or shader, and forbids unsafe code in its own Rust source. Wgpu and the
installed driver have their own unsafe/native implementations.

## Reproduce

Use Rust 1.88 or later, Python 3.10 or later, a POSIX host, and an existing
working Vulkan loader/driver. The host runner requires POSIX process groups to
clean up subprocesses; it reports other platforms as unsupported. Headless
software Vulkan adapters are supported and identified as CPU devices.

From the repository root:

```sh
cargo test --locked --manifest-path tools/vulkan-probe/Cargo.toml
cargo clippy --locked --all-targets --manifest-path tools/vulkan-probe/Cargo.toml -- -D warnings
cargo build --locked --manifest-path tools/vulkan-probe/Cargo.toml
python3 -m unittest discover -s tools/vulkan-probe -p 'test_*.py'
python3 tools/vulkan-probe/run_host.py
```

The crate has an independent workspace and lockfile. Build files go under
`tools/vulkan-probe/target`, and fresh host evidence goes under
`tools/vulkan-probe/output`; both are ignored. A missing driver or inaccessible
GPU is a failed/unavailable probe, never a pixel pass. CI can run the Rust and
runner tests without a GPU; running transfers requires an actual Vulkan
adapter, including an explicitly identified software adapter.

The runner accepts a different compiled binary/output location and an optional
existing loader directory:

```sh
python3 tools/vulkan-probe/run_host.py \
  --binary /path/to/eris-vulkan-probe \
  --output-dir /path/to/probe-results \
  --loader-directory /path/to/existing/vulkan-loader/lib
```

Omit `--loader-directory` when normal loader lookup works. On Linux the option
prepends that directory to `LD_LIBRARY_PATH` in probe subprocesses only; it does
not install a driver or modify the shell/system configuration. The checked-in
runner contains no machine-specific loader path. Other Vulkan loader/driver
environment settings are inherited, so record relevant host configuration when
comparing different environments.

The Rust binary's own interface is `--list` or `--adapter INDEX`. The runner
enumerates first, then tests every reported adapter in a separate process,
validating inventory identity and complete result records. It also bounds each
process's lifetime and captured output. Use the runner for host checks; invoking
the binary directly does not provide the outer process deadline.

## What is checked

Each adapter receives two cases, each with three changed frames:

| Case | Pixel bytes per frame | Packed upload row | Aligned readback row |
| --- | --- | --- | --- |
| 320 × 240 RGBA8Unorm | 307,200 | 1,280 bytes | 1,280 bytes |
| 319 × 239 RGBA8Unorm | 304,964 | 1,276 bytes | 1,280 bytes |

Channels vary independently across rows, columns and frames. Every alpha value
appears, including nonzero RGB under zero alpha. The second width exercises
actual row padding. Every returned pixel byte must equal the generated source;
padding bytes are excluded. Repeated writes reuse the same texture and readback
buffer within each case, exposing stale-frame or synchronization mistakes.

Uploads use `Queue::write_texture`. Readback uses an explicitly submitted
texture-to-buffer copy, submission-specific device polling, asynchronous
mapping, a borrowed mapped view, and unmapping before reuse. Error scopes check
validation, allocation and internal errors. This follows wgpu's distinction
between packed queue uploads and the 256-byte row alignment required by
[encoder copies](https://docs.rs/wgpu/30.0.1/wgpu/struct.TexelCopyBufferLayout.html).

The maximum explicitly live texture payload plus readback-buffer size is
614,400 bytes; expected CPU pixels use at most 307,200 bytes. Wgpu staging,
allocator blocks, driver memory, residency and metadata are additional. A
`MemoryUsage` hint is not a total allocation cap. Dimensions/case counts are
fixed, and the probe accepts no page content, shader source or display list.

Three Rust tests cover dimensions/row alignment, a pending-future deadline and
pattern alpha/frame coverage. Python runner tests cover result validation,
output limits and subprocess cleanup without needing Vulkan.

## Deadlines and limits

Future Pending waits, GPU submission polling and mapping callback receipt each
have a five-second application deadline. The host runner defaults to a separate
25-second deadline per enumeration/device process and kills its process group
on timeout, interruption or runner errors, with bounded cleanup waits.

Application waits cannot interrupt synchronous driver initialization, a future's
`poll`, native API calls or destructors. Driver teardown can itself wait. Process
termination helps contain those failures, but cannot guarantee timely recovery
from a driver/kernel stuck in uninterruptible work. No native presentation,
compositor synchronization, device-loss recovery or production GPU sandbox is
tested. Adapter enumeration itself precedes the application adapter-count cap.

Exact transfers do not establish rendered-image equivalence after surface
format conversion, sampling, blending, color management or presentation. They
do not measure browser performance or satisfy the custom Vulkan renderer goal.

## Recorded host result and MSRV

The compact [evidence](evidence/host-transfer.json) records the published source
and lock hashes, toolchain, observed adapters and results from an actual host
run. All three devices—RTX 4070 SUPER, AMD Ryzen 7 7800X3D integrated graphics
(RADV), and llvmpipe—passed six comparisons each. The host's existing Vulkan
loader required an explicit process-local library directory; no host setting
changed. Elapsed values, if recorded, are deadline diagnostics rather than
performance measurements.

The independent Linux dependency inventory is in
[evidence/linux-dependencies.json](evidence/linux-dependencies.json). The pinned
resolution contains 96 registry packages across targets, with 58 active Linux
dependencies. The Linux manifests declare no Rust requirement above 1.88; ten
omit `rust-version`. Wgpu's family declares 1.87, but manifest inspection cannot
establish undeclared compatibility. The recorded host run compiled with Rust
1.95; Rust 1.88 was not installed or executed locally. Subsequently, the
dedicated [Rust 1.88 CI job for
`dde3ff1`](https://github.com/Eriskii/ErisBrowser/actions/runs/36396836593/job/108845129192)
passed formatting, strict Clippy, the three Rust tests and 12 Python runner
tests with the pinned lockfile. CI did not run GPU transfers; those retain the
separate host evidence above. No local toolchain installation was performed for
the probe.

## Provenance

The Rust source was authored for Eris as an isolated `/tmp` experiment, then
published here with its own pinned lockfile and a portable, tested host runner.
The implementation uses the documented wgpu 30.0.1 APIs; it is not copied from
another browser or an upstream sample. No cached crate sources, local Cargo
metadata paths, executables, or driver libraries are checked in. Dependency
sources remain upstream packages identified by Cargo.lock checksums.

Primary API references: [Instance](https://docs.rs/wgpu/30.0.1/wgpu/struct.Instance.html),
[Device](https://docs.rs/wgpu/30.0.1/wgpu/struct.Device.html),
[Buffer](https://docs.rs/wgpu/30.0.1/wgpu/struct.Buffer.html), and
[Queue](https://docs.rs/wgpu/30.0.1/wgpu/struct.Queue.html).

The [renderer milestone design](../../docs/vulkan-rendering.md) describes the
separate presentation and custom-rasterization work still needed.
