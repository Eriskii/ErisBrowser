# Image fitting and positioning

Loaded `<img>` content supports `object-fit` and physical `object-position`.
The element keeps its existing content box, padding, border, layout flow and
hit area; only the image’s painted destination changes. This includes external
SVG files already decoded into the image store’s raster form.

```sh
./run.sh ./examples/object-fit-position.html
```

The example compares nine image boxes using one nonsquare PNG. Click the first
image to change its fit and position through inline style setters. It also
checks that invalid later setters leave the saved valid styles intact.

| Fit | Painted size |
|---|---|
| `fill` | Content-box dimensions; the initial value |
| `contain` | Largest natural-aspect-ratio image that fits inside the box |
| `cover` | Smallest natural-aspect-ratio image that covers the box |
| `none` | Natural image dimensions |
| `scale-down`, `contain scale-down`, `scale-down contain` | Smaller of natural size and contain size |
| `cover scale-down`, `scale-down cover` | Smaller of natural size and cover size |

The natural dimensions come from the existing loaded-image metadata, separately
from HTML width/height attributes used for box sizing. That metadata remains
DOM-visible and mutable; this change introduces no new decoder trust boundary.

`object-position` starts at `50% 50%`. It accepts physical keywords, supported
length units, zero, signed percentages and bounded additive `calc()` sums and
differences. One, two and four components are supported. For example:

```css
img { object-fit: contain; object-position: top; }
img.badge { object-fit: none; object-position: 25% bottom; }
img.crop { object-fit: cover; object-position: right calc(25% + 2px) bottom -5px; }
```

Percentages resolve against the signed space remaining after fitting: the
content-box size minus the painted image size. Cover can make that space
negative. Four-component right/bottom offsets measure back from the end edge.
Two-keyword forms can reorder their axes (`top left`); a length/percentage pair
uses horizontal then vertical order. `top 20%`, `20% left`, three-component
forms, `auto`, nonzero unitless lengths and nonfinite inputs are rejected.

Both properties are non-inherited, with explicit `inherit` and other CSS-wide
keywords supported. Declaration acceptance, computed variable substitution,
`CSS.supports` and inline CSSOM use shared bounded validation. An invalid later
ordinary declaration or setter does not erase a valid earlier declaration.
An invalid value after variable substitution uses `unset`, without restoring
an earlier cascade winner. CSSOM retains its existing authored-value
serialization format; it does not promise complete canonical serialization.

Cropping emits the full image destination inside a content-box clip. Intersecting
and resizing the destination would sample a different part of the source.
Uncovered areas retain the element’s background, and existing alpha blending
continues to apply. Default fill also gains a clip when fractional origins would
otherwise let the sampling loop paint before the content edge. Integer-contained
default destinations keep their original single-image command.

Fitting uses bounded scalar arithmetic and adds no per-image heap fields.
Nondefault fitted destinations must have finite origins within ±1,000,000 and
positive finite extents at most 1,000,000, still positive in the command’s f32
representation. If they cannot be represented, image paint is omitted while
layout and hits remain. Required clips consume existing command and downstream
native admission capacity; limits are not raised. A refused clip never exposes
an unclipped image. Missing-image fallback, inline SVG and video/poster paths
retain their existing behavior. Logical positions, general CSS math, full CSS
Images/SVG conformance, new rendering backends and performance claims are outside
this increment.

The published predecessor passes 0/5 new reference cases using their original
frozen expressions. The first candidate run passes 61/62 total cases: its one
failure is twenty pixels in the combined scale-down reference. An independent
comparison finds all 76,800 candidate pixels equal to the original thirty
ordered literal rectangles. Rendering that reference through a whole SVG image
exposes an existing identity-sampling precision bug: `f32(125/240)*240` falls
below 125 and truncates to source row 124.

Only that reference expression changes, to the same ordered CSS rectangles.
The unchanged candidate binary then passes **62/62**. The original reference,
first failure, pixel oracle and correction are retained; this is not a claim
that the final reference bytes are identical to the predecessor run. The
underlying image-sampling bug remains queued for a separate fix. These are
self-authored references, not Web Platform Tests.

The final Rust 1.88 native-feature suite passes **2,441 tests**, including the
Page and confined-worker click checks and the loaded alpha/external-SVG case.
The Rust 1.98 default-feature suite passes **2,271 tests**. Both runs include
ignored integration tests and use four test threads. Strict Clippy passes on
both versions, and the formatting check passes. The first native run retained
2,092 passes and one Date host-EOF helper failure before stopping; that exact
test passes in isolation, and the complete rerun passes without engine or Date
test changes. The first failure remains evidence, not an inferred diagnosis of
its host conditions.

The size diagnostic reports `ObjectFit` at 1 byte, a position component at 16,
`ObjectPosition` at 32, and `ComputedStyle` at 1,016 bytes on the tested target.
No parent size delta was measured. The [evidence summary](evidence/image-fitting.json)
and [record archive](evidence/image-fitting.tar.gz) bind the source, raw attempts,
reference correction, reviews and final observations. The source inventory’s
reference-only revision is distinct from the unchanged compiled engine. No
acquired-window, GPU execution or performance claim follows from these checks.
