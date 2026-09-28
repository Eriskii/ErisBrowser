# Original test fixtures

`css-supports.html` uses script feature queries to select a green initial box
and blue clicked box. Pipeline and confined-worker tests compare the visible
result and reject injected property/value syntax. It loads no external assets.

`custom-properties.html` checks that an inherited custom-property alias keeps
the parent's computed color despite a child override, then recomputes after a
click changes the parent's inline declaration. It loads no external assets.

`inline-style.html` checks six green initial samples and blue clicked samples
through both the direct page and confined worker. Its script checks declaration
priorities, invalid-write preservation, case-sensitive custom names, quoted
delimiters, property aliases, removal, null values and reentrant string
conversion. It loads no external assets.

`default-parameters.html` checks six green initial samples and blue samples
after an event callback. Script assertions cover omitted arguments, temporal
dead zones, body/parameter scope separation, unmapped arguments, lexical arrows,
default function names and length. Page and confined-worker tests check pixels.

`rest-parameters.html` checks dense fresh arrays, unmapped arguments, default/rest
scope and temporal dead zones, lexical arrows, methods, bound construction,
apply and event callback arguments. Page and confined-worker tests check six
green initial samples and blue clicked samples. It loads no external assets.

`calc-resize.html` uses mixed length-percentage widths and margins. Page and
confined-worker tests verify exact geometry and pixels at 320 and 520 pixels,
then repeat after CSSOM mutation, including a resize back to the smaller viewport.
Both fixtures are original and load no external assets.

## Synthetic image allocation fixtures

These original fixtures are 1×1 RGBA PNGs, with valid chunk CRCs and 4,096 bytes
of synthetic ancillary metadata. They contain no external image or profile data.

- `png-ancillary-icc-4k.png`: compressed `iCCP` payload. The pinned PNG decoder
  discards this optional metadata when its decompression exceeds the limit.
- `png-ancillary-exif-4k.png`: uncompressed `eXIf` payload. The pinned decoder
  rejects construction when this metadata exceeds its allocation limit.

The page tests exercise the production decoder-construction helper with small
limits, so the test detects late application of limits without large allocations.

The original WebP fixtures exercise the uncompressed frame-header preflight:

- `webp-oversized-inner-vp8.webp`: 48 bytes containing a 1×1 extended canvas
  and a truncated VP8 keyframe header declaring 16,383×16,383 pixels. It has no
  compressed image data. The pinned decoder otherwise allocates frame planes
  before rejecting it; tests require rejection before decoder construction.
- `webp-lossy-2x2.webp`: a 2×2 solid `#2468ac` image encoded with ImageMagick
  (`convert -size 2x2 xc:'#2468ac' -quality 80 webp-lossy-2x2.webp`). It checks
  that ordinary lossy WebP still decodes. Tests also encode original lossless
  pixels and wrap them in extended and animated containers.

These checks validate container framing and dimensions, not compressed WebP
bitstream conformance or an exact bound on every codec scratch allocation.
