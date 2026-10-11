# Throw values and callback-free diagnostics

After evaluating a JavaScript `throw` operand, Eris retains that exact value.
Diagnostic classification does not invoke its `constructor` getter or a
constructor's `name` getter. This prevents diagnostic work from substituting
another exception, mutating the document or consuming the script allowance in
an accessor that the original program never called.

Operand evaluation still runs normally. For example, `throw source.value`
invokes a getter for `source.value` if one exists; it throws the result of that
read. The suppression applies only to later inspection for a diagnostic label.

The metadata walk reads stored data descriptors through supported object bags.
An accessor terminates that lookup, including a setter-only descriptor; the
walk never skips it to inspect a prototype's value. Unsupported host metadata
falls back to a generic label. A legacy native callable with no stored metadata
bag also uses that fallback, without the ordinary property path's cached
Function-prototype traversal. This diagnostic helper is not a replacement for
ordinary JavaScript property access or reflection.

A copied data name remains an optional display label. Canonical engine-error
identity separately requires the private error brand and the matching stored
intrinsic constructor. An accessor on the constructor's optional `name` does
not erase an already established identity. A forged display name, prototype or
constructor property cannot create that identity.

The constructor is inspected once and shared by the two classifications.
Reached table lookups, prototype steps, static key materialization and copied
name storage are prepaid under the existing work and cumulative heap limits.
The original value's diagnostic formatter and the microtask reporter retain
their existing tariffs. Resource refusal remains a terminal host policy; this
change introduces no extra work allowance or Promise implementation.

The behavior follows the ECMAScript [throw evaluation](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-throw-statement-runtime-semantics-evaluation)
and [catch evaluation](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-runtime-semantics-catchclauseevaluation)
algorithms. The choice of display labels is an implementation diagnostic policy.

## Validation

The parent checkpoint compiled all nine new public groups: the value-payload
control passed and eight groups exposed the diagnostic getter defect. The new
private accounting group has no parent API and was not run there. The repaired
checkpoint passes all ten new groups, the complete Rust 1.88 suites (3,103 native
and 2,799 default tests, none ignored), strict Clippy on Rust 1.88 and 1.98, and
format checking. All 102 rendering reference pairs pass; their 204 PNGs are
byte-identical to the published microtask checkpoint.

The [evidence summary](evidence/throw-completion/summary.json) binds the 767
source inputs, baseline and final logs, frozen tests and resource ledger, source
reviews, browser build and reference output. No old test expectations changed.
The root-authored Page and confined-worker fixtures expect `CFMN` after the
script checkpoint and `CFMNABQD` after clicking. Their poisoned metadata
getters would move a blue rectangle and loop forever; the expected frames
keep that rectangle unchanged while later callbacks move the red rectangle.
Disabled-script controls run before the enabled baseline checks. These are
local authored witnesses and do not establish general web conformance,
production security, GPU performance or parity with Chromium.
