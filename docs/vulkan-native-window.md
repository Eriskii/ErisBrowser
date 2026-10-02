# Experimental native GPU rasterization

The `vulkan-raster` feature connects the custom renderer to the existing Vulkan
window owner. This remains bounded, optional rendering with complete CPU fallback.
Four controlled native-window cases pass on the NVIDIA host. See the
[retained evidence](evidence/vulkan-native-window.json).

## Run it

On Linux, build the optional native raster feature and select both the Vulkan
presenter and GPU raster route:

```sh
cargo run --locked --release --features vulkan-raster -- ./tools/native-raster-fixtures/admitted.html --presenter=vulkan --raster=gpu
```

CPU rasterization remains the default. `--presenter=vulkan --raster=cpu` uses
the existing CPU-painted framebuffer upload. The `vulkan-raster` feature enables
the Vulkan presenter, browser raster adapter and reusable GPU core together;
workers retain the existing clean-exec launcher and descriptor checks.

Native scene preparation starts only when a loaded worker snapshot exists for
the current page generation. Startup and loading frames use the complete CPU
route. An admitted scene contains the page, page overlays and browser chrome
in their original order, with separate document/fixed offsets and clips. Normal
native packets own the prepared plan; they do not contain a CPU `Canvas`,
comparison framebuffer or readback buffer.

For explicit correctness checking, add `--vulkan-verify-frames 1` (accepted
range: 1–8). This mode also paints a complete CPU reference and compares it
against the acquired Vulkan texture before presentation. CPU fallback uploads
do not count toward native verification. A successful comparison establishes
those captured texture bytes before the compositor, not the compositor's final
displayed output. `--raster=gpu --window-screenshot ...` is currently rejected;
there is no native GPU screenshot capture channel. Headless `--render` remains
the CPU renderer.

## Rendering scope and fallback

The GPU performs ordered integer compositing, image sampling, packed RGB output
and conversion to opaque BGRA8/RGBA8 surface bytes. Rounded rectangles use CPU
geometry coverage matching the existing Canvas rules. Text uses CPU coverage
from the bundled fonts and the existing font library; the resulting masks are
composited on the GPU. This is a hybrid renderer, not GPU font-outline or rounded
geometry rasterization. It does not add shaping, bidirectional layout, author
fonts or general web typography support.

Unsupported group opacity, non-unit zoom, process-error overlays, excessive
viewport dimensions, command/source limits, preparation limits or raster work
refuse the entire native scene. The complete existing CPU painter then draws
the page and chrome, and its pixels use the existing upload path. No accepted
page prefix or partial native plan is presented. Large or HiDPI windows can
fall back even below the viewport maximum because individual masks and the
complete scene must also fit their limits. Missing image resources keep their
existing legal no-op behavior.

Admission refusal is distinct from GPU failure. Encoding, device, error-scope,
submission or presentation errors retain the presenter's failure and ownership
protocol; they are not silently converted into a successful native admission.
Another graphics owner can take over only after positive acknowledgment of
actual resource destruction.

## Resource and ownership contract

The Native profile admits at most 1280×1024 physical pixels, 16 MiB of planned
explicit GPU buffers, and 4,000,000 padded invocations including clear, raster
passes and mandatory surface conversion. The buffer plan includes packed RGB
output, 256-byte-aligned conversion rows, a 16-byte conversion uniform and
raster metadata/input. Explicit verification additionally reserves readback;
planned native buffers plus readback must fit 24 MiB.

Whole-scene ledgers cover at most four phases (the browser currently supplies
three), 256 original commands and 256 lowered operations, 32 nested scopes,
and 256 combined image/mask sources. Image RGBA storage is capped at 1 MiB.
Mask axes are at most 1024, unique coverage totals at most 262,144 bytes, and
row origins total at most 65,536 entries. Text has additional input, scalar,
cold-request and CPU preparation limits. A phase boundary resets its coordinate
state, not these resource ledgers. The old Probe profile and frozen probe
fixtures retain their existing limits.

Preparation checks a conservative peak against a 16 MiB next-UI-frame reserve
before constructing the scene. Final GPU invocation admission can still reject
a plan after bounded CPU coverage preparation. The presenter's existing 128 MiB
presenter ledger includes that reserve, active and latest-pending packet
capacities, retained CPU upload scratch, active native GPU buffers and optional
readback. It does not bound process RSS: allocator/driver overhead, surface
implementation storage, already-owned page/image assets, CPU font caches and
temporary CPU-fallback paint storage are outside this explicit ledger. Trusted
font-library outline allocations and noncancellable work remain the documented
bundled-font limitation.

The owner keeps one active frame and one replaceable latest pending packet.
Generation, viewport and visibility checks prevent stale presentation; a newer
same-epoch serial does not continually invalidate active work. The active
five-second deadline is shared across acquisition, encoding, retirement and
verification rather than restarted per phase.

If encoding aborts after queue writes may have been enqueued, the partial draw
encoder is discarded. A single empty submission flushes those metadata writes
and is retired under the original deadline; the partial draw prefix is never
submitted. Resources and surface ownership remain held through required
completion or the owner's destruction path. A timeout or finished thread alone
does not prove release.


## Acquired-window validation

The fixed 1180×880 fixture is loaded through the confined page worker and
contains text, a decoded local image, fixed positioning and enough document
height for the rounded scrollbar. The page, overlays and browser controls share
one plan. On the NVIDIA RTX 4070 SUPER with a `Bgra8Unorm` surface, one acquired
texture matches **4,153,600 bytes** of the complete original Canvas reference.
The scene's generation/serial joins the actual presented and verified frame;
startup chrome cannot consume the verification quota.

A separate normal run presents a loaded native scene with no CPU reference or
readback. Opacity and deliberately redundant full-page backgrounds trigger
complete CPU fallback with no native success records. These fallback cases
verify route admission, not acquired fallback pixels. All four browser runs
exit zero and leave no owned descendants. These checks are before the desktop
compositor, on one selected native adapter and format. They establish no latency,
throughput or Chromium-relative performance result.

Two follow-up runs also pass their expected outcomes. After an actual native
presentation, an owned-window reload starts a fresh confined page worker and
broker and prepares a generation-two native scene. This checks worker startup
after Vulkan is active; it does not claim a second presentation or pixel
comparison. The opacity fixture with one requested native verification frame
exits nonzero with the expected `0/1` incomplete quota. Its complete CPU fallback
cannot satisfy native verification. Both runs leave no owned descendants.

Two earlier attempts are retained as failures: the compositor selected a
1275×764 window whose address-bar mask exceeded the 1024-pixel axis cap; after
controlling the size, the original two-background scene exceeded the unchanged
four-million invocation cap. That exact HTML is preserved as `overdraw.html`.
The admitted fixture uses one propagated white background. No budget was raised.

Both Rust 1.88 and 1.98 pass the complete native-feature suite (**1,349 tests**),
strict all-target Clippy across the affected feature variants, **47** adapter
groups, and **78 default / 89 GPU-feature** core groups. The final size-diagnostic
addition also passes the **63** browser/presenter tests on both versions.
The host's **22** synthetic tests check protocol identity/order, refusals and
owned-process window control. The separate follow-up harness adds **13** tests;
all **35** pass.

The host check is explicit and requires existing desktop/Vulkan libraries:

```sh
python3 tools/native_raster_host.py \
  --binary /path/to/eris-browser \
  --output /tmp/eris-native-check-new \
  --loader-directory /path/to/vulkan-loader/lib \
  --allow-experimental-gpu
```

It requires the fixed physical viewport. On Hyprland, add
`--hyprland-size 1180x880` to control only the newly launched browser's window.
The controller retains its direct child unreaped during PID-specific commands;
the outer subreaper owns descendant cleanup. It does not change global window
rules or operate on other applications.

`tools/native_raster_followup.py` accepts the same binary, output and loader
arguments and requires Hyprland. It sizes its owned window to 1180×880, sends
Ctrl-R only after observing native presentation, and checks the opacity quota
refusal separately. Both harnesses retain source/binary hashes, raw output and
process cleanup receipts. The tested binary is a debug build with debug sections
removed while preserving allocated ELF sections; it is not a performance build.
