# Offscreen browser display-list bridge

The implementation now lives in the browser crate's `graphics::raster_bridge`
module behind the optional `raster-bridge` feature. That feature enables only
CPU planning; it does not load a graphics driver. The probe's `browser-bridge`
feature forwards to it and preserves the original adapter imports by reexport.
The 21 adapter and seven text-adapter test groups now run explicitly in the root
crate. Native window rendering is unchanged.

The optional `browser-bridge` feature connects Eris's own worker snapshots to
the standalone custom WGSL rasterizer. The browser window still uses its CPU
painter; its optional Vulkan presenter uploads completed CPU frames. This
checker is the next integration step, with no native-surface or Chromium
performance result.

The [recorded host run](evidence/host-browser-bridge.json) passes all 16 cases
on NVIDIA, AMD and software Vulkan. Per adapter, 151 pixels are compared through
the GPU path and 46 through CPU fallback: 788 packed comparison bytes combined,
2,364 across all three adapters. Every process exits normally with empty stderr,
complete cleanup and the required pre-Vulkan grant. Both Rust 1.88 and 1.98 pass
50 feature-enabled tests; 11 supervisor process tests and 14 synthetic protocol
tests also pass. The initial private viewport-test failure and its correction
remain in the evidence.

The adapter borrows `worker::Snapshot` or an original `DrawCommand` list and
image store. Supported frames contain zero-radius rectangles, nearest-neighbor
images, Canvas-style line rectangles, and balanced clip/fixed scopes. Every
text, rounded rectangle or opacity command refuses the entire frame, even if
hidden or transparent. Missing image keys are legal no-ops. Admission never
loads an image or drops an earlier placeholder rectangle.

Whole-frame CPU fallback paints the original commands and images through
`Canvas::paint_with_viewport`, preserving clear, caller clip, document offset,
fixed viewport offset and exhaustion reporting. GPU execution or pixel
comparison failures are errors; they cannot turn into successful fallbacks.

The independently frozen [oracle](browser-fixtures/oracle/contract.md) has 16
cases: 11 direct display lists and five HTML/image documents loaded through the
actual confined browser worker. Its 197 literal pixels specify nine GPU frames
and seven CPU fallbacks. The checker verifies actual worker geometry, scopes,
image identities and decoded RGBA before accepting a plan. Canonical `EWB1`
records preserve complete display commands, image keys and shared identities,
source bytes, diagnostics and request metadata. They exclude DOM and hit-test
regions. Pixel expectations were authored before renderer execution.

The original 30 standalone fixtures, shaders and resource limits remain
unchanged. The bridge adds conservative CPU preparation limits: 256 original
commands, 256 image-store keys, map capacity at most 512, 4 KiB per key, separate
64 KiB totals for store and command keys, bounded borrowed sorting and binary
lookup, and an upper bound on CPU pixel work. Equal Arc identities share packed
source storage; equal bytes in distinct Arcs remain distinct sources. All stored
sources are validated, including unused entries. GPU case records expose both
stored and referenced source counts/bytes, missing-image counts and lowered
command counts. A refused frame has no successful-plan statistics.

All five workers finish before Vulkan initialization. The checker drops worker
clients and snapshots, checks every own task's child list, flushes its capture
footer, then waits for an explicit supervisor grant. A dedicated single-threaded
Linux subreaper grants only when its sole child is the live checker. This also
detects worker descendants already orphaned to the supervisor. No worker starts
after the grant. Adapter enumeration runs separately without worker capture.

The supervisor retains child identity with `waitid(WNOWAIT)`, signals each
owned process/group before reaping, and repeats bounded adoption cleanup. It
never treats a numeric PID after reap as continued ownership. Output is capped
at 2 MiB, checker time defaults to 60 seconds, and cleanup has five seconds.
Cancellation, surviving descendants, inaccessible ownership information and
unresolved cleanup fail the run and stop subsequent adapters. These limits do
not establish a finite bound on kernel-uninterruptible process creation,
destruction or driver calls. The parent host records an unresolved failure if
the supervisor itself cannot finish.

The checker caps retained target pixels, plan buffers and draw metadata across
all 16 cases. Those bounds exclude allocator overhead, fonts, driver internals
and other process storage. No total-process memory or production security claim
follows from this experiment.

From this directory, build and check the optional feature with:

```sh
cargo clippy --locked --all-targets --features browser-bridge -- -D warnings
cargo test --locked --features browser-bridge
cargo build --locked --bins --features browser-bridge
python3 -m unittest discover -s . -p 'test_*.py'
```

With an already-built Eris browser and installed Vulkan loader:

```sh
python3 run_browser_host.py --allow-experimental-gpu \
  --binary target/debug/eris-vulkan-browser-check \
  --browser ../../target/release/eris-browser \
  --output-dir browser-host-run-1 \
  --loader-directory /path/to/installed/loader
```

The host requires a new output directory, binds both executable files and the
frozen oracle, retains raw process logs, and checks every snapshot, case and
completion record. Compilation and CPU-only tests do not establish GPU pixel
equality. Test and host outcomes are recorded separately with their exact input
and binary hashes.
