# Validation record

## Computed custom properties and Window descriptors

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **652 Rust tests** pass with `--include-ignored`: 528 library,
35 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 30 pipeline and 23 real-worker tests. None remain ignored.
The existing **47 Python checks** and all **52 exact pixel references** pass.

Custom values now compute before inheritance and substitute through CSS tokens.
The new ten-tile independent rectangle reference covers inherited aliases,
complete RGB fallbacks, case-insensitive functions, token separation, comments,
active cycles and unselected fallbacks. It differs from the preceding release
on **18,400 pixels**, then matches exactly after the correction. Page and real
worker tests also agree after a click changes the parent's variable while a
child shadows the referenced name. All **62 CSS tests** pass.

Review corrected two additional defects: rescanning large cached values after
an earlier substitution failed, and applying an empty ordinary font-family
instead of its unset behavior. Instrumented scratch code verifies that 300
discarded references scan zero cached-value bytes after the correction. Thirteen
independent semantic probes, three empty-value inheritance controls, 10,000
deterministic UTF-8-safe mutations and eight hostile inputs pass without caught
panics, timeouts or nonfinite geometry. The
[custom-property scope](../tests/conformance/custom-properties.md) records
current short-circuit semantics, bounds and remaining grammar/CSSOM limitations.

Window descriptor queries expose authoritative global data bindings with
ordered key conversion, current flags/values, lexical/prototype exclusion and
non-aliasing UTF-16 keys. Five new runtime groups and seven independent probes
cover mutations, deletion/recreation, getter avoidance, descriptor independence
and resource preflight. All **129 script tests** pass. This is bounded host
reflection; other Window operations and full Web IDL semantics remain incomplete.
The [API record](../tests/conformance/css-supports-api.md) gives exact coverage.

Pinned upstream outcomes remain HTML **3,868 matched / two mismatched / six
unsupported**, Test262 **536 passed / 116 unsupported**, RegExp **250 / 40**, and
templates **82 / 32**, with all preflights passing and no regressions. Corpus
bytes, runner policies and baselines are unchanged. A separate 15,000-case
mutation run has zero caught panics or invariant failures: 5,000 accepted HTML,
632 accepted/4,368 rejected scripts, and 3,347 accepted/1,653 rejected SVG inputs.
Maximum DOM size is 229 nodes, maximum display-list size 349 commands, and
21 cases stop within paint limits. New variable and descriptor seeds change
the mutation inventory, so these counts are not an acceptance-rate comparison.

The [before/after warm records](PERFORMANCE.md#computed-custom-property-checkpoint)
use verified frozen inputs and binaries, nine views, 1180×880 and 100 iterations.
All nine decoded final frames remain identical. Later medians and p95 values
are higher in this uncontrolled desktop run; they do not isolate the change's
cost or establish any Chromium-relative result. Script execution, native
presentation and the isolated Vulkan prototype are outside the measured loop.

Final release SHA-256 values:

| Binary | SHA-256 |
|---|---|
| `eris-browser` | `34ca6655dd8d0ec1873ba85e9f82716e75eca79b1b6d9223264d657f0069a621` |
| `eris-js` | `21c192148042d482cbed21403e20c6c2e360d31e0058c1c6cf70a2b7b14ad4eb` |
| `eris-dom` | `ecf9052c43c5a3a968596c31c43a6baca92a4bdd3b309d127ebe23058d64bafc` |
| `eris-stress` | `ed1c2a27e07044da41a58f110982d7c17adfe6994b59a7923e1e0300056805fd` |

The source-input digest is
`dde5ce131ea272fb5a737fddf4d048a7d1578371895ec17387cff6e5b689b86c`,
using the benchmark tool's documented inventory. Adapter report hashes match
these release binaries. Detailed session logs use the `vars-descriptors-final`
suffix under `artifacts/`. These checks do not establish full web compatibility,
independently audited security or the requested Chromium performance threshold.

## Isolated Vulkan transfer probe

Recorded September 28, 2026 UTC. The independently built
[`tools/vulkan-probe`](../tools/vulkan-probe/README.md) passes three Rust checks,
12 Python runner checks, formatting and strict Clippy on the installed Rust
1.95 toolchain. These counts are separate from the engine suite below. The
standalone crate pins its own wgpu 30.0.1 resolution; browser dependencies and
rendering behavior are unchanged. The separate [Rust 1.88 CI job for
`dde3ff1`](https://github.com/Eriskii/ErisBrowser/actions/runs/36396836593/job/108845129192)
subsequently passed formatting, strict Clippy and the Rust/Python checks with
the pinned lockfile. That compiler was not installed or executed for this local
host record; CI did not run GPU transfers.

The published source performs **18 exact comparisons across three adapters**:
NVIDIA RTX 4070 SUPER, AMD integrated RADV and CPU llvmpipe. Each adapter receives
three changed frames at 320×240 and three at 319×239. Uploads/readbacks preserve
all **5,509,476 compared RGBA bytes**, including alpha, changed frames and
non-aligned source rows. Enumeration and all device processes exit successfully
within their separate 25-second deadlines, with empty stderr logs. The runner
rejects changed adapter identities, missing/duplicate results, nonzero exits,
timeouts and excessive combined output; fake-process tests verify descendant
cleanup on success, timeout, interruption and read failure.

The [compact host evidence](../tools/vulkan-probe/evidence/host-transfer.json)
binds these results to final source, lockfile, runner, tests and binary hashes,
independently rechecked against the raw local run. The existing Vulkan loader
needed a subprocess-local library directory override; no host setting changed.
The probe forbids unsafe Rust in its source but uses native driver/library
implementations. Its explicit payload sizes exclude staging and driver storage;
process termination cannot guarantee recovery from uninterruptible kernel work.

This checks offscreen transfers only. No surface, shader, browser integration,
native presentation, custom GPU rasterization or performance comparison is
exercised. The [Vulkan rendering milestones](vulkan-rendering.md) remain open.

## JavaScript feature-query API

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **637 Rust tests** pass with `--include-ignored`: 515 library,
35 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 29 pipeline and 22 real-worker tests. None remain ignored.
The existing **47 Python checks** and **51 exact pixel references** also pass.

`CSS.supports` exposes both overloads through the existing conservative
feature-query evaluator. Independent review checked argument evaluation and
ordered string conversion, literal property names, separate value parsing,
function/namespace descriptors and shared work/allocation limits. Five new
runtime groups and two CSS groups cover exceptions, surplus arguments, invalid
syntax, UTF-16 conversion, repeated calls and uncatchable resource failures.
All **124 script tests** pass. This is focused implementation coverage, not an
unchanged upstream WPT run or complete CSSOM/Web IDL conformance; the
[API record](../tests/conformance/css-supports-api.md) lists the remaining gaps.

A new original HTML fixture selects green initial pixels and blue clicked
pixels through feature queries. Direct Page and confined-worker execution
agree exactly, without diagnostics; disabling scripts prevents the state
changes. Property/value injection probes remain false, and undeclared selector
namespaces invalidate the complete condition through negation.

Pinned upstream outcomes are unchanged: HTML **3,868 matched / two mismatched /
six unsupported**; Test262 **536 passed / 116 unsupported**, RegExp **250 / 40**,
and templates **82 / 32**. All assertion preflights pass. Source inventories,
runner policies and baselines are unchanged; no prior pass regresses. The
15,000-case deterministic mutation run reports zero caught panics or invariant
failures, with the same seeds and acceptance/output counts as the preceding
checkpoint. Existing performance measurements retain their historical build
identities; this API checkpoint adds no new performance or Chromium claim.

Final release SHA-256 values:

| Binary | SHA-256 |
|---|---|
| `eris-browser` | `b1834e4dbbbe3e0ef00ef68b80dfe0287b0adae31db46068383845fb3e3dedf4` |
| `eris-js` | `fb9215d17c8cff5e8f0593065983d7185c9d3e1272f1f2c1009597b31d6f8147` |
| `eris-dom` | `ecf9052c43c5a3a968596c31c43a6baca92a4bdd3b309d127ebe23058d64bafc` |
| `eris-stress` | `00bf46bad353886c718ec2b518414db54c75e118bd3e2e65ad5984f8748000f3` |

The source-input digest is
`33bc844220ee8758d7a60176d08914a96a97dc4fdd13049c9f016a66719b6d8a`,
using the benchmark tool's documented input inventory. Adapter report hashes
match these final binaries. Session reports use the `css-supports-final` suffix
under `artifacts/`. Full web compatibility, independently audited security and
the requested performance threshold remain unfulfilled.

## Selector tokens and bounded native task continuation

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and the explicit `--include-ignored` run pass. The **628 Rust tests**
comprise 508 library, 35 browser/editor, four HTML adapter, six JavaScript
adapter, five stress-invariant, four CLI, 17 network, 28 pipeline and 21
real-worker tests. None remain ignored. **47 Python checks** and **51 exact
pixel references** pass, including the new independent selector-comment pair.
These are bounded implementation checks, not platform-wide conformance or an
independent security audit.

Matching, specificity, candidate indexing and selector capability queries now
share token-aware grammar. Comments preserve token separation without creating
whitespace or merging names. Direct conditions, raw stylesheet preludes and
conditional imports agree. Review corrected malformed functional pseudo
acceptance, quoted delimiters affecting specificity, pseudo/modifier casing,
large `An+B` arithmetic and empty forgiving-list behavior. Thirteen independent
DOM/CSS matching comparisons, 10,000 UTF-8-safe mutations and four hostile
bounds probes pass. Escaped selectors, namespaces, advanced selector features
and complete declaration tokenization remain unsupported; the
[feature-query record](../tests/conformance/supports.md) gives exact scope and
work/storage bounds.

Disclosure tasks now keep queued records, element trackers and the active task
separate. Reentrant changes retain the correct original old state; finishing
clears the tracker without deleting its queued replacement. Exact identities,
nested synthetic dispatch, named groups, batch-boundary interleavings and
failure cleanup are covered. All **119 script tests** pass.

The native bridge continues pending notifications in interruptible batches,
with at most 64 events and one shared 100,000-step budget per checkpoint. Input,
navigation and shutdown retain priority. ERW8 carries validated
idle/pending/suspended state and a separate task command; rendering invokes no
author callbacks. The parent also checks task state against its trusted script
authorization. Quota failure suspends automatic retries until document
replacement, preserving unstarted notifications. Regressions cover 150 queued
events, persistent allocation exhaustion, callback instruction exhaustion,
partial SVG updates, disabled scripts and cancellation after sending the task
command. The complete HTML event loop, timers and microtasks remain absent.

The native-window probe first captured **64** delivered notifications. A later
capture of that same owned window showed **150**, without clicking or editing
page controls. UI process `1876256` exited with status zero; it, renderer `1876259`
and broker `1876260` were all gone afterward. The later compositor
capture was restricted to the focused test window; it is a visual continuation
check, not a GPU backend result. Session artifacts are
`artifacts/window-idle-tasks.png`, `window-idle-tasks-settled.png` and
`window-idle-tasks-processes.json`.

Pinned upstream results remain **3,868 matched / two mismatched / six
unsupported** HTML trees; Test262 **536 passed / 116 unsupported**, RegExp
**250 / 40**, and templates **82 / 32**. Harness preflights pass and no prior
pass regresses. Corpus bytes, selection policy and baselines are unchanged.
The final 15,000-case deterministic mutation run reports zero caught panics or
invariant failures. A new comment-token seed changes the HTML mutation inputs;
its acceptance/output counts are not a comparison against earlier seeds.

The [warm measurement](benchmark-tasks-selectors.json) covers nine views at
1180×880 over 100 iterations, with verified input/binary hashes. It measures
style/layout/software painting, excludes task execution and the native loop,
and establishes no Chromium performance result. Vulkan remains a documented
[future backend](vulkan-rendering.md).

Final release SHA-256 values:

| Binary | SHA-256 |
|---|---|
| `eris-browser` | `f9cd09b5d56b79e4ac8c157047e03b1cf8ae5edbf747455317e1bc2073114a1f` |
| `eris-js` | `52a14ad7a57803237be0c9b020321528171221024476324f5b20644a5c86a8da` |
| `eris-dom` | `ecf9052c43c5a3a968596c31c43a6baca92a4bdd3b309d127ebe23058d64bafc` |
| `eris-stress` | `165bdaa729f49e142a77c9ba93bfcd8e61365f8ac929354652cf8f9dc0d76f68` |

Detailed session logs and reports use the `tasks-selectors-final` suffix under
`artifacts/`. Source, adapter and benchmark hashes were checked after the final
build. The earlier records below retain their own historical scope and counts.

## Disclosures, feature queries, object literals and worker measurements

Recorded September 28, 2026 UTC on the same x86-64 Linux/Rust setup. The final
explicit `--include-ignored` run passes **604 Rust tests**: 489 library, 33
browser/editor, four HTML adapter, six JavaScript adapter, five stress-invariant,
four CLI, 17 network, 28 pipeline and 18 real-worker tests. None remain ignored.
Formatting, strict all-target Clippy, release compilation, **47 Python checks**
and **50 exact pixel references** pass. These are bounded implementation checks,
not a platform-wide conformance result or security certification.

Object initializers add computed names with ordered string-hint conversion,
UTF-16 method/accessor names and descriptors, and static prototype setters.
Generic Array.reverse preserves supported ordinary property operations, holes,
inherited indices and abrupt completion. Number radix formatting covers finite
safe integers in bases 2–36; nondecimal fractions and larger magnitudes remain
unsupported. All **115 script tests** pass. Independent review found uncharged
owned parameter/name copies when functions were created and called. Those copies
now reserve work and allocation before retention. An identical 40,938-byte probe
creating 1,000 methods with 1,000 long parameters now stops with an uncatchable
Resource error at about 14 MiB observed process RSS, versus normal completion at
about 79 MiB before; a one-parameter control still completes. RSS includes more
than the interpreter's estimated heap accounting and is not its allocation limit.

All earlier upstream passes and source/mode/harness identities survive. The
original Test262 inventory is now **536 passed/116 unsupported**: both unchanged
JSON ASCII-escaping variants newly execute successfully. Its 32 preflights pass.
The RegExp inventory remains **250/40**, and templates **82/32**, with 44
preflights each. None report failures, harness errors, resource limits or timeouts.
The original baseline advances only those two statuses and its adapter hash;
corpus bytes and runner policy are unchanged. HTML remains **3,868 exact matches,
two mismatches and six unsupported modes**. Final adapter hashes are verified.

Feature queries use strict condition grammar and a conservative positive
property/value/selector inventory. False conditional imports neither fetch nor
register layers; true imports retain media resize behavior and fetch authority.
All **51 CSS** and **19 loader** tests pass. Review found positive sticky queries,
comment-separated selector tokens becoming descendant combinators, and long
undeclared namespace prefixes evading invalidation. Raw prelude preservation,
a bounded conservative commented-selector policy and token-span validation fix
those cases. Twenty independent direct/block/import probes pass, as does a
20,000-case malformed supports/stylesheet smoke. These checks do not establish
complete CSS Syntax or feature-query conformance.

Disclosures add first-summary/default-header behavior, closed-content visibility,
exclusive name groups, open/name reflection and coalesced ToggleEvents. Closed
contents retain DOM, script, stylesheet, resource and form activity. ERW7 carries
validated generated-summary actions; forged commands fail closed. Native focus
and pending edits follow disclosure visibility. Three independent rectangle
references and pipeline/real-worker checks cover these paths. Review also found
stale inline SVG rasters after toggle/input callbacks; refresh now follows each
interaction's callbacks and default actions, including partial mutations before
handled exceptions or quota termination. Two pixel regressions failed before
that correction and pass afterward. Toggle delivery remains a bounded checkpoint
approximation, including documented reentrant tracker differences, with no idle
pump, full task/microtask scheduling, complete keyboard timing or accessibility
implementation.

The deterministic **15,000-case** mutation run has zero caught panics or invariant
failures: 5,000 accepted HTML pipelines, 620 accepted/4,380 rejected scripts, and
3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size is 210 nodes and maximum
display-list size 1,173 commands; **27 cases reach paint limits** within checked
invariants. Added disclosure/feature-query/object seeds change the inventory, so
these totals are not comparisons with earlier acceptance counts.

The new field-guide example is checked at wide/narrow viewports, through grouped
summary/button clicks and toggle handlers, and against identical direct/worker
pixels. Release pixels match the visually inspected debug images. The final
native framebuffer is visually inspected, and the UI plus renderer/broker exit
with no tracked process remaining. The worker benchmark rejects document-load
error pages, preserves script/fragment pixels, and its Python runner validates
measurements and terminates the process group on timeout. These controls retain
the production confinement and snapshot validation path.

[Performance records](PERFORMANCE.md) retain verified source/binary hashes and
nine fixture views. Warm in-process runs use 100 iterations; the separate
confined-worker baseline uses five fresh processes with 100 warm frames each and
retains raw phase samples. The disclosure view records 1.222 ms in-process median
and 1.371 ms confined warm-frame median. These measure different paths in an
uncontrolled local environment, exclude native presentation and establish no
Chromium comparison. Vulkan remains planned; full compatibility, independently
audited security and the requested performance threshold remain unfulfilled.


## Template literals, media conditions and wrapped flex lines

Recorded September 28, 2026 UTC on the same x86-64 Linux/Rust setup. The explicit `--include-ignored` run passes **550 Rust tests**: 444 library, 31 browser/editor, four HTML adapter, six JavaScript adapter, five stress-invariant, one CLI, 16 network, 27 pipeline and 16 real-worker tests. Formatting, strict all-target Clippy, release compilation, **44 Python harness tests** and **46 exact pixel references** pass. These remain bounded implementation checks, not a full-platform conformance or security certification.

Untagged template literals now retain cooked UTF-16 segments, nested substitutions, line normalization and string-hint coercion in observable evaluation order. Five focused implementation groups cover grammar boundaries, side effects, malformed escapes, large strings, nesting and uncatchable limits. Independent review checked source rescans, allocation preflight and callback ordering; 318 deterministic adversarial parse probes completed without crashes or timeouts. Tagged templates, raw/call-site identity, dynamic eval and other missing language features remain explicit limitations.

The new pinned Test262 template directory retains all **57 unchanged sources and 114 modes**, including 32 parse-negative variants. It records **82 passed/32 unsupported**, with all 44 assertion preflights passing. Against the preceding binary with the same corpus/policy, all 34 prior passes survive and 48 variants newly pass; 20 old syntax failures are now classified as unsupported tagged templates rather than credited as passes. All source/mode/harness fingerprints match. The original String/JSON and RegExp inventories remain **534/118** and **250/40** passed/unsupported with zero regressions. HTML remains **3,868 exact matches, two mismatches and six unsupported modes**. Final adapter hashes are checked against the release binaries.

Media evaluation adds width/height range operators, grouped and/or/not conditions, escaped identifiers, exact equality and strict finite length parsing. Unknown truth remains unknown under negation, and recursive evaluation consumes both local and shared stylesheet work. Independent review found URL tokens mistaken for general-enclosed functions and bad URLs accepted inside unknown syntax; the corrected token validation has negative and positive controls in unit and pixel tests. This is still a bounded media-query subset, without complete CSS Syntax, full feature coverage or matchMedia.

Flex layout forms column lines against definite or maximum heights without incorrectly resolving indefinite percentage bases. Each line retains bounded flexible sizing and min/max freezing. Row and column align-content distribution, line stretch, wrap-reverse and overflow fallback have ten new implementation groups and four independent SVG-rectangle reference pairs. A 35,000-item test reaches output limits while retaining finite geometry and balanced clipping. Baselines, full intrinsic/automatic minimum sizing and writing modes remain incomplete.

The deterministic 15,000-case mutation run completed with zero caught panics or invariant failures: 5,000 accepted HTML pipelines, 651 accepted/4,349 rejected scripts and 3,347 accepted/1,653 rejected SVG samples. Maximum DOM size was 211 nodes and maximum display-list size 402 commands; **26 cases reached paint limits** within the checked invariants. New template/range/flex/URL-token seeds change the mutation inventory, so acceptance totals are not a comparison with earlier runs.

The responsive field-notes example exercises all three paths. Pipeline and real-worker tests click through nested interpolation, resize between wide/medium/narrow layouts and return to the original width while retaining script state. The final native framebuffer is visually inspected; the UI, renderer and broker exit and leave no tracked process behind. This is concrete integration evidence, not a complete security audit or full-platform compatibility claim. The eight-case [warm-render record](PERFORMANCE.md) has verified source/binary hashes; the new responsive fixture records a 1.949 ms median and 2.004 ms p95. Measurements exclude script execution, native startup/IPC and presentation and establish no Chromium comparison. Full compatibility, independently audited security and the requested performance threshold remain unfulfilled.


## Abort signals and deferred opacity allocation

Recorded September 28, 2026 UTC on the same x86-64 Linux/Rust setup. The final explicit `--include-ignored` run passed **525 Rust tests**: 421 library, 31 browser/editor, four HTML adapter, six JavaScript adapter, five stress-invariant, one CLI, 16 network, 26 pipeline and 15 real-worker tests. None failed or remained ignored. Formatting, strict all-target Clippy, release compilation, **41 Python harness tests** and all **41 exact pixel references** passed.

AbortController and AbortSignal now provide stable signal identity, branded readonly state, reason identity, idempotent abort, throwIfAborted, onabort, static AbortSignal.abort and signal-backed event-listener removal. Abort removes registered listeners before firing its synchronous trusted event; already-aborted signals suppress registration, while duplicate listeners retain their original signal. Five implementation groups and one review regression join the prior event tests, bringing the script suite to 90 tests. A real-worker integration test verifies that abort removes a canceling click listener and restores subsequent native fragment navigation. Dependent signals, timers, fetch cancellation and the broader asynchronous lifecycle remain unsupported.

Independent review found a work-accounting gap in repeated throwIfAborted calls with a long string reason. The shared thrown-value helper now reserves proportional work and UTF-8 diagnostic allocation before formatting, without reading properties from object reasons. Ordinary throws use the same preflight. A 65,536-unit reason repeatedly thrown and caught now reaches the uncatchable limit; a lone-surrogate allocation test preserves the stored reason. Signal state, observer IDs, removal traversal and recursive abort callbacks continue to share the interpreter's existing budgets.

Opacity groups now remain pending until a clipped nonzero-alpha pixel needs a surface. Pending ancestors materialize in order; empty, offscreen and transparent groups require no intermediate pixels. Scope commands still execute, preserving fixed descendants that escape document clips. Five new graphics groups exercise pending/suppressed states, first visible contributions, fixed escape, peak/cumulative caps, partial allocation failure and cleanup. All 25 graphics and 99 layout tests pass. Independent comparison with commit 2caff22 produced identical pixels for **3,000 mixed drawing streams, painted twice each**, including fractional clips, text, images, transparent sources and canvas reuse. This is bounded differential evidence, not proof of complete compositing correctness.

The CSS-wide keyword helper also avoids temporary identifier/lowercase strings for unescaped ordinary values while retaining escaped-token parsing and the existing work model. All 35 CSS tests pass. All previous upstream fingerprints and passing outcomes remain unchanged: HTML has **3,868 exact matches, two mismatches and six unsupported modes**; the original Test262 selection has **534 passed/118 unsupported** and the separate RegExp selection **250 passed/40 unsupported**. Their 32 and 44 preflights pass, and final adapter hashes were verified.

The deterministic 15,000-case mutation run completed with no caught panics or invariant failures: 5,000 HTML pipelines, 612 accepted/4,388 rejected scripts, and 3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size was 197 nodes and maximum display-list size 228 commands. **27 cases reached paint limits** and stopped within the checked invariants. The new AbortSignal seed changes the mutation inventory; these totals are not a comparison with the previous run.

The final native framebuffer was visually inspected, and its UI, renderer and broker exited and were confirmed absent from procfs. Nine decoded headless images match the preceding implementation exactly, including clicked desktop/narrow pages and both offscreen/visible opacity views. The seven-case [warm-render record](PERFORMANCE.md) has verified source/binary hashes and ran after heavy checks and the native window finished. The offscreen event median fell from 11.047 to 1.155 ms; the separately measured visible view rose from 11.763 to 12.385 ms. The benefit is specific to avoiding invisible groups, and the measurements remain uncontrolled and unrelated to Chromium parity. Full compatibility, independently audited security and the Chromium performance requirement remain unfulfilled.

Public [GitHub CI for commit 077feee](https://github.com/Eriskii/ErisBrowser/actions/runs/36387717213) passed every step after the following corrections. Initial public GitHub CI exposed newer Clippy slice-iteration warnings and inherited descriptors reaching worker startup. Fixed-size slice iteration resolves the lint warnings. The descriptor failure was reproduced locally with explicitly inherited pipes; the remote log does not identify their original source. A CI-only Cargo target runner now starts each test executable with inherited descriptors closed and stale jobserver variables removed. All **525 tests also pass through that runner with deliberately inherited pipes**, including the test that opens an unexpected descriptor after startup and verifies worker rejection. Six Python regressions cover descriptor closure, stdio/arguments/cwd, environment filtering, exit/signal status and startup errors. Production descriptor rejection and confinement are unchanged.

## Cascade layers, grouped opacity and DOM event dispatch

Recorded September 28, 2026 UTC on the same x86-64 Linux/Rust setup. The final explicit `--include-ignored` run passed **513 Rust tests**: 410 library, 31 browser/editor, four HTML adapter, six JavaScript adapter, five stress-invariant, one CLI, 16 network, 26 pipeline and 14 real-worker tests. None failed or remained ignored. Formatting, strict all-target Clippy, release compilation, 35 Python harness tests and all **41 exact pixel references** passed. These remain checks of implemented subsets, not certification of full standards support or production security.

Author cascade layers now preserve first declaration order, nested/anonymous identities, reversed important precedence, inline/unlayered tiers and bounded `revert-layer` candidates across source segments. Layered imports retain structured identities and media conjunctions, including conditional registration after failed fetches. Three new pixel pairs use independently specified opaque SVG rectangles. Loader review found and fixed malformed backslash/newline and EOF escapes being shortened into valid import keywords; regressions verify rejection, no unintended layer registration or fetch, valid escaped keywords and later-rule recovery. The loader has 15 focused tests and the CSS suite 35, including ten layer groups. [Cascade coverage](../tests/conformance/cascade-layers.md) documents grammar, allocation/work bounds and missing semantics.

Group opacity now paints an entire stacking context into premultiplied RGBA16 intermediates and composites it once into its parent. Six independent opaque-SVG references cover overlap, borders, nested contexts, fixed clips, wrapped inline groups, translucent images and root backgrounds. Transparent loaded images no longer paint fallback backgrounds underneath. ERW6 adds validated opacity commands and a shared typed clip/fixed/opacity stack; mixed closures, nonfinite/out-of-range opacity and excess nesting are rejected. Real-worker coverage exercises layered imports, scrolling and fixed opacity pixels across the process boundary. [Opacity coverage](../tests/reftests/opacity.md) records 64 MiB peak/128 MiB cumulative intermediate limits and the current full-viewport allocation cost.

Event, CustomEvent and EventTarget use private state and synchronous capture/target/bubble dispatch, with listener snapshots/removals, once/passive callbacks, cancellation, inline handlers and distinct Document/Window paths. Native events retain trusted state; author dispatch cannot create it. The 84-test script suite includes eleven event groups, with listener assertions also checked through the runtime console. Independent review exercised reentrancy, getter-side mutation, readonly/private metadata and exception cleanup. It found and fixed uncharged comparisons of cached inline-handler source: a 30 KiB handler read 3,000 times now reaches the script work limit before performing the former repeated scan. No additional concrete trust-state bypass, panic or cleanup error was found in that bounded review; this is not a general security audit. [Event limitations](../tests/conformance/events.md), including asynchronous tasks, shadow retargeting and specialized UI interfaces, remain explicit.

Inline stylesheet collection now validates media/type before copying, uses direct Text children for HTML and SVG styles, and shares limits of 256 styles, 8 MiB text and 200,000 direct-child visits per collection. Allocation is fallible and follows size/work preflight. This removes a nested SVG-style amplification path that repeatedly copied descendant text; tests cover initial loading, re-layout, exact style type matching, nested children and aggregate limits. JavaScript `textContent` retains its descendant semantics. Existing link MIME handling remains separate.

Every prior case/source fingerprint and pass survives the unchanged upstream gates. HTML records **3,868 exact matches, two mismatches and six unsupported modes**. The original Test262 selection remains **534 passed and 118 unsupported** across 652 modes; the separate RegExp selection remains **250 passed and 40 unsupported** across 290 modes. Their 32 and 44 harness preflights pass. Adapter hashes were checked against the final release binaries. No new upstream event or CSS WPT pass count is claimed.

The deterministic mutation run completed **15,000 cases** without caught panics or invariant failures: 5,000 HTML pipelines, 632 accepted/4,368 rejected scripts, and 3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size was 197 nodes and maximum display-list size 228 commands. **31 cases reached the configured paint-work/storage limits** and stopped within the tested invariants. Seeds now include layers, grouped opacity and custom event dispatch; changed seeds make acceptance totals incomparable with prior runs. This remains smoke mutation testing rather than coverage-guided fuzzing.

The new `examples/events.html` page was clicked and visually inspected at desktop and narrow widths. Its final release native framebuffer was also inspected; the logged renderer and broker enabled the existing confinement and were absent from procfs after the UI exited successfully. The six-fixture [warm-render benchmark](PERFORMANCE.md) ran after heavy checks and the window finished, with source and executable hashes verified. It includes software opacity compositing but excludes loading, scripts, process startup and IPC. Full web compatibility, independently audited security and the Chromium performance requirement remain unfulfilled.

## HTML recovery, document bases, RegExp, positioning and stylesheet imports

Recorded September 28, 2026 UTC on the same x86-64 Linux/Rust setup. The final explicit `--include-ignored` run passed **462 Rust tests**: 362 library, 31 browser/editor, four HTML adapter, six JavaScript adapter, five stress-invariant, one CLI, 15 network, 25 pipeline and 13 real-worker tests. None failed or remained ignored. Formatting, strict all-target Clippy, release compilation, 35 Python harness tests and all **32 exact pixel references** passed. These checks exercise implemented subsets; they do not certify complete standards support or production security.

The unchanged HTML inventory records **3,868 exact matches, two mismatches and six unsupported modes**. Every prior case/source fingerprint and all 3,716 prior matches were retained, with 152 newly matching trees. Recovery now covers additional block, ruby, list, select and frameset cases, leading line feeds, foreign NUL handling, and selected-option cloning into `selectedcontent`. The two remaining mismatches are the two modes of one pinned processing-instruction case whose expected tree has an extra trailing blank line; the harness and fixture remain unchanged. Six synchronous parser-script modes remain unsupported. Exact trees do not establish complete HTML conformance or parse-error accounting.

Documents now retain their authoritative URL and the first connected HTML base element's frozen URL. Tests cover detached/template bases, invalid first bases, moves/removal, mutation budgets, encoded query resolution, and same-document navigation. The native snapshot protocol preserves and validates frozen base state, including expanded non-ASCII URLs and fallback bases frozen before a fragment change. A base URL controls resolution without replacing the resource broker's document-origin authority. A regression prevents fragment syntax from bypassing a changed-origin or file target, and form fallback uses the document URL.

The custom RegExp parser and explicit-stack matcher add bounded non-Unicode UTF-16 patterns, captures/backreferences, lookahead, indices, flags and String integration. The original 326-source, 652-mode Test262 selection now records **534 passed and 118 unsupported**: all 532 prior passes and every case/source fingerprint remain intact. Its 32 preflights pass. A separate complete-directory selection at the same pinned revision contains 145 unchanged sources and 290 modes, recording **250 passed and 40 unsupported** with all 44 preflights passing. Neither selection has failed, harness-error, resource, timeout or adapter outcomes in this measurement. The new selection does not replace or narrow the original inventory. Unicode modes, lookbehind, Symbol/species and other missing semantics remain documented.

Independent RegExp review compared 5,600 generated supported pattern/input capture results with an installed Node oracle and found no differences or execution errors. Node is used only for this check and is not a runtime dependency. Ten targeted resource probes completed without a caught panic or observed quota bypass, including catastrophic backtracking, callback recursion and oversized custom capture results. Review also fixed bare named-reference escapes, observable flag coercion and capture-name enumeration order. These limited differential and adversarial checks are not an audit or proof of general correctness.

Positioning now defers out-of-flow geometry until containing-block dimensions are known; resolves opposing insets, auto margins and relative offsets; and reconstructs stacking contexts and clips in paint/hit order. Fixed commands retain viewport coordinates during native and headless scrolling and cross the validated IPC boundary. Six new independently constructed pixel references cover containing blocks, insets, stacking isolation, clips, fixed coordinates and relative flow. A 40,000-leaf nested fixed/overflow regression preserves typed scope balance and shared command/hit budgets. Remaining inline/static-position, sticky, bidi, transform and group-opacity gaps are explicit.

Stylesheet imports use redirected response URLs, parent encoding fallback, recursion-path cycle checks, media conditions and separate source segments. Loopback tests verify request ordering, no unintended origin escalation, encoding, redirects and cache behavior. A long-base/many-import probe that previously aborted now completes under a 512 MiB test cap after URL construction/cache work received shared limits. Review also fixed retained-segment accounting across loader calls, malformed import recovery, media-wrapper injection through attributes or unmatched source braces, and accidental matching of unknown media types/features or invalid dimensions. These bounds can reject valid but expensive or unusually malformed input; [stylesheet loading](STYLESHEET_LOADING.md) documents the supported grammar and remaining behavior.

The deterministic 15,000-case mutation run completed without caught panics or invariant failures: 5,000 HTML pipelines, 625 accepted/4,375 rejected scripts, and 3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size was 229 nodes and maximum display-list size 1,173 commands, with no painter budget stops. Seeds include named RegExp captures, backreferences, indices, replacement and splitting. Changed seeds make acceptance totals incomparable with earlier records; this remains smoke mutation testing rather than coverage-guided fuzzing.

The new `examples/positioning.html` page was exercised through a script click, fixed-position scroll assertions and the real confined worker path. Its final release native framebuffer was visually inspected; the UI, renderer and broker exited and were reaped after the window closed. The five-fixture [warm-render benchmark](PERFORMANCE.md) was run after the native window and heavy checks finished, with source and binary hashes checked against the frozen build. It excludes loading, scripts, process startup and IPC. Full web compatibility, audited security and the Chromium performance requirement remain unfulfilled.

## Templates, strict JavaScript, Grid and byte encodings

Recorded September 28, 2026 UTC on the same x86-64 Linux/Rust setup. The final explicit `--include-ignored` run passed **393 Rust tests**: 299 library, 30 browser/editor, four HTML adapter, six JavaScript adapter, four stress-invariant, one CLI, 13 network, 24 pipeline and 12 real-worker tests. None failed or remained ignored. Formatting, strict all-target Clippy, release compilation, 34 Python harness tests and all **26 exact pixel references** passed. These are checks of implemented subsets, not certification of complete standards support or production security.

The unchanged HTML inventory records **3,716 exact matches, 154 mismatches and six unsupported modes**. All 3,272 prior matches were retained, with 444 newly matching trees. All 412 fragment-mode cases and all 254 template.dat mode cases match. Templates now own separate hosted document fragments, with insertion modes, scoped queries, serialization, clone/transfer behavior and host-inclusive graph checks. Documents retain their doctype mode and encoding. The corpus still does not certify parse-error counts, complete quirks rendering, encoding detection or synchronous parser scripts.

The unchanged 326-source, 652-mode Test262 selection records **532 passed and 120 explicitly unsupported**, with no failed, harness, resource, timeout or adapter outcomes. Both sloppy and strict modes execute, with 266 passes each. All 265 previous passes survived the old-policy gate; the comma-expression slice case added one. The strict-policy change was reviewed against every original case/source/harness fingerprint before recording the new baseline. All 32 preflights pass. Four previously failing sources are now explicitly unsupported because they need dynamic eval, regular-expression literals or template interpolation; this does not represent implementation of those features. Strict directives/references/receivers, lexical initialization/loop scopes and mapped/unmapped arguments have separate regressions.

Fetched HTML uses BOM/transport/prescan selection and can reparse cached bytes once for an accepted late declaration. CSS and classic scripts select their own encodings with a document fallback. Tests cover quoted and duplicate MIME parameters, prescan bounds, actual meta processing, template declarations, readonly encoding getters, canonical IPC metadata and differing script charset cache entries. A loopback Shift_JIS page reaches decoded DOM text and CSS pixels with one original POST and one request per subresource. No author script or subresource request runs before the encoding reparse. String-based DOM APIs retain Unicode input unchanged.

Grid adds signed lines/spans, explicit and implicit tracks, row/column dense placement, minmax/fr sizing, spanning contributions, gaps and alignment. Seven new references use independently specified absolute rectangles. Review found and fixed copied track-list amplification, refunded source-scanning work during nested reflow, and case-folding of custom property names. Identical computed lists now share storage; unique track data is capped at 4 MiB plus shared defaults and bounded indexing metadata. Targeted 99,000-element inheritance and repeated-declaration probes completed under a tighter 512 MiB test cap; the actual native process cap remains 1.5 GiB. Existing cascade limits deliberately leave part of the universal-declaration probe unstyled. A nested-grid whitespace probe now consumes at most the shared 500,000-character source budget. Exhaustion can yield partial layout, and these probes are not a security audit.

Independent script review found and fixed a panic when RHS evaluation deletes a captured global binding, plus uncharged subtree/sibling traversal during DOM moves/removals. Regressions cover strict and sloppy writes, coercion side effects, hosted-template traversal, uncatchable exhaustion, and unchanged failed moves/fragment transfers. Native snapshot validation now checks template ownership edges, mode tags and canonical encodings; a real-process test clones template cards and invokes strict callbacks through the confined renderer/broker path.

The deterministic 15,000-case mutation run completed without caught panics or invariant failures: 5,000 HTML pipelines, 607 accepted/4,393 rejected scripts, and 3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size was 211 nodes and maximum display-list size 1,175 commands, with no painter budget stops. Seeds now include Grid, hosted templates, strict mode, arguments, cloning and global deletion. DOM invariants include hosted and detached graphs. Changed seeds make acceptance totals incomparable with previous runs; this remains smoke mutation testing rather than coverage-guided fuzzing.

The new `examples/templates.html` page was exercised through headless clicks, pipeline assertions and real worker commands, then visually inspected in the final release browser's native framebuffer. Its renderer and broker exited and were reaped after the window closed. The expanded four-fixture [warm-render benchmark](PERFORMANCE.md) retains source and binary hashes and excludes scripting, loading and native process costs. Full web compatibility, audited security and the Chromium performance requirement remain unfulfilled.

## Fragments, property descriptors, floats and isolated images

Recorded September 28, 2026 UTC on the same x86-64 Linux/Rust setup. The explicit `--include-ignored` run passed **330 Rust tests**: 245 library, 30 browser/editor, three HTML adapter, six JavaScript adapter, one stress-invariant, one CLI, 11 network, 22 pipeline and 11 real-worker tests. None failed or remained ignored. Formatting, strict all-target Clippy, release compilation, 34 Python harness tests and all 19 exact pixel references passed. These checks cover implemented subsets and concrete regressions; they do not certify full standards compatibility or production security.

The unchanged HTML inventory now has **3,272 exact matches, 590 mismatches and 14 unsupported modes**. All 2,856 prior matches were preserved. The 416 additions include all 404 supported fragment-mode cases; eight template-fragment and six synchronous-script modes remain unsupported. Fragment initialization uses its real context, tokenizer state, insertion mode and form pointer. The document's quirks mode is not retained, so fragment parsing still uses no-quirks mode.

The unchanged 652-mode Test262 selection now records **265 passed, five failed and 382 unsupported**, with no harness/resource/timeout/adapter errors. All 108 prior passes were retained. The original runner policy gained 155 passes; enabling the implemented for-in-order feature added two more. Every test/source/harness fingerprint was checked before the reviewed baseline update. Thirteen unchanged assertion/property-helper preflights pass. Strict mode, exotic array descriptors and many other language features remain unsupported.

Float support adds margin-box exclusions, same/opposite-side placement, clearance, shrink-to-fit widths and partial formatting-context isolation. Six new independently positioned pixel references bring the total to 19. A 30,000-leaf nested float/clip test exhausts command creation while preserving balanced clips; its output also passes the IPC snapshot validator. Complex margin collapse and painting order remain incomplete. Native focus now follows positive clipped layout geometry, while offscreen controls remain eligible; stale snapshots cannot restore hidden editing targets.

Cross-origin native images now use a fresh confined decoder for each response. Real-process tests cover PNG/SVG pixels, invalid JSON bodies, direct and redirected HTTP failure redaction, one-request decoder exit, process reaping and enabled confinement. Redirect taint is retained even when the final URL returns to the document origin, including cached aliases. Protocol tests reject malformed geometry, truncated payloads and oversized advertised response frames before payload allocation. PNG ancillary-metadata fixtures verify limits before decoder construction; WebP container/frame preflight addresses inner-frame allocations that the dependency's canvas limits do not cover. Remaining codec scratch storage and native process limits are documented in [security](SECURITY.md).

The expanded mutation seeds cover descriptors/accessors, for-in, contextual table fragments and floats. The 15,000-case run completed without caught panics or invariant failures: 5,000 HTML pipelines, 616 accepted/4,384 rejected scripts, and 3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size was 223 nodes, maximum display-list size 261 commands, and there were no painter budget stops. Changed seeds make acceptance counts incomparable with earlier runs; this remains bounded mutation smoke coverage.

The new `examples/flow.html` demo was clicked and visually inspected headlessly, then opened in the native browser and captured from its framebuffer. It combines floats, a descriptor setter and context-sensitive table replacement. Its pipeline test checks all three chapter updates reaching DOM/layout/paint. The native renderer and broker exited and were reaped after the window closed. Full web compatibility, audited security and the Chromium performance requirement remain unfulfilled.

## Brokered resources, foreign namespaces and Test262

Recorded September 28, 2026 UTC on the same x86-64 Linux/Rust setup. The complete explicit `--include-ignored` run passed **281 Rust tests**: 204 library, 26 browser/editor, two HTML adapter, six JavaScript adapter, one stress-invariant, one CLI, 10 network, 21 pipeline and 10 real-worker tests. None failed or remained ignored. Formatting, strict all-target Clippy, release compilation, 31 Python harness tests and all 13 exact pixel comparisons passed.

Native page resources now pass through a separate confined broker. The renderer has no filesystem read/write/execute grants and cannot create sockets. The parent authorizes one document URL/form body, records the broker's final URL, and checks subsequent snapshots against it. Real-process tests cover cross-origin document redirects, final-origin scripts, direct and redirected cross-origin image rejection, authorized POST bodies and 303 rewriting, cancellation during stalled broker I/O, one-shot cancellation propagation, and reaping both children. Kernel probes demonstrate renderer file/TCP/UDP/Unix-socket denial. Filter interpretation checks the added System V/POSIX queue restrictions and rejection of mutating process limits/scheduling. These are bounded tests of specific controls, not a complete sandbox audit.

Review found and fixed an image-kind origin bypass, namespace-map allocation amplification, and unmediated IPC syscall families. Native network images remain restricted to the committed origin until an opaque image-decoding boundary exists. The broker retains outbound IP access and scoped resolver/library/local-file reads; the UI still paints validated untrusted snapshots. Headless/library paths do not install this native confinement. These remaining boundaries are detailed in [security](SECURITY.md).

HTML now retains SVG/MathML namespace identity, adjusted foreign names and attribute namespaces, integration points and foreign CDATA. DOM metadata, selectors, layout controls, SVG painting and IPC were updated with it. The unchanged 3,876-mode HTML inventory yields **2,856 exact matches, 602 mismatches and 418 unsupported cases**: 410 additional matches with no previously matching trees lost. The old baseline gate passed before recording the new baseline. Both conformance runners now reject report/baseline aliases and adapters changed during a measurement.

The pinned [Test262 selection](../tests/conformance/test262.md) contains 326 unchanged upstream test files and all 652 required modes. Its release measurement is **108 passed, 140 failed, 23 harness errors and 381 unsupported**. All nine unchanged upstream assertion preflights pass, with no resource, timeout or adapter errors. Strict execution remains unsupported, and the 23 harness errors reflect unimplemented syntax in `propertyHelper.js`. Canonical intrinsic Error identity prevents forged constructor names from satisfying negative tests; actual subprocess regressions cover spoofed values, renamed genuine errors and wrong phases. Updating that runner policy preserved every case identity and outcome before the reviewed baseline was recorded. These results are a selected inventory, not a complete ECMAScript pass rate.

Interpreter work includes constructors, basic prototype chains, `this`, `instanceof`, function properties/hoisting and `switch`. A hostile JSON callback test exposed native stack exhaustion; calls, expressions, statements/hoisting and JSON traversal now share a weighted nesting guard. Mixed-recursion regressions terminate with uncatchable resource errors and verify restored accounting. Native focus/editing and page mutations also share namespace, attachment, inert, disabled-fieldset and readonly policies; stale snapshots cannot restore revoked editing authority.

The expanded deterministic mutation seeds cover foreign-content boundaries, namespace attributes, constructors, prototypes, switch and Error paths. The 15,000-case run completed without caught panics or invariant failures: 5,000 HTML pipelines, 623 accepted/4,377 rejected scripts and 3,347 accepted/1,653 rejected SVG samples. Maximum DOM size was 223 nodes; maximum display-list size was 136 commands, with no paint-budget stops. The seed inventory changed, so these acceptance counts are not a comparison against the previous run.

The new namespace demo was clicked and visually inspected headlessly; its pipeline test checks three colors reaching pixels, preserved namespaces/case, viewport scaling and released obsolete rasters. Native windows rendered the local demo and public `https://example.com` through distinct renderer/broker processes, captured their own framebuffers, and exited successfully. Both children were confirmed reaped afterward. [Warm fixture timings](PERFORMANCE.md) remain limited to style/layout/software paint and exclude the broker path. Full web compatibility, production security and the Chromium performance requirement remain unfulfilled.

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
