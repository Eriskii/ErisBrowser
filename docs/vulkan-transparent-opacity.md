# Transparent native opacity

This records the earlier `k/256` milestone and its measurements. The
[full-range extension](vulkan-full-opacity.md) subsequently removes the non-grid
restriction; the historical results below retain their original scope.

The optional Vulkan raster route supports transparent and nested opacity groups
at `k/256`, for integer `1 <= k <= 255`. Image-only groups, glyph coverage,
rounded shapes and holes between separate shapes no longer need an opaque
backing rectangle. The planner still admits a complete bounded scene or falls
back to the software renderer.

```sh
cargo run --locked --release --features vulkan-raster -- \
  ./examples/vulkan-transparent-opacity.html --presenter=vulkan --raster=gpu
```

Click “Toggle opacity” to change the outer group between 0.5 and 0.75.

The existing command, source, mixed-scope, CPU preparation, four-million padded
invocation and 16 MiB GPU-buffer limits remain. Each group uses a disjoint
cropped premultiplied RGBA16 region. Clear and composite passes retain their
charges, and the scratch shape remains part of the retired-buffer reuse key.
Zero opacity validates its contents before suppression; unit opacity retains
the current destination. Non-grid values still require whole-frame fallback.

## Exact transparent compositing

The software painter rounds several separate binary32 operations during a group
pop. Replacing them with a single ideal rational source-over expression changes
pixels. For example, black with alpha55 inside a group at192/256 over gray34
produces gray28; ideal rational arithmetic rounds to29. A nested case also makes
an intermediate RGBA16 difference visible: gray44, outer opacity127/256 with
gray151 backing, and inner opacity128/256 containing black alpha38 produces92,
where the rational substitute produces91.

For source alpha `A`, premultiplied source channel `S`, destination channel `D`
and opacity `k/256`, the relevant software steps are:

```
Q = f32(A / 65535)
P = f32(Q * k/256)
I = f32(1 - P)
C = f32(f32(S * k/256) + f32(D * I))
parent = round(C)
root = round(f32(C / 257))  // root D is RGB8 multiplied by257
```

Each `f32` means nearest representable binary32, ties to even. The final
`round` is halfway away from zero, as specified by [Rust's f32 operations](https://doc.rust-lang.org/std/primitive.f32.html).
The [WGSL floating-point contract](https://www.w3.org/TR/WGSL/#floating-point-accuracy)
does not provide the same exact sequence. The custom shader therefore performs
these stages with integers; it uses no floating-point arithmetic or optional
64-bit integer feature for a group pop.

For `0 < A < 65535`, normalize `A` using `h=floor(log2(A))` and
`X=A<<(23-h)`. The identity `A*2^(39-h)/65535 = X + X/65535` supplies the exact
rounded alpha significand. Multiplication by `k`, explicit ties-even rounding
and subtraction on a `2^-24` lattice recover `I=J/2^24`. Both odd-divisor
roundings use bounded u32 division.

The destination product `D*J` and weighted source are represented as two u32
words. Rounding each product and then their sum to24 significant bits preserves
the distinct CPU stages. Parent conversion uses the half-bit at position23.
For root conversion, normalization chooses a significand threshold8421376 and
the identity `B*256/257 = B-B/257` reproduces the binary32 division before final
integer rounding. Every shift is bounded; explicit branches handle zero and
values below the final rounding threshold.

Transparent source pixels are identities. Opaque source pixels retain the
previous short integer formula. For intermediate alpha, premultiplied channels
remain ordered by monotonic rounded operations; the total arithmetic error
relative to ideal source-over is less than5/512 of an RGBA16 level, keeping the
rounded result within65535. More integer instructions can increase GPU cost;
unchanged dispatch counts are not a performance measurement.

## Validation

Rust 1.88 passes 2,149 native-feature tests, including the confined-worker
checks; 137 GPU-feature and 106 default core tests; seven probe tests; strict
Clippy; and formatting. The 73 bridge tests include full Canvas comparisons for
unbacked images, text, rounded shapes, fixed descendants and nested transparent
parents. The loaded Page and worker witnesses each cover three scenes before
and after a real click, including the new public example.

Six arithmetic test groups compare all 16,711,680 alpha/opacity pairs against
separate CPU binary32 operations, plus 812,438 channel checks and 275,247 root
checks. The latter include 1,400 division-boundary checks and 262,144 seeded
premultiplied samples. These are arithmetic observations, not GPU frame counts.

The standalone release browser also builds successfully with `vulkan-raster`.
The release checker passes on NVIDIA RTX 4070 SUPER, AMD RADV and llvmpipe. Each
adapter compares 34 literal frames (512 bytes), 14 opaque reuse frames (448
bytes), and 14 transparent reuse frames (448 bytes), across BGRA8 and RGBA8:
**186 frames and 4,224 compared bytes in total**. Each adapter also checks 16
cancellation boundaries; its two successful reuse sequences together record 14
allocations, 14 reuse hits and 14 evictions. Those allocation counts exclude
cancellation leases. One non-grid fixture is refused before Vulkan execution.
All nine earlier positive literals and the old opaque reuse corpus remain.

The first checker attempt failed before adapter enumeration because the
expanded 17-case corpus exceeded its 16-case inventory cap. The corrected
checker permits 17; its retained-reference, active-GPU, timeout and all renderer
limits are unchanged. The host parser permits 128 bounded log records to cover
three inventories and both reuse sessions. All 98 Python host tests pass.

```sh
cargo build --locked --release \
  --manifest-path tools/vulkan-raster-probe/Cargo.toml \
  --bin eris-vulkan-opacity-check
python3 tools/vulkan-raster-probe/run_opacity_host.py \
  --binary tools/vulkan-raster-probe/target/release/eris-vulkan-opacity-check \
  --output /tmp/eris-transparent-opacity-check --allow-experimental-gpu
```

Pass `--loader-directory` when the installed Vulkan loader is outside the
library path. The [evidence package](evidence/vulkan-transparent-opacity.json)
preserves commands, source hashes, arithmetic observations, raw adapter logs,
cleanup records and the first failed attempt. This increment has no new acquired
browser-window measurement; the [original opaque-backed milestone](vulkan-opacity.md)
retains its separate window records. General opacity values, complete CSS
compositing, full web compatibility, production security and performance within
30% of Chromium remain unfinished.
