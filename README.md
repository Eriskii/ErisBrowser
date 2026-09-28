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
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo test --locked -- --include-ignored
cargo build --locked --release
python3 tools/reftest.py --binary target/release/eris-browser
python3 tools/html_conformance.py --baseline tests/conformance/html-tree-current.json
python3 tools/test262_conformance.py --baseline tests/conformance/test262-current.json
python3 tools/test262_conformance.py --profile regexp --baseline tests/conformance/test262-regexp-current.json
python3 tools/test262_conformance.py --profile template-literal --baseline tests/conformance/test262-template-literal-current.json
python3 -m unittest discover -s tools -p 'test_*.py'
cargo run --locked --release --bin eris-stress -- 5000
```

Network and sandbox integration tests explicitly opt in: they bind temporary loopback HTTP servers or require the Linux sandbox features above. They require no public network. Other tests cover parser recovery, selectors and cascade, box layout, DOM/script interaction, script exhaustion, file scopes, geometry and raster limits. Reference tests compare independently constructed pages pixel-for-pixel with this renderer. They are a small self-authored suite, not a claim to pass the Web Platform Tests. The deterministic stress harness mutates HTML/CSS, script, and SVG seeds and checks bounded-output invariants; it is smoke fuzzing, not coverage-guided fuzzing.

CI uses a Cargo target runner to close ambient build/runner descriptors before
each Rust test executable starts. Worker rejection of unexpected descriptors,
including descriptors deliberately opened by a test, remains enforced.

The [validation record](docs/VALIDATION.md) lists observed results and their limits. A pinned upstream HTML tree corpus now provides exact-tree comparisons and a regression baseline; [its documentation](tests/conformance/README.md) records all mismatches, unsupported modes, and untested semantics. A pinned [Test262 selection](tests/conformance/test262.md) runs unchanged upstream tests and assertion harnesses with explicit failure/unsupported categories. Neither runner establishes platform-wide compatibility.

## Implementation

The [architecture notes](docs/ARCHITECTURE.md) describe the page pipeline and native process boundaries.
The [development docket](docs/ROADMAP.md) includes a planned Vulkan rendering backend.

| Module | Responsibility |
|---|---|
| `dom.rs` | Bounded HTML tree construction, arena DOM, entities, selector matching, serialization |
| `css.rs` | CSS parsing, indexed cascade and layers, inheritance, lengths, colors, variables, media queries |
| `layout.rs` | Block/inline flow, floats, flex, Grid placement/tracks, tables, controls, display lists, hit regions |
| `script.rs` | Custom lexer, parser, interpreter, lexical environments, DOM bindings and events |
| `regexp.rs` | Custom bounded UTF-16 regular-expression parser and backtracking matcher |
| `svg.rs` | Custom SVG geometry, paths, transforms and bounded RGBA rasterization |
| `graphics.rs` | Font metrics, cached glyph masks, clipped/limited software painting, group opacity, PNG output |
| `net.rs` | HTTP/TLS resource loading, redirect and file policies, decoding and byte limits |
| `text_encoding.rs` | HTML encoding prescan/reparse selection, MIME charset parameters, CSS and classic script decoding |
| `document_url.rs`, `stylesheet_loading.rs` | Document URL resolution and bounded stylesheet import loading |
| `page.rs` | Resource ordering, page lifecycle, scripting, forms and layout integration |
| `worker.rs`, `worker/` | Per-document child processes, bounded binary IPC, validated snapshots and Linux confinement |
| `js_string.rs` | UTF-16 code-unit strings and explicit scalar-text conversion |
| `browser.rs` | Native window, address bar, history, input, worker coordination and presentation |
| `edit.rs` | Unicode scalar cursor movement, selections, replacements and deletion |

Infrastructure dependencies provide TLS/HTTP (`ureq`/`rustls`), URLs (`url`), character encodings, font outline rasterization (`ab_glyph`), image codecs (`image`), native clipboard access (`arboard`), window events (`winit`), a pixel surface (`softbuffer`), and OS confinement wrappers (`landlock`, `rustix`, `seccompiler`). These are not web layout or script engines. Their transitive dependencies remain part of the security surface. The source forbids application-level `unsafe` Rust; dependencies can contain unsafe code.

Fonts are DejaVu; redistribution notices are in [assets/FONTS-LICENSE.txt](assets/FONTS-LICENSE.txt). Project code is MIT licensed. Vendored WPT test data retains its [upstream BSD license](tests/upstream/wpt-html/LICENSE.md); Test262 data retains its [upstream license](tests/upstream/test262/LICENSE).
