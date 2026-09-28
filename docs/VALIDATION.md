# Validation record

## Isolated native pages, formatting recovery and UTF-16

Recorded September 28, 2026 UTC on x86-64 Linux with Landlock ABI 6 and seccomp enabled. The integrated suite passed **232 Rust tests**: 171 library, 22 browser/editor, one tree adapter, one CLI, 10 network, 20 pipeline and seven real-worker tests. No tests failed or remained ignored in the explicit `--include-ignored` run. Formatting, strict all-target Clippy, release compilation and ten Python harness tests passed. All 13 exact rendering comparisons passed. The deterministic stress run completed 15,000 cases with zero caught panics or invariant failures; one case reached the configured paint-work limit. These remain limited regression and smoke tests.

The native browser now starts a fresh child for each document. Real-kernel tests verify allowed document reads and rejection of reads outside the grant, symlink escapes, writes, deletes, metadata mutations, filesystem-flag ioctls, execution, Unix sockets/socket pairs, thread creation and TCP listeners; allowed nonblocking/bytes-available socket ioctls remain usable. A separate subprocess verifies rejection of an inherited descriptor. Worker tests cover load/render, shared raster transfer, edits/events, stale edit sequences, POST navigation, fragments, cancellation, reaping, bounded malformed framing and HTTP loading with the synchronous resolver. Nonblocking-pipe tests cover both stalled reads and a full input pipe. These tests are evidence for specific controls, not a sandbox escape audit or complete security certification.

Snapshot checks have regressions for disconnected cycles, parent links, depth and storage bounds, invalid geometry/clip stacks/rasters, strict framing, and 1,000 deterministic protocol mutations. Review also caught valid-output mismatches: unavailable images must preserve their fallback, expanded DOM text must use the DOM budget, and generated alt/control/marker/tab text must share the emitted-glyph cap. Those cases now have regressions. The syscall-filter tests independently interpret the emitted filter for native/foreign architecture and x32/argument-width edge cases.

HTML exact-tree comparisons now yield **2,446 matches, 1,012 mismatches and 418 unsupported cases** across the unchanged 3,876-case inventory. Formatting reconstruction and adoption recovery added 194 matches with zero regressions against the prior 2,252-match baseline; that gate passed before the new baseline was recorded. Disabled-mode matches are 1,233 and enabled-mode matches 1,213. Fragment contexts, parser scripting, namespaces and other incomplete semantics remain visible in the inventory.

The script suite includes 37 interpreter tests and two string-representation tests. They cover UTF-16 indexing, slicing, search and property keys, every code unit from 0x0000 through 0xFFFF in a JSON round trip, lone-surrogate callback/indentation behavior, and uncatchable resource limits. DOM/display conversion still replaces unpaired surrogates; this is documented as an incomplete boundary. Test262 is not yet integrated.

The Unicode/formatting example was exercised through a script click and visually inspected as a headless image. A native window rendered the same page through the confined worker and exited successfully. A separate native `https://example.com` run verified actual DNS, TLS and rendering with the syscall filter enabled. The headless benchmark still excludes process startup and IPC; no Chromium performance comparison or full-platform conformance claim is made.

## Parser, JSON and flex compatibility increment

Recorded September 28, 2026 UTC on the same Linux/Rust setup. The integrated run passed 183 Rust tests (131 library, 20 browser/editor, one tree adapter, one CLI, 10 network and 20 pipeline tests), ten Python harness tests, formatting, strict all-target Clippy, release compilation, and 13 exact rendering reference pairs.

The pinned WPT corpus contains 1,959 inputs and 3,876 scripting-mode cases. Exact-tree comparison produced **2,252 matches, 1,206 mismatches and 418 unsupported cases**, with zero regressions against the original matched trees. In directly comparable scripting-disabled mode, matches increased from 380 to 1,136: 756 newly matching trees. Enabled-mode support accounts for the remaining added matches. These are tree comparisons, not complete WPT passes; [the harness documentation](../tests/conformance/README.md) states the omitted checks and records the full inventory.

The harness checks input, expected-tree, context and corpus fingerprints. Tests verify preservation of CR and multiline node data, case/flag handling, corpus integrity, bounded subprocess output/time, and refusal to overwrite a regression baseline or record process errors. Previously matched cases gate CI, while remaining mismatches and unsupported cases continue to be reported.

The updated mutation run again completed 15,000 cases with no caught panics or invariant failures: 5,000 HTML pipelines, 627 accepted/4,373 rejected scripts, and 3,201 accepted/1,799 rejected SVG inputs. Maximum generated DOM size was 251 nodes; one case reached the painter's work limit and stopped within the configured bounds. JSON unit tests also include 500 malformed-input mutations. This remains limited smoke coverage.

The new `examples/standards.html` was rendered after clicking its JSON action, and its exact formatted result is an integration assertion. Both the full headless image and a native-window framebuffer were visually inspected. The native window closed successfully after a five-second smoke run. [Updated fixture timings](PERFORMANCE.md) cover the same limited phases as before.

Full platform compatibility, production security and performance within 30% of Chromium remain unfulfilled requirements.

## Initial implementation

Recorded September 28, 2026 UTC (September 27 local time), on Linux x86-64/NixOS with Rust 1.95.0. These checks cover this implementation's explicit subsets and regressions. They do not establish full web compatibility, production security, or Chromium performance parity.

| Check | Observed result |
|---|---|
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy --locked --offline --all-targets -- -D warnings` | Pass |
| `cargo test --locked --offline -- --include-ignored` | 149 passed, none failed or ignored |
| `cargo build --locked --offline --release` | Both browser and stress binaries built |
| `python3 tools/reftest.py --binary target/release/eris-browser` | 10/10 exact pixel comparisons passed |
| `timeout 180s target/release/eris-stress 5000` | 15,000 generated cases, zero caught panics or invariant failures |
| `python3 -m py_compile tools/benchmark.py tools/reftest.py tools/run.py` | Pass |
| `./run.sh --window-screenshot artifacts/window-final.png --exit-after 5` | Native window opened, framebuffer captured, exit status 0 |
| HTTPS render of `https://example.com` | PNG produced, exit status 0 |
| HTTPS load of `https://expired.badssl.com` | Rejected with expired-certificate error, exit status 1 |
| HTTPS load of `https://self-signed.badssl.com` | Rejected with unknown-issuer error, exit status 1 |

The Rust suite contains 100 library tests, 20 browser/editor tests, one executable integration test, 10 loopback HTTP tests, and 18 pipeline tests. It includes POST body/redirect behavior, MIME and origin restrictions, compressed-body limits, text decoding, inert content, disabled controls, DOM mutation, event cancellation, resource-limit termination, clipping, stale worker results, editing acknowledgements, and fragment navigation. Loopback tests require permission to bind temporary local sockets.

Reference fixtures cover box geometry, cascade/inheritance, flex, grid and fractional tracks, hidden content, preformatted text, table spacing, media/custom properties, and overflow clipping. Both sides use the same rasterizer; shared defects can go undetected. No upstream WPT or Test262 pass rate is claimed.

The deterministic mutation run used seed `0xe2152026` and at most 8 KiB per generated input. Of 5,000 script samples, 622 were accepted and 4,378 rejected; of 5,000 SVG samples, 3,252 were accepted and 1,748 rejected. Rejections are expected for malformed or unsupported input. The largest generated DOM had 272 nodes. This is a small mutation smoke test, not broad adversarial coverage, coverage-guided fuzzing, or an audit.

Native and headless output for the home page, forms, gallery, and a narrow viewport was visually inspected. Native clipboard contents were not read during validation. Windows and macOS were not validated.

The [performance record](PERFORMANCE.md) defines the measured phases and includes three fixture timings. There was no Chromium comparison. The [compatibility inventory](COMPATIBILITY.md) and [security boundary](SECURITY.md) list material unfulfilled requirements, including missing platform APIs and, at that initial checkpoint, the absence of OS process isolation.
