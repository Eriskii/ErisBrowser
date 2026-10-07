# Native opacity across the binary32 range

The optional Vulkan raster route accepts every finite binary32 opacity in
`[0,1]`, including transparent and nested groups. It preserves the original
stored opacity bits and the software painter’s separate rounding stages.
Admission still applies to the complete scene under existing resource limits.

```sh
cargo run --locked --release --features vulkan-raster -- \
  ./examples/vulkan-full-opacity.html --presenter=vulkan --raster=gpu
```

Click “Toggle opacity” to change the outer group between 0.1 and 0.7. The inner
transparent group remains at 0.7. Software rendering remains the default.

## Why decimal opacity needs separate arithmetic

Rounding an opacity to a nearby `k/256` changes pixels. Opaque gray232 over black
at stored `0.1` produces gray23, while `26/256` produces gray24. Even using the
exact rational value of the stored float is insufficient if intermediate
rounding is omitted: gray5 over black at stored `0.7` produces gray4 after the
software painter’s separate multiplication and division, versus gray3 with
one final rational rounding.

The [earlier transparent compositor](vulkan-transparent-opacity.md) describes
the software operations. The new path retains each operation using pairs of
u32 words; it needs neither shader floating point nor optional 64-bit integers.
Grid values retain their previous fast path and exact metadata encoding.
Other values use an explicit route flag and the original f32 bits in existing
reserved uniform words. Binding size, stride and buffer count are unchanged.

For a positive opacity `t <= 2^-25`, the rounded inverse alpha is exactly one.
The source contribution and subsequent rounding change a channel by less than
`1/256` of an RGBA16 level, so the final stored parent channel is unchanged.
Root channels begin at `257*d` for integer RGB8 `d`; division rounding also
cannot move the result to another RGB8 value. This covers all subnormal inputs
and the exact threshold. It is an identity of the final stored pop only:
nonempty tiny groups still validate, allocate scratch, paint their descendants,
and pay all clear/composite charges. Genuine zero and unit scopes retain their
existing suppression and target behavior.

Above that threshold and below one, decode `t=M/2^r` with a 24-bit significand
and `24 <= r <= 48`. The alpha quotient, alpha multiplication and inverse are
rounded separately, recovering `I=J/2^24`. Each weighted channel is represented
on a `2^-48` lattice:

```
source = R24(S*M) << (48-r)
destination = R24(D*J) << 24
V = R24(source + destination)
C = V / 2^48
```

`R24` rounds to 24 significant bits, ties to even, while retaining the integer
scale. Explicit word shifts and carry handling preserve rounding across the
32-bit boundary. The valid premultiplied channel bound keeps the pre-add sum
and rounded result below `2^64`; these helpers do not accept arbitrary
unbounded integer inputs. Parent conversion performs the software half-up
integer rounding. Root conversion retains the separately rounded binary32
division by257 before converting to RGB8. Nested groups are composited one
level at a time; their opacities are never multiplied together as a shortcut.

## Admission and ownership

The command, scope, source, mask, CPU preparation, four-million padded invocation
and 16 MiB GPU-buffer limits are unchanged. Cropped RGBA16 scratch remains
part of the retired-buffer reuse key. Invalid values and malformed descendants
still reject the complete plan. More shader integer instructions may cost GPU
time; equal dispatch counts do not establish equal performance.

The browser’s existing all-grid diagnostic stays byte-for-byte compatible.
Mixed or general groups report ordered eight-digit hexadecimal opacity bits
through a bounded formatter, without building an additional group list.
The [native-window contract](vulkan-native-window.md) describes fallback,
resource accounting and GPU ownership.

## Validation scope

The independent arithmetic verifier checks all 65,536 alpha values against each
of 30 selected opacity bit patterns: 1,966,080 inverse checks. It additionally
performs 4,778,616 channel checks and 2,256,815 root checks, including 262,144
seeded premultiplied samples and 1,567 root division boundaries. General values
compare intermediate binary32 bits; tiny values compare the final stored
channels justified by the identity proof. These selected sweeps and samples
are not an exhaustive enumeration of every opacity/channel combination.
Six new core test groups retain the verifier, alongside the unchanged grid
arithmetic suite.

The GPU corpus preserves the earlier 17 positive cases and both reuse corpora.
It adds raw-bit decimal, subnormal, tiny-threshold and below-one cases plus
mixed grid/general nesting. A third reuse corpus changes same-size contents,
resizes, and exercises cancellation using raw opacities. Literal expectations
were derived before execution; shader output does not generate its own oracle.

Rust 1.88 passes 2,153 browser tests with `vulkan-raster`, including the confined
worker checks; 146 GPU-feature and 114 default core tests; seven probe tests;
98 Python host tests; strict Clippy and formatting. The 76 bridge tests include
17 opacity groups. Page and confined-worker witnesses each cover four scenes
before and after a real click. The release browser also builds successfully.

The offscreen checker passes on NVIDIA RTX 4070 SUPER, AMD RADV and llvmpipe.
Each adapter compares 58 literal frames (608 bytes) and three reuse corpora of
14 frames (448 bytes) each, across BGRA8 and RGBA8: **300 frames and 5,856 bytes
in total**. Per adapter, successful frames record 21 allocations, 21 reuse hits
and 21 evictions; 24 separate cancellation checks are excluded from those
allocation counts. All owned child processes exit and are reaped. The offline
case limit grows from 17 to 29 and the host log-record limit from 128 to 192 solely
to admit the expanded corpus; renderer limits are unchanged.

The first browser test run exposed a test assumption: removing two empty
opacity markers also changes the status text and hence glyph work. The corrected
assertion subtracts each independently reported text debit, preserving an exact
two-frame-area opacity charge. The original failure and a later module-order
formatting correction are retained in the
[evidence package](evidence/vulkan-full-opacity.json). This increment does not
add an acquired browser-window measurement. Full CSS compositing, broader
native scene admission, complete web compatibility, production security and
performance within 30% of Chromium remain unfinished.
