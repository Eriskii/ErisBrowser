# Object.is preparation

This checkpoint imports the complete pinned Test262 `built-ins/Object/is`
subtree and independent local cases before implementing the method. It does
not add runtime support or a passing baseline.

The original [upstream BEFORE](test262-object-is-before.json) records 42 failures.
All 32 common controls verify; none of the 48 feature controls verifies.
The [local BEFORE](object-is-local-before.json) records 76 failures and two
foreign-realm host-hook exclusions, with 12 common controls verified and none
of the 48 feature controls verified. No case passes or reaches a resource
limit. Both runs use the unchanged published DataView binary
`b61e487bf1ea2c0ff70b471656b4abd15aa085b2570406d8c76ab8ab14fcdf18`.

The [local oracle](object-is-local-oracle.json) retains exact authored source
bytes and expectations; its [readable companion](object-is-local.js) is for
inspection. The [preparation evidence](../../docs/evidence/object-is-preparation/index.json)
includes both original runs, frozen contracts, proof data, reviews and earlier
preparation errors. Nine new Python test groups and all 251 Test262 tooling
groups pass. There are no new runtime results in this checkpoint.

The upstream revision is `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`;
the subtree is `04a0f3e25947dbc1100ba4b4138765366d9704b4`.
All 21 original sources run in both modes, for 42 cases, with no exclusions.
Root-linked Git proofs authenticate the sources, four helpers and legal files.
The profile retains 32 common controls and adds 48 guarded feature controls.
The new policy is local to this profile; all 43 previous profile contracts and
35 baseline files remain unchanged.

The local oracle has 39 sources in both modes, for 78 cases, plus 48 feature
and 12 unchanged common control modes. It covers numbers, exact UTF-16 strings,
symbols, object/function identities, ignored receivers, argument evaluation,
descriptors, construction rejection and terminal work limits. Resource
expectations describe the engine's execution policy, not ECMAScript exceptions.
BigInt, Proxy and foreign-realm cases retain ordinary success expectations;
missing prerequisites remain visible.

Two additional ordinary cases require Document and Element `querySelector`
and `querySelectorAll` operations to have distinct identities. The current
host representation loses that distinction. The planned scalar comparison
cannot recover it; a separate DOM identity change is required. These cases
are neither excluded nor relabeled as expected failures.

The planned method uses SameValue: NaN equals NaN, positive and negative zero
differ, and values are never coerced or inspected for conversion hooks.
Equal-length string comparisons prepay their UTF-16 traversal. Installation
prepays the new intrinsic's registry, property and order-vector work while
retaining existing ordinary allocation charges. Runtime limits and historical
maximum-string tests remain unchanged.

The preparation retains the original drafts, static review findings and
corrections. Large-string setup uses bounded literals, construction-error
tests first prove successful construction, and native-alias controls first
prove callable prerequisites. The corrected contract uses the existing
runner's `assert.js`, then `sta.js` helper order; the original scratch
attribution of the opposite order is retained and explicitly corrected.

Normative references: [Object.is](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.is),
[SameValue](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-samevalue),
[Web IDL operations](https://webidl.spec.whatwg.org/#es-operations).
