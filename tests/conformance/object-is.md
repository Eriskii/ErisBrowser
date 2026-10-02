# Object.is

The custom interpreter now implements `Object.is` for its existing value kinds.
NaN compares equal to NaN, positive and negative zero differ, and values are
never coerced or inspected for conversion hooks. Strings compare exact UTF-16
code units; symbols and ordinary objects compare by identity. Missing arguments
are undefined, and extra argument expressions still execute in source order.
The intrinsic has standard name, length and property flags and is not a
constructor.

The complete pinned upstream profile now passes **42/42 cases**, with all
80 controls healthy. The complete historical replay covers 44 profiles,
19,727 case modes and 4,564 controls: 42 gains, zero regressions, zero other
case changes and 48 newly healthy controls. Every older profile's complete
case/control records is unchanged; all 35 prior baseline gates pass.
The new Object.is gate raises the total to 36 without changing old baselines.

| Independent local outcomes | Modes |
| --- | ---: |
| Passed | 64 |
| Failed | 8 |
| Foreign-realm host-hook exclusions | 2 |
| Expected terminal resource outcomes | 4 |

The local suite verifies 68/78 expectations and all 60 controls. Four failures
are the two Document-versus-Element query-method identity cases in both modes.
The current host representation loses the defining-interface distinction;
these ordinary success-expecting cases remain failures. The other four failures
are missing Proxy and BigInt prerequisites. Two foreign-realm cases remain
static host-hook exclusions. No denominator, expected result or resource limit
was changed to hide these gaps.

The [runtime summary](object-is-runtime.json) and
[retained evidence](../../docs/evidence/object-is-runtime/index.json) bind the
source, binaries, all results, reviews and validation attempts. The earlier
[upstream BEFORE](test262-object-is-before.json) had 42 failures;
the [local BEFORE](object-is-local-before.json) had 76 failures and two
exclusions. Only their common controls were healthy. The
[preparation evidence](../../docs/evidence/object-is-preparation/index.json),
[local oracle](object-is-local-oracle.json) and
[readable source](object-is-local.js) remain unchanged.

## Scope and accounting

The method compares borrowed operands without allocating, reading properties
or traversing object contents. Equal-length strings prepay their UTF-16
comparison, even when their storage is shared. Unequal lengths and mismatched
value kinds avoid payload scans. Native function wrappers use the existing
represented method identity with bounded, iterative key comparisons. This does
not repair the separate defining-interface host identity gap.

A dedicated dispatch precedes the generic native payload scan, so ignored
receivers and extra values add no comparison charge. Argument evaluation,
ordinary calls and `call`/`apply` forwarding retain their own costs; a complete
JavaScript expression is not claimed to be allocation-free.

Installation prepays the actual registry/property node types, bounded searches,
entry moves, fresh four-slot order allocation and any full owner-order growth
buffer. Existing ordinary allocation charges remain. The measured installation
uses 507 logical work units and 11,560 charged bytes with spare owner capacity;
a separately forced growth case uses 578 units and 12,584 bytes. These are
logical budget charges, not physical allocator or processor measurements.
The resulting bootstrap has 71,429 work units remaining. Limits are unchanged.

Twenty private test groups cover numeric and UTF-16 values, identity, exact and
one-short work/heap boundaries, ignored payloads, iterative native keys,
descriptors, installation failure and terminal cleanup. Policy tests first
prove a successful long comparison before their unchanged repeated loops.
The direct page and confined-worker fixtures verify six literal pixel samples,
result text and retained intrinsic identity on both initial load and click.

Rust 1.88 and 1.98 each pass 1,338 default tests and 1,455 native-feature tests,
including the confined-worker tests and the existing maximum-string regressions.
All six strict Clippy configurations and formatting pass. The first frozen
candidate failed compilation because two child-module methods needed parent
visibility; candidate two changes only those methods to `pub(super)`.
The original failed candidate, formatter setup failure and corrections remain
in the evidence. No engine ran for candidate one.

## Pinned source inventory

The upstream revision is `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd` and
`built-ins/Object/is` subtree is `04a0f3e25947dbc1100ba4b4138765366d9704b4`.
All 21 original sources run in both modes, with no exclusions. Root-linked Git
proofs authenticate the sources, four helpers and legal files. The new profile
retains 32 common controls and adds 48 guarded feature controls; its admission
policy does not alter any previous profile. The local inventory has 39 sources
in both modes plus 48 feature and 12 unchanged common control modes.

Normative references: [Object.is](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.is),
[SameValue](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-samevalue),
[Web IDL operations](https://webidl.spec.whatwg.org/#es-operations).
