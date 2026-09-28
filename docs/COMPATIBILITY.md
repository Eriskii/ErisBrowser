# Compatibility status

This is an implementation inventory, not a conformance certificate. A feature listed as partial supports specific cases; it does not imply passing that specification's full tests. Unsupported syntax is generally ignored by HTML/CSS or reported by the script interpreter.

| Area | Implemented subset | Missing or incomplete |
|---|---|---|
| HTML | Tokenization, elements/attributes, comments, implied html/head/body, common optional end tags, raw text, all standard named entity data, numeric references, basic forms | Full insertion modes, adoption agency, foster parenting, foreign-content namespaces, incremental document.write parsing, encoding sniffing, complete quirks handling |
| DOM | Arena tree, textContent, attributes, creation/appending/removal, selector queries, bounded serialization, innerHTML subset, classList | Full interfaces, mutation observers, ranges, selections, shadow DOM, custom elements, DOM exceptions, complete live collections |
| CSS selectors | Type, universal, ID, class, attributes, descendant/child/sibling combinators, selected structural/logical pseudo-classes | Full escaping/namespaces, interaction-state pseudo-classes, generated pseudo-elements, complete Selectors conformance |
| CSS cascade | Specificity, source order, important declarations, inline styles, inherited properties, custom properties, selected media conditions | Layers, scopes, complete invalid-value semantics, supports/container queries, font-face |
| Layout | Block/inline flow, text wrapping, basic margins/padding/borders, flex rows/columns/wrap, basic grid columns, basic tables/spans, some positioning | Complete formatting contexts, floats, multicolumn/pagination, complete flex/grid sizing, stacking contexts, transforms, writing modes and bidi |
| Typography | Bundled sans/bold/monospace fonts, basic advances/kerning, synthetic italics, alignment/decoration | Complex text shaping, font fallback/downloads, full Unicode line breaking, hyphenation, variable fonts, complete font selection |
| Paint | CPU display lists, nested rectangular overflow clips, rounded backgrounds/borders, alpha blending, cached glyphs, raster images, scrolling and zoom | Full compositor, rounded clipping masks/separate overflow axes, filters, masks, shadows, gradients, animations, hardware acceleration, color management |
| Images | PNG, JPEG, first-frame GIF/WebP, BMP via image codecs; bounded SVG subset | Animated image playback, AVIF, complete SVG, image orientation/media fidelity and full responsive image selection |
| SVG | Basic shapes, paths, fill/stroke, viewBox, transforms, simple text | Full SVG/CSS integration, filters, patterns, gradients, masks, markers, foreignObject, animation, external resource references |
| JavaScript | Numbers/strings/booleans/null/undefined, arrays/objects, variables, operators, control flow, closures/functions/arrows, throw/try/catch/finally, selected builtins, DOM bindings and events | Full ECMAScript, classes/prototypes/modules, async/promises, complete Error constructors/prototypes/stacks, regex, template interpolation, JIT, GC, complete UTF-16 semantics and property behavior |
| Events | Basic listeners, inline handlers, bubbling, preventDefault/stopPropagation, readiness/click/input hooks | Complete event ordering, capture/options, trusted-event semantics, pointer/touch/drag, full keyboard/focus/composition events |
| Forms | Scalar-safe text cursor/selection editing, checkbox/radio toggling, bounded GET and HTTP(S) POST URL-encoded serialization, selected options, submitter method/action/encoding overrides, disabled-fieldset handling | Multipart/text-plain POST and file upload, constraints, complete reset/default behavior, native selection controls, autofill, password management |
| Network | HTTP(S) GET and URL-encoded document POST, verified TLS, gzip, relative URLs, checked redirects (301/302/303 convert POST to GET; 307/308 preserve it), bounded bodies, charset labels | Browser HTTP cache, cookies, authentication, full Fetch/CORS/CSP, HSTS persistence, HTTP/2/3, service workers, WebSocket/WebTransport |
| Browser | Single window/document, address entry, history, scrolling, basic keyboard focus, zoom, native text clipboard shortcuts, headless PNG output | Tabs, downloads, permissions UI, devtools, document find/selection, grapheme text editing, complete IME and accessibility |
| Other platform APIs | No broad implementation | Canvas APIs, WebGL/WebGPU, WebAssembly, audio/video, WebRTC, storage/IndexedDB, workers, geolocation, sensors, notifications, payments, credentials, extensions, printing and many others |

The URL parser and encoding libraries are infrastructure libraries, not independent reimplementations of those specifications. There is no claim that every dependency is authored in this repository.

Script exception handling preserves explicitly thrown values and exposes ordinary runtime failures as objects with `name` and `message`. Catch bindings have local scope, and `finally` preserves or overrides returns and loop control. Execution, allocation, nesting, and other resource limits terminate the current script entry without running `catch` or `finally`; these host limits cannot be overridden by page code.

## Conformance path

1. Import and record pinned upstream [Web Platform Tests](https://web-platform-tests.org/) and [Test262](https://github.com/tc39/test262) revisions.
2. Implement the required testharness bindings and WebDriver/session interfaces. This engine currently has neither a full WPT harness nor WebDriver.
3. Track individual test outcomes and expected failures by specification; never substitute a hand-selected fixture pass rate for platform-wide conformance.
4. Expand parsing/tree-building before judging rendering failures; then extend CSS used values, layout algorithms, text, and the script language/runtime.
5. Reproduce regressions with reduced fixtures and retain them in the suite.

Normative starting points: [HTML parsing](https://html.spec.whatwg.org/multipage/parsing.html), [DOM](https://dom.spec.whatwg.org/), [CSS visual formatting](https://www.w3.org/TR/CSS2/visuren.html), [ECMAScript](https://tc39.es/ecma262/), and [SVG 2](https://www.w3.org/TR/SVG2/). The included implementation is far short of these complete specifications.
