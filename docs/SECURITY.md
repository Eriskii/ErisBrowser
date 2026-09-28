# Security boundary

**This is experimental software. It has not had a professional security audit and should not be used for sensitive browsing.** Native browsing now uses a fresh confined Linux process per document. This is a concrete isolation layer, not evidence of production security, complete site isolation, or a complete web-origin model. The headless CLI, benchmarks, tree adapter and public library API do not install this sandbox.

## Native process boundary

The UI retains the window, address bar, clipboard, trusted bundled fonts and software painter. A child runs fetching, HTML/CSS parsing, scripting, image decoding and layout. It receives no desktop/session environment variables. Startup refuses any inherited file descriptor other than stdin/stdout/stderr and its temporary procfs inspection handle; inspection failure also rejects startup.

Before processing a page, the child requires fully enforced Landlock ABI 6 and `no_new_privs`. The filesystem policy grants read access to the explicitly opened local document's canonical parent directory (if any), exact resolver configuration files, and system library trees (`/lib`, `/lib64`, `/usr/lib`, `/nix/store`). It grants no filesystem write or execute rights. Remote documents receive no local document directory grant. TCP bind, signals to processes outside the sandbox, and external abstract Unix sockets are restricted. A seccomp syscall denylist also blocks Unix socket creation (including pathname sockets), socket pairs, process/thread creation, new executables, io_uring, namespace/process-group changes, selected process-inspection/privileged interfaces, and filesystem metadata mutations that Landlock does not mediate. `ioctl` is limited to nonblocking-mode and bytes-available requests. Wrong syscall architectures and the x86-64 x32 ABI are rejected. This is a scoped denylist, not a complete syscall allowlist. Procfs must be mounted. Unsupported sandbox setup produces a native error pane, with no in-process fallback.

The IPC protocol is versioned and length-prefixed. Counts, frame sizes, UTF-8, aggregate text/raster storage, tree parent links, detached subtrees, cycles, depth, hit targets, finite geometry, clip balance and raster dimensions are checked before a snapshot reaches the UI. Image aliases share one transmitted raster. The request queue holds at most 64 items and the UI mailbox one snapshot. Nonblocking pipe I/O has deadlines and navigation/shutdown cancellation; process cleanup kills and reaps the worker. Page requests cannot authorize a broader local file scope or a different displayed origin. Native cross-origin redirects currently fail because trusted response metadata is not yet brokered by the parent.

Linux address-space, descriptor and core-dump limits apply to the child. The parent has no global hard memory cap. The painter still handles untrusted validated drawing commands and pixels inside the UI process, with its own work limits. The confined worker uses synchronous DNS because thread creation is denied; its parent transaction deadline terminates a stalled lookup. Headless/library fetches keep the HTTP library's default timed resolver. Network operations remain in the worker, with general outbound IP access; there is no parent network broker or complete private-network protection.

## Implemented controls

- The application forbids unsafe Rust. Parser, layout, interpreter, and software-paint code are custom Rust. Platform/codec/crypto dependencies have their own security surface and may use unsafe code.
- Script code cannot call host filesystem, networking, process, clipboard, native eval, or FFI APIs. Native clipboard access is initiated only by explicit user keyboard shortcuts. Unsupported features produce errors. Scripts have DOM access within the current page.
- The script interpreter bounds source, tokens, AST nesting, expression depth, call depth, instructions and allocation accounting. Per-page script count/source are also capped. Long-lived pages can exhaust its arena; there is no garbage collector. Quota termination is a distinct, uncatchable error and cannot be bypassed by finally blocks.
- HTML and DOM operations bound source/node count, depth, text/attribute storage and mutation. Selectors and style resolution have explicit work limits to contain pathological matching and expansion.
- Layout bounds visits, depth, measured text, output commands and coordinate extents. Paint limits cumulative raster work and rejects invalid geometry and image buffers.
- Network loading checks every redirect, caps redirects, has request/page time budgets, limits resource counts and both encoded and decompressed response sizes. TLS certificate validation stays enabled.
- HTTPS downgrade redirects and HTTPS-to-HTTP subresources are rejected. URL credentials and unsupported schemes are rejected.
- Remote pages cannot read or navigate to local files. Explicitly opened local documents may load files only beneath their canonical parent directory; symlinks escaping that directory are rejected. The native child also receives the OS filesystem policy above. Headless/library paths use application checks only, and neither path is certified against every concurrent local filesystem race.
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
| HTTP request timeout | 12 seconds; native DNS also bounded by the parent transaction deadline |
| Entire fetch sequence | 30-second page budget, including redirects |
| DOM nodes / parsed nesting | 100,000 / 256 |
| DOM retained text/attribute accounting | 32 MiB |
| Script source / tokens | 256 KiB / 32,768 |
| Script entry instructions / call depth | 100,000 / 48 |
| Script cumulative allocation accounting | 8 MiB |
| Individual script string | 262,144 UTF-16 code units / 512 KiB backing units |
| Script tags / aggregate script source | 64 / 1 MiB |
| Image dimensions / image decode allocation | 4096 per axis / 64 MiB |
| Retained decoded page images | 64 MiB |
| Emitted layout text / font size | 500,000 Unicode scalars / 512 CSS pixels |
| Headless or window framebuffer | 16 megapixels, 8192 per axis |
| Pending page-worker requests / snapshots | 64 / 1 |
| Native IPC request / response frame | 18 MiB / 128 MiB |
| Native startup / load / other transaction deadlines | 5 / 40 / 15 seconds |
| Native worker address space / open descriptors / core dump | 1.5 GiB / 128 / 0 bytes |

Limits can reject valid documents, and not every allocation is covered by exact accounting. Only the native worker has an OS address-space limit. User-supplied public library structures can be mutated outside the checked APIs; incoming native snapshots undergo separate validation.

## Remaining work

A parent network/resource broker and stronger syscall confinement; headless and cross-platform process isolation; complete origin/opaque-origin handling; Fetch/CORS/CSP and navigation policy; cookie/storage partitioning; mixed-content/private-network protection; permissions; verified dependency vulnerability monitoring; continuous coverage-guided fuzzing; sanitizers and cross-platform hardening; independent audit. The [CSP standard](https://www.w3.org/TR/CSP3/) describes substantially more behavior than the fallback above.

The current syscall filter targets little-endian x86-64, AArch64 and RISC-V64 Linux; only x86-64 has been validated here. Native startup fails on other architectures. System resolver configurations requiring Unix-socket services may be incompatible; ordinary DNS over IP remains available. The `seccompiler` wrapper is an additional dependency and its [upstream repository](https://github.com/rust-vmm/seccompiler) is archived; it has not been independently audited here.

The [Linux Landlock documentation](https://docs.kernel.org/userspace-api/landlock.html) describes both enforcement and remaining syscall/metadata limitations. These controls must be evaluated together with the implementation and its actual regression tests; an enabled sandbox is not a security audit.

Report reproducible issues with a minimized local fixture and command. Avoid including real cookies, passwords, private URLs, or other secrets in reports.
