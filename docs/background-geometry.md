# Background size, position and repetition

Single-layer linear-gradient backgrounds now accept `background-size`, physical
`background-position` and independent repeat axes. They retain the existing
[gradient color and stop subset](linear-gradients.md) and
[bounded Native preparation](vulkan-gradients.md).

```sh
./run.sh ./examples/background-geometry.html
```

The example shows positioned stripes, percentage and edge offsets, oversized
images, independent repeat axes, auto dimensions, calculated sizes and padding.
Click the first stripe to change all three geometry properties through inline
styles; malformed later setters preserve those values. Another sample uses an
image-only shorthand to reset its earlier geometry.

## Supported values

| Property | Supported subset | Initial value |
|---|---|---|
| `background-size` | One or two `auto`/length/percentage values, bounded additive `calc()`, `cover`, `contain` | `auto auto` |
| `background-position` | Valid physical one-, two-, three- and four-component combinations of edges, `center`, lengths, percentages and bounded additive `calc()` | `0% 0%` |
| `background-repeat` | `repeat`, `no-repeat`, `repeat-x`, `repeat-y`, and the four two-axis repeat/no-repeat pairs | `repeat repeat` |

```css
.badge {
  background: linear-gradient(to right, red 50%, blue 50%)
              center / 4px 2px no-repeat white;
}
.edge { background-position: right 2px bottom; }
.inset { background-position: bottom 1px right 2px; }
.stripe { background-size: calc(50% - 2px) 2px; background-repeat: repeat-x; }
```

The shorthand accepts the supported image/color subset together with position,
optional `/ size`, and repetition. Omitted components reset all five supported
longhands to their initial values. A malformed or unsupported ordinary declaration
or setter is rejected as a whole. Variable substitution, cascade, `CSS.supports`
and inline CSSOM share validation. Invalid substituted winners keep the existing
`unset` behavior. These properties are non-inherited unless explicitly inherited;
CSS-wide values and the existing layer handling apply. CSSOM emits parseable
supported shorthand values; complete canonical serialization remains outside
this subset. The separate `object-position` grammar is unchanged.
A shorthand getter requires all five supported longhand declarations in the
same inline declaration block; stylesheet rules and computed initial values do
not supply missing inline records. See the
[CSSOM getter algorithm](https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-getpropertyvalue).

## Used geometry and pixels

The positioning area remains the padding box, with the border box as the painting
area. Size resolves before position. Percent sizes use the positioning-area
axis; position percentages use the signed difference between that area and the
used image size. An oversized image therefore has negative free space. For a
12 by 6 area and a 4 by 2 image, center places its origin at `(4,2)`, while
`25% 75%` places it at `(2,3)`.

Gradients have no natural dimensions or aspect ratio. Each `auto` axis fills
that positioning-area axis; `cover`, `contain` and `auto auto` all fill the
area for this image family. A literal negative size is invalid. Calculated sizes
preserve percentage dependency: pure-length calculations clamp at computed time,
and percentage-containing calculations clamp after their used basis is known.
A zero used width or height suppresses the image while retaining background
color, layout and hit regions.

A nonrepeating axis includes samples whose pixel centers lie within the tile's
half-open interval. It does not replace the original rounded painting outline
with a tile rectangle or introduce a fractional clip. Repeating axes retain the
same tile phase across the painting area. With the centered 4 by 2 red/blue hard
tile above, `no-repeat` gives `WWWWRRBBWWWW` only on rows 2 and 3; `repeat-x`
gives `RRBBRRBBRRBB` on those rows; `repeat-y` gives the centered strip on every
row. Here `W` is the underlying white color.

Propagated body backgrounds carry size, position and repeat together with the
image, using the root padding-box geometry and root-used `currentColor`. Wrapped
inline backgrounds resolve against the complete virtual strip before slicing.
Table rows and stretched cells use their final dimensions; flex/grid reflow
likewise resolves the final tile. Scrolling, fixed scopes, zoom, background colors
and existing hit ownership retain their existing paths.

The rounded-border shortcut requires a guaranteed opaque interior. An opaque
background color still provides that guarantee; a gradient alone uses it only
with default size/position and repeat on both axes. No-repeat or omitted images
therefore leave transparent interior holes. Existing general translucent or
nonuniform rounded-border limitations, inline box sizing, bidi and table-column
background limitations remain.

## Transport, Native admission and limits

Default repeat/repeat commands retain the previous worker tag 10 and exact
payload shape. Other repeat combinations use tag 11 with one two-bit mask byte;
unknown mask bits are rejected before geometry or stop allocation. Both forms
retain the same geometry checks and cumulative 8 MiB retained-gradient allowance.
Zoom preserves the flags while scaling the original paint/tile geometry and stops.

Native eligibility is unchanged: opaque stops, square corners and no retained
opacity scope. The shared sampler prepares only the nonempty visible fragment
and retains its absolute framebuffer origin. Other gradient cases require the
existing whole-scene fallback. Gradient interpolation and fragment generation
remain CPU work; the existing GPU image shader draws the prepared pixels.

For `N` expanded stops and `P` candidate pixels, default repetition retains
`4*N + P*(ceil(log2(N)) + 2)` evaluation work. A nonempty window with `D` disabled
repeat axes reserves `16 + 4*N + P*(ceil(log2(N)) + 2 + D)` before sampling or
pixel mutation. `P` is the filtered integer window for that case. Native adds
one unit for each generated pixel as before. Empty windows generate no source.
Loaded and generated images still share the 1 MiB source-byte allowance and
256 source slots; all existing scene, frame and work caps remain.

CSS values retain the 4,096-byte and calculation depth/operand bounds, and gradient
interning retains its 4 MiB allowance. Geometry adds fixed-size style fields and
no per-tile allocation or tile enumeration. Layout resolves coefficients in f64
and omits nonfinite, zero or out-of-extent tiles instead of distorting them with
independent coordinate clamps. Fragment placement also checks the final translated
tile origin: a locally valid tile that moves beyond the extent is omitted rather
than clamped, preserving repetition phase for admitted tiles. Existing layout
preparation and command limits remain.

`space`/`round`, author `background-origin`/`background-clip`/`background-attachment`,
URL backgrounds, multiple layers, axis-specific position properties and logical
positions are deferred. This increment does not broaden the gradient syntax,
color spaces or Native alpha/rounded/opacity support.

The primary geometry rules are in CSS Backgrounds 3:
[size](https://drafts.csswg.org/css-backgrounds-3/#the-background-size),
[position](https://drafts.csswg.org/css-backgrounds-3/#the-background-position),
[repeat](https://drafts.csswg.org/css-backgrounds-3/#the-background-repeat),
[shorthand](https://drafts.csswg.org/css-backgrounds-3/#the-background) and
[canvas propagation](https://drafts.csswg.org/css-backgrounds-3/#special-backgrounds).
Gradient natural sizing is described by
[CSS Images 3](https://drafts.csswg.org/css-images-3/#natural-dimensions), and
calculated range handling by
[CSS Values 4](https://drafts.csswg.org/css-values-4/#calc-range).

## Validation record

The initial full-suite attempts reached the existing linear-gradient click
fixture and exposed its assumption that two inline color/image declarations
could serialize a complete background shorthand. The implementation now supports
five longhands, so that incomplete block correctly returns an empty string.
The fixture migration initializes `background:none` before its valid color/image
setters, explicitly supplying the geometry defaults; its malformed-setter checks
and literal paint expectations remain. A separate regression covers partial
blocks, each missing geometry longhand, and complete initialization. Original
fixture bytes and failed attempts are retained.

A separate source review found that fragment translation could clamp a tile
accepted in local coordinates and shift its repeated phase. The correction checks
the translated origin and omits out-of-range tiles. Literal tests cover signed
boundaries, ordinary and pending tiles, and table placement with unchanged hits.
The original source hold and the additive correction are retained.

Initializing all five longhands then exposed a work-limit stop in the old click
fixture. CSSOM was validating ordinary `background-image`, `background-size`,
`background-position` and `background-repeat` values twice. After complete first
validation and the existing storage prepayment, those four normalized longhands
now publish the same records directly, avoiding the duplicate parser pass and
intermediate declaration copy. Shorthands, variables, normalization and all
remaining charges keep their existing paths; the author budget stays at
100,000 work units. This establishes a reduction in charged work, not an
elapsed-time speedup.

On held source 5, the focused Rust 1.88 direct Page and real confined-worker
linear-gradient click cases both pass. The focused Rust 1.98 background CSSOM
suite passes all eight groups, including exact/one-short admission tests and
normalization, priority, pending-value and missing-longhand cases. The old click
recovers within the unchanged budget. These focused results do not stand in for
the complete test roster.

Retained unsuccessful attempts include compiler/setup and invocation errors,
partially completed suites, and a new test's expected authored capitalization
corrected to the existing lowercase CSSOM serialization. Their partial totals
are not presented as full-suite results.

The complete final-source Rust suites pass **2,568 native-feature tests on
Rust 1.88** and **2,370 default-feature tests on Rust 1.98**. Each produces thirty
result groups with zero failed or ignored tests, using `--include-ignored` and
four test threads. Strict all-target Clippy passes for native features on both
toolchains and default features on Rust 1.98; the formatting check also passes.

All **71 self-authored browser references** pass with zero differing pixels and
zero maximum channel error, using the immutable browser built from the same
final source. The previous 68 cases remain in place and the three additions
cover position, repetition and fractional geometry. The probe's browser-bridge
suite passes **15 tests across twelve result groups**, and its strict all-target
Clippy check passes. The bridge discovery guard retains all **107** tests.
Direct Page and real confined-worker cases are included in the complete suites;
eligible packed Native records are checked on the CPU.

The separately published parent `c9231785` completed all nine CI jobs successfully.
That parent CI record is retained separately from this increment's local checks.
No new GPU execution, acquired-window, timing, broad conformance or security
claim follows from these results.

The [evidence summary](evidence/background-geometry.json) and
[record archive](evidence/background-geometry.tar.gz) retain selected sources,
immutable browser provenance, literal oracles, failed attempts, corrections,
reviews and final results.
