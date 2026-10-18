# Eris Browser

An independent Rust browser with custom HTML parsing, DOM, CSS cascade, layout, a JavaScript subset interpreter, SVG handling, software painting and experimental GPU compositing. There is no Chromium, Firefox, WebKit, Servo, Ladybird, embedded webview, or external JavaScript engine in the rendering path.

**Status: an early browser implementation, not a fully web-compatible or production-secure browser.** The original requirements—every web standard, production security, and performance within 30% of Chromium—are not achieved. Many modern websites will not function. See [compatibility](docs/COMPATIBILITY.md) and [security](docs/SECURITY.md) for concrete boundaries.

[JavaScript classes](docs/classes.md) now support declarations and expressions,
base/derived construction, public methods/accessors and native Promise
subclasses. Strict lexical bindings, `super()` ordering, alternate `new.target`
and retained arrow activations share the existing runtime limits.
[Class `super` properties](docs/class-super-properties.md) add receiver-aware
reads, calls and writes in constructors, public methods/accessors and lexical
arrows. Object-literal method `super`, fields, private elements, static blocks
and async/generator methods remain unsupported. The separate [Promise class-admission checkpoint](docs/promise-class-admission.md)
now enables declared `class` metadata in seven existing Promise profiles, with
1,330 modes across eight replays: **1,258 passed, 64 failed and eight unsupported**,
and **724 healthy controls**. This changes profile admission and its gate;
Rust semantics and limits are unchanged. Earlier API/profile totals below belong
to their original checkpoints; the linked table records the current selection.

[Promise construction and reactions](docs/promise-foundation.md) add the
constructor, `then`, `resolve`, `reject`, generic species/capabilities and
thenable adoption to the shared microtask FIFO. Jobs preserve original
rejection values and share the existing work and storage limits. The [generic `catch` method](docs/promise-catch.md) forwards through observable
`then` lookup and preserves original receivers, arguments and return values.
[`finally`](docs/promise-finally.md) now waits for cleanup, preserves the original
outcome on success and adopts cleanup failures. Its reusable callbacks preserve
observable species and `then` behavior under the existing resource limits.
[`Promise.all`](docs/promise-all.md) now collects synchronous iterable inputs in
input order, with observable constructor/resolve/iterator calls and bounded
once-only fulfillment callbacks. [`Promise.race`](docs/promise-race.md) uses the
first settlement, preserves the original capability callbacks and keeps
observable iteration order. [`Promise.allSettled`](docs/promise-all-settled.md)
collects fulfilled and rejected outcome records in input order, with a shared
once gate for each input’s two callbacks. [`Promise.any`](docs/promise-any.md)
fulfills on the first fulfillment and collects ordered rejection reasons into an
intrinsic AggregateError when every input rejects. Async functions and rejection
notifications remain unfinished. Its separate [complete pinned upstream profile](docs/promise-any-upstream.md)
retains all 94 sources and 188 modes under a 102-control gate.
It records **172 passed, 14 unsupported and two failed modes**, with all **102
controls verified**.

[`Promise.withResolvers`](docs/promise-with-resolvers.md) returns a fresh
`{promise, resolve, reject}` result using a generic constructor and the exact
capability callbacks. Ordinary constructor errors remain synchronous; existing
settlement and terminal queue behavior are preserved. Its complete pinned
six-source upstream profile retains all 12 modes: **10 passed and two unsupported**,
with all **44 controls verified**. At that checkpoint, the two class-tagged modes were excluded;
the profile uses ordinary requests and responses without a Promise host policy.

[`Promise.try`](docs/promise-try.md) calls the callback synchronously before
resolving its result or creating a rejection capability, following the current
ECMAScript 2027 draft. It preserves generic receiver behavior, argument identity
and existing terminal suspension. Its separate [complete pinned upstream
profile](docs/promise-try-upstream.md) retains all 15 sources and 30 modes:
**24 passed and six unsupported**, with all **94 controls verified**.
At that checkpoint, the six class-tagged modes were excluded; authenticated completion uses the
existing Promise host adapter.

[`AggregateError`](docs/aggregate-error.md) adds callable and constructed errors
with ordered synchronous iterable collection, optional message/cause properties
and a fresh intrinsic `errors` Array. Genuine Error instances now receive the
Error fallback tag from `Object.prototype.toString`.
The separate [AggregateError upstream profile](docs/aggregate-error-upstream.md)
retains all 25 pinned sources and 50 modes, with an ordinary-response identity
capability confined to that profile.
The optional [Promise Test262 host](docs/promise-test262-host.md) adds authenticated
`$DONE` completion and a separate final-unhandled-rejection policy for a pinned
232-source, 458-mode profile. The separate [complete `Promise.all` selection](docs/promise-all-test262.md)
records 174 passed, 14 failed and 8 unsupported modes out of 196, with all 90
harness controls passing. The remaining failures reflect the host's
final-unhandled-rejection policy. The [complete `Promise.race` selection](docs/promise-race-test262.md)
records 170 passed, 10 failed and 8 unsupported modes out of 188, with all 90
controls verified. Its failures retain the same separate host-policy reason.
The separate [complete `Promise.allSettled` selection](docs/promise-all-settled-test262.md)
retains all 104 sources and 208 modes under a 94-control gate.
It records 188 passed, 12 failed and 8 unsupported, with all 94 controls verified.
The twelve failures are final-unhandled host-policy vetoes; the eight unsupported
modes retain their class metadata or parser reasons.

[Spread arguments](docs/call-spread.md) now work in ordinary calls and `new`,
including mixed, multiple and empty spreads and trailing commas. Iterator reads
preserve evaluation order and share the existing resource limits. This closes
four upstream Promise modes; array/object spread remains unfinished.
[Argument grammar contexts](docs/call-argument-in.md) also allow `in` expressions
inside call and constructor arguments in `for` initializers, while preserving
the surrounding `for-in` grammar.
[Computed keys and conditional contexts](docs/expression-in.md) now admit
comma expressions inside brackets and `in` inside brackets or the middle of
`?:`, including in `for` initializers. Evaluation order and outer restrictions
are covered by interpreter and parser regressions.

[Shared startup keys](docs/pi-key-cache.md) remove duplicate immutable key
allocations from ten DOM function metadata bags while keeping their descriptors
and identities. Work and heap limits remain unchanged.
The [job metadata follow-up](docs/shared-job-keys.md) also shares the immutable
`length` and `name` keys between `queueMicrotask` and Promise function metadata.
The [Node accessor follow-up](docs/node-accessor-keys.md) reuses those DOM keys
for four accessor functions, saving 128 startup work units and 672 accounted
heap bytes while preserving their separate identities and behavior.
The [Node method follow-up](docs/node-method-keys.md) extends this sharing to nine
more functions, saving another 250 startup work units and 1,512 accounted heap
bytes within the same limits.
The [Text metadata follow-up](docs/text-metadata-keys.md) shares the existing keys
for `wholeText` and `splitText`, saving 60 startup work units and 336 accounted
heap bytes while preserving both functions and their behavior.
The [startup metadata follow-up](docs/startup-metadata-sharing.md) shares title
keys and PI/CharacterData owner-name payloads, saving 279 more startup work units
and 1,132 accounted heap bytes. That checkpoint left 306 raw work units.
[Trusted initialization](docs/bootstrap-budget.md) now has a separate finite
1,000,000-unit allowance. Author entries retain 100,000 work units, and both
phases share the unchanged cumulative 8 MiB allocation budget.

[Throw diagnostics](docs/throw-completion.md) retain the original JavaScript value
without invoking constructor or name accessors. Data-only metadata inspection
keeps ordinary exceptions from triggering unrelated getters or suspending
later callbacks; its work and storage use the existing runtime limits.

[Microtask callbacks](docs/microtasks.md) add `queueMicrotask` and cleanup
checkpoints for the existing single Window realm. FIFO jobs share the active
script/event work allowance and cumulative heap; host-listener jobs settle
before later listeners and default actions. Rendering remains callback-free,
and fatal resource errors keep unstarted jobs suspended until a fresh Runtime.
Timers, MutationObserver delivery and the full HTML event loop
remain unfinished.

[Linear gradient backgrounds](docs/linear-gradients.md) introduced one bounded
`linear-gradient()` layer with legacy sRGB colors, alpha, physical directions
and length/percentage stops. The software painter evaluates it directly, with
shared CSS/supports/style validation. The [native-gradient follow-up](docs/vulkan-gradients.md)
prepares opaque square-corner fragments on the CPU for the existing GPU image
shader; alpha, rounded gradients and gradients inside opacity scopes retain
whole-scene fallback. Try the [native example](examples/vulkan-gradients.html).
The [background geometry follow-up](docs/background-geometry.md) added single-layer
size, physical position and independent repeat axes, including atomic shorthand
resets. Try the [positioned stripe example](examples/background-geometry.html).

[CSS URL backgrounds](docs/css-url-backgrounds.md) introduced one loaded `url()` layer
with natural raster sizing, position and repeat controls. Winning declarations
use their [stylesheet or document base](docs/css-url-provenance.md), including
variable substitution and inheritance. Explicit resource preparation uses the
page's retained loader; read-only layout paints authenticated cached images.

[Page resource sessions](docs/page-resource-session.md) retain the original navigation
fetcher and image caches for the page lifetime. Request and decode allowances
remain cumulative, and inline SVG accounting includes cache-only rasters.
Background preparation consumes this same session without resetting its limits;
read-only layout and image-source mutations add no fetching.

[Bounded tiled raster painting](docs/tiled-image-raster.md) adds a display-list
primitive for already decoded images, with independent axis repetition,
positioned tiles and rounded clipping. It uses center-based nearest sampling.
The [Native background follow-up](docs/vulkan-backgrounds.md) admits square-corner
raster backgrounds, including alpha and opacity groups, through CPU-prepared
lookup tables and the existing GPU image shaders. Rounded backgrounds retain
whole-frame software fallback. URL backgrounds consume this primitive; the paint
command itself performs no fetching or decoding. Try the
[clickable background example](examples/vulkan-backgrounds.html).

[Rounded box decorations](docs/rounded-boxes.md) add a software display-list
foundation for elliptical fills and border rings using colors, gradients or
already decoded images. Fixed subpixel coverage shares the existing paint
budget; that checkpoint used whole-frame CPU fallback on native paths. The
foundation did not add CSS box selection; the [integration](docs/background-boxes.md)
follows below. Try the [direct Rust example](examples/rounded-boxes.rs).

[Inline physical edges](docs/inline-edges.md) make ordinary inline horizontal
padding, borders and signed margins participate in placement and wrapping.
Separate per-line content, padding and border geometry preserves nested owners
and positioned descendants. Those retained rectangles prepared the
[background box integration](docs/background-boxes.md) below; full line layout
remains outside this subset.

[Background box selection](docs/background-boxes.md) connected those two
prerequisites to single-layer `background-origin` and `background-clip`.
Border, padding and content boxes independently control image placement and
background painting; ordinary wrapped inlines use joined owner geometry with
physical end decorations. CSSOM now handles all seven supported background
longhands. That checkpoint introduced shaped fills and border rings with
whole-frame software fallback. Try the [interactive box example](examples/background-boxes.html).

[Native color box decorations](docs/native-box-color.md) now prepare bounded
coverage masks on the CPU for the existing Vulkan color and opacity-group
composition paths, including translucent fills and correlated border rings.
That checkpoint left image and gradient box paints on whole-frame software
fallback. Its recorded comparisons are offscreen and establish no
acquired-window or performance result.
The [rectangle follow-up](docs/native-box-fastpath.md) avoids mask allocation
for proven integer-aligned solid fills, allowing full-viewport backgrounds
through the same Vulkan rectangle shader and existing resource limits.
The [rounded-background follow-up](docs/vulkan-rounded-core.md) draws a proven
solid interior with that shader and masks only the edges. Selected large
rounded Color boxes now fit the unchanged mask allowance, including source
alpha and opacity groups.

The [image box follow-up](docs/native-box-image.md) sends proven integer-aligned,
square image boxes through the existing Vulkan image kernel. Background origin
and clip remain independent; repeat phase, source alpha and opacity groups use
the existing sampling path. Rounded or fractionally translated image boxes
and cutouts retain whole-frame fallback. That checkpoint left gradient boxes
unsupported; the follow-up below admits a narrow subset. No fetching or shader
change is introduced.

The [gradient box follow-up](docs/native-box-gradient.md) prepares opaque,
integer-aligned square boxes on the CPU for the existing Vulkan image kernel.
Selected origin/clip boxes, repeats and currentColor retain their software
samples. Rounded or fractional outer shapes, cutouts, translucent stops and
retained opacity scopes still fall back as a whole. Generated pixels share the
existing 1 MiB source allowance; gradient interpolation remains on the CPU.

A dedicated [background window check](docs/native-background-window.md) combines
color, image and gradient boxes in one confined page. On the tested NVIDIA adapter,
one acquired Vulkan texture matched all 4,505,600 Canvas bytes before compositor
output; a separate rounded-image case confirmed whole-frame CPU fallback.
The evidence retains four earlier failed host attempts and the final successful run.

[Layered backgrounds](docs/background-layers.md) extend those foundations to
at most eight `none`, linear-gradient or URL image slots. Size, position, repeat,
origin and clip lists retain their own lengths and cycle only when used; the
first image paints on top, with one color beneath the last layer. Principal,
canvas, table and ordinary inline painting share the existing resource and
rendering limits. CSSOM preserves authored cardinalities and conservatively
returns an empty shorthand when it cannot represent them. Try the
[layer-order demo](examples/background-layers.html).

[Table spacing](docs/table-spacing.md) adds one or two inherited `border-spacing`
lengths to the existing table layout. Horizontal and vertical gaps now affect
tracks, spans and final cell backgrounds independently; HTML tables retain
2px gaps by default. CSSOM and supports checks share the bounded length/calc
parser. Intrinsic table sizing and caption handling remain limited, and
`border-collapse` is unsupported. Try the [spacing demo](examples/table-spacing.html).

[Row backgrounds](docs/table-row-backgrounds.md) now paint through cells
originating in each row, leaving inter-cell gaps and missing slots clear.
Rowspans keep their originating row's paint; gradient and URL tiles retain one
row-based phase across all clips. This uses the existing shared budgets and
can require more commands on painted rows. Try the
[row-background demo](examples/table-row-backgrounds.html).

[Persistent inline declarations](docs/cssom-pending.md) let scripts replace or
remove one part of a pending `var()` shorthand, preserving the other parts and
their individual priorities across later script calls and renders. Longhand
indices now reflect those retained parts. Text replacement deliberately reparses
serialized values; full CSSOM serialization remains incomplete. Try the
[interactive shorthand example](examples/cssom-pending.html). Separately authored
variable longhands now return an empty shorthand getter; removing that shorthand
still removes its parts. See the [getter follow-up](docs/cssom-var-getter.md).
[Mixed CSS-wide components](docs/cssom-wide-getter.md) also refuse shorthand
synthesis; uniform keywords such as `margin: initial` remain readable.

[Image fitting and positioning](docs/image-fitting.md) add `object-fit` and
physical `object-position` to loaded `<img>` content, including decoded external
SVG images. Fitting preserves the element’s box and clips its image content;
CSS supports checks, cascade and inline style mutation share bounded validation.
Try the [clickable example](examples/object-fit-position.html). This does not
extend inline SVG, video or general CSS conformance. The
[identity-sampling follow-up](docs/image-fitting.md#identity-sampling-follow-up)
avoids skipped source rows or columns on image axes painted at their source
extent, including rasterized SVG; scaled axes retain their existing sampling.

The [ArrayBuffer checkpoint](tests/conformance/array-buffer.md) adds fixed and
resizable buffers, resizing, same-realm species-aware slicing, transfers and
detachment. [DataView](tests/conformance/data-view.md) now adds fixed and tracking
views, nine Number codec pairs including Float16, and authentic `isView`.
Its complete upstream profile records 694 passes, 12 failures and 416 exclusions.
[Number TypedArray foundations](docs/typedarray-foundation.md) add all ten
Number element types, constructors, live indexed properties and iterators over
fixed or resizable buffers. [TypedArray views and string conversion](docs/typedarray-views-test262.md)
add `subarray`, `join`, and the exact `Array.prototype.toString` function alias.
The local matrix records 46 passing case modes and 24 verified controls; it is
the views increment's 70-mode CI gate. The upstream report records 46 passes,
64 work-limit stops and 96 exclusions, with all 56 controls verified. Its
resource outcomes prevent an upstream baseline.
The [TypedArray search follow-up](docs/typedarray-search.md) adds `at`,
`includes`, `indexOf` and `lastIndexOf` for Number views, including relative
indices and fresh buffer checks after index conversion. `includes` matches
NaN; the two index searches use strict equality.
[`fill`](docs/typedarray-fill.md) writes ranges through the same shared storage
and rechecks resized buffers after argument conversion. Its
[encoding optimization](docs/typedarray-fill-encoding.md) prepares one scalar
per nonempty call while retaining checked writes and unchanged conformance results.
[`reverse`](docs/typedarray-reverse.md) reverses a Number view in place, with
checked scalar writes and shared-buffer visibility. Its complete upstream
profile records 22 passes, four work-limit stops and 18 exclusions.
[`toReversed`](docs/typedarray-to-reversed.md) creates a reversed Number view
with independent fixed storage and leaves its source unchanged. Its complete
18-mode upstream profile records eight passes, eight work-limit stops and two
exclusions.
[`copyWithin`](docs/typedarray-copy-within.md) copies overlapping ranges within
Number views while preserving raw floating-point bits and visibility through aliased views.
Its complete 130-mode profile records 20 passes, 46 work-limit stops and 64
exclusions, with all 56 controls verified. Two inline visited names per string
length reduce enumeration allocations; larger buckets still use a tree.
Remaining bulk methods, BigInt codecs and shared memory remain unfinished.
The [one-pass own-key snapshot](docs/own-keys-one-pass.md) removes a duplicate
scan while preserving all 45 conformance reports. Its local timings show small
mixed changes, including p95 regressions; no general speedup is established.
[Reflect property operations](docs/reflect-properties.md) add nine methods,
including receiver-aware `get`/`set`, descriptors, prototype changes and Boolean
integrity checks. Their frozen upstream profile passes 200 modes and retains
16 Proxy exclusions. These methods also improve TypedArray compatibility;
remaining failures and work-limit stops are recorded separately.
[Decimal Number formatting](docs/number-format.md) now selects the even decimal
significand at exact shortest-format ties, including JSON and property names.
[Object.is](tests/conformance/object-is.md) now passes its complete 42-case
upstream profile. The [DOM method follow-up](tests/conformance/dom-method-identity.md)
separates Document, Element and DocumentFragment function identities and
checks their receivers before conversion. [Document.append](docs/document-append.md)
now shares checked insertion with Element and DocumentFragment, including
hierarchy errors and ordered partial effects. [DOM own properties](docs/dom-own-properties.md)
now support per-object data, accessors and method replacement.
[DOM interface prototypes](docs/dom-prototypes.md) add represented HTML, SVG and
MathML inheritance, mutable ParentNode operations and genuine Text, Comment and
DocumentFragment construction. [Processing instructions](docs/processing-instruction.md)
now add construction, the Document factory, target and CharacterData data/length
accessors. Five [CharacterData methods](docs/character-data.md) add UTF-16
substring and checked append, insert, delete and replace operations. The
[DOM string foundation](docs/dom-strings.md) preserves exact character data
in canonical storage, tagged worker transport, reads and clones.
[Exact JavaScript data production](docs/dom-production.md) now covers Text,
Comment and PI constructors, the PI and text factories, the data setter and all
five CharacterData methods, including unpaired units and complete-splice repairs.
That producer checkpoint passes all eight earlier unmet modes and all 24 new
cases. [Node data accessors](docs/node-data.md) now add ordinary `nodeValue` and
`textContent`, exact descendant reads and checked container replacement with
fresh Text identity. [Exact append strings](docs/append-domstrings.md) preserve
each argument in a separate Text, including unmatched units and retained failure
prefixes. [Document.title](docs/document-title.md) now has ordinary accessors,
exact normalized reads and staged raw-text replacement for HTML and SVG roots.
[Text operations](docs/text-operations.md) now add exact `splitText` and adjacent
`wholeText` reads, retaining node identities and separate failure stages.
[Node.normalize](docs/node-normalize.md) merges exact descendant Text runs and
removes empty Texts while retaining detached nodes and their data. Template
content remains separate; resource refusal can preserve completed mutation stages.
[Node predicates](docs/node-predicates.md) add ordinary `hasChildNodes`,
`isSameNode` and `contains` methods over authentic nodes and current parent links.
Required nullable arguments do not invoke conversion hooks; template content
remains a separate tree. All 32 new strict/sloppy modes pass with 16 healthy
controls; historical prototype-inventory failures remain recorded separately.
[Node.getRootNode](docs/node-root.md) adds ordinary roots after options conversion,
with one member read for object options, preserving callback effects and separate template-content
roots. All 32 release cases and 16 controls pass; historical inventory failures
remain recorded separately.
[Node.isEqualNode](docs/node-equality.md) compares exact character data, represented
element attributes and ordered ordinary children. Template contents are compared
only when passed separately. All 32 release cases and 16 controls pass; historical
prototype-inventory failures remain recorded separately.
[Node.isConnected](docs/node-connected.md) adds an ordinary readonly getter over
current parent links. It shares the bounded root walk; template content remains
disconnected from its host. All 32 release cases and 16 controls pass, with a
real-click branch-move witness in Page and the confined worker.
[Node.compareDocumentPosition](docs/node-position.md) adds ordinary position masks
for authentic nodes, with required nonnullable input and current ordinary-tree
order. Disconnected directions are stable within the represented arena, and
template content remains separate. All 32 release case modes and 16 controls
pass; six historical inventory losses remain recorded with their raw failures.
[Node.cloneNode](docs/node-clone.md) now makes checked detached copies with fresh
identities, exact stored character data and separate template content. The same
increment adds `Object.hasOwn` for represented own-property descriptors, including
DOM expandos. Its complete pinned [Object.hasOwn profile](tests/conformance/object-has-own.md)
now passes all 124 modes with 64 healthy controls. Independent Document ownership
and complete cloning state remain unfinished.
[Batched Node constants](docs/node-constants-bootstrap.md) now reduce runtime
initialization work and cumulative allocation charges while preserving the
existing JavaScript observations. Other legacy writers, complete interface members
and Document reflection remain unfinished. The
[Vulkan snapshot bridge](tools/vulkan-raster-probe/BROWSER_BRIDGE.md) now checks
real browser display lists with the custom GPU rasterizer and whole-frame CPU
fallback. Its 16 cases pass on three Vulkan adapters. A separate
[glyph path](tools/vulkan-raster-probe/GLYPHS.md) now composites bounded bundled-font
masks on the GPU; all 26 literal-mask and font-reference cases pass on those
adapters. The [worker-text checker](tools/vulkan-raster-probe/WORKER_TEXT.md)
also passes seven actual HTML snapshots on all three adapters, including
whole-frame fallback. The optional Linux [native GPU route](docs/vulkan-native-window.md)
now draws admitted loaded pages, overlays and browser controls into the actual
window using the custom shaders. Wide rounded shapes now split into bounded
adjacent masks. One 1280×880 acquired texture matches all 4,505,600 CPU-reference
bytes on the NVIDIA host; a separate normal run uses no CPU reference or readback.
Earlier 1180×880 evidence remains retained. Opacity and over-budget scenes take
complete CPU fallback. Font and rounded coverage still come from CPU preparation.
Native page zoom now uses bounded command scaling. Separate 90% and 110%
window checks match 9,011,200 acquired bytes against the CPU reference, and
a normal 110% run uses no reference or readback; see the
[zoom record](docs/vulkan-native-window.md#zoom-validation).

[Native Line accounting](docs/vulkan-line-accounting.md) now reserves each
Line's clipped rectangle loop area. Forty 24×1 Lines at 512×512 use 960 pixel
work units instead of 10,485,760, within the unchanged allowance. The existing
pixels, shaders and whole-scene fallback remain in place.

The [rendering docket](docs/vulkan-rendering.md) tracks broader admission,
compositing and performance work. There is no measured Chromium parity.

The experimental [native frame benchmark](docs/native-render-benchmark.md)
records completion-paced CPU-upload or shader frames with exact scene identity
and separate correctness checks. The original CPU/native comparison is retained.
A later [native buffer-reuse comparison](docs/evidence/vulkan-native-buffer-reuse.json)
completes another 96 timed frames and two exact acquired-pixel checks. For that
single fixture, the reuse candidate's subsequent preparation-through-present
median is 1.309 ms versus 2.086 ms baseline, and p95 is 4.117 ms versus 4.790 ms.
Early second-frame tails remain included; changing, unlocked GPU clocks limit
attribution. This is no
general browser or Chromium performance result.

Drawing plans and GPU encoding live in the reusable
[raster core](crates/raster-core/README.md), separate from comparison tools.
Its earlier [native-size conversion checks](tools/vulkan-raster-probe/NATIVE_PREREQUISITES.md)
remain independent offscreen evidence in both BGRA and RGBA on three adapters.

## Open the browser

Clone the public repository:

```sh
git clone https://github.com/Eriskii/ErisBrowser.git
cd ErisBrowser
```

Then launch the home page or one of the examples:

```sh
./run.sh
./run.sh https://example.com
./run.sh ./examples/forms.html
./run.sh ./examples/standards.html
./run.sh ./examples/unicode.html
./run.sh ./examples/namespaces.html
./run.sh ./examples/flow.html
./run.sh ./examples/templates.html
./run.sh ./examples/positioning.html
./run.sh ./examples/events.html
./run.sh ./examples/responsive.html
./run.sh ./examples/disclosures.html
```

The launcher builds the release executable and exposes installed desktop libraries on NixOS. Rust, Cargo, Python 3, and a Wayland or X11 desktop are required. Native browsing currently requires Linux with Landlock ABI 6 enabled (normally kernel 6.12 or newer), mounted procfs, and seccomp support; sandbox setup fails closed. The validated platform is x86-64 Linux. Fonts are bundled. On a conventional desktop with the shared libraries available:

```sh
cargo run --locked --release -- https://example.com
```

`./run.sh` does not install anything, change system settings, or use another browser to render pages. The `shell.nix` file offers a Nix development environment. Headless rendering does not require a display server. The `--render`, `--benchmark` and library paths execute in their own calling process and do not install the native page sandbox. `--benchmark-worker` uses the confined worker and broker, with the same Linux requirements as native browsing.

## Use it

- Enter a URL or local HTML file with **Ctrl+L**, then Enter.
- **Alt+Left / Alt+Right** navigate history; **Ctrl+R** or F5 reloads.
- Scroll with the wheel, arrow keys, Page Up/Down, Home, and End.
- **Ctrl+Plus / Ctrl+Minus / Ctrl+0** change page zoom.
- Click links and buttons; Tab moves among basic form fields, links and disclosure summaries.
- Text fields support typing, cursor movement, Shift-selection, Backspace/Delete, and native Ctrl+C/X/V clipboard shortcuts. Basic GET and URL-encoded POST forms are supported.
- The home page's **Add one** button executes this engine's own script interpreter.
- `--no-scripts` disables scripting. Diagnostics are printed to the terminal.

Text editing is based on Unicode scalar boundaries; grapheme-aware movement, rich editing, full IME behavior, and accessibility are unfinished. Clipboard support uses the native platform backend (X11/XWayland on Linux). History and reload use GET and do not automatically resubmit POST bodies.

## Render and measure

```sh
cargo run --locked --release -- --render --output artifacts/home.png
cargo run --locked --release -- --render examples/forms.html --output artifacts/forms.png
cargo run --locked --release -- --render --click '#increment' --click '#increment' --output artifacts/counter.png
cargo run --locked --release -- --render --width 600 --height 1000 --output artifacts/narrow.png
cargo run --locked --release -- --benchmark 100 --output artifacts/benchmark.png
cargo run --locked --release -- --benchmark-worker 100 --output artifacts/worker-benchmark.png
```

`--dump-dom` prints the resulting DOM. `--window-screenshot artifacts/window.png --exit-after 5` captures the browser's own framebuffer during a short native-window smoke test. Run `--help` for CLI details.

The native UI loads each document in a fresh child process. Its address bar, clipboard and software painter stay in the UI process; a separate broker supplies resources and authoritative redirect URLs, while validated document and drawing snapshots return from the renderer. The renderer cannot directly read files or open sockets. Cross-origin images use a fresh restricted decoder, which returns pixels without exposing their raw responses to the renderer. See [the security boundary](docs/SECURITY.md) for remaining gaps.

For a JSON report covering the home, gallery, form, template/Grid, positioning, event/compositing, responsive and disclosure fixtures, including a scrolled view with visible opacity, run `python3 tools/benchmark.py`.

The benchmark measures **warm-cache CSS computation + cache binding + layout + software painting of the loaded page**. URL backgrounds are prepared once before timed iterations. The samples exclude that preparation, parsing, scripts, network, image decoding, PNG encoding, and native presentation. It is not a Chromium comparison or a general web-performance score. See [performance](docs/PERFORMANCE.md).

`python3 tools/benchmark_worker.py` separately measures fresh confined process startup, document loading, validated snapshot round trips, software painting and teardown. It saves raw samples and environment/build hashes. It does not open a window or measure native presentation.

## Check the implementation

```sh
cargo fmt --all -- --check
python3 tools/generate_js_identifiers.py --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo test --locked -- --include-ignored
cargo build --locked --release
python3 tools/reftest.py --binary target/release/eris-browser
python3 tools/html_conformance.py --baseline tests/conformance/html-tree-current.json
python3 tools/test262_conformance.py --baseline tests/conformance/test262-current.json
python3 tools/test262_conformance.py --profile regexp --baseline tests/conformance/test262-regexp-current.json
python3 tools/test262_conformance.py --profile template-literal --baseline tests/conformance/test262-template-literal-current.json
python3 tools/test262_conformance.py --profile functions --baseline tests/conformance/test262-functions-current.json
python3 tools/test262_conformance.py --profile rest-parameters --baseline tests/conformance/test262-rest-parameters-current.json
python3 tools/test262_conformance.py --profile is-prototype-of --baseline tests/conformance/test262-is-prototype-of-current.json
python3 tools/test262_conformance.py --profile global-values --baseline tests/conformance/test262-global-values-current.json
python3 tools/test262_conformance.py --profile array-reduce --baseline tests/conformance/test262-array-reduce-current.json
python3 tools/test262_conformance.py --profile number-statics --baseline tests/conformance/test262-number-statics-current.json
python3 tools/test262_conformance.py --profile numeric-conversion --baseline tests/conformance/test262-numeric-conversion-current.json
python3 tools/test262_conformance.py --profile compound-assignment --baseline tests/conformance/test262-compound-assignment-current.json
python3 tools/test262_conformance.py --profile addition --baseline tests/conformance/test262-addition-current.json
python3 tools/test262_conformance.py --profile logical-assignment --baseline tests/conformance/test262-logical-assignment-current.json
python3 tools/test262_conformance.py --profile relational --baseline tests/conformance/test262-relational-current.json
python3 tools/test262_conformance.py --profile equality --baseline tests/conformance/test262-equality-current.json
python3 tools/test262_conformance.py --profile labels --baseline tests/conformance/test262-labels-current.json
python3 tools/test262_conformance.py --profile symbols --baseline tests/conformance/test262-symbols-current.json
python3 tools/test262_conformance.py --profile object-integrity --baseline tests/conformance/test262-object-integrity-current.json
python3 tools/test262_conformance.py --profile array-find --baseline tests/conformance/test262-array-find-current.json
python3 tools/test262_conformance.py --profile array-splice --baseline tests/conformance/test262-array-splice-current.json
python3 tools/test262_conformance.py --profile array-buffer --baseline tests/conformance/test262-array-buffer-current.json
python3 tools/test262_conformance.py --profile data-view --baseline tests/conformance/test262-data-view-current.json
python3 tools/test262_conformance.py --profile date --baseline tests/conformance/test262-date-current.json
python3 -m unittest discover -s tools -p 'test_*.py'
cargo run --locked --release --bin eris-stress -- 5000
```

Network and sandbox integration tests explicitly opt in: they bind temporary loopback HTTP servers or require the Linux sandbox features above. They require no public network. Other tests cover parser recovery, selectors and cascade, box layout, DOM/script interaction, script exhaustion, file scopes, geometry and raster limits. Reference tests compare independently constructed pages pixel-for-pixel with this renderer. They are a small self-authored suite, not a claim to pass the Web Platform Tests. The deterministic stress harness mutates HTML/CSS, script, and SVG seeds and checks bounded-output invariants; it is smoke fuzzing, not coverage-guided fuzzing.

CI uses a Cargo target runner to close ambient build/runner descriptors before
each Rust test executable starts. Worker rejection of unexpected descriptors,
including descriptors deliberately opened by a test, remains enforced.
An additional job compiles all engine targets with the package's declared
minimum Rust version, 1.88, using the checked-in lockfile.

The [validation record](docs/VALIDATION.md) lists observed results and their limits. A pinned upstream HTML tree corpus now provides exact-tree comparisons and a regression baseline; [its documentation](tests/conformance/README.md) records all mismatches, unsupported modes, and untested semantics. A pinned [Test262 selection](tests/conformance/test262.md) runs unchanged upstream tests and assertion harnesses with explicit failure/unsupported categories. Neither runner establishes platform-wide compatibility.

A separate [function inventory](tests/conformance/test262-functions.md) retains
663 unchanged Test262 sources and 1,131 required variants. Default parameters,
prototype membership, Window.self, array sorting, identifier parsing and bounded
grammar continuations and dynamic constructor coverage bring it to 546 passing variants, 196 more than the initial
measurement with no lost passes. Both original 32-nested-function tests now pass.
The remaining 616 variants are unsupported; its healthy baseline runs in CI.

A separate complete [rest-parameter directory](tests/conformance/test262-rest-parameters.md)
retains 22 variants: 16 pass and six remain unsupported. Its assertion-checked
regression baseline runs in CI.

The complete [prototype-membership directory](tests/conformance/test262-is-prototype-of.md)
adds 20 variants: 14 pass and six require unsupported features. Its separate
CI gate requires all method assertion preflights to pass.

The complete [global-value directories](tests/conformance/test262-global-values.md)
retain 88 variants: 70 pass and 18 remain unsupported.
Global-property corrections and URI bindings add ten passes; Window reflection adds 22 more. All 64 assertion
preflights verify; CI preserves those passes and the unchanged inventory.

The complete [array-sort directory](tests/conformance/test262-array-sort.md)
retains 107 variants: 59 pass, 44 remain unsupported, and four exceed the shared
script budget. Adding reduce enables the 5- and 11-element stability tests in
both modes. Those resource stops prevent a healthy
regression baseline; the full inventory remains part of local measurements.

The complete paired [reduce/reduceRight directories](tests/conformance/test262-array-reduce.md)
retain 1,034 variants: 1,018 pass and 16 remain unsupported. Both methods and Number.MAX_SAFE_INTEGER add 764
passes with no losses; Symbol-based Math/JSON tags add eight more. All 128 assertion checks verify; CI preserves the passing
cases and the full unchanged inventory.

The [Number static builtin inventory](tests/conformance/test262-number-statics.md)
retains 340 variants: 260 pass and 80 remain unsupported. Constants, static
predicates, ordinary constructor conversion and parsing aliases add 90 passes
with no losses; Window reflection adds two more. All 104 assertion checks verify, and
the complete regression baseline runs in CI.

The complete [coercing global predicate directories](tests/conformance/test262-numeric-conversion.md)
retain 60 variants: 32 pass and 28 remain unsupported. Ordinary numeric conversion
adds eight passes with no losses; Window reflection adds four more. All 80 assertion controls verify, and CI
preserves the complete inventory and passing outcomes.

The complete [numeric parsing directories](tests/conformance/test262-numeric-parsing.md)
retain 218 variants: 176 pass, 34 remain unsupported and eight exceed the
instruction budget. Compound assignment support lets the unchanged helper load
and its dependent loops run. All 80 assertion controls verify; resource stops
prevent a healthy regression baseline.

The complete [compound-assignment directory](tests/conformance/test262-compound-assignment.md)
retains 786 variants: 606 pass and 180 remain unsupported. Six bitwise assignments
and ordinary addition conversion add 316 passes without losses. All 128 assertion
controls verify; CI preserves the complete baseline.

The complete [addition directory](tests/conformance/test262-addition.md) retains
95 variants: 67 pass and 28 remain unsupported. Ordinary
conversion adds 14 passes with no losses. All 64 assertion controls verify; CI
preserves the full baseline and its remaining nonpassing cases.

The complete [logical-assignment directory](tests/conformance/test262-logical-assignment.md)
retains 132 variants: 78 pass, six fail on class prerequisites and 48
remain unsupported. The three short-circuit assignments add 54 passes without
losses; Symbol primitive conversion adds six more. All 104 assertion controls verify; CI preserves the complete baseline.

The complete [labeled statement, break and continue directories](tests/conformance/test262-labels.md)
retain 125 variants: 100 pass and 25 need unsupported syntax or dynamic eval.
Ordinary targets, early errors and statement-context ASI add 70 passes.
All 80 assertion controls verify; CI preserves the complete inventory.

[Statement completion values](tests/conformance/statement-completion.md) preserve
results through empty statements, declarations, blocks, jumps and finally clauses.
Direct interpreter tests cover these values without requiring dynamic eval.

The four complete [equality directories](tests/conformance/test262-equality.md)
retain 286 variants: 190 pass and 96 require exotic values or dynamic eval.
Ordinary coercion and nullish dispatch add 44 passes. All 128 assertion controls
verify; CI preserves the gains and complete inventory.

The four complete [relational comparison directories](tests/conformance/test262-relational.md)
retain 364 variants: 300 pass and 64 remain outside this profile's feature policy, including Symbol-tagged cases.
Ordinary conversion adds eight passes; all 128 assertion controls verify.
CI preserves the passes and unchanged inventory.

The complete [URI builtin directories](tests/conformance/test262-uri.md) retain
346 variants: 234 pass and 112 hit the instruction limit.
All 128 assertion controls verify. Full loops remain unchanged; resource stops
prevent a healthy URI regression baseline.

The complete [identifier and whitespace directories](tests/conformance/test262-identifiers.md)
retain 669 variants: 509 pass, 154 remain unsupported and six reach compile
limits. All 88 assertion checks verify. The identifier and flat-code changes add
132 passes without losing earlier passes; retained preliminary results document 16 resource
regressions that were resolved by reducing actual lookup work and token storage.
The remaining resource stops prevent a healthy regression baseline.

The complete [Symbol tree and key-reflection inventory](tests/conformance/symbols.md)
adds 123 sources / 242 modes: 182 pass, six fail and 54 remain unsupported, with
80 verified controls. Symbol identities, UTF-16 descriptions/registry keys,
symbol properties, primitive/tag/instance hooks and key reflection now work.
Their broader protocols remain incomplete. The [full comparison](tests/conformance/symbol-properties.json)
retains all results; CI preserves the new passing-case baseline.

Supported [DOM operations](tests/conformance/dom-string-conversion.md) now honor
borrowed method receivers and perform JavaScript string conversion before their
DOM effects. Attribute/append/class-list conversion preserves callback order and
thrown values; nullable strings and Boolean setters use their respective rules.
The original probes gain 16 passes with no regression in the existing inventories.
[Exact character-data production](docs/dom-production.md) now covers the named
constructors, factories, data setter and CharacterData methods.
[Node data accessors](docs/node-data.md) add exact `nodeValue`/`textContent` with
ordinary descriptor behavior and retained detached subtrees.
[ParentNode.append](docs/append-domstrings.md) now keeps exact separate string
arguments after all conversions finish. [Document.title](docs/document-title.md)
now preserves exact strings through ordinary accessors and normalized reads.
[Node.normalize](docs/node-normalize.md) adds exact descendant Text merging and
empty removal through an ordinary method. CDATA, live Range updates and mutation
observers remain unsupported; the historical prototype-inventory changes are
retained separately from the new feature checks.
[Node predicates](docs/node-predicates.md) read direct children, identity and
inclusive ancestry without mutating the tree. Their bounded local checks and
visited sibling scans consume the existing author work allowance; prototype
inventory changes retain explicit historical observations. The new 32 modes
and 16 controls pass without changing numeric quotas.
[Node.getRootNode](docs/node-root.md) reads fresh parent links after dictionary
conversion, with bounded ancestor and sibling scans. Actual ShadowRoot and
shadow-including ancestry remain absent. Default and native Vulkan validation pass.
[Node equality](docs/node-equality.md) compares internal structure without authored
getters or coercion. Its local graph checks and paid traversal retain the existing
quotas. Default and native Vulkan suites pass, including both browser witnesses.
Legacy `innerText`, textarea setters and attribute writes, HTML parser input
and broader DOMString/Web IDL interfaces remain incomplete.

Supported [Window binding reflection](tests/conformance/window-reflection.md) now
preserves numeric/string/symbol key order, intrinsic flags and live for-in behavior.
The unchanged upstream inventories gain 54 passes without losing previous passes.
[General Window properties and global bindings](tests/conformance/window-global-bindings.md)
now add data/accessor definitions, exact UTF-16 keys and private execution receivers.
This follow-up gains another 13 upstream passes and completes all 64 frozen local
Window probe modes. Full Window interfaces and extensibility remain incomplete.

[String.prototype.concat](tests/conformance/string-concat.md) now performs ordered
string conversion and preserves UTF-16 units using one final output buffer.
Its complete 22-source Test262 directory now records 44 passed modes after the
[constructor-policy review](tests/conformance/constructor-policy.md); all 56
assertion controls verify. Earlier checkpoint records preserve their original policies.

[Constructor targets and Reflect calls](tests/conformance/construction.md) now
support `new.target`, `Reflect.apply` and `Reflect.construct`, including lexical
arrow capture and bound construction. Two complete pinned Test262 selections
now record 58 passes across 66 modes; eight remain unsupported. Alternate Web IDL targets and broader constructor semantics
remain incomplete.

Symbol now works as an alternate constructor target while its own construction
still throws. The [reviewed policy expansion](tests/conformance/constructor-policy.md)
enables 88 older Test262 modes: 86 already passed, and two pass after the Symbol
fix. All 7,231 modes preserve prior passes; all 2,008 assertion controls verify.


[Dynamic Function construction](tests/conformance/function-constructor.md) now
compiles supported ordinary functions in global scope with separate parameter/body
parsing and shared resource limits. Its complete 200-source pinned selection gains
157 passes, reaching 241 passed and 50 unsupported modes; older selections gain
35 more passes. All 7,522 mode fingerprints and prior passes are preserved, and all
2,072 assertion controls verify. Function source printing and literal unpaired
surrogates in generated source remain incomplete.

[String conversion and custom splitting](tests/conformance/string-conversion.md)
fix the two exposed slice failures and add 18 passes in a complete new 242-source
String search/split selection, which records 470 passed, two failed and 12
unsupported modes. Generic receivers and needles use ordinary string conversion;
search methods consult `Symbol.match`, and custom `Symbol.split` hooks receive
the original arguments. All 8,006 mode fingerprints and prior passes remain
preserved, with 2,144 verified assertion controls.

[RegExp splitting and species construction](tests/conformance/regexp-split.md)
now honor constructor, flags, execution and capture hooks, adding 86 passes in a
complete new 48-source selection. It now records 90 passed and six
unsupported modes. All 8,006 older case observations remain identical; the full
inventory reaches 8,102 modes and 2,216 verified assertion controls. Unicode
regexp parsing, realms and the other String symbol protocols remain incomplete.

[RegExp constructor conversion](tests/conformance/regexp-constructor.md) now
classifies through `Symbol.match` and preserves source/flags/prototype/conversion
order. The complete new 488-source directory gains 16 passes, reaching 768 passed,
200 unsupported and eight resource stops. All 8,102 older mode observations stay
identical; the combined inventory reaches 9,078 modes and 2,288 verified controls.
The resource stops keep this profile an observation report, with its limits visible.

[Flat regexp group parsing](tests/conformance/regexp-deep-groups.md) subsequently
adds four passes for 200 nested groups, bringing that directory to 772 passed,
200 unsupported and four resource stops. Both parsing and prefix analysis use
budgeted heap frames; all other 9,074 observations and 2,288 controls stay identical.

[Required-literal rejection](tests/conformance/regexp-required-literals.md) adds
both XML-pattern modes by rejecting impossible matches before backtracking. The
directory now records 774 passed, 200 unsupported and two script-work resource
stops, with unchanged budgets. All other 9,076 observations remain identical.

[String and RegExp match/search protocols](tests/conformance/regexp-match-search.md)
now honor custom symbol hooks, conversion order, global empty-match advancement
and exact search index restoration. A complete new 170-source inventory gains
138 passes, reaching 304 passed, 16 failed and 20 unsupported modes. All 9,078
older observations remain identical; the full inventory reaches 9,418 modes and
2,368 verified controls. Missing lastIndexOf and BigInt dependencies remain visible.

## Implementation

The [architecture notes](docs/ARCHITECTURE.md) describe the page pipeline and native process boundaries.
The optional [Linux Vulkan presenter](docs/vulkan-rendering.md) uploads CPU-painted
frames when built with `--features vulkan-presenter` and launched with
`--presenter=vulkan`. Build with `--features vulkan-raster` and launch with
`--presenter=vulkan --raster=gpu` for custom shaders in the actual window, with
CPU-prepared font/rounded coverage and whole-frame fallback. One exact-size
retired GPU buffer set can be reused; every frame still rewrites inputs and
draws the complete scene. [Nested opacity](docs/vulkan-full-opacity.md) uses
cropped RGBA16 intermediates for transparent or opaque groups at every finite
binary32 opacity in `[0,1]`, preserving the software painter’s rounding stages.
[Precise rectangle reservations](docs/native-rect-accounting.md) admit more
small-shape scenes under the existing CPU work limits. The
[image accounting follow-up](docs/native-image-accounting.md) reserves clipped
image sampling loops under the same limits, with a clickable forty-image
example. Its Page and confined-worker checks verify the complete pixels before
and after a real click. A separate 1200×900 NVIDIA acquired texture matches all
4,320,000 Canvas-reference bytes before the compositor. No timing result is
claimed for this change.
Software remains the default and headless path. The
separate [raster probe](tools/vulkan-raster-probe/README.md) retains its original
30 cases and 26 glyph cases on three adapters. Broader text, compositing
and scene coverage remain on the [development docket](docs/ROADMAP.md); the
narrow timing comparisons do not establish Chromium parity.

JavaScript [parses directly into flat code records](tests/conformance/flat-parser.md)
and uses [shared execution continuations](tests/conformance/activation-frames.md),
including ordinary calls and default initializers. Retained shallow recursion
probes now complete 32 calls, reaching the existing logical ceiling at call 33.
Native callbacks remain guarded. [Grammar continuations](tests/conformance/parser-continuations.md)
now also parse the original 32-nested-IIFE source within the unchanged compile
budget; both upstream modes pass. Partial syntax, pending grammar and completed
code release without recursive syntax ownership. Upstream inventories and policies
remain unchanged.

| Module | Responsibility |
|---|---|
| `dom.rs` | Bounded HTML tree construction, arena DOM, entities, selector matching, serialization |
| `css.rs` | CSS parsing, indexed cascade and layers, inheritance, lengths, colors, variables, media queries |
| `cssom.rs` | Bounded inline declaration storage, queries, priorities and staged mutation |
| `selectors.rs` | Bounded CSS tokens and selector grammar shared by matching and feature queries |
| `layout.rs` | Block/inline flow, floats, flex, Grid placement/tracks, tables, controls, display lists, hit regions |
| `script.rs`, `script/` | Custom lexer, flat parser/code storage, execution driver, lexical environments, DOM bindings and events |
| `js_identifier.rs` | Offline-generated Unicode identifier membership with pinned source data |
| `regexp.rs` | Custom bounded UTF-16 regular-expression parser and backtracking matcher |
| `svg.rs` | Custom SVG geometry, paths, transforms and bounded RGBA rasterization |
| `graphics.rs` | Font metrics, cached glyph masks, clipped/limited software painting, group opacity, PNG output |
| `net.rs` | HTTP/TLS resource loading, redirect and file policies, decoding and byte limits |
| `text_encoding.rs` | HTML encoding prescan/reparse selection, MIME charset parameters, CSS and classic script decoding |
| `document_url.rs`, `stylesheet_loading.rs` | Document URL resolution and bounded stylesheet import loading |
| `page.rs` | Resource ordering, page lifecycle, scripting, forms and layout integration |
| `worker.rs`, `worker/` | Per-document child processes, bounded binary IPC, validated snapshots and Linux confinement |
| `js_string.rs` | UTF-16 code-unit strings and explicit scalar-text conversion |
| `browser.rs` | Native window, address bar, history, input and worker coordination |
| `presenter.rs` | Checked completed CPU frames and exclusive ownership of the software surface |
| `edit.rs` | Unicode scalar cursor movement, selections, replacements and deletion |

Infrastructure dependencies provide TLS/HTTP (`ureq`/`rustls`), URLs (`url`), character encodings, font outline rasterization (`ab_glyph`), image codecs (`image`), native clipboard access (`arboard`), window events (`winit`), a pixel surface (`softbuffer`), optional Vulkan presentation (`wgpu`) and exec descriptor hygiene (`close_fds`), and OS confinement wrappers (`landlock`, `rustix`, `seccompiler`). These are not web layout or script engines. Their transitive dependencies remain part of the security surface. The source forbids application-level `unsafe` Rust; dependencies can contain unsafe code.

Fonts are DejaVu; redistribution notices are in [assets/FONTS-LICENSE.txt](assets/FONTS-LICENSE.txt). Project code is MIT licensed. Vendored WPT test data retains its [upstream BSD license](tests/upstream/wpt-html/LICENSE.md); Test262 data retains its [upstream license](tests/upstream/test262/LICENSE). Identifier tables and their pinned source data retain [Unicode License V3](tests/upstream/unicode/18.0.0/LICENSE.txt).

[String.lastIndexOf](tests/conformance/string-last-index-of.md) now performs
ordered conversion and bounded reverse UTF-16 search. Its complete 50-mode
inventory gains 46 passes; 12 existing match/search modes also pass. The two
remaining new failures call the missing Array.lastIndexOf method. All other
9,406 older observations are unchanged; 2,448 assertion controls verify.

[Array.lastIndexOf](tests/conformance/array-last-index-of.md) now supports generic
receivers, holes, inheritance and live reads. Its complete 395-mode inventory
originally gained 328 passes, and both older String.lastIndexOf failures pass.
Later follow-ups reach 385 passes; two resource stops and eight unsupported
modes remain visible. This inventory
does not qualify as a healthy baseline gate.


[Array descriptors and sparse storage](tests/conformance/array-descriptors.md)
now support indexed getters/setters, readonly length, nonextensibility and partial
length-reduction failures. Eleven existing Array methods use live property
operations, and Reflect.defineProperty shares the definition path. The complete
1,793-source upstream inventory gains 664 passes; older inventories gain another
233, with no prior pass lost. Custom species construction and broader web
compatibility remain unfinished.

[Array every and some](tests/conformance/array-predicates.md) now support generic
receivers, live sparse/inherited entries, captured length and callback
short-circuiting. All 164 frozen local variants pass, with eight verified
assertion controls. The [complete paired Test262 inventory](tests/conformance/test262-array-predicates.md)
adds 801 passes; the Date follow-up reaches 847 passed, four sparse-scan
work limits and 16 metadata exclusions across 867 modes. Four older descriptor
metadata modes also pass. All 14,304 mode fingerprints and prior passes are
preserved across 33 profiles; all 2,740 assertion controls verify. The existing
26 healthy gates remain, while the new resource stops prevent a predicate gate.

[Object integrity](tests/conformance/object-integrity.md) now supports sealing,
freezing and integrity queries for supported ECMAScript objects, including
arrays, functions, boxed strings and arguments objects. Own descriptor flags
are processed without invoking getters; children and prototypes remain shallow.
The [complete upstream profile](tests/conformance/test262-object-integrity.md)
retains 239 sources and 474 modes. With Date, ArrayBuffer and DataView support it records
398 passed, 38 missing-prerequisite failures and 38 unsupported modes, with all 224 controls
verified. All 228 frozen local variants and their 12 controls also pass.
Eight older descriptor modes gain passes, and all 14,296 other older
observations are unchanged. That checkpoint has 34 profiles, 14,778
modes, 2,964 verified controls and 27 healthy regression gates. Work and
allocation quotas are unchanged; failed and unsupported cases remain visible.

[Array find methods](tests/conformance/array-find.md) now support `find`,
`findIndex`, `findLast` and `findLastIndex` on ordinary generic receivers.
They visit holes through live property reads, preserve the captured length and
traversal direction, and return the value saved before a matching callback.
All 504 frozen local variants and twelve controls pass. The
[complete four-directory profile](tests/conformance/test262-array-find.md)
retains 94 sources and 180 modes. That checkpoint recorded 140 passes, eight
failures requiring missing `splice`, and 32 metadata exclusions, with all 288
controls verified. It added 116 passes and retained the other 64 complete records.
The splice follow-up closes all eight failures, reaching 148 passes.
At the find checkpoint, all 34 older profile contracts and observations remained
unchanged.
That checkpoint had 35 profiles, 14,958 modes, 3,252 verified controls
and 28 healthy regression gates, including the new find baseline. Shared
property reads precharge retained-tree comparisons and mapped binding-name
work. Actual-digit index formatting restores two initial lastIndexOf work-limit
regressions while preserving these charges and every quota; both attempts are
retained in the evidence. Failed and unsupported cases remain visible.

[Array.from](tests/conformance/array-from.md) now supports synchronous iterables
and array-like inputs, generic construction, live mapping, own data definitions
and iterator closing. Its independent local matrix verifies 388 of 402
expectations, including twelve expected resource stops; fourteen prerequisite
modes remain unmet. At that checkpoint the complete upstream profile had 82
passes, four failures requiring `splice` or `ArrayBuffer`, four metadata exclusions
and 96 verified controls. All 38 older profile observations remained unchanged.
The combined inventory then had 39 profiles, 17,822 modes, 3,900 verified controls
and 32 known-state regression gates. Splice brought Array.from to 84 passes;
ArrayBuffer closes its last two failures, reaching 86 passes and four exclusions.

[Shared enumeration accounting](tests/conformance/own-keys.md) preserves that
entire inventory while charging string-key snapshots, caller buffers and for-in
visited-name searches. Integer ranks and cached tree entries remove repeated
name comparisons without raising quotas. General live property-map and JSON
accounting remain open; this change adds no conformance passes.

[Array splice](tests/conformance/array-splice.md) now performs generic live
copying, ordered strict mutation and same-realm species construction, including
aliased results and partial errors. Its complete pinned profile records 138 passes
and 24 unchanged metadata exclusions across 162 modes; all 96 controls verify.
The local fixture verifies 90 of 96 expectations, including four expected resource
stops; Proxy, typed-array and cross-realm prerequisites remain unmet. Ten older
find/Array.from modes gain passes, with no other historical observation or control
changes. That checkpoint had 33 known-state gates; all final baseline projections
were checked against the completed reports. The
[evidence](tests/conformance/array-splice.json) retains all gaps and
unchanged limits; broader property accounting and full compatibility remain open.

[Array concat](tests/conformance/array-concat.md) supports generic receivers,
live `Symbol.isConcatSpreadable`, same-realm species and aliased results. Its
complete 137-mode profile records 113 passes, four missing-typed-array failures,
18 unsupported modes and two resource stops; all 96 controls verify. The local
fixture verifies 84 of 90 expectations, including four expected resource stops.
All 40 older profiles and six selected local suites remain exactly unchanged.
The two ordinary upstream resource outcomes prevent a new regression gate;
that checkpoint retained **33 known-state gates**, with no quota or policy changes.

[ArrayBuffer](tests/conformance/array-buffer.md) adds nonshared fixed and
resizable backing stores, slicing, transfers and detachment. Its complete
upstream profile records **262 passed, 50 failed and 130 excluded modes**, with
all 160 controls verified. The local suite verifies **96 of 104 expectations**
after the [for-in length-bucket follow-up](tests/conformance/for-in-length-buckets.md)
closes two metadata work stops and [DataView](tests/conformance/data-view.md)
closes two prerequisite modes. Eight prerequisite modes remain unmet.
Further TypedArray methods, BigInt codecs, shared memory and foreign realms remain unfinished. At the original ArrayBuffer checkpoint, four older Array.from/Object.seal
modes gained passes, with every other historical observation unchanged.
DataView adds 694 passes, 12 failures and 416 exclusions, with all 232 controls
verified. Its local suite verifies 128 of 138 expectations, including four
expected resource outcomes. At that checkpoint the catalog contained **43 profiles /
19,685 modes / 4,484 controls**, with **35 known-state regression gates**.
Those gates preserve recorded gaps as well as passing cases.

[Synchronous iteration](tests/conformance/for-of.md) supports identifier/member
`for…of` heads, lexical bindings, iterator closing and native Array, String and
arguments iterators. Destructuring, generators and async iteration remain ahead.

[Date](tests/conformance/date.md) now supports core construction, getters/setters,
UTC and local arithmetic, ISO and required own-string round trips, JSON and
primitive-conversion hooks, and legacy year/GMT aliases. Host timezone snapshots
carry historical transitions and recurring rules into the confined renderer
without filesystem grants. The complete 594-source Date tree records 1,166
passed modes and 22 metadata exclusions; all 340 controls and 406 independently
frozen local modes pass. Synchronous iteration closed its four year-zero parser gaps.
Intl, Temporal, cross-realm behavior and broader legacy parsing remain ahead.
The [readiness follow-up](tests/conformance/date-readiness.md) reduced helper
polling delay while preserving the earlier timing observations.
Full compatibility, security certification and the Chromium
performance target remain unfulfilled.
