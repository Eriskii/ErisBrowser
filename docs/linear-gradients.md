# Linear gradient backgrounds

This chapter records the initial software checkpoint; the [native follow-up](vulkan-gradients.md) adds bounded CPU-prepared opaque fragments to the existing GPU image path.

`background-image` and the supported `background` shorthand accept one
`linear-gradient()` layer. The software painter evaluates the gradient directly;
it does not allocate a generated bitmap or add an image-store resource.

```sh
./run.sh ./examples/linear-gradient.html
```

Click the red/blue sample to change its image and background color through
inline styles. The example also checks `CSS.supports`, explicit inheritance of
`currentColor`, shorthand resets, and rejection of invalid later setters.

## Syntax and cascade

```css
.panel { background: linear-gradient(to right, #123, #acf) white; }
.corner { background-image: linear-gradient(in srgb to bottom right, red, blue); }
.stripe { background-image: linear-gradient(90deg, currentColor 25% 50%, transparent); }
.offset { background-image: linear-gradient(red calc(2em + 10%), blue 120%); }
```

The default direction is down. Physical sides/corners, finite `deg`, `grad`,
`rad` and `turn` angles, and unitless zero are supported. An optional `in srgb`
may precede or follow the direction. Each value accepts one to sixteen authored
color items, each with zero, one or two positions, expanding to at most thirty-two
stops. A single color produces a solid image. Positions accept supported length
units, signed percentages, zero and bounded additive `calc()`; percentages refer
to the gradient-line length. Omitted positions are distributed between positioned
stops, descending positions clamp forward, and coincident stops form hard edges.

Colors use the existing strict hex, named, `rgb()`/`rgba()` and `hsl()`/`hsla()`
subset, including alpha and `currentColor`. These legacy colors interpolate in
gamma-encoded sRGB under the [CSS Color compatibility rule](https://drafts.csswg.org/css-color-4/#interpolation-space).
Interpolation uses premultiplied alpha. Interpolated fractional channels remain unquantized
through source-over blending until the final RGB8 or existing RGBA16 layer store.
Modern color functions/spaces, interpolation hints, repeating gradient functions,
radial/conic gradients and multiple image layers are unsupported.

Backgrounds are non-inherited unless explicitly requested. Inherited images
share immutable stop storage while `currentColor` resolves on the consuming
element. Declaration validation, variable substitution, `CSS.supports` and inline
CSSOM share the bounded grammar. Invalid ordinary declarations/setters leave
earlier valid values intact; invalid substituted winners use `unset`.
Color-only shorthand resets the image to `none`; image-only shorthand resets the
color to transparent. Mixed CSS-wide longhands have no shorthand serialization;
identical CSS-wide values still serialize as that keyword. Full canonical CSSOM
serialization remains outside this subset.

## Geometry and rendering

The padding box determines the image tile and the border box determines the
painting area. Tiles repeat in both axes, including behind transparent square
borders. This implements the initial origin, clip, size and repeat behavior;
author controls for those properties remain unsupported. Root/body background
propagation considers image presence as well as color alpha. A selected canvas
background extends over the canvas while retaining its root padding-box tile.
The canvas image resolves `currentColor` from the root’s computed color,
including when the image was propagated from the body.

Existing clips, fixed positioning, scrolling, zoom and group opacity apply.
Wrapped inline gradients follow a continuous horizontal strip across existing
left-to-right fragments, with each fragment clipped to its own box. Table row
and cell backgrounds retain their existing ownership and final cell dimensions.
Layout and hit regions do not gain new boxes. Uniform opaque rounded borders
use the existing inner/outer treatment; general translucent or nonuniform rounded
border rings, inline padding/border sizing, bidi, writing modes and table column
background layers retain their existing limitations.

The Vulkan native raster planner explicitly declines a scene containing a
gradient with `unsupported-gradient`, including clipped or opacity-suppressed
commands. The entire scene uses the existing software fallback; a Vulkan
presenter may still upload the resulting CPU frame. No gradient shader or
raster-core primitive is added in this checkpoint.

## Resource limits

Values retain the existing 4,096-byte, calculation-depth and operand limits.
Computed gradient interning has a separate cumulative **4 MiB** allowance per
style computation, charging keys, stops, Arc storage and conservative tree-node
storage before allocation. Duplicate values share storage. Refused retained
storage leaves the computed image at `none`.

Worker snapshots use an appended command tag and a cumulative **8 MiB** retained
gradient allowance. The decoder checks one through thirty-two stops, finite
fields, geometry and the remaining allowance before allocating either Arc.
Repeated sender aliases are charged independently on receipt. This supplements
the existing message, command and image limits.

For a visible gradient with `N` expanded stops and `P` candidate pixels in the
clipped destination loop, Canvas reserves
`4*N + P*(ceil(log2(N)) + 2)` work units before stop resolution or pixel writes.
Rounded coverage and transparent colors do not reduce that reservation. Existing
frame/layer limits still apply. Inline-strip preparation uses the existing layout
work limit and at most 1,600,000 additional temporary payload bytes; refusal omits
the unresolved image while preserving balanced scopes. Existing geometry limits
and the 200,000-command cap are unchanged. These are bounded admission rules,
not a general security or performance result.

## Validation record

The three new self-authored browser references compare interpolation, background
boxes and canvas propagation with literal rectangles. The published predecessor
fails all three; the initial candidate passes all three. The initial direct Page
and confined-worker click tests both pass, checking complete pixels, real hit
dispatch, unchanged node graph and explicit native fallback.

The first focused run records 45 passes and one table-fixture failure: its
`border-spacing:0` prerequisite was unsupported by the existing table layout,
which uses a fixed two-pixel spacing. The corrected fixture derives literal
geometry from that existing rule and checks the color-only case first. The
original test and failure are retained. A separate source review added admission
for the root-padding lookup and corrected mixed-wide shorthand serialization.

The initial size diagnostic records a 32-byte `LinearGradient`, 32-byte
`GradientStop` and 1,024-byte `ComputedStyle` on the tested target. These are type
sizes, not total allocation or performance measurements.

A later primary-source review corrected body-to-canvas propagation: a gradient’s
`currentColor` uses the root’s color, as if the background were specified on the
root. A literal test gives the root and body different colors. The earlier
candidate’s 68/68 references and 2,501/2,325 full-suite passes remain retained
separately from this correction.

The final corrected candidate passes **68/68 references**, **2,502** Rust 1.88
native-feature tests and **2,326** Rust 1.98 default-feature tests. Both full
suites include ignored tests and use four test threads. Strict all-target Clippy
passes on both toolchains, and formatting passes. The actual CI discovery guard
also verifies all **96** bridge tests.

The [evidence summary](evidence/linear-gradients.json) and
[record archive](evidence/linear-gradients.tar.gz) bind the selected sources,
immutable binaries, original attempts, corrections, reviews and final results.
No GPU execution, acquired-window, performance, Web Platform Tests or broad
compatibility claim follows from these checks.
