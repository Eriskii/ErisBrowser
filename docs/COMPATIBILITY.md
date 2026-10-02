# Compatibility status

[Object.is](../tests/conformance/object-is.md) implements SameValue for existing
runtime values, including NaN, signed zero, exact UTF-16 strings and ordinary
object/symbol identity. All **42 upstream modes pass**, with 80 healthy controls.
The [DOM method follow-up](../tests/conformance/dom-method-identity.md) closes
four Document/Element query-identity failures, bringing the unchanged local
suite to **72/78 verified expectations**, with 60 healthy controls. Proxy,
BigInt and foreign-realm cases remain unmet. Existing query/append
methods now have separate defining-interface identities, function properties
and receiver checks. [Document.append](document-append.md) adds the ninth method
and checked insertion, bringing the unchanged local suite to **54/58 verified
expectations**, with 44 healthy controls. [DOM own properties](dom-own-properties.md)
now support individual string/symbol data, accessors and method replacement.
The frozen replacement case progresses to the missing complete Document
enumeration operation in both modes; its ordinary-success expectation remains
unmet. [DOM interface prototypes](dom-prototypes.md) now provide represented
HTML, SVG and MathML chains, interface metadata, mutable ParentNode operations
and Text, Comment and DocumentFragment construction.
[Processing instructions](processing-instruction.md) add genuine construction,
the Document factory, target and CharacterData data/length accessors for scalar
strings. Five [CharacterData methods](character-data.md) now add exact UTF-16
substring results and checked splicing, including repaired surrogate boundaries.
PI pseudo-attributes, lone-surrogate DOM data and mutation notifications
remain unfinished. Complete interface members,
Document reflection and host prototype mutation remain incomplete. The unchanged
DOM suite now verifies **56/58 expectations**, with all 44 controls healthy; the
two complete-Document-enumeration expectations remain unmet. The earlier eight-method
checkpoint's complete 44-profile replay preserved all prior formal case/control
observations and all 36 baseline gates.

[ArrayBuffer](../tests/conformance/array-buffer.md) supports nonshared fixed and
resizable buffers, byteLength/maxByteLength/resizable/detached getters, resize,
same-realm species-aware slice, transfer and transferToFixedLength. Ordinary
properties and object freezing do not alter private backing storage. `isView`
recognizes authentic DataView instances, including detached or out-of-bounds
views. Typed arrays, shared buffers, Proxy and foreign realms remain incomplete.
The complete 221-source profile records **262 passed, 50 failed and 130 excluded
modes**, with all 160 controls verified. The failures retain 48 missing-Uint8Array
observations and two untagged BigInt syntax failures. The independent local
matrix verifies **96 of 104 expectations** after the
[for-in length-bucket follow-up](../tests/conformance/for-in-length-buckets.md)
closes two metadata work stops and DataView closes two prerequisite modes.
Eight prerequisite modes remain unmet.

[DataView](../tests/conformance/data-view.md) supports fixed and tracking views,
three getters and nine Number getter/setter pairs, including Float16. Conversions
run before fresh bounds checks; ordinary properties and freezing do not freeze
backing bytes. The complete 561-source profile records **694 passed, 12 failed
and 416 unsupported modes**, with all 232 controls verified. All twelve failures
require missing BigInt support. The local suite verifies **128/138 expectations**,
including four resource cases, with all 92 controls verified. BigInt codecs,
shared and immutable buffers, typed arrays, Proxy and foreign realms remain gaps.

[Array concat](../tests/conformance/array-concat.md) supports generic receivers,
live spreadability and same-realm species results, including ordinary objects,
Arrays, mapped arguments and input aliases. Species construction precedes
spreadability reads; holes advance the result index without deleting existing
keys, and the strict final length write can fail after earlier effects. The
complete 69-source profile records **113 passed, four failed, 18 unsupported
and two resource modes**, with all 96 controls verified. The independent local
matrix verifies **84 of 90 expectations**, including four expected resource
stops. Proxy, typed-array and cross-realm prerequisites remain unmet. Two ordinary
upstream hole scans reach the work limit, so no new baseline or CI gate is added.
Other Array methods' species behavior is unchanged.

[Array splice](../tests/conformance/array-splice.md) supports generic ordinary
receivers, captured safe-integer lengths, live inherited properties and same-realm
species construction. Deleted results preserve holes; source moves and deletes
follow the specified direction, and errors retain earlier effects. Species may
return the source, another Array or a supported ordinary object. The complete
81-source profile records 138 passes and 24 metadata exclusions across 162 modes,
with all 96 controls verified. The local matrix verifies 90 of 96 expectations,
including four expected resource stops; Proxy, typed-array and cross-realm cases
remain unmet. This does not extend custom species handling in map/filter/slice.

[Shared enumeration accounting](../tests/conformance/own-keys.md) preserves all
17,822 historical case observations and 3,900 controls with unchanged quotas and
baselines. String-key snapshots and for-in visited-name work are now charged;
this adds no new conformance passes or unsupported-feature coverage.

[Array.from](../tests/conformance/array-from.md) supports synchronous iterable
and array-like inputs, generic constructors, live mapping, own indexed data
properties and iterator closing. Its complete 47-source upstream profile has
86 passing modes and four generator/cross-realm metadata exclusions after the
ArrayBuffer follow-up; all 96 controls verify. The independent
local matrix verifies 388 of 402 expectations, including twelve terminal-resource
cases. Both modes of seven unsupported prerequisites remain unmet: generators,
classes, Set, Map, typed arrays, Proxy and BigInt. These are retained failures or
unsupported observations, not a claim of complete ECMAScript support.

[Synchronous iteration](../tests/conformance/for-of.md) covers identifier/member
`for…of` heads, lexical bindings and custom, Array, String and arguments iterators.
Destructuring, generators and async iteration remain incomplete.

[Date](../tests/conformance/date.md) now has internal time values, ordered
construction and setter conversions, UTC/local fields, ISO parsing/formatting,
required string round trips, JSON and primitive-conversion hooks, and the
legacy year/GMT aliases. Local operations use an explicit host timezone
snapshot with historical transitions, recurring rules and gap/overlap handling.
The complete core Date profile retains 594 sources and 1,188 modes: 1,166 pass,
and 22 retain their metadata exclusions. Synchronous iteration closed the four
previous year-zero parser gaps.
All 340 controls and 406 independently frozen local modes pass. Broader legacy
date parsing, Intl locale formatting, Temporal and cross-realm behavior remain
incomplete. Locale methods currently use the documented
English local-date forms permitted by the core specification without Intl.

Script-enabled browser/CLI/adapter startup captures host rules explicitly.
Missing or unsupported rules produce an initialization error; scripts-disabled
pages remain usable. On systems whose named-zone database is outside
`/usr/share/zoneinfo`, set `TZDIR` to its real location. Library embedders provide
a validated Date host context for local operations; UTC operations do not require
local-zone configuration.

[Array find methods](../tests/conformance/array-find.md) now support `find`,
`findIndex`, `findLast` and `findLastIndex` on ordinary generic receivers.
Their captured range includes holes and deleted indices, with live inherited
property lookup in the required direction. A matching callback returns its
previously read value or index; callback coercion, receiver and abrupt effects
use the existing call path. All 504 frozen local variants and twelve controls
pass.

The [complete profile](../tests/conformance/test262-array-find.md) retains
94 sources and 180 modes. It now records **148 passed and 32 metadata
unsupported**, with all 288 controls verified. The original find checkpoint
added 116 passes; splice closes its eight callback-body prerequisite failures.
Proxy, typed/resizable arrays, general host receivers and complete
`with`/unscopables remain gaps. The regression baseline retains every exclusion.
Two initial lastIndexOf
work-limit regressions were corrected by charging actual decimal-key digits;
stronger property-lookup charges and all quotas remain in force.

[Object integrity](../tests/conformance/object-integrity.md) now supports
`Object.seal`, `Object.freeze`, `Object.isSealed` and `Object.isFrozen` on
supported ECMAScript objects. Arrays retain holes and length flags; accessors
are preserved without getter calls; sealing mapped arguments retains aliases,
while freezing snapshots and detaches them. Operations are shallow and preserve
earlier effects if a later property or resource operation fails. Window/DOM
hosts, Proxy and typed-array behavior remain unsupported.

The [complete new profile](../tests/conformance/test262-object-integrity.md)
retains 239 sources and 474 modes. After the DataView follow-up it records
**398 passed, 38 missing-prerequisite failures and 38 unsupported**, with all 224 controls
verified. All 228 local variants and 12 controls pass. Eight older descriptor
modes gain passes, with no previous pass lost. The new profile has a healthy
regression gate; its 38 failures and 38 unsupported observations remain part of
the baseline. This does not establish complete object or host conformance.

[Array every and some](../tests/conformance/array-predicates.md) now support
generic ordinary receivers, captured length, holes, inherited/indexed accessors
and live callback mutations. They short-circuit using callback truthiness and
preserve callback receiver and abrupt-completion behavior. All 164 frozen local
variants pass. The [complete paired upstream inventory](../tests/conformance/test262-array-predicates.md)
records **847 passed, four work-limit stops and 16
metadata exclusions** across 867 modes. Huge sparse scans remain budgeted;
Proxy, typed arrays and general host receivers remain unsupported. Resource
stops prevent a healthy baseline for this profile.

[Array.lastIndexOf](../tests/conformance/array-last-index-of.md) now supports
ordinary generic receivers, holes, inherited entries, saved length and live
getters. Its complete 395-mode inventory records 385 passed,
two array-setup resource stops and eight unsupported modes. Array descriptor and
sparse-storage gaps remain; this is an observation inventory, not a healthy gate.

[String.lastIndexOf](../tests/conformance/string-last-index-of.md) now handles
ordered conversion, NaN/default positions and reverse UTF-16 search. Its complete
50-mode selection now passes all modes after the Array follow-up. The earlier
String implementation also added 12 match/search passes.

[String and RegExp match/search protocols](../tests/conformance/regexp-match-search.md)
now support custom symbol hooks, raw receivers, observable fallback dispatch,
global match collection and exact search index restoration. The complete new
340-mode selection now records 316 passed, four failed and 20 unsupported.
BigInt, Unicode pattern syntax and replacement/matchAll protocols
remain incomplete.

[RegExp constructor classification and conversion](../tests/conformance/regexp-constructor.md)
now handle Symbol.match-based identity, regexp-like source/flags and observable
allocation/conversion order. The [flat group parser](../tests/conformance/regexp-deep-groups.md)
adds four passes for deeply nested patterns, and [required-literal rejection](../tests/conformance/regexp-required-literals.md)
adds both XML-pattern modes. The complete 976-mode directory now
records 774 passed, 200 unsupported and two resource stops. Unicode regexp
syntax, broader legacy grammar, matching performance, realms and remaining symbol
protocols need work.

[RegExp split and species](../tests/conformance/regexp-split.md) now use a separately
constructed splitter and observable execution, index and capture hooks. The new
96-mode selection records 90 passed and six unsupported
modes. Unicode regexp parsing, realms and other symbol methods remain incomplete;
the constructor follow-up above addresses classification and conversion order.

[String conversion, search and custom splitting](../tests/conformance/string-conversion.md)
now respect ordinary receiver/needle conversion, `Symbol.match` classification
and object separators' `Symbol.split` hooks. The new 484-mode selection records
470 passed, two failed and 12 unsupported modes; the two older slice failures
also pass. The built-in split protocol is covered by the follow-up above;
other String symbol protocols remain incomplete.

[Dynamic ordinary Function construction](../tests/conformance/function-constructor.md)
now uses global scope, separate parameter/body grammar and the caller's remaining
resource budgets. Its new 200-source selection records 241 passed and 50 unsupported
modes; older selections gain 35 passes. Its two exposed string-slice failures
are fixed by the conversion follow-up above. Function source printing, literal unpaired
surrogates in generated source and the wider Function family remain incomplete.

The [Symbol constructor/policy follow-up](../tests/conformance/constructor-policy.md)
accepts Symbol as an alternate constructor target while preserving its throwing
construction behavior. The Symbol profile now records 184 passed, four failed and
54 unsupported modes. Across the existing inventories, 88 additional modes are
admitted and pass; two required this implementation fix, while 86 already passed.

[Constructor targets and Reflect calls](../tests/conformance/construction.md) cover
`new.target`, `Reflect.apply`, `Reflect.construct`, lexical arrows, bound forwarding
and supported ECMAScript allocation prototypes. The two complete pinned selections
now record 58 passed and eight unsupported modes.
classes/super, tagged templates, alternate Web IDL targets and broader native
constructor conformance remain incomplete.

[DOM string conversion and receivers](../tests/conformance/dom-string-conversion.md)
cover the supported creation, query, attribute, tree and class-list operations.
String hooks run in argument order before those operations; borrowed methods use
their actual receiver. Nullable text and Boolean properties retain distinct
conversion rules. Complete DOMString storage, remaining interface members, XML name
validation, live collections and DOM hierarchy semantics remain incomplete.

[Symbol primitives and property keys](../tests/conformance/symbols.md) now include
identity, registry/description behavior, descriptors, computed keys, JSON omission,
`Symbol.toPrimitive`, `Symbol.toStringTag`, `Symbol.hasInstance`,
`Object.getOwnPropertySymbols` and `Reflect.ownKeys`. The complete new profile
originally recorded 168 passed, six failed and 68 unsupported modes. Iteration, RegExp dispatch,
species, other realms and most Reflect methods remain incomplete.

[String.prototype.concat](../tests/conformance/string-concat.md) supports generic
receivers, ordered conversion, shared method identity and exact UTF-16 output.
Its complete pinned directory has 42 passing modes and two metadata exclusions
for Reflect.construct. This does not complete the remaining String methods.

This is an implementation inventory, not a conformance certificate. A feature listed as partial supports specific cases; it does not imply passing that specification's full tests. Unsupported syntax is generally ignored by HTML/CSS or reported by the script interpreter.

| Area | Implemented subset | Missing or incomplete |
|---|---|---|
| HTML | Tokenization, retained comments/doctypes/processing instructions, implied html/head/body, common optional end tags, script escaped states, scripting-aware noscript, table insertion modes and foster parenting, active formatting reconstruction and bounded adoption-agency recovery, named/numeric character references, SVG/MathML foreign-content namespaces and integration points, context-sensitive HTML/SVG/MathML/template fragments, template insertion modes, retained document modes, HTML byte encoding selection/reparse, select/frameset/ruby recovery, parser selectedcontent cloning, basic forms | Complete parsing and custom select behavior, incremental document.write parsing, declarative shadow roots/content patching, complete quirks rendering |
| DOM | Arena tree with character/doctype metadata nodes, textContent, attributes, creation/appending/removal, selector queries, bounded serialization, HTML/SVG/MathML namespace identity and adjusted attributes, context-sensitive innerHTML subset, hosted template DocumentFragments, fragment transfer, bounded cloneNode, classList, document URL and frozen base metadata | Full interfaces and arbitrary XML namespaces, mutation observers, ranges, selections, shadow DOM, custom elements, DOM exceptions, complete live collections |
| CSS selectors | Shared token-aware parsing, comment/whitespace boundaries, type, universal, ID, class, attributes, descendant/child/sibling combinators, selected structural/logical pseudo-classes | Full escaping/namespaces, interaction-state pseudo-classes, generated pseudo-elements, complete Selectors conformance |
| CSS cascade | Specificity, source order, important declarations, inline styles, inherited properties, custom properties, selected media conditions, bounded stylesheet imports, named/anonymous nested cascade layers, normal/important ordering and revert-layer, bounded supports conditions and conditional imports | Scopes, complete invalid-value semantics and feature queries, container queries, font-face |
| CSSOM | Bounded inline style value/priority queries and mutation, cssText, length/item/index reads, supported camel/dashed property aliases and cssFloat, case-sensitive custom names | Stylesheet/rule interfaces, computed style, complete value and shorthand serialization, pending-shorthand partial removal/priority replacement, complete prototypes/reflection and expandos |
| CSS calculations | Bounded additive length-percentage `calc()`, nested sums/differences, retained percentage dependency, actual sizing bases and used-value clamping | Products/division, numeric calculations, min/max/clamp and other math functions, complete percentage-radius storage, full intrinsic sizing and required nesting range |
| Media queries | Screen/all/print types, only/not, comma alternatives, grouped and/or/not conditions, width/height lengths and range comparisons, escaped identifiers, orientation and selected fixed environment values; unknown features retain unknown truth through negation | Math functions, aspect ratio/resolution, complete feature inventory and actual system/device preference integration |
| Layout | Block/inline flow, text wrapping, margins/padding/borders, flex rows/columns/reverse/order, scaled shrink and min/max freezing, align-self/auto margins, row/column wrapping and cross-line align-content, Grid lines/spans and dense row/column placement, explicit/implicit tracks, minmax/fr sizing and alignment, tables/spans, left/right floats, clear, shrink-to-fit, partial formatting contexts, relative/absolute/fixed positioning, padding-box containing blocks and bounded stacking contexts | Complete formatting contexts, intrinsic flex auto minimums, baseline alignment, writing-mode and indefinite-size interactions, full positioned flex behavior, complete Grid intrinsic sizing/named areas/subgrid, float margin-collapse/painting interactions, full inline/static-position geometry, sticky positioning, multicolumn/pagination, transforms, writing modes and bidi |
| Typography | Bundled sans/bold/monospace fonts, basic advances/kerning, synthetic italics, alignment/decoration | Complex text shaping, font fallback/downloads, full Unicode line breaking, hyphenation, variable fonts, complete font selection |
| Paint | CPU display lists, nested rectangular overflow clips, rounded backgrounds/borders, alpha blending, cached glyphs, raster images, separate viewport-fixed paint/hit coordinates, group opacity with deferred bounded premultiplied RGBA16 surfaces, scrolling and zoom | Full compositor, rounded clipping masks/separate overflow axes, filters, masks, shadows, gradients, animations, hardware acceleration, color management |
| Images | PNG, JPEG, first-frame GIF/WebP, BMP via image codecs; bounded SVG subset | Animated image playback, AVIF, complete SVG, image orientation/media fidelity and full responsive image selection |
| SVG | Basic shapes, paths, fill/stroke, viewBox, transforms, simple text | Full SVG/CSS integration, filters, patterns, gradients, masks, markers, foreignObject, animation, external resource references |
| JavaScript | Primitive values, UTF-16 strings, untagged interpolated templates, computed object keys/methods/accessors and static prototype setters, arrays/objects, lexical initialization and loop scopes, operators including comma expressions, strict directives and reference/receiver rules, control flow including switch and do/while, closures/functions/arrows with identifier default/rest parameters, bounded dynamic ordinary Function construction, constructors/this/instanceof and basic prototypes, ordinary data/accessor descriptors, for-in enumeration, deletion and bound functions, mapped/unmapped arguments, throw/try/catch/finally, selected builtins, DOM bindings and events | Full ECMAScript, classes/modules, destructured parameters, dynamic eval, complete Function source/UTF-16 handling, complete Annex B behavior, complete prototypes/exotic descriptors/coercion, async/promises, complete Error constructors/prototypes/stacks, tagged templates, complete String methods/generic receivers/property behavior, fractional/large nondecimal Number.toString, JIT and GC |
| RegExp | Custom UTF-16 non-Unicode literals/constructor, d/g/i/m/s/y flags, captures/named groups, alternatives/classes/quantifiers, anchors/boundaries, lookahead/backreferences, exec/test/lastIndex, String match/search/replace/split | Unicode u/v modes and properties/sets, lookbehind, modifier groups, duplicate/non-ASCII group names, several legacy escape forms, Symbol hooks/species and complete RegExp compatibility |
| JSON | Strict JSON grammar, ordered object properties, numeric formatting, UTF-16 strings including lone-surrogate round trips, toJSON, primitive unboxing, replacer callbacks/allowlists, code-unit indentation, cycle errors, postorder reviver with primitive source context | DOM/window serialization, complete source-context semantics, BigInt-related behavior and rawJSON APIs remain unsupported |
| Events | Event/CustomEvent/EventTarget constructors, dispatchEvent, capture/target/bubble phases, fixed paths, listener identity/mutation, once/passive/handleEvent, cancellation and propagation, inline handlers, trusted native and synthetic author events, AbortController/AbortSignal reasons and listener removal, readiness/click/input hooks, ToggleEvent, exact reentrant disclosure tracking and bounded native idle continuation | Shadow DOM retargeting, AbortSignal.any/timeout and broader cancellation, specialized UI/pointer/touch/keyboard events, complete activation/default actions, asynchronous tasks/microtasks, full Window/body handler forwarding and event lifecycle |
| Disclosures | HTML details/summary visibility, first-summary and generated-summary activation, exclusive name groups, open/name reflection and coalesced toggle notifications | Full event-loop timing, anonymous content-slot formatting, fragment reveal, keyboard event ordering, accessibility and complete marker styling |
| Forms | Scalar-safe text cursor/selection editing, checkbox/radio toggling, bounded GET and HTTP(S) POST URL-encoded serialization, selected options, submitter method/action/encoding overrides, disabled-fieldset handling | Multipart/text-plain POST and file upload, constraints, complete reset/default behavior, native selection controls, autofill, password management |
| Network | HTTP(S) GET and URL-encoded document POST, verified TLS, gzip, relative URLs, checked redirects (301/302/303 convert POST to GET; 307/308 preserve it), bounded bodies, charset labels, separate native resource broker, authoritative cross-origin document redirects, isolated cross-origin image decoding with redirect taint | Browser HTTP cache, cookies, authentication, full Fetch/CORS/CSP, HSTS persistence, HTTP/2/3, service workers, WebSocket/WebTransport |
| Browser | Single window/document, address entry, history, scrolling, basic keyboard focus, zoom, native text clipboard shortcuts, per-document confined Linux renderers and resource brokers, headless PNG output | Tabs, downloads, permissions UI, devtools, document find/selection, grapheme text editing, complete IME and accessibility |
| Other platform APIs | No broad implementation | Canvas APIs, WebGL/WebGPU, WebAssembly, audio/video, WebRTC, storage/IndexedDB, workers, geolocation, sensors, notifications, payments, credentials, extensions, printing and many others |

The URL parser and encoding libraries are infrastructure libraries, not independent reimplementations of those specifications. There is no claim that every dependency is authored in this repository.

Ordinary objects support descriptor defaults, writable/enumerable/configurable checks, getters/setters, reflective descriptor queries and deletion. Enumeration visits own keys and inherited enumerable keys while respecting shadowing. Window descriptor queries expose tracked global bindings, including the CSS namespace, with ordered property-key conversion and current flags/values. The replaceable [Window.self accessor](../tests/conformance/window-self.md) supports assignment, deletion, descriptor restoration and author accessors, with global lexical shadowing kept separate. The [global value properties](../tests/conformance/global-values.md) share this handling: globalThis is initially writable/configurable and nonenumerable; undefined, NaN and Infinity are nonwritable, nonconfigurable and nonenumerable. Replacing globalThis preserves private Window/event identity. [Window binding reflection](../tests/conformance/window-reflection.md) now supports own membership, enumerability and ordered string/symbol key enumeration over the supported bindings. [General Window definitions and global bindings](../tests/conformance/window-global-bindings.md) now share data/accessor records, preserve arbitrary UTF-16 key identity, keep execution receivers private, and validate global function declarations across scripts. Event-handler descriptor/deletion behavior, extensibility, complete interfaces and named-property behavior remain incomplete. Arrays support indexed data/accessor descriptors, writable length, sparse u32 logical lengths and nonextensibility. Length reduction deletes own indices in descending order and preserves partial effects when a nonconfigurable index blocks it. Object and Reflect property definitions share the same array rules. Eleven existing Array methods now use live property operations on supported ordinary array-like receivers. Splice supports same-realm custom species; map/filter/slice custom species, Proxy, typed arrays and general host reflection remain incomplete. See [array descriptor coverage](../tests/conformance/array-descriptors.md). The unchanged upstream property-helper harness runs directly; no native test assertion shims replace it.

The complete pinned [global-value inventory](../tests/conformance/test262-global-values.md)
retains 88 variants: 70 pass and 18 require
unsupported eval. All 64 preflights verify, but
the regression gate does not imply complete global-object support.

`Object.prototype.isPrototypeOf` checks internal prototype links and object
identity, with the primitive-argument test preceding receiver conversion. It
observes current prototype mutations without reading author `__proto__` or
constructor properties. Traversal shares the existing work and depth limits;
Proxy traps and complete host prototype behavior remain unsupported. See
[prototype membership](../tests/conformance/is-prototype-of.md) and its
[pinned upstream inventory](../tests/conformance/test262-is-prototype-of.md).

`Array.prototype.sort` implements stable sorting for arrays and supported
ordinary array-like receivers. It distinguishes absent entries from undefined,
includes inherited values, compares default strings as UTF-16 code units, and
uses live author comparators and conversion hooks. Collection and comparison
precede ordered strict writes/deletes; exceptions preserve earlier author and
write-back effects. The [bounded sort implementation](../tests/conformance/array-sort.md)
uses the shared array descriptor rules; Proxy, typed-array and
general host receiver support remain incomplete. Its work/heap limits can stop large inputs.
The [complete pinned sort directory](../tests/conformance/test262-array-sort.md)
now records 91 passed, 12 unsupported and four resource stops across 107 modes.
Array descriptor support enables 32 additional modes.
Adding reduce enables the 5- and 11-element stability tests in both modes;
the unchanged 513- and 2,048-element tests reach the shared work/allocation
limits. No healthy sort baseline is recorded.

`Array.prototype.reduce` and `reduceRight` stream over supported ordinary
array-like receivers. They capture length once, distinguish omitted initial
values from explicit undefined, and read sparse/inherited entries live in the
requested direction. Four callback arguments, actual-callee this behavior and
prior author effects survive through page and event execution. Full safe-integer
logical lengths share existing work/allocation limits without allocating from
length. Actual Array indexed/length descriptors use the same live property paths.
Host receivers, Proxy and typed arrays remain unsupported. See the
[implementation scope](../tests/conformance/array-reduce.md) and
[complete paired upstream inventory](../tests/conformance/test262-array-reduce.md).

Strictness follows exact unescaped directive prologues and lexical function inheritance. Supported code checks restricted names, duplicate simple parameters, legacy literals and identifier deletion; strict calls preserve the supplied `this`, and failed writes/deletes throw. Strict arguments and arguments of functions with defaults or rest parameters are unmapped and have a throwing `callee` accessor; supported sloppy functions with simple parameter lists retain aliases until deletion or descriptor changes detach them. Lexical bindings include initialization checks, declaration conflicts and per-iteration loop environments. This does not implement the complete ECMAScript grammar or all Annex B behaviors. Unsupported dynamic eval, tagged templates and destructured parameters are reported explicitly.

Identifier default parameters run left to right for omitted or undefined values,
with all parameter bindings created before initialization. Initializer closures
retain their parameter scope separately from body declarations; function length,
anonymous names, receiver and arguments behavior have focused checks. Ordinary
functions, arrows, methods and setters share this bounded implementation. See
[default-parameter scope](../tests/conformance/default-parameters.md) and the
[complete upstream function inventory](../tests/conformance/test262-functions.md).

JavaScript identifier names use pinned Unicode 18.0.0 `ID_Start`/`ID_Continue`
data with ECMAScript additions. Raw names and valid four-digit or braced Unicode
escapes resolve to the same binding without normalization. Decoded reserved
words remain invalid as bindings/references; property names accept them, while
keywords and accessor introducers require literal spelling. Numeric adjacency,
RegExp flag boundaries and ECMAScript whitespace use their separate lexical
rules. These changes do not implement classes, modules, private fields,
generators, async execution or Unicode RegExp capture names. See
[identifier scope](../tests/conformance/identifiers.md) and
[data provenance](../tests/conformance/unicode-identifiers-data.md).

Identifier rest parameters create fresh dense arrays from the remaining actual
arguments. Ordinary functions, arrows and concise methods support them; accessors
reject rest. Rest-only functions use unmapped arguments without introducing the
separate body scope needed by defaults. See [rest-parameter scope](../tests/conformance/rest-parameters.md)
and the [complete pinned rest directory](../tests/conformance/test262-rest-parameters.md).

Script exception handling preserves explicitly thrown values and exposes ordinary runtime failures as objects with `name` and `message`. Catch bindings have local scope, and `finally` preserves or overrides returns and loop control. Execution, allocation, nesting, and other resource limits terminate the current script entry without running `catch` or `finally`; these host limits cannot be overridden by page code.

The six [compound bitwise assignments](../tests/conformance/compound-assignment.md)
reuse a single evaluated reference, with ordered getter/RHS/conversion/setter
behavior, strict writes and masked 32-bit shifts. Their complete inventory has
606 passes and 180 unsupported modes.
BigInt behavior remains incomplete; Symbol numeric conversion now throws TypeError.

The three [logical assignments](../tests/conformance/logical-assignment.md)
read their reference once and skip both RHS and write when their condition
does not select assignment. Taken identifier assignments infer names for
anonymous function/arrow expressions; member targets retain unnamed functions.
Strict/readonly/const rules apply only to actual writes. The complete profile
retains 78 passes, six class failures and 48 unsupported modes, with
104 verified controls.

[Declaration traversal](../tests/conformance/scope-walk.md) uses bounded borrowed
ancestor cursors for name collection and hoisting. Switch scopes no longer clone
statement trees. It retains a separate 96-ancestor bound alongside the parser
and ordinary execution continuations. [Declaration-name validation](../tests/conformance/scope-names.md)
now charges temporary records, sorting, comparisons and named diagnostics while
preserving duplicate and scope-conflict behavior.

[Flat executable ownership](../tests/conformance/flat-code.md) stores syntax edges
as typed IDs within immutable shared units. Escaping closures and cross-script
callbacks retain their own units. Token pages avoid relocating the accumulated
token prefix. [Shared execution continuations](../tests/conformance/statement-frames.md)
now cover expressions, references and supported statements, including loops and
try/finally. [Ordinary activation and defaults](../tests/conformance/activation-frames.md)
also use the driver: retained shallow ordinary/default probes now complete 32
calls, with call 33 stopped by the existing logical ceiling. Native callback and
constructor bridges remain guarded. [Direct flat parsing](../tests/conformance/flat-parser.md)
now removes the temporary owning AST and second lowering pass. The parser emits
charged record pages and flat lists, preserving cover grammar, early errors and
lexical rescans. [Grammar continuations](../tests/conformance/parser-continuations.md)
also remove native grammar recursion: both modes of the original 32-nested-IIFE
test now pass within the unchanged compile quotas. The complete functions profile
has 515 passes and 616 unsupported variants and now has a healthy CI baseline.
The retained depth probes parse through 40 and run through 32 calls; declaration
traversal, labels, logical calls and native helpers retain independent limits.

[Labeled break and continue](../tests/conformance/labels.md) resolve ordinary
statement/loop targets, propagate through nested loops and switches, and preserve
finally overrides and function boundaries. The complete three-directory profile
retains 100 passes and 25 unsupported modes, with 80 controls. Legacy labeled
functions remain incomplete. [Statement completion values](../tests/conformance/statement-completion.md)
now distinguish empty results from JavaScript `undefined` through the supported
blocks, branches, loops, switches, labels and try/catch/finally. Dynamic eval,
with, destructuring `for…of` heads and other unsupported statement forms remain gaps.

[Loose equality](../tests/conformance/equality.md) follows ordinary primitive
conversion and Boolean/Number/String dispatch. Null and undefined remain unequal
to false and zero; object identity and nullish comparisons skip conversion hooks.
Strict equality remains noncoercing. All four complete upstream directories
retain 190 passes and 96 unsupported variants, with 128 controls. Exotic values
and HTMLDDA remain gaps.

[Relational comparisons](../tests/conformance/relational.md) convert both operands
in source order before choosing UTF-16 string or numeric ordering. Live hooks,
boxed strings, abrupt completion and unordered NaN are covered. The four complete
upstream directories retain 300 passes and 64 unsupported variants, with 128
controls. BigInt and broader exotic conversions remain incomplete. Symbol conversion hooks and numeric rejection now have separate coverage.

[Ordinary addition](../tests/conformance/addition.md) converts both saved operands
left-to-right before selecting numeric addition or UTF-16 concatenation. Live
valueOf/toString hooks, boxed values, exact exceptions and += reference order are
preserved. The complete addition profile has 67 passes
and 28 unsupported modes, with 64 verified controls. Symbol.toPrimitive now participates in conversion and primitive Symbol addition throws TypeError. BigInt addition remains incomplete; the existing profile policy is unchanged.

Object initializers preserve computed-key evaluation/coercion order, UTF-16 names, method/accessor descriptors and the special static `__proto__` form. Symbol keys and inferred method/accessor names now work. Spread, async/generator methods and `super` remain unsupported; see [object literal coverage](../tests/conformance/object-literals.md).

Array reversal uses bounded ordinary property operations, preserving holes, inherited indices, accessor order and abrupt completion on supported receivers. Number radix formatting adds exact finite safe integers for bases 2–36; nondecimal fractions and larger magnitudes remain explicitly unsupported. See [Array/Number method coverage](../tests/conformance/array-number-methods.md).

All eight Number constants now have their standard values and immutable flags.
Number.isFinite, isNaN, isInteger and isSafeInteger classify primitive numbers
without coercion, including subnormal values, negative zero and rounded large
integers. Saved aliases retain native identity through method/global replacement.
Number and the global isFinite/isNaN functions now perform
[ordinary numeric conversion](../tests/conformance/numeric-conversion.md),
preserving live hooks, receiver identity and abrupt effects. Symbol numeric conversion now throws TypeError; BigInt conversion and general constructor infrastructure remain incomplete.
The global parseInt/parseFloat functions use ordinary string-hint conversion,
and [Number parsing aliases](../tests/conformance/numeric-parsing.md) share
the same intrinsic identities. The
[static builtin scope](../tests/conformance/number-statics.md) and
[complete pinned inventory](../tests/conformance/test262-number-statics.md)
record 260 passes and 80 unsupported variants, with all 104
assertion checks verified.

Untagged template literals support nested substitutions, cooked escapes and multiline text. Each substitution uses string-hint conversion before the next expression executes; tagged templates remain unsupported. See [template literal coverage](../tests/conformance/template-literals.md) and the separate [57-source upstream inventory](../tests/conformance/test262-template-literal.md), which retains 82 passing and 32 unsupported variants.

The four [URI encoding/decoding functions](../tests/conformance/uri.md) preserve
ordinary string conversion, strict percent-encoded UTF-8 and reserved-character
behavior. Encoding rejects lone surrogates; decoding preserves raw unescaped
UTF-16 units and throws URIError for malformed escapes. Complete upstream
coverage retains 234 passes and 112 instruction-limit
stops, with 128 controls. Those stops prevent a healthy URI baseline.

JavaScript strings retain UTF-16 code units, including unpaired surrogates. Length, indexed access, `charAt`, `charCodeAt`, `codePointAt`, `slice`, `substring`, string searches, `match`, `search`, `replace`, string/RegExp `split`, array `join`, trimming, selected case conversion and `String.fromCharCode`/`fromCodePoint` operate on this representation. String-to-number conversion recognizes ECMAScript whitespace and decimal, hexadecimal, binary and octal forms. This is a bounded subset: normalization, locale-sensitive operations, Symbol-based RegExp dispatch/species and complete generic receiver/prototype behavior are absent. Each string is limited to 262,144 code units (512 KiB of backing storage), within the cumulative estimated 8 MiB script allocation budget.

RegExp matching uses a custom parser and an explicit backtracking stack over UTF-16 code units. Captures, empty-match progress, greedy/lazy repetition, named groups, numeric/named backreferences and positive/negative lookahead retain their implemented ECMAScript behavior. Case-insensitive non-Unicode matching follows one-code-unit uppercase canonicalization using Rust’s Unicode tables. Pattern literals use the parser’s expression context to distinguish division. Patterns are limited to 8,192 code units, 4,096 syntax nodes and 128 captures; parser/assertion nesting and repetition counts are also bounded. Compilation, backtracking, native output and callbacks consume bounded work/allocation budgets; a costly pattern can terminate with an uncatchable resource error. Unicode flags `u`/`v`, lookbehind and other listed missing syntax are explicitly unsupported. The separate [RegExp corpus](../tests/conformance/test262-regexp.md) is a narrow measurement, not full RegExp conformance.

DOM text, attributes, console output and display still use UTF-8. Valid surrogate pairs cross that boundary losslessly; unpaired surrogates become U+FFFD, without changing the original JavaScript string. Ordinary object keys retain every code unit, but non-scalar property names on host objects are unsupported. This boundary is not complete DOMString compatibility.

`textarea.value` uses the text storage shared by the current editor and form serializer, including script assignment and input-handler changes. This is not the complete HTML current/default/dirty-value model: assigning a value still changes child/default text, and reset semantics remain incomplete.

HTML foreign-content support includes adjusted SVG names, the defined XLink/XML/XMLNS attribute mappings, MathML text and annotation integration points, foreign CDATA, breakout tokens, and namespace-aware selectors/serialization. The namespace enum is deliberately limited to HTML, SVG and MathML; this is not an XML parser, full SVG DOM, or MathML layout engine. SVG script execution and foreignObject painting remain unsupported.

Fragment parsing initializes tokenizer state, the adjusted current node, insertion mode, document mode, encoding and form pointer from the actual context. Templates own separate hosted document fragments; normal document queries and layout exclude their contents. Template innerHTML, fragment queries/transfer and bounded deep cloning preserve nested template and namespace identity. Host ownership participates in mutation and IPC cycle/depth checks. Synchronous parser scripts, declarative shadow roots and content patching remain unsupported. Retaining quirks mode improves parsing but does not implement all quirks rendering behavior.

Fetched HTML, external CSS and classic scripts now choose encodings using BOM/transport/declaration/context precedence. Late accepted HTML declarations can cause one cached-byte reparse before scripts or subresources; POST requests are not repeated. Document encoding metadata survives IPC and is exposed through the characterSet aliases. This is a bounded whole-body loader, without streaming parser-script interaction or an exhaustive encoding conformance suite; see [encoding behavior](ENCODING.md).

Grid supports explicit signed lines and spans, implicit cyclic tracks, row/column auto-placement with dense packing, separate gaps, minmax/fraction track sizing, spanning intrinsic contributions and item/content alignment. Track, occupancy and shared work limits contain adversarial grids. Named areas/lines, subgrid, full intrinsic sizing and writing-mode interactions remain incomplete; see [Grid coverage](../tests/reftests/grid.md).

Flex columns wrap against a definite available height or resolved maximum height and resolve flexible sizes separately per line. Row and column lines support bounded cross-axis distribution, stretch and wrap-reverse; intrinsic automatic minimums, baseline alignment and writing-mode completeness remain missing. See [flex wrapping coverage](../tests/reftests/flex-wrapping.md).

Physical floats shorten complete line bands, use margin boxes and side-specific clearance, and share or isolate exclusions according to the supported formatting contexts. Six additional pixel pairs and bounded-work regressions cover these cases. Complex margin collapse, intrinsic inline-word sizing and overlapping painting order remain partial; see [float coverage](../tests/reftests/floats.md).

Document URL metadata and the first connected HTML base element now control relative resolution while preserving the committed fetch origin. Frozen base URLs survive fragment navigation and validated snapshots. Linked/inline stylesheets support bounded leading imports, redirected response bases, inherited encoding, media conditions and imported cascade layers. CSS imports retain separate parse boundaries; layer identities and media conditions are structured metadata, so malformed source cannot change them. Anonymous imports share one identity across their constituent source segments. Media queries reevaluate on resize; reported color scheme, motion and pointer values are fixed rather than taken from the operating system. Grouped media conditions and width/height ranges share bounded evaluation on every resize; [media-query coverage](../tests/conformance/media-queries.md) records the exact subset. Dynamic loading, base targets, nested-document fallback and complete CSS Syntax remain incomplete; see [URL and stylesheet loading](STYLESHEET_LOADING.md).

Feature queries use a conservative syntax allowlist for implemented declarations and selectors, grouped Boolean conditions, and conditional imports. False import conditions suppress both fetching and layer registration. JavaScript `CSS.supports` exposes condition and property/value overloads through the same bounded evaluator, with ordered author string conversion and separate property/value parsing. Unsupported syntax can still return false even when some related rendering works; custom properties, escaped values and complete selector/CSS syntax remain incomplete. See the [exact positive inventory and bounds](../tests/conformance/supports.md) and [JavaScript API scope](../tests/conformance/css-supports-api.md).

Custom properties compute on the declaring element before inheritance. Token-aware substitution preserves quoted text, complete comma fallbacks and token boundaries; real function names are case-insensitive while custom names retain case. Selected dependency cycles invalidate their participating values, and unused fallbacks are not evaluated. Empty custom values remain distinct from missing values; empty ordinary substitutions use unset behavior. Dynamic variable names, registered properties, animation taint, complete CSSOM serialization and full ordinary-value parsing remain incomplete. See [custom-property behavior and limits](../tests/conformance/custom-properties.md).

Inline style operations now parse tokens instead of splitting at every semicolon.
They preserve quoted delimiters and escaped custom-name identity, separate
priority from value, and stage a single attribute update after author argument
conversion. Supported property aliases agree between lookup, assignment and
`in`; invalid writes and missing-property removals do not rewrite the attribute.
This remains a bounded inline CSSOM subset; [the exact scope](../tests/conformance/cssom-inline.md)
records value grammar, shorthand limitations and serialization gaps.

Computed JavaScript property keys use string-hint conversion. Plain assignment
defers conversion until after its right-hand side; compound assignments and
updates retain the key converted for their read. Null/undefined bases fail
before key conversion. Converted symbol keys retain identity. Proxies and broader missing ECMAScript features remain unsupported.

HTML disclosures keep closed content in the DOM, script/style/resource lifecycle and form serialization while excluding it from paint, hits and native editing. Named groups enforce one open member per ordinary tree. The first direct summary and generated default affordance support activation. Toggle delivery uses bounded host checkpoints, exact reentrant task/tracker cleanup, and native idle continuation; complete HTML event-loop timing remains unsupported. Quota failure suspends automatic notification dispatch until reload. See [disclosure behavior](../tests/conformance/details.md).

Absolute boxes resolve against the nearest positioned ancestor's padding box after normal-flow sizing, skipping intervening static boxes. Relative offsets preserve flow space. Fixed boxes use viewport coordinates for painting and hits and do not extend document scroll height. A bounded stacking pass distinguishes real contexts from auto-z paint groups, reconstructs clip scopes and gives hits the same paint order. Full inline fragmentation, hypothetical static positions, sticky/transform rules remain incomplete. Group opacity composites entire contexts with bounded intermediate surfaces; see [opacity coverage](../tests/reftests/opacity.md). See [positioning coverage](../tests/reftests/positioning.md).

Cascade layers preserve first-declaration order across sources, nested and anonymous identities, reversed important precedence, unlayered/inline ordering and bounded rollback candidates. Layered imports declare their layer even after a failed fetch when their media condition matches. CSSOM layer APIs, complete conditional syntax, origin/animation behavior and several property semantics remain incomplete; see [cascade layer coverage](../tests/conformance/cascade-layers.md).

Events retain private state, snapshot propagation paths and listener inventories, and clean dispatch state on exceptions or resource termination. AbortController and AbortSignal support synchronous trusted abort events, reason identity, throwIfAborted and signal-based listener removal; any/timeout and fetch cancellation remain unsupported. Ordinary listener exceptions are reported while later listeners continue. Native input/change events are non-cancelable; click/submit can cancel their host actions. Readiness remains synchronous. See [event behavior and limitations](../tests/conformance/events.md).

## Conformance path

1. A pinned [WPT HTML tree-construction corpus](../tests/conformance/README.md) now exercises 1,959 inputs in 3,876 scripting-flag modes. Current exact-tree results are 3,868 matched, two unchanged expectation-framing mismatches and six unsupported. All 412 fragment-mode cases match, including template contexts; synchronous parser scripts remain unsupported. Parse-error counts and encoding/quirks mode are not certified. Broad WPT testharness coverage remains to be added. A separate pinned [Test262 selection](../tests/conformance/test262.md) retains 326 unchanged sources and all 652 mode variants: 558 passed and 94 unsupported, with no failed or harness-error modes. Both strict and sloppy variants execute; 32 unchanged-harness/strict-semantics preflights pass. A separate [RegExp inventory](../tests/conformance/test262-regexp.md) retains all 145 sources and 290 variants: 262 passed and 28 unsupported, with 44 preflights and no failures or resource/timeout outcomes. These are selected cases, not a full Test262 pass rate; assertion preflights and previously passing cases gate the runner.
2. Implement the required testharness bindings and WebDriver/session interfaces. This engine currently has neither a full WPT harness nor WebDriver.
3. Track individual test outcomes and expected failures by specification; never substitute a hand-selected fixture pass rate for platform-wide conformance.
4. Expand parsing/tree-building before judging rendering failures; then extend CSS used values, layout algorithms, text, and the script language/runtime.
5. Reproduce regressions with reduced fixtures and retain them in the suite.

Normative starting points: [HTML parsing](https://html.spec.whatwg.org/multipage/parsing.html), [DOM](https://dom.spec.whatwg.org/), [CSS visual formatting](https://www.w3.org/TR/CSS2/visuren.html), [ECMAScript](https://tc39.es/ecma262/), and [SVG 2](https://www.w3.org/TR/SVG2/). The included implementation is far short of these complete specifications.
