# DOM defining-interface method identity preparation

This checkpoint records independent tests before changing the existing DOM
bindings. Document, Element and DocumentFragment currently share selector
function identities and property bags. Their selector calls also accept the
other interfaces as receivers. Element and DocumentFragment share the same
problem for `append`.

The scoped change will give the six `querySelector`/`querySelectorAll` functions
and two existing `append` functions separate defining-interface identities,
independent function properties and receiver checks before argument conversion.
Methods inherited from Node must remain shared. Document.append, real interface
prototypes and ordinary host method replacement remain separate prerequisites.

The [local oracle](dom-method-identity-local-oracle.json) contains 29 sources,
each in sloppy and strict mode: 24 ordinary behavior sources, three ordinary
prerequisite expectations and two terminal resource policies. Its
[readable companion](dom-method-identity-local.js) preserves the exact sources.
There are 32 paired feature controls and 12 unchanged common controls.

The original [BEFORE report](dom-method-identity-local-before.json) records:

| Outcome | Modes |
| --- | ---: |
| Passed | 30 |
| Failed | 24 |
| Expected terminal resource outcomes | 4 |

All 44 controls verify. The run verifies 34/58 fixture expectations, with no
exclusions. Eighteen failing modes cover the identity, property isolation,
receiver and member-exposure issues in this increment. Six other failures
remain ordinary success expectations for Document.append, real interface
prototypes and host replacement. Resource expectations describe the engine's
unchanged execution policy, rather than ECMAScript exceptions.

The run uses the unchanged published Object.is binary
`804bc8dd688182641be37be577feade9fe043a0869000cd272977707c320b4df`
from commit `8e2a2740ef1d63c9b3d0d2626400f8b125e4575c`. Its process completed,
inputs were unchanged before and after, and all 102 case/control records were
retained. No candidate implementation has been measured in this checkpoint.
The four existing Object.is Document-versus-Element identity modes also remain
unchanged for later comparison.
The [preparation evidence](../../docs/evidence/dom-method-identity-preparation/index.json)
retains the runner, original observations, source reviews and corrections.

Source review corrected five property-helper calls to restore descriptors
before subsequent mutation or explicit deletion. The original draft remains
retained. The browser fixture also gained positive selector prerequisites and
calls on distinct same-interface receivers before execution. No observation
was used to change an expected result or resource limit.

The planned installer prepays the new metadata, registry and reached object
arena allocations. Method acquisition, row lookup and the temporary native
name copy have scoped work/storage charges. Failures in the pre-copy check
must still release call-stack ownership. These are implementation requirements,
not completed security or conformance claims.

Normative references: [Web IDL operations](https://webidl.spec.whatwg.org/#es-operations)
and [DOM ParentNode](https://dom.spec.whatwg.org/#interface-parentnode).
