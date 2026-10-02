# DOM defining-interface method identity

Document, Element and DocumentFragment now have separate `querySelector` and
`querySelectorAll` function identities and property bags. Element and
DocumentFragment also have separate `append` functions. Repeated access through
the same interface shares its function properties; editing one interface's
function does not alter another. Inherited Node methods retain shared identity.

Saved methods check the actual receiver's defining interface before required
arguments or author conversion. Valid calls keep the existing query/append
algorithms and conversion order. Text and Comment nodes no longer expose these
ParentNode methods. Document.append, real interface prototypes and ordinary
method replacement on individual host objects remain incomplete.

## Independent outcomes

The unchanged [local oracle](dom-method-identity-local-oracle.json) contains
29 sources in sloppy and strict mode: 24 behavior sources, three ordinary
prerequisite expectations and two terminal resource policies. Its
[readable companion](dom-method-identity-local.js) preserves the exact sources.
All 32 paired feature controls and 12 common controls verify.

| Outcome | Original BEFORE | Corrected runtime |
| --- | ---: | ---: |
| Passed modes | 30 | 48 |
| Failed modes | 24 | 6 |
| Expected terminal resource modes | 4 | 4 |
| Verified expectations | 34/58 | 52/58 |

The six failures are `document-append-standard-prerequisite`,
`real-interface-prototypes-standard-prerequisite` and
`host-method-replacement-standard-prerequisite`, each in both modes. They remain
ordinary success expectations. No case was excluded or rewritten after observing
its result. Resource expectations describe the unchanged engine policy, not
ECMAScript exceptions.

The four original Object.is Document/Element identity failures also pass now.
Across all 11 local suites, 1,756 modes and 588 controls, the replay gains 22
passes, loses none and preserves every other complete case/control record.
All 48 explicit terminal-policy expectations match. The full 44-profile formal
replay preserves all 19,727 case modes and 4,564 controls exactly, and all 36
existing baseline gates pass without changed baseline bytes.

The [runtime summary](dom-method-identity-runtime.json) and
[runtime evidence](../../docs/evidence/dom-method-identity-runtime/index.json)
bind the source, binaries, attempts and complete reports. The original
[BEFORE report](dom-method-identity-local-before.json) and
[preparation evidence](../../docs/evidence/dom-method-identity-preparation/index.json)
remain retained. BEFORE used published Object.is commit
`8e2a2740ef1d63c9b3d0d2626400f8b125e4575c`.

## Accounting and validation

The metadata installer, getter wrappers, row lookup and temporary native
call-name copies have scoped work/storage admission. A refused name copy still
unwinds its call-stack ownership. Callback effects that precede a later error
remain visible. These checks do not establish complete DOM atomicity, total
process-memory bounds or production security.

The first frozen candidate passed 18 focused groups, then failed the unchanged
exhaustive UTF-16 JSON test on both Rust versions. Its eager metadata setup left
66,390 bootstrap work units; 1,148 library tests passed and one failed in each
run. Those attempts remain in the evidence.

The correction installs the eight rows into an empty registry, builds final
descriptors directly and shares immutable strings between distinct mutable
bags. An explicitly prepaid 512-slot reservation before initialization removes
later bootstrap object-buffer growth. Ordinary object/property charges and
numeric limits remain unchanged. The measured bootstrap retains 70,334 work
units, charges 734,594 bytes, and uses 351 of the reserved object slots. These
are logical budget measurements, not allocator or CPU performance measurements.

The corrected candidate passes 21 focused groups and the original UTF-16 test.
Rust 1.88 and 1.98 each pass 1,361 default and 1,478 native-feature tests,
including ignored confinement tests, both new browser callback fixtures and
existing maximum-string regressions. Six strict Clippy configurations and
formatting pass. The direct-page and confined-worker fixtures check six literal
pixels and result text on initial load and click.

One parallel Rust 1.88 attempt timed out in an unchanged timezone-helper stress
test. That attempt is retained; the isolated test and complete matrix then
passed with identical source and deadlines. Contention is an inference, not a
proven cause. No external conformance replay ran for the rejected first candidate.

Before execution, source review added five property-helper `restore` options
and positive browser-fixture selector guards; original drafts remain retained.
The release verifier also corrected path sorting to match the frozen digest.
None of these corrections changed an observed case's expectation or quota.

Normative references: [Web IDL operations](https://webidl.spec.whatwg.org/#es-operations)
and [DOM ParentNode](https://dom.spec.whatwg.org/#interface-parentnode).
