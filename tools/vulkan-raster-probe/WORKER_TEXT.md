# Worker text through the Vulkan rasterizer

`eris-vulkan-worker-text-check` loads seven local HTML documents in real,
confined browser workers with scripts disabled. It passes their complete
snapshots to the fonts-aware adapter and compares the custom Vulkan output
with the original CPU `Canvas` painting those same snapshots. Six documents
must use GPU rasterization; the rounded-rectangle document must fall back for
the complete frame.

This is a same-snapshot differential check. It does not independently establish
correct HTML layout, font shaping or typography. The separate
[glyph suite](GLYPHS.md) retains its independent literal-mask cases and
preserved-parent font references. Native windows still use CPU painting, with
an optional Vulkan upload presenter. The
[native integration proposal](../../docs/vulkan-native-plan.md) describes the
remaining work.

## Captured inputs and comparison

The [fixture inventory](worker-text-fixtures/inventory.json) specifies each
160×80 viewport, navigation generation, paint offsets, exact text contents,
font flags, size, color, clip/fixed scope, meaningful paint order and image
bytes. The cases cover Unicode block flow, bold/italic/monospace selection,
overflow clipping and restoration, scroll with fixed content, text/image
ordering, translucent underlines, and rounded-rectangle fallback.

Each snapshot is retained as a bounded EWB1 record before its assumptions are
checked. No drawing commands are removed from the CPU or GPU inputs. Incidental
transparent rectangles are ignored only when checking the expected sequence
of meaningful primitives. Normal worker isolation diagnostics remain in the
record. Positions produced by layout are observed inputs, not independent
geometry expectations.

For every snapshot, the checker requires identical cold/warm CPU pixels and
identical cold/warm admission results. Admitted plans must also match in their
metadata, draws, uniform parameters and packed input bytes. GPU execution
compares the complete target pixel buffer. No CPU-painted target is uploaded
as raster input. Coverage masks and decoded images remain the only pixel
sources supplied to the custom drawing kernels.

Seven CPU references contain 358,400 bytes. Each CPU-only run performs fourteen
paints, comparing 716,800 bytes. Each GPU adapter compares 307,200 GPU bytes
across six frames; the remaining 51,200-byte reference belongs to the CPU
fallback frame and is never counted as GPU comparison.

The [recorded host run](evidence/host-worker-text.json) passes on NVIDIA RTX
4070 SUPER, AMD RADV and llvmpipe: **921,600 GPU-compared bytes** in total.
Both the Rust 1.88 and 1.98 checkers also pass CPU-only capture. All supervised
processes exit zero with empty stderr and complete cleanup. The original
sixteen-case browser suite still passes on all three adapters after capture
helpers are shared between the two checkers.

## Process boundary and limits

All workers, snapshots and font objects are dropped before GPU initialization
within a capture run. The checker verifies that its own threads have no child
processes, prints `TEXT_CAPTURE_COMPLETE`, and blocks for `GPU_READY`. The
isolated Linux subreaper grants that transition only to a live checker with
the exact seven-worker inventory and no adopted descendants. The original
browser suite uses its separate marker and cannot satisfy this gate.

The host checks exit status, empty stderr, complete descendant cleanup, exact
phase ordering and all comparisons. Failure stops subsequent adapter runs.
CPU-only mode neither requests a GPU grant nor initializes Vulkan. Adapter
enumeration is a separate supervised process. Binary, fixture, host-source and
explicit loader bindings are checked around every process; this is evidence
binding, not an atomic defense against concurrent changes by a trusted user.

The existing GPU and font caps are unchanged: 320×240 maximum engine target,
256 lowered operations, 32 scopes, 1 MiB explicit GPU storage per plan,
four million padded invocations, and bounded glyph masks/rows. This suite
narrows targets to 160×80 and retains at most seven plans. Each EWB1 snapshot
is at most 65,536 bytes; total process output is at most 2 MiB. The host timeout
defaults to 60 seconds and permits at most 120 seconds, followed by bounded
owned-child cleanup.

## Running it

Build the normal browser and the optional checker:

```sh
cargo build --locked --release --bin eris-browser
cargo build --locked --manifest-path tools/vulkan-raster-probe/Cargo.toml --features browser-bridge
python3 tools/vulkan-raster-probe/run_worker_text_host.py \
  --binary tools/vulkan-raster-probe/target/debug/eris-vulkan-worker-text-check \
  --browser target/release/eris-browser --cpu-check --output /tmp/eris-worker-text-cpu
```

For actual GPU comparisons, use a fresh output directory and replace
`--cpu-check` with `--allow-experimental-gpu`. On hosts that require an explicit
Vulkan loader directory, supply `--loader-directory /path/to/lib`. Every
enumerated Vulkan adapter is tested in order, stopping on the first failure.
The output directory retains exact stdout, stderr, cleanup receipts and the
validated host result.
