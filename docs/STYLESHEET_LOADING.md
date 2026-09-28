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
media wrappers. Repeated resources share decoded source; each occurrence still
costs scan work. Current recursion-path URLs, including redirect destinations,
stop cycles. Each imported sheet remains a separate parser input so malformed
EOF comments, strings or blocks cannot consume its parent's rules. Media
conditions are evaluated during style computation, so resizing reevaluates them.
This implements part of [CSS stylesheet imports](https://drafts.csswg.org/css-cascade-5/#at-import),
not the full CSS grammar or loading model.

Media evaluation implements a bounded subset of [Media Queries 4](https://www.w3.org/TR/mediaqueries-4/#mq-syntax):
comma-separated alternatives, media types, `only`/`not`, plain feature
conjunctions with `and`, and negation of a single feature. `screen` and `all`
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
or attached input devices. Range comparisons, `or`, grouped Boolean conditions,
`calc()`, escaped media identifiers and the full media-feature inventory remain
unsupported.

Generated media wrappers require a structurally valid condition: strings,
comments and delimiters must close, and unquoted braces or semicolons are
rejected. Quoted or escaped delimiter characters remain data. Conditional sheet
source with an unmatched closing brace is conservatively ignored so it cannot
escape its wrapper. This guard is intentionally stricter than complete CSS
Syntax recovery. Imported URL tokens retain literal braces and semicolons;
their contents are not mistaken for import-rule boundaries.

Per-load bounds are 8 MiB of decoded stylesheet text, 8 MiB of constructed text,
32 MiB of repeated scan input, 256 import attempts, 256 cached resources, 256
retained segments and 16 recursive external-sheet levels. Parsing an import
prelude or validating a wrapper condition permits at most 128 nested delimiters.
Media evaluation separately permits 64 KiB of input, 64 comma-separated queries,
64 conjunctive features per query and 16 nested delimiters. Stylesheet URL work
and retained URL cache data each have a 32 MiB allowance, including repeated
cache lookups and failed-resource keys. Page resource resolution has its own
32 MiB work limit. Page-wide fetch count/body limits and the final
8 MiB/256-style-input cascade limit also apply. Media wrapping is charged before
allocation; import diagnostics have a bounded inventory.

Remaining limitations include cascade layers, `supports()` import conditions,
complete CSS Syntax recovery and namespace semantics, asynchronous/parser-driven
loading, dynamic stylesheet fetching, CSSOM, alternate stylesheet sets and full
CORS/CSP. Unsupported import layer/supports clauses produce a diagnostic and do
not fetch. Non-CSS style types are inert. Inline DOM text changes invalidate the
cached expanded source, but do not start new import fetches. `Page::from_html`
uses the supplied text without fetching external resources. Nested browsing
contexts, srcdoc/about-base fallback, base targets and complete URL-reflecting DOM
properties are not implemented.
