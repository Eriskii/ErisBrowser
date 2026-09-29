# Security boundary

**This is experimental software. It has not had a professional security audit and should not be used for sensitive browsing.** Native browsing uses a fresh confined Linux renderer per document and a separate resource broker when resources are needed. This is a concrete isolation layer, not evidence of production security, complete site isolation, or a complete web-origin model. The `--render` and `--benchmark` CLI modes, conformance adapters and public library API do not install this sandbox. `--benchmark-worker` exercises the confined renderer/broker and validated snapshot path, with the same fail-closed Linux requirements as native browsing.

The optional Linux Vulkan presenter runs a graphics driver on a dedicated thread
inside the privileged native browser process. This is not GPU process isolation.
Only completed CPU pixel frames cross its mailbox; it receives no page shader
source. It limits active/pending frames and counts application pixel-buffer
capacities, while driver staging, swapchains and internal allocation remain
outside that ledger. A five-second application deadline cannot interrupt a
native call or destructor. Timeout or thread panic never authorizes a competing
software surface; fallback requires an acknowledgment after actual resource
drops return. Feature-enabled worker launches use an isolated exec-only stage
that marks all non-stdio descriptors close-on-exec before replacing itself with
the worker. It validates the internal role, requires single-thread startup and
handles no page data. Parent descriptors remain unchanged. This handles both
transient loader files and retained driver handles; a graphics/spawn mutex was
insufficient. Worker startup still rejects every unexpected descriptor that
survives exec before handling page content. Driver crashes
can still affect the browser process. The renderer,
broker and image-decoder sandbox policies are unchanged. See the
[Vulkan boundary](vulkan-rendering.md#isolation-resource-ownership-and-recovery).

## Native process boundary

The UI retains the window, address bar, clipboard, trusted bundled fonts and software painter. A renderer child runs HTML/CSS parsing, scripting, same-origin image decoding and layout. A separate broker child performs resource loading; the parent relays its bounded responses and records the broker's final document URL. Each cross-origin image is decoded in a fresh third child which exits after one response. All children start with a cleared environment and reject inherited descriptors other than stdin/stdout/stderr and the temporary procfs inspection handle. Inspection failure rejects startup.

All children require fully enforced Landlock ABI 6 and `no_new_privs` before processing page data. **The renderer has no filesystem read/write/execute grants and cannot create any socket.** TCP connect/bind is also denied by its Landlock policy. It receives document and authorized resource bytes only through the parent-controlled pipe. Cross-origin image decoders use the same zero-file-grant, zero-socket policy. Metadata observations such as some pathname/stat operations are not fully mediated by these filesystem access rules.

The broker grants read access to the explicitly opened local document's canonical parent directory (if any), exact resolver files, and system library trees (`/lib`, `/lib64`, `/usr/lib`, `/nix/store`). It grants no filesystem write or execute rights, and remote documents receive no local directory grant. General outbound IP access remains available to this process for DNS and HTTP/TLS; TCP listeners and Unix sockets are denied. Broker compromise is therefore a different, broader boundary than renderer compromise. The broker does not parse HTML, run scripts, or decode images.

A shared seccomp denylist blocks process/thread creation, new executables, io_uring, namespace/process-group changes, socket pairs, System V shared memory/semaphores/message queues, POSIX message queues, selected process-inspection/privileged interfaces, changes to process scheduling/limits, and filesystem metadata mutations that Landlock does not mediate. Limit queries remain allowed. `ioctl` is limited to nonblocking-mode and bytes-available requests. Wrong syscall architectures and x86-64 x32 are rejected. Signals outside the sandbox and external abstract Unix sockets are additionally scoped by Landlock. This is not a complete syscall allowlist. Procfs must be mounted; unsupported setup fails closed, with no in-process native fallback.

The broker accepts exactly one parent-authorized document URL and form body. Subresources use the committed response URL as their initiator; the renderer cannot supply or broaden it. Scripts and styles are restricted to that origin, with the same check at every redirect. Cross-origin images pass through a fresh confined decoder; the renderer receives validated pixels, the originally requested URL and an origin-taint flag. Raw response bodies, headers, status and final redirect URLs are withheld. All native image-load failures use a fixed generic error, including failures after redirects. A decoder is never reused between images, so a compromised decoder cannot retain state to inspect a later response from another origin. The trusted broker records taint across every redirect hop; returning to the document origin does not remove it. Data images remain supported. This boundary does not implement full Fetch/CORS or Canvas security APIs; no script API currently exposes image pixels. HTTP(S) document redirects can cross origins, while HTTPS downgrades remain blocked. The UI checks snapshot URLs against the broker's exact committed document URL, allowing fragment changes.

The versioned, length-prefixed IPC protocol checks counts, frame sizes, UTF-8, aggregate text/raster storage, DOM namespaces and attribute bindings, parent links, detached subtrees, cycles, depth, hit targets, finite geometry and raster dimensions before a snapshot reaches the UI. ERW8 validates reciprocal template/fragment ownership and includes those edges in cycle/depth checks. Document modes, canonical encoding names, selected base nodes and frozen base URLs are also validated. Clip, fixed-coordinate and opacity commands share a typed scope stack: cross-kind closures, unclosed scopes and combined depth above 128 are rejected. Opacity values must be finite and between zero and one. Hit regions carry validated document/fixed-coordinate flags and typed ordinary/generated-summary actions. Generated-summary hints require an active, visible HTML details element without a direct authored summary; forged activation commands fail the worker channel closed. Namespace metadata must name existing attributes before its entries are allocated. Image aliases share one transmitted raster. Resource response headers, status and byte lengths have separate bounds. Decoder response frames are limited to the remaining aggregate image budget before allocation; geometry and exact RGBA length are checked again. Raster and SVG dimensions are checked against the remaining pixel budget before allocating their output. The request queue holds at most 64 items and the UI mailbox one snapshot. Task state is a validated three-value enum; the parent rejects pending or suspended task metadata when it authorized scripts to be disabled. Idle task continuation waits at least 16 ms after its previous rendered snapshot and gives queued native input priority; cancellation and transaction deadlines apply to task commands too. Nonblocking pipe I/O has deadlines; navigation/shutdown cancellation also reaches a stalled broker exchange. Cleanup kills and reaps renderer, broker and any active image decoder. Image decoders are also reaped after each successful response.

Linux address-space, descriptor and core-dump limits apply separately to every child. The UI has no global hard memory cap. Its painter still handles untrusted validated drawing commands and pixels, with its own work limits. The broker uses synchronous DNS because thread creation is denied; the parent deadline can terminate a stalled lookup. Headless/library fetches retain the HTTP library's timed resolver. There is no complete private-network protection or independent sandbox audit.

## Implemented controls

The standalone [Vulkan development probe](../tools/vulkan-probe/README.md) uses
host device/driver access with fixed synthetic pixels and no page content. It
does not change the browser's dependency graph or renderer authority. Its finite
process deadlines and output bounds are test controls, not a GPU sandbox or a
hard limit on driver allocations. Native driver calls and teardown can outlast
application waits. The optional browser presenter has the additional ownership
and launch controls described above; neither path is a GPU sandbox.

- The application forbids unsafe Rust. Parser, layout, interpreter, and software-paint code are custom Rust. Platform/codec/crypto dependencies have their own security surface and may use unsafe code.
- Script code cannot call host filesystem, networking, process, clipboard, native eval, or FFI APIs. Native clipboard access is initiated only by explicit user keyboard shortcuts. Unsupported features produce errors. Scripts have DOM access within the current page.
- The script interpreter bounds source, tokens, grammar continuation storage/work, logical calls, native recursion, instructions and allocation accounting. Per-page script count/source are also capped. Template text scanning, parser-directed rescans, substitution coercion and output copying consume compile/runtime budgets; template expressions use the shared execution driver and coercion callbacks retain native guards. Owned executable names and parser lists are charged during direct flat construction; functions share immutable code and retain conservative activation metadata allowances. Generic array reversal and radix conversion retain shared work/heap/coercion limits. Long-lived pages can exhaust its arena; there is no garbage collector. Quota termination is a distinct, uncatchable error and cannot be bypassed by finally blocks. Native callbacks, constructors, events and JSON traversal share a weighted nesting budget. Iterative JavaScript uses precharged continuation storage, with all invocations sharing the logical-call ceiling.
- HTML and DOM operations bound source/node count, depth, text/attribute storage and mutation. Script-driven moves charge host-inclusive subtree and ancestor traversal, sibling scans and fragment transfers before mutation; removals charge sibling scans. Selectors and style resolution have explicit work limits to contain pathological matching and expansion.
- HTML encoding changes use at most one cached-byte reparse before author execution or subresource requests. Template contents stay outside document queries/resource discovery until inserted; fragment transfer and cloning share DOM depth/storage limits. Neither templates nor strict JavaScript are security isolation boundaries.
- Layout bounds visits, depth, measured text, output commands and coordinate extents. Grid shares and interns computed track lists, with a cap on unique retained track data. Whitespace scans and tokenization consume a source-work budget that discarded measurements cannot refund. Fragments share one hit-allocation budget. Deferred positioning and stacking share a work limit; reconstruction reserves scope closures and removes hits for commands beyond its paint cutoff. Paint limits cumulative raster work and rejects invalid geometry and image buffers.
- Stylesheet imports retain the committed document authority, preserve separate parse boundaries and bound recursion, count, source bytes, repeated scan work, URL resolution and cache storage. Media conditions and imported layer identities are separate metadata, so stylesheet contents cannot close a generated wrapper. Anonymous layer identities cannot collide with authored names. URL accounting includes repeated cache lookups and failed-resource keys. Media evaluation bounds input, alternatives, leaf terms and delimiter nesting, with recursive conditions consuming both a local work cap and shared stylesheet parsing work. Every valid branch is checked even when its truth value could short-circuit. Cascade layer names, depth and rollback candidates have explicit limits. Script base mutations charge scanning and URL work before mutation. Frozen base metadata never grants an origin or local directory.
- Event paths, listener traversal/snapshots, inline handler compilation and recursive dispatch consume script work/allocation/stack budgets. Readonly event state is stored in private slots; synthetic dispatch cannot make an event trusted. AbortSignal state and listener IDs share allocation limits; abort reserves complete listener-removal work before changing observable state. Ordinary listener exceptions are reported; resource termination remains uncatchable and clears dispatch state.
- Feature queries use bounded grammar and a conservative positive syntax inventory; invalid condition syntax and exhausted query budgets remain invalid through negation. False import conditions are checked before fetching or registering a layer. Matching and selector queries share bounded token grammar, preserving comment and whitespace distinctions. Undeclared named namespace prefixes and exhausted query budgets invalidate the complete condition. Compiled selector/index records have an 8 MiB accounted storage cap; byte scans and recursive serialization consume shared work before growth.
- JavaScript `CSS.supports` shares script instruction, allocation and coercion limits. UTF-16 conversion and parser scratch reserve proportional work/storage before allocation. The two-argument overload parses its value separately from its literal property name, preventing argument text from becoming a different condition. Extra arguments are not coerced by this API; selected author conversions run before query-size rejection. Resource failures remain uncatchable, and repeated probes consume the existing cumulative arena.
- Custom-property substitution tokenizes before expansion, bounds active resolution and fallback nesting, and charges output copies before growth. Computed maps share inherited values and retain an aggregate storage cap. Cycles invalidate active participants, while unused fallbacks are not traversed. After a result becomes invalid, later references still perform bounded dependency lookup without repeatedly scanning discarded cached values. These bounds contain expansion and repeated-scan costs; incomplete ordinary-property parsing remains a compatibility limitation.
- Window property-descriptor queries read tracked global binding records without invoking their getters or exposing lexical bindings. Definition of self, globalThis, undefined, NaN and Infinity uses ordinary descriptor compatibility checks; other host definitions remain unsupported. Key conversion retains author side effects and exceptions, while property lookup, storage and accessor calls share the script budgets. Exact UTF-16 matching prevents malformed keys from aliasing those five names. Replacing public globalThis does not change the runtime's private Window identity or event targets. These operations add no cross-origin WindowProxy or host access authority.
- Inline CSSOM operations finish ordered author conversions before parsing current DOM state. They reserve parser/serialization scratch, share script work and allocation limits, and stage one final attribute write. Escaped names and value-token boundaries prevent a serialized record from injecting or consuming adjacent declarations. Recursive calc validation consumes shared work. Resource failures terminate the entry before the staged write; earlier author side effects are retained. This is bounded parser/runtime behavior, not complete CSSOM conformance.
- Computed property references retain raw keys until their specified read/write point, then use bounded string-hint conversion and cache the resulting UTF-16 key. Compound operations cannot repeat author key conversion during their final write; exceptions and quota failures retain the existing execution limits.
- Functions and defaults share flat executable units across closures and calls, while retaining conservative per-closure/per-activation metadata allowances and charging environment bindings. Initialization and body execution share the same driver, work, logical-call and cumulative allocation allowances; native helpers retain the weighted stack guard. A default cannot reset those limits; resource termination stops later initializers and body execution. Parameter/lexical conflict checks use a charged name set instead of repeated full parameter scans.
- Additive length-percentage calculations retain finite coefficients and percentage dependency without per-style syntax trees. Original tokens are validated before comment normalization and after variable substitution; consumed arithmetic tokens spend shared work. Context-specific clamping happens after resolution. Grid track interning includes both coefficients and the dependency flag, and accounts for the larger track representation.
- The parser directly builds flat executable expression, statement and function records with typed IDs. Record pages, parser lists, names/operators, copied payloads and publication share the compile ledger and use checked reservations. Completed and partial syntax have no owning child links; abandoned cover-grammar records remain charged tombstones. Closures retain their whole unit, including otherwise unreachable code, and carry it across callbacks. The old owning AST/lowering path is test-only. Supported JavaScript grammar uses precharged flat continuations, including saved function contexts and pending defaults/templates. Frame slots derive from the shared compile storage budget; source, token and work limits remain. The old grammar-depth and member/constructor chain guards are removed; active labels, declaration traversal and the separate RegExp compiler retain their own limits. The ledger is cumulative without refunds and is not a proof of total process memory: diagnostic formatting, allocator overhead, runtime arenas and native helpers still need broader accounting. This stage does not establish production security.
- Expressions, references, statements, ordinary activation, defaults and bound forwarding use one runtime-owned continuation vector. Frame capacity and relocation work are precharged; the slot ceiling derives from the existing 8 MiB allowance divided by the record size. Queued invocations acquire their logical count after a successful push; native callback roots preserve their externally owned count and guard. Cleanup restores the current frame boundary and counters without allocating or running author code. Ordinary exceptions reach try continuations; resource/unsupported host errors bypass catch/finally. Code handles avoid copying identifier names and body lists, while array/argument payloads and bound forwarding buffers are precharged. Constructors and native callback bridges retain weighted recursion guards. Fully iterative JavaScript no longer consumes native-depth units; all calls still share the 32-call ceiling. Refused charges remain in the cumulative ledger. These controls do not establish complete runtime allocation accounting or production security.
- Declaration-name collection and runtime hoisting use a fixed array of 96 borrowed ancestor cursors. Cursor movement and empty switch-case lists consume shared work, and traversal errors are terminal. This removes their native recursion and switch-body syntax cloning without raising parser or execution limits. Declaration-name validation now precharges borrowed record growth, merge scratch, copies, UTF-8 comparisons and named diagnostics. This covers scope, for-head, parameter and catch-name validation; it is not complete parser or runtime map accounting.
- Identifier lexing uses static Unicode tables with two bounded table reads. Initial scanning and parser-directed suffix rescans share one cumulative compile ledger; token pages, decoded names, provisional diagnostics and charged syntax copies consume that allowance. Page descriptors may move during growth; existing token pages do not. The parser borrows source only during compilation; executable units own flat records and string data. Escapes cannot turn a decoded name into a literal keyword terminal. Compile resource failures remain uncatchable, and discarded provisional tokens do not refund work or storage. These lexical rules add no runtime authority and do not establish a security audit.
- Identifier rest parameters reserve copying work and estimated array/bookkeeping storage before allocating or retaining the remaining actual values. They create ordinary interpreter arrays without invoking author iterators, global constructors or inherited setters. Empty rest arrays also consume the cumulative runtime allowance; defaults and rest construction share the calling entry's limits.
- Prototype membership reads internal links iteratively, charging each visit against shared work and the existing 96-link guard. It invokes no author getters or coercion hooks to discover a chain. Primitive arguments return before receiver boxing; cyclic or overlong internal graphs terminate with an uncatchable resource error.
- Array sorting uses a bounded value list and iterative merge over precharged index buffers. Every collection, comparison and copy consumes the caller's work/heap allowance; author comparators and conversions retain shared recursion guards. Inconsistent comparators cannot prevent cursor progress. Sorting writes begin after collection/comparison, while later write/delete failures preserve earlier successful effects. Callback mutations are not rolled back. The existing 65,536 array-like length cap does not promise completion below that size.
- Disclosure groups and coalesced toggle records are bounded by the DOM node inventory. Group/tree mutation work is charged before script-driven changes. One host checkpoint dispatches at most 64 toggle records under a shared instruction budget. Exact reentrant tracking permits at most two live records per element, counting the single active task, with a separately bounded tracker map. Checkpoint failure preserves unstarted tasks and suspends automatic retry until document replacement, so an exhausted persistent script arena cannot cause idle polling. Ordinary listener exceptions retain existing reporting behavior. Closed content retains document activity but cannot receive native editing; snapshots revoke newly hidden edit focus. This is a rendering/input policy, not an origin or script isolation boundary.
- Opacity surfaces are deferred until a clipped nonzero-alpha pixel needs them, then allocated under separate peak/cumulative byte caps. Initialization and compositing consume shared paint work; unfinished surfaces are dropped on invalid streams, allocation failure or quota exhaustion. Opacity never disables hit testing by itself.
- RegExp parsing and matching share script work/allocation budgets. Backtracking uses an explicit bounded stack; assertions and callbacks retain nesting guards. Quota failures remain uncatchable.
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
| Additive calculation input / nesting / terms | 4 KiB / 16 / 256, within shared token and work budgets |
| Script source / tokens | 256 KiB / 32,768 |
| Script entry instructions / hard call-depth cap | 100,000 / 32 |
| Shared execution nesting | 96 weighted units; calls cost 4, other guarded nesting costs 1 |
| Script cumulative allocation accounting | 8 MiB |
| Disclosure notifications per host checkpoint | 64, sharing one 100,000-step budget |
| Individual script string | 262,144 UTF-16 code units / 512 KiB backing units |
| Script tags / aggregate script source | 64 / 1 MiB |
| RegExp pattern units / syntax nodes / captures | 8,192 / 4,096 / 128 |
| Stylesheet decoded / constructed / repeatedly scanned source | 8 / 8 / 32 MiB per load |
| Supports condition bytes / terms / nesting | 16 KiB / 64 / 16 |
| Supports local work / shared import-query work | 256 KiB / 8 MiB |
| CSS.supports combined UTF-8 argument bytes / property bytes / value bytes | 16 KiB / 256 / 4,096, also subject to shared script limits |
| Inline CSSOM source/serialized allowance / declaration records | 64 KiB / 256, also subject to shared script limits |
| Inline CSSOM property-name bytes / value bytes / component depth | 256 / 4,096 / 16 |
| Retained custom-property names per element / name bytes / authored value bytes | 128 / 256 / 4,096 |
| Expanded custom value / active resolution or component nesting | 64 KiB / 16 |
| Aggregate retained custom-property maps | 8 MiB accounted names, values and entry overhead; inherited maps/strings share storage |
| Stylesheet import attempts / retained segments / depth | 256 / 256 / 16 |
| Inline stylesheet collection: count / copied text / direct-child visits | 256 / 8 MiB / 200,000 per collection |
| Import prelude / media condition delimiter nesting | 128 |
| Media-query input / alternatives / conjunctive features / nesting | 64 KiB / 64 / 64 per query / 16 |
| Stylesheet URL work / retained URL cache data | 32 / 32 MiB per load |
| Page subresource URL resolution work | 32 MiB per load |
| Frozen base URL metadata | 32 MiB, separate from general IPC string limit |
| Image dimensions / image decode allocation | 4096 per axis / 64 MiB |
| Retained decoded page images | 64 MiB |
| Emitted layout text / font size | 500,000 Unicode scalars / 512 CSS pixels |
| Layout whitespace scanning and text tokenization | 500,000 shared, non-refundable scalar visits |
| Grid tracks per axis / participating items | 256 / 4,096 |
| Shared Grid placement and sizing work | 2,000,000 charged operations per layout |
| Shared positioning/stacking work | 2,000,000 charged operations per layout |
| Layout hit inventory / final commands / combined clip-fixed-opacity depth | 100,000 / 200,000 / 128 |
| Opacity offscreen peak / cumulative allocated pixels | 64 / 128 MiB (8,388,608 / 16,777,216 RGBA16 pixels) per paint |
| Event path targets | 258, with shared script work/allocation/nesting limits |
| CSS layers / layer depth / registered names | 1,024 / 16 / 64 KiB |
| Expanded stylesheet declarations / direct declaration list | 16 / 1 MiB of names and values |
| Cascade candidate entries / text per element | 4,096 / 1 MiB, with shared cascade work |
| Headless or window framebuffer | 16 megapixels, 8192 per axis |
| Pending page-worker requests / snapshots | 64 / 1 |
| Native IPC request / response frame | 18 MiB / 128 MiB |
| Native startup / load / other transaction deadlines | 5 / 40 / 15 seconds |
| Each native child: address space / open descriptors / core dump | 1.5 GiB / 128 / 0 bytes |

Limits can reject valid documents, and not every allocation is covered by exact accounting. Only native renderer, broker and image-decoder children have these OS address-space limits. User-supplied public library structures can be mutated outside the checked APIs; incoming native snapshots undergo separate validation.

## Symbol storage and callbacks

Symbol identities use retained safe-Rust handles; descriptions do not identify keys.
Registry lookup/scans, key lists, descriptive strings, function names and custom
object tags share the existing work/allocation ledger. Refused registry entries
and property insertions are not published. Primitive/instance hooks share native
and script guards; looping or recursive callbacks cannot catch quota exhaustion.
Diagnostic console formatting charges Symbol description scanning and UTF-8
storage before formatting. Supported DOM operations use checked JavaScript string
conversion separately.

The [Symbol validation record](../tests/conformance/symbol-properties.json)
includes exhaustion and unwind checks. Quotas are unchanged. The registry belongs
to the current page Runtime; shared multi-realm behavior is unimplemented.
This remains estimated accounting, without an independent security audit.

## DOM conversion callbacks

Supported DOM methods check their receiver and required arguments before invoking
author conversion hooks. Variadic append/class-list arguments are converted before
their operation; callbacks may themselves mutate the DOM. Title and class-list
lookups observe that live state. Host conversion charges scans and worst-case
UTF-8 storage before decoding. Staged arguments, token parsing/deduplication,
vector growth, serialization, query scratch and native method handles share the
existing ledger. No quota increased.

The [focused validation](../tests/conformance/dom-string-conversion.json) includes
allocation refusals and looping/recursive callbacks that cannot catch resource
exhaustion, with execution-state cleanup checked afterward. These checks do not
establish full DOM operation atomicity or exact process-wide memory accounting.
Lone UTF-16 surrogates are still replaced at the DOM storage boundary.

## Window binding reflection

Window binding reflection now precharges snapshots, name scans, UTF-16 key copies,
both output vectors and sorting work. Sorting allocates no additional buffer;
vector reservations are fallible. The binding ledger includes each creation serial,
and serial exhaustion rejects a new property before publication. Descriptor and
membership queries do not invoke author getters or expose global lexical bindings.
The [reflection checks](../tests/conformance/window-reflection.json) cover refused
storage/work and counter exhaustion; quotas remain unchanged. The later
[general binding work](../tests/conformance/window-global-bindings.md) also charges
private receiver slots, declaration selection maps, UTF-16 property records and
key conversions. Author `this` properties cannot replace execution receivers.
Getter/setter callbacks share existing limits and unwind on uncatchable resource
errors. Event-handler descriptor/deletion behavior, full interface reflection and
WindowProxy semantics remain incomplete.

## String concatenation

[String.prototype.concat](../tests/conformance/string-concat.md) precharges a
fallibly reserved fragment vector, copying work, output buffer and Rc conversion
storage. Cumulative UTF-16 length is checked after each argument conversion;
callbacks share existing instruction, allocation and recursion limits. Abrupt
conversions retain earlier author effects, and resource failures unwind without
running catch/finally recovery. This changes no script authority or quota. The
ledger remains estimated; the checks do not establish production security.

## Constructor execution state

[Constructor targets](../tests/conformance/construction.md) use private environment
slots charged to the existing heap ledger. Array-like and bound argument storage
is checked and prepaid before reservation/copying; the argument cap and native
stack guards remain in force. Getter callbacks and construction share runtime
work, heap and recursion limits. Resource failures bypass script recovery and
unwind continuation/call counters. These changes add no filesystem, network or
GPU authority. The [Symbol target correction](../tests/conformance/constructor-policy.md)
allows Symbol as an allocation target without installing Symbol value slots on
an ordinary object. Argument callbacks still share the existing limits, and
Symbol construction throws before description conversion. The ledger remains
an estimate, and no independent audit is claimed.

## Remaining work

Stronger syscall confinement and complete opaque-response semantics; headless and cross-platform process isolation; complete origin/opaque-origin handling; Fetch/CORS/CSP and navigation policy; cookie/storage partitioning; mixed-content/private-network protection; permissions; verified dependency vulnerability monitoring; continuous coverage-guided fuzzing; sanitizers and cross-platform hardening; independent audit. The [CSP standard](https://www.w3.org/TR/CSP3/) describes substantially more behavior than the fallback above.

The current syscall filter targets little-endian x86-64, AArch64 and RISC-V64 Linux; only x86-64 has been validated here. Native startup fails on other architectures. System resolver configurations requiring Unix-socket services may be incompatible; ordinary DNS over IP remains available. The `seccompiler` wrapper is an additional dependency and its [upstream repository](https://github.com/rust-vmm/seccompiler) is archived; it has not been independently audited here.

The [Linux Landlock documentation](https://docs.kernel.org/userspace-api/landlock.html) describes both enforcement and remaining syscall/metadata limitations. These controls must be evaluated together with the implementation and its actual regression tests; an enabled sandbox is not a security audit.

Report reproducible issues with a minimized local fixture and command. Avoid including real cookies, passwords, private URLs, or other secrets in reports.

Dynamic ordinary Function compilation parses parameter and body inputs separately
and charges the active runtime's remaining work and cumulative allocation budget
on success and failure. No dynamic compile calls the top-level entry point that
resets instructions. Generated closures use the global environment; conversion
callbacks and prototype getters retain existing nesting guards. The existing CSP
fallback still disables all scripts for a policy-bearing page. Literal unpaired
UTF-16 source surrogates are explicitly unsupported pending lossless parser input;
no lossy replacement is used. See the [tests and limits](../tests/conformance/function-constructor.md).

Generic String conversion, `Symbol.match` classification and custom `Symbol.split`
dispatch invoke ordinary getters/callbacks under the existing shared runtime
limits. Split-hook argument storage is charged before calling author code.
Recursive getter/conversion/hook regressions verify resource exhaustion and
counter/frame cleanup; a heap-exhausted split cannot invoke the hook. No quotas,
dependencies or process authorities change. See the [coverage and limits](../tests/conformance/string-conversion.md).

The [RegExp split follow-up](../tests/conformance/regexp-split.md) runs species,
flags, execution and capture callbacks under the same resource guards. It charges
flag assembly, constructor arguments, substring copies and result growth; the
private result array cannot expose intermediate values to inherited setters.
Tests cover recursive callbacks, unbounded capture lengths, backward-moving match
indices, heap refusal and frame cleanup. No complete allocation-accounting or
independent security-audit claim follows from these checks.
