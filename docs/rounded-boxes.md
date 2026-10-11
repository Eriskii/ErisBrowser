# Rounded box decorations

`DrawCommand::BoxDecoration(Arc<BoxDecoration>)` adds one software painting
command: an outer elliptical shape, optional cutout, and color, gradient or
already decoded image. This foundation adds no CSS `background-origin` or
`background-clip` acceptance or layout emission of the command.

Run the direct example from the repository root:

```sh
cargo run --example rounded-boxes -- OUTPUT.png
```

It renders a fill, unequal-width ring, repeated raster and gradient. The command
performs no fetching, URL resolution or decoding.

`RoundedBox` stores a rectangle and four `(x, y)` radius pairs in top-left,
top-right, bottom-right, bottom-left order. `valid()` bounds finite raw geometry
and endpoints to ±16,000,000, with nonnegative sizes/radii. Zero-size shapes are
inert. Painting uses the supplied radii without normalization and checks every
applicable corner constraint.

`normalized()` explicitly prepares an outer shape using one overlap factor
from all original radii, then makes zero-axis corners square.
`inset([top, right, bottom, left])` normalizes its outer input and subtracts
adjacent thicknesses, floored at zero. Combine border and padding insets before
calling it once for content geometry; repeated insetting would renormalize
truncated inner arcs. See [CSS Backgrounds §4.2](https://drafts.csswg.org/css-backgrounds-3/#corner-shaping)
and [§4.5](https://drafts.csswg.org/css-backgrounds-3/#corner-overlap).

Coverage uses a fixed 4×4 grid at offsets `0.125`, `0.375`, `0.625`, `0.875`,
producing `floor(255 * hits / 16)`. Ring samples must be inside the outer shape
and outside the cutout; independent masks are not multiplied. This new-command
contract is not a CSS-mandated antialiasing algorithm. Existing primitive
coverage and image sampling remain unchanged.

No mask or tile bitmap is allocated. Conservative classification skips full
interior/exterior sampling; ambiguous pixels use sixteen samples. For `P`
candidates, `A` ambiguous pixels and `S` shapes (one outer, two with cutout),
mask work is `8 + 2*P*S + 16*A*S`. Units represent fixed, bounded geometry
queries, not CPU instructions. Both classification passes and samples are
prepaid before painting; candidates can include transparent/nonrepeated holes.
Existing primitive and opacity costs apply additionally. The mask is cleared
on completion/refusal and is not reapplied when opacity groups composite.
Fixed scopes preserve their established clipping.

Tag 13 validates geometry/paint and shares image-key and gradient-stop counters;
old valid command bytes stay unchanged. Primitive, text and Native bridges
refuse the complete frame before payload preparation, after earlier cheap
guards, including after browser zoom. No GPU lowering or timing claim is added.

The first full-suite attempts aborted with stack overflow in the existing deep
layout test after `DrawCommand` grew from 64 to 152 bytes. An `Arc` payload
restores its measured size to 64 bytes. On the measured x86-64 host, each new
payload is 152 bytes plus two reference-count words (16 bytes), excluding
allocator overhead; command clones share it. `Canvas` remains 232 bytes versus
136 in the parent. No quota or test stack was raised.

Source 3 passes 2,755 native-feature tests on Rust 1.88 and 2,517 default tests
on Rust 1.98, each across 33 result rows with no failures or ignored tests. All
44 focused groups and the exact deep-layout regression also pass. The retained
initial focused failure used a clamping fixture helper; a literal rectangle
corrected that setup without changing its expected omission. Strict Clippy
passes for native builds on both versions, default on 1.98 and all Probe targets
on 1.88. Probe tests pass all 15 cases. Both feature configurations discover the
same 138 bridge tests; root/Probe formatting checks pass. The built browser
passes all 76 unchanged references with zero differing pixels or channel error,
and the direct example renders successfully. GPU core/shaders are unchanged
and were not reexecuted for this software-only milestone.

The [summary](evidence/rounded-boxes/summary.json) and
[validation archive](evidence/rounded-boxes/validation.tar.gz) retain the source,
results and unsuccessful attempts. These checks do not establish full CSS
conformance or general allocation, security or performance guarantees.
