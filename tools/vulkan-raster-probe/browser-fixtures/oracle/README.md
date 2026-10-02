# Worker-snapshot GPU bridge: independent preparation

This directory is a proposed, unexecuted oracle contract. Root must review the
bridge interfaces and admission policy before implementation or engine execution.

Read `contract.md` for scope and required worker verification, `derivations.md`
for manual arithmetic, and `fixtures.json` for exact definitions and literal rows.
`expected/*.rgb` is row-major RGB without a header; packed target hashes use
little-endian 0x00RRGGBB words. `prepare.py` only serializes these authored inputs.

| Case | Input | Expected path |
| --- | --- | --- |
| direct-fixed-escape-and-nested-restoration | direct-draw-command | gpu |
| direct-translucent-fractional-clip-edges | direct-draw-command | gpu |
| direct-repeated-key-alias-and-alpha | direct-draw-command | gpu |
| direct-missing-image-is-a-legal-noop | direct-draw-command | gpu |
| direct-lines-are-bounding-rectangles | direct-draw-command | gpu |
| direct-hidden-text-refuses-whole-frame | direct-draw-command | cpu-fallback |
| direct-rounded-refuses-whole-frame | direct-draw-command | cpu-fallback |
| direct-unit-opacity-refuses-whole-frame | direct-draw-command | cpu-fallback |
| direct-transparent-hidden-rounded-still-refuses | direct-draw-command | cpu-fallback |
| html-ordered-rectangles | worker-html | gpu |
| html-fixed-child-escapes-scrolling-clip | worker-html | gpu |
| html-repeated-decoded-image-key | worker-html | gpu |
| html-unavailable-image-keeps-placeholder | worker-html | gpu |
| html-rounded-page-requires-full-cpu-fallback | worker-html | cpu-fallback |
| direct-command-cap-refuses-whole-frame | direct-draw-command | cpu-fallback |
| direct-scope-cap-refuses-balanced-whole-frame | direct-draw-command | cpu-fallback |

Totals: 16 cases, 11 direct lists, five HTML documents, nine GPU admissions,
seven complete CPU fallbacks; 197 literal pixels / 591 RGB bytes / 788 packed bytes.

No browser, planner, CPU painter, shader compiler or GPU execution has occurred.
The existing 30 probe fixtures are referenced unchanged in `data-checks.json`.
HTML assumptions are source-derived and remain explicitly unverified.
