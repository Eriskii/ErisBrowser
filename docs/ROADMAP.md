# Development docket

[Promise construction and reactions](promise-foundation.md) now include
thenable adoption, generic capabilities/species and a shared FIFO with
`queueMicrotask`. [Generic `catch`](promise-catch.md) and
[`finally` cleanup](promise-finally.md) are implemented. The optional
[Promise Test262 host](promise-test262-host.md) now supplies authenticated
`$DONE` completion and a final unhandled-rejection check for a bounded pinned
profile. Its recorded results guide subsequent compatibility work.
[`Promise.all`](promise-all.md) now implements ordered collection from synchronous
iterables within the existing limits. Its separate 98-source, 196-mode upstream
selection now [records 174 passed, 14 failed and 8 unsupported modes](promise-all-test262.md),
with all 90 harness controls passing. [Call/new spread arguments](call-spread.md)
close four prior syntax failures. Classes and the distinction between host
rejection policy and language conformance remain visible in this result.
The existing 458-mode profile has no regressions.
[`Promise.race`](promise-race.md) now reuses capabilities and the shared FIFO,
preserving direct callbacks and ordinary iterator closing. Its complete pinned
[94-source upstream profile](promise-race-test262.md) records 170 passed, 10 failed
and 8 unsupported modes out of 188, with all 90 controls verified. The retained
failures reflect host rejection policy; class support remains unfinished.
[`Promise.allSettled`](promise-all-settled.md) now collects both outcomes in
input order with generic capabilities and paired callbacks sharing one gate.
Its separate [104-source, 208-mode upstream profile](promise-all-settled-test262.md)
uses a 94-control gate. It records 188 passed, 12 failed and 8 unsupported,
with all 94 controls verified. The twelve failures retain the final-unhandled
host-policy reason, and eight class-bearing modes remain unsupported.
[`AggregateError`](aggregate-error.md) now supplies ordered iterable errors,
message/cause handling and genuine Error fallback branding under the existing
limits. `Promise.any` and host decoding of uncaught AggregateError identities
remain separate follow-ups; the constructor alone does not implement either.
Remaining asynchronous work
includes `any`, async functions/`await`, rejection notifications, timers,
MutationObserver delivery and multiple realms.

[Shared startup keys](pi-key-cache.md) reduce duplicate DOM metadata allocations
within the existing bootstrap allowance, making room for Promise installation.
The [job metadata follow-up](shared-job-keys.md) removes two more duplicate
strings. Installing `finally` uses 122 of the 127 work units it made available,
leaving five under the unchanged startup limit. The
[Node accessor follow-up](node-accessor-keys.md) saves another 128 units and
672 accounted heap bytes, leaving 133 units at that checkpoint. Installing
`Promise.all` uses 108 units and leaves 25 under the same startup allowance.
The [Node method follow-up](node-method-keys.md) saves another 250 work units and
1,512 accounted heap bytes, leaving 275 units. Installing `Promise.race` uses
121 units and leaves 154 under the unchanged startup allowance. The
[Text metadata follow-up](text-metadata-keys.md) saves 60 further work units and
336 accounted heap bytes, leaving 214 at that checkpoint. The subsequent
`allSettled` installation and its validation are recorded in the linked chapter.
The [startup metadata follow-up](startup-metadata-sharing.md) saves another 279
work units and 1,132 accounted heap bytes, leaving 306 at that checkpoint.
[Trusted initialization](bootstrap-budget.md) now has a separate finite
1,000,000-unit allowance; author entries retain 100,000 and cumulative heap
accounting remains 8 MiB. This policy headroom does not establish implementation
or measured admission for the remaining Promise APIs.
Call/new spread
uses the same runtime limits. [Argument grammar contexts](call-argument-in.md)
now restore `+In` inside argument lists nested in a `for` initializer, while
preserving the surrounding `for-in` grammar. [Computed member keys and
conditional middle operands](expression-in.md) now handle their own `+In`
contexts; brackets also admit comma expressions with ordered evaluation.
Existing parser work and storage charges cover the added sequence parsing.
Array/object spread remains separate unfinished work.

[Throw diagnostics](throw-completion.md) now preserve the original operand
without running constructor/name getters. This removes an exception-identity
prerequisite for Promise reactions while preserving the existing runtime limits.

[Microtask callbacks](microtasks.md) now provide the first shared job FIFO and
script/host-callback cleanup checkpoints in the existing Window realm.
`queueMicrotask` preserves callback identity, ordinary invocation and ordered
appends while sharing existing work/storage limits. Fatal failures suspend
asynchronous work without turning rendering into a scheduler. Rejection
notifications, MutationObserver delivery, timers
and cross-realm callback settings are subsequent work; the current loader
still executes supported classic scripts after parsing.

[Decimal Number formatting](number-format.md) handles exact shortest-decimal
ties across runtime conversion and compiled property names.
[Number TypedArray foundations](typedarray-foundation.md) provide ten Number
kinds, constructors, live integer-indexed properties and fixed/tracking views
over resizable buffers. [Views and string conversion](typedarray-views-test262.md)
add shared `subarray`, `join` and the exact Array `toString` alias.
The [search follow-up](typedarray-search.md) adds `at`, `includes`, `indexOf`
and `lastIndexOf`; full pinned populations and resource stops remain visible.
[`fill`](typedarray-fill.md) now writes Number view ranges with ordered conversion
and fresh buffer bounds, recovering the two search cases that needed it.
The [fill encoding follow-up](typedarray-fill-encoding.md) removes repeated scalar
preparation with checked writes and unchanged compatibility observations. Its
local benchmark records larger-fill gains and small empty/single-element slowdowns.
[`reverse`](typedarray-reverse.md) now reverses Number views in place with
ordered scalar writes and current resized bounds. Its complete profile retains
four work-limit stops and 18 exclusions alongside 22 passes.
[`toReversed`](typedarray-to-reversed.md) now copies Number views into fresh
fixed storage, including empty and resized sources. Its complete profile records
eight passes, eight work-limit stops and two host exclusions; the resource
outcomes still prevent an upstream baseline. Remaining methods, BigInt views,
shared memory and broader interpreter accounting remain open work.
[`copyWithin`](typedarray-copy-within.md) now handles Number view overlap with
raw bit preservation, ordered index conversion and fresh final bounds. The
complete selection has 20 passes, 46 work-limit stops and 64 exclusions. Its
visited-name allocation optimization restores the unchanged descriptor test
within the original budget; broader interpreter performance remains open.
The [one-pass own-key snapshot](own-keys-one-pass.md) removes a duplicate
creation-order scan without changing any of the 45 compatibility reports.
Later symbols can increase charged scratch storage, and late numeric fallback
still pays for discarded work. The local benchmark has small mixed median
changes and several p95 regressions; broader interpreter performance remains open.

The initial [linear-gradient checkpoint](linear-gradients.md) provided a bounded
software implementation for one legacy-sRGB layer. CSS declarations/supports/style setters,
default background geometry, worker transport and explicit whole-scene native
fallback were included. Broader gradient syntax, attachment controls, color
spaces and GPU gradient interpolation remain open. At that checkpoint, all
68 self-authored references pass after the canvas currentColor correction; the
full Rust suites and strict checks also pass. The [native-gradient follow-up](vulkan-gradients.md)
adds CPU-prepared opaque square-corner fragments outside opacity scopes to the
existing GPU image path; interpolation stays on the CPU. All three frozen
fixtures pass in both output formats on NVIDIA, AMD RADV and llvmpipe, and all
68 browser references pass. Native alpha/rounded/opacity-gradient paths and
timing remain open. The [background geometry follow-up](background-geometry.md)
added size, physical position and repeat/no-repeat axes to that same single layer,
with five-property shorthand resets and final root/inline/table tile resolution.
Its complete suites pass 2,568 Rust 1.88 native-feature tests and 2,370 Rust 1.98
default-feature tests, with all 71 browser references pixel-exact and strict checks passing.
Later [URL backgrounds](css-url-backgrounds.md) and
[box selection](background-boxes.md) added retained-session URL preparation and
origin/clip controls. [Layered backgrounds](background-layers.md) now extend
those paths to at most eight existing image slots, with independent geometry
lists, seven-longhand atomic resets and stable resource bindings. Their shared
budgets and conservative CSSOM getter remain explicit limits. Attachment modes,
`space`/`round`, blending, clip-to-text and broader gradient/color support remain
open. The earlier test totals above belong to their named checkpoints; layered
validation is recorded separately in its chapter.

[Table spacing](table-spacing.md) now uses an inherited computed pair throughout
the existing track, colspan and rowspan equations. HTML's 2px default remains;
zero and unequal axes can be authored through the shared CSS/supports/CSSOM
parser. Final cell geometry still drives background retiling under the same
budgets. The [row-background follow-up](table-row-backgrounds.md) uses each real
originating cell as a paint clip, with row-shared gradient/URL positioning and
no invented paint for missing slots. Shared command, work and key limits still
apply; painted rows can consume more commands. Full intrinsic table sizing,
row-group/column backgrounds, `empty-cells`, `border-collapse`, general
relative/rounded-cell clipping and caption placement remain open. Each
checkpoint's validation belongs to its own chapter.

[Persistent pending inline declarations](cssom-pending.md) now preserve surviving
longhands and their priorities after partial CSSOM edits, across later entries
and worker renders. Typed state shares existing storage/work limits and commits
with serialized attributes. UI snapshots project only attributes; full CSSOM
serialization, computed style, rule APIs and semantic typed transport remain open.
The [getter follow-up](cssom-var-getter.md) prevents unresolved ordinary longhands
from being recombined into a shorthand with different parsing semantics.
The [mixed-wide follow-up](cssom-wide-getter.md) also refuses combinations of
CSS-wide and ordinary components, retaining the same getter work allowance.

[Image fitting and positioning](image-fitting.md) now have a bounded loaded-img
implementation, including Images 4 scale-down pairs and physical one/two/four
component positions. Paint fitting leaves box layout and hit areas intact, with
content-box clipping that also fixes fractional default-fill edges. All 62 self-authored references pass,
with Page/worker and decoded alpha/external-SVG coverage in the passing full
suites. The retained reference-expression correction exposed an image
identity-sampling precision bug. Its separate
[follow-up](image-fitting.md#identity-sampling-follow-up) removes the redundant
divide/multiply on matching image axes and adds three literal browser references.
All 65 browser references now pass, as does a separate replay of the original
SVG reference. Three offscreen fixtures match 710,400 bytes across three Vulkan
adapters. Full CSS Images, inline SVG/video object fitting and broader scaled-image
fidelity remain open. This establishes no acquired-window or performance result.

[Native rectangle accounting](native-rect-accounting.md) reserves the clipped
pre-blend loop area for each Rect. The [image follow-up](native-image-accounting.md)
extends that coordinate pass to Image sampling loops, preserving fractional
clip work and fixed-scope restoration under the same CPU allowance. Line and
opacity reservations, source/mask limits and GPU admission remain unchanged.
The forty-image Page/worker witness now changes from the parent’s CPU-budget
refusal to candidate admission with exact pixels before and after a click.
A separate 1200×900 NVIDIA acquired texture matches all 4,320,000 reference
bytes before the compositor. Broader scene and performance measurements remain
open; the first oversized-surface refusal is retained.

[Native group opacity](vulkan-full-opacity.md) now represents every finite
binary32 opacity value in `0..=1`, including nested transparent sources, without
quantizing values to `k/256`. Cropped RGBA16 scratch, clear/composite dispatches
and retained reuse share the existing limits. The earlier
[opaque](vulkan-opacity.md) and [transparent](vulkan-transparent-opacity.md)
`k/256` checkpoints retain their separate three-adapter evidence.
Larger scene admission remains on the docket.
Native clip/fixed scope accounting charges metadata work without reserving
full-frame pixel work, allowing sparsely painted nested-clipping scenes to fit
the existing limits. That earlier checkpoint retained all primitive and opacity
pixel reserves; the rectangle and image follow-ups refine primitive estimates.

The native owner now reuses one exact-size, fully retired GPU buffer set while
rewriting every input and drawing each complete scene. Limits, queued-write
failure retirement and positive-drop accounting remain unchanged. The
[native benchmark](native-render-benchmark.md#native-baseline-versus-retired-buffer-reuse)
retains an eight-process baseline/reuse comparison with 96 timed frames and two
exact acquired-pixel checks. For its one scene, subsequent median/p95 move from
2.086/4.790 ms to 1.309/4.117 ms; all early submission tails remain included.
Both Rust versions pass 1,397 native-feature tests. The
[reuse evidence](evidence/vulkan-native-buffer-reuse.json) preserves the separate
three-adapter changed-content checks and original CPU/native timing record.
Broader scene measurements, queue staging costs and early submission tails
remain open. GPU clocks were unlocked and changed between environment
snapshots; no Chromium threshold or general speedup is claimed.

[ArrayBuffer](../tests/conformance/array-buffer.md) adds nonshared fixed and
resizable backing storage, resizing, same-realm species slicing, transfers and
detachment. Its complete pinned profile records 262 passes, 50 failures and 130
exclusions. [DataView](../tests/conformance/data-view.md) adds fixed and tracking
views and nine Number codec pairs. Its complete upstream profile records 694
passes, 12 BigInt prerequisite failures and 416 exclusions; the local suite
verifies 128/138 expectations, including four expected resource outcomes.
Further TypedArray methods, BigInt codecs, shared memory, Proxy and foreign realms remain ahead. The [length-bucketed for-in follow-up](../tests/conformance/for-in-length-buckets.md)
closes two metadata-heavy local work stops without raising quotas;
broader interpreter performance and accounting remain open work.

[Object.is](../tests/conformance/object-is.md) passes all 42 upstream modes.
The [DOM method follow-up](../tests/conformance/dom-method-identity.md) separates
eight Document, Element and DocumentFragment function identities, property bags
and receiver checks. It closes four earlier Object.is identity failures and
18 modes in the new DOM suite. [Checked Document.append](document-append.md)
adds the ninth method and closes two further modes; that suite now verifies
54/58 expectations with all 44 controls healthy.
At the eight-method checkpoint, all 44 formal profiles retained their complete
results and all 36 existing gates passed. Its 11 local suites gained 22 passes
without regressions or control changes.
[DOM own properties](dom-own-properties.md) now add per-object string/symbol
storage, descriptors, accessors and method replacement. The unchanged frozen
replacement case reaches the missing Document enumeration operation, so the
suite remained at 54/58 verified expectations at that checkpoint.
[DOM interface prototypes](dom-prototypes.md) add represented HTML, SVG and
MathML inheritance, interface metadata, mutable ParentNode methods and genuine
Text, Comment and DocumentFragment construction. Complete native members,
Document reflection, independent document ownership, host prototype mutation and
custom-element registration remain open work. The unchanged DOM suite now
verifies 56/58 expectations with all 44 controls healthy.

[Processing instructions](processing-instruction.md) add checked construction,
the Document factory, a target accessor and CharacterData data/UTF-16 length.
Five [CharacterData methods](character-data.md) add UTF-16 substring and checked
append, insert, delete and replace operations. [Exact runtime production](dom-production.md)
now preserves units through the named Text/Comment/PI constructors, factories,
data setter and complete splices. It uses the [DOM string foundation](dom-strings.md)
for canonical storage, unchanged ERWA transport and exact reads/clones. All eight
earlier unmet modes now pass, as do all 24 new independent modes.
[Node data accessors](node-data.md) now add ordinary descriptors, exact
`nodeValue`/`textContent` and admitted container replacement while retaining old
detached data. [Exact append strings](append-domstrings.md) now reuse the builder
for separate Text arguments and admit actual arena growth at each creation step;
all fourteen new modes pass. [Document.title](document-title.md) adds ordinary
accessors, exact normalized reads, HTML/SVG selection and retained failure
prefixes. [Text operations](text-operations.md) add UTF-16 `splitText` and adjacent
`wholeText` reads, preserving exact halves, ordinary descriptors and retained
failure stages. [Node normalization](node-normalize.md) now removes empty Texts
and merges ordinary descendant runs with retained detached data and separately
admitted mutation stages. CDATA, live Range updates and mutation notifications
remain outside this increment. [Node predicates](node-predicates.md) add
`hasChildNodes`, `isSameNode` and `contains` through ordinary prototype methods.
[Root lookup](node-root.md) adds `getRootNode`, including observable dictionary
conversion and fresh ordinary ancestry after callbacks. Shadow roots and their
composed ancestry remain absent.
[Structural equality](node-equality.md) adds `isEqualNode` for represented kinds,
comparing exact retained data, namespaced attributes and ordered ordinary children.
The comparison uses the existing work and storage limits.
[Connection state](node-connected.md) now adds `isConnected` through the shared
bounded ordinary-root walk, preserving `getRootNode` options and traversal costs.
Saved getters, readonly assignment and live branch moves are covered.
[Ordinary position](node-position.md) now adds `compareDocumentPosition` over
represented trees, with constant traversal storage, required nonnullable authentic
operands, paid 256-edge bounds and reached sibling scans. Template content remains
separate. The release passes all 32 new modes and 16 controls; default/native
suites and the independent audit pass, with historical inventory failures retained.
[Checked cloning](node-clone.md) now stages complete selected graphs before
publication, with fresh identities and exact stored payloads. It also adds the
`Object.hasOwn` static API used by the browser witness. Full default/native suites
and the new clone and ownership release cases pass. The complete pinned
[Object.hasOwn directory](../tests/conformance/object-has-own.md) now passes all
124 modes with 64 healthy controls and a dedicated CI gate. Full Document cloning, form
dirty state and script cloning hooks remain ahead.
Actual Attr ordering, ShadowRoot, independent document ownership and other Node
members remain open.
Legacy `innerText`, textarea setters and attribute
writes, unit-aware native editing, nonscalar source/HTML serialization,
PI pseudo-attributes, remaining Node/Text operations and mutation notifications
remain open work.

[Fallible runtime initialization](runtime-initialization.md) now carries checked
bootstrap failures through the browser, adapter and stress tool. Partial realms
are not published, and failed initialization does not trigger a second attempt
to draw an error page. Existing budgets and all compatibility observations remain
unchanged. [Batched Node constants](node-constants-bootstrap.md) now save 4,742
charged work units and 69,264 charged bytes during initialization, with ordinary
property order and separate mutable owners preserved. Broader fallible allocation
coverage and bootstrap optimization remain open work.
The [explicit work boundary](runtime-budget-boundary.md) now grants a successful
runtime its fixed author allowance after charged initialization. Raw bootstrap
limits, cumulative heap accounting and existing script/event entry resets remain
unchanged. This prepares for additional interface metadata.

The [native GPU route](vulkan-native-window.md) now connects the custom shaders
to the actual browser window behind `vulkan-raster`. Admitted loaded pages,
overlays and chrome share one bounded plan. Normal shader frames require no CPU
framebuffer or readback; explicit verification matches 4,153,600 acquired bytes
on the NVIDIA host. Separate normal, opacity-fallback and overdraw-fallback
cases pass. Initial size/work refusals remain in the evidence. Rounded geometry
and bundled-font coverage are prepared on the CPU, with GPU color compositing.

The reusable [raster core](../crates/raster-core/README.md) retains its original
Probe limits and now supports phase-local coordinates with frame-wide resource
limits. The [offscreen prerequisite](../tools/vulkan-raster-probe/NATIVE_PREREQUISITES.md),
[glyph](../tools/vulkan-raster-probe/GLYPHS.md) and
[worker-text](../tools/vulkan-raster-probe/WORKER_TEXT.md) checkpoints remain
separate three-adapter evidence. Native page zoom now uses bounded command
scaling within the existing preparation limits. Wider scene/viewport admission,
broader CSS compositing, shaping, native GPU screenshots and driver isolation remain ahead.
Performance within 30% of Chromium has not been demonstrated.

[Array concat](../tests/conformance/array-concat.md) now streams live
spreadability, same-realm species and aliased results. Its complete upstream
profile gains 99 passes with no lost pass; four typed-array failures, eighteen
unsupported modes and two ordinary hole-scan resource stops remain visible.
All 40 older profiles and six selected local suites retain their complete
observations. Resource outcomes prevent a new gate; the 33 established gates
remain unchanged. Proxy, typed arrays, foreign realms, remaining Array methods
and broader accounting remain ahead. That checkpoint left the optional
CPU-frame Vulkan presenter and standalone raster probe unchanged; native-window
GPU rasterization and compositing were still open at that checkpoint.

[Array splice](../tests/conformance/array-splice.md) now streams live property
operations with same-realm species, aliased results and ordered partial effects.
The complete pinned profile has 138 passes and 24 unchanged exclusions; all 96
controls verify. The frozen local matrix verifies 90 of 96 expectations, including
four resource stops. Ten older find/Array.from modes gain passes without other
observation or control changes. Proxy, typed arrays, cross-realm species, other
Array methods and general runtime accounting remain ahead; no quota was raised.

[Shared own-key enumeration accounting](../tests/conformance/own-keys.md) now
uses ranked snapshots and cached for-in tree entries, removing retained-name
deduplication and duplicate searches while keeping live descriptor ordering.
The corrected release preserves all 17,822 historical case observations and
3,900 controls under unchanged limits. Four initial control regressions remain
recorded. General live property-map and JSON accounting remain separate work.

[Array.from](../tests/conformance/array-from.md) now consumes synchronous
iterators and array-like inputs with generic construction, live mapping,
own data definitions and specified iterator closing. Its unchanged local
matrix verifies 388 of 402 expectations, including twelve expected resource
stops; fourteen prerequisite modes remain unmet. The complete upstream profile
now has 86 passes and four metadata exclusions after the ArrayBuffer follow-up,
with all 96 controls verified. Remaining Array methods and broader iterator
consumers remain ahead.

[Synchronous iteration](../tests/conformance/for-of.md) now adds custom
`for…of` protocols and native Array, String and arguments iterators, including
lexical bindings, live properties and iterator closing. The unchanged local
matrix verifies 392 of 404 expectations; twelve prerequisite modes remain
unsupported. Complete new pinned inventories retain 137 for-of passes and
106 core-iterator passes, with two missing-Proxy failures and all exclusions
visible. Destructuring, generators, async iteration and broader consumers remain
ahead. The four Date year-zero modes now pass.

[Date support](../tests/conformance/date.md) now covers the core constructor,
methods, host timezone conversion and bounded discovery. Its complete pinned
Date tree now has 1,166 passing modes and 22 metadata exclusions. Intl/Temporal,
broader legacy parsing and cross-realm behavior remain ahead.
The custom Vulkan rasterizer and compositing milestones below remain on the
docket alongside standards work.
The [Date helper performance follow-up](../tests/conformance/date-readiness.md)
replaces I/O polling sleeps with deadline-bounded readiness waits. Local worker
startup medians improved by about 9 ms in the default build and 8 ms in the
feature build, with unchanged fixture pixels. A race-dependent exit wait remains;
these measurements establish no Chromium or GPU speed comparison. The earlier
[Date startup measurements](benchmark-worker-date.json) remain preserved.

The [Window global-binding checkpoint](../tests/conformance/window-global-bindings.md)
adds general data/accessor definitions, exact UTF-16 property keys, private
execution receivers and global declaration validation. The
[String concat checkpoint](../tests/conformance/string-concat.md) adds bounded,
ordered conversion and exact UTF-16 assembly. The
[constructor checkpoint](../tests/conformance/construction.md) adds Reflect calls,
private `new.target` bindings and alternate ECMAScript allocation targets.
Next work includes alternate Web IDL targets, remaining native constructor
semantics, Function source retention/toString and lossless UTF-16 source parsing,
Window extensibility and interface coverage, exact DOMString production through
remaining legacy innerText/textarea, attribute and parser
paths, remaining Web IDL interfaces,
remaining Symbol and iterator consumers, and broader
ECMAScript dependencies. The
[constructor-policy review](../tests/conformance/constructor-policy.md) now enables
88 older modes and corrects Symbol constructor classification. These sit alongside
the Vulkan milestones below. [Dynamic Function construction](../tests/conformance/function-constructor.md)
now adds 157 passes in a complete new 200-source selection and 35 older passes.
The [String conversion follow-up](../tests/conformance/string-conversion.md)
fixes its two exposed slice failures and adds 18 search/split passes.
[RegExp split/species](../tests/conformance/regexp-split.md) adds 86 passes with
observable construction, execution and captures.
[RegExp constructor conversion](../tests/conformance/regexp-constructor.md)
adds classification and ordered conversion with 16 more passes.
[Flat bounded regexp group parsing](../tests/conformance/regexp-deep-groups.md)
now adds four deep-pattern passes. [Required-literal rejection](../tests/conformance/regexp-required-literals.md)
adds two XML-pattern passes without raising budgets.
[Match/search symbol protocols](../tests/conformance/regexp-match-search.md) now add
138 passes in a complete new inventory.
[String lastIndexOf](../tests/conformance/string-last-index-of.md) adds 46 passes
in its complete directory and 12 older passes.
[Array lastIndexOf](../tests/conformance/array-last-index-of.md) adds 328 passes
and closes the two remaining String.lastIndexOf failures.
[Array descriptors and sparse storage](../tests/conformance/array-descriptors.md)
now support indexed accessors, writable length, partial shrink failure and
nonextensibility. Live array methods consume these properties.
[Array every and some](../tests/conformance/array-predicates.md) now add generic
short-circuiting predicates, with all 164 frozen local variants passing. Their
[complete paired inventory](../tests/conformance/test262-array-predicates.md)
gains 801 passes and four older descriptor metadata modes also pass. The Date follow-up closes all twelve Date prerequisites. Four long-sparse-scan
work stops and 16 resizable-buffer metadata exclusions remain visible; no healthy predicate gate is recorded.
[Object integrity](../tests/conformance/object-integrity.md) now adds shallow
seal/freeze and descriptor-based queries for supported ECMAScript objects,
including sparse arrays, boxed strings and mapped arguments. The
[complete four-directory inventory](../tests/conformance/test262-object-integrity.md)
retains 239 sources and 474 modes, gains 378 passes and verifies all 224 controls.
Eight older descriptor modes also pass, with no previous pass lost. The new
Date, ArrayBuffer and DataView follow-ups leave 38 missing-prerequisite failures and
38 unsupported modes. Host integrity, Proxy/typed-array semantics and untagged
prerequisites remain explicit gaps.
[Array find methods](../tests/conformance/array-find.md) now add ascending and
descending predicate searches with live holes, captured length and saved values.
The complete 94-source / 180-mode profile originally gained 116 passes;
splice adds eight more, reaching 148 passed and 32 metadata exclusions. All 504
local variants and 288 upstream controls pass. Shared property lookup now
precharges retained-tree comparisons and borrowed mapped binding names; no
quota is raised. Actual-digit formatting restores two initial lastIndexOf
resource regressions without weakening those charges. At the find checkpoint,
every older observation was unchanged; its baseline retained the failures and
unsupported modes subsequently measured by the splice follow-up.
Next dependencies include remaining Array methods,
iterators and BigInt, replacement/matchAll protocols, general matching performance,
the remaining full-UTF-16 loop, Unicode regexp syntax and Function source retention.

The goal remains an independent, fully web-compatible Rust browser with a
defensible security boundary and measured performance within the requested
Chromium threshold. The current implementation does not satisfy that goal;
[compatibility](COMPATIBILITY.md), [security](SECURITY.md), and
[validation](VALIDATION.md) track concrete coverage and gaps.

## Vulkan rendering backend

Requested on September 28, 2026. An optional bounded custom Vulkan raster and
compositing path is now implemented; broader coverage remains in progress.

The [Vulkan milestone design](vulkan-rendering.md) records the host device
probe, separates upload/presentation from custom GPU rasterization, and compares
bindings. The standalone [Vulkan transfer probe](../tools/vulkan-probe/README.md)
pins wgpu 30.0.1 in its own crate and lockfile. Its bounded offscreen uploads and
readbacks preserve the tested bytes on the NVIDIA, AMD and software Vulkan
adapters. A separate temporary native prototype presented changed/resized
frames on the NVIDIA adapter; five acquired-surface readbacks matched all
3,316,800 tested bytes. Full compositor captures remained non-exact, with
counts and caveats preserved in the
[native evidence](evidence/vulkan-native-surface.json).

The optional [Linux Vulkan presenter](vulkan-rendering.md) now uploads completed
CPU-painted frames through a Vulkan-only wgpu binding. It uses one owner thread,
one active upload and one replaceable pending frame, fixed operation deadlines,
and confirmed resource release before software fallback. Verification reads
back actual acquired surface textures; it does not certify compositor output.
Software remains the default/headless path. The optional bounded
[native raster route](vulkan-native-window.md) now uses this owner; broader
compositing, driver isolation and performance work remain open.
Native rounded shapes can use two adjacent masks without raising shared limits.
Two gated 1280×880 window cases pass, including 4,505,600 acquired bytes matching
the CPU reference. Release CPU/native and native baseline/reuse comparisons
now use equal scenes with preparation and presentation costs kept separate.
Further measurements need more scenes and repeated environment observations.

[Large rounded Color backgrounds](vulkan-rounded-core.md) now use a proven
interior rectangle and at most seven edge masks when an individual mask would
exceed the existing allowance. Ordinary small-mask routes and resource limits
remain unchanged. Wider geometry, acquired-window coverage and performance
measurements remain open.

Acceptance work:

- Expand the bounded native route to more browser scenes, preserving clipping,
  fixed coordinates, text coverage, full-range nested opacity and alpha under
  explicit resource limits.
- Bound GPU allocations, command work, uploads and retained resources. Decide
  the GPU process/driver boundary explicitly; GPU access must not broaden the
  renderer's filesystem or network authority.
- Handle synchronization, resizing, surface loss, device loss and resource
  cleanup. Keep the software renderer available for comparison and fallback.
- Compare supported output against independent pixel references and the
  software renderer. Record justified rasterization tolerances explicitly.
- Measure equivalent workloads, including upload/synchronization/presentation
  costs, memory and frame latency. Vulkan alone does not establish a Chromium
  performance result.

The standalone [custom raster probe](../tools/vulkan-raster-probe/README.md)
executes Eris WGSL shaders through Vulkan, with **30 exact offscreen fixtures on
each of three host adapters (2,764,860 compared bytes)**. It rasterizes ordered
unrounded rectangles and nearest-neighbor images with integer source-over onto
opaque RGB, including clip/fixed scopes and per-draw rounding. Nine independently
frozen alpha fixtures join the unchanged 21 rectangle/image cases. Source,
metadata and target buffers retain the 1 MiB explicit GPU limit, with unchanged
work, command and scope caps. The [alpha evidence](../tools/vulkan-raster-probe/evidence/host-alpha.json)
records its 25 Rust tests per toolchain, ten Python tests and actual GPU readback.
Those historical fixture and evidence bytes remain unchanged.

The optional [browser bridge](../tools/vulkan-raster-probe/BROWSER_BRIDGE.md)
now borrows snapshots/display lists and image stores. Its
[host checkpoint](../tools/vulkan-raster-probe/evidence/host-browser-bridge.json)
passes 16 cases per NVIDIA, AMD and software Vulkan adapter, including five real
confined-page captures. Nine cases use GPU drawing and seven use complete CPU
fallback: 151 GPU pixels plus 46 fallback pixels, 788 packed bytes per adapter
and 2,364 combined bytes. Hidden unsupported commands refuse the whole frame;
image aliases, missing images and typed scopes retain their original semantics.
All 30 prior standalone cases still pass on all three adapters.

Both Rust 1.88 and 1.98 pass 50 feature-enabled tests; the unchanged default
25-test checks are reused from the first candidate. Eleven new process-supervisor
and fourteen protocol tests pass. The 320×240, 256-command, 32-scope, 1 MiB GPU
buffer and four-million-invocation caps remain. The native window's 400×250
minimum and text chrome exceed this subset; its upload presenter continues to
use CPU-painted frames. A subsequent [glyph checkpoint](../tools/vulkan-raster-probe/GLYPHS.md)
passes 12 independent mask cases and 14 parent-CPU font comparisons on all three
adapters, using a separate fonts-aware entry point and unchanged GPU caps.
Seven [worker-text snapshots](../tools/vulkan-raster-probe/WORKER_TEXT.md) now
also pass on those adapters: six GPU frames and one complete CPU fallback.
Those probe limits are unchanged. The later [native route](vulkan-native-window.md)
adds separately bounded scene construction and surface conversion. Shaping, group
opacity, full compatibility, production security and Chromium comparisons remain open.

## JavaScript execution depth

The original Test262 source with 32 nested immediately invoked functions now
passes in both modes. Its source and the complete function inventory remain
unchanged. The [parser checkpoint](../tests/conformance/parser-continuations.md)
recorded 511 passes; the later [constructor policy](../tests/conformance/constructor-policy.md)
brought coverage to 515 passes; dynamic Function construction now brings it to
546 passes and 585 unsupported variants. A healthy baseline
runs in CI.

The implemented progression is:

1. Syntax ownership is flat from direct parsing through execution. Charged record
   pages and typed IDs cover partial syntax, defaults, methods and function bodies.
   Closures and callbacks retain their unit. The old owning tree/lowering path is
   test-only. Declaration walks and name validation use bounded charged storage.
2. Expressions, statements, ordinary activation, defaults and bound forwarding
   share an execution driver. Reentry and cleanup preserve frame boundaries and
   logical-call ownership. Native callbacks and constructors retain weighted guards.
3. Supported JavaScript grammar uses a separate bounded continuation stack over
   flat records. Function context, strict checks, cover grammar and RegExp/template
   lexical goals are preserved. The former grammar-depth and member/constructor
   chain guards are removed; source/token/work/storage quotas remain unchanged.

The [480 depth observations](../tests/conformance/parser-continuations-depth.json)
now show all three shapes parsing through the tested depth of 40 and executing
through 32 calls; call 33 reaches the existing logical ceiling. Relative to direct
flat parsing, nested IIFEs gain 104 completions and change 16 resource observations
from parsing to the logical runtime limit. No completion is lost. Separate source
and malformed-cleanup tests request a 128 KiB native stack.

Remaining depth work includes the independent 96-ancestor declaration traversal
limit, native callback/constructor bridges and wider allocation accounting.
Keep source inventories, strict/early-error checks, work/heap refusal tests and
small-stack cleanup checks as these boundaries evolve. No stage establishes full
language coverage, production security or the requested Chromium performance.
