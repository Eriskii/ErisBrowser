# Validation record

## String lastIndexOf

The [implementation record](../tests/conformance/string-last-index-of.md) and
[evidence](../tests/conformance/string-last-index-of.json) compare against `569991c`.
The complete new **25-source / 50-mode** inventory gains **46 passes**, reaching
**48 passed / two failed**, with no exclusions. The remaining modes invoke missing
Array.lastIndexOf. Match/search gains **12 older passes**; all other **9,406 older
observations / 2,368 older controls** are identical. The **30-profile / 9,468-mode**
inventory has **2,448 verified controls** and 25 regression gates.

All **28 corrected local modes** pass. The original descriptor fixture accidentally
deleted its method through `verifyProperty`; its original before/after observations
remain in the evidence. A one-line restoration fix changes exactly two fingerprints,
and both binaries were rerun. No pinned source changed. Rust **1.88 / 1.95** pass
strict all-target Clippy and **1,030 default / 1,041 Vulkan-feature tests**, none
ignored. A deterministic search differential checks **12,288** UTF-16 inputs and
positions. **180 Python tests**, **15,000 mutation cases**, unchanged HTML observations
and both release builds' **57 CPU pixel references** pass. No independent review,
GPU exercise, security audit or Chromium performance comparison was performed.

## String and RegExp match/search protocols

The [implementation record](../tests/conformance/regexp-match-search.md) and
[evidence](../tests/conformance/regexp-match-search.json) compare against `d7cee45`.
The complete new **170-source / 340-mode** selection gains **138 passes**, reaching
**304 passed / 16 failed / 20 unsupported**. Remaining failures expose lastIndexOf
and BigInt dependencies. Every other new observation is identical, as are all
**9,078 older observations / 2,288 older controls**. Source fingerprints, policies
and 23 older baselines remain unchanged. The combined **29-profile / 9,418-mode**
inventory has **2,368 verified controls** and adds a 24th regression gate.

All **44 supported local modes** pass; two original modes still stop at the
unimplemented Array indexed-descriptor API. Their source/outcomes and all 24
original diagnostic fingerprints are retained. A separate inherited Object
prototype setter test passes. All 80 new-profile controls verify, versus 56 before.
Rust **1.88 / 1.95** pass strict all-target Clippy and **1,026 default / 1,037
Vulkan-feature tests**, none ignored. Both releases preserve **57 CPU pixel
references**. **177 Python tests**, **15,000 mutation cases** and unchanged HTML
observations pass. Callback recursion, infinite match results, heap refusal and
cleanup are tested. No independent agent review, GPU exercise or Chromium
performance comparison was performed.

## Required literal counts before regexp matching

The [implementation record](../tests/conformance/regexp-required-literals.md) and
[evidence](../tests/conformance/regexp-required-literals.json) compare against
`45fb59e`. Both unchanged XML-pattern modes now complete, bringing the constructor
directory to **774 passed / 200 unsupported / two script-work resource stops**.
All **9,076 other observations / 2,288 controls** are identical; sources, policies,
mode fingerprints and 23 healthy baseline gates are unchanged. All **30 frozen
local modes** pass, including eight former resource stops.

The direct comparison preserves **3,888 parse outcomes / 106,040 complete capture
or error outcomes**. Rust **1.88 / 1.95** pass strict all-target Clippy and **1,023
default / 1,034 Vulkan-feature tests**, none ignored. Both releases preserve
**57 CPU pixel references**. **174 Python tests**, **15,000 mutation cases** and
unchanged HTML observations pass. Seven alternating timing samples retain the
measured compilation overhead and matching results; they are synthetic direct
Rust calls, not browser workloads or Chromium measurements. No independent agent
review or GPU exercise was performed.

## Flat regexp group parsing and prefix analysis

The [implementation record](../tests/conformance/regexp-deep-groups.md) and
[evidence](../tests/conformance/regexp-deep-groups.json) compare against `1c660e8`.
Four unchanged Test262 modes for 200 nested groups now pass, bringing the complete
constructor directory to **772 passed / 200 unsupported / four resource stops**.
All **9,074 other observations / 2,288 controls** are identical. No source,
policy, mode fingerprint or healthy baseline changes. The remaining work-budget
stops keep this profile an observation inventory.

All **20 frozen local modes** pass after previously stopping on depth/capture
limits. A direct comparison preserves **3,888 parse outcomes / 106,040 complete
capture or error outcomes**. Small-stack regressions compile and match 2,000
nested groups; syntax, resource and lookahead execution guards remain tested.
Rust **1.88 / 1.95** pass strict all-target Clippy and **1,020 default / 1,031
Vulkan-feature tests**, none ignored. Both releases preserve **57 CPU pixel
references**. **174 Python tests**, **15,000 mutation cases** and unchanged HTML
observations pass. No independent agent review, GPU exercise or Chromium
performance comparison was performed.

## RegExp constructor classification and conversion

The [implementation record](../tests/conformance/regexp-constructor.md) and
[evidence](../tests/conformance/regexp-constructor.json) retain all 488 direct
constructor-directory sources: **768 passed / 200 unsupported / 8 resource
stops**, gaining **16 passes** over the preserved `f3a786b` binary. The resource
cases remain identical and prevent a healthy baseline for this new profile.
No old baseline changes. All **8,102 older case objects / 2,216 controls** remain
identical; the combined inventory reaches **28 profiles / 9,078 modes / 2,288
verified assertion controls**, with unchanged policies between comparisons.

All **36 corrected local modes** pass. An added fixture's unrelated prototype
assumption was corrected and remeasured on both binaries; its initial source and
observations are retained, as are all 20 original diagnostic fingerprints. Rust
**1.88 / 1.95** pass strict all-target Clippy and full tests: **1,017 default /
1,028 Vulkan-feature tests**, none ignored. Both release builds preserve all
**57 CPU pixel references**. **174 Python tests**, **15,000 mutation cases** and
unchanged upstream HTML observations pass. Resource tests cover parser charges
on success/failure, every representative work cutoff, recursion and heap refusal.
Agent quota remains unavailable; no independent agent review, GPU rendering or
Chromium comparison is claimed.

## RegExp split and species construction

The [implementation record](../tests/conformance/regexp-split.md) and
[evidence](../tests/conformance/regexp-split.json) cover 48 complete pinned sources:
**88 passed / 2 failed / 6 unsupported modes**, gaining **86 passes** over the
preserved `cd1c016` binary. The remaining failures now reach missing Date; realm
and Unicode regexp prerequisites remain unsupported. Every older case object and
assertion control is identical. The inventory reaches **27 profiles / 8,102 modes /
2,216 verified controls**, with only one new baseline and no old policy changes.

All **32 supported local modes** pass; the two original array-descriptor
prerequisite modes remain unsupported. Rust **1.88 / 1.95**, strict all-target
Clippy and full tests pass: **1,014 default / 1,025 Vulkan-feature tests**, none
ignored. Both release builds preserve all **57 CPU pixel references**. **171
Python tests**, **15,000 mutation cases** and the unchanged upstream HTML baseline
pass. Resource tests cover recursive callbacks, hostile capture lengths and
indices, heap refusal and cleanup. Agent quota remains unavailable; no independent
agent review, GPU rendering or Chromium comparison is claimed.

## String conversion, search and custom splitting

The [implementation record](../tests/conformance/string-conversion.md) and
[evidence](../tests/conformance/string-conversion.json) retain a new complete
242-source String search/split selection: **470 passed / 2 failed / 12 unsupported
modes**, gaining 18 passes over the frozen `707ada1` binary. The two older
string-slice failures also pass; every other old case observation is identical.
All **8,006 modes** preserve their fingerprints and previous passes. All **2,144
assertion controls** verify; the 2,072 older controls and all old policies remain
unchanged. One existing baseline and one new baseline record these results.

The frozen local conversion and split-hook fixtures now pass all **54 modes**.
Rust **1.88 / 1.95** pass strict all-target Clippy and full tests: **1,011 default /
1,022 Vulkan-feature Rust tests**, none ignored. Both release configurations
preserve all **57 CPU pixel references**. **168 Python tests** and **45,000
mutation cases** pass. Upstream HTML case objects remain identical. Resource
tests cover recursive conversion, Symbol getters/calls, exhausted heap refusal
and counter/frame cleanup. No GPU rendering or Chromium comparison was run.
Agent quota remains unavailable; no independent-agent review is claimed.

The two remaining failures require undeclared BigInt syntax in unchanged upstream
sources. Unsupported eval and RegExp parsing cases remain visible. Built-in
RegExp Symbol.split/species, other String symbol protocols and Function source
printing remain incomplete; this is not full standards conformance or audited
security.

## Dynamic ordinary Function construction

The [implementation record](../tests/conformance/function-constructor.md) and
[evidence](../tests/conformance/function-constructor.json) cover the 200-source
Function constructor selection: **241 passed / 50 unsupported modes**, gaining
157 passes over the frozen old binary. Older selections gain another 35 passes;
two previously unsupported string-slice modes now reach retained failures. All
**7,522 mode fingerprints** and prior passes remain preserved, with **2,072
verified assertion controls**. Existing policies and all 2,008 old controls are
unchanged. Three old baselines and one new baseline record the reviewed results.

All **30 frozen local semantic modes** move from unsupported to passed. On Rust
**1.88 / 1.95**, strict all-target Clippy and full tests pass: **1,008 default /
1,019 Vulkan-feature Rust tests**, none ignored. Both release builds preserve
**57 CPU pixel references**; GPU rendering was not exercised. **165 Python tests**
and **15,000 mutation cases** pass. All upstream HTML case objects remain identical.
Resource tests exercise successful and failed compilation charges, every work
cutoff for representative parses, heap refusals, recursion and counter cleanup.

The initial full Rust run exposed an obsolete assertion that Function was
unsupported; it now asserts the constructed function's result. A Python policy
test was updated to recognize the new profile's explicitly selected rest support.
The original malformed resource-test draft and rest-parser failure are described
in the implementation record. Function string conversion/source printing and
literal unpaired surrogate source remain incomplete. Full web compatibility,
independently audited security and Chromium performance parity remain unfulfilled.

## Symbol targets and constructor policy expansion

The [policy checkpoint](../tests/conformance/constructor-policy.md) admits 88
previously excluded modes. Its [evidence](../tests/conformance/constructor-policy.json)
separates 86 cases that already passed from two Symbol constructor failures fixed
by the implementation. All 7,231 mode fingerprints remain unchanged and no prior
pass is lost. Two cross-realm exclusion diagnostics change without admitting
those tests. All other observations and all 1,960 old assertion controls are
unchanged; 48 added controls bring the verified total to 2,008.

The ten frozen local modes pass after failing before the fix. On Rust 1.88 and
1.95, strict all-target Clippy and all tests pass: **1,005 default / 1,016 feature
Rust tests**, none ignored. Both release builds preserve **57 CPU pixel references**.
**162 Python tests** and **15,000 mutation cases** pass. HTML case objects remain
exactly unchanged. Eighteen policy baselines were reviewed and updated; the four
resource-observation profiles remain observations. These checks do not establish
full platform compatibility, audited security or Chromium performance parity.

## Reflect calls and constructor targets

The [constructor checkpoint](../tests/conformance/construction.md) adds private
`new.target` state, Reflect call/construction operations, bound forwarding and
alternate allocation prototypes. Its [evidence](../tests/conformance/construction.json)
binds the source, binaries, complete inventories, controls and observations.

The 33 newly imported Test262 sources produce 66 modes: **52 passed, four failed,
ten unsupported**, gaining **46 passes** against the preserved prior binary.
All **160 new assertion controls** verify. The four failed modes reach missing
Date support; unsupported modes and unchanged sources remain in the inventory.
All **7,165 modes and 1,800 controls** in the older 22 profiles are exactly
unchanged, including observations and policies. All upstream HTML case objects
are also unchanged: 3,868 matched, two mismatched, six unsupported.

On Rust **1.88 / 1.95**, formatting, strict all-target Clippy and full tests pass:
**1,003 default / 1,014 Vulkan-feature Rust tests**, none ignored. Both release
builds preserve **57 headless CPU pixel references**; GPU presentation is outside
these checks. **161 Python tests**, all **26 local constructor modes**, and
**45,000 deterministic mutation cases** pass. Existing resource quotas and call guards remain in force; private slots and forwarded argument storage are charged. A targeted
exhausted-heap test first exposed allocation before bound-arrow rejection, then
passed after validation moved ahead of copying. No compatibility-wide, audited
security or Chromium performance conclusion follows from these results.

## Optional Vulkan upload presenter

The optional Linux presenter uploads frames produced by the custom CPU painter.
Software remains the default/headless path. The [implementation record](vulkan-rendering.md)
and [compact evidence](evidence/vulkan-presenter.json) bind the final source,
dependencies, binaries, case inventories and native outcomes to their hashes.

Formatting passes. On both Rust **1.88** and local **1.95**, strict all-target
Clippy and full tests pass: **997 default / 1,008 feature-enabled Rust tests**, none ignored.
Both release binaries preserve all **57 headless CPU pixel references**. Those
references do not exercise the GPU. **158 Python checks** pass, and **45,000
generated cases** finish without a caught panic or invariant failure. Three
pre-existing Clippy findings were fixed without changing their operations.

The actual browser presented on **Wayland / NVIDIA GeForce RTX 4070 SUPER** using
Bgra8Unorm, Srgb display color space, opaque composition and FIFO. **Eight distinct
acquired textures matched all 30,841,144 bytes** before presentation, including
changed viewport revisions and odd widths. A missing-driver run cleanly selected
software after confirmed release. Missing-driver verification and insufficient
sampling both exited nonzero. **20 additional missing-driver launches** passed,
including ten traced runs. All tracked test processes exited and were reaped.
No new compositor capture, alternate-adapter result or performance claim is made.

Initial native failures remain recorded. The first presenter exposed transient
Vulkan loader manifest descriptors inherited by the page worker. Four of ten
traced diagnostic launches reproduced rejection. A graphics/spawn mutex candidate
still failed: hardware initialization retained an inheritable `/dev/udmabuf`
handle, rejected in three diagnostic launches. The mutex was removed. The final
feature-enabled launch stage marks all non-stdio descriptors close-on-exec and
execs the worker; its strict descriptor rejection stays intact. The real-binary
regression checks both direct rejection and sanitized startup, preserving the
parent's descriptor flags and handle. Renderer/broker/decoder policies and IPC
framing are unchanged; only the internal feature-enabled launch command changes.

All **22 Test262 profiles / 7,165 modes / 1,800 controls** have identical sources,
policies, cases and observations to the concat checkpoint. No baseline changes.
HTML remains **3,868 matched / two mismatched / six unsupported**. The combined
optional graph adds 47 lock packages (35 extra active Linux dependencies); no
existing version, source or checksum changes. The two stripped browser binaries
are 11,327,312 bytes by default and 15,123,208 with the optional feature. These
sizes are observations, not startup, throughput or memory benchmarks.

The tests cover mailbox storms, stale epochs, conversion/padding, fixed deadlines,
callback reentrancy, simulated stuck cleanup, panic and late release. They do not
establish real driver-hang recovery. Native minimize/restore, forced surface/device
loss, broader platform coverage and full GPU reference coverage remain open.
The earlier four non-exact compositor captures remain failures. Agent sessions
were unavailable; this implementation has local self-review and tests, not an
independent implementation review. Full web compatibility, audited security and
the requested Chromium performance bound remain unmet.


## String.prototype.concat

The [concat implementation](../tests/conformance/string-concat.md) passes **995
Rust tests** with `--include-ignored` (813 library, none ignored), formatting,
strict all-target Clippy, release compilation, **158 Python checks**, **57 exact
pixel references**, and **15,000 mutation cases** without a caught panic or
invariant failure. Seven Rust groups and three Python groups cover conversion,
UTF-16/resource boundaries, complete corpus integrity and assertion failures.

The new complete **22-source / 44-mode** directory records **42 passed / two
metadata-unsupported** modes. The same frozen before binary records 40 failed,
two passed and two unsupported; its new feature preflight fails (34/56 verify).
Those two initial passes accept a construction exception even when the method is
absent. The new binary verifies all **56 controls**. The two unsupported modes
still require Reflect.construct. The new healthy baseline retains all 44 modes.

All **21 previous profiles / 7,121 modes / 1,744 controls** preserve their sources
and policies. No pass is lost and no control changes. Two existing Symbol failures
now reach missing Date instead of missing concat; all other observations remain
identical, including resource stops. Existing baselines are unchanged. The combined
inventory is **22 profiles / 7,165 modes / 1,800 verified controls**, with 18 healthy
baseline gates and four resource observation profiles. HTML remains 3,868 matched,
two mismatched and six unsupported.

The **28 frozen local probe modes** retain all sources and fingerprints: 26 improve,
while two original metadata probes remain failed because their own use of upstream
verifyProperty deletes the method without restoration. The initial focused Rust
run exposed this fixture error. The unchanged original source now has a regression
check for its TypeError and deletion; a separate corrected metadata probe passes
both modes using `{restore:true}`. These retained failures are not counted as
conformance passes, and no upstream source or helper was changed.

The [public comparison](../tests/conformance/string-concat.json) records exact
cases, controls, policies and hashes. Local evidence is `artifacts/*string-concat*`.
Prior commit `ab5740b` passed all jobs in [CI run 36492738848](https://github.com/Eriskii/ErisBrowser/actions/runs/36492738848).
Quotas remain unchanged. Agent sessions were unavailable, so no independent-agent
review is claimed. No native-window check, browser Vulkan integration, security
audit or Chromium comparison is claimed. The full browser requirements remain unmet.


## Window properties and global bindings

The [binding checkpoint](../tests/conformance/window-global-bindings.md) adds
ordinary Window definitions/accessors, exact UTF-16 keys, private execution
receivers and general global declaration validation. **988 Rust tests** pass
with `--include-ignored` (806 library, none ignored), along with formatting,
strict all-target Clippy, release compilation, **155 Python checks**, **57 exact
pixel references**, and **15,000 mutation cases** with no caught panic or invariant
failure. Seven new Rust groups cover the change. Three older groups initially
expected definitions to be unsupported; they retain the original operations and
now check successful property creation. Unimplemented event-handler deletion
remains an explicit unsupported result.

All **64 frozen self-authored modes** now pass: 32 gains and 32 preserved passes.
Their sources, language modes and fingerprints are unchanged. All **21 upstream
profiles / 7,121 modes / 1,744 controls** retain their inventories and policies.
**13 cases gain passes** (11 compound assignment, two RegExp), with no lost passes
or other observation/control changes. Both healthy improved baselines preserve
all cases. All 17 healthy gates pass; four resource-stopped profiles remain
observations. HTML retains 3,868 matched / two mismatched / six unsupported.

The [public comparison](../tests/conformance/window-global-bindings.json) records
source/release hashes, exact probe outcomes, full profile comparisons and baseline
updates. Local evidence is `artifacts/*global-bindings*`. Prior commit `5e69e4a`
passed all jobs in [CI run 36490270776](https://github.com/Eriskii/ErisBrowser/actions/runs/36490270776).
Quotas are unchanged; accounting remains estimated. Agent sessions were unavailable,
so no independent-agent review is claimed. Full Window/ECMAScript compatibility,
production security and the Chromium target remain unmet. No native-window check
or browser Vulkan integration is claimed for this change.


## Window property order and reflection

Formatting, strict all-target Clippy and release compilation pass. **981 Rust
tests** pass with `--include-ignored` (799 library; none ignored), alongside
**155 Python checks**, **57 exact pixel references** and **15,000 mutation cases**
without a caught panic or invariant failure.

[Window binding reflection](../tests/conformance/window-reflection.md) adds own
membership/enumerability and ordered key snapshots, preserves live `for-in`
deletion/shadowing, and corrects intrinsic binding flags. Six new Rust groups cover
these behaviors and resource refusals. Two older groups initially failed because
they expected Window enumeration to be unsupported. They now check successful
enumeration; the old `finally` source is retained as a success check and unsupported
Document enumeration preserves the host-stop check. Original resource cuts and
looping checks are unchanged.

The **32 frozen self-authored modes** retain all sources and fingerprints:
**26 improve**, two earlier passes remain, and four general Window definition
cases remain unsupported. All **21 upstream profiles / 7,121 modes / 1,744 controls**
retain their sources and policies. Exactly **54 modes improve**: global values 22,
Symbols two, Number statics two, numeric conversion four, numeric parsing eight
and URI functions 16. No passes are lost; all controls verify. Every other
observation, including resource stops, is unchanged.

Four healthy baseline files gain the new passes while preserving their complete
inventories. Numeric parsing and URI remain resource-stopped observations. All
17 existing baseline gates pass. HTML retains **3,868 matched / two mismatched /
six unsupported** modes without change. The
[public comparison](../tests/conformance/window-reflection.json) records all probe
outcomes, profile comparisons, source/release hashes and baseline changes. Local
evidence is `artifacts/*global-reflection*`. Prior commit `6b4d18e` passed all jobs
in [CI run 36488324312](https://github.com/Eriskii/ErisBrowser/actions/runs/36488324312).

Creation serial storage and snapshots use the existing estimated ledger; no quota
increased. Full Window interfaces, general property definitions/accessors and
arbitrary UTF-16 host property storage remain incomplete. Agent sessions were
unavailable, so no independent-agent review is claimed. No native-window, Vulkan
integration or Chromium comparison was run. Full compatibility and production
security remain unverified.

## DOM string conversion and operation receivers

Formatting, strict all-target Clippy and release compilation pass. **975 Rust
tests** pass with `--include-ignored` (793 library; none remain ignored), along
with **155 Python checks**, **57 exact pixel references** and **15,000 mutation
cases** without a caught panic or invariant failure.

Supported [DOM methods](../tests/conformance/dom-string-conversion.md) now use
their actual receiver, validate required arguments, and convert strings before
their DOM effects. Six Rust groups cover method identity/call/apply/bind, abrupt
and reentrant hooks, nullable/Boolean rules, token validation and resource limits.
Two existing private native-call fixtures initially failed because they used old
internal method identifiers. Updating those identifiers retained their original
budgets, input trees, operations and no-mutation assertions; the full suite passes.

All **22 frozen self-authored probe modes** remain in the
[comparison](../tests/conformance/dom-string-conversion.json): **16 improve**, two
prior passes remain, and four unrelated global-reflection modes remain unsupported.
Sources, case fingerprints and language modes are unchanged. The nine DOM sources
also run in both modes as Rust regression fixtures. This is focused implementation
evidence, not a full upstream DOM or Web IDL conformance result.

All **21 existing Test262 profiles / 7,121 modes / 1,744 controls** preserve every
source/case fingerprint, policy and observation, including resource stops. All
controls verify; no passing case is lost. Seventeen healthy baseline gates pass;
four resource-stopped inventories remain observations. No baseline was changed.
HTML retains **3,868 matched / two mismatched / six unsupported** modes, with no
regression or improvement.

The public comparison records the source-input digest, source and release binary
hashes, exact probe outcomes and per-profile comparisons. Local evidence is
`artifacts/*dom-conversion*`. Prior commit `cbbfd22` passed all jobs in
[CI run 36485477521](https://github.com/Eriskii/ErisBrowser/actions/runs/36485477521).

Quotas remain unchanged. DOM storage still replaces lone UTF-16 surrogates, and
full interfaces, live collections, XML name validation and hierarchy semantics
remain incomplete. Agent sessions were unavailable; no independent-agent review
is claimed. No native-window, Vulkan integration or Chromium performance result
is assigned to this change. Full compatibility and production security remain
unverified.

## Symbol identities and property-key integration

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy and release
compilation pass. **969 Rust tests** pass with `--include-ignored` (787 library;
none remain ignored), alongside **155 Python checks**, **57 exact pixel references**
and a **15,000-case** mutation run with no caught panic or invariant failure.

[Symbol support](../tests/conformance/symbols.md) introduces distinct primitive
identities and property keys, boxed/registry/description behavior, computed key
names, descriptors, JSON omission, primitive/tag/instance hooks and key reflection.
Ten focused Rust groups cover both language modes, abrupt effects, mutation order,
resource refusals and callback cleanup. Numeric consumers now reject Symbol
conversion. Symbol descriptions are charged before legacy host/display formatting.
No quota changed. Existing host string conversions and broader protocols still
need work.

The new complete **123-source / 242-mode** Symbol and key-reflection profile
records **166 passed / 6 failed / 70 unsupported**, with all **64 controls** verified.
Sixty-six unsupported modes are excluded by metadata; four stop on class syntax or
host/global reflection. The six failures reach missing String concat/Date, Proxy
and collection-species dependencies. The frozen prior binary ran the same inventory
and policy but failed preflights; its raw report is retained without treating it as
a healthy baseline.

All **20 existing profiles / 6,879 modes / 1,680 controls** retain their source
identities and policies. Exactly **14 modes improve**: eight reduction cases use the
actual Math/JSON tags, and six logical-assignment cases gain Symbol conversion.
There are no lost passes, changed controls or other changed observations, including
resource stops. The new profile brings the total to **7,121 modes / 1,744 controls**.
Seventeen healthy passing-case gates are now enabled; four resource-stopped profiles
remain explicit observations. HTML remains 3,868 matched, two mismatched and six
unsupported modes. The new Symbol baseline and the two improved baselines preserve
all failures and unsupported cases.

The [public comparison](../tests/conformance/symbol-properties.json) and complete
[before](../tests/conformance/test262-symbols-before-properties.json)/
[after](../tests/conformance/test262-symbols-properties.json) Symbol reports retain
provenance. Local evidence is `artifacts/*symbols*`. Prior checkpoint `97ea66c`
passed all jobs in [CI run 36480560165](https://github.com/Eriskii/ErisBrowser/actions/runs/36480560165).

Release SHA-256 values:

| Binary | SHA-256 |
| --- | --- |
| `eris-browser` | `cbe8691a6bc4c5af6edaba8bbf15349a3aedd669a17d5d52e130ff9a8f04aea4` |
| `eris-js` | `4b890d92a4027dc929be92ba366b0cf670f94f18e4bfb28025e365aba6bbab57` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `3335cf7c7732ac618dee11202075241ce00fb8c00dab452b37e33bb13538b381` |

Source-input SHA-256: `857547575004ab839a59028a72d9e709413f2b1ced8cad2d52c2da5bb39dc0c9`.

Agent sessions remain unavailable; no independent-agent review is claimed.
No native-window, Vulkan integration or Chromium performance comparison was run.
Full web compatibility and production security remain unverified.

## Bounded JavaScript grammar continuations

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **959 Rust tests** pass with `--include-ignored`: 777 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 53 pipeline, one declaration-name matrix, nine completion
and 44 confined-worker tests. None remain ignored. All **151 Python checks** and
**57 exact pixel references** pass.

[Grammar continuations](../tests/conformance/parser-continuations.md) replace
supported JavaScript grammar recursion with a parser-owned flat work stack.
Expressions, statements, function/default contexts and template substitutions
retain the shared compile ledger. Obsolete grammar-depth and member/constructor
chain guards are removed after the guarded conversion passed its initial checks.
Source/token/work/storage quotas, active-label and declaration-walk bounds,
logical calls and native-helper guards remain unchanged.

Six new driver test groups cover small-stack source parsing, unchanged upstream
32-IIFE parsing/execution/release, 128-link member/call/constructor chains,
malformed deep input cleanup, refused frame growth and literal dispatch. Requested
128 KiB stacks parse and release 256 nested IIFEs, 512 blocks/functions and the
other documented shapes. The literal test compares 28 sources in four contexts
against the private oracle and parses the unchanged 2,048-element sort source.
Existing grammar/scope/completion, work-cutoff, heap-refusal and flat-destruction
checks remain. Earlier guard-refusal assertions retain their original sources
and now assert acceptance, with larger independent-quota cases added.

The initial complete upstream run exposed two earlier resource stops in the
2,048-element sort test: redundant grammar dispatch exhausted compile work.
Terminated numeric/string literals now finish directly instead of suspending
empty grammar layers. The unchanged source again reaches the same runtime
instruction-limit outcome as before, under the original budgets. Initial logs,
source/binary hashes and reports remain recorded locally.

All **20 Test262 profiles / 6,879 modes / 1,680 controls** preserve source identities
and policies. Exactly two upstream observations improve: both modes of the original
32-nested-IIFE function test now pass. Every other case and all controls are identical.
The function profile reaches **511 passed / 620 unsupported** and now has a healthy
CI baseline. Sixteen healthy gates pass; four resource-stopped profiles remain
nonpassing observations. No old baseline, upstream test, assertion or policy changed.
HTML remains 3,868 matches, two mismatches and six unsupported modes.

All **480 existing depth sources** remain unchanged. All three shapes parse
through the tested depth of 40 and execute through 32 calls, with call 33 stopped
by the logical ceiling. The nested-IIFE shape gains 104 completions and changes
16 parse-resource outcomes to runtime logical-call stops, with no lost completion.
Ordinary/default observations are identical.

The final **15,000-case** mutation run catches no panic or invariant failure.
An earlier 45,000-case run also completed without caught panics before the literal
dispatch correction; the final release was checked again after that change.

Release SHA-256 values:

| Binary | SHA-256 |
| --- | --- |
| `eris-browser` | `dfc989f3cff62c3422bc78972b1bd53f0e2bd74f54a92fe4f141e97d6c32459f` |
| `eris-js` | `af3f3205e0a7f13ade2a9236ca22bf8bfefe20ef947a3c106e48c7f0bb711da7` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `36bef395c7f9b391e5e3ac7d53c622ffa7dd67e832d29a82393fccf744f618aa` |

Source-input SHA-256: `e314cfc86bed205fa1f32ae17320d4dead05055c468d81e94f424afe9ab107fb`.
Local evidence is `artifacts/*parser-continuations*`; the
[public comparison](../tests/conformance/parser-continuations.json),
[full function report](../tests/conformance/test262-functions-parser-continuations.json)
and [depth inventory](../tests/conformance/parser-continuations-depth.json)
preserve provenance. Prior checkpoint `e546e7a` passed every job in
[CI run 36474559666](https://github.com/Eriskii/ErisBrowser/actions/runs/36474559666).

Agent sessions remain unavailable; no independent-agent review is assigned.
Declaration depth, remaining allocation accounting and native bridges need further
work. No native-window, Vulkan or Chromium comparison was run for this change.
Full compatibility, production security and the requested performance threshold
remain unverified.

## Direct flat parser ownership

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **953 Rust tests** pass with `--include-ignored`: 771 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 53 pipeline, one declaration-name matrix, nine completion
and 44 confined-worker tests. None remain ignored. All **151 Python checks**
and **57 exact pixel references** pass.

[Direct flat parsing](../tests/conformance/flat-parser.md) replaces the production
owning AST and second lowering pass. Typed record IDs cover defaults, methods,
function bodies and inline handlers from their initial construction. Record pages
and parser lists share the original compile ledger; one-pass cover grammar moves
names/default IDs into parameters and retains charged tombstones. The old owning
parser remains only as a private frozen test oracle. Ordinary driver source tests
now use the production parser.

Seven new parser groups include **85 grammar sources / 340 mode-context cases**,
144 existing name-diagnostic modes, 240 completion-fixture modes and 144 nesting
guard comparisons. Payloads and early errors match the oracle. Every work cutoff
and sampled heap refusals in five fixtures share the same compile/publication
ledger. Other checks cover default/RegExp/template identity, handler metadata and
partial-unit release. Two record-page groups verify indices, descriptor work,
retained values and precharged refusals. A thread requesting **64 KiB** releases
directly constructed **16,000 unary edges and 8,000 labeled-statement edges** after
successful or refused publication. This is a destruction test, not deeper parsing.

The initial complete library run exposed a compile-work regression in the existing
8,327-declaration Unicode fixture. Paged records alone were insufficient; the root
body now also reserves its ID slots from the already lexed prefix, avoiding
repeated copies. The unchanged fixture passes with the original quotas. Initial
build/test-integration diagnostics and those failed attempts remain in local logs.
Grammar recursion and its depth guards have not changed.

All **20 Test262 profiles / 6,879 modes / 1,680 controls** retain identical case
identities, policies, preflight observations and every case observation. Fifteen
healthy baseline gates pass; five resource-stopped profiles remain nonpassing
observations, with no new baselines. HTML remains at 3,868 matches, two mismatches
and six unsupported modes. All **480 depth observations** are unchanged: ordinary
and default recursion reaches 32 calls, while nested IIFEs still stop in parsing
at depth eleven. The original upstream 32-nested-IIFE source remains unchanged.

The **15,000-case** mutation run catches no panic or invariant failure: 5,000
accepted HTML, 623 accepted/4,377 rejected scripts and 3,347 accepted/1,653 rejected
SVG inputs. Maximum DOM/display-list sizes are 191 nodes/1,173 commands;
17 cases stop within paint limits.

Release SHA-256 values:

| Binary | SHA-256 |
| --- | --- |
| `eris-browser` | `9abb7c64d2a0af29bf671db035416774e9b48bca9706f8c094f192fc0ae49143` |
| `eris-js` | `caf321e9de54b1e24268d6d8b8b9bfbe99e97dd9f1e7320a5d6e07ae1d38590e` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `511ff5a9db04537425b304ed5fe28f4146d64eee2810fc68ece14d100f0473b7` |

Source-input SHA-256: `d7f8d3fea81a4f2d89aabd02aab976961f44c2d044b91aca2806a18fee884cd2`.
Local evidence is `artifacts/*flat-parser*`; the
[public comparison](../tests/conformance/flat-parser.json) and
[depth inventory](../tests/conformance/flat-parser-depth.json) preserve provenance.
The prior activation checkpoint `2c6cf75` passed every job in
[CI run 36470693599](https://github.com/Eriskii/ErisBrowser/actions/runs/36470693599).

Agent sessions remain unavailable; no independent-agent review is assigned.
Grammar continuations, remaining allocation accounting and bounded declaration
traversal still need work. No native-window, Vulkan or Chromium comparison was
run for this parser change. Full compatibility, production security and the
requested performance threshold remain unverified.

## Ordinary activation and default continuations

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **944 Rust tests** pass with `--include-ignored`: 762 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 53 pipeline, one declaration-name matrix, nine completion
and 44 confined-worker tests. None remain ignored. All **151 Python checks**
and **57 exact pixel references** pass.

[Ordinary activations, defaults and bound forwarding](../tests/conformance/activation-frames.md)
now share the expression/statement driver. Queued invocations own their logical
count after a successful reservation; native callback roots retain their external
count and guard. Parameter environments, TDZ, argument mapping, default closures
and body completion preserve their behavior. Bound arguments use one checked,
precharged copy; parameter evaluation drops its actual-value buffer before body
execution. Native helpers and constructors remain guarded bridges.

Fully iterative JavaScript no longer consumes obsolete native-depth charges.
Continuation storage now derives its ceiling from the unchanged 8 MiB allowance
and actual record size, with growth work/storage prepaid. The 32-call, 100,000-work,
96-native-stack-unit and parser constants remain. This changes depth behavior:
the [480-case inventory](../tests/conformance/activation-frames-depth.json) gains
**72 successful runs**, while **32 resource diagnostics** change to the logical
call ceiling. These exact ordinary/default sources now complete **32 calls**
instead of 13/15, with call 33 refused. All parsing and nested-IIFE observations
remain unchanged; the original upstream 32-nested-IIFE source still stops in parsing.

Seven new groups cover strict/sloppy call boundaries, every work cutoff in eight
activation fixtures, storage refusal, bound/preentered ownership, TDZ/closures,
finally identity and native recursion. A thread requesting a **128 KiB native
stack** completes directly constructed 32-call ordinary/default fixtures in both
modes and refuses call 33. Equivalent source parsing is checked separately.
The prior 96-unary and 95/96-block fixtures now complete; 97 blocks stops through
the independent declaration-traversal limit. Exhausted growth preserves queued
frames and the caller's count. Refused charges remain in the cumulative ledger.

All **20 Test262 profiles / 6,879 modes / 1,680 controls** retain identical case
identities, policies and every observation. All controls verify; fifteen healthy
baseline gates pass. Five resource-stopped profiles remain nonpassing observations,
with no new baselines. HTML stays at 3,868 matches, two mismatches and six unsupported
modes. The **15,000-case** mutation smoke run has zero caught panics or invariant
failures: 5,000 accepted HTML, 623 accepted/4,377 rejected scripts and 3,347 accepted/
1,653 rejected SVG inputs. Maximum DOM size is 191 nodes and display-list size
1,173 commands; 17 cases stop within paint limits.

Release SHA-256 values:

| Binary | SHA-256 |
| --- | --- |
| `eris-browser` | `a3c76c013ae8fcab2180a794f7e393cb6574af606b2812ab6673aed7268b9568` |
| `eris-js` | `7dd5587f39f42f26c629ac3f195825b4f171a93c2cade8337c980e6dc50f540c` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `4cb0c8a03832268c1c8ca88f22eb7dc91ba38ee276dd2ef1394f62f5c64aeeef` |

Source-input SHA-256:
`832ffcee3212595a1b67dbb8fa5ef75a15adf444dbf9c87abd05fb3d3aae88aa`.
Local records are `artifacts/*activation-frames*`; the
[public comparison](../tests/conformance/activation-frames.json) preserves provenance
and all profile checks. Initial private-test failures came from an unsupported
String.repeat call in a fixture and an incorrect assumption that refused charges
were absent from the ledger; corrected checks retain exact expected output and
verify actual retained capacity. Parser ownership/accounting and native recursion
remain separate work. Agent sessions remain unavailable, so no independent-agent
review is assigned. No native-window, Vulkan or Chromium performance result is
assigned. Full compatibility, production security and the requested performance
threshold remain unverified.

## Shared statement continuations

Published checkpoint `3564451` passed
[all GitHub CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36467482921),
including all existing conformance gates, Rust 1.88 and the Vulkan probe.

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **937 Rust tests** pass with `--include-ignored`: 755 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 53 pipeline, one declaration-name matrix, nine completion
and 44 confined-worker tests. None remain ignored. All **151 Python checks**
and **57 exact pixel references** pass.

[Statement/list continuations](../tests/conformance/statement-frames.md) share
the expression driver, including declarations, branches, loops, switch and
try/catch/finally. Typed body locators avoid copying lists; for-loop iteration
bindings borrow their declaration names. Ordinary exceptions unwind to try
continuations; resource/unsupported host termination bypasses catch/finally.
Reentrant drives retain their own frame boundaries. Existing depth counters and
execution quotas remain; one additional continuation slot accommodates the
program root list. Function activation and defaults still enter through native
recursion, so this stage establishes no deeper-call acceptance.

Five new test groups check work/storage cutoffs, cleanup/code release, cross-unit
callback exceptions, finally overrides, per-iteration closures and live for-in
mutation. A thread requesting a 128 KiB native stack executes directly constructed
80- and 94-block chains. The 95-block fixture retains the existing combined-depth
refusal; the frame cache reaches its 193-slot bound. These isolate execution from
parsing. Existing 144 declaration-name and 240 scalar completion variants pass.

All **20 Test262 profiles / 6,879 modes / 1,680 controls** retain identical case
identities, policies and every observation. All controls verify; fifteen healthy
baseline gates pass. Five resource-stopped profiles remain nonpassing observations,
with no new baselines. All [480 depth observations](../tests/conformance/statement-frames-depth.json)
are unchanged. HTML stays at 3,868 matches, two mismatches and six unsupported
modes. The **15,000-case** mutation smoke run has zero caught panics or invariant
failures: 5,000 accepted HTML, 623 accepted/4,377 rejected scripts and 3,347 accepted/
1,653 rejected SVG inputs. Maximum DOM size is 191 nodes and display-list size
1,173 commands; 17 cases stop within paint limits.

Release SHA-256 values:

| Binary | SHA-256 |
| --- | --- |
| `eris-browser` | `83f06d277b5c776a082ec42782806ef335b846eaaae6d29d419cb0b74b699639` |
| `eris-js` | `9cbd79075a71145f7f6fb7f7c646a87b64ea0d8c462d6c95fee45f338cdbe706` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `a2fd6339ad68ebd6aa649e4738e4193e00445c7503ecb59673721eeaf73280bc` |

Source-input SHA-256:
`b745b416797170ae95f7d7986ad447d0b7c400e966151895c1f1d56e5ca3a809`.
Local records are `artifacts/*statement-frames*`; the
[public comparison](../tests/conformance/statement-frames.json) preserves provenance
and all profile checks. Parser syntax remains recursive and incompletely
accounted; runtime allocation accounting is also incomplete. Agent sessions
remain unavailable, so no independent-agent review is assigned. No native-window,
Vulkan or Chromium performance result is assigned. Full compatibility, production
security and the requested performance threshold remain unverified.

## Expression and reference continuations

Published checkpoint `57e9a0c` passed
[all GitHub CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36465368973),
including all existing conformance gates, Rust 1.88 and the Vulkan probe.

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **932 Rust tests** pass with `--include-ignored`: 750 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 53 pipeline, one declaration-name matrix, nine completion
and 44 confined-worker tests. None remain ignored. All **151 Python checks**
and **57 exact pixel references** pass.

[Expression/reference frames](../tests/conformance/expression-frames.md) retain
code units, environments and pending values in a checked runtime vector. Reentrant
native callbacks preserve their outer frame boundary. Work/storage refusal drops
only the current drive's frames and restores depth counters without fresh quotas.
Identifier references retain code rather than copying names. Array and argument
payloads are precharged; property allocation failure cannot leave the parallel
array arenas out of sync. Statements, function bodies/defaults and native bridges
remain guarded native recursion. No limits were raised.

Six private test groups cover resource cutoffs, growth/reuse, code release,
callback order and identity, global references and allocation refusal. Directly
constructed 80- and 95-unary chains execute on a thread requesting a 128 KiB native
stack; 96 hits the retained guard. Parsing is checked separately. The
[480-case depth record](../tests/conformance/expression-frames-depth.json) preserves
all previous observations: these ordinary-call sources first stop at 14 calls,
default-initializer sources at 16 and nested IIFEs at parser depth eleven. This
stage establishes no deeper-call acceptance.

All **20 Test262 profiles / 6,879 modes / 1,680 controls** retain identical case
identities, policies and every observation. All controls verify; fifteen healthy
baseline gates pass. Five resource-stopped profiles remain nonpassing observations,
with no new baselines. HTML stays at 3,868 matches, two mismatches and six unsupported
modes. The **15,000-case** mutation smoke run has zero caught panics or invariant
failures: 5,000 accepted HTML, 623 accepted/4,377 rejected scripts and 3,347 accepted/
1,653 rejected SVG inputs. Maximum DOM size is 191 nodes and display-list size
1,173 commands; 17 cases stop within paint limits.

Release SHA-256 values:

| Binary | SHA-256 |
| --- | --- |
| `eris-browser` | `22ce7be21f1c7e761e3122444e68d7d442ed23394db17da636ef67d4444f46bd` |
| `eris-js` | `4c9fbe276d4fd08006b692613513d6e5d3479c969d5ea389439a50e650d80081` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `1670858039f7d22dd3e88c8b2cd7fbb895f75596ca2edc0dd21ebc698fa835ee` |

Source-input SHA-256:
`cbd36365629468ceb6322490c597fc068966802abde992f16488423a1a8dcd23`.
Local records are `artifacts/*expression-frames*`; the
[public comparison](../tests/conformance/expression-frames.json) preserves provenance
and all profile checks. The parser remains recursive and incompletely accounted;
this is not a total memory bound. Agent sessions remain unavailable, so no
independent-agent review is assigned. No native-window, Vulkan or Chromium
performance result is assigned. Full compatibility, production security and the
requested performance threshold remain unverified.

## Flat executable ownership and paged tokens

Published checkpoint `58a0ef7` passed
[all GitHub CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36461625132),
including all existing conformance gates, Rust 1.88 and the Vulkan probe.

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **926 Rust tests** pass with `--include-ignored`: 744 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 53 pipeline, one declaration-name matrix, nine completion
and 44 confined-worker tests. None remain ignored. All **151 Python checks**
and **57 exact pixel references** pass.

[Flat executable units](../tests/conformance/flat-code.md) own expression,
statement and function records connected by typed IDs. Closures/defaults and
cross-script callbacks retain their own unit; inline handlers use the same form.
Iterative lowering shares the remaining compile ledger and releases temporary
parser syntax on success. Exact edge reservations and direct leaf emission remove
redundant growth and worklist visits. Paged tokens avoid relocating the complete
prefix. These changes preserve the existing 8,327-declaration Unicode regression,
which the initial lowering candidate resource-stopped. No quota increases.

Six code groups and three token groups cover structure, lifetimes, precharges,
truncation and boundaries. Directly constructed flat code with 32,000 expression,
8,000 statement and 8,000 function records releases its last handle on a thread
requesting a 64 KiB native stack. This is an ownership check, not deeper source
acceptance or execution. Existing 144 declaration-name and 240 scalar completion
variants pass. The final validation includes an exact reservation for the fixed
intrinsic empty unit; earlier validation artifacts remain separate.

All **20 Test262 profiles / 6,879 modes / 1,680 controls** preserve case identities,
policies and all preflight observations. Every control verifies and every prior
pass remains. The escaped Unicode 5.2 identifier source gains both modes, leaving
**509 passed / 154 unsupported / six compile resource stops** in that profile.
Array sort retains four resource stops; its two 2,048-element stability modes now
stop on instructions instead of allocation. Every other observation is identical.
Fifteen healthy baseline gates pass. Five resource-stopped profiles remain
nonpassing observations, with no new baselines. HTML stays at 3,868 matches,
two mismatches and six unsupported modes.

The **15,000-case** deterministic mutation smoke run reports zero caught panics
or invariant failures: 5,000 accepted HTML, 623 accepted/4,377 rejected scripts,
and 3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size is 191 nodes and
display-list size 1,173 commands; 17 cases stop within paint limits.

Release SHA-256 values:

| Binary | SHA-256 |
| --- | --- |
| `eris-browser` | `7b99a721b25cb0864a6d10a58b28f02c7e40f4b5a052c8e60c1be750dc0b56cf` |
| `eris-js` | `822b017948d135761cc0074dd910b55dba9cb79941c4c30a27eb7d2158f3ba80` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `e27fbacdfb8472823e588c2bc3bbc032a59744b7fcc2f702639d56c9b77332c1` |

Source-input SHA-256:
`abc454be3cea7233cdbd00ba8315fbb650b0020918dbb378aa4a93386e805f77`.
Local records are `artifacts/*flat-code*`; the
[public comparison](../tests/conformance/flat-code.json) retains provenance and
all four changed observations. The complete updated
[identifier](../tests/conformance/test262-identifiers-flat-code.json) and
[sort](../tests/conformance/test262-array-sort-flat-code.json) observations remain
explicitly nonpassing profiles.

Temporary parser syntax and evaluator/callback execution remain recursive.
Parser allocations are not comprehensively accounted, so the compile ledger does
not prove total peak memory. Whole-unit retention can keep otherwise unused code
alive; conservative legacy activation metadata allowances remain. Agent sessions
remain unavailable, so this is local validation without independent-agent review.
No native-window, Vulkan or Chromium performance result is assigned. Full
compatibility, production security and the requested performance threshold remain
unverified.

## Metered declaration-name validation

Published checkpoint `faa7298` passed
[all GitHub CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36458164451),
including all existing conformance gates, Rust 1.88 and the Vulkan probe.

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **917 Rust tests** pass with `--include-ignored`: 735 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 53 pipeline, one declaration-name matrix, nine completion
and 44 confined-worker tests. None remain ignored. All **151 Python checks**
and **57 exact pixel references** pass.

The [implementation and evidence](../tests/conformance/scope-names.md) charge
borrowed declaration-name collection, buffer growth, stable iterative sorting,
string comparisons and named diagnostic construction before the corresponding
work or allocation. Scope/for-header/parameter/catch validation uses the shared
compile ledger; existing limits and no-lexical-scope skips remain. Earliest source
duplicates, diagnostic kinds and sorted first-conflict selection are preserved.
Five private test groups cover ordering, independent oracles, pointer identity and
resource boundaries. A root-authored **72-source / 144-mode** syntax/diagnostic
matrix passes before and after with identical observations.

All **20 Test262 profiles / 6,879 modes / 1,680 controls** preserve every case and
preflight fingerprint, policy and observation. Every control verifies; fifteen
healthy gates pass. Resource-stopped profiles remain nonpassing observations,
with no baseline re-recording. HTML stays at 3,868 matches, two mismatches and
six unsupported modes.

The **15,000-case** deterministic mutation smoke run reports zero caught panics
or invariant failures: 5,000 accepted HTML, 623 accepted/4,377 rejected scripts,
and 3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size is 191 nodes and
display-list size 1,173 commands; 17 cases stop within paint limits.

Release SHA-256 values:

| Binary | SHA-256 |
| --- | --- |
| `eris-browser` | `fc708729306aa5c7d5acb9cdf9944751da183c22d6e23f18eb58b1fd6c998037` |
| `eris-js` | `6cd0a1426298b533c262ff1796ec6fec49bdd52a63ad8d860d8f2e8f72da621c` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `de1d825f74d7f73bf632cbfe0d36992226ac5960bd1763e1a0988cb5ecc7fbe0` |

Source-input SHA-256:
`e43f320ee08598bb1419c120b736e447674393f40fab3d42cad156de5d60a6c0`.
Local records are `artifacts/*scope-names*`; the
[public comparison](../tests/conformance/scope-names.json) retains report/provenance
hashes and the complete local matrix. Sorting precedes duplicate selection, so
resource exhaustion can precede syntax reporting on larger inputs; duplicates
occupy temporary records and accounting does not refund released storage.
Other parser operations and runtime maps remain separate accounting work.
Owned syntax and parser/evaluator execution still have recursive paths. Agent
sessions remain unavailable, so this is local validation without independent-agent
review. No native-window, Vulkan or Chromium performance result is assigned.
Full compatibility, production security and the requested performance threshold
remain unverified.

## Bounded borrowed declaration traversal

Published checkpoint `2715048` passed
[all GitHub CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36455629280),
including all existing conformance gates, Rust 1.88 and the Vulkan probe.

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **911 Rust tests** pass with `--include-ignored`: 730 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 53 pipeline, nine completion and 44 confined-worker tests.
None remain ignored. All **151 Python checks** and **57 exact pixel references**
pass.

The [implementation and evidence](../tests/conformance/scope-walk.md) replace
recursive declaration-name collection and var hoisting with a borrowed preorder
walk. Ninety-six fixed ancestor cursors bound depth independently of list width;
work is charged before advancing, including empty case lists, and errors are
terminal. Switch scope checks no longer recursively clone their bodies. Parsing
records direct lexical declarations and skips unnecessary conflict analysis when
none exist. Owner scope, function boundaries, diagnostic order and declaration
insertion/validation modes are preserved. Existing limits are unchanged.

Four focused groups exercise order/boundaries, 96/97-depth handling, terminal
work failures, empty cases, 16,000-statement width and lexical conflicts. The
nine completion groups also pass. Initial unnecessary analysis scans exhausted
the compile allowance on the preserved 8,327-declaration Unicode 10 identifier
case; eliminating actual redundant scans/set construction restores both modes.
No quota increase or source change was used. Final validation includes the later
empty root-case precharge; preliminary results remain separate local artifacts.

All **20 Test262 profiles / 6,879 modes / 1,680 controls** preserve every case and
preflight fingerprint, policy and observation. Every control verifies; fifteen
healthy gates pass. Resource-stopped profiles remain nonpassing observations,
with no baseline re-recording. HTML stays at 3,868 matches, two mismatches and
six unsupported modes.

The **15,000-case** deterministic mutation smoke run reports zero caught panics
or invariant failures: 5,000 accepted HTML, 623 accepted/4,377 rejected scripts,
and 3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size is 191 nodes and
display-list size 1,173 commands; 17 cases stop within paint limits.

Release SHA-256 values:

| Binary | SHA-256 |
| --- | --- |
| `eris-browser` | `d1ffd7ec0925c7e65d9bd5564c1c6f56a08ad814973798d3d9267b6b4b7d9b27` |
| `eris-js` | `93e5893a39bbdb36398561bb518be6b3649c0a8d64981fcef342528938c117e2` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `ff770b0b0eaf6ea2ae72deabea8e1cb24ca01a5339e316ebb6b05fd759d39ba2` |

Source-input SHA-256:
`d12aeb709649fe291b6a24ef70fdd555b2c58ffca5ed7cda83864efe929b7fe4`.
Local records are `artifacts/*scope-walk*`; the
[public comparison](../tests/conformance/scope-walk.json) retains report/provenance
hashes. Borrowed name-set comparison/storage accounting needs further auditing;
owned syntax and parser/evaluator execution still have recursive paths. Agent
sessions remain unavailable, so this is local validation without independent-agent
review. No native-window, Vulkan or Chromium performance result is assigned.
Full compatibility, production security and the requested performance threshold
remain unverified.

## Statement completion values and runtime stack correction

Published checkpoint `a56597e` passed
[all GitHub CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36453447238),
including all existing conformance gates, Rust 1.88 and the Vulkan probe.

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **907 Rust tests** pass with `--include-ignored`: 726 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 53 pipeline, nine completion and 44 confined-worker tests.
None remain ignored. All **151 Python checks** and **57 exact pixel references**
pass.

The [implementation and evidence](../tests/conformance/statement-completion.md)
distinguish internal empty completions from JavaScript undefined. Lists, labels,
branches, four supported loops, switches and try/catch/finally preserve or replace
values according to their completion rules. Return/throw identity, loop ordering,
lexical environments and uncatchable host termination remain intact. No execution,
allocation or nesting ceiling changes; no dynamic eval implementation is added.

The root-authored direct-API table has **120 sources / 240 strict-sloppy modes**.
A frozen release-library probe at `f4c287c` passes 118; the final probe passes all
240, adding 122 without losses. The public report retains all expected/before/after
values and source/library/executable hashes. Additional Rust checks cover reference
identity, NaN, host stops and nine recursive statement forms. These checks are not
upstream conformance evidence; original eval-based cases remain unsupported.

The initial implementation overflowed the native stack in the existing nested
statement/function regression. Extracting declaration setup before nested execution
and four runtime branch helpers reduced local debug dispatcher size from 15,400
to 7,000 bytes and statement-list size from 7,800 to 1,352 bytes. The original
regression and new recursive forms pass, without increasing stacks or quotas.
The measurements compare initial/final candidates in this increment, are
compiler-specific, and do not prove a portable native-stack bound.

All **20 Test262 profiles / 6,879 modes / 1,680 controls** preserve every case and
preflight fingerprint, policy and observation. Every control verifies; fifteen
healthy gates pass. Existing resource stops remain nonpassing observations, with
no new baseline recording. HTML remains at 3,868 matches, two mismatches and six
unsupported modes. No upstream source or runner-policy changes.

The **15,000-case** deterministic mutation smoke run reports zero caught panics
or invariant failures: 5,000 accepted HTML, 623 accepted/4,377 rejected scripts,
and 3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size is 191 nodes and
display-list size 1,173 commands; 17 cases stop within paint limits.

Release SHA-256 values:

| Binary | SHA-256 |
| --- | --- |
| `eris-browser` | `746d1cb755ce7fa74a32ec8fc1be7b597c47de265c09a2b66c07c8bdd7944a5e` |
| `eris-js` | `760a8821afc3bd83928b01be4caf0d729113eb26c843051265197043f9326459` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `d568d90d7756fe839a2b1588809f1797a611c82ac813a941c109ae0f58bcdfb3` |

Source-input SHA-256:
`0fe716f518fcb4c21499cfdd974479494a4d2121f1fee2057e1c6ddf4880aca2`.
Local records are `artifacts/*completion*`; the
[public report](../tests/conformance/statement-completion.json) retains probe and
comparison provenance. Agent sessions remain unavailable; this is local validation,
not independent-agent review. No native-window, Vulkan or Chromium performance
measurement is assigned to this checkpoint. Full compatibility, production security
and the requested performance threshold remain unverified.

## Ordinary labeled control flow and parser stack correction

Published checkpoint `f4c287c` passed
[all GitHub CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36450662664),
including the labels gate, Rust 1.88 and the Vulkan probe.

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **898 Rust tests** pass with `--include-ignored`: 726 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 53 pipeline and 44 confined-worker tests. None remain
ignored. All **151 Python checks** and **57 exact pixel references** pass.

The [implementation](../tests/conformance/labels.md) resolves labeled statements,
break and continue to numeric targets, retaining loop updates, lexical iteration
bindings, finally overrides, exact exceptions and function boundaries. Consecutive
aliases share one AST target. Active names, copies, lookup work and storage are
bounded. Strict reserved names and invalid targets are syntax errors; newline ASI
and statement-only grammar are preserved. Existing resource ceilings are unchanged.
Nine focused groups and page/worker fixtures cover these behaviors.

A new deep-label debug test initially aborted with a native stack overflow.
Counting the two extra label helper frames was insufficient alone. Extracting
nine keyword parsers reduced the local debug statement-dispatch frame from
40,216 bytes to 6,344 bytes in the final build. The regression and excessive
nesting of eight statement forms now return resource errors. No thread stack
or global quota was enlarged. Compiler-specific sizes and local failure/prologue
logs are evidence for this fix, not a portable stack bound or security proof.

The [complete upstream profile](../tests/conformance/test262-labels.md) retains
**68 sources / 125 modes / 80 preflights**. It moves from **30 passed / 95
unsupported** to **100 passed / 25 unsupported**, with all controls. The 70 gains,
four changed passing-negative diagnostics and four shifts to dynamic-eval
unsupported outcomes are all retained. Fifty-two negative modes pass. The 25
remaining unsupported modes cover metadata prerequisites and dynamic eval;
there are no ordinary failures, resource stops, harness errors, timeouts or
adapter errors in this profile. Recording and a subsequent gate reproduce all
candidate observations, and CI now protects them.

A preliminary release measured 95 passes, one failure and 29 unsupported modes.
The unchanged labeled-let/newline case exposed a Statement-versus-Declaration
ambiguity. Correcting it also handles the block/array variants and rejects class
declarations in statement position. Five further passes result. Both preliminary
reports and validation are retained locally; all full checks above were repeated
on the final frozen source and binary.

All nineteen previous profiles preserve **6,754 case and 1,600 preflight
fingerprints** and policies. Eighteen preserve every observation. The
[URI comparison](../tests/conformance/test262-uri-labels.json) adds two passes
and eight instruction stops from formerly unsupported labels: **210 passed /
24 unsupported / 112 resources**, with all 128 controls. All 104 prior resource
stops remain; no healthy URI baseline is recorded. HTML stays at 3,868 matches,
two mismatches and six unsupported modes.

The **15,000-case** mutation smoke run reports zero caught panics or invariant
failures: 5,000 accepted HTML, 623 accepted/4,377 rejected scripts and
3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size is 191 nodes and
display-list size is 1,173 commands; 17 cases stop within paint limits. The new
seed changes the inventory, so counts are not acceptance-rate comparisons.

Release SHA-256 values:

| Binary | SHA-256 |
| --- | --- |
| `eris-browser` | `8d84259900e9112511c11b92f754801f1e0f8de2b77e3bdf8b61ff85e833637e` |
| `eris-js` | `9e643c885523b75a1e1a991bb3e7c7a9c84601b6adacb1243932f379716c2cc0` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `b643203b66afced4d41636fee61b448c694d75b17c69a8637ef15cda2596d657` |

Source-input SHA-256:
`e34329d60bba0a821277b13b537bf0021285d2d2add3e554160f51e48e684c61`.
Local records are `artifacts/*labels*`. Agent sessions remain unavailable;
this is local validation, not independent-agent review. No native window,
Vulkan or Chromium performance measurement is assigned to this checkpoint.
Full compatibility, production security and the requested performance threshold
remain unverified.

## Ordinary equality conversion

Published checkpoint `52b373f` passed
[all GitHub CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36447803488),
including the equality gate, Rust 1.88 and the Vulkan probe.

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **887 Rust tests** pass with `--include-ignored`: 717 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 52 pipeline and 43 confined-worker tests. None remain
ignored. All **145 Python checks** and **57 exact pixel references** pass.

The [implementation](../tests/conformance/equality.md) adds iterative type
dispatch for == and !=, including ordinary object conversion and Boolean,
Number and String coercion. It fixes false == null while preserving null/undefined
equivalence and ordinary object identity. Strict equality, object/object and
object/nullish pairs do not run conversion hooks. String work and numeric scratch
storage are charged before use; no resource ceiling changes. Seven focused
groups and the page/worker fixtures cover coercion order, exact exceptions,
UTF-16, skipped hooks and resource boundaries. A repeated-getter test was fixed
to declare its redefined property configurable; no runtime relaxation was made.

The [complete equality inventory](../tests/conformance/test262-equality.md)
retains **145 sources / 286 modes / 128 preflights**. It moves from **146 passed /
44 failed / 96 unsupported** to **190 passed / 96 unsupported**, with all 128
controls. The gains are 20 equals and 24 does-not-equals modes; strict-operator
observations are unchanged. Exotic-value metadata and dynamic-eval prerequisites
remain explicit. There are no resource stops, harness errors, timeouts or adapter
errors in this profile. Actual recording and a subsequent gate reproduce every
candidate observation; CI now preserves the gains and complete inventory.

All eighteen previous profiles preserve every observation, all **6,468 case
and 1,472 preflight fingerprints**, and their policies. Existing resource stops
remain nonpassing. HTML stays at 3,868 matches, two mismatches and six unsupported
modes.

The **15,000-case** mutation smoke run reports zero caught panics or invariant
failures: 5,000 accepted HTML, 623 accepted/4,377 rejected scripts and
3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size is 191 nodes and
display-list size is 1,173 commands; 17 cases stop within paint limits. The new
seed changes the inventory, so counts are not acceptance-rate comparisons.

Release SHA-256 values:

| Binary | SHA-256 |
| --- | --- |
| `eris-browser` | `b871833f233728553ebc26170bbfbfc3f4ed48dba7c8ecac37c300448924e65c` |
| `eris-js` | `5663f74effc73282db269e77aeede8ece73795c324713a4646a2e4c73df0dfb8` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `65c696ecf715332851d48771a4a9bef628d61486b9ebb6ef8d3f19fc6fca122c` |

Source-input SHA-256:
`816e4c1ee47a2096de279c92b7ac0b7b55ddf14283ad0590ee92d1141e70f2f4`.
Local records are `artifacts/*equality*`. Agent sessions remain unavailable;
this is local validation, not independent-agent review. No native window,
Vulkan or Chromium performance measurement is assigned to this checkpoint.
Full compatibility, production security and the requested performance threshold
remain unverified.

## Ordinary relational comparisons

Published checkpoint `9adf86a` passed
[all GitHub CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36446347264),
including the comparison gate, Rust 1.88 and the Vulkan probe.

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **878 Rust tests** pass with `--include-ignored`: 710 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 51 pipeline and 42 confined-worker tests. None remain
ignored. All **139 Python checks** and **57 exact pixel references** pass.

The [implementation](../tests/conformance/relational.md) converts both saved
operands in source order, then selects UTF-16 string ordering or numeric
comparison. This fixes boxed strings and objects that produce strings while
preserving live conversion hooks, receivers, abrupt identity and unordered NaN.
Seven focused groups cover conversion, chains, Unicode boundaries and resource
accounting. The page and confined-worker fixtures retain six green then six blue
samples after a click. No resource ceiling changes.

The [complete comparison inventory](../tests/conformance/test262-relational.md)
retains **184 sources / 364 modes / 128 preflights**. It moves from **292 passed /
eight failed / 64 unsupported** to **300 passed / 64 unsupported**, with all
128 controls. The eight gains are boxed-string comparisons in both modes of each
operator. BigInt/Symbol metadata and dynamic-eval prerequisites remain explicit.
There are no resource stops, harness errors, timeouts or adapter errors in this
profile. Actual recording and a subsequent gate reproduce every candidate
observation; CI now protects these passes and the unchanged inventory.

All seventeen previous profiles preserve every observation, all **6,104 case
and 1,344 preflight fingerprints**, and their policies. Existing resource stops
remain nonpassing. HTML stays at 3,868 matches, two mismatches and six unsupported
modes.

The **15,000-case** mutation smoke run reports zero caught panics or invariant
failures: 5,000 accepted HTML, 618 accepted/4,382 rejected scripts and
3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size is 191 nodes and
display-list size is 1,173 commands; 17 cases stop within paint limits. The new
seed changes the inventory, so counts are not acceptance-rate comparisons.

Release SHA-256 values:

| Binary | SHA-256 |
| --- | --- |
| `eris-browser` | `45455784bcc92aa20c46e7c09f17dcb52a4270703df2701745d1116cff30ecbc` |
| `eris-js` | `a1697ac1b5fa8c70b6c8dc2990bc473bcd5680e892b6f9cfbdc8fc5788740157` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `2dd59004ebcc6122242e3528c339e5e20ad960249fbc9815bdec363c2a3448b9` |

Source-input SHA-256:
`6838e6c870859dfd799bfe7204775ad81c61e4bd27ef2c8ca7f996ad429f4e88`.
Local records are `artifacts/*relational*`. Agent sessions remain unavailable;
this is local validation, not independent-agent review. No native window,
Vulkan or Chromium performance measurement is assigned to this checkpoint.
Full compatibility, production security and the requested performance threshold
remain unverified.

## URI encoding and decoding

Published checkpoint `bf0fa0d` passed
[all GitHub CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36444302786),
including the updated global-values gate, Rust 1.88 and the Vulkan probe.

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **869 Rust tests** pass with `--include-ignored`: 703 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 50 pipeline and 41 confined-worker tests. None remain
ignored. All **133 Python checks** and **57 exact pixel references** pass.

The [implementation](../tests/conformance/uri.md) adds all four global URI
functions with ordinary string conversion, stable native identity and standard
metadata. A bounded-stack visitor validates UTF-16/UTF-8 percent transformations,
preserves reserved escape case and raw decoded surrogates, and rejects malformed
encoding with canonical URIError. Both traversal passes, result copy work and
simultaneous Vec/Rc storage are precharged; resource ceilings remain unchanged.
Two codec and seven runtime groups cover boundaries, all isolated surrogates,
metadata, hook order, abrupt identity and resource stops. Page and confined-worker
fixtures preserve six green then six blue samples. A preliminary test cleanup
was changed from unsupported Window descriptor redefinition to assignment;
that separate runtime gap remains.

The [complete URI profile](../tests/conformance/test262-uri.md) retains
**173 sources / 346 modes / 128 preflights**. It moves from **zero passed /
260 failed / 34 unsupported / 52 resources** to **208 passed / 34 unsupported /
104 resources**, with all 128 controls. All 52 old instruction stops persist;
52 former failures now reach the instruction limit in full loops. These are
not passes. Labeled statements, host reflection and Reflect prerequisites
remain explicit. There are no harness errors, timeouts or adapter errors.
An actual baseline-recording attempt returns one, reproduces every candidate
observation and writes no healthy URI baseline. No URI CI gate is added.

All sixteen previous profiles retain **5,758 case and 1,216 preflight
fingerprints** and policies. Fifteen preserve every observation. The
[global-values comparison](../tests/conformance/test262-global-values-uri.json)
adds four passes: **42 passed / six failed / 40 unsupported**, with all 64
controls. Actual recording and a subsequent gate protect the gains, preserving
historical evidence. HTML remains at 3,868 matches, two mismatches and six
unsupported modes.

A root-authored Python codec oracle checks **414 input strings / 3,312 assertions /
104 strict/sloppy batches** across ASCII, scalar boundaries and all Unicode
planes. All candidate batches pass; the old adapter fails every guarded batch.
A wrong-answer control produces Test262Error. Its input digest and local reports
are recorded in the implementation scope. This supplements the complete upstream
inventory and is not independent-agent review.

The **15,000-case** mutation smoke run reports zero caught panics or invariant
failures: 5,000 accepted HTML, 623 accepted/4,377 rejected scripts and
3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size is 191 nodes and
display-list size is 1,173 commands; 17 cases stop within paint limits. The new
seed changes the inventory, so counts are not acceptance-rate comparisons.

Release SHA-256 values:

| Binary | SHA-256 |
| --- | --- |
| `eris-browser` | `632df680fc4a9db8604939d8bc020a1889b186b8729946c46ec0c26cb791ce4d` |
| `eris-js` | `1c7c0853356cc5282833d3ab0ed5deede421cbab3022668e1ba3cb5bcb7f7d06` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `476c97cb80d389651b62723e13abd97afe2f460f35a63ad0c4fd4985e306dc6d` |

Source-input SHA-256:
`9b4cf0cd976c9fe947cbbe8addf0bfc281dc5ec5dcd19a8c7026a7835bdbfe50`.
Local records are `artifacts/*uri*`. Agent sessions remain unavailable; this is
local validation, not independent-agent review. No native window, Vulkan or
Chromium performance measurement is assigned to this checkpoint. Full
compatibility, production security and the requested performance threshold
remain unverified.

## Short-circuit logical assignment

Published checkpoint `dbe8990` passed
[all GitHub CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36442294655),
including the logical-assignment gate, Rust 1.88 and the Vulkan probe.

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **858 Rust tests** pass with `--include-ignored`: 694 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 49 pipeline and 40 confined-worker tests. None remain
ignored. All **127 Python checks** and **57 exact pixel references** pass.

The [implementation](../tests/conformance/logical-assignment.md) recognizes &&=,
||= and ??=, reads a single saved reference and skips RHS/write when appropriate.
Taken paths preserve original receivers, keys, exact right values and abrupt
identity. Anonymous ordinary/arrow functions receive inferred names only for
identifier targets; conversion storage is precharged. Eight focused groups
cover truthiness without hooks, readonly and const skip/take paths, name flags,
right association, early rejection, exhausted-allocation short circuits and
uncatchable callback limits. Page and confined-worker fixtures verify six green
then six blue samples. Resource ceilings remain unchanged.

The [complete profile](../tests/conformance/test262-logical-assignment.md)
retains **78 sources / 132 modes / 104 preflights**. It moves from **18 passed /
66 failed / 48 unsupported** to **72 passed / twelve failed / 48 unsupported**,
adding 54 passes without losses. All 104 controls verify, with no resources,
harness errors, timeouts or adapter errors. Eighteen negative cases retain
passes with specific target/binding diagnostics. Twelve failures now reach
class syntax or missing Symbol; they remain explicit failures because their
metadata does not declare those prerequisites. Every diagnostic change is
retained. Actual baseline recording and a subsequent gate reproduce the full
candidate observations; CI protects the complete baseline.

All fifteen previous profiles retain **5,626 case and 1,112 preflight
fingerprints**, policies and every observation. Existing resource outcomes in
numeric parsing, identifiers, functions and sort remain nonpassing; no healthy
baseline is assigned to them. HTML remains at 3,868 matches, two mismatches
and six unsupported modes.

The **15,000-case** mutation smoke run reports zero caught panics or invariant
failures: 5,000 accepted HTML, 620 accepted/4,380 rejected scripts and
3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size is 191 nodes and
display-list size is 1,173 commands; 17 cases stop within paint limits. The new
seed changes the inventory, so counts are not acceptance-rate comparisons.

Release SHA-256 values:

| Binary | SHA-256 |
| --- | --- |
| `eris-browser` | `112145418bf22b4d9a0e78b9b0cf277677a3edf88538bcc633475266273860bc` |
| `eris-js` | `e9d33d19a55217d159ab6e337e6f4af21a69651a7325b762247505f7c3b5cdf4` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `9e457b906d96c731d00859d9b49979832a7943ad38e73ac22672117198002b6a` |

Source-input SHA-256:
`8c452cdea5acad7f74b0af769ec0686ed0f6eeda2ec67f47464a0230537dfbed`.
Local records are `artifacts/*logical-assignment*`. Agent sessions remain
unavailable; this is local validation, not independent-agent review. No native
window, Vulkan or Chromium performance measurement is assigned to this
checkpoint. Full compatibility, production security and the requested
performance threshold remain unverified.

## Ordinary addition conversion

Published checkpoint `789165b` passed
[all GitHub CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36440734155),
including the addition gate, Rust 1.88 and the Vulkan probe.

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **848 Rust tests** pass with `--include-ignored`: 686 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 48 pipeline and 39 confined-worker tests. None remain
ignored. All **121 Python checks** and **57 exact pixel references** pass.

The [implementation](../tests/conformance/addition.md) converts saved operands
left-to-right using live ordinary hooks before selecting numeric addition or
UTF-16 string concatenation. Boxed values, original receivers, abrupt identity,
left association and compound references retain their effects and order.
Primitive formatting, copying work and simultaneous Vec/Rc storage are charged
before allocation. Seven focused groups include exact resource boundaries,
late result-size rejection after both hooks and uncatchable recursive/looping
hooks. Existing instruction, allocation, string and nesting limits are unchanged.
Page and confined-worker fixtures preserve six green then six blue samples.

The [complete addition profile](../tests/conformance/test262-addition.md) retains
**48 sources / 95 modes / 64 preflights**. It moves from **51 passed / 16 failed /
28 unsupported** to **65 passed / two failed / 28 unsupported**, adding 14 passes
without losses. All 64 controls verify, with no resources, harness errors,
timeouts or adapter errors. Both remaining failures require Date; unsupported
Symbol/BigInt prerequisites and dynamic eval remain explicit. Actual baseline
recording and a subsequent gate reproduce every observation; CI protects the
full baseline without presenting it as all-pass conformance.

All fourteen prior profiles retain **5,531 case and 1,048 preflight fingerprints**.
Every observation in thirteen is identical. The
[compound-assignment comparison](../tests/conformance/test262-compound-assignment-addition.json)
records all ten remaining += failures becoming passes: **606 passed /
180 unsupported**, with 128 controls. Actual recording and gate verification
protect the gains; historical evidence remains linked. Existing resource stops
in numeric parsing, identifiers, functions and sort remain nonpassing and have
no healthy baselines. HTML remains at 3,868 matches, two mismatches and six
unsupported modes.

The **15,000-case** mutation smoke run reports zero caught panics or invariant
failures: 5,000 accepted HTML, 598 accepted/4,402 rejected scripts and
3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size is 191 nodes and
display-list size is 1,173 commands; 17 cases stop within paint limits. The new
seed changes the inventory, so these counts are not acceptance-rate comparisons.

Release SHA-256 values:

| Binary | SHA-256 |
| --- | --- |
| `eris-browser` | `ba2f27941a6f25065d11c898da7d86bbd2bbcfd8ae868cb2f7da2ac17e56c5cf` |
| `eris-js` | `c9e47a8245eaa6e14c60f1ae0874213bdef2b00a6dacb4648c845155fdb136a3` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `c9a60e9432b13016dbd77a126c319522e65a690f99a8514cf63f27761dec6d48` |

Source-input SHA-256:
`dc2e14d6e60eb413384ed88b7518a3207b5a79f1052f18be6549731a637d25a6`.
Local records are `artifacts/*addition*`. Agent sessions remain unavailable;
this is local validation, not independent-agent review. No native window,
Vulkan or Chromium performance measurement is assigned to this checkpoint.
Full compatibility, production security and the requested performance
threshold remain unverified.

## Compound bitwise assignment

Published checkpoint `09fe914` passed
[all GitHub CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36438219734),
including the compound-assignment gate, Rust 1.88 and the Vulkan probe.

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **839 Rust tests** pass with `--include-ignored`: 679 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 47 pipeline and 38 confined-worker tests. None remain
ignored. All **115 Python checks** and **57 exact pixel references** pass.

The [implementation](../tests/conformance/compound-assignment.md) adds lexer/parser
recognition for <<=, >>=, >>>=, &=, ^= and |= and reuses the existing assignment
reference evaluator. Target/key evaluation occurs once, Get precedes the RHS,
operand conversion precedes Put, and abrupt stages preserve identity and prior
effects. Seven focused groups check reference order, live conversion, strict
writes, const/TDZ/missing bindings, integer boundaries, early rejection, shared
resource stops and actual execution of unchanged decimalToHexString.js.
A preliminary stress test's unsupported String.repeat prerequisite was replaced
with direct recursive conversion; runtime limits were not changed. Page and
real confined-worker fixtures preserve six green then six blue samples through
callbacks exercising all six operators.

The [complete profile](../tests/conformance/test262-compound-assignment.md)
retains **454 sources / 786 modes / 128 preflights**. It moves from **290 passed /
328 failed / 168 unsupported** to **596 passed / ten failed / 180 unsupported**,
adding 306 passes without losses. All 128 controls verify, with no resources,
harness errors, timeouts or adapter errors. Twelve previous failures now reach
explicit unsupported eval/host-definition operations. Twenty-four negative
syntax cases preserve their passing status with more specific invalid-target
or strict-binding diagnostics. All transitions are recorded. The ten remaining
failures concern existing boxed/string += conversion. Actual baseline recording
and a subsequent gate reproduce every observation; CI protects the full baseline.

All thirteen earlier profiles retain **4,745 case and 920 preflight fingerprints**.
Every observation in twelve of them is identical. The
[parsing comparison](../tests/conformance/test262-numeric-parsing-compound-assignment.json)
records four former helper parse errors becoming instruction-limit stops in
complete Unicode loops: **164 passed / 46 unsupported / eight resources**,
with all 80 controls. This is helper-loading progress, not new parsing passes;
no healthy parsing baseline is recorded. HTML remains at 3,868 matches, two
mismatches and six unsupported modes.

The **15,000-case** mutation smoke run reports zero caught panics or invariant
failures: 5,000 accepted HTML, 621 accepted/4,379 rejected scripts and
3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size is 191 nodes and
display-list size is 1,173 commands; 17 cases stop within paint limits. The new
seed changes the inventory, so counts are not acceptance-rate comparisons.

Release SHA-256 values:

| Binary | SHA-256 |
| --- | --- |
| `eris-browser` | `0949c4474fcd7ecda1f944df6870ee108229c28996430f055a6b063d39f393a0` |
| `eris-js` | `92710e2646cc80ba2d2839837941e5c6fe62242f6cad772396074bb880805d78` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `6a17607e6de61fadd2ca42c6a26805752435cca94e7df50816df7855d216f056` |

Source-input SHA-256:
`100da85dd37e4aee42d21f6b83afa7ff6459a21e87042cd4b9274a4a9e6f6931`.
Local records are `artifacts/*compound-assignment*`. Agent sessions remain
unavailable; this is local validation, not independent-agent review. No native
window, Vulkan or Chromium performance measurement is assigned to this
checkpoint. Full compatibility, production security and the requested
performance threshold remain unverified.

## Numeric parsing conversion and aliases

Published checkpoint `479e6b7` passed
[all GitHub CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36436404985),
including the Number alias regression gate, Rust 1.88 and the Vulkan probe.

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **830 Rust tests** pass with `--include-ignored`: 672 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 46 pipeline and 37 confined-worker tests. None remain
ignored. All **109 Python checks** and **57 exact pixel references** pass.

The [implementation](../tests/conformance/numeric-parsing.md) gives parseInt and
parseFloat ordinary string-hint conversion, with parseInt's radix converted
afterward. Live hooks, receiver identity, argument effects and abrupt completion
are preserved. Number.parseInt and Number.parseFloat share the global intrinsic
objects; standard metadata and independent mutable bindings preserve aliases.
Dispatch avoids scanning ignored values. Shared conversion keys and worst-case
UTF-8 scratch are precharged; all resource ceilings are unchanged.

Seven focused groups cover these semantics, prefix grammar, radix wrapping,
signed zero, rounding and shared resource termination. An initially incorrect
private-test expectation that failed charges were refunded was corrected; the
runtime's cumulative ledger was preserved. Page and actual confined-worker
fixtures retain six green then six blue pixel samples through a callback using
saved aliases after both global and Number-property replacement.

The new [complete parsing profile](../tests/conformance/test262-numeric-parsing.md)
retains **109 sources / 218 modes / 80 preflights**. It moves from **142 passed /
20 failed / 48 unsupported / four harness errors / four resources** to **164
passed / 46 unsupported / four harness errors / four resources**. All 80 controls
verify. Twenty gains resolve failures and two resolve parseInt's own length
reflection. Every prior pass is retained. The unchanged hexadecimal helper uses
unsupported >>>= syntax; full nested radix loops retain instruction-limit stops.
An actual baseline-recording attempt reproduces the candidate and refuses to
create a baseline. This profile remains a complete local observation.

The [Number comparison](../tests/conformance/test262-number-statics-numeric-parsing.json)
adds four alias passes: **248 passed / 92 unsupported**, with all 104 controls.
Actual recording and a subsequent gate preserve identical results; the existing
Number CI gate protects the gains. All twelve prior profiles retain **4,527 case
and 840 preflight fingerprints**. Every result in the other eleven profiles is
identical, including all prior resources. HTML remains at 3,868 matches, two
mismatches and six unsupported modes.

A root-authored supplemental matrix checks **1,050 signed inputs** across radices
2–36 and widths 1–10. Python integer arithmetic supplies expectations below 2^53;
direct calls and coercing aliases pass **4,200 assertions / 132 bounded batches**
in both modes. The frozen before adapter fails at the guard in every batch;
a wrong candidate assertion throws Test262Error. These cases do not replace
upstream loops or remove their resource outcomes. Input SHA-256:
`0b09024529f5d2cf8488e9741faccee6ef103e2a56f15ba10c844a1a89574073`.

The **15,000-case** deterministic smoke run reports zero caught panics or
invariant failures: 5,000 accepted HTML, 630 accepted/4,370 rejected scripts and
3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size is 191 nodes and
display-list size is 1,173 commands; 17 cases stop within paint limits. The new
seed changes the inventory; counts are not an acceptance-rate comparison.

Release SHA-256 values:

| Binary | SHA-256 |
| --- | --- |
| `eris-browser` | `d2dd4c8ac2526202e35bef4dc1672d28fc1568e9f9887e1104fc6c136806984b` |
| `eris-js` | `528693af5794bc16e18b956ffe781cbab91b335615c82ea773eca6fe2b13c126` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `dfdfde7d066e0c411f2523aa3dc16d2adb5a279e9cd36c00b434b05aad3b4aab` |

Source-input SHA-256:
`4e98b1e45cd79f3b964d213cfc6c2bbfd012977d7243e9a1e9fe69bfd144a58c`.
Local records are `artifacts/*numeric-parsing*`; matrix source is retained in
session scratch storage. Agent sessions remain unavailable, so this is local
validation, not independent-agent review. No native-window, Vulkan or Chromium
performance measurement is assigned to this checkpoint. Full compatibility,
production security and the requested performance threshold remain unverified.

## Ordinary numeric conversion

Published checkpoint `cf28fc4` passed
[all GitHub CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36434500199),
including the new conversion regression gate, Rust 1.88 and the Vulkan probe.

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **821 Rust tests** pass with `--include-ignored`: 665 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 45 pipeline and 36 confined-worker tests. None remain
ignored. All **103 Python checks** and **57 exact pixel references** pass.

The [implementation](../tests/conformance/numeric-conversion.md) routes Number,
global isFinite and global isNaN through ordinary numeric conversion. Live
valueOf/toString lookup preserves the original receiver, hook mutations,
exceptions and prior effects. Omitted Number arguments remain distinct from
explicit undefined. Native and bound construction box the converted result;
saved functions survive global replacement. Static Number predicates retain
non-coercing behavior.

Eight focused groups check conversion order, primitives, UTF-16 grammar,
boxing/aliases and resource accounting. Private calls ignore prebuilt large
extra values at the heap ceiling; strings returned from getters consume scratch
storage before conversion, and failed conversion unwinds call/stack guards.
Recursive or looping hooks cannot catch resource termination. Array-index
recognition is allocation-free with canonical boundary checks and 30,000
comparisons against the previous decoder's definition. No resource ceiling
was increased. The Page and actual confined-worker fixtures preserve six green
then six blue pixel samples, including saved conversions inside a click callback.

The new [complete global profile](../tests/conformance/test262-numeric-conversion.md)
retains **30 sources / 60 modes / 80 preflights**. It moves from **16 passed /
eight failed / 36 unsupported** to **24 passed / 36 unsupported**, with all
80 assertion controls verified. Initial conversion-guard failures are retained.
The 36 unsupported modes include 32 unchanged prerequisite exclusions and four
runtime global-reflection outcomes. There are no resources, timeouts or adapter
errors. Actual baseline recording and a subsequent gate reproduce the candidate
exactly; CI now runs this complete regression gate.

The existing Number inventory gains ten constructor-conversion passes:
**244 passed / four failed / 92 unsupported**, with all 104 preflights.
Its [full comparison](../tests/conformance/test262-number-statics-numeric-conversion.json)
retains all prior outcomes; the updated baseline protects the gains. Missing
parseInt/parseFloat aliases account for the remaining four failures. All eleven
earlier profiles retain **4,467 case and 760 preflight fingerprints**. Every
observation in the other ten profiles is identical, including the 14 retained
function/sort/identifier resource stops. HTML remains at 3,868 matches, two
mismatches and six unsupported modes.

The **15,000-case** mutation smoke run reports zero caught panics or invariant
failures: 5,000 accepted HTML, 611 accepted/4,389 rejected scripts and
3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size is 191 nodes and
display-list size is 1,173 commands; 17 cases stop within paint limits. A new
conversion seed changes the inventory, so counts are not acceptance-rate comparisons.

Release SHA-256 values:

| Binary | SHA-256 |
| --- | --- |
| `eris-browser` | `6d516838ae17232e01cbc37d2bc59369c23159727467b26124da4c944780d8a3` |
| `eris-js` | `b3b0964931661f63d570d6b3bbb52061a9054fe00e4ec8a1025f624326c95d12` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `23d94649dcdaeae35a8c0b21a9f3bb6d4b6fb69c00e3bc077b93f916a1e50876` |

Source-input SHA-256:
`41577621af344b5f9301064d69abbf7a65dcdf4f46de8a5d1524e9e03734004d`.
Local records are `artifacts/*numeric-conversion*`. Agent sessions were unavailable;
these are local implementation and validation results, not independent-agent
review. No native-window, Vulkan or Chromium performance measurement is assigned
to this checkpoint. Full web compatibility, production security and the requested
performance threshold remain unverified.

## Number constants and non-coercing predicates

Published checkpoint `a2b22a1` passed
[all GitHub CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36431356192),
including both numeric regression gates, Rust 1.88 and the Vulkan probe.

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **811 Rust tests** pass with `--include-ignored`: 657 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 44 pipeline and 35 confined-worker tests. None remain
ignored. All **97 Python checks** and **57 exact pixel references** pass.

The [Number implementation](../tests/conformance/number-statics.md) adds five
constants and four static predicates, preserving the three existing constants.
MIN_VALUE is the smallest positive binary64 subnormal; integer classification
handles the actual rounded value without an integer cast. Predicates borrow
their first argument and do not coerce objects, inspect ignored arguments or
allocate result storage. New bootstrap metadata is precharged; shared work,
heap and stack limits remain unchanged.

Ten focused script groups check exact bits, descriptors, strict/sloppy writes,
boundaries, noncoercion, aliases, argument effects and accounting. Private tests
call the actual native path at the heap ceiling with prebuilt large string/array
arguments; the same fixed work suffices and failure cleans call/stack guards.
Page and real confined-worker fixtures retain six green then six blue pixel
samples, including native aliases called after method replacement and restoration.

The complete [Number inventory](../tests/conformance/test262-number-statics.md)
retains **170 sources / 340 variants / 104 preflights**. It moves from **158
passed / 90 failed / 92 unsupported** to **234 passed / 14 failed / 92
unsupported**, adding 76 passes without losses. Every preflight verifies and
there are no resources, timeouts or adapter errors. Ten remaining failure modes
exercise Number constructor conversion; four exercise parsing aliases. The
unsupported inventory retains 90 metadata exclusions and two host-reflection
outcomes. Actual baseline recording and a subsequent gate preserve identical
observations. CI checks the new complete regression baseline, including its
nonpassing inventory.

All ten prior profiles retain **4,127 case and 656 preflight fingerprints**.
The [reduction comparison](../tests/conformance/test262-array-reduce-number-statics.json)
adds two passes for the unchanged near-safe-integer source in strict and sloppy
modes: **848 passed / 16 failed / 170 unsupported**, with all 128 preflights.
Its updated baseline protects both gains. Every other earlier profile preserves
all passing cases and reported outcomes, including the retained function, sort
and identifier resource stops. HTML retains 3,868 matches, two mismatches and
six unsupported modes.

A separate local check derives expected classification from binary64 exponent
and significand fields, rather than using floating-point truncation. It maps
**2,300 bit patterns** to numeric literals, including 2,048 deterministic random
patterns and targeted boundaries. NaN payloads map to the same JavaScript NaN.
The adapter passes **18,400 predicate assertions** across both modes in 144
bounded batches; a deliberately wrong control throws Test262Error. The frozen
before adapter fails all 144 batches at method-availability checks. These are
sampled local checks, not exhaustive binary64 or independent-agent review.
Inputs SHA-256:
`d2b60045ddcefe6c3f4f3340767ac72145c1bbfc473b6e9f98ac53a9a389287e`.

The **15,000-case** mutation smoke run reports zero caught panics or invariant
failures: 5,000 accepted HTML, 609 accepted/4,391 rejected scripts and
3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size is 191 nodes and
display-list size is 1,173 commands; 17 cases stop within paint limits. The new
seed changes the inventory; these counts are not an acceptance-rate comparison.

Release SHA-256 values:

| Binary | SHA-256 |
|---|---|
| `eris-browser` | `5a1b85733ffdd20cdd50860278efa8baafe257765d31d9718e2f7358f2fde3ad` |
| `eris-js` | `f6ee32dffda7d56c3197cac56586cb9442134f141af1ed69528baf15d8e23675` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `29182360b91d9e31870ee50a8766143f4c38064852fa25459e1bfab07a64a233` |

Source-input SHA-256:
`f9f8fbdd7b40a1a276b7eb493113639e9bd9478913571e7543e4141b8ed324a5`.
Local records are `artifacts/*number-statics*`; the bit-field oracle script is
retained in session scratch storage. No native-window, GPU or Chromium
performance measurement is assigned to this checkpoint. Full web compatibility,
production security and the requested performance threshold remain unverified.

## Streaming array reductions

Published checkpoint `58c027a` passed
[all GitHub CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36420373943),
including the new reduction gate, Rust 1.88 and the Vulkan probe.

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **799 Rust tests** pass with `--include-ignored`: 647 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 43 pipeline and 34 confined-worker tests. None remain
ignored. All **91 Python checks** and **57 exact pixel references** pass.

The [reduction implementation](../tests/conformance/array-reduce.md) adds
reduce/reduceRight with one captured ToLength, live sparse/inherited property
reads, omitted-initial distinction and four callback arguments. It streams over
the full safe-integer logical range with a fixed stack key formatter. Separate
presence and value walks precharge work and temporary storage at each prototype
edge; callbacks share the unchanged work, allocation and stack limits. Fourteen
new script groups include conversion order, boxed strings, actual-callee this,
mutation, exceptions, reentrancy and accounting boundaries.

Direct Page and real confined-worker fixtures preserve six green samples and
six blue samples after a retained callback. They check callback direction,
captured length, live values, abrupt effects and object accumulator identity.
Worker pixels equal the direct output.

The complete [paired upstream inventory](../tests/conformance/test262-array-reduce.md)
retains **520 sources / 1,034 variants / 128 preflights**. Results move from
**84 passed / 780 failed / 170 unsupported** to **846 passed / 18 failed /
170 unsupported**: 762 gains and no lost passes. All 128 preflights verify;
there are no resource stops, timeouts or adapter errors. The initial 84 raw
passes are retained with their missing-method exception caveat, rather than
counted as gains. Remaining failures expose Date, Math/JSON receiver tags and
Number.MAX_SAFE_INTEGER prerequisites. The full reports preserve all unchanged
identities and every nonpassing outcome.

Baseline recording and a subsequent CLI gate both succeed with identical
outcomes, zero regressions and zero additional passes. CI now checks this
separate regression baseline. It retains all 1,034 statuses, including every
failure and unsupported case; a plain conformance run still fails.

Independent frozen-release review matches **370 unchanged outcomes**: 312
normal completions, 20 explicit unsupported controls, 16 intended missing-global
ReferenceErrors, two deliberately failed assertion controls and 20 uncatchable
resources. Three bounded private groups pass, covering failure after getter
effects, exact key/callback allocation boundaries, prototype cycles and
per-edge scratch accounting, shared work and recursion. Static delta review
found no concrete scoped defect.

All nine prior JavaScript profiles preserve their **3,093 case identities and
528 preflight identities**, with no prior passing case lost. The
[full sort comparison](../tests/conformance/test262-array-sort-reduce.json)
gains four modes: the unchanged 5- and 11-element stability sources now pass
in both modes. Sort totals are **57 passed / 46 unsupported / four resources**.
The 513-element source now reaches the runtime instruction limit instead of a
missing-method failure; the two 2,048-element allocation stops are unchanged.
Functions remain 509 passed / 620 unsupported / two parser resources, and
identifiers remain 507 passed / 154 unsupported / eight compiler resources,
with unchanged diagnostics. The six existing healthy JavaScript gates and HTML
gate preserve all results; HTML retains 3,868 matches, two mismatches and six
unsupported modes. No sort, function or identifier baseline is invented.

The **15,000-case** mutation smoke run reports zero caught panics or invariant
failures: 5,000 accepted HTML, 615 accepted/4,385 rejected scripts and
3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size is 191 nodes and
display-list size is 1,173 commands; 17 cases stop within paint limits. The
added reduction seed changes the inventory, so these counts are not an
acceptance-rate comparison.

Release SHA-256 values:

| Binary | SHA-256 |
|---|---|
| `eris-browser` | `b7bac529a1fb2739c806d955ad186aa2894a3cc40b6c6873be334bbab745dceb` |
| `eris-js` | `2b0e59686ff8fb8db409c8a54b8f92df592f00668fcd9ff00da2de0b3585705a` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `01f92eac3d21d87bdfd9e2bed7b892980d040e3266e8b11d7108a0aa8d61f4f1` |

Source-input SHA-256:
`be187dd3a00025c8461dc08013ca49696231b350d5b4490ea8166c15de83a79f`.
Local logs and reports use `artifacts/*-reduce-final*`; independent review
records remain in session scratch storage. No native-window run, GPU execution
or Chromium performance comparison is assigned to this checkpoint. The Vulkan
docket incorporates a reviewed upload/ownership contract; its browser backend
remains unimplemented. These results do not establish full web compatibility
or production security.

## Unicode identifiers and bounded compilation

Published checkpoint `9d5add8` passed
[all GitHub CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36418501821),
including Rust 1.88, the offline Unicode check and the Vulkan probe.

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **783 Rust tests** pass with `--include-ignored`: 633 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 42 pipeline and 33 confined-worker tests. None remain
ignored. All **85 Python checks**, offline table verification and **57 exact
pixel references** pass.

The [identifier implementation](../tests/conformance/identifiers.md) decodes
valid Unicode escapes, distinguishes raw grammar terminals from decoded names,
and uses exact pinned Unicode 18.0.0 ID properties. It preserves combining marks,
supplementary characters and distinct normalization forms while rejecting
invalid start/continuation characters, surrogate escapes and escaped reserved
bindings. Numeric adjacency, RegExp flag scanning and ECMAScript whitespace have
their separate rules. Sixteen script groups cover these paths, compiler limits,
diagnostic offsets, parse atomicity and retained callbacks after source disposal.

The [pinned data and generator](../tests/conformance/unicode-identifiers-data.md)
retain official bytes, license, version evidence and hashes. A deterministic
two-level packed table has 4,352 byte indices and 132 unique 64-byte pages:
**12,800 bytes** of static payload. Membership uses two bounded table reads and
fixed shifts/masks. Independent parsing of source records and generated numeric
literals matches all 1,114,112 code-point positions; the compiled predicate test
checks every scalar. Invalid indices and changed membership bits are rejected
by the independent corruption controls.

Initial scanning and all suffix rescans share the existing work/allocation
ledger. No source, token, nesting, work or heap ceiling increased. Actual token
storage shrank from **120 to 48 bytes** on the tested target by sharing rare
provisional diagnostics. Their allocation is precharged and offset adjustment
requires unique ownership. The parser borrows source during compilation and
retains owned syntax/string data, removing a real whole-source allocation/copy.
Raw first-character membership is reused instead of searched twice.

Direct Page and real confined-worker fixtures preserve six green samples and
six blue samples after an event callback changes retained Unicode bindings.
Worker pixels equal direct output. Separate Page checks reject invalid scripts
before any effects and allow later valid scripts to execute.

The complete [identifier/whitespace inventory](../tests/conformance/test262-identifiers.md)
retains **335 sources / 669 variants / 88 preflights**. It moves from **377
passed / 138 failed / 154 unsupported** to **507 passed / 154 unsupported /
eight resources**: 130 new passes, no lost passes and all 88 preflights verified.
The remaining resources are both modes of four large escaped-Unicode sources.
The runner refuses a healthy baseline; no passing conformance gate was added.
Syntax-negative preflights also require a separately executed positive lexer
control in the same mode, preserving raw results even when that prerequisite
fails.

The [preliminary report](../tests/conformance/test262-identifiers-preliminary.json)
is retained: it had 483 passes and 32 resources, including 16 prior passes that
regressed to compile limits. All 16 are restored by the real work/storage
reductions above, with the same corpus and assertion contract. One private
20,000-character identifier sample now fits the unchanged limits; its exact
input remains a positive control, accompanied by a larger 50,001-character
resource control. No upstream or independent-matrix expectation changed.

Independent frozen-release review matches **435 unchanged outcomes**: 175
normal completions, 229 intended parse SyntaxErrors, 25 unsupported controls
and six resources. Seven private groups pass under 512 MiB address-space,
three-second CPU and eight-second wall limits, including the real exhaustive
scalar test and largest raw Unicode file in both modes. Four resource diagnostics
change as different existing limits become the first exhausted resource; all
remain parse-phase resource outcomes. Static delta review found no concrete
scoped defect. Original and optimized sources, binaries and review records are
retained separately in session scratch storage.

All prior passing conformance cases remain passing. The
[function inventory](../tests/conformance/test262-functions-identifiers.json)
adds six passes, reaching **509 passed / 620 unsupported / two parser resources**
with 48 verified preflights. The
[sort comparison](../tests/conformance/test262-array-sort-identifiers.json)
retains **53 passed / six failed / 46 unsupported / two resources**, with all
80 preflights. Its two large stability cases now stop at runtime allocation
rather than instruction limits; exact diagnostics are retained. HTML retains
3,868 matching trees, two mismatches and six unsupported modes. String/JSON,
RegExp, templates, rest, prototype membership and global-value gates retain
their prior results and policies. All eight earlier profiles preserve their
2,424 case and 440 preflight identities.

The **15,000-case** mutation smoke run reports zero caught panics or invariant
failures: 5,000 accepted HTML, 627 accepted/4,373 rejected scripts and
3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size is 191 nodes and
display-list size is 1,173 commands; 17 cases stop within paint limits. The new
identifier seed changes the inventory; these counts are not an acceptance-rate
comparison.

Release SHA-256 values:

| Binary | SHA-256 |
|---|---|
| `eris-browser` | `8b4a850f591997e3e4a716d821b89e1980e0c87856fe673b4edd1aa7b684c943` |
| `eris-js` | `b7f77cbbc11f14297b7b34716668a10c0522be62562a33a07756dcc89f17d12f` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `d909fc7cd0b21c52b6d3afcbdb4759a563381b66f30667bb32b27cf1be4a8eef` |

Source-input digest:
`aa1d611fc199bdff6eab451307c2166ca0fe71ab3b8c39dd1bf0182ca5431cb7`.
Session logs use `identifiers-final` under `artifacts/`. Classes/modules,
generators/async execution, labels, eval, full goal validation and Unicode
RegExp capture names remain incomplete. No Chromium performance result or new
native-window measurement is assigned to this increment. Full compatibility,
audited security and the requested performance threshold remain unfulfilled;
Vulkan browser integration remains on the development docket.

## Stable array sorting

The runtime checkpoint `eee6586` exposed a race in the existing Python process
cleanup test: Linux procfs returned ESRCH while a terminated child disappeared
during a read. Test-only follow-up `d678745` accepts that specific disappearance
alongside ENOENT, while still failing for live descendants and unrelated errors.
All 68 Python checks and 20 repetitions of the process-group cleanup check pass.
The follow-up passed [all GitHub CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36415370021),
including Rust 1.88 and the Vulkan probe. No browser runtime changed in that fix.

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **762 Rust tests** pass with `--include-ignored`: 615 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 40 pipeline and 32 confined-worker tests. None remain
ignored. All **68 Python checks** and **57 exact pixel references** pass.

The [custom sort implementation](../tests/conformance/array-sort.md) collects
live indexed values, performs stable iterative comparison, then applies strict
ordered writes and tail deletions. Thirteen new script groups cover generic
receivers, inherited indices, holes versus undefined, UTF-16 default order,
comparator conversion/receivers, abrupt completion, reentrant mutation and
resource accounting. Collection and merge buffers, index keys and callback
arguments are charged before allocation. Earlier callback effects and successful
write-back operations survive later exceptions or resource failure.

Direct Page and real confined-worker fixtures produce six green samples and
six blue samples after an event callback sorts the retained records by descending
rank. They check metadata, UTF-16 ordering, stable ties, sparse entries, inherited
ordinary-object values, comparator exceptions and partial strict write failure.
Worker pixels match the direct Page output, with no script diagnostics.

Independent frozen-release review matches **170 unchanged expected outcomes**:
148 normal completions, ten explicitly unsupported controls and twelve
uncatchable resource stops. Both private scratch/prior-effect and work/heap/
recursion groups pass under 512 MiB, three-second CPU and eight-second wall
limits. Before/candidate source, assertion-helper, mode and expectation
fingerprints match, and all recorded source/binary hashes remain frozen.
Static review found no concrete new defect in the collection, comparison,
callback, allocation or copy-back paths. Receiver-mutating comparator probes
are identified as engine invariants where the specification leaves ordering
implementation-defined.

The complete [pinned sort directory](../tests/conformance/test262-array-sort.md)
retains all 54 sources, 107 modes and six unchanged harness files. It moves
from **zero passed / 61 failed / 46 unsupported** to **53 passed / six failed /
46 unsupported / two resource stops**, with all **80 preflights verified**.
All 107 case and 80 corrected preflight fingerprints, manifest and policy match
between frozen adapters. The 46 unsupported observations retain their exact
diagnostics. The six failures now reach missing reduce calls in the 5-, 11-
and 513-element stability files. Both modes of the unchanged 2,048-element
file reach the instruction limit; those failed-to-resource transitions are
explicit in the [full report](../tests/conformance/test262-array-sort-latest.json).
They are not passing tests. A requested healthy baseline was refused because
resource outcomes remain, and no sort baseline or passing CI gate was added.

One new metadata preflight initially misused the unchanged property helper:
it checked a configurable property and then read it after the helper deleted it.
Only the new paired setup gained the helper's documented restore option.
Both frozen adapters were rerun with the same corrected contract; all earlier
profiles stayed unchanged. The
[correction record](../tests/conformance/test262-array-sort-preflight-correction.json)
and both preliminary full reports retain the four changed preflight fingerprints
and original setup failures. Full Python and prior-profile checks were rerun
after this tooling correction; no engine rebuild or changed expectation was
needed for the independent semantic matrix.

The complete functions inventory improves to **503 passed / 620 unsupported /
six failed / two resource stops**. Both modes of the unchanged closure case
`statements/function/S13.2.1_A5_T1.js` newly pass, with no lost passes and all
48 preflights verified. The
[full comparison](../tests/conformance/test262-functions-sort.json) preserves
all 1,131 identities and the policy. Six identifier-escape failures and both
original nested-function parser stops remain; there is still no healthy
functions baseline.

Earlier regression gates retain their outcomes: HTML **3,868 matched /
two mismatched / six unsupported**, String/JSON **536 passed / 116 unsupported**,
RegExp **250 / 40**, templates **82 / 32**, rest parameters **16 / six**,
prototype membership **ten / ten**, and global values **38 passed / ten failed /
40 unsupported**. All earlier inventories, policies, assertions and baselines
remain unchanged.

The **15,000-case** mutation smoke run has zero caught panics or invariant
failures: 5,000 accepted HTML, 627 accepted/4,373 rejected scripts, and
3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size is 191 nodes and
display-list size is 1,173 commands; 17 cases stop within paint limits. The new
sort seed changes the inventory, so acceptance counts are not a comparison
against the previous run.

Release SHA-256 values:

| Binary | SHA-256 |
|---|---|
| `eris-browser` | `580e75688c07d916a3caf326731ed788becad6ef6b442fc3cffaa87290e82fcc` |
| `eris-js` | `c039eb6be9360342f9cec2fd8da08cd0efc4245a04370bca0c114e1ff812825c` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `de77d3948aa0486c326d92aaab251c32dd21571e209b18c82543b02c28ea0ace` |

The final source-input digest is
`dce62370c4dbfa806975d8c207bc38ad955997f89d653357848bc2e1b36a148c`.
Session logs use `sort-final` under `artifacts/`; independent review records
remain session scratch files. General Array exotic/host semantics, reduce,
Proxy, typed arrays and larger workloads remain incomplete. No new native-window
or Chromium performance measurement is assigned to this increment. Full web
compatibility and independently audited security remain unverified.

## Global value properties

Published checkpoint `cbe6efa` passed
[all GitHub CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36412306983),
including the new global-value regression gate, Rust 1.88 and the Vulkan probe.

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **747 Rust tests** pass with `--include-ignored`: 602 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 39 pipeline and 31 confined-worker tests. None remain
ignored. All **63 Python checks** and **57 exact pixel references** pass.

The [global-value implementation](../tests/conformance/global-values.md) gives
globalThis, undefined, NaN and Infinity their actual initial descriptor flags
and shares the existing Window.self property machinery through an exact
five-name selector. Eleven new script groups cover immutable writes/deletes,
SameValue descriptor compatibility, reentrant accessors, inherited and captured
references, lexical separation, private Window/event identity and allocation/work
limits. Tracked global functions are checked before declaration insertion;
reverse-order checks and the last declaration of each name avoid partial
bindings and unnecessary function allocations.

Direct Page and real confined-worker fixtures produce six green samples and
six blue samples after a callback replaces the public property and a separate
lexical binding. Initial descriptors, strict immutable writes, deletion,
restoration, sloppy/strict receiver rules and Window events are checked before
the pixels are accepted. Worker pixels match the direct Page output.

Independent frozen-release review matches **178 expected outcomes**: 138 normal
completions, 18 intrinsic declaration errors, 14 explicitly unsupported controls
and eight uncatchable resource stops. All eight strict/sloppy continuation pages
match exact final text and the single intended declaration-error diagnostic.
They confirm a rejected function declaration leaves no earlier lexical, var or
function binding behind, permits later declarations, and preserves the locked
property and private Window identity. The review uses unchanged expectations,
upstream assertion helpers and before/candidate source and binary fingerprints.
Four compiled private resource/declaration groups also pass under 512 MiB,
three-second CPU and eight-second wall limits. Static review found no concrete
new defect in the changed descriptor, declaration, callback or allocation paths.

The complete new [global-value profile](../tests/conformance/test262-global-values.md)
retains 49 upstream files and all 88 required modes. It improves from
**32 passed / 16 failed / 40 unsupported** to **38 passed / ten failed /
40 unsupported**, with all **64 preflights verified**. The six gains are the
undefined, NaN and Infinity direct descriptor tests in both modes. All earlier
passes are preserved, with identical source/harness/mode, manifest and execution
policy fingerprints. The ten failures still require missing Date/URI globals;
the 40 unsupported cases still require eval or host reflection/enumeration.
The [full report](../tests/conformance/test262-global-values-latest.json) keeps
every observation. The new regression baseline and its CLI gate pass; this
execution-health gate permits documented ordinary failures and does not mean
the entire selection passes.

All earlier regression gates retain their outcomes: HTML **3,868 matched /
two mismatched / six unsupported**, String/JSON **536 passed / 116 unsupported**,
RegExp **250 / 40**, templates **82 / 32**, rest parameters **16 / six**, and
prototype membership **ten / ten**. The complete unchanged functions inventory
retains **501 passed / 620 unsupported / eight failed / two resource stops**
with all 48 preflights verified, no gained/lost passes and identical case/policy
fingerprints. Its two parser stops still prevent a healthy functions baseline.
No previously imported corpus, baseline or assertion helper was modified.

The **15,000-case** mutation smoke run has zero caught panics or invariant
failures: 5,000 accepted HTML, 628 accepted/4,372 rejected scripts, and
3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size is 191 nodes and
display-list size is 1,173 commands; 17 cases stop within paint limits. The new
global-property seed changes the inventory, so acceptance counts are not a
comparison against the previous run.

Release SHA-256 values:

| Binary | SHA-256 |
|---|---|
| `eris-browser` | `3f42c2e722c16c97f8d4021fcb0128d56c06aef6dbccd25f788ee2a8aea29e0c` |
| `eris-js` | `3bcf5b560c662c70568c0748c6646f85fe056710231bd339b0317b67ec3c7663` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `d0a80ce7ca3255a95b4bf31103020a26fd851f7d1b8478d2f2ca1449380e4c96` |

The final source-input digest is
`7ab85433d856ca68c585e4a02c3f52a96dcb3ae90f72414b8e8282b1f6415975`.
Session logs use `global-values-final` under `artifacts/`; independent review
records remain session scratch files. General host definitions/reflection,
window/document descriptor correctness, WindowProxy and cross-realm behavior
remain incomplete. No new native-window or performance measurement is assigned
to this increment. Full web compatibility, independently audited security and
Chromium-relative performance remain unverified.

## Replaceable Window.self

Published checkpoint `a9f4530` passed
[all GitHub CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36410428928),
including every pinned regression gate, Rust 1.88 compilation and the Vulkan probe.

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **734 Rust tests** pass with `--include-ignored`: 591 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 38 pipeline and 30 confined-worker tests. None remain
ignored. All **59 Python checks** and **57 exact pixel references** pass.

The [Window.self correction](../tests/conformance/window-self.md) installs a
real replaceable accessor and keeps bare identifiers, member accesses, saved
descriptors, deletion and lexical shadows coherent. Nine new script groups
cover brands/metadata, strictness, reentrancy, inherited lookup, declaration
rejection before inserting bindings, shared work and allocation failure.
The shared Object.defineProperty key conversion also observes ordinary
array/function overrides and key-before-descriptor evaluation order.

Direct Page and real confined-worker fixtures produce six green samples and
six blue samples after replacement in a callback. They preserve Window event
identity and runtime state across fragment navigation. Fresh Page/worker
instances restore the initial accessor. A separate three-script Page check
confirms a rejected declaration leaves no lexical, var or function bindings
behind for later scripts.

Independent release review matches **113 expected outcomes**: 95 successful
cases, eight intrinsic declaration errors, four explicitly unsupported host
reflection controls and six uncatchable resource stops. Both strict/sloppy
continuation pages confirm absent names after rejection, successful later
redeclaration and an unchanged locked accessor. Private declaration and
allocation/work/cycle tests also pass under 512 MiB, three-second CPU and
eight-second wall limits. Static source review found no concrete new defect
within the changed descriptor, declaration and callback paths. The final
Clippy predicate simplification produces byte-identical release binaries;
the reviewer verified both source snapshots and final binary identities.

The complete unchanged functions profile records **501 passed / 620 unsupported /
eight failed / two resource stops**, with all 48 preflights verified. Only
`statements/function/13.2-30-s.js:strict` newly passes: its `var self = {}` now
replaces the browser binding before the intended bound-function checks run.
The sloppy variant already passed without checking receiver identity; focused
tests now verify the assignment in both modes. The
[full comparison report](../tests/conformance/test262-functions-self.json)
preserves all 1,131 source/harness/mode identities and the exact execution policy,
with no lost passes. The two original parser stops still prevent a healthy
functions baseline. No upstream source or assertion helper was modified.

All existing regression gates retain their outcomes: HTML **3,868 matched /
two mismatched / six unsupported**, String/JSON **536 passed / 116 unsupported**,
RegExp **250 / 40**, templates **82 / 32**, rest parameters **16 / six**, and
prototype membership **ten / ten**. Their corpus bytes, policies and baselines
are unchanged. The **15,000-case** mutation smoke run has zero caught panics
or invariant failures: 5,000 accepted HTML, 634 accepted/4,366 rejected scripts,
and 3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size is 191 nodes and
display-list size is 1,173 commands; 17 cases stop within paint limits. The new
accessor-mutation seed changes the inventory, so these acceptance counts are
not a comparison against the previous run.

Release SHA-256 values:

| Binary | SHA-256 |
|---|---|
| `eris-browser` | `76f1325b20285310fcce91f6c55253214f96417a64d6657936b136976b30ad0e` |
| `eris-js` | `073e46ca2273fb55e458432d2b17c4edac6b6f6110f61a83d79b3eaeb2bc6d8b` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `cf5cd6881ba508b5db736da36c918419989af8ad87edc1644f96bf955ee62133` |

The final source-input digest is
`5e4cfb6d251433ae56cc1900fd3380dc19e8b759de5bc2a162a45ac3efddcab2`.
Session logs use `self-final` under `artifacts/`; independent reviewer records
remain session scratch files. Host own-key/reflection APIs, other initial
global-property flags, WindowProxy and cross-realm behavior remain incomplete.
No new native-window or performance measurement is assigned to this increment.
Full web compatibility, independently audited security and Chromium-relative
performance remain unverified.

## Prototype membership

Published checkpoint `9d6aa9e` passed
[all GitHub CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36407861216),
including the new prototype-membership gate and Rust 1.88 compilation.

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **720 Rust tests** pass with `--include-ignored`: 582 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 35 pipeline and 28 confined-worker tests. None remain
ignored. All **59 Python checks** and **57 exact pixel references** pass.

The [prototype-membership implementation](../tests/conformance/is-prototype-of.md)
preserves receiver-conversion order and walks internal links without invoking
author getters. Six new script groups check metadata/aliases, identity and
mutation, ordering, noncoercion, work/allocation limits and ten unchanged
upstream function variants. Direct Page and real-worker fixtures verify six
green samples and six blue samples after a callback changes a prototype link.

Independent final-release review passes **48 semantic probes** across sloppy
and strict modes, including native/array/function identity and current host
links. The private malformed-graph/resource group also passes under 512 MiB,
three-second CPU and eight-second wall limits: valid-record cycles, exact
96/97-read boundaries, retained-allocation nonmutation, shared work exhaustion
and low-heap boxing. Frozen binary and source hashes remain unchanged. These
checks do not establish unrestricted depth, Proxy or full host conformance.

The complete new [method inventory](../tests/conformance/test262-is-prototype-of.md)
records **ten passed / ten unsupported**, with all **64 preflights verified**.
Compared with the frozen pre-method adapter, six variants newly pass and none
lose a pass; every source/harness/mode and execution-policy fingerprint matches.
The old four receiver-error passes could arise from calling a missing method;
its failing positive method preflight correctly prevented a healthy baseline.
The new baseline and its CLI gate now pass while retaining all ten Proxy,
Reflect.construct and Symbol-dependent unsupported variants.

The complete unchanged functions profile improves to **500 passed / 620
unsupported / nine failed / two resource stops**. Ten previously failing
variants now pass with no lost passes and all 48 existing preflights verified.
The full [comparison report](../tests/conformance/test262-functions-prototypes.json)
retains the same 1,131 identities and policy. The two resource stops still
prevent recording a healthy functions baseline.

All other pinned outcomes are unchanged: HTML **3,868 matched / two mismatched /
six unsupported**, String/JSON **536 passed / 116 unsupported**, RegExp **250 / 40**,
templates **82 / 32**, and rest parameters **16 / six**. Their corpus bytes,
policies and baselines are unchanged. A **15,000-case** mutation smoke run has
zero caught panics or invariant failures: 5,000 accepted HTML, 617 accepted/4,383
rejected scripts, and 3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size
is 191 nodes and display-list size is 1,173 commands; 17 cases stop within paint
limits. One new prototype-mutation seed changes the inventory, so these counts
are not an acceptance-rate comparison.

Release SHA-256 values:

| Binary | SHA-256 |
|---|---|
| `eris-browser` | `975853abcae9e7e6ce9ccb68e040fcd76f09df3846b0946dd521172629597684` |
| `eris-js` | `566886403cd490547bf5475078b60f68fdc0a5d5c5f3007d36ee1179a50f6e0c` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `25a575936a81c4cc2731927d152cb1da4e55a542c8ac7f0f4d3a68fe98f594f2` |

The source-input digest is
`ebcdac762343020bcb6ba88090738526933567fdf496d0468c204706b71f9a17`.
Session logs use `prototype-final` under `artifacts/`; reviewer probes remain
separate session scratch files. No new performance claim is assigned to this
runtime increment. Full web compatibility, independently audited security and
Chromium-relative performance remain unverified.

## Identifier rest parameters

Published checkpoint `5a38ff7` passed
[all GitHub CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36406582688),
including the rest-parameter regression gate and Rust 1.88 compilation.

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **712 Rust tests** pass with `--include-ignored`: 576 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 34 pipeline and 27 confined-worker tests. None remain
ignored. All **55 Python checks** and **57 exact pixel references** pass.

The [rest implementation](../tests/conformance/rest-parameters.md) distinguishes
non-simple parameter lists from lists containing default expressions. Six new
script groups cover dense intrinsic arrays, original actual values, arguments
non-aliasing, scope, strictness, arrow parsing, metadata, call entry points and
allocation/work failure before array retention. The direct Page and real worker
fixtures each verify six green samples, then six blue samples after a callback
receives its event through a rest array.

An independent final-release audit passes 18 semantic checks, 28 early-error
checks, three unsupported-destructuring controls, 300 deterministic UTF-8
mutations and ten hostile controls. Seed `0x524553545441494c` and per-child
512 MiB, two-second CPU and three-second wall limits produce no panic, signal
or timeout. Nine hostile cases reach resource limits; 5,000 short defaults
followed by rest parse within bounds. The frozen source/binary hashes remain
unchanged throughout. These are bounded observations, not proof of security.

The complete pinned [rest directory](../tests/conformance/test262-rest-parameters.md)
improves from **two passed / 20 unsupported** to **16 passed / six unsupported**,
with the same 22 source/harness/mode fingerprints and execution policy. All
64 assertion preflights pass, including deliberate mismatches. A new healthy
regression baseline retains all unsupported variants; its CI command passes
locally with no regressions. The six remaining variants require destructuring
or classes, not a modified assertion harness.

Existing upstream outcomes are unchanged: HTML **3,868 matched / two mismatched /
six unsupported**, String/JSON **536 passed / 116 unsupported**, RegExp **250 / 40**,
templates **82 / 32**, and functions **490 passed / 620 unsupported / 19 failed /
two resource stops**. The functions profile still cannot record a healthy
baseline. Its policy and all previous corpora/baselines remain unchanged.

A **15,000-case** mutation smoke run has zero caught panics or invariant failures:
5,000 accepted HTML, 628 accepted/4,372 rejected scripts, and 3,347 accepted/1,653
rejected SVG inputs. Maximum DOM size is 191 nodes and maximum display-list size
is 1,173 commands; 17 cases stop within paint limits. Three new rest seeds change
the inventory, so these counts are not an acceptance-rate comparison.

Release SHA-256 values:

| Binary | SHA-256 |
|---|---|
| `eris-browser` | `e75c159e64bd3f64bd798991567848e4362fd96cdcb39451089f7b0a6931a98b` |
| `eris-js` | `f7303916323638bd7965ffbffeee4afd27e8f630bed6c8357a8a51fc10621b56` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `fabe4f0713477e2be903d02b74b84866dd8b46edfffd19c1f72e4f1e7b6e21fc` |

The source-input digest is
`a61c16761f309ea4e51f42b44ef2bd3471a04d1b5c8843a4d8ec15b4fc0d191c`.
Session logs use the `rest-final` suffix under `artifacts/`. This increment
changes parser/runtime behavior, not native presentation. No new performance
claim is assigned. The unchanged WPT harness now passes the former rest syntax
blocker and stops at destructuring; it still cannot execute. Full web
compatibility, independently audited security and Chromium-relative performance
remain unverified.

## Length-percentage calculations and default parameters

Published checkpoint `ed7dcb8` passed
[all GitHub CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36405141684),
including all-target compilation with Rust 1.88.

Recorded September 28, 2026 UTC. The recorded integrated run passes formatting,
strict all-target Clippy, release compilation and **704 Rust tests** with
`--include-ignored`: 570 library, 39 browser/editor, four HTML adapter, six
JavaScript adapter, five stress, four CLI, 17 network, 33 pipeline and 26
real-worker tests. None remain ignored. All **51 Python checks** and **57 exact
pixel references** pass. The five new references use independently specified
SVG rectangles for containing-block bases, Grid tracks/gaps, definite and
indefinite flex bases, range/token rejection and cyclic Grid/Flex gaps.

The [calculation scope](../tests/conformance/calc-lengths.md) records the shared
CSS/CSSOM additive grammar, retained percentage dependency, context-sensitive
resolution and explicit limits. Nine new geometry groups bring layout coverage
to **120 passing tests**. Independent release probes pass **34 calculation
grammar cases** and **24 default-parameter cases**. A separate cross-layer audit
passes **100 checks** across authored declarations, variable substitution,
unrelated inline mutations and CSSOM round trips, plus **39 invalid-value and
priority nonmutation checks**. Its deterministic seed `0x43414c4320260928`
generates 2,000 mutated values and 39 hostile cases: 2,007 evaluations succeed,
32 reach resource limits without changing the original style attribute, and
none produce a caught panic. External limits are 30 seconds wall time,
10 seconds CPU and 512 MiB address space. These malformed-input probes establish
bounded observed behavior, not acceptance conformance.

An independent default-parameter audit adds 12 semantic checks, 14 early-error
controls, three valid-but-unsupported rest controls, 1,200 deterministic UTF-8
mutations and 12 hostile inputs, with no panic, signal or timeout. Each child
has external 512 MiB, two-second CPU and three-second wall limits. Eleven hostile
inputs reach resource limits; 5,000 short defaults parse within the existing
budgets. The audit found that extra ellipses were classified as unsupported
instead of `SyntaxError`. A narrow correction and four regression cases fix
that classification; valid rest remains unsupported. After the correction,
all 704 Rust tests, formatting, strict Clippy, release compilation and all four
Test262 profiles pass their applicable checks with unchanged outcome counts.

An independent layout audit passes **38 bounded probes** for counter
monotonicity, state restoration and finite output. A 24-level cyclic Grid case
terminates at its shared Grid-work limit after 16,145 visits. Nine of ten
ordinary fixed-pixel before/after controls remain exact; the remaining change
corrects a control's paint height to its resolved content box instead of its
old overflowing intrinsic height. Five additional storage/work probes complete
under 512 MiB. Repeated declarations over 99,000 siblings share three nonempty
Grid track arrays totaling **3,096 bytes**. Distinct plain and calculated track
lists both stop retaining new lists at **4,193,304 bytes**, with 2,730 populated
column lists; later values take the documented bounded fallback. The larger
calculation representation is included in retained-byte accounting.

The [unchanged function inventory](../tests/conformance/test262-functions.md)
now reports **490 passed / 620 unsupported / 19 failed / two resource stops**
across 1,131 required variants, with **140 new passes and no lost passes**
against the same sources, harnesses, modes and execution policy. All **48
preflights** pass, including deliberately incorrect assertions. The remaining
resource outcomes prevent recording a healthy baseline; failures and
unsupported variants remain in the denominator. The
[default-parameter implementation record](../tests/conformance/default-parameters.md)
separates supported identifier defaults from remaining rest, destructuring,
async and host gaps. The unchanged WPT JavaScript harness advances past its
first default-parameter blocker but still cannot execute.

Existing pinned upstream outcomes remain HTML **3,868 matched / two mismatched /
six unsupported**, String/JSON Test262 **536 passed / 116 unsupported**, RegExp
**250 / 40**, and templates **82 / 32**. Existing policies, corpus bytes and
baselines are unchanged. A separate **15,000-case** mutation run has zero caught
panics or invariant failures: 5,000 accepted HTML inputs, 637 accepted/4,363
rejected scripts, and 3,347 accepted/1,653 rejected SVG inputs. Maximum DOM size
is 191 nodes, maximum display-list size 1,173 commands, and 17 cases stop within
paint limits. New calculation/default-parameter seeds change this inventory;
these counts are not an acceptance-rate comparison or coverage-guided fuzzing.

Direct Page and confined-worker fixtures pass calculation resize/mutation and
default-parameter interactions. A bounded native software-window check confirms
the default-parameter fixture's green framebuffer state, exits successfully,
and leaves no tracked UI, renderer or broker process alive. This checks the
owned CPU framebuffer, not compositor pixels or a Vulkan browser backend.
All nine existing benchmark views are pixel-identical to frozen `3491721`;
the [warm measurement record](PERFORMANCE.md#calculation-and-default-parameter-checkpoint)
reports both builds without claiming a speed change.

Session records use `calc-params-final`, `calc-params-corrected`, `calc-storage-final` and
`default-parameters-native` names under `artifacts/`; focused peer-review
reproducers remain session scratch artifacts. Native, stress and adversarial
audits preceded the narrow extra-ellipsis correction; their reports retain the
original binary/source hashes. Final release SHA-256 values are:

| Binary | SHA-256 |
|---|---|
| `eris-browser` | `371a91b712ea2d112948ae905bf381faeb3ec2908ef363b024c5c536dc61e7eb` |
| `eris-js` | `77003eefd7a91dccb2b418e0d59bc234091edbfb1a7bb6a217de63cd1ece8b69` |
| `eris-dom` | `b775436e90fd2c6c8ffd97f0fabffebde7129b6c206f0efe325e4276b22a1bfb` |
| `eris-stress` | `8b5babf065524debe90ecd7714bb0a89f74dd419b9cc2c308af01654e7223b48` |

The final source-input digest is
`e6d4d857c922db8942abaec7cf230454b3487313f1121dbc6e7babbe0385bcd6`.
Full CSS/JavaScript conformance, independently
audited security and the requested Chromium-relative performance threshold
remain unverified.

## Expanded upstream function inventory

Recorded September 28, 2026 UTC. The new complete four-directory
[function selection](../tests/conformance/test262-functions.md) imports 663
unchanged Test262 files at the existing pinned revision. A frozen pre-change
adapter records **350 passed / 718 unsupported / 61 failed / two resource stops**
across 1,131 required variants. All 32 existing assertion preflights pass, but
the resource outcomes prevent recording a healthy baseline. No cases are
removed or limits raised to change that result. The report identifies every
source, mode, expected negative, observation and source/harness fingerprint.

Metadata handling now accepts common top-level indentation without rewriting
upstream bytes. Inventory, negative-mode and feature-policy checks bring the
Python suite to **50 passing checks**. The original three Test262 profile
policies and baselines remain unchanged. The preceding CSSOM compatibility
correction `cfee40f` passed
[all GitHub CI jobs](https://github.com/Eriskii/ErisBrowser/actions/runs/36402059569),
including actual all-target compilation with Rust 1.88.

## Inline CSSOM and computed property references

Recorded September 28, 2026 UTC. Formatting, strict all-target Clippy, release
compilation and **676 Rust tests** pass with `--include-ignored`: 546 library,
39 browser/editor, four HTML adapter, six JavaScript adapter, five stress,
four CLI, 17 network, 31 pipeline and 24 real-worker tests. None remain ignored.
The existing **47 Python checks** and all **52 exact pixel references** pass.

Inline declaration bindings now use bounded token-aware storage, ordered author
conversions and a single final DOM write. Eight helper groups and seven script
groups exercise priorities, aliases, shorthands, custom names, reentrancy,
serialization boundaries and resource nonmutation. Three further script groups
check deferred computed-key conversion and cached keys for compound operations.
The [CSSOM scope record](../tests/conformance/cssom-inline.md) documents the
supported operations and remaining storage, grammar and reflection limitations.

Independent final-binary probes pass **42 binding, key-order and malformed-value
cases**, plus **47 preservation/precedence cases**. A separate deterministic
robustness audit uses seed `0x4353534f4d202609`: **2,120 cases** produce 2,060
successful evaluations and 60 expected resource failures, all failures preserving
the original style attribute byte for byte, with no other errors or caught
panics. Another **90 name/value injection combinations** preserve sentinel
declarations across serialization and reparsing. These scratch audits used
external 30-second wall, 10-second CPU and 512 MiB address-space limits. They are
focused safety checks, not upstream WPT conformance or performance measurements.

Pinned upstream outcomes remain HTML **3,868 matched / two mismatched / six
unsupported**, Test262 **536 passed / 116 unsupported**, RegExp **250 / 40**, and
templates **82 / 32**, with passing preflights and no regressions or improvements.
Corpus bytes, runner policies and baselines are unchanged. A separate
**15,000-case** mutation run has zero caught panics or invariant failures:
5,000 accepted HTML, 642 accepted/4,358 rejected scripts, and 3,347 accepted/1,653
rejected SVG inputs. Maximum DOM size is 229 nodes, maximum display-list size
349 commands, and 21 cases stop within paint limits. The added CSSOM seed changes
the inventory; these counts are not an acceptance-rate comparison.

The inline-style fixture passes direct Page and confined-worker tests for six
green initial samples and six blue samples after a click. A bounded native
software-window run exits successfully and reaps the UI, renderer and broker
processes. Inspection of its own framebuffer confirms the initial green samples
and browser chrome; no compositor capture or Vulkan backend is involved.
Nine existing 1180×880 views remain pixel-identical to frozen commit `738f492`.
This comparison measures decoded pixels only, not speed.

Release SHA-256 values for checkpoint `3491721`:

| Binary | SHA-256 |
|---|---|
| `eris-browser` | `7497619b9c11606fd6415a387860d9ee395e8cfbf56da5def70e53136f355783` |
| `eris-js` | `15902a07b1ef9c866942c9418264a3fee8e5d045ef64ccef8b60a83794f75edc` |
| `eris-dom` | `ecf9052c43c5a3a968596c31c43a6baca92a4bdd3b309d127ebe23058d64bafc` |
| `eris-stress` | `ec0cd72c861a2e23d1cd3e5a84e41b9923c376714edef788794d245ba3a28c31` |

The source-input digest is
`5fb0934ebb83e25d634853caac2d6ef133aaa1458313a4bcfe182acc063dcb14`,
using the benchmark tool's documented inventory. Session logs and reports use
the `cssom-final` suffix or `cssom-native` / `cssom-pixels` directories under
`artifacts/`. Local checks used Rust 1.95. The new engine CI job
[passed all-target checking with Rust 1.88](https://github.com/Eriskii/ErisBrowser/actions/runs/36401843150/job/108861294379).
That run's newer stable Clippy rejected one constant-size `chunks_exact` call;
a subsequent equivalent `as_chunks` rewrite addresses the lint. The hashes
above identify the locally validated pre-rewrite build, not a later binary.
The previous software-presenter checkpoint passed
[GitHub CI](https://github.com/Eriskii/ErisBrowser/actions/runs/36399652737).
Full web compatibility, independently audited security and the requested
Chromium-relative performance threshold remain unverified.

## Software presentation boundary and native Vulkan investigation

Recorded September 28, 2026 UTC. The software presenter extraction passes all
**39 browser/editor tests**, including four new frame-boundary checks, strict
binary Clippy and formatting. The boundary borrows completed pixels, rejects
invalid dimensions and mismatched source/destination lengths, and preserves
pixel words without conversion. No GPU dependency or browser Vulkan backend
was added. The preceding custom-property checkpoint also completed
[GitHub CI](https://github.com/Eriskii/ErisBrowser/actions/runs/36398730489).

A separately reviewed native Vulkan prototype matched **3,316,800 bytes** in
five acquired-surface readbacks on the NVIDIA adapter, including changed frames
and resize. Full compositor comparisons remain failed, with mismatch counts
172,800 / 252 / 252 / 252; later inset agreement is diagnostic only. Normal
exit and cleanup were verified; forced hang recovery was not exercised.
[Sanitized evidence](evidence/vulkan-native-surface.json) retains source,
binary, log and capture hashes, independently checked against the frozen
session artifacts. The [milestone record](vulkan-rendering.md) explains the
capabilities, provenance and limitations. This is neither browser GPU
integration nor a performance result.

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
