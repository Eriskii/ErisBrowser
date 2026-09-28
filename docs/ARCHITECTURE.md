# Engine and process layout

The HTML parser, DOM, CSS cascade, layout, script interpreter, SVG geometry and
software display-list painter are implemented in this repository. HTTP/TLS,
URL/encoding handling, font outlines, image codecs, windowing and OS sandbox
wrappers are infrastructure dependencies. No external browser or JavaScript
engine executes a page.

```mermaid
flowchart LR
    UI[Native UI and painter] <-->|Commands and validated snapshots| R[Confined renderer]
    R -->|Resource request| UI
    UI <-->|Authorized navigation and bounded resources| B[Confined resource broker]
    B --> N[HTTP and TLS]
    B --> F[Explicit local document directory]
    UI -->|One cross-origin encoded image| I[Fresh confined image decoder]
    I -->|Validated pixels only, then exit| UI
    R --> D[HTML and DOM]
    D --> J[Script and DOM events]
    D --> C[CSS and layout]
    J --> C
    C --> S[Display list and decoded images]
    S --> UI
```

The native UI owns address entry, history, clipboard, window events and the final
pixel surface. A document navigation creates a new renderer; its first resource
request starts a separate broker. Both children have cleared environments,
checked inherited descriptors, Linux confinement and resource limits. The
renderer cannot directly open resource files or create sockets. Headless CLI
and library entry points currently use the page pipeline in their calling
process and do not install this confinement.

The parent authorizes the initial URL and optional form body. The broker allows
that document fetch once, validates redirects, and uses the final document URL
as the initiator for later resource requests. The parent remembers that URL and
rejects snapshots which claim a different document. Renderer requests cannot
supply their own initiator or file grant. Network scripts and styles require
the committed origin, including redirects. Cross-origin images use a fresh
decoder process with no file or socket access. The renderer receives pixels and
origin provenance, while the parent withholds the encoded response, headers,
status and final URL. The decoder exits after one image and is never reused
across origins. Same-origin images and inline SVG remain in the renderer.

Within the renderer, `Page` loads resources, parses HTML, runs the supported
script subset and computes styles/layout. The DOM stores HTML, SVG and MathML
namespace identity. Inline SVG is rasterized into the page's image store.
Interaction requests update the same document and interpreter, then return a
new snapshot. A shared DOM control policy governs both native editor focus and
page edits; navigation generations and edit acknowledgements prevent stale
results from replacing newer input.

Snapshots contain the DOM, display list, hit regions, shared raster images and
page metadata. The parent checks their graph structure, namespace bindings,
storage limits, geometry and raster dimensions before publishing them to the
UI. Resource traffic has separate request/response messages. All pipe channels
use bounded framing, nonblocking I/O and deadlines; cancellation remains latched
across nested broker and decoder exchanges. Failure or replacement kills and reaps the
corresponding children.

The painter runs in the UI and draws validated commands with bundled fonts and
a glyph cache. Scrolling reuses the display list; edits currently recompute
styles and layout. Rendering is CPU based, without a GPU compositor or general
incremental invalidation. The existing warm-render benchmark excludes process
startup, resource transfer and snapshot serialization, so it cannot measure the
cost of this complete native path.

See [security](SECURITY.md) for exact grants, limits and remaining attack surface,
[compatibility](COMPATIBILITY.md) for implemented subsets, and
[validation](VALIDATION.md) for observed test results. These boundaries are
implementation facts, not a claim of complete standards support or audited
security.
