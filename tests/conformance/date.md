# Independent Date semantic fixtures

The unchanged [local fixture](date.js) contains **203 independently authored
cases, executed in both sloppy and strict mode (406 modes)**, plus twelve
separate paired harness controls. The [evidence record](date.json) preserves
the expected observations, source/mode/helper fingerprints and complete
published-before results. Fixture SHA-256 is
`9c7fd9adf2cb973f72747c200f2eaeb279afe65b72b1114af2a19237ae60089e`.

Cases were derived from the current [ECMAScript Date algorithms](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-date-objects)
and [Annex B Date properties](https://tc39.es/ecma262/multipage/additional-ecmascript-features-for-web-browsers.html#sec-additional-properties-of-the-date.prototype-object),
the original Date integration plan and unchanged pinned tests. They were
frozen before any candidate evaluation and without inspecting candidate
implementation or outcomes. Every semantic case first checks Date availability,
UTC epoch construction and exact ISO formatting. Method error tests also call
the selected method successfully before their negative assertions.

Coverage includes constructor/static/prototype metadata and identities,
internal-slot brands, TimeClip boundaries, ordered coercion and abrupt
completion, construction with a live newTarget prototype getter, setter
snapshot/reentrancy semantics, UTC and local field arithmetic, ISO and required
own-format round trips, JSON and Symbol hooks, saved aliases and Annex B
getYear/setYear/toGMTString behavior. An invalid saved Date whose setter's
coercion changes its slot retains that author change on the specified early
NaN return.

Local expectations use the **actual captured host**, without setting `TZ` or
substituting UTC. Python `time.localtime`/`time.mktime` supplies independent
ordinary-date facts; `ZoneInfo` reads the exact `/etc/localtime` bytes for the
earlier-fold/pre-gap-offset inverse cases. Replay binds timezone bytes and the
`TZ`, `TZDIR`, `LANG`, `LC_ALL`, `LC_TIME` values. These facts were captured on
a non-UTC EST/EDT host. Locale presentation is tested only where the core
specification defines behavior; no Intl implementation is assumed. A separate
wall-clock bracket permits one second of backward clock adjustment and is
not a deterministic clock-injection test.

The published before adapter at commit
`f4af8ce6813337903e2b890909f581623ac92eb8` (SHA-256
`68a36c3e0d2641c63c90d72a5596e503fa19ad30e2e751888113f4c28b58d977`)
fails all 406 semantic modes at the positive Date-availability assertion;
all twelve harness controls verify. There are no process, timeout, unsupported
or resource outcomes in that local before run. The source and expectations
remain unchanged. Both the first candidate and the final frozen release pass
all 406 modes and all twelve controls, with identical complete observations.
The first candidate retains its explicit post-build-source-archive limitation.
The final release instead binds exact inputs captured before its build and
verified unchanged afterward.

Final adapter SHA-256:
`ab40a96f2cc1908359186e7c0648cecddc5ad8f8b2adf8a60de0e0d2c9a8f220`.
Exact production input digest:
`7bb6ff9b49fa7a66f88cf803a9fec4aee3eb0c4b76d3e14bd5472900c2218a02`.
The [limit comparison](date-limits.json) retains identical definitions for all
12 pre-existing script runtime/parser limit constants. Date's additional
storage and work are charged within those limits.

The complete Date profile records **1,162 passed, four retained for-of parser
failures and 22 metadata exclusions**, with all 340 controls verified. Its
known-state baseline gate passes with zero regressions; that does not mean all
cases pass. Across all 35 older profiles, the release adds **130 passes and
loses none**, with no changed control observations. Eight existing baselines
were strengthened; twenty healthy baselines remain byte-identical and the
seven observation-only profiles remain unbaselined.

The separate [complete pinned Date profile](test262-date.md) retains every
source and mode, including unrelated missing prerequisites. This fixture is
additional bounded evidence, not a claim of full ECMAScript, host timezone,
Intl, Temporal, Proxy or cross-realm compatibility.

The [integration record](date-integration-validation.json) binds the final
Rust 1.88/1.95 checks: 1,138 default and 1,149 feature-enabled tests per
compiler, strict Clippy in all four configurations, zero failed/ignored tests,
223 Python tests, both sets of 57 pixel references and 45,000 generated stress
cases without a panic/invariant failure. The retained HTML inventory remains
3,868 matched / two mismatched / six unsupported, with no regression.

[Worker timing evidence](date-worker-timing.json) preserves 27 fresh processes
and 540 warm samples per version across nine fixtures. Fresh startup medians
increase by approximately **10.1–10.4 ms** with timezone discovery's 10 ms
polling slices; warm-frame medians remain similar on these fixtures. This is
an uncontrolled before/after observation, not a Chromium comparison or
performance-parity claim. Its companion record distinguishes the benchmark
driver checkout hash from each binary's actual source inputs; original raw
reports remain unchanged. Polling optimization is a separate future change.
