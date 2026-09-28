# Document URLs and stylesheet imports

The document retains its authoritative navigation URL separately from the first
connected HTML `base[href]` element's frozen URL. Template contents and detached
nodes do not select the document base. Invalid, `data:` and `javascript:` base
values fall back to the document URL; a later base does not replace an invalid
first one. Initial URL and encoding setup precedes resource loading and scripts.
Supported DOM mutations update the frozen value, while fragment navigation keeps
it. JavaScript exposes readonly `document.URL`, `document.documentURI` and
`document.baseURI`. Snapshot transfer preserves and validates the selected base
node and frozen URL. Legacy document encodings affect URL queries through the URL
library's encoding override.

Resolution follows the [HTML base element algorithms](https://html.spec.whatwg.org/multipage/semantics.html#the-base-element).
The base affects resource and navigation resolution; it does not grant an origin
or a local directory. Resource requests still use the committed document URL as
their initiator. The broker independently enforces that authority, including
redirects. A remote base cannot authorize cross-origin active resources, and a
fragment reference resolved through a base cannot bypass scheme or file checks.
Missing or empty form actions use the document URL.

During `Page::load`, linked and inline stylesheets can import other stylesheets
with leading `@import` string or `url()` rules. The scanner handles comments,
quoted strings and CSS escapes, and bounds delimiter nesting. Imported URLs use
the importing response's final URL. Each sheet chooses its encoding from its
own BOM/transport/charset information, falling back to its parent's encoding.
After decoding, CSS URLs use UTF-8 percent encoding regardless of the sheet's
byte encoding, following the [CSS URL processing model](https://drafts.csswg.org/css-values-4/#url-processing).
Decoded source caching includes that fallback encoding. Fetch permissions remain
those of the document throughout the import graph.

Import traversal preserves cascade positions, repeated imports and conditional
media metadata. Repeated resources share decoded source; each occurrence still
costs scan work. Current recursion-path URLs, including redirect destinations,
stop cycles. Each imported sheet remains a separate parser input so malformed
EOF comments, strings or blocks cannot consume its parent's rules. Media
conditions are evaluated during style computation, so resizing reevaluates them.
This implements part of [CSS stylesheet imports](https://drafts.csswg.org/css-cascade-5/#at-import),
not the full CSS grammar or loading model.

`@import` supports both `layer(name.path)` and the anonymous `layer` keyword.
The loader retains one anonymous identity for every source segment belonging to
the same imported layer. Layer order statements before imports remain before
them in the cascade inventory. A valid layered import registers its layer even
when fetching fails, subject to its media and supports conditions. Layers and media contexts
are carried in `StyleSource` metadata, including empty registration sources;
they are never serialized into invented author-visible names or CSS wrappers.
Nested imports inherit both the parent layer and media conjunction. See
[cascade behavior](../tests/conformance/cascade-layers.md).

An optional `supports(...)` clause follows the optional layer clause and precedes media conditions. It accepts either a full supports condition or a bare declaration. A false, invalid or over-budget condition skips the import before fetching or registering a layer, even if that URL is already cached. Static capability tests need no retained resize metadata. True imports retain media conditions and nested source scopes. The loader charges condition evaluation against an additional shared 8 MiB work limit; see [feature-query syntax and bounds](../tests/conformance/supports.md).

Media evaluation implements a bounded subset of [Media Queries 4](https://www.w3.org/TR/mediaqueries-4/#mq-syntax):
comma-separated alternatives, media types, `only`/`not`, grouped conditions
with `and`, `or` and `not`, escaped identifiers, and width/height range comparisons. Mixed operators require explicit grouping. `screen` and `all`
match; `print` and unknown media types do not. Unknown types are false, so
`not bogus` matches. Unknown features and unsupported feature values retain the
specification's unknown truth value through negation: neither
`(unknown-feature)` nor `not (unknown-feature)` matches. Malformed alternatives
do not match; the scanner can resume at a following top-level comma.

Supported dimensions are `width`/`height` and their `min-`/`max-` forms, using
supported CSS length units and unitless zero. Percentages, `auto`, nonzero
unitless values and failed length parses do not match. Relative font units use
the initial 16px font size. Orientation treats a square viewport as portrait.
Boolean color, hover and pointer features are supported. Reported environment
values are fixed: light color scheme, reduced motion, hover, a fine pointer and
browser display mode. They are not measurements of operating-system preferences
or attached input devices. `calc()`, aspect ratio/resolution and the full media-feature inventory remain unsupported. See [media-query grammar and limitations](../tests/conformance/media-queries.md).

Media conditions require a structurally valid prelude: strings,
comments and delimiters must close, and unquoted braces or semicolons are
rejected. Quoted or escaped delimiter characters remain data. Conditional sheet
contents remain independent parser inputs; malformed closing braces cannot
change their metadata conditions or layer identity. Imported URL tokens retain literal braces and semicolons;
their contents are not mistaken for import-rule boundaries.

Per-load bounds are 8 MiB of decoded stylesheet text, 8 MiB of constructed text,
32 MiB of repeated scan input, 256 import attempts, 256 cached resources, 256
retained segments and 16 recursive external-sheet levels. Parsing an import
prelude or validating a media condition permits at most 128 nested delimiters.
Media evaluation separately permits 64 KiB of input, 64 comma-separated queries,
64 leaf terms across the list, 16 nested delimiters and 2 MiB of local work; stylesheet evaluation also charges the shared 32 MiB parse allowance. Stylesheet URL work
and retained URL cache data each have a 32 MiB allowance, including repeated
cache lookups and failed-resource keys. Page resource resolution has its own
32 MiB work limit. Page-wide fetch count/body limits and the final
8 MiB/256-style-input cascade limit also apply. New layer-name and media text is charged before
allocation; import diagnostics have a bounded inventory. Structured inputs allow
at most 32 media conditions per source under the shared 8 MiB input budget.

Inline collection validates `type` and media syntax before copying text, both
while loading and during later style recalculation. HTML and SVG `style`
elements use only their direct Text children; descendant element text does not
become stylesheet source. This follows [HTML style processing](https://html.spec.whatwg.org/multipage/semantics.html#the-style-element),
which [SVG 2 also adopts](https://www.w3.org/TR/SVG2/styling.html#StyleElement).
The type must be absent, empty, or an ASCII case-insensitive `text/css` match;
surrounding spaces and MIME parameters make a style element inert. Linked
stylesheet type hints retain their separate MIME processing. JavaScript
`textContent` continues to concatenate descendant text.

Each collection accepts at most 256 inline sheets and 8 MiB of copied text.
It reserves up to 200,000 direct-child visits across sizing and copying passes,
checks the complete output size before allocation, and uses fallible reservation.
Initial loading, re-layout and the raw `Document::stylesheets` helper all use
these bounds. The first exhausted inline collection stops further copying;
initial loading records a diagnostic and can still process other resource kinds.
The separate final segment/media budget above still applies.

Remaining limitations include complete feature-query support,
CSS Syntax recovery and namespace semantics, asynchronous/parser-driven
loading, dynamic stylesheet fetching, CSSOM, alternate stylesheet sets and full
CORS/CSP. False and invalid import supports clauses do not fetch. Non-CSS style types are inert. Inline DOM text changes invalidate the
cached expanded source, but do not start new import fetches. `Page::from_html`
uses the supplied text without fetching external resources. Nested browsing
contexts, srcdoc/about-base fallback, base targets and complete URL-reflecting DOM
properties are not implemented.
