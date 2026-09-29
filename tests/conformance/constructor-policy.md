# Symbol constructor targets and broader Reflect coverage

`Symbol` has a constructor internal method, even though invoking that method
throws TypeError. Eris now accepts Symbol (and bound Symbol functions) as an
alternate `newTarget`. For example, `Reflect.construct(Object, [], Symbol)`
creates an ordinary object inheriting from `Symbol.prototype`; it does not give
that object a Symbol's internal value slot.

Constructing Symbol itself still throws. Reflect validates targets first, then
reads the array-like arguments, then reaches Symbol's throwing constructor.
Argument getter failures propagate before that TypeError. Description conversion
and `newTarget.prototype` lookup do not occur inside Symbol construction.
The [ECMAScript Symbol algorithm](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-symbol-description)
defines this distinction. Existing bound forwarding, allocation and callback
limits remain in force.

## Reviewed policy change

All 24 pinned profiles now admit the four implemented feature flags `Reflect`,
`Reflect.apply`, `Reflect.construct`, and `new.target`. This admits **88 modes**
across 12 older profiles. **86 already passed on the preserved prior binary**;
those are expanded coverage, not new engine fixes. The Symbol implementation
fix turns the other **two modes** from failed to passed under the same expanded
policy. All 88 now pass.

| Profile | Newly admitted modes | Passed before fix | Passed after fix |
| --- | ---: | ---: | ---: |
| string-concat | 2 | 2 | 2 |
| symbols | 14 | 12 | 14 |
| string-json | 22 | 22 | 22 |
| regexp | 10 | 10 | 10 |
| functions | 4 | 4 | 4 |
| is-prototype-of | 4 | 4 | 4 |
| array-sort | 2 | 2 | 2 |
| array-reduce | 4 | 4 | 4 |
| number-statics | 10 | 10 | 10 |
| numeric-conversion | 4 | 4 | 4 |
| numeric-parsing | 4 | 4 | 4 |
| uri | 8 | 8 | 8 |

Every source, helper, manifest and mode fingerprint is preserved. Across all
**7,231 modes**, no previous pass is lost. Two Number realm tests remain excluded:
their diagnostic drops the now-supported Reflect prerequisite and still names
cross-realm support. All other observations outside the admitted modes are
unchanged. Existing failures and resource outcomes remain recorded.

**18 regression baselines** receive the reviewed policy hashes and results. The
two constructor profiles already admitted these flags, so their baseline files
remain unchanged. URI, numeric-parsing, array-sort and identifiers retain their
resource-observation status; this change does not promote them to healthy gates.
Historical contract tests retain their original hashes by projecting out exactly
the four reviewed feature additions. A separate inventory test verifies those
four flags and the complete 88-mode admission delta.

## Evidence

The [machine-readable record](constructor-policy.json) preserves the old policy,
the new policy on the old binary, and the new policy on the fixed binary. It
includes the complete admission delta and the two actual implementation gains.
All **1,960 existing assertion controls** are unchanged. **48 added controls**
verify alternate Symbol targets, bound targets, getter ordering and abrupt
completion in both modes, with deliberate mismatches required to fail with
Test262Error. All **2,008 controls** verify.

The [five frozen local cases](symbol-constructor-targets.js) fail in all ten modes
before the fix and pass afterward without source changes. Two Rust regression
checks were recorded failing before the implementation change and passing after
it. They also cover oversized argument lists, recursive getters and cleanup of
call counters and continuations after uncatchable resource errors.

On Rust **1.88 / 1.95**, strict all-target Clippy and all tests pass:
**1,005 default / 1,016 Vulkan-feature Rust tests**, none ignored. Both release
builds preserve **57 headless CPU pixel references**. **162 Python tests** and
**45,000 deterministic mutation cases** pass. Upstream HTML results are exactly
preserved: 3,868 matched, two mismatched and six unsupported.

Full ECMAScript/web compatibility, independently reviewed security and the
requested Chromium performance comparison remain unproven. Proxy, other realms,
classes/super, dynamic Function construction, asynchronous completion, most Reflect
methods, and alternate Web IDL targets remain incomplete. Earlier records,
including [constructor implementation evidence](construction.json), retain the
policies and observations of their own checkpoints.
