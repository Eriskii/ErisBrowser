# Eris Browser

An independent Rust browser with custom HTML parsing, DOM, CSS cascade, layout, a JavaScript subset interpreter, SVG handling, and software painting. There is no Chromium, Firefox, WebKit, Servo, Ladybird, embedded webview, or external JavaScript engine in the rendering path.

**Status: an early browser implementation, not a fully web-compatible or production-secure browser.** The original requirements—every web standard, production security, and performance within 30% of Chromium—are not achieved. Many modern websites will not function. See [compatibility](docs/COMPATIBILITY.md) and [security](docs/SECURITY.md) for concrete boundaries.

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

The benchmark measures **warm-cache CSS computation + layout + software painting of the loaded page**. It excludes parsing, scripts, network, image decoding, PNG encoding, and native presentation. It is not a Chromium comparison or a general web-performance score. See [performance](docs/PERFORMANCE.md).

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
retain 88 variants: 64 pass, six fail on missing Date, and 18 remain unsupported.
Global-property corrections and URI bindings add ten passes; Window reflection adds 22 more. All 64 assertion
preflights verify; CI preserves those passes and the unchanged inventory.

The complete [array-sort directory](tests/conformance/test262-array-sort.md)
retains 107 variants: 59 pass, 44 remain unsupported, and four exceed the shared
script budget. Adding reduce enables the 5- and 11-element stability tests in
both modes. Those resource stops prevent a healthy
regression baseline; the full inventory remains part of local measurements.

The complete paired [reduce/reduceRight directories](tests/conformance/test262-array-reduce.md)
retain 1,034 variants: 860 pass, eight fail on missing Date behavior and 166
remain unsupported. Both methods and Number.MAX_SAFE_INTEGER add 764
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
95 variants: 65 pass, two fail on missing Date and 28 remain unsupported. Ordinary
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
Complete DOMString storage and Web IDL interfaces remain incomplete.

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
gain 46 passes across 66 modes; four Date-dependent modes still fail and ten
remain unsupported. Alternate Web IDL targets and broader constructor semantics
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
complete new 48-source selection. It records 88 passed, two failed and six
unsupported modes. All 8,006 older case observations remain identical; the full
inventory reaches 8,102 modes and 2,216 verified assertion controls. Unicode
regexp parsing, realms, Date and the other String symbol protocols remain incomplete.

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
The browser has an optional [Linux Vulkan presenter](docs/vulkan-rendering.md): build with `--features vulkan-presenter`, then launch the binary with `--presenter=vulkan`. It uploads custom CPU-painted frames; software remains the default and headless path. Custom Vulkan rasterization and compositing remain on the [development docket](docs/ROADMAP.md).

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
gains 328 passes, and both older String.lastIndexOf failures pass. Eight resource
stops, 47 unsupported modes and two Date failures remain visible; this inventory
does not qualify as a healthy baseline gate.
