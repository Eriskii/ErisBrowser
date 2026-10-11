# Background boxes in a native window

This bounded host check combines Native color, image and opaque-gradient boxes
in one page loaded through the confined renderer and resource broker. The recorded
NVIDIA run matched one acquired Vulkan texture against the complete Canvas
reference before the compositor. A separate rounded-image case confirmed ordinary
whole-frame CPU fallback.

The portable [fixture bundle](../tools/native-background-fixtures/) contains two
HTML cases, a literal 4×2 PNG, geometry/pixel oracles and their freeze manifest.
The owned loopback listener serves `/admitted.html` and `/two-pixels.png`.
The rounded case changes only the document's body class at the same route.

Each page has three 4×2 content boxes with 1px padding and transparent border.
Padding-box positioning and content-box clipping produce one white canvas command
and three box commands: a color rectangle, an image and a gradient. Their box work
is `32 + 61 + 66 = 159`, with no box masks and one 12-byte generated gradient source.
Browser chrome has separate commands and resources.

The negative case's outer radius of 3px leaves a 1px content radius. Native
preparation refuses phase 0, original command 2; normal drawing uploads the complete
CPU frame. Its independent software oracle changes pixel `(29, 11)` from blue to
`#3030ff`, using 13 of 16 coverage samples. That route makes no acquired-pixel claim.

## Running the controlled check

Use a pinned `vulkan-raster` binary, its recorded SHA-256, a Vulkan loader
directory and a fresh output directory. This is the successful X11 invocation
pattern; choose an existing unit-scale monitor ID appropriate to your host:

```sh
env -u WAYLAND_DISPLAY -u WAYLAND_SOCKET WINIT_X11_SCALE_FACTOR=1 \
  python3 tools/native_background_host.py \
  --binary /absolute/path/to/eris-browser \
  --binary-sha256 YOUR_SHA256 \
  --loader-directory /absolute/path/to/vulkan-loader/lib \
  --fixtures tools/native-background-fixtures \
  --output /absolute/path/to/new-results \
  --with-fallback --x11-physical-size --unit-scale-monitor 1 \
  --allow-experimental-gpu
```

The controller requires Hyprland, `hyprctl`, Xlib and supported Linux
Landlock/seccomp confinement. The optional monitor control moves only the owned
window to the selected monitor's existing active workspace. A bounded helper
confirms the exact 1280×880 physical X11 size before document release. The controls
share a three-second deadline and 48-command limit. They issue no explicit focus
or global configuration change; moving a window can still affect focus naturally.
Headless `--width`/`--height` do not set this window size. Non-unit browser zoom
continues to refuse BoxDecoration commands.

The positive route uses `--benchmark-native-check`, which collects no timing
samples. A check-only diagnostic joins box counters to frame serial, navigation
generation and viewport revision, then to Native presentation, exact acquired
texture verification, the one-frame report and cleanup. A prepared-frame line
alone does not prove submission or presentation.

The host preserves served-body hashes, the pre-release diagnostic prefix,
canonical EBNI paint inputs, executable/helper/loader bindings and raw output.
Page and broker PIDs are distinct values logged by the trusted browser, without
independent procfs lineage authentication. X11 window attribution uses its PID
property, not XRes authentication. Owned-process control and supervised cleanup
are checked separately. Renderer resource authority is unchanged.

## Recorded validation

The final host run passed both cases on an NVIDIA GeForce RTX 4070 SUPER, using
monitor 1 (`DP-3`), scale 1 and zoom 1. The admitted frame matched all 4,505,600 bytes
at serial 7, generation 1, viewport revision 5. The rounded case reported the exact
unsupported-box fallback and compared no acquired bytes. Both owned runs exited
successfully with zero surviving-at-exit descendants. The pinned browser was a
Rust 1.88 dev build stripped with `--strip-debug`, 62,699,912 bytes.

The two literal Page/Native tests and complete Native suite passed: 2,902 tests,
zero failures or ignored tests. Strict Native Clippy passed on Rust 1.88 and 1.98,
as did formatting. Final host tests passed 19 cases; existing Native and supervisor
helper suites passed 91 and 19. The final source inventory contains 678 inputs.
The Rust checks retain their earlier snapshots; exact comparison shows that only
the two host Python files changed before the final host tests, browser build and
window run. All nine CI jobs also passed for the published parent `4bd9f887`.

Four failed whole-host attempts remain in the evidence:

- Attempts 1 and 2 stopped before document release: logical 1280×880 became physical
  1920×1320 at scale 1.5. The second attempt's X11 environment alone did not fix it.
- Attempt 3 could not confirm exact physical size and made no HTTP requests.
  Cleanup completed, but the supervisor recorded `surviving_descendants`, two
  descendants and three reaped processes after forced browser termination.
- Attempt 4 produced an exact raw acquired frame, then failed its wrapper-input
  join because the default duration serialized as `8` versus `8.0`. The retained
  read-only validation confirms that raw frame; the whole-host attempt stays failed.
  Changing the argparse default to `8.0` and adding a CLI regression preceded run 5.

Attempts 1, 2 and 4 recorded zero descendants at supervisor exit. No shader or
rendering algorithm changed. These observations cover the sampled NVIDIA adapter
and acquired texture before compositor output; they establish no performance or
broader hardware/compatibility result. The
[summary](evidence/native-background-window/summary.json) and
[archive](evidence/native-background-window/validation.tar.gz) retain the results,
corrections and failed attempts.
