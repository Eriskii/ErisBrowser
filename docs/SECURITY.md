# Security boundary

**This is experimental software. It has not had a professional security audit and should not be used for sensitive browsing.** Native browsing uses a fresh confined Linux renderer per document and a separate resource broker when resources are needed. This is a concrete isolation layer, not evidence of production security, complete site isolation, or a complete web-origin model. The headless CLI, benchmarks, conformance adapters and public library API do not install this sandbox.

## Native process boundary

The UI retains the window, address bar, clipboard, trusted bundled fonts and software painter. A renderer child runs HTML/CSS parsing, scripting, same-origin image decoding and layout. A separate broker child performs resource loading; the parent relays its bounded responses and records the broker's final document URL. Each cross-origin image is decoded in a fresh third child which exits after one response. All children start with a cleared environment and reject inherited descriptors other than stdin/stdout/stderr and the temporary procfs inspection handle. Inspection failure rejects startup.

All children require fully enforced Landlock ABI 6 and `no_new_privs` before processing page data. **The renderer has no filesystem read/write/execute grants and cannot create any socket.** TCP connect/bind is also denied by its Landlock policy. It receives document and authorized resource bytes only through the parent-controlled pipe. Cross-origin image decoders use the same zero-file-grant, zero-socket policy. Metadata observations such as some pathname/stat operations are not fully mediated by these filesystem access rules.

The broker grants read access to the explicitly opened local document's canonical parent directory (if any), exact resolver files, and system library trees (`/lib`, `/lib64`, `/usr/lib`, `/nix/store`). It grants no filesystem write or execute rights, and remote documents receive no local directory grant. General outbound IP access remains available to this process for DNS and HTTP/TLS; TCP listeners and Unix sockets are denied. Broker compromise is therefore a different, broader boundary than renderer compromise. The broker does not parse HTML, run scripts, or decode images.

A shared seccomp denylist blocks process/thread creation, new executables, io_uring, namespace/process-group changes, socket pairs, System V shared memory/semaphores/message queues, POSIX message queues, selected process-inspection/privileged interfaces, changes to process scheduling/limits, and filesystem metadata mutations that Landlock does not mediate. Limit queries remain allowed. `ioctl` is limited to nonblocking-mode and bytes-available requests. Wrong syscall architectures and x86-64 x32 are rejected. Signals outside the sandbox and external abstract Unix sockets are additionally scoped by Landlock. This is not a complete syscall allowlist. Procfs must be mounted; unsupported setup fails closed, with no in-process native fallback.

The broker accepts exactly one parent-authorized document URL and form body. Subresources use the committed response URL as their initiator; the renderer cannot supply or broaden it. Scripts and styles are restricted to that origin, with the same check at every redirect. Cross-origin images pass through a fresh confined decoder; the renderer receives validated pixels, the originally requested URL and an origin-taint flag. Raw response bodies, headers, status and final redirect URLs are withheld. All native image-load failures use a fixed generic error, including failures after redirects. A decoder is never reused between images, so a compromised decoder cannot retain state to inspect a later response from another origin. The trusted broker records taint across every redirect hop; returning to the document origin does not remove it. Data images remain supported. This boundary does not implement full Fetch/CORS or Canvas security APIs; no script API currently exposes image pixels. HTTP(S) document redirects can cross origins, while HTTPS downgrades remain blocked. The UI checks snapshot URLs against the broker's exact committed document URL, allowing fragment changes.

The versioned, length-prefixed IPC protocol checks counts, frame sizes, UTF-8, aggregate text/raster storage, DOM namespaces and attribute bindings, parent links, detached subtrees, cycles, depth, hit targets, finite geometry, clip balance and raster dimensions before a snapshot reaches the UI. ERW4 checks reciprocal template/fragment ownership and includes those edges in cycle/depth validation; document mode tags and canonical encoding names are validated separately. Namespace metadata must name existing attributes before its entries are allocated. Image aliases share one transmitted raster. Resource response headers, status and byte lengths have separate bounds. Decoder response frames are limited to the remaining aggregate image budget before allocation; geometry and exact RGBA length are checked again. Raster and SVG dimensions are checked against the remaining pixel budget before allocating their output. The request queue holds at most 64 items and the UI mailbox one snapshot. Nonblocking pipe I/O has deadlines; navigation/shutdown cancellation also reaches a stalled broker exchange. Cleanup kills and reaps renderer, broker and any active image decoder. Image decoders are also reaped after each successful response.

Linux address-space, descriptor and core-dump limits apply separately to every child. The UI has no global hard memory cap. Its painter still handles untrusted validated drawing commands and pixels, with its own work limits. The broker uses synchronous DNS because thread creation is denied; the parent deadline can terminate a stalled lookup. Headless/library fetches retain the HTTP library's timed resolver. There is no complete private-network protection or independent sandbox audit.

## Implemented controls

- The application forbids unsafe Rust. Parser, layout, interpreter, and software-paint code are custom Rust. Platform/codec/crypto dependencies have their own security surface and may use unsafe code.
- Script code cannot call host filesystem, networking, process, clipboard, native eval, or FFI APIs. Native clipboard access is initiated only by explicit user keyboard shortcuts. Unsupported features produce errors. Scripts have DOM access within the current page.
- The script interpreter bounds source, tokens, AST nesting, expression depth, call depth, instructions and allocation accounting. Per-page script count/source are also capped. Long-lived pages can exhaust its arena; there is no garbage collector. Quota termination is a distinct, uncatchable error and cannot be bypassed by finally blocks. Calls, expressions, statements/hoisting and JSON traversal share a weighted nesting budget, preventing nested callbacks from multiplying individually permitted recursion limits.
- HTML and DOM operations bound source/node count, depth, text/attribute storage and mutation. Script-driven moves charge host-inclusive subtree and ancestor traversal, sibling scans and fragment transfers before mutation; removals charge sibling scans. Selectors and style resolution have explicit work limits to contain pathological matching and expansion.
- HTML encoding changes use at most one cached-byte reparse before author execution or subresource requests. Template contents stay outside document queries/resource discovery until inserted; fragment transfer and cloning share DOM depth/storage limits. Neither templates nor strict JavaScript are security isolation boundaries.
- Layout bounds visits, depth, measured text, output commands and coordinate extents. Grid shares and interns computed track lists, with a cap on unique retained track data. Whitespace scans and tokenization consume a source-work budget that discarded Grid measurements cannot refund. Paint limits cumulative raster work and rejects invalid geometry and image buffers.
- Network loading checks every redirect, caps redirects, has request/page time budgets, limits resource counts and both encoded and decompressed response sizes. TLS certificate validation stays enabled.
- HTTPS downgrade redirects and HTTPS-to-HTTP subresources are rejected. URL credentials and unsupported schemes are rejected.
- Remote pages cannot read or navigate to local files. Explicitly opened local documents may load files only beneath their canonical parent directory; symlinks escaping that directory are rejected. The native broker additionally receives the OS filesystem policy above; the renderer cannot open files directly. Headless/library paths use application checks only, and neither path is certified against every concurrent local filesystem race.
- Styles and scripts are conservatively restricted to the document origin. Script/style MIME types are checked. Cross-origin native images use the isolated opaque-pixel path described above; headless/library fetches decode them in the calling process. There is no credential store.
- When a Content-Security-Policy header or meta policy is present, scripts, author styles and external resources are disabled. Header CSP is also enforced by the broker for subresources; meta policies and script/style execution are enforced in the renderer. This intentionally conservative fallback is **not a CSP implementation**. It rejects content that a compliant browser would permit.
- Decoded raster images have dimension limits and a retained pixel budget. PNG metadata limits are installed before decoder construction. WebP preflight checks container/frame dimensions before the codec can allocate inner VP8 planes. Codec scratch storage is not fully described by the retained pixel budget; native process memory caps remain part of the boundary. SVG has separate source, element, geometry, work and image bounds; SVG scripts/external resource references are not executed.
- Page-worker requests and pending snapshots are bounded. Navigation generations prevent stale page results from receiving new user input. Edit acknowledgements keep old results from overwriting newer typing. Password fields are masked and native copy/cut shortcuts do not expose their values.

## Representative limits

| Resource | Bound |
|---|---:|
| Individual fetched/decompressed resource | 8 MiB |
| Aggregate fetched page data | 32 MiB |
| Resource fetches | 48 |
| Redirects per fetch | 8 |
| Encoded form request body | 1 MiB |
| HTTP request timeout | 12 seconds; native DNS also bounded by the parent transaction deadline |
| Entire fetch sequence | 30-second page budget, including redirects |
| DOM nodes / parsed nesting | 100,000 / 256 |
| DOM retained text/attribute accounting | 32 MiB |
| Unique computed Grid track data | 4 MiB per style computation, plus shared defaults and bounded interning metadata |
| Script source / tokens | 256 KiB / 32,768 |
| Script entry instructions / hard call-depth cap | 100,000 / 32 |
| Shared execution nesting | 96 weighted units; calls cost 4, other guarded nesting costs 1 |
| Script cumulative allocation accounting | 8 MiB |
| Individual script string | 262,144 UTF-16 code units / 512 KiB backing units |
| Script tags / aggregate script source | 64 / 1 MiB |
| Image dimensions / image decode allocation | 4096 per axis / 64 MiB |
| Retained decoded page images | 64 MiB |
| Emitted layout text / font size | 500,000 Unicode scalars / 512 CSS pixels |
| Layout whitespace scanning and text tokenization | 500,000 shared, non-refundable scalar visits |
| Grid tracks per axis / participating items | 256 / 4,096 |
| Shared Grid placement and sizing work | 2,000,000 charged operations per layout |
| Headless or window framebuffer | 16 megapixels, 8192 per axis |
| Pending page-worker requests / snapshots | 64 / 1 |
| Native IPC request / response frame | 18 MiB / 128 MiB |
| Native startup / load / other transaction deadlines | 5 / 40 / 15 seconds |
| Each native child: address space / open descriptors / core dump | 1.5 GiB / 128 / 0 bytes |

Limits can reject valid documents, and not every allocation is covered by exact accounting. Only native renderer, broker and image-decoder children have these OS address-space limits. User-supplied public library structures can be mutated outside the checked APIs; incoming native snapshots undergo separate validation.

## Remaining work

Stronger syscall confinement and complete opaque-response semantics; headless and cross-platform process isolation; complete origin/opaque-origin handling; Fetch/CORS/CSP and navigation policy; cookie/storage partitioning; mixed-content/private-network protection; permissions; verified dependency vulnerability monitoring; continuous coverage-guided fuzzing; sanitizers and cross-platform hardening; independent audit. The [CSP standard](https://www.w3.org/TR/CSP3/) describes substantially more behavior than the fallback above.

The current syscall filter targets little-endian x86-64, AArch64 and RISC-V64 Linux; only x86-64 has been validated here. Native startup fails on other architectures. System resolver configurations requiring Unix-socket services may be incompatible; ordinary DNS over IP remains available. The `seccompiler` wrapper is an additional dependency and its [upstream repository](https://github.com/rust-vmm/seccompiler) is archived; it has not been independently audited here.

The [Linux Landlock documentation](https://docs.kernel.org/userspace-api/landlock.html) describes both enforcement and remaining syscall/metadata limitations. These controls must be evaluated together with the implementation and its actual regression tests; an enabled sandbox is not a security audit.

Report reproducible issues with a minimized local fixture and command. Avoid including real cookies, passwords, private URLs, or other secrets in reports.
