# Security boundary

**This is experimental software. It has not had a professional security audit and should not be used for sensitive browsing.** There is no OS renderer sandbox, process isolation, site isolation, comprehensive web-origin model, or complete policy implementation. The page worker is a thread in the browser process. Memory-safe application code and explicit limits reduce some risks; they do not establish production security.

## Implemented controls

- The application forbids unsafe Rust. Parser, layout, interpreter, and software-paint code are custom Rust. Platform/codec/crypto dependencies have their own security surface and may use unsafe code.
- Script code cannot call host filesystem, networking, process, clipboard, native eval, or FFI APIs. Native clipboard access is initiated only by explicit user keyboard shortcuts. Unsupported features produce errors. Scripts have DOM access within the current page.
- The script interpreter bounds source, tokens, AST nesting, expression depth, call depth, instructions and allocation accounting. Per-page script count/source are also capped. Long-lived pages can exhaust its arena; there is no garbage collector. Quota termination is a distinct, uncatchable error and cannot be bypassed by finally blocks.
- HTML and DOM operations bound source/node count, depth, text/attribute storage and mutation. Selectors and style resolution have explicit work limits to contain pathological matching and expansion.
- Layout bounds visits, depth, measured text, output commands and coordinate extents. Paint limits cumulative raster work and rejects invalid geometry and image buffers.
- Network loading checks every redirect, caps redirects, has request/page time budgets, limits resource counts and both encoded and decompressed response sizes. TLS certificate validation stays enabled.
- HTTPS downgrade redirects and HTTPS-to-HTTP subresources are rejected. URL credentials and unsupported schemes are rejected.
- Remote pages cannot read or navigate to local files. Explicitly opened local documents may load files only beneath their canonical parent directory; symlinks escaping that directory are rejected. This is a policy check, not an OS sandbox and not a complete defense against concurrent local filesystem races.
- Styles and scripts are conservatively restricted to the document origin. Script/style MIME types are checked. Images may be cross-origin; there is no credential store.
- When a Content-Security-Policy header or meta policy is present, scripts, author styles and external resources are disabled. This intentionally conservative fallback is **not a CSP implementation**. It rejects content that a compliant browser would permit.
- Decoded raster images have dimension/allocation caps, and the retained decoded-image budget is limited. SVG has separate source, element, geometry, work and image bounds; SVG scripts/external resource references are not executed.
- Page-worker requests and pending snapshots are bounded. Navigation generations prevent stale page results from receiving new user input. Edit acknowledgements keep old results from overwriting newer typing. Password fields are masked and native copy/cut shortcuts do not expose their values.

## Representative limits

| Resource | Bound |
|---|---:|
| Individual fetched/decompressed resource | 8 MiB |
| Aggregate fetched page data | 32 MiB |
| Resource fetches | 48 |
| Redirects per fetch | 8 |
| Encoded form request body | 1 MiB |
| HTTP request timeout | 12 seconds |
| Entire fetch sequence | 30-second page budget, including redirects |
| DOM nodes / parsed nesting | 100,000 / 256 |
| DOM retained text/attribute accounting | 32 MiB |
| Script source / tokens | 256 KiB / 32,768 |
| Script entry instructions / call depth | 100,000 / 48 |
| Script cumulative allocation accounting | 8 MiB |
| Script tags / aggregate script source | 64 / 1 MiB |
| Image dimensions / image decode allocation | 4096 per axis / 64 MiB |
| Retained decoded page images | 64 MiB |
| Headless or window framebuffer | 16 megapixels, 8192 per axis |
| Pending page-worker requests / snapshots | 64 / 1 |

Limits can reject valid documents, and not every allocation is covered by exact accounting. A global hard process memory limit is not implemented. User-supplied public library structures can also be mutated outside the checked APIs; the browser path uses the checked APIs.

## Remaining work

Process separation and an OS sandbox with a narrow broker protocol; complete origin/opaque-origin handling; Fetch/CORS/CSP and navigation policy; cookie/storage partitioning; mixed-content/private-network protection; permissions; verified dependency vulnerability monitoring; continuous coverage-guided fuzzing; sanitizers and cross-platform hardening; independent audit. The [CSP standard](https://www.w3.org/TR/CSP3/) describes substantially more behavior than the fallback above.

Report reproducible issues with a minimized local fixture and command. Avoid including real cookies, passwords, private URLs, or other secrets in reports.
