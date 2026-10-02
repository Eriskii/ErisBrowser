# Independent pixel derivations

`prepare.py` serializes authored commands and separately authored literal rows.
Its only target construction is palette lookup and row concatenation: no command
interpreter, layout, clipping implementation or candidate import is used. PNG
generation only encodes the authored two source pixels using PNG chunks and zlib.
The RGB grids, not output from any renderer, are the oracle.

- **Fixed escape and restoration:** document offset (0,-1) moves the first red
  rectangle to x=1..4, y=0..2. Its nested clip is x=2, y=0..1. Fixed offset (1,1)
  resets to caller clip x=1..4, letting blue reach (1,1) outside that nested clip.
  A nested fixed reset lets yellow reach (3,2),(4,2). Popping it restores the
  x=2,y=1 clip for magenta. Popping the outer fixed scope restores document space
  and the first nested clip: cyan reaches (2,0). After popping that clip, black
  reaches (4,2), not a fixed-space position. Final rows are WRCRRW / WBMRRW /
  WRRYKW / WWWWWW. Green was intentionally overwritten completely.
- **Fractional clip:** integer pixel origins must satisfy x≥0.5, x<3.5, y≥0.5,
  y<2.0, so only x=1,2,3 at y=1 can blend. Red alpha 128 over white gives
  (255,127,127). The outer rows and x=0,4 remain white.
- **Repeated source alpha:** integer source-over is
  `(source*a + destination*(255-a) + 127) / 255`, using integer division each
  pass. Source (200,20,40), alpha 128 over (32,64,96) yields (116,42,68); repeating
  yields (158,31,54). The first source pixel maps to x=0,1 for a 2→4 scale.
  The second pixel's alpha zero preserves (32,64,96), regardless of hidden RGB.
  The alias row draws once, so rows are DDTT / AATT.
- **Missing direct image:** it paints nothing. The red x=0,1 columns and final
  blue (3,1) remain; image-key absence does not prevent later commands.
- **Lines:** the reverse diagonal (4,2)→(1,0), width 1, is the rectangle [1,0,3,2].
  The horizontal (0,3)→(4,3) is [0,3,4,1]. The zero-length line at (0,0), width 2,
  is [0,0,2,2], overwriting the upper-left blue column. These are filled bounding
  rectangles, not antialiased line segments.
- **Hidden text:** the empty typed clip causes CPU text painting to do no work;
  the later blue square still paints. The GPU admission must reject the text
  before considering invisibility. No font rasterization contributes to the grid.
- **Rounded CPU fallback:** a 2×2 radius-one blue rectangle has four samples with
  dx=dy=0.5. Coverage is trunc(255*(1.5−sqrt(0.5))) = 202. A narrow elementary
  interval 0.7071067<sqrt(0.5)<0.7071069 places the pre-truncation coverage strictly
  between 202.1877 and 202.1878, far from a byte boundary even with f32 rounding.
  Alpha 255 preserves that coverage; blue over white gives (53,53,255), hex
  3535ff. The separate red bottom-left pixel verifies the rest of the CPU frame.
- **Opacity-one fallback:** CPU opacity one creates no isolated blending layer;
  blue background, top-left two red pixels and bottom-right green are literal.
  The first bridge deliberately rejects every opacity scope, so this remains a
  fallback test without a font or floating group-opacity oracle dependency.
- **Transparent hidden rounded input:** the CPU skips the zero-alpha rectangle;
  the bridge must still reject radius three before invisibility/alpha elision.
  The red first and blue last pixels remain in the complete CPU result.
- **HTML rectangle order:** blue [3,0,2,3] paints after red [1,1,4,2], yielding
  WWWBBW / WRRBBW / WRRBBW / WWWWWW over white.
- **HTML fixed child:** document offset (0,-1) moves the red clip to [1,0,2,2]
  and green to [2,1,2,2], leaving only green (2,1). Fixed blue remains [5,0,2,1]
  and escapes that ancestor clip. The body background/clear is white elsewhere.
- **HTML images:** source red is opaque; blue alpha 128 over white gives
  (127,127,255). Nearest-neighbor doubling makes RRHH in each image. They occupy
  x=0..3 at y=0 and x=2..5 at y=1. The PNG has no gamma/color-profile chunks.
- **HTML missing image:** current layout emits an opaque RGB(236,238,242)
  placeholder at [1,0,2,1], then a legal Image command for the missing key. The
  bridge skips that command, not the placeholder. Empty alt prevents text.
- **HTML rounded:** the same [1,0,2,2], radius-one geometry as the direct rounded
  case supplies the same four 3535ff pixels; red remains at (0,2).
- **Limits:** 257 transparent zero-size rectangles exceed the unchanged 256
  command cap even though each has no CPU pixels. Thirty-three balanced fixed
  pushes/pop pairs exceed GPU depth 32 while staying below CPU depth 128. Both
  complete CPU results are a single white pixel with no paint exhaustion.

The five HTML cases intentionally freeze assumptions, not observed command
lists. Required command/image checks are attached to each fixture and explained
in `contract.md`. Absolute positioning uses explicit small containing-block
dimensions, no body margin or borders, empty element content and explicit image
dimensions. These choices avoid text metrics and shrink-to-fit dependencies.
